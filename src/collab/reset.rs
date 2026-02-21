//! Regenerate template files in shared collaborator repos.

use anyhow::Result;
use std::path::Path;
use std::process::Command;

use crate::accounts::load_owner;
use crate::config::collaborator::{collab_dir, load_collaborators, Collaborator};
use crate::resolve;

use super::templates::{generate_agents_md, generate_readme_md};

fn run_git(args: &[&str]) -> (String, String, i32) {
    let output = Command::new(args[0])
        .args(&args[1..])
        .output()
        .unwrap_or_else(|_| panic!("Failed to run: {}", args.join(" ")));
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let code = output.status.code().unwrap_or(-1);
    (stdout, stderr, code)
}

/// Regenerate template files for one collaborator.
fn regenerate(_name: &str, display_name: &str, owner_name: &str, sub_path: &Path) -> Result<()> {
    // AGENTS.md
    std::fs::write(
        sub_path.join("AGENTS.md"),
        generate_agents_md(display_name, owner_name),
    )?;
    println!("  Updated AGENTS.md");

    // CLAUDE.md symlink
    let claude_md = sub_path.join("CLAUDE.md");
    if claude_md.exists() || claude_md.is_symlink() {
        std::fs::remove_file(&claude_md)?;
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink("AGENTS.md", &claude_md)?;
    println!("  Updated CLAUDE.md -> AGENTS.md");

    // README.md
    std::fs::write(
        sub_path.join("README.md"),
        generate_readme_md(display_name, owner_name),
    )?;
    println!("  Updated README.md");

    // .gitignore
    std::fs::write(
        sub_path.join(".gitignore"),
        "AGENTS.local.md\nCLAUDE.local.md\n__pycache__/\n",
    )?;
    println!("  Updated .gitignore");

    // voice.md
    let voice_file = resolve::voice_md();
    if voice_file.exists() {
        std::fs::copy(&voice_file, sub_path.join("voice.md"))?;
        println!("  Updated voice.md");
    }

    Ok(())
}

/// Pull, regenerate templates, commit, and push for one collaborator.
fn reset_one(
    name: &str,
    collab: &Collaborator,
    owner_name: &str,
    do_sync: bool,
) -> Result<()> {
    let sub_path = collab_dir(collab);
    if !sub_path.exists() {
        println!(
            "  {}: submodule not found at {} -- skipping",
            name,
            sub_path.display()
        );
        return Ok(());
    }

    println!("Resetting {}...", name);
    let sp = sub_path.to_string_lossy().to_string();

    // 1. Pull latest
    if do_sync {
        let (stdout, _, code) = run_git(&["git", "-C", &sp, "pull", "--rebase"]);
        if code == 0 {
            if !stdout.contains("Already up to date") {
                println!("  Pulled changes");
            }
        } else {
            println!("  Pull failed -- continuing with reset");
        }
    }

    // 2. Regenerate template files
    let display_name = if collab.name.is_empty() {
        name
    } else {
        &collab.name
    };
    regenerate(name, display_name, owner_name, &sub_path)?;

    if !do_sync {
        return Ok(());
    }

    // 3. Stage, commit, push
    run_git(&["git", "-C", &sp, "add", "-A"]);

    let (status_out, _, _) = run_git(&["git", "-C", &sp, "status", "--porcelain"]);
    if !status_out.trim().is_empty() {
        run_git(&[
            "git",
            "-C",
            &sp,
            "commit",
            "-m",
            "Reset template files to current version",
        ]);
        let (_, stderr, code) = run_git(&["git", "-C", &sp, "push"]);
        if code == 0 {
            println!("  Pushed changes");
        } else {
            println!("  Push failed: {}", stderr.trim());
        }
    } else {
        println!("  Templates already up to date");
    }

    // 4. Update submodule ref in parent
    run_git(&["git", "add", &sp]);

    Ok(())
}

/// corrkit for reset [NAME] [--no-sync]
pub fn run(name: Option<&str>, no_sync: bool) -> Result<()> {
    let collabs = load_collaborators(None)?;
    if collabs.is_empty() {
        println!("No collaborators configured in collaborators.toml");
        return Ok(());
    }

    let names: Vec<String> = if let Some(n) = name {
        if !collabs.contains_key(n) {
            anyhow::bail!("Unknown collaborator: {}", n);
        }
        vec![n.to_string()]
    } else {
        collabs.keys().cloned().collect()
    };

    let owner = load_owner(None)?;
    let owner_name = if owner.name.is_empty() {
        &owner.github_user
    } else {
        &owner.name
    };

    for n in &names {
        reset_one(n, &collabs[n], owner_name, !no_sync)?;
    }

    Ok(())
}
