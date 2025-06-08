//! File matcher for RustyHook
//!
//! This module provides functionality for matching files against patterns.

use std::path::{Path, PathBuf};
use regex::Regex;
use globset::{Glob, GlobSet, GlobSetBuilder};

/// Error type for file matcher operations
#[derive(Debug)]
pub enum FileMatcherError {
    /// Error with regex pattern
    RegexError(regex::Error),
    /// Error with glob pattern
    GlobError(globset::Error),
}

impl From<regex::Error> for FileMatcherError {
    fn from(err: regex::Error) -> Self {
        FileMatcherError::RegexError(err)
    }
}

impl From<globset::Error> for FileMatcherError {
    fn from(err: globset::Error) -> Self {
        FileMatcherError::GlobError(err)
    }
}

/// Represents a file matcher
#[derive(Debug)]
pub enum FileMatcher {
    /// Match files using a regex pattern
    Regex(Regex),
    /// Match files using a glob pattern
    Glob(GlobSet),
}

impl FileMatcher {
    /// Create a new file matcher from a regex pattern
    pub fn from_regex(pattern: &str) -> Result<Self, FileMatcherError> {
        let regex = Regex::new(pattern)?;
        Ok(FileMatcher::Regex(regex))
    }
    
    /// Create a new file matcher from a glob pattern
    pub fn from_glob(pattern: &str) -> Result<Self, FileMatcherError> {
        let mut builder = GlobSetBuilder::new();
        builder.add(Glob::new(pattern)?);
        let globset = builder.build()?;
        Ok(FileMatcher::Glob(globset))
    }
    
    /// Create a new file matcher from multiple glob patterns
    pub fn from_globs(patterns: &[String]) -> Result<Self, FileMatcherError> {
        let mut builder = GlobSetBuilder::new();
        for pattern in patterns {
            builder.add(Glob::new(pattern)?);
        }
        let globset = builder.build()?;
        Ok(FileMatcher::Glob(globset))
    }
    
    /// Check if a file matches the pattern
    pub fn matches(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        match self {
            FileMatcher::Regex(regex) => regex.is_match(&path_str),
            FileMatcher::Glob(globset) => globset.is_match(path),
        }
    }
    
    /// Filter a list of files to only those that match the pattern
    pub fn filter_files(&self, files: &[PathBuf]) -> Vec<PathBuf> {
        files.iter()
            .filter(|path| self.matches(path))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_regex_matcher() {
        let matcher = FileMatcher::from_regex(r".*\.rs$").unwrap();
        assert!(matcher.matches(Path::new("src/main.rs")));
        assert!(!matcher.matches(Path::new("src/main.py")));
    }
    
    #[test]
    fn test_glob_matcher() {
        let matcher = FileMatcher::from_glob("**/*.rs").unwrap();
        assert!(matcher.matches(Path::new("src/main.rs")));
        assert!(!matcher.matches(Path::new("src/main.py")));
    }
    
    #[test]
    fn test_filter_files() {
        let matcher = FileMatcher::from_regex(r".*\.rs$").unwrap();
        let files = vec![
            PathBuf::from("src/main.rs"),
            PathBuf::from("src/lib.rs"),
            PathBuf::from("src/main.py"),
        ];
        let filtered = matcher.filter_files(&files);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains(&PathBuf::from("src/main.rs")));
        assert!(filtered.contains(&PathBuf::from("src/lib.rs")));
        assert!(!filtered.contains(&PathBuf::from("src/main.py")));
    }
}