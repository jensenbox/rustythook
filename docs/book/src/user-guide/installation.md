# Installation

This guide provides detailed instructions for installing RustyHook on different platforms.

## Prerequisites

- **Rust and Cargo**: Required if installing via Cargo
- **Git**: Required for version control integration

## Installation Methods

### Using Cargo (Recommended)

The simplest way to install RustyHook is using Cargo, Rust's package manager:

```sh
cargo install rustyhook
```

This will download, compile, and install the latest version of RustyHook.

### Using Package Managers

#### Homebrew (macOS and Linux)

```sh
brew install rustyhook
```

#### Chocolatey (Windows)

```sh
choco install rustyhook
```

#### Scoop (Windows)

```sh
scoop install rustyhook
```

### Manual Installation

If you prefer to build from source:

1. Clone the repository:
   ```sh
   git clone https://github.com/your-org/rustyhook.git
   ```

2. Navigate to the directory:
   ```sh
   cd rustyhook
   ```

3. Build the project:
   ```sh
   cargo build --release
   ```

4. The binary will be available at `./target/release/rustyhook`

5. (Optional) Add to your PATH:
   ```sh
   # On Unix-like systems
   cp ./target/release/rustyhook /usr/local/bin/

   # On Windows, move to a directory in your PATH or add the release directory to your PATH
   ```

## Verifying Installation

To verify that RustyHook is installed correctly:

```sh
rustyhook --version
# or using the alias
rh --version
```

## Updating RustyHook

To update to the latest version:

```sh
cargo install rustyhook --force
```

## Uninstalling

To uninstall RustyHook:

```sh
cargo uninstall rustyhook
```

## Troubleshooting

### Common Issues

- **Command not found**: Ensure the installation directory is in your PATH
- **Permission denied**: You may need to use `sudo` on Unix-like systems
- **Compilation errors**: Make sure you have the latest version of Rust and Cargo

### Getting Help

If you encounter any issues during installation, please:

1. Check the [FAQ](../faq.md) for common problems
2. Search for existing issues on the [GitHub repository](https://github.com/your-org/rustyhook/issues)
3. Open a new issue if your problem hasn't been reported

## Next Steps

After installation, you might want to:

- [Get started](getting-started.md) with RustyHook
- Learn about [CLI usage](cli-usage.md)
- Set up [shell completions](shell-completions.md)
