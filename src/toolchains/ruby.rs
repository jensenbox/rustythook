//! Ruby toolchain support for RustyHook
//!
//! This module provides functionality for managing Ruby environments and gems.

use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::env;

use flate2::read::GzDecoder;
use reqwest::blocking::Client;
use tar::Archive;
use zip::ZipArchive;

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

    /// Read Ruby version from .ruby-version file
    fn read_ruby_version_file(dir: &Path) -> Option<String> {
        // Start from the given directory and look for .ruby-version file
        let mut current_dir = Some(dir.to_path_buf());

        while let Some(dir) = current_dir {
            let version_file = dir.join(".ruby-version");

            if version_file.exists() {
                // Read the file content
                match fs::read_to_string(&version_file) {
                    Ok(content) => {
                        // Trim whitespace and return the version
                        let version = content.trim().to_string();
                        if !version.is_empty() {
                            log::info!("Found Ruby version {} in {:?}", version, version_file);
                            return Some(version);
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to read .ruby-version file: {}", e);
                    }
                }
            }

            // Move up to the parent directory
            current_dir = dir.parent().map(|p| p.to_path_buf());
        }

        None
    }

    /// Determine the Ruby version to use
    fn determine_ruby_version(&self, specified_version: Option<&str>) -> Result<String, ToolError> {
        // If version is specified, use it
        if let Some(version) = specified_version {
            return Ok(version.to_string());
        }

        // Try to find .ruby-version in the current directory or parent directories
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        if let Some(version) = Self::read_ruby_version_file(&current_dir) {
            return Ok(version);
        }

        // Default to a recent stable version
        Ok("3.2.2".to_string())
    }

    /// Get the Ruby download URL based on the operating system and architecture
    fn get_ruby_download_url(version: &str) -> Result<String, ToolError> {
        // Determine the OS and architecture
        let os = env::consts::OS;
        let arch = env::consts::ARCH;

        let url = match (os, arch) {
            ("windows", "x86_64") => format!("https://github.com/oneclick/rubyinstaller2/releases/download/RubyInstaller-{}-1/rubyinstaller-{}-x64.zip", 
                version, version),
            ("linux", "x86_64") => format!("https://cache.ruby-lang.org/pub/ruby/{}/ruby-{}.tar.gz", 
                version.split('.').take(2).collect::<Vec<_>>().join("."), version),
            ("macos", "x86_64") | ("macos", "aarch64") => format!("https://cache.ruby-lang.org/pub/ruby/{}/ruby-{}.tar.gz", 
                version.split('.').take(2).collect::<Vec<_>>().join("."), version),
            _ => return Err(ToolError::ExecutionError(format!("Unsupported OS/architecture: {}/{}", os, arch))),
        };

        Ok(url)
    }

    /// Download Ruby from the official website
    fn download_ruby(download_dir: &PathBuf, version: &str) -> Result<PathBuf, ToolError> {
        // Create the download directory if it doesn't exist
        fs::create_dir_all(download_dir)?;

        // Get the download URL
        let url = Self::get_ruby_download_url(version)?;

        // Extract the filename from the URL
        let filename = url.split('/').last().unwrap_or("ruby.tgz");
        let download_path = download_dir.join(filename);

        // Skip download if the file already exists
        if download_path.exists() {
            log::info!("Ruby already downloaded at {:?}", download_path);
            return Ok(download_path);
        }

        // Download the file
        log::info!("Downloading Ruby from {}", url);
        let client = Client::new();
        let mut response = client.get(&url)
            .send()
            .map_err(|e| ToolError::ExecutionError(format!("Failed to download Ruby: {}", e)))?;

        // Check if the request was successful
        if !response.status().is_success() {
            return Err(ToolError::ExecutionError(format!("Failed to download Ruby: HTTP {}", response.status())));
        }

        // Create the file
        let mut file = fs::File::create(&download_path)
            .map_err(|e| ToolError::ExecutionError(format!("Failed to create file: {}", e)))?;

        // Copy the response body to the file
        let mut buffer = Vec::new();
        response.read_to_end(&mut buffer)
            .map_err(|e| ToolError::ExecutionError(format!("Failed to read response: {}", e)))?;
        file.write_all(&buffer)
            .map_err(|e| ToolError::ExecutionError(format!("Failed to write to file: {}", e)))?;

        log::info!("Downloaded Ruby to {:?}", download_path);
        Ok(download_path)
    }

    /// Extract the downloaded Ruby archive
    fn extract_ruby(archive_path: &PathBuf, extract_dir: &PathBuf) -> Result<PathBuf, ToolError> {
        // Create the extraction directory if it doesn't exist
        fs::create_dir_all(extract_dir)?;

        // Get the filename to determine the archive type
        let filename = archive_path.file_name().unwrap().to_string_lossy();

        if filename.ends_with(".tar.gz") || filename.ends_with(".tgz") {
            // Extract .tar.gz archive
            log::info!("Extracting Ruby from {:?} to {:?}", archive_path, extract_dir);
            let file = fs::File::open(archive_path)
                .map_err(|e| ToolError::ExecutionError(format!("Failed to open archive: {}", e)))?;
            let tar = GzDecoder::new(file);
            let mut archive = Archive::new(tar);
            archive.unpack(extract_dir)
                .map_err(|e| ToolError::ExecutionError(format!("Failed to extract archive: {}", e)))?;

            // Find the Ruby directory (usually ruby-x.y.z)
            let entries = fs::read_dir(extract_dir)
                .map_err(|e| ToolError::ExecutionError(format!("Failed to read directory: {}", e)))?;

            for entry in entries {
                let entry = entry.map_err(|e| ToolError::ExecutionError(format!("Failed to read directory entry: {}", e)))?;
                let path = entry.path();
                if path.is_dir() && path.file_name().unwrap().to_string_lossy().starts_with("ruby-") {
                    log::info!("Found Ruby directory at {:?}", path);
                    return Ok(path);
                }
            }

            Err(ToolError::ExecutionError("Failed to find Ruby directory after extraction".to_string()))
        } else if filename.ends_with(".zip") {
            // Extract .zip archive (for Windows)
            log::info!("Extracting Ruby from {:?} to {:?}", archive_path, extract_dir);
            let file = fs::File::open(archive_path)
                .map_err(|e| ToolError::ExecutionError(format!("Failed to open archive: {}", e)))?;
            let mut archive = ZipArchive::new(file)
                .map_err(|e| ToolError::ExecutionError(format!("Failed to read zip archive: {}", e)))?;

            for i in 0..archive.len() {
                let mut file = archive.by_index(i)
                    .map_err(|e| ToolError::ExecutionError(format!("Failed to read zip entry: {}", e)))?;
                let outpath = extract_dir.join(file.name());

                if file.name().ends_with('/') {
                    fs::create_dir_all(&outpath)
                        .map_err(|e| ToolError::ExecutionError(format!("Failed to create directory: {}", e)))?;
                } else {
                    if let Some(p) = outpath.parent() {
                        if !p.exists() {
                            fs::create_dir_all(p)
                                .map_err(|e| ToolError::ExecutionError(format!("Failed to create directory: {}", e)))?;
                        }
                    }
                    let mut outfile = fs::File::create(&outpath)
                        .map_err(|e| ToolError::ExecutionError(format!("Failed to create file: {}", e)))?;
                    io::copy(&mut file, &mut outfile)
                        .map_err(|e| ToolError::ExecutionError(format!("Failed to write file: {}", e)))?;
                }
            }

            // For Windows, the Ruby directory is the extract_dir itself
            Ok(extract_dir.clone())
        } else {
            Err(ToolError::ExecutionError(format!("Unsupported archive format: {}", filename)))
        }
    }

    /// Build Ruby from source (for Unix systems)
    fn build_ruby(ruby_dir: &PathBuf, install_dir: &PathBuf) -> Result<PathBuf, ToolError> {
        log::info!("Building Ruby from source at {:?}", ruby_dir);

        // Configure
        let status = Command::new("sh")
            .current_dir(ruby_dir)
            .arg("-c")
            .arg(format!("./configure --prefix={}", install_dir.display()))
            .status()
            .map_err(|e| ToolError::ExecutionError(format!("Failed to configure Ruby: {}", e)))?;

        if !status.success() {
            return Err(ToolError::ExecutionError("Failed to configure Ruby".to_string()));
        }

        // Make
        let status = Command::new("make")
            .current_dir(ruby_dir)
            .status()
            .map_err(|e| ToolError::ExecutionError(format!("Failed to build Ruby: {}", e)))?;

        if !status.success() {
            return Err(ToolError::ExecutionError("Failed to build Ruby".to_string()));
        }

        // Make install
        let status = Command::new("make")
            .current_dir(ruby_dir)
            .arg("install")
            .status()
            .map_err(|e| ToolError::ExecutionError(format!("Failed to install Ruby: {}", e)))?;

        if !status.success() {
            return Err(ToolError::ExecutionError("Failed to install Ruby".to_string()));
        }

        // Return the path to the Ruby executable
        let ruby_path = if cfg!(windows) {
            install_dir.join("bin").join("ruby.exe")
        } else {
            install_dir.join("bin").join("ruby")
        };

        if !ruby_path.exists() {
            return Err(ToolError::ExecutionError(format!("Ruby executable not found at {:?}", ruby_path)));
        }

        log::info!("Built Ruby at {:?}", ruby_path);
        Ok(ruby_path)
    }

    /// Install Ruby locally
    fn install_ruby(ctx: &SetupContext) -> Result<PathBuf, ToolError> {
        // Create directories
        let download_dir = ctx.cache_dir.join("downloads");
        let extract_dir = ctx.cache_dir.join("extracted");
        let install_dir = ctx.install_dir.join("ruby");

        // Determine Ruby version to use
        let ruby_tool = RubyTool::new("bundler", "2.4.10", vec![]);
        let version = ruby_tool.determine_ruby_version(ctx.version.as_deref())?;

        // Download Ruby
        let archive_path = Self::download_ruby(&download_dir, &version)?;

        // Extract Ruby
        let ruby_dir = Self::extract_ruby(&archive_path, &extract_dir)?;

        // For Windows, we can use the extracted Ruby directly
        if cfg!(windows) {
            // Find the Ruby executable in the extracted directory
            let ruby_exe = ruby_dir.join("bin").join("ruby.exe");
            if ruby_exe.exists() {
                return Ok(ruby_exe);
            }

            // Try alternative paths for Windows RubyInstaller
            let alt_ruby_exe = ruby_dir.join("ruby.exe");
            if alt_ruby_exe.exists() {
                return Ok(alt_ruby_exe);
            }

            return Err(ToolError::ExecutionError(format!("Ruby executable not found in extracted directory")));
        } else {
            // For Unix systems, build from source
            Self::build_ruby(&ruby_dir, &install_dir)
        }
    }

    /// Find the Bundler executable
    fn find_bundler(ruby_path: &PathBuf) -> Result<PathBuf, ToolError> {
        // Check if bundler is installed with the Ruby we just installed
        let bundler_path = if cfg!(windows) {
            ruby_path.parent().unwrap().join("bundle.bat")
        } else {
            ruby_path.parent().unwrap().join("bundle")
        };

        if bundler_path.exists() {
            return Ok(bundler_path);
        }

        // If not found, install bundler
        log::info!("Bundler not found, installing...");
        let status = Command::new(ruby_path)
            .arg("-S")
            .arg("gem")
            .arg("install")
            .arg("bundler")
            .status()
            .map_err(|e| ToolError::ExecutionError(format!("Failed to install bundler: {}", e)))?;

        if !status.success() {
            return Err(ToolError::ExecutionError("Failed to install bundler".to_string()));
        }

        // Check again for bundler
        if bundler_path.exists() {
            return Ok(bundler_path);
        }

        Err(ToolError::ToolNotFound("Bundler not found and could not be installed".to_string()))
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
        // Install Ruby locally
        let ruby_path = Self::install_ruby(ctx)?;

        // Find or install Bundler
        let bundler = Self::find_bundler(&ruby_path)?;

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
