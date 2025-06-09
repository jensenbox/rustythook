//! rustyhook-hooks binary
//!
//! This binary provides a command-line interface to the native hook implementations.

use std::env;
use std::path::PathBuf;
use std::process;
use std::io::{self, Write};

use rustyhook::hooks::{HookFactory, HookError};

fn main() {
    // Initialize logger
    env_logger::init();

    // Get the command-line arguments
    let args: Vec<String> = env::args().collect();

    // Check if we have at least one argument (the hook ID)
    if args.len() < 2 {
        eprintln!("Usage: rustyhook-hooks <hook-id> [args...] [files...]");
        process::exit(1);
    }

    // The first argument is the hook ID
    let hook_id = &args[1];

    // The rest of the arguments are either hook arguments or files
    let mut hook_args = Vec::new();
    let mut files = Vec::new();

    for arg in &args[2..] {
        if arg.starts_with("--") {
            hook_args.push(arg.clone());
        } else {
            // Check if the file exists
            let path = PathBuf::from(arg);
            if path.exists() {
                files.push(path);
            } else {
                eprintln!("Warning: File not found: {}", arg);
            }
        }
    }

    // If no files were specified or found, exit successfully
    if files.is_empty() {
        println!("No files to process for hook {}", hook_id);
        process::exit(0);
    }

    // Create the hook
    let hook = match HookFactory::create_hook(hook_id, &hook_args) {
        Ok(hook) => hook,
        Err(err) => {
            eprintln!("Error creating hook: {:?}", err);
            process::exit(1);
        }
    };

    // Run the hook
    match hook.run(&files) {
        Ok(()) => {
            // Ensure stdout is flushed before exiting
            io::stdout().flush().unwrap_or_default();
            println!("Hook {} ran successfully", hook_id);
            process::exit(0);
        }
        Err(err) => {
            // Ensure stderr is flushed before exiting
            io::stderr().flush().unwrap_or_default();
            eprintln!("Error running hook: {:?}", err);
            process::exit(1);
        }
    }
}
