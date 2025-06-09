//! Implementation of the check-yaml hook

use std::path::PathBuf;
use std::fs;
use crate::hooks::common::{Hook, HookError};

/// Check YAML files for parseable syntax
pub struct CheckYaml;

impl Hook for CheckYaml {
    fn run(&self, files: &[PathBuf]) -> Result<(), HookError> {
        for file in files {
            // Read the file
            let content = fs::read(file)?;
            let content_str = String::from_utf8_lossy(&content);

            // Try to parse the YAML
            match serde_yaml::from_str::<serde_yaml::Value>(&content_str) {
                Ok(_) => continue,
                Err(err) => return Err(HookError::Other(format!("Invalid YAML in {}: {}", file.display(), err))),
            }
        }

        Ok(())
    }
}