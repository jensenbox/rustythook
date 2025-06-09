//! Hook resolver for RustyHook
//!
//! This module provides functionality for resolving and running hooks.

use std::collections::HashMap;
use std::path::PathBuf;
use std::env;

use crate::config::{Config, Hook};
use crate::toolchains::{Tool, ToolError, SetupContext, PythonTool, NodeTool, RubyTool, SystemTool};
use crate::hooks::HookError;
use super::file_matcher::{FileMatcher, FileMatcherError};
use super::hook_context::HookContext;

/// Error type for hook resolver operations
#[derive(Debug)]
pub enum HookResolverError {
    /// Error with file matcher
    FileMatcherError(FileMatcherError),
    /// Error with tool
    ToolError(ToolError),
    /// Error with hook
    HookError(HookError),
    /// Hook not found
    HookNotFound(String),
    /// Unsupported language
    UnsupportedLanguage(String),
    /// Error running process
    ProcessError(String),
    /// IO error
    IoError(std::io::Error),
}

impl From<FileMatcherError> for HookResolverError {
    fn from(err: FileMatcherError) -> Self {
        HookResolverError::FileMatcherError(err)
    }
}

impl From<ToolError> for HookResolverError {
    fn from(err: ToolError) -> Self {
        HookResolverError::ToolError(err)
    }
}

impl From<HookError> for HookResolverError {
    fn from(err: HookError) -> Self {
        HookResolverError::HookError(err)
    }
}

impl From<std::io::Error> for HookResolverError {
    fn from(err: std::io::Error) -> Self {
        HookResolverError::IoError(err)
    }
}

/// Represents a hook resolver
pub struct HookResolver {
    /// Configuration
    config: Config,
    /// Cache directory
    cache_dir: PathBuf,
    /// Tool cache
    tool_cache: HashMap<String, Box<dyn Tool>>,
}

impl HookResolver {
    /// Create a new hook resolver
    pub fn new(config: Config, cache_dir: PathBuf) -> Self {
        HookResolver {
            config,
            cache_dir,
            tool_cache: HashMap::new(),
        }
    }

    /// Get the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Create a hook context from a hook
    fn create_context(&self, hook: &Hook, files: &[PathBuf]) -> Result<HookContext, HookResolverError> {
        // Get the current working directory
        let working_dir = env::current_dir()?;

        // Create a file matcher if the hook has a file pattern
        let filtered_files = if !hook.files.is_empty() {
            let matcher = FileMatcher::from_regex(&hook.files)?;
            matcher.filter_files(files)
        } else {
            files.to_vec()
        };

        // Create the context
        let context = HookContext::from_hook(hook, working_dir, filtered_files);

        Ok(context)
    }

    /// Resolve a hook by ID
    pub fn resolve_hook(&self, repo_id: &str, hook_id: &str) -> Result<Hook, HookResolverError> {
        // Find the repository
        let repo = self.config.repos.iter()
            .find(|r| r.repo == repo_id)
            .ok_or_else(|| HookResolverError::HookNotFound(format!("Repository {} not found", repo_id)))?;

        // Find the hook
        let hook = repo.hooks.iter()
            .find(|h| h.id == hook_id)
            .ok_or_else(|| HookResolverError::HookNotFound(format!("Hook {} not found in repository {}", hook_id, repo_id)))?;

        // Return a clone of the hook to avoid borrowing issues
        Ok(hook.clone())
    }

