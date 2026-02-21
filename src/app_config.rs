//! App-level config for corrkit (spaces, defaults).
//!
//! Reads/writes {user_config_dir}/corrkit/config.toml.

use anyhow::{bail, Result};
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::resolve;

/// Return the OS-native corrkit config directory.
pub fn app_config_dir() -> PathBuf {
    if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "corrkit") {
        proj_dirs.config_dir().to_path_buf()
    } else {
        resolve::home_dir().join(".config").join("corrkit")
    }
}

/// Return the path to config.toml.
pub fn app_config_path() -> PathBuf {
    app_config_dir().join("config.toml")
}

/// Read config.toml, returning empty table if missing.
pub fn load() -> Result<toml::Value> {
    let path = app_config_path();
    if !path.exists() {
        return Ok(toml::Value::Table(toml::map::Map::new()));
    }
    let content = std::fs::read_to_string(&path)?;
    let val: toml::Value = toml::from_str(&content)?;
    Ok(val)
}

/// Write config.toml, creating parent dir if needed.
pub fn save(config: &toml::Value) -> Result<()> {
    let path = app_config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(config)?;
    std::fs::write(&path, content)?;
    Ok(())
}

/// Resolve a space name to a data directory path.
///
/// - If name given: look up, error if not found.
/// - No name + default_space set: use default.
/// - No name + exactly 1 space: use it implicitly.
/// - No name + multiple spaces, no default: error with list.
/// - No spaces configured: return None.
pub fn resolve_space(name: Option<&str>) -> Result<Option<PathBuf>> {
    let config = load()?;
    let table = config.as_table().cloned().unwrap_or_default();
    let spaces = match table.get("spaces") {
        Some(toml::Value::Table(s)) => s.clone(),
        _ => return Ok(None),
    };

    if spaces.is_empty() {
        return Ok(None);
    }

    if let Some(name) = name {
        match spaces.get(name) {
            Some(space_val) => {
                let path = space_path(space_val)?;
                return Ok(Some(path));
            }
            None => {
                let available: Vec<&String> = spaces.keys().collect();
                bail!(
                    "Unknown space '{}'. Available: {}",
                    name,
                    available.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
                );
            }
        }
    }

    // No name given — try defaults
    if let Some(toml::Value::String(default)) = table.get("default_space") {
        if let Some(space_val) = spaces.get(default.as_str()) {
            let path = space_path(space_val)?;
            return Ok(Some(path));
        }
    }

    if spaces.len() == 1 {
        let (_, space_val) = spaces.iter().next().unwrap();
        let path = space_path(space_val)?;
        return Ok(Some(path));
    }

    // Multiple spaces, no default
    eprintln!("Multiple spaces configured. Use --space NAME or set default_space.");
    eprintln!();
    for (sname, sconf) in &spaces {
        if let Some(p) = sconf.get("path").and_then(|v| v.as_str()) {
            eprintln!("  {}  {}", sname, p);
        }
    }
    std::process::exit(1);
}

/// Register a space, auto-default if first.
pub fn add_space(name: &str, path: &str) -> Result<()> {
    let mut config = load()?;
    let table = config.as_table_mut().unwrap();

    let spaces = table
        .entry("spaces")
        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()))
        .as_table_mut()
        .unwrap();

    let mut space_entry = toml::map::Map::new();
    space_entry.insert("path".to_string(), toml::Value::String(path.to_string()));
    spaces.insert(name.to_string(), toml::Value::Table(space_entry));

    if spaces.len() == 1 {
        table.insert(
            "default_space".to_string(),
            toml::Value::String(name.to_string()),
        );
    }

    save(&config)
}

/// List all configured spaces as (name, path, is_default).
pub fn list_spaces() -> Result<Vec<(String, String, bool)>> {
    let config = load()?;
    let table = config.as_table().cloned().unwrap_or_default();
    let default = table
        .get("default_space")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let spaces = match table.get("spaces") {
        Some(toml::Value::Table(s)) => s.clone(),
        _ => return Ok(vec![]),
    };

    let mut result = vec![];
    // Use BTreeMap for sorted output
    let sorted: BTreeMap<_, _> = spaces.into_iter().collect();
    for (name, val) in sorted {
        let path = val
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let is_default = name == default;
        result.push((name, path, is_default));
    }
    Ok(result)
}

fn space_path(space_val: &toml::Value) -> Result<PathBuf> {
    let path_str = space_val
        .get("path")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    Ok(resolve::expand_tilde(path_str))
}
