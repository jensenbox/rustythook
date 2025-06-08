//! System tool implementation for RustyHook
//!
//! This module provides a tool implementation for system commands.

use std::path::PathBuf;
use std::process::Command;

use super::r#trait::{SetupContext, Tool, ToolError};

/// A tool that runs system commands
pub struct SystemTool {
    /// The name of the tool
    name: String,

    /// The version of the tool
    version: String,

    /// The command to run
    command: String,

    /// The installation directory
    install_dir: PathBuf,
}

impl SystemTool {
    /// Create a new system tool
    pub fn new(name: String, version: String, command: String) -> Self {
        SystemTool {
            name,
            version,
            command,
            install_dir: PathBuf::from("/usr/bin"), // Default to /usr/bin
        }
    }
}

impl Tool for SystemTool {
    fn setup(&self, _ctx: &SetupContext) -> Result<(), ToolError> {
        // For system tools, we don't need to do any setup
        // Just check if the command exists
        let parts: Vec<&str> = self.command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(ToolError::ToolNotFound(format!("Empty command")));
        }

        let cmd = parts[0];
        match which::which(cmd) {
            Ok(_) => Ok(()),
            Err(_) => Err(ToolError::ToolNotFound(format!("Command not found: {}", cmd))),
        }
    }

    fn run(&self, files: &[PathBuf]) -> Result<(), ToolError> {
        // Split the command into parts
        let parts: Vec<&str> = self.command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(ToolError::ExecutionError(format!("Empty command")));
        }

        let cmd = parts[0];
        let args: Vec<&str> = parts[1..].to_vec();

        // Add the files to the arguments
        let file_args: Vec<String> = files.iter()
            .map(|f| f.to_string_lossy().to_string())
            .collect();

        // Run the command
        let status = Command::new(cmd)
            .args(args)
            .args(file_args)
            .status()
            .map_err(|e| ToolError::ExecutionError(format!("Failed to execute command: {}", e)))?;

        if status.success() {
            Ok(())
        } else {
            Err(ToolError::ExecutionError(format!("Command failed with exit code: {:?}", status.code())))
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn is_installed(&self) -> bool {
        // Check if the command exists
        let parts: Vec<&str> = self.command.split_whitespace().collect();
        if parts.is_empty() {
            return false;
        }

        let cmd = parts[0];
        which::which(cmd).is_ok()
    }

    fn install_dir(&self) -> &PathBuf {
        &self.install_dir
    }
}
