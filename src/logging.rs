//! Logging module for RustyHook
//!
//! This module provides logging functionality for RustyHook, including
//! the ability to log to a file or other outputs in addition to stdout.
//!
//! ## Log Levels
//!
//! The following log levels are available, in order of increasing verbosity:
//!
//! - `error`: Only errors are logged
//! - `warn`: Errors and warnings are logged
//! - `info`: Errors, warnings, and informational messages are logged (default)
//! - `debug`: Errors, warnings, informational messages, and debug messages are logged
//! - `trace`: All messages are logged, including detailed tracing information
//!
//! The log level can be set via:
//! - The `--log-level` command line argument
//! - The `RUSTYHOOK_LOG_LEVEL` environment variable
//! - The `log_level` parameter in the `init` function
//!
//! ## Module-Specific Log Levels
//!
//! You can set different log levels for different modules using the `RUSTYHOOK_LOG_MODULES` 
//! environment variable. The format is a comma-separated list of `module=level` pairs.
//!
//! For example:
//!
//! ```
//! # Set the default log level to info
//! export RUSTYHOOK_LOG_LEVEL=info
//!
//! # Set module-specific log levels
//! export RUSTYHOOK_LOG_MODULES=rustyhook::runner=debug,rustyhook::hooks=trace
//! ```
//!
//! This will set the log level for the `rustyhook::runner` module to `debug` and the
//! log level for the `rustyhook::hooks` module to `trace`, while keeping the default
//! log level for all other modules at `info`.

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use env_logger::Builder;
use log::LevelFilter;

/// Parse a log level string into a LevelFilter
///
/// # Arguments
///
/// * `level` - The log level string to parse
///
/// # Returns
///
/// * `Result<LevelFilter, String>` - The parsed log level or an error message
pub fn parse_log_level(level: &str) -> Result<LevelFilter, String> {
    match level.to_lowercase().as_str() {
        "error" => Ok(LevelFilter::Error),
        "warn" => Ok(LevelFilter::Warn),
        "info" => Ok(LevelFilter::Info),
        "debug" => Ok(LevelFilter::Debug),
        "trace" => Ok(LevelFilter::Trace),
        "off" => Ok(LevelFilter::Off),
        _ => Err(format!("Invalid log level: {}. Valid levels are: error, warn, info, debug, trace, off", level))
    }
}

/// Initialize the logger with the specified configuration
///
/// # Arguments
///
/// * `log_file` - Optional path to a log file. If provided, logs will be written to this file
///                in addition to stdout.
/// * `log_level` - The log level to use. If not provided, defaults to "info".
///                 Valid values are: error, warn, info, debug, trace, off
///
/// # Returns
///
/// * `Result<(), String>` - Ok if the logger was initialized successfully, Err otherwise
///
/// # Examples
///
/// ```
/// use rustyhook::logging;
/// use std::path::PathBuf;
///
/// // Initialize with default log level (info)
/// logging::init(None, None).unwrap();
///
/// // Initialize with debug log level
/// logging::init(None, Some("debug")).unwrap();
///
/// // Initialize with log file
/// logging::init(Some(PathBuf::from("rustyhook.log")), Some("info")).unwrap();
/// ```
pub fn init(log_file: Option<PathBuf>, log_level: Option<&str>) -> Result<(), String> {
    // Get the log level from the parameter or environment variable
    let level_str = match log_level {
        Some(level) => level.to_string(),
        None => std::env::var("RUSTYHOOK_LOG_LEVEL").unwrap_or_else(|_| "info".to_string())
    };

    // Check for module-specific log levels in the environment
    let module_filter = std::env::var("RUSTYHOOK_LOG_MODULES").ok();

    // Parse and validate the log level
    let level_filter = parse_log_level(&level_str)?;

    // Create a builder with the validated log level
    let mut builder = Builder::new();

    // Apply module-specific log levels if provided
    if let Some(ref filter) = module_filter {
        builder.parse_filters(filter);
    } else {
        // Otherwise, apply the global log level
        builder.filter_level(level_filter);
    }

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

    // Always log to stdout
    builder.target(env_logger::Target::Stdout);

    // If a log file is provided, also log to the file
    if let Some(log_file_path) = log_file {
        // Create the parent directory if it doesn't exist
        if let Some(parent) = log_file_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create log directory: {}", e))?;
        }

        // Open the log file
        let file = File::create(&log_file_path)
            .map_err(|e| format!("Failed to create log file: {}", e))?;

        // Create a separate builder for the file logger
        let mut file_builder = Builder::new();

        // Apply the same filter level
        if let Some(filter) = &module_filter {
            file_builder.parse_filters(filter);
        } else {
            file_builder.filter_level(level_filter);
        }

        // Set the same format
        file_builder.format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        });

        // Set the file as the target
        file_builder.target(env_logger::Target::Pipe(Box::new(file)));

        // Initialize the file logger with a unique name
        file_builder.init();
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
