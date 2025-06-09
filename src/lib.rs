//! RustyHook: A Rust-native, language-agnostic, monorepo-friendly Git hook runner
//!
//! This library provides the core functionality for the RustyHook CLI tool.

pub mod config;
pub mod toolchains;
pub mod runner;
pub mod cache;
pub mod hooks;
pub mod logging;

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell as ClapShell};
use std::io;
use std::path::PathBuf;
use log::{debug, info, warn, error};

/// Supported shells for completion script generation
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Shell {
    /// Bash shell
    Bash,
    /// Zsh shell
    Zsh,
    /// Fish shell
    Fish,
    /// PowerShell
    PowerShell,
}

#[derive(Parser)]
#[command(
    name = "rustyhook",
    about = "A Rust-native, language-agnostic, monorepo-friendly Git hook runner",
    version,
    author,
    bin_name = "rustyhook"
)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Maximum number of hooks to run in parallel (0 means unlimited)
    #[arg(short, long, default_value_t = 0)]
    pub parallelism: usize,

    /// Path to the log file (if not specified, logs will only go to stdout)
    #[arg(long)]
    pub log_file: Option<PathBuf>,

    /// Log level (debug, info, warn, error)
    #[arg(long, default_value = "info")]
    pub log_level: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run hooks using native config if present
    Run,

    /// Run hooks using .pre-commit-config.yaml
    Compat,

    /// Convert pre-commit config to .rustyhook/config.yaml
    Convert {
        /// Convert from pre-commit config
        #[arg(long)]
        from_precommit: bool,

        /// Delete the original pre-commit config file after conversion
        #[arg(long)]
        delete_original: bool,

        /// Path to the pre-commit config file
        #[arg(long)]
        config_path: Option<PathBuf>,
    },

    /// Create a starter .rustyhook/config.yaml
    Init,

    /// List all available hooks and their status
    List,

    /// Diagnose issues with setup or environments
    Doctor,

    /// Remove cached environments and tool installs
    Clean,

    /// Generate shell completion scripts
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Install rustyhook as a Git hook
    Install {
        /// Type of Git hook to install (pre-commit, pre-push, etc.)
        #[arg(long, default_value = "pre-commit")]
        hook_type: String,

        /// Force overwrite of existing hooks
        #[arg(long)]
        force: bool,
    },

    /// Run a specific hook directly
    Hook {
        /// ID of the hook to run
        hook_id: String,

        /// Arguments to pass to the hook
        #[arg(long, short)]
        args: Vec<String>,

        /// Files to process
        #[arg(last = true)]
        files: Vec<PathBuf>,
    },
}

/// Main entry point for the RustyHook CLI
pub fn main() {
    let cli = Cli::parse();

    // Initialize the logger
    let log_file = cli.log_file.clone().or_else(|| {
        // If no log file is specified but we want to log to a file,
        // use the default log file path
        if std::env::var("RUSTYHOOK_LOG_TO_FILE").unwrap_or_default() == "true" {
            Some(logging::default_log_file())
        } else {
            None
        }
    });

    if let Err(e) = logging::init(log_file, Some(&cli.log_level)) {
        eprintln!("Failed to initialize logger: {}", e);
        return;
    }

    // Log the startup information
    if let Some(log_path) = &cli.log_file {
        info!("Logging to file: {}", log_path.display());
    }
    debug!("Log level set to: {}", cli.log_level);

    match cli.command {
        Commands::Run => {
            info!("Running hooks using native config...");
            run_hooks_with_native_config();
        }
        Commands::Compat => {
            info!("Running hooks using .pre-commit-config.yaml...");
            run_hooks_with_compat_config();
        }
        Commands::Convert { from_precommit, delete_original, config_path } => {
            if from_precommit {
                info!("Converting from .pre-commit-config.yaml to .rustyhook/config.yaml...");
                if delete_original {
                    info!("The original pre-commit config file will be deleted after conversion.");
                }
                if let Some(path) = &config_path {
                    info!("Using pre-commit config file at: {}", path.display());
                    match config::convert_from_precommit(Some(path), None, delete_original) {
                        Ok(_) => info!("Conversion successful!"),
                        Err(e) => error!("Error converting configuration: {:?}", e),
                    }
                } else {
                    match config::convert_from_precommit::<&str>(None, None, delete_original) {
                        Ok(_) => info!("Conversion successful!"),
                        Err(e) => error!("Error converting configuration: {:?}", e),
                    }
                }
            } else {
                warn!("Please specify --from-precommit to convert from pre-commit config");
            }
        }
        Commands::Init => {
            info!("Creating starter .rustyhook/config.yaml...");
            match config::create_starter_config::<&str>(None) {
                Ok(_) => info!("Starter configuration created successfully!"),
                Err(e) => error!("Error creating starter configuration: {:?}", e),
            }
        }
        Commands::List => {
            info!("Listing all available hooks and their status...");
            list_hooks();
        }
        Commands::Doctor => {
            info!("Diagnosing issues with setup or environments...");
            diagnose_issues();
        }
        Commands::Clean => {
            info!("Removing cached environments and tool installs...");
            clean_environments();
        }
        Commands::Completions { shell } => {
            info!("Generating completion script for {:?}...", shell);
            generate_completion_script(shell);
        }
        Commands::Install { hook_type, force } => {
            info!("Installing rustyhook as a {} Git hook...", hook_type);
            install_git_hook(&hook_type, force);
        }
        Commands::Hook { hook_id, args, files } => {
            info!("Running hook {}...", hook_id);
            run_hook(&hook_id, &args, &files);
        }
    }
}

