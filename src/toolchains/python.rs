//! Python toolchain support for RustyHook
//!
//! This module provides functionality for managing Python environments and packages.

use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::Command;
use which::which;

use flate2::read::GzDecoder;
use reqwest::blocking::Client;
use tar::Archive;
use zip::ZipArchive;
use zstd::stream::Decoder as ZstdDecoder;

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

        // If Python is not found, we'll download it
        log::info!("Python not found on system, will download and install locally");
        Err(ToolError::ToolNotFound("Python 3.7+ not found".to_string()))
    }

    /// Get the Python download URL based on the operating system and architecture
    /// Uses python-build-standalone from Gregory Szorc's project
    fn get_python_download_url() -> Result<String, ToolError> {
        // Default to Python 3.9.18 as it's stable and widely compatible
        let version = "3.9.18";

        // python-build-standalone version
        let pbs_version = "20240224";

        // Determine the OS and architecture
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        let url = match (os, arch) {
            ("windows", "x86_64") => format!("https://github.com/indygreg/python-build-standalone/releases/download/{}/cpython-{}-{}-windows-amd64-shared-pgo.tar.zst", 
                pbs_version, version, pbs_version),
            ("windows", "aarch64") => format!("https://github.com/indygreg/python-build-standalone/releases/download/{}/cpython-{}-{}-windows-arm64-shared-pgo.tar.zst", 
                pbs_version, version, pbs_version),
            ("macos", "x86_64") => format!("https://github.com/indygreg/python-build-standalone/releases/download/{}/cpython-{}-{}-macos-x86_64-shared-install_only.tar.zst", 
                pbs_version, version, pbs_version),
            ("macos", "aarch64") => format!("https://github.com/indygreg/python-build-standalone/releases/download/{}/cpython-{}-{}-macos-arm64-shared-install_only.tar.zst", 
                pbs_version, version, pbs_version),
            ("linux", "x86_64") => format!("https://github.com/indygreg/python-build-standalone/releases/download/{}/cpython-{}-{}-linux-x86_64-shared-install_only.tar.zst", 
                pbs_version, version, pbs_version),
            ("linux", "aarch64") => format!("https://github.com/indygreg/python-build-standalone/releases/download/{}/cpython-{}-{}-linux-aarch64-shared-install_only.tar.zst", 
                pbs_version, version, pbs_version),
            _ => return Err(ToolError::ExecutionError(format!("Unsupported OS/architecture: {}/{}", os, arch))),
        };

        Ok(url)
    }

    /// Download Python from the official website
    fn download_python(download_dir: &PathBuf) -> Result<PathBuf, ToolError> {
        // Create the download directory if it doesn't exist
        fs::create_dir_all(download_dir)?;

        // Get the download URL
        let url = Self::get_python_download_url()?;

        // Extract the filename from the URL
        let filename = url.split('/').last().unwrap_or("python.tgz");
        let download_path = download_dir.join(filename);

        // Skip download if the file already exists
        if download_path.exists() {
            log::info!("Python already downloaded at {:?}", download_path);
            return Ok(download_path);
        }

        // Download the file
        log::info!("Downloading Python from {}", url);
        let client = Client::new();
        let mut response = client.get(&url)
            .send()
            .map_err(|e| ToolError::ExecutionError(format!("Failed to download Python: {}", e)))?;

        // Check if the request was successful
        if !response.status().is_success() {
            return Err(ToolError::ExecutionError(format!("Failed to download Python: HTTP {}", response.status())));
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

        log::info!("Downloaded Python to {:?}", download_path);
        Ok(download_path)
    }

    /// Extract the downloaded Python archive
    fn extract_python(archive_path: &PathBuf, extract_dir: &PathBuf) -> Result<PathBuf, ToolError> {
        // Create the extraction directory if it doesn't exist
        fs::create_dir_all(extract_dir)?;

        // Get the filename to determine the archive type
        let filename = archive_path.file_name().unwrap().to_string_lossy();

        if filename.ends_with(".tgz") || filename.ends_with(".tar.gz") {
            // Extract .tgz or .tar.gz archive
            log::info!("Extracting Python from {:?} to {:?}", archive_path, extract_dir);
            let file = fs::File::open(archive_path)
                .map_err(|e| ToolError::ExecutionError(format!("Failed to open archive: {}", e)))?;
            let tar = GzDecoder::new(file);
            let mut archive = Archive::new(tar);
            archive.unpack(extract_dir)
                .map_err(|e| ToolError::ExecutionError(format!("Failed to extract archive: {}", e)))?;

            // Find the Python directory (usually Python-x.y.z)
            let entries = fs::read_dir(extract_dir)
                .map_err(|e| ToolError::ExecutionError(format!("Failed to read directory: {}", e)))?;

            for entry in entries {
                let entry = entry.map_err(|e| ToolError::ExecutionError(format!("Failed to read directory entry: {}", e)))?;
                let path = entry.path();
                if path.is_dir() && path.file_name().unwrap().to_string_lossy().starts_with("Python-") {
                    log::info!("Found Python directory at {:?}", path);
                    return Ok(path);
                }
            }

            Err(ToolError::ExecutionError("Failed to find Python directory after extraction".to_string()))
        } else if filename.ends_with(".tar.zst") {
            // Extract .tar.zst archive (used by python-build-standalone)
            log::info!("Extracting Python from {:?} to {:?}", archive_path, extract_dir);
            let file = fs::File::open(archive_path)
                .map_err(|e| ToolError::ExecutionError(format!("Failed to open archive: {}", e)))?;
            let zstd = ZstdDecoder::new(file)
                .map_err(|e| ToolError::ExecutionError(format!("Failed to create zstd decoder: {}", e)))?;
            let mut archive = Archive::new(zstd);
            archive.unpack(extract_dir)
                .map_err(|e| ToolError::ExecutionError(format!("Failed to extract archive: {}", e)))?;

            // python-build-standalone has a different structure
            // The Python executable is in the 'python/bin' directory
            let python_dir = extract_dir.join("python");
            if python_dir.exists() {
                log::info!("Found Python directory at {:?}", python_dir);
                return Ok(python_dir);
            }

            // If not found directly, look for it in subdirectories
            let entries = fs::read_dir(extract_dir)
                .map_err(|e| ToolError::ExecutionError(format!("Failed to read directory: {}", e)))?;

            for entry in entries {
                let entry = entry.map_err(|e| ToolError::ExecutionError(format!("Failed to read directory entry: {}", e)))?;
                let path = entry.path();
                if path.is_dir() {
                    let python_subdir = path.join("python");
                    if python_subdir.exists() && python_subdir.is_dir() {
                        log::info!("Found Python directory at {:?}", python_subdir);
                        return Ok(python_subdir);
                    }
                }
            }

            Err(ToolError::ExecutionError("Failed to find Python directory after extraction".to_string()))
        } else if filename.ends_with(".zip") {
            // Extract .zip archive
            log::info!("Extracting Python from {:?} to {:?}", archive_path, extract_dir);
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

            Ok(extract_dir.clone())
        } else {
            // For Windows .exe and macOS .pkg installers, we can't extract them directly
            // We would need to run the installer, which is more complex
            Err(ToolError::ExecutionError(format!("Unsupported archive format: {}", filename)))
        }
    }

    /// Build Python from source (for Linux)
    fn build_python(python_dir: &PathBuf, install_dir: &PathBuf) -> Result<PathBuf, ToolError> {
        log::info!("Building Python from source at {:?}", python_dir);

        // Configure
        let status = Command::new("sh")
            .current_dir(python_dir)
            .arg("-c")
            .arg(format!("./configure --prefix={}", install_dir.display()))
            .status()
            .map_err(|e| ToolError::ExecutionError(format!("Failed to configure Python: {}", e)))?;

        if !status.success() {
            return Err(ToolError::ExecutionError("Failed to configure Python".to_string()));
        }

        // Make
        let status = Command::new("make")
            .current_dir(python_dir)
            .status()
            .map_err(|e| ToolError::ExecutionError(format!("Failed to build Python: {}", e)))?;

        if !status.success() {
            return Err(ToolError::ExecutionError("Failed to build Python".to_string()));
        }

        // Make install
        let status = Command::new("make")
            .current_dir(python_dir)
            .arg("install")
            .status()
            .map_err(|e| ToolError::ExecutionError(format!("Failed to install Python: {}", e)))?;

        if !status.success() {
            return Err(ToolError::ExecutionError("Failed to install Python".to_string()));
        }

        // Return the path to the Python executable
        let python_path = if cfg!(windows) {
            install_dir.join("bin").join("python.exe")
        } else {
            install_dir.join("bin").join("python3")
        };

        if !python_path.exists() {
            return Err(ToolError::ExecutionError(format!("Python executable not found at {:?}", python_path)));
        }

        log::info!("Built Python at {:?}", python_path);
        Ok(python_path)
    }

    /// Install Python locally
    fn install_python(ctx: &SetupContext) -> Result<PathBuf, ToolError> {
        // Create directories
        let download_dir = ctx.cache_dir.join("downloads");
        let extract_dir = ctx.cache_dir.join("extracted");
        let install_dir = ctx.install_dir.join("python");

        // Download Python
        let archive_path = Self::download_python(&download_dir)?;

        // Extract Python
        let python_dir = Self::extract_python(&archive_path, &extract_dir)?;

        // Get the filename to determine if we're using python-build-standalone
        let filename = archive_path.file_name().unwrap().to_string_lossy();

        let python_path = if filename.ends_with(".tar.zst") {
            // For python-build-standalone, we don't need to build from source
            // The Python executable is already in the bin directory
            let bin_dir = python_dir.join("bin");
            let python_exe = if cfg!(windows) {
                bin_dir.join("python.exe")
            } else {
                bin_dir.join("python3")
            };

            if !python_exe.exists() {
                return Err(ToolError::ExecutionError(
                    format!("Python executable not found at {:?}", python_exe)
                ));
            }

            // Copy the extracted Python to the install directory
            if install_dir.exists() {
                fs::remove_dir_all(&install_dir)
                    .map_err(|e| ToolError::ExecutionError(format!("Failed to remove existing install directory: {}", e)))?;
            }

            fs::create_dir_all(&install_dir)
                .map_err(|e| ToolError::ExecutionError(format!("Failed to create install directory: {}", e)))?;

            // Use a platform-specific copy method
            if cfg!(windows) {
                // On Windows, use xcopy
                let status = Command::new("xcopy")
                    .arg("/E")
                    .arg("/I")
                    .arg("/Y")
                    .arg(python_dir.to_str().unwrap())
                    .arg(install_dir.to_str().unwrap())
                    .status()
                    .map_err(|e| ToolError::ExecutionError(format!("Failed to copy Python: {}", e)))?;

                if !status.success() {
                    return Err(ToolError::ExecutionError("Failed to copy Python".to_string()));
                }
            } else {
                // On Unix-like systems, use cp
                let status = Command::new("cp")
                    .arg("-R")
                    .arg(python_dir.to_str().unwrap())
                    .arg(install_dir.to_str().unwrap())
                    .status()
                    .map_err(|e| ToolError::ExecutionError(format!("Failed to copy Python: {}", e)))?;

                if !status.success() {
                    return Err(ToolError::ExecutionError("Failed to copy Python".to_string()));
                }
            }

            // Return the path to the Python executable in the install directory
            if cfg!(windows) {
                install_dir.join("bin").join("python.exe")
            } else {
                install_dir.join("bin").join("python3")
            }
        } else {
            // For traditional Python source, build from source
            Self::build_python(&python_dir, &install_dir)?
        };

        Ok(python_path)
    }

    /// Create a virtualenv
    fn create_virtualenv(&self, ctx: &SetupContext) -> Result<(), ToolError> {
        // Try to find Python on the system first
        let python = match Self::find_python() {
            Ok(path) => path,
            Err(_) => {
                // If Python is not found, download and install it
                Self::install_python(ctx)?
            }
        };

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

    /// Install packages in the virtualenv using pip (traditional approach)
    fn install_packages(&self, ctx: &SetupContext) -> Result<(), ToolError> {
        // Find the python executable in the virtualenv
        let python = if cfg!(windows) {
            ctx.install_dir.join("Scripts").join("python.exe")
        } else {
            ctx.install_dir.join("bin").join("python")
        };

        // Find the pip executable in the virtualenv
        let pip = if cfg!(windows) {
            ctx.install_dir.join("Scripts").join("pip.exe")
        } else {
            ctx.install_dir.join("bin").join("pip")
        };

        // Check if the python executable exists in the virtualenv
        if !python.exists() {
            return Err(ToolError::ExecutionError(
                format!("Python executable not found at {:?}", python),
            ));
        }

        log::debug!("Python executable found at {:?}", python);

        // Install all packages at once for better performance
        if !self.packages.is_empty() {
            // First, try to install uv directly using pip
            let status = Command::new(&pip)
                .arg("install")
                .arg("uv")
                .status()
                .map_err(|e| ToolError::ExecutionError(format!("Failed to install uv: {}", e)))?;

            if !status.success() {
                log::warn!("Failed to install uv, falling back to regular pip for package installation");

                // If uv installation fails, fall back to regular pip for package installation
                let mut cmd = Command::new(&pip);
                cmd.arg("install");

                // Add all packages as arguments
                for package in &self.packages {
                    cmd.arg(package);
                }

                log::debug!("Running pip command: {:?}", cmd);

                let output = cmd.output()
                    .map_err(|e| ToolError::ExecutionError(format!("Failed to install packages with pip: {}", e)))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    log::error!("pip stderr: {}", stderr);
                    log::error!("pip stdout: {}", stdout);
                    return Err(ToolError::ExecutionError(
                        format!("Failed to install packages with pip: {}", stderr),
                    ));
                }

                log::debug!("Successfully installed packages with pip");
            } else {
                // If uv installation succeeds, use it to install packages
                let uv = if cfg!(windows) {
                    ctx.install_dir.join("Scripts").join("uv.exe")
                } else {
                    ctx.install_dir.join("bin").join("uv")
                };

                // Check if the uv executable exists
                if !uv.exists() {
                    log::warn!("uv executable not found at {:?}, falling back to regular pip", uv);

                    // If uv is not found, fall back to regular pip
                    let mut cmd = Command::new(&pip);
                    cmd.arg("install");

                    // Add all packages as arguments
                    for package in &self.packages {
                        cmd.arg(package);
                    }

                    log::debug!("Running pip command: {:?}", cmd);

                    let output = cmd.output()
                        .map_err(|e| ToolError::ExecutionError(format!("Failed to install packages with pip: {}", e)))?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        log::error!("pip stderr: {}", stderr);
                        log::error!("pip stdout: {}", stdout);
                        return Err(ToolError::ExecutionError(
                            format!("Failed to install packages with pip: {}", stderr),
                        ));
                    }

                    log::debug!("Successfully installed packages with pip");
                } else {
                    // Use uv to install packages
                    let mut cmd = Command::new(&uv);
                    cmd.arg("pip")
                        .arg("install");

                    // Add all packages as arguments
                    for package in &self.packages {
                        cmd.arg(package);
                    }

                    log::debug!("Running uv command: {:?}", cmd);

                    let output = cmd.output()
                        .map_err(|e| ToolError::ExecutionError(format!("Failed to install packages with uv: {}", e)))?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        log::error!("uv stderr: {}", stderr);
                        log::error!("uv stdout: {}", stdout);

                        log::warn!("Failed to install packages with uv, falling back to regular pip");

                        // If uv fails, fall back to regular pip
                        let mut cmd = Command::new(&pip);
                        cmd.arg("install");

                        // Add all packages as arguments
                        for package in &self.packages {
                            cmd.arg(package);
                        }

                        log::debug!("Running pip command: {:?}", cmd);

                        let output = cmd.output()
                            .map_err(|e| ToolError::ExecutionError(format!("Failed to install packages with pip: {}", e)))?;

                        if !output.status.success() {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            log::error!("pip stderr: {}", stderr);
                            log::error!("pip stdout: {}", stdout);
                            return Err(ToolError::ExecutionError(
                                format!("Failed to install packages with pip: {}", stderr),
                            ));
                        }

                        log::debug!("Successfully installed packages with pip");
                    } else {
                        log::debug!("Successfully installed packages with uv");
                    }
                }
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

        // Execute the command with output capture
        let output = command
            .output()
            .map_err(|e| ToolError::ExecutionError(format!("Failed to run {}: {}", self.name, e)))?;

        // Check the status
        if output.status.success() {
            Ok(())
        } else {
            // Try to convert stdout and stderr to strings, but handle non-UTF-8 data
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            // Log the command and its output
            log::error!("Command failed: {} {}", tool_path.display(), files.iter().map(|f| f.display().to_string()).collect::<Vec<_>>().join(" "));
            if !stdout.is_empty() {
                log::error!("Command stdout: {}", stdout);
            }
            if !stderr.is_empty() {
                log::error!("Command stderr: {}", stderr);
            }

            Err(ToolError::ExecutionError(
                format!("{} failed with exit code {:?}", self.name, output.status.code()),
            ))
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn is_installed(&self) -> bool {
        // For Python tools, we need to check if the Python executable and the tool executable exist
        let python_path = if cfg!(windows) {
            self.install_dir.join("Scripts").join("python.exe")
        } else {
            self.install_dir.join("bin").join("python")
        };

        // Check if the tool executable exists in the virtualenv
        let tool_path = if cfg!(windows) {
            self.install_dir.join("Scripts").join(format!("{}.exe", self.name))
        } else {
            self.install_dir.join("bin").join(&self.name)
        };

        // Log the paths for debugging
        log::debug!("Checking if Python tool is installed:");
        log::debug!("  Python path: {:?}, exists: {}", python_path, python_path.exists());
        log::debug!("  Tool path: {:?}, exists: {}", tool_path, tool_path.exists());

        // For Python tools, we consider them installed if both the Python executable
        // and the tool executable exist
        python_path.exists() && tool_path.exists()
    }

    fn install_dir(&self) -> &PathBuf {
        &self.install_dir
    }
}
