//! Sync shared collaborator submodules: pull changes, push updates.

use anyhow::Result;
use std::path::Path;
use std::process::Command;

use crate::config::collaborator::{collab_dir, load_collaborators};
use crate::resolve;

fn run_git(args: &[&str]) -> (String, String, i32) {
    let output = Command::new(args[0])
        .args(&args[1..])
        .output()
        .unwrap_or_else(|_| {
            panic!("Failed to run: {}", args.join(" "));
        });
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let code = output.status.code().unwrap_or(-1);
    (stdout, stderr, code)
}

fn submodule_status(name: &str, sub_path: &Path) {
    let sp = sub_path.to_string_lossy().to_string();
    run_git(&["git", "-C", &sp, "fetch"]);

    let (incoming, _, inc_code) =
        run_git(&["git", "-C", &sp, "rev-list", "--count", "HEAD..@{u}"]);
    let (outgoing, _, out_code) =
        run_git(&["git", "-C", &sp, "rev-list", "--count", "@{u}..HEAD"]);

    let inc = if inc_code == 0 {
        incoming.trim().to_string()
    } else {
        "?".to_string()
    };
    let out = if out_code == 0 {
        outgoing.trim().to_string()
    } else {
        "?".to_string()
    };

    if inc == "0" && out == "0" {
        println!("  {}: up to date", name);
    } else {
        let mut parts = Vec::new();
        if inc != "0" {
            parts.push(format!("{} incoming", inc));
        }
        if out != "0" {
            parts.push(format!("{} outgoing", out));
        }
        println!("  {}: {}", name, parts.join(", "));
    }
}

/// Full sync for one collaborator submodule.
pub fn sync_one(name: &str) -> Result<()> {
    let collabs = load_collaborators(None)?;
    let collab = match collabs.get(name) {
        Some(c) => c,
        None => {
            println!("  {}: not found in collaborators.toml -- skipping", name);
            return Ok(());
        }
    };

    let sub_path = collab_dir(collab);
    if !sub_path.exists() {
        println!(
            "  {}: submodule not found at {} -- skipping",
            name,
            sub_path.display()
        );
        return Ok(());
    }

    println!("Syncing {}...", name);
    let sp = sub_path.to_string_lossy().to_string();

    // Pull collaborator's changes
    let (stdout, _stderr, code) = run_git(&["git", "-C", &sp, "pull", "--rebase"]);
    if code == 0 {
        if !stdout.contains("Already up to date") {
            println!("  Pulled changes");
        }
    } else {
        println!("  Pull failed -- continuing with push");
    }

    // Copy voice.md if root copy is newer
    let voice_file = resolve::voice_md();
    let sub_voice = sub_path.join("voice.md");
    if voice_file.exists() {
        let should_copy = if sub_voice.exists() {
            let root_mtime = voice_file.metadata().ok().and_then(|m| m.modified().ok());
            let sub_mtime = sub_voice.metadata().ok().and_then(|m| m.modified().ok());
            match (root_mtime, sub_mtime) {
                (Some(r), Some(s)) => r > s,
                _ => true,
            }
        } else {
            true
        };
        if should_copy {
            std::fs::copy(&voice_file, &sub_voice)?;
            println!("  Updated voice.md");
        }
    }

    // Stage, commit, push any local changes
    run_git(&["git", "-C", &sp, "add", "-A"]);

    let (status_out, _, _) = run_git(&["git", "-C", &sp, "status", "--porcelain"]);
    if !status_out.trim().is_empty() {
        run_git(&[
            "git",
            "-C",
            &sp,
            "commit",
            "-m",
            "Sync shared conversations",
        ]);
        let (_, stderr, code) = run_git(&["git", "-C", &sp, "push"]);
        if code == 0 {
            println!("  Pushed changes");
        } else {
            println!("  Push failed: {}", stderr.trim());
        }
    } else {
        println!("  No local changes to push");
    }

    // Update submodule ref in parent
    run_git(&["git", "add", &sp]);

    Ok(())
}

/// corrkit for sync [NAME]
pub fn run(name: Option<&str>) -> Result<()> {
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

    for n in &names {
        sync_one(n)?;
    }

    Ok(())
}

/// corrkit for status
pub fn status() -> Result<()> {
    let collabs = load_collaborators(None)?;
    if collabs.is_empty() {
        println!("No collaborators configured in collaborators.toml");
        return Ok(());
    }

    println!("Collaborator status:");
    for (name, collab) in &collabs {
        let sub_path = collab_dir(collab);
        if sub_path.exists() {
            submodule_status(name, &sub_path);
        } else {
            println!("  {}: submodule not found", name);
        }
    }

    Ok(())
}
