# RustyHook: Pre-commit Framework Specification

## Overview

RustyHook is a Rust-native, language-agnostic, monorepo-friendly Git hook runner designed to replace `pre-commit`. It supports automatic setup of linter/checker environments (Python, Node, etc.), drop-in compatibility with `.pre-commit-config.yaml`, and optional migration to a native RustyHook config format.

## Key Features

* Language-agnostic hook runner
* Monorepo support with per-project config
* Automatic environment setup:

    * Python (via virtualenv)
    * Node (via npm/yarn/pnpm)
    * Ruby (via rbenv + bundler)
* `.pre-commit-config.yaml` compatibility mode
* Conversion tool for migrating `pre-commit` config
* Config caching and tool version pinning
* Short alias `rh` available for CLI use

## File Layout

```
project-root/
├── .rustyhook/
│   ├── config.yaml          # Native RustyHook config (optional)
│   ├── cache/               # Tool installs and metadata
│   └── venvs/               # Python virtualenvs, Node installs, Ruby bundles
├── .pre-commit-config.yaml  # Optional compatibility mode
└── src/
    └── ...
```

## CLI Interface

RustyHook provides both `rustyhook` and `rh` as command-line entry points.

```sh
rustyhook run                          # Run using native config if present
rustyhook compat                      # Run using .pre-commit-config.yaml
rustyhook convert --from-precommit   # Convert pre-commit config to .rustyhook/config.yaml
rustyhook init                        # Create a starter .rustyhook/config.yaml
rustyhook list                        # List all available hooks and their status
rustyhook doctor                      # Diagnose issues with setup or environments
rustyhook clean                       # Remove cached environments and tool installs

# Short alias (rh) available for all above commands:
rh run
rh doctor
...
```

## Environment Setup Guidance

RustyHook will automatically install and manage environments in isolated directories per tool/version. The following languages are supported:

### Python

* Looks for system Python 3.7+
* Uses `virtualenv` under `.rustyhook/venvs/`
* Installs packages using `pip` with optional version constraints
* Falls back to `pyenv` if Python is not found

### Node.js

* Detects `node` from system or installs via `nvm`-style prebuilt binaries
* Uses `npm` or `pnpm` to install tools in `.rustyhook/venvs/node-<version>/`
* Automatically generates minimal `package.json` for tool install

### Ruby

* Uses `rbenv` or system Ruby for version management
* Installs tools using `bundler` into a local `.bundle` directory within `.rustyhook/venvs/ruby-<version>/`
* Supports creating a `Gemfile` dynamically for known hooks

Each tool environment is versioned and cached. Hashes of tool versions and config are used to determine when environments need to be reinstalled.

## Internal Architecture

### Modules

* `toolchains/`

    * `python.rs` – Create venv, install via pip
    * `node.rs` – Install tools via npm/pnpm
    * `ruby.rs` – Install bundler env and gems
    * `trait.rs` – `Tool` trait definition
* `config/`

    * `parser.rs` – Parse RustyHook and pre-commit YAML
* `runner/`

    * `hook_runner.rs` – Match files, execute hooks
* `utils/`

    * `file_matcher.rs`, `env.rs`, `logging.rs`

### Tool Trait

```rust
trait Tool {
    fn setup(&self, ctx: &SetupContext) -> Result<(), ToolError>;
    fn run(&self, files: &[PathBuf]) -> Result<()>;
}
```

### Hook Resolution

Hooks are defined either via `.rustyhook/config.yaml` or parsed from `.pre-commit-config.yaml`. A `HookResolver` maps known `repo + id` to native tool commands.

### Environment Setup

Each tool is installed into `.rustyhook/cache/{tool}-{version}`. Python uses `virtualenv`, Node uses `npm`/`pnpm` install to a local `node_modules`, Ruby uses `bundler` with a generated `Gemfile`.

### File Matching

Regex- or glob-based filtering of changed files. Only matching files are passed to tools.

### Parallel Execution

Hooks run in parallel across tools and projects using `rayon` or `tokio`, ensuring concurrency and speed.

## Compatibility Mode

RustyHook can parse and run from `.pre-commit-config.yaml`. Known repositories and hook IDs are mapped to known tool setups (e.g., `ruff-pre-commit` → `ruff==0.4.0`). Unsupported hooks are skipped with a warning.

## Migration Tool

A built-in converter reads `.pre-commit-config.yaml` and writes a RustyHook-native config, resolving tool versions and setup.

## Crate Recommendations

* `serde_yaml` for config parsing
* `regex` and `globset` for file filtering
* `clap` for CLI
* `which` for detecting runtimes
* `tokio` or `rayon` for parallel execution

## Summary

RustyHook is a drop-in, Rust-native modern replacement for `pre-commit` that emphasizes reproducibility, performance, and multi-language support in monorepos. It supports existing `.pre-commit-config.yaml` files for easy adoption and can convert them into its own format for long-term maintainability.
