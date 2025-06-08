//! Native implementations of pre-commit hooks
//!
//! This module provides native Rust implementations of the hooks from
//! https://github.com/pre-commit/pre-commit-hooks

use std::path::PathBuf;
use std::fs;
use std::io::{self};
use std::collections::HashSet;

/// Error type for hook operations
#[derive(Debug)]
pub enum HookError {
    /// IO error
    IoError(io::Error),
    /// Invalid UTF-8
    Utf8Error(std::string::FromUtf8Error),
    /// Other error
    Other(String),
}

impl From<io::Error> for HookError {
    fn from(err: io::Error) -> Self {
        HookError::IoError(err)
    }
}

impl From<std::string::FromUtf8Error> for HookError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        HookError::Utf8Error(err)
    }
}

/// Trait for hooks
pub trait Hook {
    /// Run the hook on files
    fn run(&self, files: &[PathBuf]) -> Result<(), HookError>;
}

/// Trim trailing whitespace
pub struct TrailingWhitespace;

impl Hook for TrailingWhitespace {
    fn run(&self, files: &[PathBuf]) -> Result<(), HookError> {
        for file in files {
            // Read the file
            let content = fs::read_to_string(file)?;

            // Check if the file has trailing whitespace
            let mut has_trailing_whitespace = false;
            let mut new_content = String::new();

            for line in content.lines() {
                let trimmed = line.trim_end();
                if trimmed.len() != line.len() {
                    has_trailing_whitespace = true;
                }
                new_content.push_str(trimmed);
                new_content.push('\n');
            }

            // If the file has trailing whitespace, fix it
            if has_trailing_whitespace {
                fs::write(file, new_content)?;
            }
        }

        Ok(())
    }
}

/// Fix end of files
pub struct EndOfFileFixer;

impl Hook for EndOfFileFixer {
    fn run(&self, files: &[PathBuf]) -> Result<(), HookError> {
        for file in files {
            // Read the file
            let content = fs::read_to_string(file)?;

            // Check if the file is empty or ends with a newline
            if content.is_empty() || content.ends_with('\n') {
                continue;
            }

            // Fix the file
            let mut new_content = content;
            new_content.push('\n');
            fs::write(file, new_content)?;
        }

        Ok(())
    }
}

/// Check YAML files for parseable syntax
pub struct CheckYaml;

impl Hook for CheckYaml {
    fn run(&self, files: &[PathBuf]) -> Result<(), HookError> {
        for file in files {
            // Read the file
            let content = fs::read_to_string(file)?;

            // Try to parse the YAML
            match serde_yaml::from_str::<serde_yaml::Value>(&content) {
                Ok(_) => continue,
                Err(err) => return Err(HookError::Other(format!("Invalid YAML in {}: {}", file.display(), err))),
            }
        }

        Ok(())
    }
}

/// Check for added large files
pub struct CheckAddedLargeFiles {
    /// Maximum file size in kilobytes
    max_size_kb: usize,
}

impl CheckAddedLargeFiles {
    /// Create a new instance with the given maximum file size
    pub fn new(max_size_kb: usize) -> Self {
        CheckAddedLargeFiles { max_size_kb }
    }
}

impl Hook for CheckAddedLargeFiles {
    fn run(&self, files: &[PathBuf]) -> Result<(), HookError> {
        for file in files {
            // Get the file size
            let metadata = fs::metadata(file)?;
            let size_kb = metadata.len() as usize / 1024;

            // Check if the file is too large
            if size_kb > self.max_size_kb {
                return Err(HookError::Other(format!("File {} is too large ({} KB > {} KB)", file.display(), size_kb, self.max_size_kb)));
            }
        }

        Ok(())
    }
}

/// Check for merge conflicts
pub struct CheckMergeConflict;

impl Hook for CheckMergeConflict {
    fn run(&self, files: &[PathBuf]) -> Result<(), HookError> {
        for file in files {
            // Read the file
            let content = fs::read_to_string(file)?;

            // Check for merge conflict markers
            if content.contains("<<<<<<<") || content.contains("=======") || content.contains(">>>>>>>") {
                return Err(HookError::Other(format!("Merge conflict markers found in {}", file.display())));
            }
        }

        Ok(())
    }
}

/// Check JSON files for parseable syntax
pub struct CheckJson;

impl Hook for CheckJson {
    fn run(&self, files: &[PathBuf]) -> Result<(), HookError> {
        for file in files {
            // Read the file
            let content = fs::read_to_string(file)?;

            // Try to parse the JSON
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(_) => continue,
                Err(err) => return Err(HookError::Other(format!("Invalid JSON in {}: {}", file.display(), err))),
            }
        }

        Ok(())
    }
}

/// Check TOML files for parseable syntax
pub struct CheckToml;

