//! Compatibility parser for pre-commit configuration
//!
//! This module provides functionality for parsing .pre-commit-config.yaml files.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::parser::{Config, Hook, Repo, ConfigError, HookType, AccessMode};

/// Represents a hook in a .pre-commit-hooks.yaml file
#[derive(Debug, Serialize, Deserialize)]
pub struct PreCommitHookDefinition {
    /// Hook identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Description of the hook
    #[serde(default)]
    pub description: String,

    /// Entry point for the hook
    pub entry: String,

    /// Programming language or environment
    pub language: String,

    /// File pattern to match
    #[serde(default)]
    pub files: String,

    /// Additional arguments to pass to the hook
    #[serde(default)]
    pub args: Vec<String>,

    /// Stages to run this hook on
    #[serde(default)]
    pub stages: Vec<String>,
}

/// Represents a .pre-commit-hooks.yaml file
#[derive(Debug, Serialize, Deserialize)]
pub struct PreCommitHooksFile {
    /// List of hooks in this repository
    pub hooks: Vec<PreCommitHookDefinition>,
}

/// Parse a .pre-commit-hooks.yaml file
pub fn parse_precommit_hooks_file<P: AsRef<Path>>(path: P) -> Result<PreCommitHooksFile, ConfigError> {
    let hooks_str = fs::read_to_string(path)?;
    let hooks: PreCommitHooksFile = serde_yaml::from_str(&hooks_str)?;
    Ok(hooks)
}

