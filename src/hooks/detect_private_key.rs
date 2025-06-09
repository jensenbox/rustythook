//! Implementation of the detect-private-key hook

use std::path::PathBuf;
use std::fs;
use crate::hooks::common::{Hook, HookError};

/// Detect private keys
pub struct DetectPrivateKey;

impl Hook for DetectPrivateKey {
    fn run(&self, files: &[PathBuf]) -> Result<(), HookError> {
        // Patterns that indicate a private key
        let patterns = [
            "-----BEGIN RSA PRIVATE KEY-----",
            "-----BEGIN DSA PRIVATE KEY-----",
            "-----BEGIN EC PRIVATE KEY-----",
            "-----BEGIN OPENSSH PRIVATE KEY-----",
            "-----BEGIN PRIVATE KEY-----",
            "PuTTY-User-Key-File-",
        ];

        for file in files {
            // Read the file
            let content = fs::read(file)?;
            let content_str = String::from_utf8_lossy(&content);

            // Check for private key patterns
            for pattern in &patterns {
                if content_str.contains(pattern) {
                    return Err(HookError::Other(format!("Private key found in {}", file.display())));
                }
            }
        }

        Ok(())
    }
}