    /// Create a tool for a hook
    fn create_tool(&self, hook: &Hook) -> Result<Box<dyn Tool>, HookResolverError> {
        // Get the version to use
        let version = hook.version.clone().unwrap_or_else(|| "latest".to_string());

        match hook.language.as_str() {
            "python" => {
                // Create a Python tool
                // Extract the package name from the entry (first part before space)
                let package_name = hook.entry.split_whitespace().next().unwrap_or(&hook.entry).to_string();

                // For pre-commit-hooks, we need to install the pre-commit-hooks package
                let package = if package_name == "pre-commit-hooks" {
                    "pre-commit-hooks".to_string()
                } else if package_name == "ruff" {
                    "ruff".to_string()
                } else if package_name == "shellcheck" {
                    "shellcheck-py".to_string()
                } else if package_name == "codespell" {
                    "codespell".to_string()
                } else if package_name == "djhtml" {
                    "djhtml".to_string()
                } else {
                    package_name
                };

                let packages = vec![package];
                let tool = PythonTool::new(hook.id.clone(), version, packages);
                Ok(Box::new(tool))
            },
            "node" | "javascript" | "typescript" => {
                // Create a Node.js tool
                // Extract the package name from the entry (first part before space)
                let package_name = hook.entry.split_whitespace().next().unwrap_or(&hook.entry).to_string();

                // For biome, we need to install the @biomejs/biome package
                let package = if package_name == "biome" {
                    "@biomejs/biome".to_string()
                } else {
                    package_name
                };

                let packages = vec![package];
                let tool = NodeTool::new(hook.id.clone(), version, packages, true, None);
                Ok(Box::new(tool))
            },
            "ruby" => {
                // Create a Ruby tool
                // Extract the package name from the entry (first part before space)
                let package_name = hook.entry.split_whitespace().next().unwrap_or(&hook.entry).to_string();
                let gems = vec![package_name];
                let tool = RubyTool::new(hook.id.clone(), version, gems);
                Ok(Box::new(tool))
            },
            "system" => {
                // For system hooks, we create a SystemTool
                let tool = SystemTool::new(hook.id.clone(), version, hook.entry.clone());
                Ok(Box::new(tool))
            },
            _ => {
                // Unsupported language
                Err(HookResolverError::UnsupportedLanguage(hook.language.clone()))
            }
        }
    }

    /// Set up a tool for a hook
    fn setup_tool(&mut self, hook: &Hook) -> Result<&Box<dyn Tool>, HookResolverError> {
        // Check if the tool is already in the cache
        let tool_key = format!("{}-{}", hook.language, hook.id);
        if !self.tool_cache.contains_key(&tool_key) {
            // Create the tool
            let tool = self.create_tool(hook)?;

            // Set up the tool
            let ctx = SetupContext {
                install_dir: self.cache_dir.join("venvs").join(&tool_key),
                cache_dir: self.cache_dir.join("cache").join(&tool_key),
                force: false,
                version: Some(hook.version.clone().unwrap_or_else(|| "latest".to_string())),
            };

            // Set up the tool
            tool.setup(&ctx)?;

            // Add the tool to the cache
            self.tool_cache.insert(tool_key.clone(), tool);
        }

        // Return the tool from the cache
        Ok(self.tool_cache.get(&tool_key).unwrap())
    }


    /// Run a hook on files
    pub fn run_hook(&mut self, repo_id: &str, hook_id: &str, files: &[PathBuf]) -> Result<(), HookResolverError> {
        // First, get all the information we need from immutable borrows
        let hook_clone = {
            let hook = self.resolve_hook(repo_id, hook_id)?;
            hook.clone()
        };

        // Create the context for running the hook
        let context = self.create_context(&hook_clone, files)?;

        // If there are no files to process, we're done
        if context.files_to_process.is_empty() {
            return Ok(());
        }

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
            // Run the hook in the same process using the tool
            // Now we can do the mutable borrow since the immutable borrow is no longer active
            let tool = self.setup_tool(&hook_clone)?;

            // Execute the hook using the context
            context.execute(Some(tool.as_ref())).map_err(|err| match err {
                super::hook_context::HookContextError::ProcessError(msg) => HookResolverError::ProcessError(msg),
                super::hook_context::HookContextError::IoError(err) => HookResolverError::IoError(err),
                super::hook_context::HookContextError::HookError(err) => HookResolverError::HookError(err),
                super::hook_context::HookContextError::ToolError(err) => HookResolverError::ToolError(err),
            })
        }
    }

    /// Run all hooks on files
    pub fn run_all_hooks(&mut self, files: &[PathBuf]) -> Result<(), HookResolverError> {
        // Collect all hooks first to avoid borrowing issues
        let hooks_to_run: Vec<(String, String)> = self.config.repos.iter()
            .flat_map(|repo| {
                repo.hooks.iter().map(move |hook| (repo.repo.clone(), hook.id.clone()))
            })
            .collect();

        // Run each hook
        for (repo_id, hook_id) in hooks_to_run {
            self.run_hook(&repo_id, &hook_id, files)?;
        }

        Ok(())
    }
}
