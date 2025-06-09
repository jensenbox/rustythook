//! Tests for toolchain functionality

use rustyhook::toolchains::{PythonTool, Tool, SetupContext};

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
