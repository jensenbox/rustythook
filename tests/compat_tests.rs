//! Compatibility tests with pre-commit configs

use std::fs;
use std::path::PathBuf;
use rustyhook::config::{
    PreCommitConfig, PreCommitRepo, PreCommitHook,
    parse_precommit_config, convert_to_rustyhook_config
};

#[test]
fn test_parse_precommit_config() {
    // Create a temporary directory for the test
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join(".pre-commit-config.yaml");
    
    // Create a test pre-commit configuration
    let config_str = r#"
repos:
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v4.4.0
    hooks:
      - id: trailing-whitespace
      - id: end-of-file-fixer
      - id: check-yaml
      - id: check-added-large-files
  - repo: https://github.com/psf/black
    rev: 23.3.0
    hooks:
      - id: black
"#;
    
    // Write the configuration to a file
    fs::write(&config_path, config_str).unwrap();
    
    // Parse the configuration
    let config = parse_precommit_config(&config_path).unwrap();
    
    // Check the configuration
    assert_eq!(config.repos.len(), 2);
    
    // Check the first repository
    let repo1 = &config.repos[0];
    assert_eq!(repo1.repo, "https://github.com/pre-commit/pre-commit-hooks");
    assert_eq!(repo1.rev, "v4.4.0");
    assert_eq!(repo1.hooks.len(), 4);
    
    // Check the hooks in the first repository
    assert_eq!(repo1.hooks[0].id, "trailing-whitespace");
    assert_eq!(repo1.hooks[1].id, "end-of-file-fixer");
    assert_eq!(repo1.hooks[2].id, "check-yaml");
    assert_eq!(repo1.hooks[3].id, "check-added-large-files");
    
    // Check the second repository
    let repo2 = &config.repos[1];
    assert_eq!(repo2.repo, "https://github.com/psf/black");
    assert_eq!(repo2.rev, "23.3.0");
    assert_eq!(repo2.hooks.len(), 1);
    
    // Check the hook in the second repository
    assert_eq!(repo2.hooks[0].id, "black");
}

#[test]
fn test_convert_to_rustyhook_config() {
    // Create a pre-commit configuration
    let precommit_config = PreCommitConfig {
        default_stages: vec!["commit".to_string()],
        fail_fast: false,
        repos: vec![
            PreCommitRepo {
                repo: "https://github.com/pre-commit/pre-commit-hooks".to_string(),
                rev: "v4.4.0".to_string(),
                hooks: vec![
                    PreCommitHook {
                        id: "trailing-whitespace".to_string(),
                        name: None,
                        entry: None,
                        language: None,
                        files: None,
                        stages: None,
                        args: None,
                        env: None,
                    },
                ],
            },
        ],
    };
    
    // Convert to RustyHook configuration
    let rustyhook_config = convert_to_rustyhook_config(&precommit_config);
    
    // Check the configuration
    assert_eq!(rustyhook_config.default_stages, vec!["commit".to_string()]);
    assert_eq!(rustyhook_config.fail_fast, false);
    assert_eq!(rustyhook_config.repos.len(), 1);
    
    // Check the repository
    let repo = &rustyhook_config.repos[0];
    assert_eq!(repo.repo, "https://github.com/pre-commit/pre-commit-hooks");
    assert_eq!(repo.hooks.len(), 1);
    
    // Check the hook
    let hook = &repo.hooks[0];
    assert_eq!(hook.id, "trailing-whitespace");
    assert_eq!(hook.name, "trailing-whitespace");
    assert_eq!(hook.entry, "pre-commit-hooks trailing-whitespace");
    assert_eq!(hook.language, "system");
}