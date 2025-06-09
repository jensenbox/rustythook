//! Runner module for RustyHook
//!
//! This module provides functionality for running hooks.

pub mod file_matcher;
pub mod hook_resolver;
pub mod parallel;
pub mod hook_context;

pub use file_matcher::{FileMatcher, FileMatcherError};
pub use hook_resolver::{HookResolver, HookResolverError};
pub use parallel::{ParallelExecutor, ParallelExecutionError};
pub use hook_context::HookContext;
