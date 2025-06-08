//! Configuration module for RustyHook
//!
//! This module provides functionality for parsing and handling RustyHook configurations.

pub mod parser;
pub mod compat;
pub mod converter;

pub use parser::{Config, ConfigError, Hook, Repo, find_config, parse_config};
pub use compat::{PreCommitConfig, PreCommitRepo, PreCommitHook, find_precommit_config, find_precommit_config_path, parse_precommit_config, convert_to_rustyhook_config};
pub use converter::{ConversionError, convert_from_precommit, create_starter_config};
