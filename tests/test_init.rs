//! Integration tests for corrkit init (src/init.rs).
//!
//! Each test sets HOME to a temp dir to isolate from the real
//! ~/.config/corrkit/config.toml. Tests run serially via
//! a shared mutex to avoid env var races.

mod common;

use std::sync::Mutex;
use tempfile::TempDir;

use corrkit::accounts::{load_accounts, load_owner};

static ENV_MUTEX: Mutex<()> = Mutex::new(());

/// Run init::run with HOME set to the temp dir parent, so
/// app_config::add_space writes to an isolated config.
fn run_init_isolated(
    tmp: &TempDir,
    data_dir: &std::path::Path,
    user: &str,
    provider: &str,
    password_cmd: &str,
    labels: &str,
    github_user: &str,
    name: &str,
    space: &str,
    force: bool,
) -> anyhow::Result<()> {
    let _lock = ENV_MUTEX.lock().unwrap();
    let old_home = std::env::var("HOME").ok();
    std::env::set_var("HOME", tmp.path().to_string_lossy().as_ref());
    let result = corrkit::init::run(
        user, data_dir, provider, password_cmd, labels, github_user, name,
        false, // sync
        space, force,
    );
    // Restore HOME
    if let Some(h) = old_home {
        std::env::set_var("HOME", h);
    }
    result
}

#[test]
fn test_init_creates_directory_structure() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().join("mydata");

    run_init_isolated(
        &tmp, &data_dir, "test@example.com", "gmail", "",
        "correspondence", "testgh", "Test User",
        "test-init-space", true,
    )
    .unwrap();

    assert!(data_dir.join("conversations").is_dir());
    assert!(data_dir.join("drafts").is_dir());
    assert!(data_dir.join("contacts").is_dir());
    assert!(data_dir.join("conversations").join(".gitkeep").exists());
    assert!(data_dir.join("drafts").join(".gitkeep").exists());
    assert!(data_dir.join("contacts").join(".gitkeep").exists());
    assert!(data_dir.join("accounts.toml").exists());
    assert!(data_dir.join("collaborators.toml").exists());
    assert!(data_dir.join("contacts.toml").exists());
}

#[test]
fn test_init_accounts_toml_content() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().join("initdata");

    run_init_isolated(
        &tmp, &data_dir, "alice@gmail.com", "gmail",
        "pass show email/personal", "inbox, sent",
        "alicegh", "Alice", "test-init-acct", true,
    )
    .unwrap();

    let accounts_path = data_dir.join("accounts.toml");
    let accounts = load_accounts(Some(&accounts_path)).unwrap();
    assert!(accounts.contains_key("default"));
    let acct = accounts.get("default").unwrap();
    assert_eq!(acct.provider, "gmail");
    assert_eq!(acct.user, "alice@gmail.com");
    assert!(acct.default);

    let owner = load_owner(Some(&accounts_path)).unwrap();
    assert_eq!(owner.github_user, "alicegh");
    assert_eq!(owner.name, "Alice");
}

#[test]
fn test_init_with_custom_provider() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().join("pmdata");

    run_init_isolated(
        &tmp, &data_dir, "user@proton.me", "protonmail-bridge",
        "", "correspondence", "", "", "test-init-pm", true,
    )
    .unwrap();

    let accounts_path = data_dir.join("accounts.toml");
    let accounts = load_accounts(Some(&accounts_path)).unwrap();
    let acct = accounts.get("default").unwrap();
    assert_eq!(acct.provider, "protonmail-bridge");
    assert_eq!(acct.user, "user@proton.me");
}

#[test]
fn test_init_labels_parsing() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().join("lbldata");

    run_init_isolated(
        &tmp, &data_dir, "user@example.com", "imap",
        "", "inbox, sent, important", "", "",
        "test-init-labels", true,
    )
    .unwrap();

    let accounts_path = data_dir.join("accounts.toml");
    let accounts = load_accounts(Some(&accounts_path)).unwrap();
    let acct = accounts.get("default").unwrap();
    assert_eq!(acct.labels.len(), 3);
    assert!(acct.labels.contains(&"inbox".to_string()));
    assert!(acct.labels.contains(&"sent".to_string()));
    assert!(acct.labels.contains(&"important".to_string()));
}

#[test]
fn test_init_force_overwrites() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().join("forcedata");
    std::fs::create_dir_all(&data_dir).unwrap();
    std::fs::write(data_dir.join("accounts.toml"), "# old config").unwrap();

    run_init_isolated(
        &tmp, &data_dir, "new@example.com", "gmail",
        "", "correspondence", "", "",
        "test-init-force", true,
    )
    .unwrap();

    let content = std::fs::read_to_string(data_dir.join("accounts.toml")).unwrap();
    assert!(content.contains("new@example.com"));
    assert!(!content.contains("# old config"));
}

#[test]
fn test_init_preserves_existing_config_files() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().join("preservedata");
    std::fs::create_dir_all(&data_dir).unwrap();
    std::fs::write(
        data_dir.join("collaborators.toml"),
        "[alex]\nlabels = [\"for-alex\"]\n",
    )
    .unwrap();

    run_init_isolated(
        &tmp, &data_dir, "user@example.com", "gmail",
        "", "correspondence", "", "",
        "test-init-preserve", true,
    )
    .unwrap();

    let content = std::fs::read_to_string(data_dir.join("collaborators.toml")).unwrap();
    assert!(content.contains("alex"));
}

#[test]
fn test_init_tilde_expansion() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().join("tildetest");

    run_init_isolated(
        &tmp, &data_dir, "user@example.com", "gmail",
        "", "correspondence", "", "",
        "test-init-tilde", true,
    )
    .unwrap();

    assert!(data_dir.join("accounts.toml").exists());
}

#[test]
fn test_init_empty_labels() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().join("emptylbl");

    run_init_isolated(
        &tmp, &data_dir, "user@example.com", "gmail",
        "", "", "", "",
        "test-init-emptylbl", true,
    )
    .unwrap();

    let accounts_path = data_dir.join("accounts.toml");
    let accounts = load_accounts(Some(&accounts_path)).unwrap();
    let acct = accounts.get("default").unwrap();
    assert!(acct.labels.is_empty());
}
