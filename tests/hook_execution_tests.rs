//! Integration tests for hook execution

use std::path::PathBuf;
use rustyhook::config::{Config, Hook, Repo};
use rustyhook::runner::{HookResolver, FileMatcher};

#[test]
fn test_file_matcher() {
    // Create a file matcher with a regex pattern
    let matcher = FileMatcher::from_regex(r".*\.rs$").unwrap();

    // Create some test files
    let files = vec![
        PathBuf::from("src/main.rs"),
        PathBuf::from("src/lib.rs"),
        PathBuf::from("src/main.py"),
    ];

    // Filter the files
    let filtered = matcher.filter_files(&files);

    // Check the filtered files
    assert_eq!(filtered.len(), 2);
    assert!(filtered.contains(&PathBuf::from("src/main.rs")));
    assert!(filtered.contains(&PathBuf::from("src/lib.rs")));
    assert!(!filtered.contains(&PathBuf::from("src/main.py")));
}

#[test]
fn test_hook_resolver() {
    // Create a temporary directory for the test
    let temp_dir = tempfile::tempdir().unwrap();
    let cache_dir = temp_dir.path().to_path_buf();

    // Create a test configuration
    let config = Config {
        default_stages: vec!["commit".to_string()],
        fail_fast: false,
        parallelism: 0, // 0 means unlimited
        repos: vec![
            Repo {
                repo: "local".to_string(),
                hooks: vec![
                    Hook {
                        id: "test-hook".to_string(),
                        name: "Test Hook".to_string(),
                        entry: "echo".to_string(),
                        language: "system".to_string(),
                        files: ".*\\.rs$".to_string(),
                        stages: vec!["commit".to_string()],
                        args: vec!["Hello, world!".to_string()],
                        env: std::collections::HashMap::new(),
                        version: None,
                    },
                ],
            },
        ],
    };

    // Create a hook resolver
    let resolver = HookResolver::new(config, cache_dir);

    // Check that the resolver was created successfully
    assert!(resolver.config().repos.len() == 1);
    assert!(resolver.config().repos[0].hooks.len() == 1);
    assert_eq!(resolver.config().repos[0].hooks[0].id, "test-hook");
}
