//! Implementation of the trailing-whitespace hook

use std::path::PathBuf;
use std::fs;
use crate::hooks::common::{Hook, HookError};

/// Trim trailing whitespace
pub struct TrailingWhitespace;

impl Hook for TrailingWhitespace {
    fn run(&self, files: &[PathBuf]) -> Result<(), HookError> {
        for file in files {
            // Read the file
            let content = match fs::read(file) {
                Ok(content) => content,
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::PermissionDenied {
                        // Skip files that can't be accessed due to permission issues
                        log::warn!("Skipping file due to permission denied: {}", file.display());
                        continue;
                    } else {
                        return Err(HookError::IoError(e));
                    }
                }
            };
            let content = String::from_utf8_lossy(&content);

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
                if let Err(e) = fs::write(file, new_content) {
                    if e.kind() == std::io::ErrorKind::PermissionDenied {
                        // Skip files that can't be written to due to permission issues
                        log::warn!("Skipping file write due to permission denied: {}", file.display());
                        continue;
                    } else {
                        return Err(HookError::IoError(e));
                    }
                }
            }
        }

        Ok(())
    }
}