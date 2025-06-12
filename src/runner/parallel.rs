//! Parallel execution for RustyHook
//!
//! This module provides functionality for running hooks in parallel.

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinSet;
use std::collections::HashMap;

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
        let working_dir = std::env::current_dir().map_err(HookResolverError::IoError)?;

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

        // Run hooks with parallelism limit
        if parallelism > 0 {
            // Limited parallelism
            println!("Running hooks with parallelism limit of {}", parallelism);

            // Process hooks in batches
            for chunk in hook_contexts.chunks(parallelism) {
                // Spawn tasks for this batch
                for (repo_id, hook_id, hook, filtered_files) in chunk {
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
            }
        } else {
            // Unlimited parallelism
            println!("Running hooks with unlimited parallelism");

            // Run each hook in parallel
            for (repo_id, hook_id, hook, filtered_files) in hook_contexts {
                // Clone the necessary data for the task
                let resolver = Arc::clone(&self.resolver);
                let tool_cache = Arc::clone(&self.tool_cache);

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

            // Wait for all tasks to complete
            while let Some(result) = tasks.join_next().await {
                result??;
            }
        }

        Ok(())
    }
}
