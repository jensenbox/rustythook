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
    /// File not found error with path information
    FileNotFound {
        /// The path that was not found
        path: std::path::PathBuf,
        /// Additional context about the error
        context: String,
    },
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

impl std::fmt::Display for HookResolverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookResolverError::FileMatcherError(err) => write!(f, "ERROR: File matching error.\n\nDetails: {:?}\n\nSOLUTION: Check the file pattern in your configuration and ensure it's a valid regex pattern.", err),
            HookResolverError::ToolError(err) => write!(f, "ERROR: Tool setup or execution failed.\n\nDetails: {:?}\n\nSOLUTION: Ensure the required tools are installed and properly configured. Run 'rustyhook doctor' for diagnostics.", err),
            HookResolverError::HookError(err) => write!(f, "ERROR: Hook execution failed.\n\nDetails: {:?}\n\nSOLUTION: Check the hook configuration and ensure all dependencies are installed.", err),
            HookResolverError::HookNotFound(msg) => write!(f, "ERROR: Hook not found.\n\nDetails: {}\n\nSOLUTION: Verify that the hook ID is correct and defined in your configuration file.", msg),
            HookResolverError::UnsupportedLanguage(lang) => write!(f, "ERROR: Unsupported language: {}\n\nSOLUTION: Use one of the supported languages: python, node, javascript, typescript, ruby, or system.", lang),
            HookResolverError::ProcessError(msg) => write!(f, "ERROR: Process execution failed.\n\nDetails: {}\n\nSOLUTION: Check that the command exists and has the correct permissions.", msg),
            HookResolverError::FileNotFound { path, context } => {
                write!(f, "ERROR: Specific file not found: {}\n\nContext: {}\n\nSOLUTION: Please check that this file exists and that the path is correct. If this is a configuration file, ensure it's properly formatted.", 
                       path.display(), context)
            },
            HookResolverError::IoError(err) => {
                match err.kind() {
                    std::io::ErrorKind::NotFound => write!(f, "ERROR: File or directory not found.\n\nThis could be due to one of the following issues:\n\
                       - Missing configuration file (check for .rustyhook/config.yaml or .pre-commit-config.yaml)\n\
                       - Missing hook script or executable (verify the 'entry' path in your config)\n\
                       - Missing dependencies required by a hook\n\
                       - Incorrect working directory (ensure you're running from the repository root)\n\n\
                       SOLUTION: Try running 'rustyhook doctor' for more detailed diagnostics, or check the paths in your configuration."),
                    std::io::ErrorKind::PermissionDenied => write!(f, "ERROR: Permission denied.\n\nDetails: {}\n\nSOLUTION: Check file permissions and ensure you have the necessary access rights. You may need to run with elevated privileges.", err),
                    _ => write!(f, "ERROR: IO operation failed.\n\nDetails: {}\n\nSOLUTION: Check system resources, disk space, and file access. If the issue persists, try running 'rustyhook doctor' for diagnostics.", err),
                }
            }
        }
    }
}

impl std::error::Error for HookResolverError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            HookResolverError::IoError(err) => Some(err),
            _ => None,
        }
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
    /// Hooks to skip
    hooks_to_skip: Vec<String>,
}

impl HookResolver {
    /// Create a new hook resolver
    pub fn new(config: Config, cache_dir: PathBuf) -> Self {
        HookResolver {
            config,
            cache_dir,
            tool_cache: HashMap::new(),
            hooks_to_skip: Vec::new(),
        }
    }

    /// Set hooks to skip
    pub fn set_hooks_to_skip(&mut self, hooks: Vec<String>) {
        self.hooks_to_skip = hooks;
    }

    /// Get hooks to skip
    pub fn hooks_to_skip(&self) -> &Vec<String> {
        &self.hooks_to_skip
    }

    /// Get the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Create a hook context from a hook
    fn create_context(&self, hook: &Hook, files: &[PathBuf]) -> Result<HookContext, HookResolverError> {
        // Get the current working directory
        let working_dir = env::current_dir().map_err(|err| {
            HookResolverError::FileNotFound {
                path: PathBuf::from("."),
                context: format!("Failed to access current working directory when creating context for hook '{}': {}", hook.id, err)
            }
        })?;

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
                super::hook_context::HookContextError::CommandNotFound { command, hook_id, error: _ } => {
                    HookResolverError::FileNotFound {
                        path: PathBuf::from(command),
                        context: format!("Command not found when running hook '{}'. Make sure the command is installed and available in your PATH.", hook_id)
                    }
                }
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
                super::hook_context::HookContextError::CommandNotFound { command, hook_id, error: _ } => {
                    HookResolverError::FileNotFound {
                        path: PathBuf::from(command),
                        context: format!("Command not found when running hook '{}'. Make sure the command is installed and available in your PATH.", hook_id)
                    }
                }
            })
        }
    }

    /// Run all hooks on files
    pub fn run_all_hooks(&mut self, files: &[PathBuf]) -> Result<(), HookResolverError> {
        // Collect all hooks first to avoid borrowing issues
        let hooks_to_run: Vec<(String, String)> = self.config.repos.iter()
            .flat_map(|repo| {
                repo.hooks.iter()
                    .filter(|hook| !self.hooks_to_skip.contains(&hook.id))
                    .map(move |hook| (repo.repo.clone(), hook.id.clone()))
            })
            .collect();

        // Log which hooks are being skipped
        if !self.hooks_to_skip.is_empty() {
            log::info!("Skipping hooks: {}", self.hooks_to_skip.join(", "));
        }

        // Run each hook
        for (repo_id, hook_id) in hooks_to_run {
            self.run_hook(&repo_id, &hook_id, files)?;
        }

        Ok(())
    }
}
