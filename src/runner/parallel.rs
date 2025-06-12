//! Parallel execution for RustyHook
//!
//! This module provides functionality for running hooks in parallel.

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinSet;
use std::collections::HashMap;

use crate::config::parser::AccessMode;

use crate::config::{Config, Hook};
use crate::toolchains::Tool;
use super::hook_resolver::{HookResolver, HookResolverError};
use super::file_matcher::FileMatcher;
use super::hook_context::HookContext;

/// Error type for parallel execution operations
#[derive(Debug)]
pub enum ParallelExecutionError {
    /// Error with hook resolver
    HookResolverError(HookResolverError),
    /// Error with tokio
    TokioError(tokio::task::JoinError),
}

impl From<HookResolverError> for ParallelExecutionError {
    fn from(err: HookResolverError) -> Self {
        ParallelExecutionError::HookResolverError(err)
    }
}

impl From<tokio::task::JoinError> for ParallelExecutionError {
    fn from(err: tokio::task::JoinError) -> Self {
        ParallelExecutionError::TokioError(err)
    }
}

impl std::fmt::Display for ParallelExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParallelExecutionError::HookResolverError(err) => write!(f, "{}", err),
            ParallelExecutionError::TokioError(err) => write!(f, "Task execution error: {}", err),
        }
    }
}

impl std::error::Error for ParallelExecutionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParallelExecutionError::HookResolverError(err) => Some(err),
            ParallelExecutionError::TokioError(err) => Some(err),
        }
    }
}

/// Represents a parallel executor
pub struct ParallelExecutor {
    /// Hook resolver
    resolver: Arc<Mutex<HookResolver>>,
    /// Thread-safe tool cache
    tool_cache: Arc<RwLock<HashMap<String, Arc<Box<dyn Tool + Send + Sync>>>>>,
}

