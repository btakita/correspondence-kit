//! Integration tests for collaborator config parsing (src/config/collaborator.rs).

mod common;

use std::collections::BTreeMap;
use tempfile::TempDir;

use corrkit::config::collaborator::{self, Collaborator};

#[test]
fn test_load_collaborators_empty_file() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("collaborators.toml");
    std::fs::write(&path, "").unwrap();

    let collabs = collaborator::load_collaborators(Some(&path)).unwrap();
    assert!(collabs.is_empty());
}

#[test]
fn test_load_collaborators_missing_file() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("nonexistent.toml");

    let collabs = collaborator::load_collaborators(Some(&path)).unwrap();
    assert!(collabs.is_empty());
}

#[test]
fn test_load_collaborators_basic() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("collaborators.toml");
    std::fs::write(
        &path,
        r#"
[alex]
labels = ["for-alex"]
name = "Alex"
"#,
    )
    .unwrap();

    let collabs = collaborator::load_collaborators(Some(&path)).unwrap();
    assert_eq!(collabs.len(), 1);

    let alex = collabs.get("alex").unwrap();
    assert_eq!(alex.labels, vec!["for-alex"]);
    assert_eq!(alex.name, "Alex");
    assert_eq!(alex.github_user, "alex");
    // Auto-repo derivation depends on load_owner(None) finding accounts.toml
    // via resolve::accounts_toml(). If owner is not found, repo stays empty.
    // We test auto-repo derivation separately via explicit repo.
}

#[test]
fn test_load_collaborators_with_account_scoped_labels() {
    let tmp = TempDir::new().unwrap();
    common::write_accounts_toml(tmp.path(), "owner@test.com");
    std::env::set_var("CORRKIT_DATA", tmp.path().to_string_lossy().as_ref());

    let path = tmp.path().join("collaborators.toml");
    std::fs::write(
        &path,
        r#"
[bot-agent]
labels = ["for-bot", "proton-dev:INBOX"]
account = "personal"
"#,
    )
    .unwrap();

    let collabs = collaborator::load_collaborators(Some(&path)).unwrap();
    let bot = collabs.get("bot-agent").unwrap();
    assert_eq!(bot.labels, vec!["for-bot", "proton-dev:INBOX"]);
    assert_eq!(bot.account, "personal");

    std::env::remove_var("CORRKIT_DATA");
}

#[test]
fn test_load_collaborators_explicit_repo() {
    let tmp = TempDir::new().unwrap();
    common::write_accounts_toml(tmp.path(), "owner@test.com");
    std::env::set_var("CORRKIT_DATA", tmp.path().to_string_lossy().as_ref());

    let path = tmp.path().join("collaborators.toml");
    std::fs::write(
        &path,
        r#"
[custom]
labels = ["for-custom"]
repo = "myorg/custom-repo"
"#,
    )
    .unwrap();

    let collabs = collaborator::load_collaborators(Some(&path)).unwrap();
    let custom = collabs.get("custom").unwrap();
    // Explicit repo should NOT be overridden
    assert_eq!(custom.repo, "myorg/custom-repo");

    std::env::remove_var("CORRKIT_DATA");
}

#[test]
fn test_load_collaborators_multiple() {
    let tmp = TempDir::new().unwrap();
    common::write_accounts_toml(tmp.path(), "owner@test.com");
    std::env::set_var("CORRKIT_DATA", tmp.path().to_string_lossy().as_ref());

    let path = tmp.path().join("collaborators.toml");
    std::fs::write(
        &path,
        r#"
[alice]
labels = ["for-alice"]
name = "Alice"

[bob]
labels = ["for-bob"]
name = "Bob"

[charlie]
labels = ["for-charlie"]
name = "Charlie"
"#,
    )
    .unwrap();

    let collabs = collaborator::load_collaborators(Some(&path)).unwrap();
    assert_eq!(collabs.len(), 3);
    assert!(collabs.contains_key("alice"));
    assert!(collabs.contains_key("bob"));
    assert!(collabs.contains_key("charlie"));

    std::env::remove_var("CORRKIT_DATA");
}

#[test]
fn test_save_collaborators() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("collaborators.toml");

    let mut collabs = BTreeMap::new();
    collabs.insert(
        "alex".to_string(),
        Collaborator {
            labels: vec!["for-alex".to_string()],
            repo: "owner/to-alex".to_string(),
            github_user: "alex".to_string(),
            name: "Alex".to_string(),
            account: "personal".to_string(),
        },
    );

    collaborator::save_collaborators(&collabs, Some(&path)).unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("[alex]"));
    assert!(content.contains("\"for-alex\""));
    assert!(content.contains("name = \"Alex\""));
    assert!(content.contains("repo = \"owner/to-alex\""));
    assert!(content.contains("account = \"personal\""));
}

#[test]
fn test_save_and_reload_collaborators() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("collaborators.toml");

    // We need owner for auto-repo derivation on reload
    common::write_accounts_toml(tmp.path(), "owner@test.com");
    std::env::set_var("CORRKIT_DATA", tmp.path().to_string_lossy().as_ref());

    let mut collabs = BTreeMap::new();
    collabs.insert(
        "alex".to_string(),
        Collaborator {
            labels: vec!["for-alex".to_string(), "shared".to_string()],
            repo: "owner/to-alex".to_string(),
            github_user: "alex".to_string(),
            name: "Alex".to_string(),
            account: String::new(),
        },
    );

    collaborator::save_collaborators(&collabs, Some(&path)).unwrap();
    let reloaded = collaborator::load_collaborators(Some(&path)).unwrap();

    assert_eq!(reloaded.len(), 1);
    let alex = reloaded.get("alex").unwrap();
    assert_eq!(alex.labels, vec!["for-alex", "shared"]);
    assert_eq!(alex.name, "Alex");

    std::env::remove_var("CORRKIT_DATA");
}

#[test]
fn test_collab_dir() {
    let tmp = TempDir::new().unwrap();
    std::env::set_var("CORRKIT_DATA", tmp.path().to_string_lossy().as_ref());

    let collab = Collaborator {
        labels: vec![],
        repo: String::new(),
        github_user: "AlexUser".to_string(),
        name: String::new(),
        account: String::new(),
    };

    let dir = collaborator::collab_dir(&collab);
    // Should lowercase the github user
    assert!(dir.to_string_lossy().ends_with("collabs/alexuser/to"));

    std::env::remove_var("CORRKIT_DATA");
}

#[test]
fn test_collaborator_default_fields() {
    let tmp = TempDir::new().unwrap();
    common::write_accounts_toml(tmp.path(), "owner@test.com");
    std::env::set_var("CORRKIT_DATA", tmp.path().to_string_lossy().as_ref());

    let path = tmp.path().join("collaborators.toml");
    std::fs::write(
        &path,
        r#"
[minimal]
labels = ["test"]
"#,
    )
    .unwrap();

    let collabs = collaborator::load_collaborators(Some(&path)).unwrap();
    let minimal = collabs.get("minimal").unwrap();
    assert!(minimal.name.is_empty());
    assert!(minimal.account.is_empty());
    assert_eq!(minimal.github_user, "minimal");

    std::env::remove_var("CORRKIT_DATA");
}
