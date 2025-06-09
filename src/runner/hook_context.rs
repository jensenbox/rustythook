//! Hook context for RustyHook
//!
//! This module provides the context for running hooks.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use crate::config::parser::HookType;
use crate::hooks::HookError;

/// Error type for hook context operations
#[derive(Debug)]
pub enum HookContextError {
    /// Error running process
    ProcessError(String),
    /// IO error
    IoError(std::io::Error),
    /// Hook error
    HookError(HookError),
    /// Tool error
    ToolError(crate::toolchains::ToolError),
}

impl From<std::io::Error> for HookContextError {
    fn from(err: std::io::Error) -> Self {
        HookContextError::IoError(err)
    }
}

impl From<HookError> for HookContextError {
    fn from(err: HookError) -> Self {
        HookContextError::HookError(err)
    }
}

impl From<crate::toolchains::ToolError> for HookContextError {
    fn from(err: crate::toolchains::ToolError) -> Self {
        HookContextError::ToolError(err)
    }
}

/// Represents the context for running a hook
#[derive(Debug, Clone)]
pub struct HookContext {
    /// Hook identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Command or script to run
    pub entry: String,

    /// Programming language or environment
    pub language: String,

    /// File pattern to match
    pub files: String,

    /// Stages to run this hook on
    pub stages: Vec<String>,

    /// Additional arguments to pass to the hook
    pub args: Vec<String>,

    /// Additional environment variables
    pub env: HashMap<String, String>,

    /// Version of the tool to use
    pub version: Option<String>,

    /// Whether this hook is built-in or external
    pub hook_type: HookType,

    /// Whether to run this hook in a separate process
    pub separate_process: bool,

    /// Working directory for the hook
    pub working_dir: PathBuf,

    /// Files to process
    pub files_to_process: Vec<PathBuf>,
}

impl HookContext {
    /// Create a new hook context
    pub fn new(
        id: String,
        name: String,
        entry: String,
        language: String,
        files: String,
        stages: Vec<String>,
        args: Vec<String>,
        env: HashMap<String, String>,
        version: Option<String>,
        hook_type: HookType,
        separate_process: bool,
        working_dir: PathBuf,
        files_to_process: Vec<PathBuf>,
    ) -> Self {
        HookContext {
            id,
            name,
            entry,
            language,
            files,
            stages,
            args,
            env,
            version,
            hook_type,
            separate_process,
            working_dir,
            files_to_process,
        }
    }

    /// Create a hook context from a hook configuration
    pub fn from_hook(
        hook: &crate::config::Hook,
        working_dir: PathBuf,
        files_to_process: Vec<PathBuf>,
    ) -> Self {
        HookContext {
            id: hook.id.clone(),
            name: hook.name.clone(),
            entry: hook.entry.clone(),
            language: hook.language.clone(),
            files: hook.files.clone(),
            stages: hook.stages.clone(),
            args: hook.args.clone(),
            env: hook.env.clone(),
            version: hook.version.clone(),
            hook_type: hook.hook_type.clone(),
            separate_process: hook.separate_process,
            working_dir,
            files_to_process,
        }
    }

    /// Determine if the hook should be run in a separate process
    pub fn should_run_in_separate_process(&self) -> bool {
        self.separate_process || self.hook_type == HookType::External
    }

    /// Run the hook in a separate process
    pub fn run_in_separate_process(&self) -> Result<(), HookContextError> {
        println!("Running hook {} in separate process", self.id);

        // Parse the entry to separate the command from any arguments
        let parts: Vec<&str> = self.entry.split_whitespace().collect();
        if parts.is_empty() {
            return Err(HookContextError::ProcessError(format!(
                "Empty entry for hook {}", self.id
            )));
        }

        // The first part is the command, the rest are arguments
        let command_name = parts[0];
        let command_args = &parts[1..];

        // Create a command to run the hook
        let mut command = Command::new(command_name);

        // Add any arguments from the entry
        for arg in command_args {
            command.arg(arg);
        }

        // Add arguments from the hook configuration
        for arg in &self.args {
            command.arg(arg);
        }

        // Add files to process
        for file in &self.files_to_process {
            command.arg(file);
        }

        // Set environment variables
        for (key, value) in &self.env {
            command.env(key, value);
        }

        // Set working directory
        command.current_dir(&self.working_dir);

        // Run the command
        let output = command.output()?;

        // Check if the command was successful
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(HookContextError::ProcessError(format!(
                "Hook {} failed: {}", self.id, stderr
            )));
        }

        Ok(())
    }

    /// Execute the hook using the appropriate method
    pub fn execute(&self, tool: Option<&dyn crate::toolchains::Tool>) -> Result<(), HookContextError> {
        // If there are no files to process, we're done
        if self.files_to_process.is_empty() {
            return Ok(());
        }

        // Decide how to run the hook based on the context
        if self.should_run_in_separate_process() {
            // Run the hook in a separate process
            self.run_in_separate_process()
        } else {
            // Run the hook in the same process using the tool
            if let Some(tool) = tool {
                tool.run(&self.files_to_process).map_err(HookContextError::ToolError)
            } else {
                Err(HookContextError::ProcessError(format!(
                    "No tool provided for hook {}", self.id
                )))
            }
        }
    }
}
