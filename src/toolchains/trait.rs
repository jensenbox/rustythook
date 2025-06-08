//! Tool trait definition for RustyHook
//!
//! This module defines the Tool trait, which is implemented by all toolchains.

use std::path::PathBuf;

/// Context for setting up a tool
pub struct SetupContext {
    /// The directory where the tool should be installed
    pub install_dir: PathBuf,

    /// The directory where the tool's cache should be stored
    pub cache_dir: PathBuf,

    /// Whether to force reinstallation even if the tool is already installed
    pub force: bool,

    /// The version of the tool to install
    pub version: Option<String>,
}

/// Error type for tool operations
#[derive(Debug)]
pub enum ToolError {
    /// Error executing a command
    ExecutionError(String),

    /// Error finding a required tool
    ToolNotFound(String),

    /// Error installing a tool
    InstallationError(String),

    /// Error with the file system
    IoError(std::io::Error),
}

impl From<std::io::Error> for ToolError {
    fn from(err: std::io::Error) -> Self {
        ToolError::IoError(err)
    }
}

/// Trait for tools that can be used by RustyHook
pub trait Tool: Send {
    /// Set up the tool in the given context
    fn setup(&self, ctx: &SetupContext) -> Result<(), ToolError>;

    /// Run the tool on the given files
    fn run(&self, files: &[PathBuf]) -> Result<(), ToolError>;

    /// Get the name of the tool
    fn name(&self) -> &str;

    /// Get the version of the tool
    fn version(&self) -> &str;

    /// Check if the tool is installed
    fn is_installed(&self) -> bool;

    /// Get the installation directory of the tool
    fn install_dir(&self) -> &PathBuf;
}