impl Hook for CheckToml {
    fn run(&self, files: &[PathBuf]) -> Result<(), HookError> {
        for file in files {
            // Read the file
            let content = fs::read_to_string(file)?;

            // This is a simple check that looks for basic TOML syntax errors
            // A more robust solution would use a proper TOML parser

            // Check for key-value pairs
            let mut has_key_value = false;

            for line in content.lines() {
                let line = line.trim();

                // Skip empty lines and comments
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                // Check for key-value pairs
                if line.contains('=') {
                    has_key_value = true;
                    break;
                }

                // Check for section headers
                if line.starts_with('[') && line.ends_with(']') {
                    continue;
                }

                // If we get here, the line is not a valid TOML construct
                return Err(HookError::Other(format!("Invalid TOML in {}: unexpected line format", file.display())));
            }

            if !has_key_value && !content.is_empty() {
                return Err(HookError::Other(format!("Invalid TOML in {}: no key-value pairs found", file.display())));
            }
        }

        Ok(())
    }
}

/// Check XML files for parseable syntax
pub struct CheckXml;

impl Hook for CheckXml {
    fn run(&self, files: &[PathBuf]) -> Result<(), HookError> {
        for file in files {
            // Read the file
            let content = fs::read_to_string(file)?;

            // Try to parse the XML
            // This is a simple check that looks for basic XML syntax errors
            // A more robust solution would use a proper XML parser
            if !content.contains("<") || !content.contains(">") {
                return Err(HookError::Other(format!("Invalid XML in {}: missing tags", file.display())));
            }

            // Check for mismatched tags (very basic check)
            let mut open_tags = 0;
            let mut close_tags = 0;

            for c in content.chars() {
                if c == '<' {
                    open_tags += 1;
                } else if c == '>' {
                    close_tags += 1;
                }
            }

            if open_tags != close_tags {
                return Err(HookError::Other(format!("Invalid XML in {}: mismatched tags", file.display())));
            }
        }

        Ok(())
    }
}

/// Check for files with names that would conflict on a case-insensitive filesystem
pub struct CheckCaseConflict;

impl Hook for CheckCaseConflict {
    fn run(&self, files: &[PathBuf]) -> Result<(), HookError> {
        let mut lowercase_names = HashSet::new();
        let mut conflicts = Vec::new();

        for file in files {
            let filename = file.file_name()
                .ok_or_else(|| HookError::Other(format!("Invalid file name: {}", file.display())))?
                .to_string_lossy()
                .to_lowercase();

            if lowercase_names.contains(&filename) {
                conflicts.push(file.clone());
            } else {
                lowercase_names.insert(filename);
            }
        }

        if !conflicts.is_empty() {
            let conflict_list = conflicts.iter()
                .map(|f| f.display().to_string())
                .collect::<Vec<_>>()
                .join(", ");

            return Err(HookError::Other(format!("Case-insensitive filename conflicts found: {}", conflict_list)));
        }

        Ok(())
    }
}

/// Detect private keys
pub struct DetectPrivateKey;

impl Hook for DetectPrivateKey {
    fn run(&self, files: &[PathBuf]) -> Result<(), HookError> {
        // Patterns that indicate a private key
        let patterns = [
            "-----BEGIN RSA PRIVATE KEY-----",
            "-----BEGIN DSA PRIVATE KEY-----",
            "-----BEGIN EC PRIVATE KEY-----",
            "-----BEGIN OPENSSH PRIVATE KEY-----",
            "-----BEGIN PRIVATE KEY-----",
            "PuTTY-User-Key-File-",
        ];

        for file in files {
            // Read the file
            let content = fs::read_to_string(file)?;

            // Check for private key patterns
            for pattern in &patterns {
                if content.contains(pattern) {
                    return Err(HookError::Other(format!("Private key found in {}", file.display())));
                }
            }
        }

        Ok(())
    }
}


/// Factory for creating hooks
pub struct HookFactory;

impl HookFactory {
    /// Create a hook by ID
    pub fn create_hook(id: &str, args: &[String]) -> Result<Box<dyn Hook>, HookError> {
        match id {
            "trailing-whitespace" => Ok(Box::new(TrailingWhitespace)),
            "end-of-file-fixer" => Ok(Box::new(EndOfFileFixer)),
            "check-yaml" => Ok(Box::new(CheckYaml)),
            "check-added-large-files" => {
                // Parse the max size argument
                let max_size_kb = if let Some(arg) = args.iter().find(|a| a.starts_with("--maxkb=")) {
                    arg.trim_start_matches("--maxkb=").parse::<usize>().unwrap_or(500)
                } else {
                    500 // Default to 500 KB
                };

                Ok(Box::new(CheckAddedLargeFiles::new(max_size_kb)))
            },
            "check-merge-conflict" => Ok(Box::new(CheckMergeConflict)),
            _ => Err(HookError::Other(format!("Unknown hook ID: {}", id))),
        }
    }
}
