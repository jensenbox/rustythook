//! RustyHook: A Rust-native, language-agnostic, monorepo-friendly Git hook runner
//!
//! This library provides the core functionality for the RustyHook CLI tool.

pub mod config;
pub mod toolchains;
pub mod runner;
pub mod cache;
pub mod hooks;

use clap::{Parser, Subcommand};

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
    },

    /// Create a starter .rustyhook/config.yaml
    Init,

    /// List all available hooks and their status
    List,

    /// Diagnose issues with setup or environments
    Doctor,

    /// Remove cached environments and tool installs
    Clean,
}

/// Main entry point for the RustyHook CLI
pub fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run => {
            println!("Running hooks using native config...");
            run_hooks_with_native_config();
        }
        Commands::Compat => {
            println!("Running hooks using .pre-commit-config.yaml...");
            run_hooks_with_compat_config();
        }
        Commands::Convert { from_precommit, delete_original } => {
            if from_precommit {
                println!("Converting from .pre-commit-config.yaml to .rustyhook/config.yaml...");
                if delete_original {
                    println!("The original pre-commit config file will be deleted after conversion.");
                }
                match config::convert_from_precommit::<&str>(None, None, delete_original) {
                    Ok(_) => println!("Conversion successful!"),
                    Err(e) => eprintln!("Error converting configuration: {:?}", e),
                }
            } else {
                println!("Please specify --from-precommit to convert from pre-commit config");
            }
        }
        Commands::Init => {
            println!("Creating starter .rustyhook/config.yaml...");
            match config::create_starter_config::<&str>(None) {
                Ok(_) => println!("Starter configuration created successfully!"),
                Err(e) => eprintln!("Error creating starter configuration: {:?}", e),
            }
        }
        Commands::List => {
            println!("Listing all available hooks and their status...");
            list_hooks();
        }
        Commands::Doctor => {
            println!("Diagnosing issues with setup or environments...");
            diagnose_issues();
        }
        Commands::Clean => {
            println!("Removing cached environments and tool installs...");
            clean_environments();
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
            }

            // Create a cache directory
            let cache_dir = std::env::temp_dir().join(".rustyhook");
            std::fs::create_dir_all(&cache_dir).unwrap_or_else(|e| {
                eprintln!("Error creating cache directory: {}", e);
                std::process::exit(1);
            });

            // Create a hook resolver
            let mut resolver = runner::HookResolver::new(config, cache_dir);

            // Get the list of files to check
            // For now, we'll just use all files in the current directory
            let files = get_files_to_check();

            // Run all hooks
            match resolver.run_all_hooks(&files) {
                Ok(_) => println!("All hooks passed!"),
                Err(e) => {
                    eprintln!("Error running hooks: {:?}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error finding configuration: {:?}", e);
            std::process::exit(1);
        }
    }
}

/// Run hooks using .pre-commit-config.yaml
fn run_hooks_with_compat_config() {
    // Find the pre-commit config
    match config::find_precommit_config() {
        Ok(precommit_config) => {
            // Convert to native config
            let mut config = config::convert_to_rustyhook_config(&precommit_config);

            // Get the parallelism limit from the CLI
            let cli = Cli::parse();
            if cli.parallelism > 0 {
                // Override the parallelism limit from the config with the one from the CLI
                config.parallelism = cli.parallelism;
            }

            // Create a cache directory
            let cache_dir = std::env::temp_dir().join(".rustyhook");
            std::fs::create_dir_all(&cache_dir).unwrap_or_else(|e| {
                eprintln!("Error creating cache directory: {}", e);
                std::process::exit(1);
            });

            // Create a hook resolver
            let mut resolver = runner::HookResolver::new(config, cache_dir);

            // Get the list of files to check
            // For now, we'll just use all files in the current directory
            let files = get_files_to_check();

            // Run all hooks
            match resolver.run_all_hooks(&files) {
                Ok(_) => println!("All hooks passed!"),
                Err(e) => {
                    eprintln!("Error running hooks: {:?}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error finding pre-commit configuration: {:?}", e);
            std::process::exit(1);
        }
    }
}

/// List all available hooks and their status
fn list_hooks() {
    // Find the native config
    match config::find_config() {
        Ok(config) => {
            println!("Available hooks:");
            for repo in &config.repos {
                println!("Repository: {}", repo.repo);
                for hook in &repo.hooks {
                    println!("  - {}: {}", hook.id, hook.name);
                    println!("    Language: {}", hook.language);
                    println!("    Files: {}", hook.files);
                    println!("    Stages: {}", hook.stages.join(", "));
                }
            }
        }
        Err(e) => {
            eprintln!("Error finding configuration: {:?}", e);
            std::process::exit(1);
        }
    }
}

/// Diagnose issues with setup or environments
fn diagnose_issues() {
    // Check if the .rustyhook directory exists
    let rustyhook_dir = std::env::current_dir().unwrap().join(".rustyhook");
    if !rustyhook_dir.exists() {
        println!("The .rustyhook directory does not exist. Run 'rustyhook init' to create it.");
    } else {
        println!("The .rustyhook directory exists.");
    }

    // Check if the config file exists
    let config_file = rustyhook_dir.join("config.yaml");
    if !config_file.exists() {
        println!("The .rustyhook/config.yaml file does not exist. Run 'rustyhook init' to create it.");
    } else {
        println!("The .rustyhook/config.yaml file exists.");
    }

    // Check if the cache directory exists
    let cache_dir = rustyhook_dir.join("cache");
    if !cache_dir.exists() {
        println!("The .rustyhook/cache directory does not exist. It will be created when needed.");
    } else {
        println!("The .rustyhook/cache directory exists.");
    }

    // Check if the venvs directory exists
    let venvs_dir = rustyhook_dir.join("venvs");
    if !venvs_dir.exists() {
        println!("The .rustyhook/venvs directory does not exist. It will be created when needed.");
    } else {
        println!("The .rustyhook/venvs directory exists.");
    }

    // Check if Python is installed
    match which::which("python3") {
        Ok(_) => println!("Python 3 is installed."),
        Err(_) => println!("Python 3 is not installed. Some hooks may not work."),
    }

    // Check if Node.js is installed
    match which::which("node") {
        Ok(_) => println!("Node.js is installed."),
        Err(_) => println!("Node.js is not installed. Some hooks may not work."),
    }

    // Check if Ruby is installed
    match which::which("ruby") {
        Ok(_) => println!("Ruby is installed."),
        Err(_) => println!("Ruby is not installed. Some hooks may not work."),
    }
}

/// Remove cached environments and tool installs
fn clean_environments() {
    // Remove the .rustyhook/cache directory
    let cache_dir = std::env::current_dir().unwrap().join(".rustyhook").join("cache");
    if cache_dir.exists() {
        match std::fs::remove_dir_all(&cache_dir) {
            Ok(_) => println!("Removed .rustyhook/cache directory."),
            Err(e) => eprintln!("Error removing .rustyhook/cache directory: {}", e),
        }
    } else {
        println!("The .rustyhook/cache directory does not exist.");
    }

    // Remove the .rustyhook/venvs directory
    let venvs_dir = std::env::current_dir().unwrap().join(".rustyhook").join("venvs");
    if venvs_dir.exists() {
        match std::fs::remove_dir_all(&venvs_dir) {
            Ok(_) => println!("Removed .rustyhook/venvs directory."),
            Err(e) => eprintln!("Error removing .rustyhook/venvs directory: {}", e),
        }
    } else {
        println!("The .rustyhook/venvs directory does not exist.");
    }
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
