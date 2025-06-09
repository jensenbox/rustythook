//! Node.js toolchain support for RustyHook
//!
//! This module provides functionality for managing Node.js environments and packages.
//! It uses fnm (Fast Node Manager) to install and manage Node.js versions.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use which::which;
use serde::{Serialize, Deserialize};
use log::{debug, info, warn, error};

use super::r#trait::{SetupContext, Tool, ToolError};

/// Represents a Node.js package.json file
#[derive(Debug, Serialize, Deserialize)]
struct PackageJson {
    name: String,
    version: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    dependencies: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "devDependencies")]
    dev_dependencies: Option<serde_json::Value>,
}

/// Represents a Node.js tool
pub struct NodeTool {
    /// Name of the tool
    name: String,

    /// Version of the tool
    version: String,

    /// Node.js packages to install
    packages: Vec<String>,

    /// Whether to install as dev dependencies
    dev_dependencies: bool,

    /// Package manager to use (npm, yarn, pnpm)
    package_manager: String,

    /// Installation directory
    install_dir: PathBuf,
}

impl NodeTool {
    /// Create a new Node.js tool
    pub fn new<S: Into<String>>(
        name: S,
        version: S,
        packages: Vec<String>,
        dev_dependencies: bool,
        package_manager: Option<S>,
    ) -> Self {
        let name_str = name.into();
        let version_str = version.into();
        let package_manager_str = package_manager
            .map(|s| s.into())
            .unwrap_or_else(|| "npm".to_string());

        // Default installation directory
        let mut install_dir = std::env::temp_dir();
        install_dir.push(".rustyhook");
        install_dir.push("venvs");
        install_dir.push(format!("node-{}-{}", name_str, version_str));

        NodeTool {
            name: name_str,
            version: version_str,
            packages,
            dev_dependencies,
            package_manager: package_manager_str,
            install_dir,
        }
    }

    /// Check if fnm is installed
    fn is_fnm_installed(&self) -> bool {
        which("fnm").is_ok()
    }

    /// Install fnm
    fn install_fnm(&self) -> Result<(), ToolError> {
        debug!("Installing fnm...");

        // Create a temporary directory for the installation
        let temp_dir = std::env::temp_dir().join("rustyhook_fnm_install");
        std::fs::create_dir_all(&temp_dir)?;

        // Download the fnm installation script
        let curl_output = Command::new("curl")
            .arg("-fsSL")
            .arg("https://fnm.vercel.app/install")
            .current_dir(&temp_dir)
            .output()
            .map_err(|e| ToolError::ExecutionError(format!("Failed to download fnm: {}", e)))?;

        if !curl_output.status.success() {
            let stderr = String::from_utf8_lossy(&curl_output.stderr);
            return Err(ToolError::ExecutionError(format!("Failed to download fnm: {}", stderr)));
        }

        // Save the script to a file
        let script_path = temp_dir.join("install.sh");
        std::fs::write(&script_path, curl_output.stdout)?;

        // Make the script executable
        Command::new("chmod")
            .arg("+x")
            .arg(&script_path)
            .status()
            .map_err(|e| ToolError::ExecutionError(format!("Failed to make fnm install script executable: {}", e)))?;

        // Run the installation script with --skip-shell to avoid modifying shell config
        let install_output = Command::new("bash")
            .arg(&script_path)
            .arg("--skip-shell")
            .current_dir(&temp_dir)
            .output()
            .map_err(|e| ToolError::ExecutionError(format!("Failed to install fnm: {}", e)))?;

        if !install_output.status.success() {
            let stderr = String::from_utf8_lossy(&install_output.stderr);
            return Err(ToolError::ExecutionError(format!("Failed to install fnm: {}", stderr)));
        }

        info!("fnm installed successfully");
        Ok(())
    }

