//! Contact configuration — parse contacts.toml.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::resolve;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    #[serde(default)]
    pub emails: Vec<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub account: String,
}

/// Load contacts.toml and return {name: Contact} mapping.
pub fn load_contacts(path: Option<&Path>) -> Result<BTreeMap<String, Contact>> {
    let path = path
        .map(PathBuf::from)
        .unwrap_or_else(resolve::contacts_toml);
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let content = std::fs::read_to_string(&path)?;
    if content.trim().is_empty() {
        return Ok(BTreeMap::new());
    }
    let raw: toml::Value = toml::from_str(&content)?;
    let table = raw.as_table().ok_or_else(|| anyhow::anyhow!("Invalid contacts.toml"))?;

    let mut result = BTreeMap::new();
    for (name, data) in table {
        let contact: Contact = data.clone().try_into()?;
        result.insert(name.clone(), contact);
    }
    Ok(result)
}

/// Write contacts back to TOML.
pub fn save_contacts(
    contacts: &BTreeMap<String, Contact>,
    path: Option<&Path>,
) -> Result<()> {
    let path = path
        .map(PathBuf::from)
        .unwrap_or_else(resolve::contacts_toml);
    let mut lines = Vec::new();
    for (name, c) in contacts {
        lines.push(format!("[{}]", name));
        if !c.emails.is_empty() {
            let emails: Vec<String> = c.emails.iter().map(|e| format!("\"{}\"", e)).collect();
            lines.push(format!("emails = [{}]", emails.join(", ")));
        }
        if !c.labels.is_empty() {
            let labels: Vec<String> = c.labels.iter().map(|l| format!("\"{}\"", l)).collect();
            lines.push(format!("labels = [{}]", labels.join(", ")));
        }
        if !c.account.is_empty() {
            lines.push(format!("account = \"{}\"", c.account));
        }
        lines.push(String::new());
    }
    std::fs::write(&path, lines.join("\n"))?;
    Ok(())
}
