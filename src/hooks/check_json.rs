//! Implementation of the check-json hook

use std::path::PathBuf;
use std::fs;
use crate::hooks::common::{Hook, HookError};

/// Check JSON files for parseable syntax
pub struct CheckJson;

impl Hook for CheckJson {
    fn run(&self, files: &[PathBuf]) -> Result<(), HookError> {
        for file in files {
            // Read the file
            let content = fs::read(file)?;
            let content_str = String::from_utf8_lossy(&content);

            // Try to parse the JSON
            match serde_json::from_str::<serde_json::Value>(&content_str) {
                Ok(_) => continue,
                Err(err) => return Err(HookError::Other(format!("Invalid JSON in {}: {}", file.display(), err))),
            }
        }

        Ok(())
    }
}