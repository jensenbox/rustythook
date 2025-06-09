//! Tests for toolchain functionality

use rustyhook::toolchains::{PythonTool, NodeTool, Tool, SetupContext};
use std::path::PathBuf;

#[test]
fn test_python_tool_with_uv() {
    // Create a temporary directory for the test
    let temp_dir = tempfile::tempdir().unwrap();
    let cache_dir = temp_dir.path().join(".rustyhook").join("cache");

    // Create a Python tool with a test package
    let python_tool = PythonTool::new("pytest", "1.0.0", vec!["pytest".to_string()]);

    // Get the installation directory from the Python tool
    let install_dir = python_tool.install_dir().clone();

    // Create the cache directory
    std::fs::create_dir_all(&cache_dir).unwrap();

    // Create a setup context
    let ctx = SetupContext {
        cache_dir: cache_dir.clone(),
        install_dir: install_dir.clone(),
        force: false,
        version: Some("1.0.0".to_string()),
    };

    // Set up the Python tool (this will install uv and use it to install pytest)
    let result = python_tool.setup(&ctx);

    // Check that the setup was successful
    assert!(result.is_ok(), "Failed to set up Python tool: {:?}", result);

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
        let bin_dir = if cfg!(windows) {
            install_dir.join("Scripts")
        } else {
            install_dir.join("bin")
        };

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

    // Check that the Python tool is installed
    println!("Checking if Python tool is installed...");
    let is_installed = python_tool.is_installed();
    println!("Python tool is installed: {}", is_installed);

    // Get the tool path that is being checked
    let tool_path = if cfg!(windows) {
        install_dir.join("Scripts").join(format!("{}.exe", python_tool.name()))
    } else {
        install_dir.join("bin").join(python_tool.name())
    };
    println!("Tool path being checked: {:?}", tool_path);
    println!("Tool path exists: {}", tool_path.exists());

    // Check that the pytest package is installed
    let pytest_path = if cfg!(windows) {
        install_dir.join("Scripts").join("pytest.exe")
    } else {
        install_dir.join("bin").join("pytest")
    };
    println!("Pytest path: {:?}", pytest_path);
    println!("Pytest path exists: {}", pytest_path.exists());

    // Assert that the Python tool is installed
    assert!(is_installed, "Python tool is not installed");

    // Assert that the pytest package is installed
    assert!(pytest_path.exists(), "pytest package is not installed");
}

#[test]
fn test_python_tool_with_python_version_file() {
    // Create a temporary directory for the test
    let temp_dir = tempfile::tempdir().unwrap();
    let cache_dir = temp_dir.path().join(".rustyhook").join("cache");

    // Create a .python-version file in the temporary directory
    let python_version = "3.9.18"; // Use a version that's compatible with python-build-standalone
    let python_version_file = temp_dir.path().join(".python-version");
    std::fs::write(&python_version_file, python_version).unwrap();
    println!("Created .python-version file at {:?} with version {}", python_version_file, python_version);

    // Change to the temporary directory to ensure .python-version is found
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();
    println!("Changed current directory to {:?}", temp_dir.path());

    // Create a Python tool with a test package
    let python_tool = PythonTool::new("black", "1.0.0", vec!["black".to_string()]);

    // Get the installation directory from the Python tool
    let install_dir = python_tool.install_dir().clone();

    // Create the cache directory
    std::fs::create_dir_all(&cache_dir).unwrap();

    // Create a setup context
    let ctx = SetupContext {
        cache_dir: cache_dir.clone(),
        install_dir: install_dir.clone(),
        force: true, // Force reinstallation to ensure we use the specified Python version
        version: Some("1.0.0".to_string()),
    };

    // Set up the Python tool (this should use the Python version from .python-version)
    println!("Setting up Python tool with .python-version file...");
    let result = python_tool.setup(&ctx);

    // Change back to the original directory
    std::env::set_current_dir(original_dir).unwrap();

    // Check that the setup was successful
    assert!(result.is_ok(), "Failed to set up Python tool: {:?}", result);

    // Check that the Python tool is installed
    println!("Checking if Python tool is installed...");
    let is_installed = python_tool.is_installed();
    println!("Python tool is installed: {}", is_installed);

    // Check that the black package is installed
    let black_path = if cfg!(windows) {
        install_dir.join("Scripts").join("black.exe")
    } else {
        install_dir.join("bin").join("black")
    };
    println!("Black path: {:?}", black_path);
    println!("Black path exists: {}", black_path.exists());

    // Assert that the Python tool is installed
    assert!(is_installed, "Python tool is not installed");

    // Assert that the black package is installed
    assert!(black_path.exists(), "black package is not installed");
}

#[test]
fn test_python_build_standalone() {
    // Create a temporary directory for the test
    let temp_dir = tempfile::tempdir().unwrap();
    let cache_dir = temp_dir.path().join(".rustyhook").join("cache");

    // Create a Python tool with a test package
    let python_tool = PythonTool::new("black", "1.0.0", vec!["black".to_string()]);

    // Get the installation directory from the Python tool
    let install_dir = python_tool.install_dir().clone();

    // Create the cache directory
    std::fs::create_dir_all(&cache_dir).unwrap();

    // Create a setup context with force=true to ensure we download python-build-standalone
    let ctx = SetupContext {
        cache_dir: cache_dir.clone(),
        install_dir: install_dir.clone(),
        force: true,
        version: Some("1.0.0".to_string()),
    };

    // Set up the Python tool (this will download python-build-standalone and use it to install black)
    let result = python_tool.setup(&ctx);

    // Check that the setup was successful
    assert!(result.is_ok(), "Failed to set up Python tool: {:?}", result);

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
        let bin_dir = if cfg!(windows) {
            install_dir.join("Scripts")
        } else {
            install_dir.join("bin")
        };

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

    // Check that the Python tool is installed
    println!("Checking if Python tool is installed...");
    let is_installed = python_tool.is_installed();
    println!("Python tool is installed: {}", is_installed);

    // Get the tool path that is being checked
    let tool_path = if cfg!(windows) {
        install_dir.join("Scripts").join(format!("{}.exe", python_tool.name()))
    } else {
        install_dir.join("bin").join(python_tool.name())
    };
    println!("Tool path being checked: {:?}", tool_path);
    println!("Tool path exists: {}", tool_path.exists());

    // Check that the black package is installed
    let black_path = if cfg!(windows) {
        install_dir.join("Scripts").join("black.exe")
    } else {
        install_dir.join("bin").join("black")
    };
    println!("Black path: {:?}", black_path);
    println!("Black path exists: {}", black_path.exists());

    // Assert that the Python tool is installed
    assert!(is_installed, "Python tool is not installed");

    // Assert that the black package is installed
    assert!(black_path.exists(), "black package is not installed");
}
