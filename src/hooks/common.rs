//! Common types and traits for hooks

use std::path::PathBuf;
use std::io;

/// Error type for hook operations
#[derive(Debug)]
pub enum HookError {
    /// IO error
    IoError(io::Error),
    /// Invalid UTF-8
    Utf8Error(std::string::FromUtf8Error),
    /// Other error
    Other(String),
}

impl From<io::Error> for HookError {
    fn from(err: io::Error) -> Self {
        HookError::IoError(err)
    }
}

impl From<std::string::FromUtf8Error> for HookError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        HookError::Utf8Error(err)
    }
}

/// Trait for hooks
pub trait Hook {
    /// Run the hook on files
    fn run(&self, files: &[PathBuf]) -> Result<(), HookError>;
}