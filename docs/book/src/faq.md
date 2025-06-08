# Frequently Asked Questions

## General Questions

### What is RustyHook?

RustyHook is a blazing-fast, Rust-native Git hook runner designed to be a modern, drop-in replacement for `pre-commit`. It is language-agnostic, monorepo-friendly, and automatically provisions environments for linters and checkers written in Python, Node.js, Ruby, and more.

### How does RustyHook compare to pre-commit?

RustyHook is designed to be a faster, more efficient alternative to pre-commit with these key advantages:
- Written in Rust for better performance
- Concurrent hook execution
- Native support for monorepos
- Automatic environment setup for multiple languages
- Compatible with existing pre-commit configurations

### Is RustyHook compatible with pre-commit configurations?

Yes! RustyHook can run your existing `.pre-commit-config.yaml` files using the `rh compat` command. You can also convert your pre-commit configuration to RustyHook's native format using `rh convert`.

## Installation and Setup

### How do I install RustyHook?

The easiest way to install RustyHook is using Cargo:

```sh
cargo install rustyhook
```

For other installation methods, see the [Installation Guide](user-guide/installation.md).

### How do I set up RustyHook in my project?

To set up RustyHook in your project:

1. Install RustyHook
2. Initialize a configuration file:
   ```sh
   rh init
   ```
3. Edit the `.rustyhook/config.yaml` file to configure your hooks
4. Install the Git hooks:
   ```sh
   rh install
   ```

### Can I use RustyHook with CI/CD systems?

Yes, RustyHook works well in CI/CD environments. You can run hooks using:

```sh
rh run --all-files
```

This will run all hooks on all files, not just changed ones.

## Configuration

### How do I configure RustyHook?

RustyHook uses a YAML configuration file located at `.rustyhook/config.yaml`. See the [Configuration Guide](user-guide/configuration.md) for details.

### Can I use RustyHook in a monorepo?

Yes, RustyHook is designed with monorepos in mind. You can have multiple configuration files in different directories, and RustyHook will use the closest configuration file to the Git root.

### How do I add a new hook?

To add a new hook, edit your `.rustyhook/config.yaml` file and add a new entry to the `hooks` array:

```yaml
hooks:
  - id: my-new-hook
    language: python
    version: "==1.0.0"
    entry: "my-hook-command"
    files: "\\.py$"
```

## Troubleshooting

### My hooks aren't running. What should I check?

1. Make sure RustyHook is installed correctly (`rh --version`)
2. Check that your configuration file is valid (`rh doctor`)
3. Verify that Git hooks are installed (`ls -la .git/hooks`)
4. Run hooks manually to see any errors (`rh run --verbose`)

### How do I debug hook failures?

Use the `--verbose` flag to get more detailed output:

```sh
rh run --verbose
```

You can also use the `doctor` command to diagnose issues:

```sh
rh doctor
```

### How do I clean up cached environments?

To clean up cached environments:

```sh
rh clean --all
```

Or to clean only specific language environments:

```sh
rh clean --language python
```

## Advanced Usage

### Can I run hooks on specific files?

Yes, you can run hooks on specific files:

```sh
rh run --files src/main.rs,src/lib.rs
```

### How do I skip hooks for a specific commit?

To skip hooks for a specific commit:

```sh
git commit --no-verify
```

Or with the `-n` shorthand:

```sh
git commit -n
```

### Can I use RustyHook with hooks other than pre-commit?

Yes, RustyHook supports various Git hooks:

```sh
rh install --hook-type pre-push
```

## Contributing

### How can I contribute to RustyHook?

We welcome contributions! See the [Contributing Guide](contributing/development-setup.md) for details on how to get started.

### Where can I report bugs or request features?

You can report bugs or request features on the [GitHub issue tracker](https://github.com/your-org/rustyhook/issues).