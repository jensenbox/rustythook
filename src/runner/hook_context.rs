//! Hook context for RustyHook
//!
//! This module provides the context for running hooks.

use std::collections::HashMap;
use std::path::PathBuf;
use crate::config::parser::HookType;

/// Represents the context for running a hook
#[derive(Debug, Clone)]
pub struct HookContext {
    /// Hook identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Command or script to run
    pub entry: String,

    /// Programming language or environment
    pub language: String,

    /// File pattern to match
    pub files: String,

    /// Stages to run this hook on
    pub stages: Vec<String>,

    /// Additional arguments to pass to the hook
    pub args: Vec<String>,

    /// Additional environment variables
    pub env: HashMap<String, String>,

    /// Version of the tool to use
    pub version: Option<String>,

    /// Whether this hook is built-in or external
    pub hook_type: HookType,

    /// Whether to run this hook in a separate process
    pub separate_process: bool,

    /// Working directory for the hook
    pub working_dir: PathBuf,

    /// Files to process
    pub files_to_process: Vec<PathBuf>,
}

impl HookContext {
    /// Create a new hook context
    pub fn new(
        id: String,
        name: String,
        entry: String,
        language: String,
        files: String,
        stages: Vec<String>,
        args: Vec<String>,
        env: HashMap<String, String>,
        version: Option<String>,
        hook_type: HookType,
        separate_process: bool,
        working_dir: PathBuf,
        files_to_process: Vec<PathBuf>,
    ) -> Self {
        HookContext {
            id,
            name,
            entry,
            language,
            files,
            stages,
            args,
            env,
            version,
            hook_type,
            separate_process,
            working_dir,
            files_to_process,
        }
    }

    /// Create a hook context from a hook configuration
    pub fn from_hook(
        hook: &crate::config::Hook,
        working_dir: PathBuf,
        files_to_process: Vec<PathBuf>,
    ) -> Self {
        HookContext {
            id: hook.id.clone(),
            name: hook.name.clone(),
            entry: hook.entry.clone(),
            language: hook.language.clone(),
            files: hook.files.clone(),
            stages: hook.stages.clone(),
            args: hook.args.clone(),
            env: hook.env.clone(),
            version: hook.version.clone(),
            hook_type: hook.hook_type.clone(),
            separate_process: hook.separate_process,
            working_dir,
            files_to_process,
        }
    }
}