//! Integration tests for hook execution

use std::path::PathBuf;
use rustyhook::config::{Config, Hook, Repo};
use rustyhook::config::parser::{HookType, AccessMode};
use rustyhook::runner::{HookResolver, FileMatcher, HookContext, ParallelExecutor};

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
                        access_mode: AccessMode::ReadWrite,
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
        access_mode: AccessMode::ReadWrite,
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
                        access_mode: AccessMode::ReadWrite,
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

#[test]
fn test_skip_hooks() {
    // Create a temporary directory for the test
    let temp_dir = tempfile::tempdir().unwrap();
    let cache_dir = temp_dir.path().to_path_buf();

    // Create a test configuration with multiple hooks
    let config = Config {
        default_stages: vec!["commit".to_string()],
        fail_fast: false,
        parallelism: 0, // 0 means unlimited
        repos: vec![
            Repo {
                repo: "local".to_string(),
                hooks: vec![
                    Hook {
                        id: "hook1".to_string(),
                        name: "Hook 1".to_string(),
                        entry: "echo".to_string(),
                        language: "system".to_string(),
                        files: ".*\\.rs$".to_string(),
                        stages: vec!["commit".to_string()],
                        args: vec!["Hook 1".to_string()],
                        env: std::collections::HashMap::new(),
                        version: None,
                        hook_type: HookType::External,
                        separate_process: true,
                        access_mode: AccessMode::ReadWrite,
                    },
                    Hook {
                        id: "hook2".to_string(),
                        name: "Hook 2".to_string(),
                        entry: "echo".to_string(),
                        language: "system".to_string(),
                        files: ".*\\.rs$".to_string(),
                        stages: vec!["commit".to_string()],
                        args: vec!["Hook 2".to_string()],
                        env: std::collections::HashMap::new(),
                        version: None,
                        hook_type: HookType::External,
                        separate_process: true,
                        access_mode: AccessMode::ReadWrite,
                    },
                    Hook {
                        id: "hook3".to_string(),
                        name: "Hook 3".to_string(),
                        entry: "echo".to_string(),
                        language: "system".to_string(),
                        files: ".*\\.rs$".to_string(),
                        stages: vec!["commit".to_string()],
                        args: vec!["Hook 3".to_string()],
                        env: std::collections::HashMap::new(),
                        version: None,
                        hook_type: HookType::External,
                        separate_process: true,
                        access_mode: AccessMode::ReadWrite,
                    },
                ],
            },
        ],
    };

    // Create a hook resolver
    let mut resolver = HookResolver::new(config, cache_dir);

    // Set hooks to skip
    let hooks_to_skip = vec!["hook2".to_string()];
    resolver.set_hooks_to_skip(hooks_to_skip);

    // Check that the hooks_to_skip list is set correctly
    assert_eq!(resolver.hooks_to_skip().len(), 1);
    assert_eq!(resolver.hooks_to_skip()[0], "hook2");

    // Create some test files
    let files = vec![
        PathBuf::from("src/main.rs"),
        PathBuf::from("src/lib.rs"),
    ];

    // Run all hooks
    let result = resolver.run_all_hooks(&files);

    // Check that the hooks ran successfully
    assert!(result.is_ok());

    // We can't easily verify that hook2 was skipped in this test framework,
    // but the implementation in run_all_hooks should filter it out
}

