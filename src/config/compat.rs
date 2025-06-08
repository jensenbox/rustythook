//! Compatibility parser for pre-commit configuration
//!
//! This module provides functionality for parsing .pre-commit-config.yaml files.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::parser::{Config, Hook, Repo, ConfigError};

/// Represents a pre-commit configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct PreCommitConfig {
    /// Default stages to run hooks on
    #[serde(default = "default_stages")]
    pub default_stages: Vec<String>,

    /// Whether to stop running hooks after the first failure
    #[serde(default)]
    pub fail_fast: bool,

    /// List of repositories containing hooks
    pub repos: Vec<PreCommitRepo>,
}

/// Represents a repository in a pre-commit configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct PreCommitRepo {
    /// Repository URL or identifier
    pub repo: String,

    /// Repository revision or version
    #[serde(default)]
    pub rev: String,

    /// List of hooks in this repository
    pub hooks: Vec<PreCommitHook>,
}

/// Represents a hook in a pre-commit configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct PreCommitHook {
    /// Hook identifier
    pub id: String,

    /// Human-readable name (optional)
    #[serde(default)]
    pub name: Option<String>,

    /// Command or script to run (optional)
    #[serde(default)]
    pub entry: Option<String>,

    /// Programming language or environment (optional)
    #[serde(default)]
    pub language: Option<String>,

    /// File pattern to match (optional)
    #[serde(default)]
    pub files: Option<String>,

    /// Stages to run this hook on (optional)
    #[serde(default)]
    pub stages: Option<Vec<String>>,

    /// Additional arguments to pass to the hook (optional)
    #[serde(default)]
    pub args: Option<Vec<String>>,

    /// Additional environment variables (optional)
    #[serde(default)]
    pub env: Option<HashMap<String, String>>,
}

/// Default stages for hooks
fn default_stages() -> Vec<String> {
    vec!["commit".to_string()]
}

/// Parse a pre-commit configuration file
pub fn parse_precommit_config<P: AsRef<Path>>(path: P) -> Result<PreCommitConfig, ConfigError> {
    let config_str = fs::read_to_string(path)?;
    let config: PreCommitConfig = serde_yaml::from_str(&config_str)?;
    Ok(config)
}

/// Find the path to the pre-commit configuration file
pub fn find_precommit_config_path() -> Result<PathBuf, ConfigError> {
    // Look for .pre-commit-config.yaml in the current directory and parent directories
    let mut current_dir = std::env::current_dir().map_err(ConfigError::IoError)?;

    loop {
        let config_path = current_dir.join(".pre-commit-config.yaml");
        if config_path.exists() {
            return Ok(config_path);
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
        "No .pre-commit-config.yaml file found",
    )))
}

/// Find and parse the pre-commit configuration file
pub fn find_precommit_config() -> Result<PreCommitConfig, ConfigError> {
    // Find the path to the pre-commit config file
    let config_path = find_precommit_config_path()?;

    // Parse the pre-commit config file
    parse_precommit_config(config_path)
}

/// Convert a pre-commit configuration to a RustyHook configuration
pub fn convert_to_rustyhook_config(precommit_config: &PreCommitConfig) -> Config {
    let mut repos = Vec::new();

    for precommit_repo in &precommit_config.repos {
        let mut hooks = Vec::new();

        for precommit_hook in &precommit_repo.hooks {
            let hook = Hook {
                id: precommit_hook.id.clone(),
                name: precommit_hook.name.clone().unwrap_or_else(|| precommit_hook.id.clone()),
                entry: precommit_hook.entry.clone().unwrap_or_else(|| format!("pre-commit-hooks {}", precommit_hook.id)),
                language: precommit_hook.language.clone().unwrap_or_else(|| "system".to_string()),
                files: precommit_hook.files.clone().unwrap_or_default(),
                stages: precommit_hook.stages.clone().unwrap_or_else(|| precommit_config.default_stages.clone()),
                args: precommit_hook.args.clone().unwrap_or_default(),
                env: precommit_hook.env.clone().unwrap_or_default(),
                version: Some(precommit_repo.rev.clone()),
            };

            hooks.push(hook);
        }

        let repo = Repo {
            repo: precommit_repo.repo.clone(),
            hooks,
        };

        repos.push(repo);
    }

    Config {
        default_stages: precommit_config.default_stages.clone(),
        fail_fast: precommit_config.fail_fast,
        parallelism: 0,
        repos,
    }
}