/// Run hooks using native config
fn run_hooks_with_native_config() {
    // Find the native config
    match config::find_config() {
        Ok(mut config) => {
            // Get the parallelism limit from the CLI
            let cli = Cli::parse();
            if cli.parallelism > 0 {
                // Override the parallelism limit from the config with the one from the CLI
                config.parallelism = cli.parallelism;
                debug!("Overriding parallelism limit to: {}", cli.parallelism);
            }

            // Create a cache directory
            let cache_dir = std::env::temp_dir().join(".rustyhook");
            std::fs::create_dir_all(&cache_dir).unwrap_or_else(|e| {
                error!("Error creating cache directory: {}", e);
                std::process::exit(1);
            });
            debug!("Using cache directory: {}", cache_dir.display());

            // Create a hook resolver
            let mut resolver = runner::HookResolver::new(config, cache_dir);
            debug!("Hook resolver created");

            // Get the list of files to check
            // For now, we'll just use all files in the current directory
            let files = get_files_to_check();
            debug!("Found {} files to check", files.len());

            // Run all hooks
            match resolver.run_all_hooks(&files) {
                Ok(_) => info!("All hooks passed!"),
                Err(e) => {
                    error!("Error running hooks: {:?}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            error!("Error finding configuration: {:?}", e);
            std::process::exit(1);
        }
    }
}

/// Run hooks using .pre-commit-config.yaml
fn run_hooks_with_compat_config() {
    // Find the pre-commit config
    match config::find_precommit_config() {
        Ok(precommit_config) => {
            debug!("Found pre-commit configuration");

            // Convert to native config
            let mut config = config::convert_to_rustyhook_config(&precommit_config);
            debug!("Converted pre-commit configuration to rustyhook configuration");

            // Get the parallelism limit from the CLI
            let cli = Cli::parse();
            if cli.parallelism > 0 {
                // Override the parallelism limit from the config with the one from the CLI
                config.parallelism = cli.parallelism;
                debug!("Overriding parallelism limit to: {}", cli.parallelism);
            }

            // Create a cache directory
            let cache_dir = std::env::temp_dir().join(".rustyhook");
            std::fs::create_dir_all(&cache_dir).unwrap_or_else(|e| {
                error!("Error creating cache directory: {}", e);
                std::process::exit(1);
            });
            debug!("Using cache directory: {}", cache_dir.display());

            // Create a hook resolver
            let mut resolver = runner::HookResolver::new(config, cache_dir);
            debug!("Hook resolver created");

            // Get the list of files to check
            // For now, we'll just use all files in the current directory
            let files = get_files_to_check();
            debug!("Found {} files to check", files.len());

            // Run all hooks
            match resolver.run_all_hooks(&files) {
                Ok(_) => info!("All hooks passed!"),
                Err(e) => {
                    error!("Error running hooks: {:?}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            error!("Error finding pre-commit configuration: {:?}", e);
            std::process::exit(1);
        }
    }
}

/// List all available hooks and their status
fn list_hooks() {
    // Find the native config
    match config::find_config() {
        Ok(config) => {
            info!("Available hooks:");
            for repo in &config.repos {
                info!("Repository: {}", repo.repo);
                for hook in &repo.hooks {
                    info!("  - {}: {}", hook.id, hook.name);
                    info!("    Language: {}", hook.language);
                    info!("    Files: {}", hook.files);
                    info!("    Stages: {}", hook.stages.join(", "));
                }
            }
            debug!("Found {} repositories with a total of {} hooks", 
                  config.repos.len(), 
                  config.repos.iter().map(|r| r.hooks.len()).sum::<usize>());
        }
        Err(e) => {
            error!("Error finding configuration: {:?}", e);
            std::process::exit(1);
        }
    }
}

/// Diagnose issues with setup or environments
fn diagnose_issues() {
    debug!("Starting diagnosis of setup and environments");

    // Check if the .rustyhook directory exists
    let rustyhook_dir = std::env::current_dir().unwrap().join(".rustyhook");
    if !rustyhook_dir.exists() {
        info!("The .rustyhook directory does not exist. Run 'rustyhook init' to create it.");
    } else {
        info!("The .rustyhook directory exists.");
    }

    // Check if the config file exists
    let config_file = rustyhook_dir.join("config.yaml");
    if !config_file.exists() {
        info!("The .rustyhook/config.yaml file does not exist. Run 'rustyhook init' to create it.");
    } else {
        info!("The .rustyhook/config.yaml file exists.");
    }

    // Check if the cache directory exists
    let cache_dir = rustyhook_dir.join("cache");
    if !cache_dir.exists() {
        info!("The .rustyhook/cache directory does not exist. It will be created when needed.");
    } else {
        info!("The .rustyhook/cache directory exists.");
    }

    // Check if the venvs directory exists
    let venvs_dir = rustyhook_dir.join("venvs");
    if !venvs_dir.exists() {
        info!("The .rustyhook/venvs directory does not exist. It will be created when needed.");
    } else {
        info!("The .rustyhook/venvs directory exists.");
    }

    // Check if Python is installed
    match which::which("python3") {
        Ok(path) => {
            info!("Python 3 is installed at: {}", path.display());
            debug!("Python 3 found at path: {}", path.display());
        },
        Err(_) => {
            warn!("Python 3 is not installed. Some hooks may not work.");
            debug!("Failed to find Python 3 in PATH");
        },
    }

    // Check if Node.js is installed
    match which::which("node") {
        Ok(path) => {
            info!("Node.js is installed at: {}", path.display());
            debug!("Node.js found at path: {}", path.display());
        },
        Err(_) => {
            warn!("Node.js is not installed. Some hooks may not work.");
            debug!("Failed to find Node.js in PATH");
        },
    }

    // Check if Ruby is installed
    match which::which("ruby") {
        Ok(path) => {
            info!("Ruby is installed at: {}", path.display());
            debug!("Ruby found at path: {}", path.display());
        },
        Err(_) => {
            warn!("Ruby is not installed. Some hooks may not work.");
            debug!("Failed to find Ruby in PATH");
        },
    }

    debug!("Diagnosis completed");
}

/// Remove cached environments and tool installs
fn clean_environments() {
    debug!("Starting cleanup of cached environments and tool installs");

    // Remove the .rustyhook/cache directory
    let cache_dir = std::env::current_dir().unwrap().join(".rustyhook").join("cache");
    if cache_dir.exists() {
        debug!("Found cache directory at: {}", cache_dir.display());
        match std::fs::remove_dir_all(&cache_dir) {
            Ok(_) => {
                info!("Removed .rustyhook/cache directory.");
                debug!("Successfully removed directory: {}", cache_dir.display());
            },
            Err(e) => {
                error!("Error removing .rustyhook/cache directory: {}", e);
                debug!("Failed to remove directory: {}, error: {}", cache_dir.display(), e);
            },
        }
    } else {
        info!("The .rustyhook/cache directory does not exist.");
        debug!("Cache directory not found at: {}", cache_dir.display());
    }

    // Remove the .rustyhook/venvs directory
    let venvs_dir = std::env::current_dir().unwrap().join(".rustyhook").join("venvs");
    if venvs_dir.exists() {
        debug!("Found venvs directory at: {}", venvs_dir.display());
        match std::fs::remove_dir_all(&venvs_dir) {
            Ok(_) => {
                info!("Removed .rustyhook/venvs directory.");
                debug!("Successfully removed directory: {}", venvs_dir.display());
            },
            Err(e) => {
                error!("Error removing .rustyhook/venvs directory: {}", e);
                debug!("Failed to remove directory: {}, error: {}", venvs_dir.display(), e);
            },
        }
    } else {
        info!("The .rustyhook/venvs directory does not exist.");
        debug!("Venvs directory not found at: {}", venvs_dir.display());
    }

    debug!("Cleanup completed");
}

/// Get the list of files to check
fn get_files_to_check() -> Vec<std::path::PathBuf> {
    // For now, we'll just use all files in the current directory
    let mut files = Vec::new();
    let current_dir = std::env::current_dir().unwrap();

    // Walk the directory tree
    for entry in walkdir::WalkDir::new(&current_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        files.push(entry.path().to_path_buf());
    }

    files
}

/// Generate shell completion script for the specified shell
fn generate_completion_script(shell: Shell) {
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();

    match shell {
        Shell::Bash => {
            generate(ClapShell::Bash, &mut cmd, bin_name, &mut io::stdout());
        }
        Shell::Zsh => {
            generate(ClapShell::Zsh, &mut cmd, bin_name, &mut io::stdout());
        }
        Shell::Fish => {
            generate(ClapShell::Fish, &mut cmd, bin_name, &mut io::stdout());
        }
        Shell::PowerShell => {
            generate(ClapShell::PowerShell, &mut cmd, bin_name, &mut io::stdout());
        }
    }
}

/// Install rustyhook as a Git hook
fn install_git_hook(hook_type: &str, force: bool) {
    debug!("Installing rustyhook as a {} Git hook", hook_type);

    // Find the .git directory
    let git_dir = find_git_directory();
    if git_dir.is_none() {
        error!("Could not find .git directory. Are you in a Git repository?");
        std::process::exit(1);
    }
    let git_dir = git_dir.unwrap();
    debug!("Found .git directory at: {}", git_dir.display());

    // Create the hooks directory if it doesn't exist
    let hooks_dir = git_dir.join("hooks");
    if !hooks_dir.exists() {
        debug!("Creating hooks directory at: {}", hooks_dir.display());
        match std::fs::create_dir_all(&hooks_dir) {
            Ok(_) => debug!("Created hooks directory"),
            Err(e) => {
                error!("Error creating hooks directory: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Check if the hook already exists
    let hook_path = hooks_dir.join(hook_type);
    if hook_path.exists() && !force {
        error!("Hook {} already exists. Use --force to overwrite.", hook_type);
        std::process::exit(1);
    }

    // Get the path to the rustyhook executable
    let rustyhook_path = std::env::current_exe().unwrap_or_else(|e| {
        error!("Error getting path to rustyhook executable: {}", e);
        std::process::exit(1);
    });
    debug!("Using rustyhook executable at: {}", rustyhook_path.display());

    // Create the hook script
    let hook_script = format!(
        "#!/bin/sh\n\
         # RustyHook Git hook\n\
         # Generated by rustyhook\n\
         \n\
         {} run\n",
        rustyhook_path.display()
    );

    // Write the hook script
    debug!("Writing hook script to: {}", hook_path.display());
    match std::fs::write(&hook_path, hook_script) {
        Ok(_) => debug!("Wrote hook script"),
        Err(e) => {
            error!("Error writing hook script: {}", e);
            std::process::exit(1);
        }
    }

    // Make the hook executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&hook_path).unwrap().permissions();
        perms.set_mode(0o755);
        match std::fs::set_permissions(&hook_path, perms) {
            Ok(_) => debug!("Made hook script executable"),
            Err(e) => {
                error!("Error making hook script executable: {}", e);
                std::process::exit(1);
            }
        }
    }

    info!("Successfully installed rustyhook as a {} Git hook", hook_type);
}

/// Find the .git directory
fn find_git_directory() -> Option<std::path::PathBuf> {
    let mut current_dir = std::env::current_dir().ok()?;
    loop {
        let git_dir = current_dir.join(".git");
        if git_dir.exists() && git_dir.is_dir() {
            return Some(git_dir);
        }
        if !current_dir.pop() {
            return None;
        }
    }
}

/// Run a specific hook directly
fn run_hook(hook_id: &str, args: &[String], files: &[PathBuf]) {
    // Create the hook
    let hook = match hooks::HookFactory::create_hook(hook_id, args) {
        Ok(hook) => hook,
        Err(err) => {
            error!("Error creating hook: {:?}", err);
            std::process::exit(1);
        }
    };

    // If no files were specified or found, exit successfully
    if files.is_empty() {
        info!("No files to process for hook {}", hook_id);
        return;
    }

    // Run the hook
    match hook.run(files) {
        Ok(()) => {
            info!("Hook {} ran successfully", hook_id);
        }
        Err(err) => {
            error!("Error running hook: {:?}", err);
            std::process::exit(1);
        }
    }
}
