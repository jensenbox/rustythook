//! Parallel execution for RustyHook
//!
//! This module provides functionality for running hooks in parallel.

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinSet;

use crate::config::Config;
use super::hook_resolver::{HookResolver, HookResolverError};

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
}

impl ParallelExecutor {
    /// Create a new parallel executor
    pub fn new(config: Config, cache_dir: PathBuf) -> Self {
        let resolver = HookResolver::new(config, cache_dir);
        ParallelExecutor {
            resolver: Arc::new(Mutex::new(resolver)),
        }
    }

    /// Run all hooks on files in parallel
    pub async fn run_all_hooks(&self, files: Vec<PathBuf>) -> Result<(), ParallelExecutionError> {
        // Get the configuration from the resolver
        // First, acquire the lock and get a reference to the resolver
        let resolver_guard = self.resolver.lock().await;
        // Clone the config to get an owned copy that doesn't depend on the resolver
        let config = resolver_guard.config();
        // The lock is released here when resolver_guard goes out of scope

        // Get the parallelism limit from the config
        let parallelism = config.parallelism;

        // Create a JoinSet to collect all tasks
        let mut tasks = JoinSet::new();

        // Collect all hooks to run
        let mut hooks_to_run = Vec::new();
        for repo in &config.repos {
            for hook in &repo.hooks {
                hooks_to_run.push((repo.repo.clone(), hook.id.clone()));
            }
        }

        // Run hooks with parallelism limit
        if parallelism > 0 {
            // Limited parallelism
            println!("Running hooks with parallelism limit of {}", parallelism);

            // Process hooks in batches
            for chunk in hooks_to_run.chunks(parallelism) {
                // Spawn tasks for this batch
                for (repo_id, hook_id) in chunk {
                    // Clone the necessary data for the task
                    let resolver = Arc::clone(&self.resolver);
                    let repo_id = repo_id.clone();
                    let hook_id = hook_id.clone();
                    let files = files.clone();

                    // Spawn a task to run the hook
                    tasks.spawn(async move {
                        let mut resolver = resolver.lock().await;
                        resolver.run_hook(&repo_id, &hook_id, &files).map_err(ParallelExecutionError::from)
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

            // Run each hook in each repository in parallel
            for (repo_id, hook_id) in hooks_to_run {
                // Clone the necessary data for the task
                let resolver = Arc::clone(&self.resolver);
                let files = files.clone();

                // Spawn a task to run the hook
                tasks.spawn(async move {
                    let mut resolver = resolver.lock().await;
                    resolver.run_hook(&repo_id, &hook_id, &files).map_err(ParallelExecutionError::from)
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
