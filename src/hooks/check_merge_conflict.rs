//! Implementation of the check-merge-conflict hook

use std::path::PathBuf;
use std::fs;
use crate::hooks::common::{Hook, HookError};

/// Check for merge conflicts
pub struct CheckMergeConflict;

impl Hook for CheckMergeConflict {
    fn run(&self, files: &[PathBuf]) -> Result<(), HookError> {
        for file in files {
            // Read the file
            let content = fs::read(file)?;
            let content_str = String::from_utf8_lossy(&content);

            // Check for merge conflict markers
            if content_str.contains("<<<<<<<") || content_str.contains("=======") || content_str.contains(">>>>>>>") {
                return Err(HookError::Other(format!("Merge conflict markers found in {}", file.display())));
            }
        }

        Ok(())
    }
}