//! Token store for social media OAuth tokens.
//!
//! Stores tokens keyed by URN in ~/.config/corky/tokens.json with 0600 permissions.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::app_config;

/// A stored OAuth token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredToken {
    pub access_token: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub scopes: Vec<String>,
    pub platform: String,
}

/// Grace window: tokens expiring within this many seconds are considered expired.
const GRACE_SECONDS: i64 = 300; // 5 minutes

impl StoredToken {
    /// Check if the token is still valid (not expired, accounting for grace window).
    pub fn is_valid(&self) -> bool {
        let now = Utc::now();
        let grace = chrono::Duration::seconds(GRACE_SECONDS);
        self.expires_at > now + grace
    }
}

/// URN-keyed token store.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenStore {
    pub tokens: HashMap<String, StoredToken>,
}

/// Return the path to tokens.json.
pub fn tokens_path() -> PathBuf {
    app_config::app_config_dir().join("tokens.json")
}

impl TokenStore {
    /// Load the token store from disk. Returns empty store if file doesn't exist.
    pub fn load() -> Result<Self> {
        Self::load_from(&tokens_path())
    }

    /// Load from a specific path.
    pub fn load_from(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            return Ok(TokenStore::default());
        }
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read tokens from {}", path.display()))?;
        let store: TokenStore = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse tokens from {}", path.display()))?;
        Ok(store)
    }

    /// Save the token store to disk with 0600 permissions.
    pub fn save(&self) -> Result<()> {
        self.save_to(&tokens_path())
    }

    /// Save to a specific path.
    pub fn save_to(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, &content)?;

        // Set file permissions to 0600 (owner read/write only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(path, perms)?;
        }

        Ok(())
    }

    /// Get a valid (non-expired) token for a URN.
    pub fn get_valid(&self, urn: &str) -> Option<&StoredToken> {
        self.tokens.get(urn).filter(|t| t.is_valid())
    }

    /// Insert or update a token for a URN.
    pub fn upsert(&mut self, urn: String, token: StoredToken) {
        self.tokens.insert(urn, token);
    }

    /// Remove a token by URN.
    pub fn remove(&mut self, urn: &str) -> Option<StoredToken> {
        self.tokens.remove(urn)
    }
}