impl ParallelExecutor {
    /// Create a new parallel executor
    pub fn new(config: Config, cache_dir: PathBuf) -> Self {
        let resolver = HookResolver::new(config, cache_dir);
        ParallelExecutor {
            resolver: Arc::new(Mutex::new(resolver)),
            tool_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set hooks to skip
    pub async fn set_hooks_to_skip(&self, hooks: Vec<String>) {
        let mut resolver = self.resolver.lock().await;
        resolver.set_hooks_to_skip(hooks);
    }

    /// Prepare hook contexts for parallel execution
    async fn prepare_hook_contexts(&self, files: &[PathBuf]) -> Result<Vec<(String, String, Hook, Vec<PathBuf>)>, ParallelExecutionError> {
        // Acquire the lock and get a reference to the resolver
        let resolver_guard = self.resolver.lock().await;

        // Clone the config to get an owned copy that doesn't depend on the resolver
        let config = resolver_guard.config().clone();

        // Get the hooks to skip
        let hooks_to_skip = resolver_guard.hooks_to_skip().clone();

        // Release the lock
        drop(resolver_guard);

        // Collect all hooks to run, excluding those that should be skipped
        let mut hook_contexts = Vec::new();
        for repo in &config.repos {
            for hook in &repo.hooks {
                if !hooks_to_skip.contains(&hook.id) {
                    // Filter files based on the hook's file pattern
                    let filtered_files = if !hook.files.is_empty() {
                        match FileMatcher::from_regex(&hook.files) {
                            Ok(matcher) => matcher.filter_files(files),
                            Err(err) => return Err(ParallelExecutionError::HookResolverError(err.into())),
                        }
                    } else {
                        files.to_vec()
                    };

                    // Skip hooks with no matching files
                    if !filtered_files.is_empty() {
                        hook_contexts.push((repo.repo.clone(), hook.id.clone(), hook.clone(), filtered_files));
                    }
                }
            }
        }

        Ok(hook_contexts)
    }

    /// Run a hook with the prepared context
    async fn run_hook_with_context(
        resolver: Arc<Mutex<HookResolver>>,
        _tool_cache: Arc<RwLock<HashMap<String, Arc<Box<dyn Tool + Send + Sync>>>>>,
        repo_id: &str,
        hook_id: &str,
        hook: &Hook,
        files: &[PathBuf]
    ) -> Result<(), HookResolverError> {
        // If there are no files to process, we're done
        if files.is_empty() {
            return Ok(());
        }

        // Get the current working directory
        let working_dir = std::env::current_dir().map_err(|err| {
            HookResolverError::FileNotFound {
                path: PathBuf::from("."),
                context: format!("Failed to access current working directory when running hook '{}': {}", hook_id, err)
            }
        })?;

        // Create the context for running the hook
        let context = HookContext::from_hook(hook, working_dir, files.to_vec());

        // Use the context to decide how to run the hook
        if context.should_run_in_separate_process() {
            // Run the hook in a separate process using the context
            context.run_in_separate_process().map_err(|err| match err {
                super::hook_context::HookContextError::ProcessError(msg) => HookResolverError::ProcessError(msg),
                super::hook_context::HookContextError::IoError(err) => HookResolverError::IoError(err),
                super::hook_context::HookContextError::HookError(err) => HookResolverError::HookError(err),
                super::hook_context::HookContextError::ToolError(err) => HookResolverError::ToolError(err),
                super::hook_context::HookContextError::CommandNotFound { command, hook_id, error: _ } => {
                    HookResolverError::FileNotFound {
                        path: PathBuf::from(command),
                        context: format!("Command not found when running hook '{}'. Make sure the command is installed and available in your PATH.", hook_id)
                    }
                }
            })
        } else {
            // Instead of using the tool cache or setup_tool, use run_hook directly
            // This avoids the trait bound error and the private method issue
            let mut resolver_guard = resolver.lock().await;
            resolver_guard.run_hook(repo_id, hook_id, files)
        }
    }

    /// Run all hooks on files in parallel
    pub async fn run_all_hooks(&self, files: Vec<PathBuf>) -> Result<(), ParallelExecutionError> {
        // Prepare all hook contexts upfront to minimize mutex contention
        let hook_contexts = self.prepare_hook_contexts(&files).await?;

        // Get the parallelism limit from the config
        let parallelism = {
            let resolver_guard = self.resolver.lock().await;
            resolver_guard.config().parallelism
        };

        // Create a JoinSet to collect all tasks
        let mut tasks = JoinSet::new();

        // Separate hooks into read-only and read-write groups
        let mut read_hooks = Vec::new();
        let mut write_hooks = Vec::new();

        for context in hook_contexts {
            if context.2.access_mode == AccessMode::Read {
                read_hooks.push(context);
            } else {
                write_hooks.push(context);
            }
        }

        // Run read-only hooks first (they can all run in parallel)
        println!("Running {} read-only hooks", read_hooks.len());

        // Apply parallelism limit if configured
        if parallelism > 0 {
            // Process read hooks in batches
            for chunk in read_hooks.chunks(parallelism) {
                self.run_hook_batch(chunk, &mut tasks).await?;
            }
        } else {
            // Run all read hooks in parallel
            self.run_hook_batch(&read_hooks, &mut tasks).await?;
        }

        // Group read-write hooks by their file globs to avoid conflicts
        println!("Running {} read-write hooks", write_hooks.len());

        if write_hooks.is_empty() {
            return Ok(());
        }

        // Create groups of non-overlapping hooks
        let mut hook_groups: Vec<Vec<(String, String, Hook, Vec<PathBuf>)>> = Vec::new();

        // Helper function to check if two hooks have overlapping file patterns
        let hooks_overlap = |hook1: &Hook, hook2: &Hook| -> bool {
            // If either hook has an empty files pattern, assume they overlap
            if hook1.files.is_empty() || hook2.files.is_empty() {
                return true;
            }

            // If the file patterns are different, assume they don't overlap
            // This is a simplification - in a real implementation, we would need to check
            // if the regex patterns could match the same files
            hook1.files == hook2.files
        };

        // Group hooks that don't overlap
        for (repo_id, hook_id, hook, filtered_files) in write_hooks {
            // Try to find a group where this hook doesn't overlap with any hook
            let mut found_group = false;

            for group in &mut hook_groups {
                let mut can_add_to_group = true;

                // Check if this hook overlaps with any hook in the group
                for (_, _, existing_hook, _) in group.iter() {
                    if hooks_overlap(existing_hook, &hook) {
                        can_add_to_group = false;
                        break;
                    }
                }

                if can_add_to_group {
                    group.push((repo_id.clone(), hook_id.clone(), hook.clone(), filtered_files.clone()));
                    found_group = true;
                    break;
                }
            }

            // If no suitable group was found, create a new group
            if !found_group {
                hook_groups.push(vec![(repo_id, hook_id, hook, filtered_files)]);
            }
        }

        // Run each group of non-overlapping hooks in parallel
        for (i, group) in hook_groups.iter().enumerate() {
            println!("Running group {} of {} non-overlapping read-write hooks", i + 1, group.len());

            if parallelism > 0 {
                // Process hooks in batches
                for chunk in group.chunks(parallelism) {
                    self.run_hook_batch(chunk, &mut tasks).await?;
                }
            } else {
                // Run all hooks in this group in parallel
                self.run_hook_batch(group, &mut tasks).await?;
            }
        }

        Ok(())
    }

    /// Run a batch of hooks in parallel
    async fn run_hook_batch(
        &self,
        hooks: &[(String, String, Hook, Vec<PathBuf>)],
        tasks: &mut JoinSet<Result<(), ParallelExecutionError>>
    ) -> Result<(), ParallelExecutionError> {
        // Spawn tasks for this batch
        for (repo_id, hook_id, hook, filtered_files) in hooks {
            // Clone the necessary data for the task
            let resolver = Arc::clone(&self.resolver);
            let tool_cache = Arc::clone(&self.tool_cache);
            let repo_id = repo_id.clone();
            let hook_id = hook_id.clone();
            let hook = hook.clone();
            let filtered_files = filtered_files.clone();

            // Spawn a task to run the hook
            tasks.spawn(async move {
                Self::run_hook_with_context(
                    resolver,
                    tool_cache,
                    &repo_id,
                    &hook_id,
                    &hook,
                    &filtered_files
                ).await.map_err(ParallelExecutionError::from)
            });
        }

        // Wait for all tasks in this batch to complete
        while tasks.len() > 0 {
            let result = tasks.join_next().await.unwrap();
            result??;
        }

        Ok(())
    }
}
