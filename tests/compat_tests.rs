//! Compatibility tests with pre-commit configs

use std::fs;
use std::path::PathBuf;
use rustyhook::config::{
    PreCommitConfig, PreCommitRepo, PreCommitHook,
    parse_precommit_config, convert_to_rustyhook_config
};
use rustyhook::config::compat::{PreCommitHooksFile, find_precommit_hooks_for_repo};
use rustyhook::config::parser::HookType;

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
                        name: Some("trailing-whitespace".to_string()),
                        entry: Some("trailing-whitespace".to_string()),
                        language: Some("python".to_string()),
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
    println!("Hook ID: {}", hook.id);
    println!("Hook Name: {}", hook.name);
    println!("Hook Language: {}", hook.language);
    println!("Hook Entry: {}", hook.entry);
    println!("Hook Type: {:?}", hook.hook_type);

    assert_eq!(hook.id, "trailing-whitespace");
    assert_eq!(hook.name, "trailing-whitespace");
    assert_eq!(hook.language, "python");
    assert_eq!(hook.entry, "trailing-whitespace");
    assert_eq!(hook.hook_type, HookType::BuiltIn);
}

#[test]
fn test_fetch_precommit_hooks_file() {
    // Test fetching hooks from a real repository
    // This test requires internet connection and might fail if the repository structure changes
    let hooks_file = find_precommit_hooks_for_repo("https://github.com/pre-commit/pre-commit-hooks");

    // Verify that we got a hooks file
    assert!(hooks_file.is_some(), "Failed to fetch hooks file from pre-commit-hooks repository");

    let hooks = hooks_file.unwrap();

    // Verify that the hooks file contains some expected hooks
    // We don't check all hooks as they might change over time
    let hook_ids: Vec<String> = hooks.hooks.iter().map(|h| h.id.clone()).collect();

    // Check for some common hooks that should be present
    assert!(hook_ids.contains(&"trailing-whitespace".to_string()), 
            "Hooks file should contain trailing-whitespace hook");

    // Verify that hooks have the expected structure
    let trailing_whitespace = hooks.hooks.iter()
        .find(|h| h.id == "trailing-whitespace")
        .expect("trailing-whitespace hook should exist");

    assert_eq!(trailing_whitespace.language, "python");
    assert!(trailing_whitespace.entry.contains("trailing-whitespace"));
}

#[test]
fn test_repository_cloned_to_cache_directory() {
    // Clean up any existing cache directory for this test
    let repo_url = "https://github.com/pre-commit/pre-commit-hooks";
    let cache_dir = std::env::current_dir().unwrap().join(".rustyhook").join("cache").join("repos");

    // Create a hash of the repo URL to find the expected directory
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    repo_url.hash(&mut hasher);
    let repo_hash = hasher.finish();

    let repo_dir = cache_dir.join(format!("{}", repo_hash));

    // Remove the directory if it exists
    if repo_dir.exists() {
        std::fs::remove_dir_all(&repo_dir).unwrap();
    }

    // Fetch the hooks file, which should clone the repository to the cache directory
    let hooks_file = find_precommit_hooks_for_repo(repo_url);

    // Verify that we got a hooks file
    assert!(hooks_file.is_some(), "Failed to fetch hooks file from pre-commit-hooks repository");

    // Verify that the repository was cloned to the cache directory
    assert!(repo_dir.exists(), "Repository should be cloned to the cache directory");
    assert!(repo_dir.join(".git").exists(), "Repository directory should contain a .git directory");
    assert!(repo_dir.join(".pre-commit-hooks.yaml").exists(), "Repository directory should contain a .pre-commit-hooks.yaml file");

    // Fetch the hooks file again, which should use the cached repository
    let hooks_file2 = find_precommit_hooks_for_repo(repo_url);

    // Verify that we got a hooks file
    assert!(hooks_file2.is_some(), "Failed to fetch hooks file from pre-commit-hooks repository (second time)");

    // The hooks files should be the same
    let hooks1 = hooks_file.unwrap();
    let hooks2 = hooks_file2.unwrap();

    assert_eq!(hooks1.hooks.len(), hooks2.hooks.len(), "Hook counts should be the same");
}

#[test]
fn test_convert_to_rustyhook_config_with_multiple_repos() {
    // Create a pre-commit configuration with multiple repositories
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
                        name: Some("trailing-whitespace".to_string()),
                        entry: Some("trailing-whitespace".to_string()),
                        language: Some("python".to_string()),
                        files: None,
                        stages: None,
                        args: None,
                        env: None,
                    },
                ],
            },
            PreCommitRepo {
                repo: "https://github.com/astral-sh/ruff-pre-commit".to_string(),
                rev: "v0.0.262".to_string(),
                hooks: vec![
                    PreCommitHook {
                        id: "ruff".to_string(),
                        name: Some("ruff".to_string()),
                        entry: Some("ruff".to_string()),
                        language: Some("python".to_string()),
                        files: None,
                        stages: None,
                        args: None,
                        env: None,
                    },
                ],
            },
            PreCommitRepo {
                repo: "https://github.com/biomejs/pre-commit".to_string(),
                rev: "v1.0.0".to_string(),
                hooks: vec![
                    PreCommitHook {
                        id: "biome-check".to_string(),
                        name: Some("biome-check".to_string()),
                        entry: Some("biome check".to_string()),
                        language: Some("node".to_string()),
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
    assert_eq!(rustyhook_config.repos.len(), 3);

    // Check the first repository (pre-commit-hooks)
    let repo1 = &rustyhook_config.repos[0];
    assert_eq!(repo1.repo, "https://github.com/pre-commit/pre-commit-hooks");
    assert_eq!(repo1.hooks.len(), 1);

    // Check the hook in the first repository
    let hook1 = &repo1.hooks[0];
    assert_eq!(hook1.id, "trailing-whitespace");
    assert_eq!(hook1.name, "trailing-whitespace");
    assert_eq!(hook1.language, "python");
    assert_eq!(hook1.entry, "trailing-whitespace");
    assert_eq!(hook1.hook_type, HookType::BuiltIn);

    // Check the second repository (ruff)
    let repo2 = &rustyhook_config.repos[1];
    assert_eq!(repo2.repo, "https://github.com/astral-sh/ruff-pre-commit");
    assert_eq!(repo2.hooks.len(), 1);

    // Check the hook in the second repository
    let hook2 = &repo2.hooks[0];
    assert_eq!(hook2.id, "ruff");
    assert_eq!(hook2.name, "ruff");
    assert_eq!(hook2.language, "python");
    assert_eq!(hook2.entry, "ruff");
    // Since the entry matches the ID, it should be a built-in hook according to our new logic
    assert_eq!(hook2.hook_type, HookType::BuiltIn);

    // Check the third repository (biome)
    let repo3 = &rustyhook_config.repos[2];
    assert_eq!(repo3.repo, "https://github.com/biomejs/pre-commit");
    assert_eq!(repo3.hooks.len(), 1);

    // Check the hook in the third repository
    let hook3 = &repo3.hooks[0];
    assert_eq!(hook3.id, "biome-check");
    assert_eq!(hook3.name, "biome-check");
    assert_eq!(hook3.language, "node");
    assert_eq!(hook3.entry, "biome check");
    assert_eq!(hook3.hook_type, HookType::External);
}
