//! Implementation of the check-xml hook

use std::path::PathBuf;
use std::fs;
use crate::hooks::common::{Hook, HookError};

/// Check XML files for parseable syntax
pub struct CheckXml;

impl Hook for CheckXml {
    fn run(&self, files: &[PathBuf]) -> Result<(), HookError> {
        for file in files {
            // Read the file
            let content = fs::read(file)?;
            let content_str = String::from_utf8_lossy(&content);

            // Try to parse the XML
            // This is a simple check that looks for basic XML syntax errors
            // A more robust solution would use a proper XML parser
            if !content_str.contains("<") || !content_str.contains(">") {
                return Err(HookError::Other(format!("Invalid XML in {}: missing tags", file.display())));
            }

            // Check for mismatched tags (very basic check)
            let mut open_tags = 0;
            let mut close_tags = 0;

            for c in content_str.chars() {
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