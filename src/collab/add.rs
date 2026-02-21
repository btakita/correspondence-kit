//! Add a new collaborator: create shared GitHub repo, init it, add as submodule.

use anyhow::Result;

use crate::accounts::load_owner;
use crate::config::collaborator::{
    collab_dir, load_collaborators, save_collaborators, Collaborator,
};
use crate::resolve;
use crate::util::run_cmd_checked;

use super::templates::{generate_agents_md, generate_readme_md};

pub fn run(
    github_user: &str,
    labels: &[String],
    name: &str,
    pat: bool,
    public: bool,
    account: &str,
    org: &str,
) -> Result<()> {
    let owner = load_owner(None)?;
    let gh_user_lower = github_user.to_lowercase();
    let org = if org.is_empty() {
        &owner.github_user
    } else {
        org
    };
    let owner_name = if owner.name.is_empty() {
        &owner.github_user
    } else {
        &owner.name
    };
    let repo_name = format!("to-{}", gh_user_lower);
    let repo_full = format!("{}/{}", org, repo_name);

    // Check not already configured
    let mut collabs = load_collaborators(None)?;
    if collabs.contains_key(github_user) {
        anyhow::bail!(
            "Collaborator '{}' already exists in collaborators.toml",
            github_user
        );
    }

    let collab_obj = Collaborator {
        labels: labels.to_vec(),
        github_user: github_user.to_string(),
        name: name.to_string(),
        repo: repo_full.clone(),
        account: account.to_string(),
    };
    let submodule_path = collab_dir(&collab_obj);
    if submodule_path.exists() {
        anyhow::bail!("Directory {} already exists", submodule_path.display());
    }

    // 1. Create GitHub repo
    let visibility = if public { "--public" } else { "--private" };
    println!(
        "Creating GitHub repo: {} ({})",
        repo_full,
        visibility.trim_start_matches("--")
    );
    run_cmd_checked(&["gh", "repo", "create", &repo_full, visibility, "--confirm"])?;

    // 2. Add collaborator if not --pat
    if !pat {
        println!("Adding {} as collaborator on {}", github_user, repo_full);
        run_cmd_checked(&[
            "gh",
            "api",
            &format!("repos/{}/collaborators/{}", repo_full, github_user),
            "-X",
            "PUT",
            "--silent",
        ])?;
    } else {
        println!();
        println!("PAT access mode selected. The collaborator should:");
        println!("  1. Go to https://github.com/settings/personal-access-tokens/new");
        println!("  2. Create a fine-grained PAT scoped to: {}", repo_full);
        println!("  3. Grant 'Contents' read/write permission");
        println!(
            "  4. Use the PAT to clone: https://github.com/{}.git",
            repo_full
        );
        println!();
    }

    // 3. Initialize the shared repo
    let display = if name.is_empty() { github_user } else { name };
    println!("Initializing shared repo contents...");

    let tmpdir = tempfile::tempdir()?;
    let tmp = tmpdir.path();

    run_cmd_checked(&["gh", "repo", "clone", &repo_full, &tmp.to_string_lossy()])?;

    // AGENTS.md + CLAUDE.md symlink + README.md
    std::fs::write(
        tmp.join("AGENTS.md"),
        generate_agents_md(display, owner_name),
    )?;
    #[cfg(unix)]
    std::os::unix::fs::symlink("AGENTS.md", tmp.join("CLAUDE.md"))?;
    std::fs::write(
        tmp.join("README.md"),
        generate_readme_md(display, owner_name),
    )?;

    // .gitignore
    std::fs::write(
        tmp.join(".gitignore"),
        "AGENTS.local.md\nCLAUDE.local.md\n__pycache__/\n",
    )?;

    // voice.md
    let voice_file = resolve::voice_md();
    if voice_file.exists() {
        std::fs::copy(&voice_file, tmp.join("voice.md"))?;
    }

    // directories
    std::fs::create_dir_all(tmp.join("conversations"))?;
    std::fs::write(tmp.join("conversations/.gitkeep"), "")?;
    std::fs::create_dir_all(tmp.join("drafts"))?;
    std::fs::write(tmp.join("drafts/.gitkeep"), "")?;

    // commit and push
    let tmp_str = tmp.to_string_lossy().to_string();
    run_cmd_checked(&["git", "-C", &tmp_str, "add", "-A"])?;
    run_cmd_checked(&[
        "git",
        "-C",
        &tmp_str,
        "commit",
        "-m",
        &format!(
            "Initialize shared correspondence repo for {}",
            display
        ),
    ])?;
    run_cmd_checked(&["git", "-C", &tmp_str, "push"])?;

    // 4. Add as git submodule
    let repo_url = format!("git@github.com:{}.git", repo_full);
    let sub_path = submodule_path.to_string_lossy().to_string();
    println!("Adding submodule: {} -> {}", sub_path, repo_url);
    run_cmd_checked(&["git", "submodule", "add", &repo_url, &sub_path])?;

    // 5. Update collaborators.toml
    collabs.insert(github_user.to_string(), collab_obj);
    save_collaborators(&collabs, None)?;
    println!("Updated collaborators.toml");

    // 6. Remind about label sync
    println!();
    println!("Done! Next steps:");
    for label in labels {
        println!(
            "  - Ensure '{}' is in your account's labels in accounts.toml",
            label
        );
    }
    println!("  - Run: corrkit sync --full");
    println!("  - Run: corrkit for sync {}", github_user);

    Ok(())
}