#[test]
fn test_hook_context_execution() {
    // Create a hook that should run in a separate process (external hook)
    let external_hook = Hook {
        id: "external-hook".to_string(),
        name: "External Hook".to_string(),
        entry: "echo".to_string(),
        language: "system".to_string(),
        files: ".*\\.rs$".to_string(),
        stages: vec!["commit".to_string()],
        args: vec!["Hello, world!".to_string()],
        env: std::collections::HashMap::new(),
        version: None,
        hook_type: HookType::External,
        separate_process: false, // Even though this is false, it should run in a separate process because it's an external hook
        access_mode: AccessMode::ReadWrite,
    };

    // Create a hook that should run in a separate process (separate_process = true)
    let separate_process_hook = Hook {
        id: "separate-process-hook".to_string(),
        name: "Separate Process Hook".to_string(),
        entry: "echo".to_string(),
        language: "system".to_string(),
        files: ".*\\.rs$".to_string(),
        stages: vec!["commit".to_string()],
        args: vec!["Hello, world!".to_string()],
        env: std::collections::HashMap::new(),
        version: None,
        hook_type: HookType::BuiltIn,
        separate_process: true, // This should cause the hook to run in a separate process
        access_mode: AccessMode::ReadWrite,
    };

    // Create a hook that should run in the same process
    let same_process_hook = Hook {
        id: "same-process-hook".to_string(),
        name: "Same Process Hook".to_string(),
        entry: "echo".to_string(),
        language: "system".to_string(),
        files: ".*\\.rs$".to_string(),
        stages: vec!["commit".to_string()],
        args: vec!["Hello, world!".to_string()],
        env: std::collections::HashMap::new(),
        version: None,
        hook_type: HookType::BuiltIn,
        separate_process: false, // This should cause the hook to run in the same process
        access_mode: AccessMode::ReadWrite,
    };

    // Create a working directory and files to process
    let working_dir = std::env::current_dir().unwrap();
    let files_to_process = vec![
        PathBuf::from("src/main.rs"),
        PathBuf::from("src/lib.rs"),
    ];

    // Create hook contexts
    let external_context = HookContext::from_hook(&external_hook, working_dir.clone(), files_to_process.clone());
    let separate_process_context = HookContext::from_hook(&separate_process_hook, working_dir.clone(), files_to_process.clone());
    let same_process_context = HookContext::from_hook(&same_process_hook, working_dir.clone(), files_to_process.clone());

    // Test should_run_in_separate_process
    assert!(external_context.should_run_in_separate_process());
    assert!(separate_process_context.should_run_in_separate_process());
    assert!(!same_process_context.should_run_in_separate_process());

    // Test run_in_separate_process
    let result = external_context.run_in_separate_process();
    assert!(result.is_ok());

    let result = separate_process_context.run_in_separate_process();
    assert!(result.is_ok());

    // Test execute
    // For hooks that run in a separate process, we don't need to provide a tool
    let result = external_context.execute(None);
    assert!(result.is_ok());

    let result = separate_process_context.execute(None);
    assert!(result.is_ok());

    // For hooks that run in the same process, we need to provide a tool
    // Since we can't easily create a real tool for testing, we'll just test that it fails as expected
    let result = same_process_context.execute(None);
    assert!(result.is_err());
}

#[test]
fn test_parallel_execution() {
    // Create a temporary directory for the test
    let temp_dir = tempfile::tempdir().unwrap();
    let cache_dir = temp_dir.path().to_path_buf();

    // Create a test configuration with multiple hooks
    let config = Config {
        default_stages: vec!["commit".to_string()],
        fail_fast: false,
        parallelism: 2, // Limit to 2 parallel tasks
        repos: vec![
            Repo {
                repo: "local".to_string(),
                hooks: vec![
                    Hook {
                        id: "hook1".to_string(),
                        name: "Hook 1".to_string(),
                        entry: "echo".to_string(),
                        language: "system".to_string(),
                        files: ".*\\.rs$".to_string(),
                        stages: vec!["commit".to_string()],
                        args: vec!["Hook 1".to_string()],
                        env: std::collections::HashMap::new(),
                        version: None,
                        hook_type: HookType::External,
                        separate_process: true,
                        access_mode: AccessMode::ReadWrite,
                    },
                    Hook {
                        id: "hook2".to_string(),
                        name: "Hook 2".to_string(),
                        entry: "echo".to_string(),
                        language: "system".to_string(),
                        files: ".*\\.rs$".to_string(),
                        stages: vec!["commit".to_string()],
                        args: vec!["Hook 2".to_string()],
                        env: std::collections::HashMap::new(),
                        version: None,
                        hook_type: HookType::External,
                        separate_process: true,
                        access_mode: AccessMode::ReadWrite,
                    },
                    Hook {
                        id: "hook3".to_string(),
                        name: "Hook 3".to_string(),
                        entry: "echo".to_string(),
                        language: "system".to_string(),
                        files: ".*\\.rs$".to_string(),
                        stages: vec!["commit".to_string()],
                        args: vec!["Hook 3".to_string()],
                        env: std::collections::HashMap::new(),
                        version: None,
                        hook_type: HookType::External,
                        separate_process: true,
                        access_mode: AccessMode::ReadWrite,
                    },
                ],
            },
        ],
    };

    // Create a parallel executor
    let executor = ParallelExecutor::new(config, cache_dir);

    // Create some test files
    let files = vec![
        PathBuf::from("src/main.rs"),
        PathBuf::from("src/lib.rs"),
    ];

    // Create a tokio runtime for async execution
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Set hooks to skip
    let hooks_to_skip = vec!["hook2".to_string()];
    rt.block_on(executor.set_hooks_to_skip(hooks_to_skip));

    // Run all hooks in parallel
    let result = rt.block_on(executor.run_all_hooks(files));

    // Check that the hooks ran successfully
    assert!(result.is_ok());

    // We can't easily verify that hook2 was skipped or that hooks ran in parallel in this test framework,
    // but the implementation in ParallelExecutor should handle it correctly
}

