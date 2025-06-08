//! Configuration converter for RustyHook
//!
//! This module provides functionality for converting between different configuration formats.

use std::fs;
use std::path::{Path, PathBuf};

use super::compat::{find_precommit_config, find_precommit_config_path, parse_precommit_config, convert_to_rustyhook_config};
use super::parser::{Config, ConfigError};

/// Error type for conversion operations
#[derive(Debug)]
pub enum ConversionError {
    /// Error reading or writing configuration files
    IoError(std::io::Error),
    /// Error parsing or serializing YAML
    YamlError(serde_yaml::Error),
    /// Error with the configuration
    ConfigError(ConfigError),
}

impl From<std::io::Error> for ConversionError {
    fn from(err: std::io::Error) -> Self {
        ConversionError::IoError(err)
    }
}

impl From<serde_yaml::Error> for ConversionError {
    fn from(err: serde_yaml::Error) -> Self {
        ConversionError::YamlError(err)
    }
}

impl From<ConfigError> for ConversionError {
    fn from(err: ConfigError) -> Self {
        ConversionError::ConfigError(err)
    }
}

/// Convert a pre-commit configuration to a RustyHook configuration and write it to a file
pub fn convert_from_precommit<P: AsRef<Path>>(
    precommit_path: Option<P>,
    output_path: Option<PathBuf>,
    delete_original: bool,
) -> Result<(), ConversionError> {
    // Store the path to the pre-commit config file for later use
    let original_path = match &precommit_path {
        Some(path) => Some(path.as_ref().to_path_buf()),
        None => {
            // Try to find the pre-commit config file
            let mut path = std::env::current_dir()?;
            path.push(".pre-commit-config.yaml");
            if path.exists() {
                Some(path)
            } else {
                None
            }
        }
    };

    // Find or parse the pre-commit configuration
    let precommit_config = match precommit_path {
        Some(path) => parse_precommit_config(path)?,
        None => find_precommit_config()?,
    };

    // Convert the pre-commit configuration to a RustyHook configuration
    let rustyhook_config = convert_to_rustyhook_config(&precommit_config);

    // Determine the output path
    let output_path = match output_path {
        Some(path) => path,
        None => {
            let mut path = std::env::current_dir()?;
            path.push(".rustyhook");
            fs::create_dir_all(&path)?;
            path.push("config.yaml");
            path
        }
    };

    // Write the RustyHook configuration to the output file
    let yaml = serde_yaml::to_string(&rustyhook_config)?;
    fs::write(output_path, yaml)?;

    // Delete the original pre-commit config file if requested
    if delete_original {
        // Use the stored path to the pre-commit config file
        if let Some(path) = original_path {
            // Delete the file if it exists
            if path.exists() {
                fs::remove_file(path)?;
                println!("Deleted original pre-commit config file.");
            }
        } else {
            // Try to find the pre-commit config file using the new function
            match find_precommit_config_path() {
                Ok(path) => {
                    fs::remove_file(path)?;
                    println!("Deleted original pre-commit config file.");
                },
                Err(e) => {
                    eprintln!("Warning: Could not find pre-commit config file to delete: {:?}", e);
                }
            }
        }
    }

    Ok(())
}

/// Create a starter RustyHook configuration and write it to a file
pub fn create_starter_config<P: AsRef<Path>>(output_path: Option<P>) -> Result<(), ConversionError> {
    // Create a simple starter configuration
    let config = Config {
        default_stages: vec!["commit".to_string()],
        fail_fast: false,
        parallelism: 0,
        repos: vec![],
    };

    // Determine the output path
    let output_path = match output_path {
        Some(path) => path.as_ref().to_path_buf(),
        None => {
            let mut path = std::env::current_dir()?;
            path.push(".rustyhook");
            fs::create_dir_all(&path)?;
            path.push("config.yaml");
            path
        }
    };

    // Write the starter configuration to the output file
    let yaml = serde_yaml::to_string(&config)?;
    fs::write(output_path, yaml)?;

    Ok(())
}
