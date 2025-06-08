//! Python toolchain support for RustyHook
//!
//! This module provides functionality for managing Python environments and packages.

use std::path::PathBuf;
use std::process::Command;
use which::which;

use super::r#trait::{SetupContext, Tool, ToolError};

/// Represents a Python tool
pub struct PythonTool {
    /// Name of the tool
    name: String,

    /// Version of the tool
    version: String,

    /// Python packages to install
    packages: Vec<String>,

    /// Installation directory
    install_dir: PathBuf,
}

impl PythonTool {
    /// Create a new Python tool
    pub fn new<S: Into<String>>(name: S, version: S, packages: Vec<String>) -> Self {
        let name_str = name.into();
        let version_str = version.into();

        // Default installation directory
        let mut install_dir = std::env::temp_dir();
        install_dir.push(".rustyhook");
        install_dir.push("venvs");
        install_dir.push(format!("python-{}-{}", name_str, version_str));

        PythonTool {
            name: name_str,
            version: version_str,
            packages,
            install_dir,
        }
    }

    /// Find the Python executable
    fn find_python() -> Result<PathBuf, ToolError> {
        // Try to find Python 3.7+
        for version in &["python3", "python3.7", "python3.8", "python3.9", "python3.10", "python3.11", "python"] {
            if let Ok(path) = which(version) {
                return Ok(path);
            }
        }

        // If Python is not found, return an error
        Err(ToolError::ToolNotFound("Python 3.7+ not found".to_string()))
    }

    /// Create a virtualenv
    fn create_virtualenv(&self, ctx: &SetupContext) -> Result<(), ToolError> {
        // Find Python
        let python = Self::find_python()?;

        // Create the installation directory if it doesn't exist
        std::fs::create_dir_all(&ctx.install_dir)?;

        // Create the virtualenv
        let status = Command::new(python)
            .arg("-m")
            .arg("venv")
            .arg(&ctx.install_dir)
            .status()
            .map_err(|e| ToolError::ExecutionError(format!("Failed to create virtualenv: {}", e)))?;

        if !status.success() {
            return Err(ToolError::ExecutionError(
                "Failed to create virtualenv".to_string(),
            ));
        }

        Ok(())
    }

    /// Install packages in the virtualenv
    fn install_packages(&self, ctx: &SetupContext) -> Result<(), ToolError> {
        // Find the pip executable in the virtualenv
        let pip = if cfg!(windows) {
            ctx.install_dir.join("Scripts").join("pip.exe")
        } else {
            ctx.install_dir.join("bin").join("pip")
        };

        // Install each package
        for package in &self.packages {
            let status = Command::new(&pip)
                .arg("install")
                .arg(package)
                .status()
                .map_err(|e| ToolError::ExecutionError(format!("Failed to install {}: {}", package, e)))?;

            if !status.success() {
                return Err(ToolError::ExecutionError(
                    format!("Failed to install {}", package),
                ));
            }
        }

        Ok(())
    }
}

impl Tool for PythonTool {
    fn setup(&self, ctx: &SetupContext) -> Result<(), ToolError> {
        // Check if the tool is already installed and we're not forcing reinstallation
        if self.is_installed() && !ctx.force {
            return Ok(());
        }

        // Create the virtualenv
        self.create_virtualenv(ctx)?;

        // Install packages
        self.install_packages(ctx)?;

        Ok(())
    }

    fn run(&self, files: &[PathBuf]) -> Result<(), ToolError> {
        // Find the tool executable in the virtualenv
        let tool_path = if cfg!(windows) {
            self.install_dir.join("Scripts").join(format!("{}.exe", self.name))
        } else {
            self.install_dir.join("bin").join(&self.name)
        };

        // Run the tool on the files
        let mut command = Command::new(&tool_path);

        // Add files as arguments
        for file in files {
            command.arg(file);
        }

        // Execute the command
        let status = command
            .status()
            .map_err(|e| ToolError::ExecutionError(format!("Failed to run {}: {}", self.name, e)))?;

        if !status.success() {
            return Err(ToolError::ExecutionError(
                format!("{} failed with exit code {:?}", self.name, status.code()),
            ));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn is_installed(&self) -> bool {
        // Check if the tool executable exists in the virtualenv
        let tool_path = if cfg!(windows) {
            self.install_dir.join("Scripts").join(format!("{}.exe", self.name))
        } else {
            self.install_dir.join("bin").join(&self.name)
        };

        tool_path.exists()
    }

    fn install_dir(&self) -> &PathBuf {
        &self.install_dir
    }
}
