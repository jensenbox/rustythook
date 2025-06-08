//! Unit tests for the configuration module

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use rustyhook::config::{Config, Hook, Repo, parse_config};

#[test]
fn test_parse_config() {
    // Create a temporary directory for the test
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    // Create a test configuration
    let config_str = r#"
default_stages: [commit, push]
fail_fast: true
repos:
  - repo: local
    hooks:
      - id: test-hook
        name: Test Hook
        entry: test-command
        language: system
        files: ".*\\.rs$"
        stages: [commit]
        args: [--verbose]
        env:
          TEST_VAR: test_value
        version: "1.0.0"
"#;
    
    // Write the configuration to a file
    fs::write(&config_path, config_str).unwrap();
    
    // Parse the configuration
    let config = parse_config(&config_path).unwrap();
    
    // Check the configuration
    assert_eq!(config.default_stages, vec!["commit".to_string(), "push".to_string()]);
    assert_eq!(config.fail_fast, true);
    assert_eq!(config.repos.len(), 1);
    
    // Check the repository
    let repo = &config.repos[0];
    assert_eq!(repo.repo, "local");
    assert_eq!(repo.hooks.len(), 1);
    
    // Check the hook
    let hook = &repo.hooks[0];
    assert_eq!(hook.id, "test-hook");
    assert_eq!(hook.name, "Test Hook");
    assert_eq!(hook.entry, "test-command");
    assert_eq!(hook.language, "system");
    assert_eq!(hook.files, ".*\\.rs$");
    assert_eq!(hook.stages, vec!["commit".to_string()]);
    assert_eq!(hook.args, vec!["--verbose".to_string()]);
    
    // Check the environment variables
    let mut expected_env = HashMap::new();
    expected_env.insert("TEST_VAR".to_string(), "test_value".to_string());
    assert_eq!(hook.env, expected_env);
    
    // Check the version
    assert_eq!(hook.version, Some("1.0.0".to_string()));
}

#[test]
fn test_default_values() {
    // Create a temporary directory for the test
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    // Create a test configuration with minimal values
    let config_str = r#"
repos:
  - repo: local
    hooks:
      - id: test-hook
        name: Test Hook
        entry: test-command
        language: system
"#;
    
    // Write the configuration to a file
    fs::write(&config_path, config_str).unwrap();
    
    // Parse the configuration
    let config = parse_config(&config_path).unwrap();
    
    // Check the default values
    assert_eq!(config.default_stages, vec!["commit".to_string()]);
    assert_eq!(config.fail_fast, false);
    
    // Check the hook default values
    let hook = &config.repos[0].hooks[0];
    assert_eq!(hook.files, "");
    assert_eq!(hook.stages, vec!["commit".to_string()]);
    assert_eq!(hook.args, Vec::<String>::new());
    assert_eq!(hook.env, HashMap::new());
    assert_eq!(hook.version, None);
}