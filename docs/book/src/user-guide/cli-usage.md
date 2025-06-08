# CLI Usage

RustyHook provides a powerful command-line interface (CLI) for managing and running Git hooks. This guide covers all available commands and their options.

## Command Overview

RustyHook can be invoked using either `rustyhook` or the shorter alias `rh`. Both commands provide identical functionality.

```sh
# These are equivalent
rustyhook [command] [options]
rh [command] [options]
```

## Available Commands

### `run`

Run hooks based on your configuration.

```sh
rh run [options]
```

Options:
- `--hook <HOOK_ID>`: Run only the specified hook
- `--all-files`: Run on all files, not just changed ones
- `--files <FILES>`: Run on specific files (comma-separated)
- `--verbose`: Show detailed output
- `--no-cache`: Skip using cached environments

Examples:
```sh
# Run all hooks on changed files
rh run

# Run only the 'ruff' hook
rh run --hook ruff

# Run all hooks on all files
rh run --all-files

# Run hooks on specific files
rh run --files src/main.rs,src/lib.rs
```

### `compat`

Run hooks using a `.pre-commit-config.yaml` file for compatibility with pre-commit.

```sh
rh compat [options]
```

Options:
- Same as `run` command

Example:
```sh
# Run pre-commit compatible hooks
rh compat
```

### `convert`

Convert a pre-commit configuration to RustyHook's native format.

```sh
rh convert [options]
```

Options:
- `--from-precommit`: Convert from `.pre-commit-config.yaml`
- `--output <FILE>`: Output file (default: stdout)

Example:
```sh
# Convert pre-commit config to RustyHook config
rh convert --from-precommit > .rustyhook/config.yaml
```

### `init`

Initialize a new RustyHook configuration.

```sh
rh init [options]
```

Options:
- `--force`: Overwrite existing configuration
- `--template <TEMPLATE>`: Use a specific template (basic, full, minimal)

Example:
```sh
# Create a new configuration
rh init

# Create a new configuration with the full template
rh init --template full
```

### `list`

List all configured hooks.

```sh
rh list [options]
```

Options:
- `--verbose`: Show detailed information

Example:
```sh
# List all hooks
rh list
```

### `doctor`

Diagnose and fix issues with your RustyHook setup.

```sh
rh doctor [options]
```

Options:
- `--fix`: Attempt to fix issues automatically

Example:
```sh
# Check for issues
rh doctor

# Check and fix issues
rh doctor --fix
```

### `clean`

Clean cached environments and tools.

```sh
rh clean [options]
```

Options:
- `--all`: Remove all cached data
- `--language <LANGUAGE>`: Clean only specific language environments

Example:
```sh
# Clean all caches
rh clean --all

# Clean only Python environments
rh clean --language python
```

### `completions`

Generate shell completion scripts.

```sh
rh completions <SHELL>
```

Arguments:
- `SHELL`: The shell to generate completions for (bash, zsh, fish, powershell)

Example:
```sh
# Generate Bash completions
rh completions bash > ~/.bash_completion.d/rustyhook
```

### `install`

Install RustyHook as a Git hook.

```sh
rh install [options]
```

Options:
- `--hook-type <TYPE>`: Hook type to install (pre-commit, pre-push, etc.)
- `--force`: Overwrite existing hooks

Example:
```sh
# Install as pre-commit hook
rh install --hook-type pre-commit
```

### `uninstall`

Remove RustyHook Git hooks.

```sh
rh uninstall [options]
```

Options:
- `--hook-type <TYPE>`: Hook type to uninstall (pre-commit, pre-push, etc.)
- `--all`: Uninstall all hook types

Example:
```sh
# Uninstall all hooks
rh uninstall --all
```

## Global Options

These options can be used with any command:

- `--help`: Show help information
- `--version`: Show version information
- `--config <FILE>`: Use a specific configuration file
- `--no-color`: Disable colored output
- `--quiet`: Suppress all output except errors

## Environment Variables

RustyHook respects the following environment variables:

- `RUSTYHOOK_CONFIG`: Path to configuration file
- `RUSTYHOOK_CACHE_DIR`: Directory for cached environments
- `RUSTYHOOK_LOG_LEVEL`: Log level (debug, info, warn, error)
- `RUSTYHOOK_NO_COLOR`: Disable colored output if set to any value

## Exit Codes

- `0`: Success
- `1`: Hook failure
- `2`: Configuration error
- `3`: System error
- `4`: User error

## Next Steps

- Learn about [Configuration](configuration.md) options
- Set up [Shell Completions](shell-completions.md)
- Explore [Supported Languages](../reference/supported-languages.md)