//! Rename a collaborator's local directory and config entry.

use anyhow::Result;

use crate::config::collaborator::{load_collaborators, save_collaborators};
use crate::resolve;
use crate::util::run_cmd_checked;

/// Find the collaborator directory (collabs/{name}/to or collabs/{name}/from inside correspondence).
fn find_collab_dir(name: &str) -> Option<std::path::PathBuf> {
    let dd = resolve::data_dir();
    let collab_base = dd.join("collabs").join(name.to_lowercase());
    for suffix in &["to", "from"] {
        let candidate = collab_base.join(suffix);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

pub fn run(old_name: &str, new_name: &str, rename_repo: bool) -> Result<()> {
    let mut collabs = load_collaborators(None)?;

    if !collabs.contains_key(old_name) {
        anyhow::bail!(
            "Collaborator '{}' not found in collaborators.toml",
            old_name
        );
    }

    if collabs.contains_key(new_name) {
        anyhow::bail!(
            "Collaborator '{}' already exists in collaborators.toml",
            new_name
        );
    }

    let collab = collabs.get(old_name).unwrap().clone();

    // 1. Move directory via git mv (if it exists)
    if let Some(old_dir) = find_collab_dir(old_name) {
        let new_dir = old_dir.parent().unwrap().join(new_name.to_lowercase());
        println!(
            "Moving {} \u{2192} {}",
            old_dir.display(),
            new_dir.display()
        );
        run_cmd_checked(&[
            "git",
            "mv",
            &old_dir.to_string_lossy(),
            &new_dir.to_string_lossy(),
        ])?;
    } else {
        println!(
            "Directory for '{}' not found on disk \u{2014} skipping git mv",
            old_name
        );
    }

    // 2. Optionally rename the GitHub repo
    let new_repo = if rename_repo && !collab.repo.is_empty() {
        let owner_gh = crate::accounts::load_owner(None)
            .map(|o| o.github_user)
            .unwrap_or_default();
        let new_repo_name = format!("to-{}", new_name.to_lowercase());
        println!(
            "Renaming GitHub repo {} \u{2192} {}",
            collab.repo, new_repo_name
        );
        run_cmd_checked(&[
            "gh",
            "repo",
            "rename",
            &new_repo_name,
            "-R",
            &collab.repo,
            "--yes",
        ])?;
        if owner_gh.is_empty() {
            new_repo_name
        } else {
            format!("{}/{}", owner_gh, new_repo_name)
        }
    } else {
        collab.repo.clone()
    };

    // 3. Update collaborators.toml
    let mut updated = collab.clone();
    updated.github_user = new_name.to_string();
    updated.repo = new_repo;
    collabs.remove(old_name);
    collabs.insert(new_name.to_string(), updated);
    save_collaborators(&collabs, None)?;
    println!(
        "Renamed '{}' \u{2192} '{}' in collaborators.toml",
        old_name, new_name
    );

    println!(
        "Done. Collaborator '{}' renamed to '{}'.",
        old_name, new_name
    );
    Ok(())
}
