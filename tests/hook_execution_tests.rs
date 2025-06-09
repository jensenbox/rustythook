//! Integration tests for hook execution

use std::path::PathBuf;
use rustyhook::config::{Config, Hook, Repo};
use rustyhook::config::parser::HookType;
use rustyhook::runner::{HookResolver, FileMatcher, HookContext};

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
                        hook_type: HookType::External,
                        separate_process: false,
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

#[test]
fn test_hook_context() {
    // Create a hook
    let hook = Hook {
        id: "test-hook".to_string(),
        name: "Test Hook".to_string(),
        entry: "echo".to_string(),
        language: "system".to_string(),
        files: ".*\\.rs$".to_string(),
        stages: vec!["commit".to_string()],
        args: vec!["Hello, world!".to_string()],
        env: std::collections::HashMap::new(),
        version: None,
        hook_type: HookType::External,
        separate_process: true,
    };

    // Create a working directory and files to process
    let working_dir = std::env::current_dir().unwrap();
    let files_to_process = vec![
        PathBuf::from("src/main.rs"),
        PathBuf::from("src/lib.rs"),
    ];

    // Create a hook context
    let context = HookContext::from_hook(&hook, working_dir, files_to_process.clone());

    // Check that the context was created correctly
    assert_eq!(context.id, "test-hook");
    assert_eq!(context.name, "Test Hook");
    assert_eq!(context.entry, "echo");
    assert_eq!(context.language, "system");
    assert_eq!(context.files, ".*\\.rs$");
    assert_eq!(context.stages, vec!["commit".to_string()]);
    assert_eq!(context.args, vec!["Hello, world!".to_string()]);
    assert_eq!(context.env.len(), 0);
    assert_eq!(context.version, None);
    assert_eq!(context.hook_type, HookType::External);
    assert_eq!(context.separate_process, true);
    assert_eq!(context.files_to_process, files_to_process);
}

#[test]
fn test_run_hook_in_separate_process() {
    // Create a temporary directory for the test
    let temp_dir = tempfile::tempdir().unwrap();
    let cache_dir = temp_dir.path().to_path_buf();

    // Create a test configuration with an external hook that runs in a separate process
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
                        hook_type: HookType::External,
                        separate_process: true,
                    },
                ],
            },
        ],
    };

    // Create a hook resolver
    let mut resolver = HookResolver::new(config, cache_dir);

    // Create some test files
    let files = vec![
        PathBuf::from("src/main.rs"),
        PathBuf::from("src/lib.rs"),
    ];

    // Run the hook
    let result = resolver.run_hook("local", "test-hook", &files);

    // Check that the hook ran successfully
    assert!(result.is_ok());
}
