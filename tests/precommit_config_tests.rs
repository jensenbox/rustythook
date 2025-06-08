//! Tests for the pre-commit configuration compatibility

use std::path::PathBuf;
use std::fs;
use std::env;
use rustyhook::config::{find_precommit_config, convert_to_rustyhook_config};

#[test]
fn test_find_precommit_config() {
    // The test should find the .pre-commit-config.yaml file in the project root
    let result = find_precommit_config();
    assert!(result.is_ok());
    
    let config_path = result.unwrap();
    assert!(config_path.exists());
    assert_eq!(config_path.file_name().unwrap(), ".pre-commit-config.yaml");
}

#[test]
fn test_convert_to_rustyhook_config() {
    // Find the pre-commit config
    let precommit_config_path = find_precommit_config().unwrap();
    
    // Read the pre-commit config
    let precommit_config_str = fs::read_to_string(&precommit_config_path).unwrap();
    
    // Convert to RustyHook config
    let rustyhook_config = convert_to_rustyhook_config(&precommit_config_str);
    
    // Check that the conversion was successful
    assert_eq!(rustyhook_config.default_stages, vec!["commit".to_string(), "push".to_string()]);
    assert_eq!(rustyhook_config.fail_fast, true);
    
    // Check that all repositories were converted
    assert!(rustyhook_config.repos.len() >= 5); // We have 5 repos in our test config
    
    // Check that the pre-commit-hooks repo was converted correctly
    let precommit_hooks_repo = rustyhook_config.repos.iter()
        .find(|repo| repo.repo == "https://github.com/pre-commit/pre-commit-hooks")
        .expect("pre-commit-hooks repo not found");
    
    assert_eq!(precommit_hooks_repo.hooks.len(), 4); // We have 4 hooks in this repo
    
    // Check that the trailing-whitespace hook was converted correctly
    let trailing_whitespace_hook = precommit_hooks_repo.hooks.iter()
        .find(|hook| hook.id == "trailing-whitespace")
        .expect("trailing-whitespace hook not found");
    
    assert_eq!(trailing_whitespace_hook.name, "Trim Trailing Whitespace");
    assert_eq!(trailing_whitespace_hook.stages, vec!["commit".to_string()]);
    
    // Check that the Rust repo was converted correctly
    let rust_repo = rustyhook_config.repos.iter()
        .find(|repo| repo.repo == "https://github.com/doublify/pre-commit-rust")
        .expect("Rust repo not found");
    
    assert_eq!(rust_repo.hooks.len(), 2); // We have 2 hooks in this repo
    
    // Check that the fmt hook was converted correctly
    let fmt_hook = rust_repo.hooks.iter()
        .find(|hook| hook.id == "fmt")
        .expect("fmt hook not found");
    
    assert_eq!(fmt_hook.name, "Rust Formatter");
    assert_eq!(fmt_hook.language, "rust");
    
    // Check that the local repo was converted correctly
    let local_repo = rustyhook_config.repos.iter()
        .find(|repo| repo.repo == "local")
        .expect("local repo not found");
    
    assert_eq!(local_repo.hooks.len(), 2); // We have 2 hooks in this repo
    
    // Check that the custom-python-script hook was converted correctly
    let custom_python_hook = local_repo.hooks.iter()
        .find(|hook| hook.id == "custom-python-script")
        .expect("custom-python-script hook not found");
    
    assert_eq!(custom_python_hook.name, "Custom Python Script");
    assert_eq!(custom_python_hook.language, "python");
    assert!(custom_python_hook.additional_dependencies.contains(&"requests==2.28.2".to_string()));
}

#[test]
fn test_convert_from_precommit() {
    // Create a temporary directory for the test
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = env::current_dir().unwrap();
    
    // Copy the .pre-commit-config.yaml file to the temporary directory
    let precommit_config_path = original_dir.join(".pre-commit-config.yaml");
    let temp_precommit_config_path = temp_dir.path().join(".pre-commit-config.yaml");
    fs::copy(&precommit_config_path, &temp_precommit_config_path).unwrap();
    
    // Change to the temporary directory
    env::set_current_dir(&temp_dir).unwrap();
    
    // Convert from pre-commit config
    let result = rustyhook::config::convert_from_precommit::<&str>(None, None);
    assert!(result.is_ok());
    
    // Check that the .rustyhook/config.yaml file was created
    let rustyhook_config_path = temp_dir.path().join(".rustyhook").join("config.yaml");
    assert!(rustyhook_config_path.exists());
    
    // Read the RustyHook config
    let rustyhook_config_str = fs::read_to_string(&rustyhook_config_path).unwrap();
    
    // Check that the conversion was successful
    assert!(rustyhook_config_str.contains("default_stages: [commit, push]"));
    assert!(rustyhook_config_str.contains("fail_fast: true"));
    assert!(rustyhook_config_str.contains("repo: https://github.com/pre-commit/pre-commit-hooks"));
    assert!(rustyhook_config_str.contains("id: trailing-whitespace"));
    assert!(rustyhook_config_str.contains("repo: https://github.com/doublify/pre-commit-rust"));
    assert!(rustyhook_config_str.contains("id: fmt"));
    assert!(rustyhook_config_str.contains("repo: local"));
    assert!(rustyhook_config_str.contains("id: custom-python-script"));
    
    // Change back to the original directory
    env::set_current_dir(original_dir).unwrap();
}