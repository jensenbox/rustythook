//! Node.js toolchain support for RustyHook
//!
//! This module provides functionality for managing Node.js environments and packages.
//! It downloads precompiled Node.js binaries directly from nodejs.org.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use which::which;
use serde::{Serialize, Deserialize};
use log::{debug, info};
use std::env;

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

    /// Determine the platform triple for Node.js download
    fn get_platform_triple(&self) -> Result<String, ToolError> {
        let os = env::consts::OS;
        let arch = env::consts::ARCH;

        let platform = match (os, arch) {
            ("linux", "x86_64") => "linux-x64",
            ("linux", "aarch64") => "linux-arm64",
            ("macos", "x86_64") => "darwin-x64",
            ("macos", "aarch64") => "darwin-arm64",
            ("windows", "x86_64") => "win-x64",
            ("windows", "aarch64") => "win-arm64",
            _ => return Err(ToolError::ExecutionError(format!("Unsupported platform: {}-{}", os, arch))),
        };

        Ok(platform.to_string())
    }

    /// Read Node.js version from .node-version or .nvmrc file
    fn read_node_version_file(dir: &Path) -> Option<String> {
        // Start from the given directory and look for .node-version or .nvmrc file
        let mut current_dir = Some(dir.to_path_buf());

        while let Some(dir) = current_dir {
            // Check for .node-version file
            let node_version_file = dir.join(".node-version");
            if node_version_file.exists() {
                match fs::read_to_string(&node_version_file) {
                    Ok(content) => {
                        let version = content.trim().to_string();
                        if !version.is_empty() {
                            log::info!("Found Node.js version {} in {:?}", version, node_version_file);
                            return Some(version);
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to read .node-version file: {}", e);
                    }
                }
            }

            // Check for .nvmrc file
            let nvmrc_file = dir.join(".nvmrc");
            if nvmrc_file.exists() {
                match fs::read_to_string(&nvmrc_file) {
                    Ok(content) => {
                        let version = content.trim().to_string();
                        if !version.is_empty() {
                            log::info!("Found Node.js version {} in {:?}", version, nvmrc_file);
                            return Some(version);
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to read .nvmrc file: {}", e);
                    }
                }
            }

            // Move up to the parent directory
            current_dir = dir.parent().map(|p| p.to_path_buf());
        }

        None
    }

    /// Determine the Node.js version to use
    fn determine_node_version(&self, specified_version: Option<&str>) -> Result<String, ToolError> {
        // If version is specified, use it
        if let Some(version) = specified_version {
            if version == "lts" {
                // For LTS, we'll use a hardcoded recent LTS version
                // In a real implementation, this would fetch the latest LTS version from nodejs.org
                return Ok("20.11.1".to_string());
            }
            return Ok(version.to_string());
        }

        // Try to find .node-version or .nvmrc in the current directory or parent directories
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        if let Some(version) = Self::read_node_version_file(&current_dir) {
            return Ok(version);
        }

        // Default to a recent LTS version
        Ok("20.11.1".to_string())
    }

    /// Get the Node.js binary path for the installed version
    fn get_node_binary_path(&self, version: &str) -> PathBuf {
        let runtime_dir = PathBuf::from(".runtime");
        let platform = self.get_platform_triple().unwrap_or_else(|_| "unknown".to_string());
        let node_dir = runtime_dir.join("node").join(version);

        let bin_dir = if env::consts::OS == "windows" {
            node_dir.join(format!("node-v{}-{}", version, platform))
        } else {
            node_dir.join(format!("node-v{}-{}", version, platform)).join("bin")
        };

        if env::consts::OS == "windows" {
            bin_dir.join("node.exe")
        } else {
            bin_dir.join("node")
        }
    }

    /// Check if Node.js is installed
    fn is_node_installed(&self, version: &str) -> bool {
        let node_binary = self.get_node_binary_path(version);
        node_binary.exists()
    }

    /// Download and extract Node.js
    fn download_and_extract_node(&self, version: &str) -> Result<PathBuf, ToolError> {
        let platform = self.get_platform_triple()?;
        let runtime_dir = PathBuf::from(".runtime");
        let node_dir = runtime_dir.join("node").join(version);

        // Create directories
        fs::create_dir_all(&node_dir)?;

        // Determine file extension based on platform
        let file_ext = if env::consts::OS == "windows" { "zip" } else { "tar.xz" };

        // Construct download URL
        let download_url = format!(
            "https://nodejs.org/dist/v{}/node-v{}-{}.{}",
            version, version, platform, file_ext
        );

        info!("Downloading Node.js {} for {} from {}", version, platform, download_url);

        // Download the archive
        let archive_path = node_dir.join(format!("node-v{}-{}.{}", version, platform, file_ext));

        let curl_output = Command::new("curl")
            .arg("-fsSL")
            .arg("--output")
            .arg(&archive_path)
            .arg(&download_url)
            .output()
            .map_err(|e| ToolError::ExecutionError(format!("Failed to download Node.js: {}", e)))?;

        if !curl_output.status.success() {
            let stderr = String::from_utf8_lossy(&curl_output.stderr);
            return Err(ToolError::ExecutionError(format!("Failed to download Node.js: {}", stderr)));
        }

        // Extract the archive
        info!("Extracting Node.js {} to {}", version, node_dir.display());

        if file_ext == "zip" {
            // For Windows, use PowerShell to extract zip
            let extract_output = Command::new("powershell")
                .arg("-Command")
                .arg(format!("Expand-Archive -Path \"{}\" -DestinationPath \"{}\" -Force",
                    archive_path.display(), node_dir.display()))
                .output()
                .map_err(|e| ToolError::ExecutionError(format!("Failed to extract Node.js: {}", e)))?;

            if !extract_output.status.success() {
                let stderr = String::from_utf8_lossy(&extract_output.stderr);
                return Err(ToolError::ExecutionError(format!("Failed to extract Node.js: {}", stderr)));
            }
        } else {
            // For Unix, use tar
            let extract_output = Command::new("tar")
                .arg("-xf")
                .arg(&archive_path)
                .arg("-C")
                .arg(&node_dir)
                .output()
                .map_err(|e| ToolError::ExecutionError(format!("Failed to extract Node.js: {}", e)))?;

            if !extract_output.status.success() {
                let stderr = String::from_utf8_lossy(&extract_output.stderr);
                return Err(ToolError::ExecutionError(format!("Failed to extract Node.js: {}", stderr)));
            }
        }

        // Verify installation
        let node_binary = self.get_node_binary_path(version);

        if !node_binary.exists() {
            return Err(ToolError::ExecutionError(format!(
                "Node.js binary not found at expected path: {}",
                node_binary.display()
            )));
        }

        // Make the binary executable on Unix systems
        if env::consts::OS != "windows" {
            let chmod_output = Command::new("chmod")
                .arg("+x")
                .arg(&node_binary)
                .output()
                .map_err(|e| ToolError::ExecutionError(format!("Failed to make Node.js binary executable: {}", e)))?;

            if !chmod_output.status.success() {
                let stderr = String::from_utf8_lossy(&chmod_output.stderr);
                return Err(ToolError::ExecutionError(format!("Failed to make Node.js binary executable: {}", stderr)));
            }
        }

        // Verify by running node --version
        let version_output = Command::new(&node_binary)
            .arg("--version")
            .output()
            .map_err(|e| ToolError::ExecutionError(format!("Failed to run Node.js: {}", e)))?;

        if !version_output.status.success() {
            let stderr = String::from_utf8_lossy(&version_output.stderr);
            return Err(ToolError::ExecutionError(format!("Failed to run Node.js: {}", stderr)));
        }

        let installed_version = String::from_utf8_lossy(&version_output.stdout).trim().to_string();
        info!("Node.js {} installed successfully. Version: {}", version, installed_version);

        Ok(node_binary)
    }

    /// Ensure Node.js is installed
    fn ensure_node_installed(&self, node_version: &str) -> Result<PathBuf, ToolError> {
        debug!("Ensuring Node.js {} is installed...", node_version);

        // Determine the actual version to use
        let version = self.determine_node_version(Some(node_version))?;

        // Check if Node.js is already installed
        if self.is_node_installed(&version) {
            debug!("Node.js {} is already installed", version);
            return Ok(self.get_node_binary_path(&version));
        }

        // Download and install Node.js
        info!("Node.js {} not found, downloading...", version);
        self.download_and_extract_node(&version)
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

        // Ensure Node.js is installed
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