/// Find and parse the .pre-commit-hooks.yaml file for a repository
pub fn find_precommit_hooks_for_repo(repo_url: &str) -> Option<PreCommitHooksFile> {
    // In a real implementation, this would fetch the repository and parse its .pre-commit-hooks.yaml file
    // For now, we'll simulate fetching and parsing the .pre-commit-hooks.yaml file

    // This function should fetch the repository, look for a .pre-commit-hooks.yaml file,
    // and parse it to determine the hooks available in the repository.

    // For the purpose of this implementation, we'll create a mock function that returns
    // a simulated .pre-commit-hooks.yaml file for well-known repositories.
    // In a production environment, this would be replaced with actual fetching and parsing logic.

    // Extract the repository name from the URL for logging purposes
    let repo_parts: Vec<&str> = repo_url.split('/').collect();
    if repo_parts.len() < 2 {
        return None;
    }

    // Get the last part of the URL (repo name)
    let _repo = repo_parts.last().unwrap_or(&"");

    // In a real implementation, we would:
    // 1. Clone or fetch the repository
    // 2. Look for a .pre-commit-hooks.yaml file
    // 3. Parse the file and return the hooks

    // For now, we'll return a simulated set of hooks for well-known repositories
    // This is just for demonstration purposes until the actual fetching logic is implemented

    // Create a mock .pre-commit-hooks.yaml file based on the repository URL
    // These are representative examples of what these files might contain

    // For pre-commit-hooks repository
    if repo_url.contains("pre-commit/pre-commit-hooks") {
        let hooks = vec![
            PreCommitHookDefinition {
                id: "trailing-whitespace".to_string(),
                name: "Trim Trailing Whitespace".to_string(),
                description: "Trims trailing whitespace".to_string(),
                entry: "trailing-whitespace".to_string(),
                language: "python".to_string(),
                files: "".to_string(),
                args: vec![],
                stages: vec!["commit".to_string()],
            },
            PreCommitHookDefinition {
                id: "end-of-file-fixer".to_string(),
                name: "Fix End of Files".to_string(),
                description: "Ensures that a file is either empty, or ends with one newline".to_string(),
                entry: "end-of-file-fixer".to_string(),
                language: "python".to_string(),
                files: "".to_string(),
                args: vec![],
                stages: vec!["commit".to_string()],
            },
            PreCommitHookDefinition {
                id: "check-yaml".to_string(),
                name: "Check Yaml".to_string(),
                description: "Checks yaml files for parseable syntax".to_string(),
                entry: "check-yaml".to_string(),
                language: "python".to_string(),
                files: "".to_string(),
                args: vec![],
                stages: vec!["commit".to_string()],
            },
            PreCommitHookDefinition {
                id: "check-added-large-files".to_string(),
                name: "Check for added large files".to_string(),
                description: "Prevents giant files from being committed".to_string(),
                entry: "check-added-large-files".to_string(),
                language: "python".to_string(),
                files: "".to_string(),
                args: vec![],
                stages: vec!["commit".to_string()],
            },
        ];
        return Some(PreCommitHooksFile { hooks });
    }

    // For ruff repository
    else if repo_url.contains("astral-sh/ruff-pre-commit") {
        let hooks = vec![
            PreCommitHookDefinition {
                id: "ruff".to_string(),
                name: "Ruff".to_string(),
                description: "Run Ruff to check Python code".to_string(),
                entry: "ruff".to_string(),
                language: "python".to_string(),
                files: "".to_string(),
                args: vec![],
                stages: vec!["commit".to_string()],
            },
            PreCommitHookDefinition {
                id: "ruff-format".to_string(),
                name: "Ruff Format".to_string(),
                description: "Run Ruff formatter on Python code".to_string(),
                entry: "ruff format".to_string(),
                language: "python".to_string(),
                files: "".to_string(),
                args: vec![],
                stages: vec!["commit".to_string()],
            },
        ];
        return Some(PreCommitHooksFile { hooks });
    }

    // For biome repository
    else if repo_url.contains("biomejs/pre-commit") {
        let hooks = vec![
            PreCommitHookDefinition {
                id: "biome-check".to_string(),
                name: "Biome Check".to_string(),
                description: "Run Biome check on JavaScript/TypeScript files".to_string(),
                entry: "biome check".to_string(),
                language: "node".to_string(),
                files: "".to_string(),
                args: vec![],
                stages: vec!["commit".to_string()],
            },
            PreCommitHookDefinition {
                id: "biome-format".to_string(),
                name: "Biome Format".to_string(),
                description: "Run Biome format on JavaScript/TypeScript files".to_string(),
                entry: "biome format".to_string(),
                language: "node".to_string(),
                files: "".to_string(),
                args: vec![],
                stages: vec!["commit".to_string()],
            },
        ];
        return Some(PreCommitHooksFile { hooks });
    }

    // For other repositories, we would need to fetch and parse their .pre-commit-hooks.yaml file
    // For now, we'll return None to indicate that we couldn't find a hooks file
    None
}

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
            // Determine the appropriate language and entry based on the hook
            let (language, entry) = if let Some(lang) = &precommit_hook.language {
                // If the hook specifies a language, use it
                (
                    lang.clone(),
                    precommit_hook.entry.clone().unwrap_or_else(|| precommit_hook.id.clone())
                )
            } else {
                // If no language is specified, look up the hook in the repository's .pre-commit-hooks.yaml file
                if let Some(hooks_file) = find_precommit_hooks_for_repo(&precommit_repo.repo) {
                    // Try to find the hook in the hooks file
                    if let Some(hook_def) = hooks_file.hooks.iter().find(|h| h.id == precommit_hook.id) {
                        // Use the language and entry from the hook definition
                        (
                            hook_def.language.clone(),
                            hook_def.entry.clone()
                        )
                    } else {
                        // If the hook is not found in the hooks file, use system language as a fallback
                        (
                            "system".to_string(),
                            precommit_hook.entry.clone().unwrap_or_else(|| precommit_hook.id.clone())
                        )
                    }
                } else {
                    // If no hooks file is found, use system language as a fallback
                    (
                        "system".to_string(),
                        precommit_hook.entry.clone().unwrap_or_else(|| precommit_hook.id.clone())
                    )
                }
            };

            // Determine the hook type based on the hook definition
            // This should be determined from the hook definition in the .pre-commit-hooks.yaml file
            // For now, we'll use a simple heuristic: hooks with simple entry points that match their IDs
            // are likely built-in, while hooks with more complex entry points are likely external
            let hook_type = if entry == precommit_hook.id {
                // If the entry point is the same as the ID, it's likely a built-in hook
                HookType::BuiltIn
            } else {
                // Otherwise, it's likely an external hook
                HookType::External
            };

            let hook = Hook {
                id: precommit_hook.id.clone(),
                name: precommit_hook.name.clone().unwrap_or_else(|| precommit_hook.id.clone()),
                entry,
                language,
                files: precommit_hook.files.clone().unwrap_or_default(),
                stages: precommit_hook.stages.clone().unwrap_or_else(|| precommit_config.default_stages.clone()),
                args: precommit_hook.args.clone().unwrap_or_default(),
                env: precommit_hook.env.clone().unwrap_or_default(),
                version: Some(precommit_repo.rev.clone()),
                hook_type,
                separate_process: false,
                access_mode: AccessMode::ReadWrite, // Default to read-write for safety
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
