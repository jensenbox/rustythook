//! Tests for Node.js toolchain functionality

use rustyhook::toolchains::{NodeTool, Tool, SetupContext};
use std::path::PathBuf;
use std::env;

#[test]
fn test_node_tool_with_direct_download() {
    // Create a temporary directory for the test
    let temp_dir = tempfile::tempdir().unwrap();
    let cache_dir = temp_dir.path().join(".rustyhook").join("cache");

    // Create a runtime directory for Node.js
    let runtime_dir = temp_dir.path().join(".runtime");
    std::fs::create_dir_all(&runtime_dir).unwrap();

    // Set the current directory to the temp directory for the test
    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(temp_dir.path()).unwrap();

    // Create a Node.js tool with a test package
    let node_tool = NodeTool::new("eslint", "1.0.0", vec!["eslint".to_string()], true, None);

    // Get the installation directory from the Node tool
    let install_dir = node_tool.install_dir().clone();

    // Create the cache directory
    std::fs::create_dir_all(&cache_dir).unwrap();

    // Create a setup context
    let ctx = SetupContext {
        cache_dir: cache_dir.clone(),
        install_dir: install_dir.clone(),
        force: false,
        version: Some("lts".to_string()), // Use LTS version of Node.js
    };

    // Set up the Node tool (this will download and install Node.js LTS)
    println!("Setting up Node tool with direct download...");
    let result = node_tool.setup(&ctx);

    // Check that the setup was successful
    assert!(result.is_ok(), "Failed to set up Node tool: {:?}", result);

    // Check the installation directory structure
    println!("Installation directory: {:?}", install_dir);
    if install_dir.exists() {
        println!("Installation directory exists");

        // List the contents of the installation directory
        if let Ok(entries) = std::fs::read_dir(&install_dir) {
            println!("Contents of installation directory:");
            for entry in entries {
                if let Ok(entry) = entry {
                    println!("  {:?}", entry.path());
                }
            }
        } else {
            println!("Failed to read installation directory");
        }

        // Check node_modules/.bin directory
        let bin_dir = install_dir.join("node_modules").join(".bin");

        if bin_dir.exists() {
            println!("Bin directory exists: {:?}", bin_dir);

            // List the contents of the bin directory
            if let Ok(entries) = std::fs::read_dir(&bin_dir) {
                println!("Contents of bin directory:");
                for entry in entries {
                    if let Ok(entry) = entry {
                        println!("  {:?}", entry.path());
                    }
                }
            } else {
                println!("Failed to read bin directory");
            }
        } else {
            println!("Bin directory does not exist: {:?}", bin_dir);
        }
    } else {
        println!("Installation directory does not exist");
    }

    // Check that the Node tool is installed
    println!("Checking if Node tool is installed...");
    let is_installed = node_tool.is_installed();
    println!("Node tool is installed: {}", is_installed);

    // Get the tool path that is being checked
    let tool_path = install_dir.join("node_modules").join(".bin").join("eslint");
    println!("Tool path being checked: {:?}", tool_path);
    println!("Tool path exists: {}", tool_path.exists());

    // Assert that the Node tool is installed
    assert!(is_installed, "Node tool is not installed");

    // Assert that the eslint package is installed
    assert!(tool_path.exists(), "eslint package is not installed");

    // Check that the Node.js runtime was installed
    let node_runtime_dir = PathBuf::from(".runtime").join("node");
    println!("Node.js runtime directory: {:?}", node_runtime_dir);
    assert!(node_runtime_dir.exists(), "Node.js runtime directory does not exist");

    // Reset the current directory
    env::set_current_dir(original_dir).unwrap();
}
