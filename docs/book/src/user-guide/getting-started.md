# Getting Started with RustyHook

This guide will help you get up and running with RustyHook quickly. We'll cover installation, basic configuration, and running your first hooks.

## Prerequisites

Before you begin, make sure you have:

- Rust and Cargo installed (if installing via Cargo)
- Git installed and initialized in your project
- Basic familiarity with Git hooks

## Installation

You can install RustyHook using one of the following methods:

### Using Cargo

```sh
cargo install rustyhook
```

### Manual Installation

Clone the repository and build from source:

```sh
git clone https://github.com/your-org/rustyhook.git
cd rustyhook
cargo build --release
```

The binary will be available at `./target/release/rustyhook`.

### Verifying Installation

To verify that RustyHook is installed correctly, run:

```sh
rustyhook --version
# or using the alias
rh --version
```

## Creating Your First Configuration

RustyHook uses a YAML configuration file to define your hooks. You can create a new configuration file using the `init` command:

```sh
rh init
```

This will create a `.rustyhook/config.yaml` file in your project root with some example hooks.

## Basic Configuration Example

Here's a simple configuration example:

```yaml
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

## Running Hooks

To run all configured hooks:

```sh
rh run
```

To run specific hooks:

```sh
rh run --hook ruff
```

## Setting Up Git Hooks

To set up RustyHook as a Git pre-commit hook:

```sh
rh install
```

This will create a Git hook that runs RustyHook before each commit.

## Next Steps

Now that you have RustyHook set up, you might want to:

- Learn more about [CLI Usage](cli-usage.md)
- Explore [Configuration](configuration.md) options
- Set up [Shell Completions](shell-completions.md)
- If you're migrating from pre-commit, check out the [Migration Guide](migration.md)