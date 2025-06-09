//! Implementation of the end-of-file-fixer hook

use std::path::PathBuf;
use std::fs;
use crate::hooks::common::{Hook, HookError};

/// Fix end of files
pub struct EndOfFileFixer;

impl Hook for EndOfFileFixer {
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
            let content_str = String::from_utf8_lossy(&content);

            // Check if the file is empty or ends with a newline
            if content_str.is_empty() || content_str.ends_with('\n') {
                continue;
            }

            // Fix the file
            let mut new_content = content_str.to_string();
            new_content.push('\n');
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

        Ok(())
    }
}