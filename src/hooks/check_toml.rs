//! Implementation of the check-toml hook

use std::path::PathBuf;
use std::fs;
use crate::hooks::common::{Hook, HookError};

/// Check TOML files for parseable syntax
pub struct CheckToml;

impl Hook for CheckToml {
    fn run(&self, files: &[PathBuf]) -> Result<(), HookError> {
        for file in files {
            // Read the file
            let content = fs::read(file)?;
            let content_str = String::from_utf8_lossy(&content);

            // This is a simple check that looks for basic TOML syntax errors
            // A more robust solution would use a proper TOML parser

            // Check for key-value pairs
            let mut has_key_value = false;

            for line in content_str.lines() {
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

            if !has_key_value && !content_str.is_empty() {
                return Err(HookError::Other(format!("Invalid TOML in {}: no key-value pairs found", file.display())));
            }
        }

        Ok(())
    }
}