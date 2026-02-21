//! Integration tests for app config / spaces (src/app_config.rs).

mod common;

use corrkit::app_config;

#[test]
fn test_app_config_dir_returns_path() {
    let dir = app_config::app_config_dir();
    // Should contain "corrkit" somewhere in the path
    assert!(dir.to_string_lossy().contains("corrkit"));
}

#[test]
fn test_app_config_path_returns_toml() {
    let path = app_config::app_config_path();
    assert!(path.to_string_lossy().ends_with("config.toml"));
}

#[test]
fn test_resolve_space_no_config() {
    // When asked for a nonexistent space, should error or return None
    // Either way, it shouldn't panic
    let result = app_config::resolve_space(Some("nonexistent-space-xyz"));
    let _ = result;
}

#[test]
fn test_list_spaces_no_panic() {
    // Should not panic even if config doesn't exist or is corrupted
    let _ = app_config::list_spaces();
}

#[test]
fn test_load_no_panic() {
    // Should not panic even if config doesn't exist or is corrupted
    let _ = app_config::load();
}
