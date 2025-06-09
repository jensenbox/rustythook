//! Implementation of the check-case-conflict hook

use std::path::PathBuf;
use std::collections::HashSet;
use crate::hooks::common::{Hook, HookError};

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