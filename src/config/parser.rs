//! Configuration parser for RustyHook
//!
//! This module provides functionality for parsing RustyHook configuration files.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::fmt;

/// Represents a complete RustyHook configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// Default stages to run hooks on
    #[serde(default = "default_stages")]
    pub default_stages: Vec<String>,

    /// Whether to stop running hooks after the first failure
    #[serde(default)]
    pub fail_fast: bool,

    /// Maximum number of hooks to run in parallel (0 means unlimited)
    #[serde(default = "default_parallelism")]
    pub parallelism: usize,

    /// List of repositories containing hooks
    pub repos: Vec<Repo>,
}

/// Represents a repository containing hooks
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Repo {
    /// Repository identifier
    pub repo: String,

    /// List of hooks in this repository
    pub hooks: Vec<Hook>,
}

/// Type of hook (built-in or external)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum HookType {
    /// Built-in hook that is part of RustyHook
    BuiltIn,
    /// External hook that is run as a separate command
    External,
}

impl fmt::Display for HookType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HookType::BuiltIn => write!(f, "built-in"),
            HookType::External => write!(f, "external"),
        }
    }
}

/// Default hook type (external)
fn default_hook_type() -> HookType {
    HookType::External
}

/// Access mode for hooks (read-only or read-write)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AccessMode {
    /// Read-only access to files (can run in parallel with other hooks)
    Read,
    /// Read-write access to files (cannot run in parallel with other hooks on the same files)
    ReadWrite,
}

impl fmt::Display for AccessMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccessMode::Read => write!(f, "read"),
            AccessMode::ReadWrite => write!(f, "read-write"),
        }
    }
}

/// Default access mode (read-write for safety)
fn default_access_mode() -> AccessMode {
    AccessMode::ReadWrite
}

/// Represents a single hook
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Hook {
    /// Hook identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Command or script to run
    pub entry: String,

    /// Programming language or environment
    pub language: String,

    /// File pattern to match
    #[serde(default)]
    pub files: String,

    /// Stages to run this hook on
    #[serde(default = "default_stages")]
    pub stages: Vec<String>,

    /// Additional arguments to pass to the hook
    #[serde(default)]
    pub args: Vec<String>,

    /// Additional environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Version of the tool to use
    #[serde(default)]
    pub version: Option<String>,

    /// Whether this hook is built-in or external
    #[serde(default = "default_hook_type")]
    pub hook_type: HookType,

    /// Whether to run this hook in a separate process
    #[serde(default)]
    pub separate_process: bool,

    /// Access mode for this hook (read-only or read-write)
    #[serde(default = "default_access_mode")]
    pub access_mode: AccessMode,
}

/// Default stages for hooks
fn default_stages() -> Vec<String> {
    vec!["commit".to_string()]
}

/// Default parallelism for hook execution (0 means unlimited)
fn default_parallelism() -> usize {
    0
}

/// Error type for configuration operations
#[derive(Debug)]
pub enum ConfigError {
    /// Error reading the configuration file
    IoError(std::io::Error),
    /// Error parsing the YAML configuration
    ParseError(serde_yaml::Error),
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::IoError(err)
    }
}

impl From<serde_yaml::Error> for ConfigError {
    fn from(err: serde_yaml::Error) -> Self {
        ConfigError::ParseError(err)
    }
}

/// Parse a RustyHook configuration file
pub fn parse_config<P: AsRef<Path>>(path: P) -> Result<Config, ConfigError> {
    let config_str = fs::read_to_string(path)?;
    let config: Config = serde_yaml::from_str(&config_str)?;
    Ok(config)
}

/// Find and parse the RustyHook configuration file
pub fn find_config() -> Result<Config, ConfigError> {
    // Look for .rustyhook/config.yaml in the current directory and parent directories
    let mut current_dir = std::env::current_dir().map_err(ConfigError::IoError)?;

    loop {
        let config_path = current_dir.join(".rustyhook").join("config.yaml");
        if config_path.exists() {
            return parse_config(config_path);
        }

        // Move to the parent directory
        if !current_dir.pop() {
            // We've reached the root directory and haven't found a config file
            break;
        }
    }

    // If no config file is found, return an error
    Err(ConfigError::IoError(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "No .rustyhook/config.yaml file found",
    )))
}
