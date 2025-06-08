//! Logging module for RustyHook
//!
//! This module provides logging functionality for RustyHook, including
//! the ability to log to a file or other outputs in addition to stdout.

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use env_logger::{Builder, Env};

/// Initialize the logger with the specified configuration
///
/// # Arguments
///
/// * `log_file` - Optional path to a log file. If provided, logs will be written to this file
///                in addition to stdout.
/// * `log_level` - The log level to use. If not provided, defaults to "info".
///
/// # Returns
///
/// * `Result<(), String>` - Ok if the logger was initialized successfully, Err otherwise
pub fn init(log_file: Option<PathBuf>, log_level: Option<&str>) -> Result<(), String> {
    let env = Env::default()
        .filter_or("RUSTYHOOK_LOG_LEVEL", log_level.unwrap_or("info"));

    let mut builder = Builder::from_env(env);

    // Set the default format
    builder.format(|buf, record| {
        writeln!(
            buf,
            "{} [{}] - {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            record.args()
        )
    });

    // If a log file is provided, add a file logger
    if let Some(log_file_path) = log_file {
        // Create the parent directory if it doesn't exist
        if let Some(parent) = log_file_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create log directory: {}", e))?;
        }

        // Open the log file
        let file = File::create(&log_file_path)
            .map_err(|e| format!("Failed to create log file: {}", e))?;

        // Add the file logger
        builder.target(env_logger::Target::Pipe(Box::new(file)));
    }

    // Initialize the logger
    builder.init();

    Ok(())
}

/// Get the default log file path
///
/// Returns a path to the default log file location, which is
/// `.rustyhook/logs/rustyhook.log` in the current directory.
pub fn default_log_file() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".rustyhook")
        .join("logs")
        .join("rustyhook.log")
}