#[test]
fn test_mutex_system() {
    use rustyhook::config::parser::AccessMode;
    
    // Create a temporary directory for the test
    let temp_dir = tempfile::tempdir().unwrap();
    let cache_dir = temp_dir.path().to_path_buf();

    // Create a test configuration with hooks of different access modes
    let config = Config {
        default_stages: vec!["commit".to_string()],
        fail_fast: false,
        parallelism: 0, // Unlimited parallelism
        repos: vec![
            Repo {
                repo: "local".to_string(),
                hooks: vec![
                    // Read-only hooks with different file patterns
                    Hook {
                        id: "read-hook1".to_string(),
                        name: "Read Hook 1".to_string(),
                        entry: "echo".to_string(),
                        language: "system".to_string(),
                        files: ".*\\.rs$".to_string(),
                        stages: vec!["commit".to_string()],
                        args: vec!["Read Hook 1".to_string()],
                        env: std::collections::HashMap::new(),
                        version: None,
                        hook_type: HookType::External,
                        separate_process: true,
                        access_mode: AccessMode::Read,
                    },
                    Hook {
                        id: "read-hook2".to_string(),
                        name: "Read Hook 2".to_string(),
                        entry: "echo".to_string(),
                        language: "system".to_string(),
                        files: ".*\\.py$".to_string(),
                        stages: vec!["commit".to_string()],
                        args: vec!["Read Hook 2".to_string()],
                        env: std::collections::HashMap::new(),
                        version: None,
                        hook_type: HookType::External,
                        separate_process: true,
                        access_mode: AccessMode::Read,
                    },
                    // Read-write hooks with different file patterns
                    Hook {
                        id: "write-hook1".to_string(),
                        name: "Write Hook 1".to_string(),
                        entry: "echo".to_string(),
                        language: "system".to_string(),
                        files: ".*\\.rs$".to_string(),
                        stages: vec!["commit".to_string()],
                        args: vec!["Write Hook 1".to_string()],
                        env: std::collections::HashMap::new(),
                        version: None,
                        hook_type: HookType::External,
                        separate_process: true,
                        access_mode: AccessMode::ReadWrite,
                    },
                    Hook {
                        id: "write-hook2".to_string(),
                        name: "Write Hook 2".to_string(),
                        entry: "echo".to_string(),
                        language: "system".to_string(),
                        files: ".*\\.py$".to_string(),
                        stages: vec!["commit".to_string()],
                        args: vec!["Write Hook 2".to_string()],
                        env: std::collections::HashMap::new(),
                        version: None,
                        hook_type: HookType::External,
                        separate_process: true,
                        access_mode: AccessMode::ReadWrite,
                    },
                    // Another read-write hook with the same file pattern as write-hook1
                    Hook {
                        id: "write-hook3".to_string(),
                        name: "Write Hook 3".to_string(),
                        entry: "echo".to_string(),
                        language: "system".to_string(),
                        files: ".*\\.rs$".to_string(),
                        stages: vec!["commit".to_string()],
                        args: vec!["Write Hook 3".to_string()],
                        env: std::collections::HashMap::new(),
                        version: None,
                        hook_type: HookType::External,
                        separate_process: true,
                        access_mode: AccessMode::ReadWrite,
                    },
                ],
            },
        ],
    };

    // Create a parallel executor
    let executor = ParallelExecutor::new(config, cache_dir);

    // Create some test files
    let files = vec![
        PathBuf::from("src/main.rs"),
        PathBuf::from("src/lib.rs"),
        PathBuf::from("src/main.py"),
    ];

    // Create a tokio runtime for async execution
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Run all hooks in parallel
    let result = rt.block_on(executor.run_all_hooks(files));

    // Check that the hooks ran successfully
    assert!(result.is_ok());

    // We can't easily verify the exact execution order in this test framework,
    // but the implementation in ParallelExecutor should:
    // 1. Run all read-only hooks in parallel
    // 2. Group read-write hooks by their file patterns
    // 3. Run read-write hooks in parallel only if their file patterns don't overlap
}