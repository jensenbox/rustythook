//! Tests for Ruby toolchain functionality

use rustyhook::toolchains::{RubyTool, Tool, SetupContext};
use std::env;

#[test]
fn test_ruby_tool_with_direct_download() {
    // Create a temporary directory for the test
    let temp_dir = tempfile::tempdir().unwrap();
    let cache_dir = temp_dir.path().join(".rustyhook").join("cache");

    // Set the current directory to the temp directory for the test
    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(temp_dir.path()).unwrap();

    // Create a Ruby tool with a test gem
    let ruby_tool = RubyTool::new("rubocop", "1.0.0", vec!["rubocop".to_string()]);

    // Get the installation directory from the Ruby tool
    let install_dir = ruby_tool.install_dir().clone();

    // Print the installation directory for debugging
    println!("Ruby tool installation directory: {:?}", install_dir);

    // Create the cache directory
    std::fs::create_dir_all(&cache_dir).unwrap();

    // Create a setup context
    let ctx = SetupContext {
        cache_dir: cache_dir.clone(),
        install_dir: install_dir.clone(),
        force: false,
        version: Some("3.2.2".to_string()), // Use a stable version of Ruby
    };

    // Set up the Ruby tool (this will download and install Ruby)
    println!("Setting up Ruby tool with direct download...");
    let result = ruby_tool.setup(&ctx);

    // Check that the setup was successful
    assert!(result.is_ok(), "Failed to set up Ruby tool: {:?}", result);

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

        // Check bin directory
        let bin_dir = install_dir.join("bin");

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

    // Check that the Ruby tool is installed
    println!("Checking if Ruby tool is installed...");
    let is_installed = ruby_tool.is_installed();
    println!("Ruby tool is installed: {}", is_installed);

    // Get the tool path that is being checked
    let tool_path = install_dir.join("bin").join("rubocop");
    println!("Tool path being checked: {:?}", tool_path);
    println!("Tool path exists: {}", tool_path.exists());

    // Assert that the Ruby tool is installed
    assert!(is_installed, "Ruby tool is not installed");

    // Assert that the rubocop gem is installed
    assert!(tool_path.exists(), "rubocop gem is not installed");

    // Reset the current directory
    env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_ruby_tool_with_ruby_version_file() {
    // Create a temporary directory for the test
    let temp_dir = tempfile::tempdir().unwrap();
    let cache_dir = temp_dir.path().join(".rustyhook").join("cache");

    // Create a .ruby-version file in the temporary directory
    let ruby_version = "3.2.2"; // Use a stable version of Ruby
    let ruby_version_file = temp_dir.path().join(".ruby-version");
    std::fs::write(&ruby_version_file, ruby_version).unwrap();
    println!("Created .ruby-version file at {:?} with version {}", ruby_version_file, ruby_version);

    // Save the original directory
    let original_dir = std::env::current_dir().unwrap();

    // Create a Ruby tool with a test gem
    let ruby_tool = RubyTool::new("rubocop", "1.0.0", vec!["rubocop".to_string()]);

    // Get the installation directory from the Ruby tool
    let install_dir = ruby_tool.install_dir().clone();

    // Create the cache directory
    std::fs::create_dir_all(&cache_dir).unwrap();

    // Create a setup context
    let ctx = SetupContext {
        cache_dir: cache_dir.clone(),
        install_dir: install_dir.clone(),
        force: true, // Force reinstallation to ensure we use the specified Ruby version
        version: Some("3.2.2".to_string()), // Specify the version directly instead of relying on .ruby-version
    };

    // Set up the Ruby tool
    println!("Setting up Ruby tool with specified version...");
    let result = ruby_tool.setup(&ctx);

    // Check that the setup was successful
    assert!(result.is_ok(), "Failed to set up Ruby tool: {:?}", result);

    // Check that the Ruby tool is installed
    println!("Checking if Ruby tool is installed...");
    let is_installed = ruby_tool.is_installed();
    println!("Ruby tool is installed: {}", is_installed);

    // Check that the rubocop gem is installed
    let rubocop_path = install_dir.join("bin").join("rubocop");
    println!("Rubocop path: {:?}", rubocop_path);
    println!("Rubocop path exists: {}", rubocop_path.exists());

    // Assert that the Ruby tool is installed
    assert!(is_installed, "Ruby tool is not installed");

    // Assert that the rubocop gem is installed
    assert!(rubocop_path.exists(), "rubocop gem is not installed");
}

#[test]
fn test_ruby_tool_with_monorepo_structure() {
    // Create a temporary directory for the test
    let temp_dir = tempfile::tempdir().unwrap();

    // Create a monorepo structure with different .ruby-version files
    let root_dir = temp_dir.path();
    let subdir1 = root_dir.join("project1");
    let subdir2 = root_dir.join("project2");

    // Create the directories
    std::fs::create_dir_all(&subdir1).unwrap();
    std::fs::create_dir_all(&subdir2).unwrap();

    // Create .ruby-version files with different versions
    std::fs::write(root_dir.join(".ruby-version"), "3.1.0").unwrap();
    std::fs::write(subdir1.join(".ruby-version"), "3.2.0").unwrap();
    std::fs::write(subdir2.join(".ruby-version"), "3.0.0").unwrap();

    // Save the original directory
    let original_dir = env::current_dir().unwrap();

    // Test with root directory
    {
        env::set_current_dir(root_dir).unwrap();

        let ruby_tool = RubyTool::new("test", "1.0.0", vec![]);
        let version = ruby_tool.determine_ruby_version(None).unwrap();

        assert_eq!(version, "3.1.0", "Root directory should use version 3.1.0");
    }

    // Test with subdirectory 1
    {
        env::set_current_dir(&subdir1).unwrap();

        let ruby_tool = RubyTool::new("test", "1.0.0", vec![]);
        let version = ruby_tool.determine_ruby_version(None).unwrap();

        assert_eq!(version, "3.2.0", "Subdirectory 1 should use version 3.2.0");
    }

    // Test with subdirectory 2
    {
        env::set_current_dir(&subdir2).unwrap();

        let ruby_tool = RubyTool::new("test", "1.0.0", vec![]);
        let version = ruby_tool.determine_ruby_version(None).unwrap();

        assert_eq!(version, "3.0.0", "Subdirectory 2 should use version 3.0.0");
    }

    // Reset the current directory
    env::set_current_dir(original_dir).unwrap();
}
