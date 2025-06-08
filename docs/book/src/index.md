# RustyHook

**RustyHook** is a blazing-fast, Rust-native Git hook runner designed to be a modern, drop-in replacement for [`pre-commit`](https://pre-commit.com/). It is language-agnostic, monorepo-friendly, and automatically provisions environments for linters and checkers written in Python, Node.js, Ruby, and more.

## Features

* 🚀 **Fast and concurrent** execution powered by Rust
* 🧰 **Automatic setup** of Python virtualenvs, Node/npm environments, and Ruby/bundler installs
* 🌐 **Language-agnostic** support with consistent hook interface
* 🏗️ **Monorepo-first** with per-directory or per-project configurations
* 🪝 **Compatibility with `.pre-commit-config.yaml`**
* 🧙 **Migration tool** to convert pre-commit configs to native `.rustyhook/config.yaml`
* 📦 **Cache-aware** tool installs and version pinning
* 🆔 CLI alias: `rh`

## Quick Start

```sh
# Install RustyHook
cargo install rustyhook

# Initialize a new configuration
rh init

# Run your hooks
rh run
```

For more detailed information, check out the [Getting Started](user-guide/getting-started.md) guide.

## Documentation Sections

- **[User Guide](user-guide/getting-started.md)**: Learn how to install, configure, and use RustyHook
- **[Reference](reference/supported-languages.md)**: Detailed reference for supported languages, hook types, and configuration options
- **[Advanced](advanced/monorepo.md)**: Advanced topics like monorepo support, custom hooks, and performance tuning
- **[Contributing](contributing/development-setup.md)**: Information for developers who want to contribute to RustyHook

## License

RustyHook is licensed under the MIT License.

## Acknowledgments

* Inspired by `pre-commit`, `lefthook`, `moonrepo`, and the Rust community
* Shoutout to contributors and early testers helping shape this project