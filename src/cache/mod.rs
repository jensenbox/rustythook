//! Cache module for RustyHook
//!
//! This module provides functionality for caching environments and tools.

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

/// Error type for cache operations
#[derive(Debug)]
pub enum CacheError {
    /// Error with the file system
    IoError(std::io::Error),
    /// Error with serialization
    SerializationError(serde_yaml::Error),
}

impl From<std::io::Error> for CacheError {
    fn from(err: std::io::Error) -> Self {
        CacheError::IoError(err)
    }
}

impl From<serde_yaml::Error> for CacheError {
    fn from(err: serde_yaml::Error) -> Self {
        CacheError::SerializationError(err)
    }
}

/// Represents a cache manager
pub struct CacheManager {
    /// Cache directory
    cache_dir: PathBuf,
    /// Maximum age of cache entries
    max_age: Duration,
}

impl CacheManager {
    /// Create a new cache manager
    pub fn new(cache_dir: PathBuf, max_age: Duration) -> Self {
        CacheManager {
            cache_dir,
            max_age,
        }
    }
    
    /// Initialize the cache directory
    pub fn init(&self) -> Result<(), CacheError> {
        fs::create_dir_all(&self.cache_dir)?;
        Ok(())
    }
    
    /// Check if a cache entry exists and is valid
    pub fn is_valid(&self, key: &str) -> bool {
        let path = self.cache_dir.join(key);
        
        // Check if the cache entry exists
        if !path.exists() {
            return false;
        }
        
        // Check if the cache entry is too old
        if let Ok(metadata) = fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    return elapsed < self.max_age;
                }
            }
        }
        
        false
    }
    
    /// Get a cache entry
    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>, CacheError> {
        let path = self.cache_dir.join(key);
        
        // Check if the cache entry exists and is valid
        if !self.is_valid(key) {
            return Ok(None);
        }
        
        // Read the cache entry
        let data = fs::read_to_string(path)?;
        let value = serde_yaml::from_str(&data)?;
        
        Ok(Some(value))
    }
    
    /// Set a cache entry
    pub fn set<T: serde::Serialize>(&self, key: &str, value: &T) -> Result<(), CacheError> {
        let path = self.cache_dir.join(key);
        
        // Create the parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Write the cache entry
        let data = serde_yaml::to_string(value)?;
        fs::write(path, data)?;
        
        Ok(())
    }
    
    /// Remove a cache entry
    pub fn remove(&self, key: &str) -> Result<(), CacheError> {
        let path = self.cache_dir.join(key);
        
        // Remove the cache entry if it exists
        if path.exists() {
            fs::remove_file(path)?;
        }
        
        Ok(())
    }
    
    /// Clear all cache entries
    pub fn clear(&self) -> Result<(), CacheError> {
        // Remove all files in the cache directory
        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                fs::remove_file(path)?;
            } else if path.is_dir() {
                fs::remove_dir_all(path)?;
            }
        }
        
        Ok(())
    }
    
    /// Invalidate cache entries that are too old
    pub fn invalidate(&self) -> Result<(), CacheError> {
        // Remove cache entries that are too old
        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Ok(metadata) = fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(elapsed) = modified.elapsed() {
                            if elapsed > self.max_age {
                                fs::remove_file(path)?;
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
}