//! Unit tests for the command line interface

use std::path::PathBuf;
use std::process::Command;
use std::str;
use std::env;

// Helper function to run the CLI with arguments
fn run_cli(args: &[&str]) -> Result<(String, String, i32), Box<dyn std::error::Error>> {
    let rustyhook_bin = env::current_exe()?
        .parent().unwrap()
        .parent().unwrap()
        .join("rustyhook");
    
    let output = Command::new(rustyhook_bin)
        .args(args)
        .output()?;
    
    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    let status = output.status.code().unwrap_or(-1);
    
    Ok((stdout, stderr, status))
}

#[test]
fn test_run_command() {
    // Test the 'run' command
    let result = run_cli(&["run"]);
    assert!(result.is_ok());
    
    let (stdout, stderr, status) = result.unwrap();
    assert!(stdout.contains("Running hooks using native config..."));
    // Note: The actual result might vary depending on whether a config file exists
}

#[test]
fn test_compat_command() {
    // Test the 'compat' command
    let result = run_cli(&["compat"]);
    assert!(result.is_ok());
    
    let (stdout, stderr, status) = result.unwrap();
    assert!(stdout.contains("Running hooks using .pre-commit-config.yaml..."));
    // Note: The actual result might vary depending on whether a pre-commit config file exists
}

#[test]
fn test_convert_command() {
    // Test the 'convert' command without arguments
    let result = run_cli(&["convert"]);
    assert!(result.is_ok());
    
    let (stdout, stderr, status) = result.unwrap();
    assert!(stdout.contains("Please specify --from-precommit"));
    
    // Test the 'convert' command with --from-precommit
    let result = run_cli(&["convert", "--from-precommit"]);
    assert!(result.is_ok());
    
    let (stdout, stderr, status) = result.unwrap();
    assert!(stdout.contains("Converting from .pre-commit-config.yaml"));
    // Note: The actual result might vary depending on whether a pre-commit config file exists
}

#[test]
fn test_init_command() {
    // Create a temporary directory for the test
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = env::current_dir().unwrap();
    
    // Change to the temporary directory
    env::set_current_dir(&temp_dir).unwrap();
    
    // Test the 'init' command
    let result = run_cli(&["init"]);
    assert!(result.is_ok());
    
    let (stdout, stderr, status) = result.unwrap();
    assert!(stdout.contains("Creating starter .rustyhook/config.yaml..."));
    
    // Check if the config file was created
    let config_path = temp_dir.path().join(".rustyhook").join("config.yaml");
    assert!(config_path.exists());
    
    // Change back to the original directory
    env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_list_command() {
    // Test the 'list' command
    let result = run_cli(&["list"]);
    assert!(result.is_ok());
    
    let (stdout, stderr, status) = result.unwrap();
    assert!(stdout.contains("Listing all available hooks"));
    // Note: The actual result might vary depending on whether a config file exists
}

#[test]
fn test_doctor_command() {
    // Test the 'doctor' command
    let result = run_cli(&["doctor"]);
    assert!(result.is_ok());
    
    let (stdout, stderr, status) = result.unwrap();
    assert!(stdout.contains("Diagnosing issues with setup or environments"));
}

#[test]
fn test_clean_command() {
    // Test the 'clean' command
    let result = run_cli(&["clean"]);
    assert!(result.is_ok());
    
    let (stdout, stderr, status) = result.unwrap();
    assert!(stdout.contains("Removing cached environments and tool installs"));
}

#[test]
fn test_invalid_command() {
    // Test an invalid command
    let result = run_cli(&["invalid-command"]);
    
    // The command should fail with a non-zero exit code
    assert!(result.is_ok());
    let (stdout, stderr, status) = result.unwrap();
    assert_ne!(status, 0);
    assert!(stderr.contains("error:"));
}

#[test]
fn test_help_command() {
    // Test the '--help' flag
    let result = run_cli(&["--help"]);
    assert!(result.is_ok());
    
    let (stdout, stderr, status) = result.unwrap();
    assert_eq!(status, 0);
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("Commands:"));
    assert!(stdout.contains("run"));
    assert!(stdout.contains("compat"));
    assert!(stdout.contains("convert"));
    assert!(stdout.contains("init"));
    assert!(stdout.contains("list"));
    assert!(stdout.contains("doctor"));
    assert!(stdout.contains("clean"));
}

#[test]
fn test_version_command() {
    // Test the '--version' flag
    let result = run_cli(&["--version"]);
    assert!(result.is_ok());
    
    let (stdout, stderr, status) = result.unwrap();
    assert_eq!(status, 0);
    assert!(stdout.contains("rustyhook"));
}