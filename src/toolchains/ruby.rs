//! Ruby toolchain support for RustyHook
//!
//! This module provides functionality for managing Ruby environments and gems.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use which::which;

use super::r#trait::{SetupContext, Tool, ToolError};

/// Represents a Ruby tool
pub struct RubyTool {
    /// Name of the tool
    name: String,

    /// Version of the tool
    version: String,

    /// Ruby gems to install
    gems: Vec<String>,

    /// Installation directory
    install_dir: PathBuf,
}

impl RubyTool {
    /// Create a new Ruby tool
    pub fn new<S: Into<String>>(name: S, version: S, gems: Vec<String>) -> Self {
        let name_str = name.into();
        let version_str = version.into();

        // Default installation directory
        let mut install_dir = std::env::temp_dir();
        install_dir.push(".rustyhook");
        install_dir.push("venvs");
        install_dir.push(format!("ruby-{}-{}", name_str, version_str));

        RubyTool {
            name: name_str,
            version: version_str,
            gems,
            install_dir,
        }
    }

    /// Find the Bundler executable
    fn find_bundler() -> Result<PathBuf, ToolError> {
        // Try to find Bundler
        which("bundle").map_err(|_| ToolError::ToolNotFound("Bundler not found".to_string()))
    }

    /// Generate a Gemfile
    fn generate_gemfile(&self, ctx: &SetupContext) -> Result<(), ToolError> {
        // Create a basic Gemfile
        let mut gemfile_content = String::from("source 'https://rubygems.org'\n\n");

        // Add each gem to the Gemfile
        for gem in &self.gems {
            gemfile_content.push_str(&format!("gem '{}'\n", gem));
        }

        // Write to file
        let gemfile_path = ctx.install_dir.join("Gemfile");
        fs::write(gemfile_path, gemfile_content)?;

        Ok(())
    }

    /// Install gems using Bundler
    fn install_gems(&self, ctx: &SetupContext) -> Result<(), ToolError> {
        // Find Bundler
        let bundler = Self::find_bundler()?;

        // Create the .bundle directory
        let bundle_dir = ctx.install_dir.join(".bundle");
        fs::create_dir_all(&bundle_dir)?;

        // Create a bundle config file to install gems locally
        let config_content = "---\nBUNDLE_PATH: vendor/bundle\nBUNDLE_BIN: bin\n";
        fs::write(bundle_dir.join("config"), config_content)?;

        // Run bundle install
        let status = Command::new(bundler)
            .arg("install")
            .current_dir(&ctx.install_dir)
            .status()
            .map_err(|e| ToolError::ExecutionError(format!("Failed to install gems: {}", e)))?;

        if !status.success() {
            return Err(ToolError::ExecutionError(
                "Failed to install gems".to_string(),
            ));
        }

        Ok(())
    }
}

impl Tool for RubyTool {
    fn setup(&self, ctx: &SetupContext) -> Result<(), ToolError> {
        // Check if the tool is already installed and we're not forcing reinstallation
        if self.is_installed() && !ctx.force {
            return Ok(());
        }

        // Create the installation directory if it doesn't exist
        std::fs::create_dir_all(&ctx.install_dir)?;

        // Generate Gemfile
        self.generate_gemfile(ctx)?;

        // Install gems
        self.install_gems(ctx)?;

        Ok(())
    }

    fn run(&self, files: &[PathBuf]) -> Result<(), ToolError> {
        // Find the tool executable in the bin directory
        let tool_path = self.install_dir.join("bin").join(&self.name);

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
        // Check if the tool executable exists in the bin directory
        let tool_path = self.install_dir.join("bin").join(&self.name);
        tool_path.exists()
    }

    fn install_dir(&self) -> &PathBuf {
        &self.install_dir
    }
}
