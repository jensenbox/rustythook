//! Native implementations of pre-commit hooks
//!
//! This module provides native Rust implementations of the hooks from
//! https://github.com/pre-commit/pre-commit-hooks

// Re-export common types
mod common;
pub use common::{Hook, HookError};

// Import individual hook implementations
mod trailing_whitespace;
mod end_of_file_fixer;
mod check_yaml;
mod check_added_large_files;
mod check_merge_conflict;
mod check_json;
mod check_toml;
mod check_xml;
mod check_case_conflict;
mod detect_private_key;

// Re-export hook implementations
pub use trailing_whitespace::TrailingWhitespace;
pub use end_of_file_fixer::EndOfFileFixer;
pub use check_yaml::CheckYaml;
pub use check_added_large_files::CheckAddedLargeFiles;
pub use check_merge_conflict::CheckMergeConflict;
pub use check_json::CheckJson;
pub use check_toml::CheckToml;
pub use check_xml::CheckXml;
pub use check_case_conflict::CheckCaseConflict;
pub use detect_private_key::DetectPrivateKey;

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
            "check-json" => Ok(Box::new(CheckJson)),
            "check-toml" => Ok(Box::new(CheckToml)),
            "check-xml" => Ok(Box::new(CheckXml)),
            "check-case-conflict" => Ok(Box::new(CheckCaseConflict)),
            "detect-private-key" => Ok(Box::new(DetectPrivateKey)),
            _ => Err(HookError::Other(format!("Unknown hook ID: {}", id))),
        }
    }
}
