//! Compatibility parser for pre-commit configuration
//!
//! This module provides functionality for parsing .pre-commit-config.yaml files.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use git2;

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
/// 
/// This is a wrapper around a vector of hook definitions to make it easier to work with.
#[derive(Debug, Serialize, Deserialize)]
pub struct PreCommitHooksFile {
    /// List of hooks in this repository
    pub hooks: Vec<PreCommitHookDefinition>,
}

impl From<Vec<PreCommitHookDefinition>> for PreCommitHooksFile {
    fn from(hooks: Vec<PreCommitHookDefinition>) -> Self {
        PreCommitHooksFile { hooks }
    }
}

/// Parse a .pre-commit-hooks.yaml file
pub fn parse_precommit_hooks_file<P: AsRef<Path>>(path: P) -> Result<PreCommitHooksFile, ConfigError> {
    let hooks_str = fs::read_to_string(path)?;

    // Try to parse as a PreCommitHooksFile first
    match serde_yaml::from_str::<PreCommitHooksFile>(&hooks_str) {
        Ok(hooks_file) => Ok(hooks_file),
        Err(_) => {
            // If that fails, try to parse as a Vec<PreCommitHookDefinition>
            let hooks: Vec<PreCommitHookDefinition> = serde_yaml::from_str(&hooks_str)?;
            Ok(PreCommitHooksFile::from(hooks))
        }
    }
}

/// Find and parse the .pre-commit-hooks.yaml file for a repository
/// 
/// This function clones the repository to the local cache directory and looks for
/// .pre-commit-hooks.yaml in the root of the repository.
/// If found, it parses the file and returns the hooks defined in it.
/// If the file can't be found or parsed, it returns None.
pub fn find_precommit_hooks_for_repo(repo_url: &str) -> Option<PreCommitHooksFile> {
    // Create a cache directory for repositories
    let cache_dir = std::env::current_dir().unwrap_or_default().join(".rustyhook").join("cache").join("repos");

    // Create a subdirectory for this specific repository
    // Use a hash of the repo URL to create a unique directory name
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    repo_url.hash(&mut hasher);
    let repo_hash = hasher.finish();

    let repo_dir = cache_dir.join(format!("{}", repo_hash));

    // Create the directory if it doesn't exist
    if !repo_dir.exists() {
        if let Err(err) = std::fs::create_dir_all(&repo_dir) {
            log::warn!("Failed to create cache directory: {}", err);
            return None;
        }

        log::debug!("Cloning repository {} into {}", repo_url, repo_dir.display());

        // Clone the repository
        match git2::Repository::clone(repo_url, &repo_dir) {
            Ok(_repo) => {},
            Err(err) => {
                log::warn!("Failed to clone repository {}: {}", repo_url, err);
                // Clean up the directory if the clone failed
                let _ = std::fs::remove_dir_all(&repo_dir);
                return None;
            }
        };
    } else {
        log::debug!("Using cached repository at {}", repo_dir.display());
    }

    // Look for .pre-commit-hooks.yaml in the repository
    let path = repo_dir.join(".pre-commit-hooks.yaml");

    // Try to find and parse the file
        log::debug!("Looking for .pre-commit-hooks.yaml at: {}", path.display());

        if path.exists() {
            log::debug!("Found .pre-commit-hooks.yaml at: {}", path.display());

            // Read the file
            match fs::read_to_string(&path) {
                Ok(content) => {
                    // Parse the YAML content
                    // Try to parse as a PreCommitHooksFile first
                    match serde_yaml::from_str::<PreCommitHooksFile>(&content) {
                        Ok(hooks_file) => {
                            log::info!("Successfully parsed .pre-commit-hooks.yaml from {} as a struct", path.display());
                            return Some(hooks_file);
                        }
                        Err(_) => {
                            // If that fails, try to parse as a Vec<PreCommitHookDefinition>
                            match serde_yaml::from_str::<Vec<PreCommitHookDefinition>>(&content) {
                                Ok(hooks) => {
                                    log::info!("Successfully parsed .pre-commit-hooks.yaml from {} as a sequence", path.display());
                                    return Some(PreCommitHooksFile::from(hooks));
                                }
                                Err(err) => {
                                    log::warn!("Failed to parse .pre-commit-hooks.yaml from {}: {}", path.display(), err);
                                }
                            }
                        }
                    }
                }
                Err(err) => {
                    log::warn!("Failed to read .pre-commit-hooks.yaml from {}: {}", path.display(), err);
                }
            }
        }

    log::warn!("Could not fetch .pre-commit-hooks.yaml for {}", repo_url);
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
