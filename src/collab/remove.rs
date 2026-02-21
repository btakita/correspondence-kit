//! Remove a collaborator: deinit submodule, remove from config.

use anyhow::Result;
use std::io::Write;
use std::path::PathBuf;

use crate::config::collaborator::{collab_dir, load_collaborators, save_collaborators};
use crate::util::run_cmd_checked;

pub fn run(name: &str, delete_repo: bool) -> Result<()> {
    let mut collabs = load_collaborators(None)?;

    let collab = collabs.get(name).ok_or_else(|| {
        anyhow::anyhow!(
            "Collaborator '{}' not found in collaborators.toml",
            name
        )
    })?;
    let collab = collab.clone();
    let sub_path = collab_dir(&collab);

    // 1. Deinit and remove submodule
    if sub_path.exists() {
        println!("Removing submodule: {}", sub_path.display());
        let sp = sub_path.to_string_lossy().to_string();
        run_cmd_checked(&["git", "submodule", "deinit", "-f", &sp])?;
        run_cmd_checked(&["git", "rm", "-f", &sp])?;
    } else {
        println!(
            "Submodule {} not found on disk -- skipping git cleanup",
            sub_path.display()
        );
    }

    // Clean up .git/modules entry
    let modules_path = PathBuf::from(".git/modules").join(sub_path.to_string_lossy().as_ref());
    if modules_path.exists() {
        std::fs::remove_dir_all(&modules_path)?;
        println!("  Cleaned up {}", modules_path.display());
    }

    // 2. Remove from collaborators.toml
    collabs.remove(name);
    save_collaborators(&collabs, None)?;
    println!("Removed '{}' from collaborators.toml", name);

    // 3. Optionally delete the GitHub repo
    if delete_repo && !collab.repo.is_empty() {
        print!(
            "Delete GitHub repo {}? This cannot be undone. [y/N] ",
            collab.repo
        );
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() == "y" {
            run_cmd_checked(&["gh", "repo", "delete", &collab.repo, "--yes"])?;
            println!("Deleted GitHub repo: {}", collab.repo);
        } else {
            println!("Skipped repo deletion");
        }
    }

    println!("Done. Collaborator '{}' removed.", name);
    Ok(())
}