    /// Ensure Node.js is installed using fnm
    fn ensure_node_installed(&self, node_version: &str) -> Result<(), ToolError> {
        debug!("Ensuring Node.js {} is installed...", node_version);

        // Check if fnm is installed, install it if not
        if !self.is_fnm_installed() {
            info!("fnm not found, installing...");
            self.install_fnm()?;
        }

        // Check if the specified Node.js version is installed
        let list_output = Command::new("fnm")
            .arg("list")
            .output()
            .map_err(|e| ToolError::ExecutionError(format!("Failed to list Node.js versions: {}", e)))?;

        let list_output_str = String::from_utf8_lossy(&list_output.stdout);

        // If the version is not installed, install it
        if !list_output_str.contains(node_version) {
            info!("Installing Node.js {}...", node_version);

            let install_output = Command::new("fnm")
                .arg("install")
                .arg(node_version)
                .output()
                .map_err(|e| ToolError::ExecutionError(format!("Failed to install Node.js {}: {}", node_version, e)))?;

            if !install_output.status.success() {
                let stderr = String::from_utf8_lossy(&install_output.stderr);
                return Err(ToolError::ExecutionError(format!("Failed to install Node.js {}: {}", node_version, stderr)));
            }

            info!("Node.js {} installed successfully", node_version);
        } else {
            debug!("Node.js {} is already installed", node_version);
        }

        // Use the specified Node.js version
        let use_output = Command::new("fnm")
            .arg("use")
            .arg(node_version)
            .output()
            .map_err(|e| ToolError::ExecutionError(format!("Failed to use Node.js {}: {}", node_version, e)))?;

        if !use_output.status.success() {
            let stderr = String::from_utf8_lossy(&use_output.stderr);
            return Err(ToolError::ExecutionError(format!("Failed to use Node.js {}: {}", node_version, stderr)));
        }

        debug!("Using Node.js {}", node_version);
        Ok(())
    }

    /// Find the package manager executable
    fn find_package_manager(&self) -> Result<PathBuf, ToolError> {
        which(&self.package_manager).map_err(|_| {
            ToolError::ToolNotFound(format!("{} not found", self.package_manager))
        })
    }

    /// Generate a package.json file
    fn generate_package_json(&self, ctx: &SetupContext) -> Result<(), ToolError> {
        // Create a basic package.json
        let package_json = PackageJson {
            name: format!("rustyhook-{}", self.name),
            version: "1.0.0".to_string(),
            description: format!("RustyHook tool: {}", self.name),
            dependencies: None,
            dev_dependencies: None,
        };

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&package_json)
            .map_err(|e| ToolError::ExecutionError(format!("Failed to generate package.json: {}", e)))?;

        // Write to file
        let package_json_path = ctx.install_dir.join("package.json");
        fs::write(package_json_path, json)?;

        Ok(())
    }

    /// Install packages using the package manager
    fn install_packages(&self, ctx: &SetupContext) -> Result<(), ToolError> {
        // Find the package manager
        let package_manager = self.find_package_manager()?;

        // Build the install command
        let mut command = Command::new(package_manager);
        command.current_dir(&ctx.install_dir);

        // Add the install command
        command.arg("install");

        // Add the --save-dev flag if needed
        if self.dev_dependencies {
            command.arg("--save-dev");
        }

        // Add the packages
        for package in &self.packages {
            command.arg(package);
        }

        // Execute the command
        let status = command
            .status()
            .map_err(|e| ToolError::ExecutionError(format!("Failed to install packages: {}", e)))?;

        if !status.success() {
            return Err(ToolError::ExecutionError(
                "Failed to install packages".to_string(),
            ));
        }

        Ok(())
    }
}

impl Tool for NodeTool {
    fn setup(&self, ctx: &SetupContext) -> Result<(), ToolError> {
        // Check if the tool is already installed and we're not forcing reinstallation
        if self.is_installed() && !ctx.force {
            return Ok(());
        }

        // Create the installation directory if it doesn't exist
        std::fs::create_dir_all(&ctx.install_dir)?;

        // Ensure Node.js is installed using fnm
        // Use LTS version if not specified
        let node_version = ctx.version.as_deref().unwrap_or("lts");
        self.ensure_node_installed(node_version)?;

        // Generate package.json
        self.generate_package_json(ctx)?;

        // Install packages
        self.install_packages(ctx)?;

        Ok(())
    }

    fn run(&self, files: &[PathBuf]) -> Result<(), ToolError> {
        // Find the tool executable in node_modules
        let tool_path = self.install_dir.join("node_modules").join(".bin").join(&self.name);

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
        // Check if the tool executable exists in node_modules/.bin
        let tool_path = self.install_dir.join("node_modules").join(".bin").join(&self.name);
        tool_path.exists()
    }

    fn install_dir(&self) -> &PathBuf {
        &self.install_dir
    }
}
