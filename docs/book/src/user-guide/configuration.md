# Configuration

RustyHook uses a YAML-based configuration file to define hooks and their behavior. This guide explains how to configure RustyHook for your project.

## Configuration File Location

By default, RustyHook looks for configuration in the following locations (in order of precedence):

1. Path specified by `--config` command-line option
2. Path specified by `RUSTYHOOK_CONFIG` environment variable
3. `.rustyhook/config.yaml` in the current directory
4. `.rustyhook/config.yml` in the current directory
5. `.pre-commit-config.yaml` (compatibility mode)

## Basic Configuration Structure

A basic RustyHook configuration file looks like this:

```yaml
# .rustyhook/config.yaml
hooks:
  - id: ruff
    language: python
    version: "==0.4.0"
    entry: "ruff check"
    files: "\\.py$"

  - id: biome
    language: node
    version: "^1.6.2"
    entry: "biome lint"
    files: "\\.(ts|js|json)$"
```

## Hook Configuration Options

Each hook in the `hooks` array can have the following properties:

| Property | Required | Description |
|----------|----------|-------------|
| `id` | Yes | Unique identifier for the hook |
| `language` | Yes | Language runtime (python, node, ruby, system) |
| `entry` | Yes | Command to execute |
| `files` | No | Regex pattern for files to include |
| `exclude` | No | Regex pattern for files to exclude |
| `args` | No | Additional arguments to pass to the command |
| `version` | No | Version requirement for the tool |
| `pass_filenames` | No | Whether to pass filenames to the command (default: true) |
| `always_run` | No | Run even when no matching files are changed (default: false) |
| `verbose` | No | Show verbose output for this hook (default: false) |
| `stages` | No | Git stages to run on (pre-commit, pre-push, etc.) |
| `fail_fast` | No | Stop execution on first failure (default: false) |
| `env` | No | Environment variables to set |
| `working_dir` | No | Directory to run the hook in |

## Language-Specific Configuration

### Python

```yaml
hooks:
  - id: ruff
    language: python
    version: "==0.4.0"  # Pinned version
    entry: "ruff check"
    files: "\\.py$"
    args: ["--fix"]
```

Python hooks use virtualenv to create isolated environments. The `version` field specifies the package version to install with pip.

### Node.js

```yaml
hooks:
  - id: eslint
    language: node
    version: "^8.0.0"  # Semver compatible version
    entry: "eslint"
    files: "\\.(js|jsx|ts|tsx)$"
    args: ["--fix"]
```

Node hooks use npm to install packages. The `version` field specifies the package version to install with npm.

### Ruby

```yaml
hooks:
  - id: rubocop
    language: ruby
    version: "~> 1.0"  # Compatible version
    entry: "rubocop"
    files: "\\.(rb)$"
    args: ["--auto-correct"]
```

Ruby hooks use bundler to install gems. The `version` field specifies the gem version to install with bundler.

### System

```yaml
hooks:
  - id: shellcheck
    language: system
    entry: "shellcheck"
    files: "\\.(sh)$"
```

System hooks use commands available on the system PATH. No environment setup is performed.

## Advanced Configuration

### Global Settings

You can specify global settings that apply to all hooks:

```yaml
default_stages: [pre-commit, pre-push]
fail_fast: true
exclude: "^vendor/"

hooks:
  # Hook definitions...
```

### Environment Variables

You can set environment variables for hooks:

```yaml
hooks:
  - id: my-hook
    language: system
    entry: "./my-script.sh"
    env:
      DEBUG: "true"
      NODE_ENV: "development"
```

### Working Directory

You can specify a working directory for hooks:

```yaml
hooks:
  - id: frontend-lint
    language: node
    entry: "eslint"
    files: "\\.(js|jsx)$"
    working_dir: "./frontend"
```

### Multiple Configurations

For monorepos, you can have multiple configuration files in different directories. RustyHook will use the closest configuration file to the Git root.

## Configuration Templates

RustyHook provides templates to help you get started:

```sh
# Create a basic configuration
rh init

# Create a configuration with more examples
rh init --template full
```

## Migrating from pre-commit

If you're migrating from pre-commit, you can convert your existing configuration:

```sh
rh convert --from-precommit > .rustyhook/config.yaml
```

## Next Steps

- Learn about [CLI Usage](cli-usage.md)
- Explore [Supported Languages](../reference/supported-languages.md)
- Set up [Shell Completions](shell-completions.md)