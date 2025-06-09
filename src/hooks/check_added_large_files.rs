//! Implementation of the check-added-large-files hook

use std::path::PathBuf;
use std::fs;
use crate::hooks::common::{Hook, HookError};

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