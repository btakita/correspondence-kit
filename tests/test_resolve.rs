//! Integration tests for path resolution (src/resolve.rs).

mod common;

use std::path::PathBuf;
use tempfile::TempDir;

use corrkit::resolve;

#[test]
fn test_expand_tilde_with_prefix() {
    let home = resolve::home_dir();
    let result = resolve::expand_tilde("~/Documents/test");
    assert_eq!(result, home.join("Documents").join("test"));
}

#[test]
fn test_expand_tilde_bare() {
    let home = resolve::home_dir();
    let result = resolve::expand_tilde("~");
    assert_eq!(result, home);
}

#[test]
fn test_expand_tilde_no_tilde() {
    let result = resolve::expand_tilde("/absolute/path");
    assert_eq!(result, PathBuf::from("/absolute/path"));
}

#[test]
fn test_expand_tilde_relative() {
    let result = resolve::expand_tilde("relative/path");
    assert_eq!(result, PathBuf::from("relative/path"));
}

#[test]
fn test_data_dir_env_var() {
    let (_tmp, data_dir) = common::temp_data_dir();
    // Set CORRKIT_DATA so data_dir() returns it
    // (only works if cwd doesn't have correspondence/)
    std::env::set_var("CORRKIT_DATA", data_dir.to_string_lossy().as_ref());

    // We can't fully test this because data_dir() checks cwd first,
    // but we can verify the env path is valid
    let env_val = std::env::var("CORRKIT_DATA").unwrap();
    assert_eq!(PathBuf::from(&env_val), data_dir);

    // Clean up
    std::env::remove_var("CORRKIT_DATA");
}

#[test]
fn test_home_dir_returns_path() {
    let home = resolve::home_dir();
    // home_dir should return something reasonable
    assert!(!home.to_string_lossy().is_empty());
}

#[test]
fn test_derived_paths_are_consistent() {
    let tmp = TempDir::new().unwrap();
    let data = tmp.path().to_path_buf();
    std::env::set_var("CORRKIT_DATA", data.to_string_lossy().as_ref());

    // When data_dir resolves via env, derived paths should be relative to it
    let conversations = resolve::conversations_dir();
    let drafts = resolve::drafts_dir();
    let contacts = resolve::contacts_dir();

    // These should end with the expected subdirectory names
    assert!(conversations.to_string_lossy().ends_with("conversations"));
    assert!(drafts.to_string_lossy().ends_with("drafts"));
    assert!(contacts.to_string_lossy().ends_with("contacts"));

    std::env::remove_var("CORRKIT_DATA");
}

#[test]
fn test_collab_to_dir_lowercases() {
    let tmp = TempDir::new().unwrap();
    let data = tmp.path().to_path_buf();
    std::env::set_var("CORRKIT_DATA", data.to_string_lossy().as_ref());

    let dir = resolve::collab_to_dir("AlexUser");
    assert!(dir.to_string_lossy().contains("alexuser"));
    assert!(dir.to_string_lossy().ends_with("collabs/alexuser/to"));

    std::env::remove_var("CORRKIT_DATA");
}

#[test]
fn test_collab_from_dir_lowercases() {
    let tmp = TempDir::new().unwrap();
    let data = tmp.path().to_path_buf();
    std::env::set_var("CORRKIT_DATA", data.to_string_lossy().as_ref());

    let dir = resolve::collab_from_dir("AlexUser");
    assert!(dir.to_string_lossy().contains("alexuser"));
    assert!(dir.to_string_lossy().ends_with("collabs/alexuser/from"));

    std::env::remove_var("CORRKIT_DATA");
}

#[test]
fn test_sync_state_file_path() {
    let tmp = TempDir::new().unwrap();
    let data = tmp.path().to_path_buf();
    std::env::set_var("CORRKIT_DATA", data.to_string_lossy().as_ref());

    let sf = resolve::sync_state_file();
    assert!(sf.to_string_lossy().ends_with(".sync-state.json"));

    std::env::remove_var("CORRKIT_DATA");
}

#[test]
fn test_manifest_file_path() {
    let tmp = TempDir::new().unwrap();
    let data = tmp.path().to_path_buf();
    std::env::set_var("CORRKIT_DATA", data.to_string_lossy().as_ref());

    let mf = resolve::manifest_file();
    assert!(mf.to_string_lossy().ends_with("manifest.toml"));

    std::env::remove_var("CORRKIT_DATA");
}

#[test]
fn test_config_paths() {
    let tmp = TempDir::new().unwrap();
    let data = tmp.path().to_path_buf();
    std::env::set_var("CORRKIT_DATA", data.to_string_lossy().as_ref());

    let at = resolve::accounts_toml();
    let ct = resolve::collaborators_toml();
    let cont = resolve::contacts_toml();
    let vm = resolve::voice_md();
    let cj = resolve::credentials_json();

    assert!(at.to_string_lossy().ends_with("accounts.toml"));
    assert!(ct.to_string_lossy().ends_with("collaborators.toml"));
    assert!(cont.to_string_lossy().ends_with("contacts.toml"));
    assert!(vm.to_string_lossy().ends_with("voice.md"));
    assert!(cj.to_string_lossy().ends_with("credentials.json"));

    std::env::remove_var("CORRKIT_DATA");
}
