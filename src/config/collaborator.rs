//! Collaborator configuration — parse collaborators.toml.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::resolve;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collaborator {
    pub labels: Vec<String>,
    #[serde(default)]
    pub repo: String,
    #[serde(default, skip_serializing)]
    pub github_user: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub account: String,
}

/// Return the local collab directory (correspondence/for/{gh_user}/).
pub fn collab_dir(collab: &Collaborator) -> PathBuf {
    resolve::collab_for_dir(&collab.github_user)
}

/// Derive the default repo name: {owner}/to-{collab}.
fn auto_repo(owner_gh: &str, collab_gh: &str) -> String {
    format!("{}/to-{}", owner_gh, collab_gh.to_lowercase())
}

/// Load collaborators.toml and return {github_user: Collaborator} mapping.
pub fn load_collaborators(path: Option<&Path>) -> Result<BTreeMap<String, Collaborator>> {
    let path = path
        .map(PathBuf::from)
        .unwrap_or_else(resolve::collaborators_toml);
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let content = std::fs::read_to_string(&path)?;
    if content.trim().is_empty() {
        return Ok(BTreeMap::new());
    }
    let raw: toml::Value = toml::from_str(&content)?;
    let table = raw.as_table().ok_or_else(|| anyhow::anyhow!("Invalid collaborators.toml"))?;

    // Load owner for repo auto-derivation
    let owner_gh = crate::accounts::load_owner(None)
        .map(|o| o.github_user)
        .unwrap_or_default();

    let mut result = BTreeMap::new();
    for (gh_user, data) in table {
        let mut collab: Collaborator = data.clone().try_into()?;
        collab.github_user = gh_user.clone();
        if collab.repo.is_empty() && !owner_gh.is_empty() {
            collab.repo = auto_repo(&owner_gh, gh_user);
        }
        result.insert(gh_user.clone(), collab);
    }
    Ok(result)
}

/// Write collaborators back to TOML.
pub fn save_collaborators(
    collabs: &BTreeMap<String, Collaborator>,
    path: Option<&Path>,
) -> Result<()> {
    let path = path
        .map(PathBuf::from)
        .unwrap_or_else(resolve::collaborators_toml);
    let mut lines = Vec::new();
    for (gh_user, c) in collabs {
        lines.push(format!("[{}]", gh_user));
        let labels: Vec<String> = c.labels.iter().map(|l| format!("\"{}\"", l)).collect();
        lines.push(format!("labels = [{}]", labels.join(", ")));
        if !c.name.is_empty() {
            lines.push(format!("name = \"{}\"", c.name));
        }
        if !c.repo.is_empty() {
            lines.push(format!("repo = \"{}\"", c.repo));
        }
        if !c.account.is_empty() {
            lines.push(format!("account = \"{}\"", c.account));
        }
        lines.push(String::new());
    }
    std::fs::write(&path, lines.join("\n"))?;
    Ok(())
}
