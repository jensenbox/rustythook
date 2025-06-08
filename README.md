# RustyHook

**RustyHook** is a blazing-fast, Rust-native Git hook runner designed to be a modern, drop-in replacement for [`pre-commit`](https://pre-commit.com/). It is language-agnostic, monorepo-friendly, and automatically provisions environments for linters and checkers written in Python, Node.js, Ruby, and more.

---

## 🔧 Features

* 🚀 **Fast and concurrent** execution powered by Rust
* 🧰 **Automatic setup** of Python virtualenvs, Node/npm environments, and Ruby/bundler installs
* 🌐 **Language-agnostic** support with consistent hook interface
* 🏗️ **Monorepo-first** with per-directory or per-project configurations
* 🪝 **Compatibility with `.pre-commit-config.yaml`**
* 🧙 **Migration tool** to convert pre-commit configs to native `.rustyhook/config.yaml`
* 📦 **Cache-aware** tool installs and version pinning
* 🆔 CLI alias: `rh`

---

## 🚀 Getting Started

### Installation

```sh
cargo install rustyhook
```

Or clone and build manually:

```sh
git clone https://github.com/your-org/rustyhook.git
cd rustyhook
cargo build --release
```

---

## 🛠 CLI Usage

Both `rustyhook` and `rh` are available:

```sh
rh run               # Run hooks from .rustyhook/config.yaml
rh compat            # Run from .pre-commit-config.yaml
rh convert           # Convert pre-commit config to native format
rh init              # Scaffold a new .rustyhook/config.yaml
rh list              # List configured hooks
rh doctor            # Diagnose tool/setup issues
rh clean             # Remove cached environments
rh completions       # Generate shell completion scripts
```

---

## 🧪 Example Configuration

### `.rustyhook/config.yaml`

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

---

## 🔄 Migration from `pre-commit`

RustyHook is designed to be a drop-in replacement for [pre-commit](https://pre-commit.com/), making migration as seamless as possible.

### Compatibility Mode

The easiest way to start using RustyHook is with compatibility mode, which allows RustyHook to use your existing `.pre-commit-config.yaml` file without any changes:

```sh
rh compat
```

This command will read your `.pre-commit-config.yaml` file, set up the necessary environments, and run the hooks as defined in your pre-commit configuration.

### Converting Your Configuration

While compatibility mode works well, you'll get the best performance and features by converting to RustyHook's native configuration format:

```sh
# Convert pre-commit config to RustyHook config
rh convert --from-precommit > .rustyhook/config.yaml
```

This will create a new `.rustyhook/config.yaml` file based on your existing pre-commit configuration.

### Migrating Git Hooks

If you've installed pre-commit as a Git hook, you'll need to uninstall it and install RustyHook instead:

```sh
# Uninstall pre-commit hooks
pre-commit uninstall

# Install RustyHook hooks
rh install
```

### Key Differences

1. **Repository References**: RustyHook doesn't use the `repos` structure. Instead, it directly defines hooks with their language and version.
2. **Version Specification**: RustyHook uses package version specifiers (`version`) instead of Git revisions (`rev`).
3. **Dependencies**: RustyHook uses `dependencies` instead of `additional_dependencies`.
4. **Entry Point**: RustyHook requires an explicit `entry` field, while pre-commit infers it from the hook ID.
5. **Local Hooks**: RustyHook treats all hooks as "local" by default. There's no need for a special `local` repository designation.

For more detailed information about migrating from pre-commit to RustyHook, see the [Migration Guide](https://your-org.github.io/rustyhook/user-guide/migration.html).

---

## 🧰 Supported Environments

| Language | Setup Method       |
| -------- | ------------------ |
| Python   | `virtualenv + pip` |
| Node.js  | `npm` or `pnpm`    |
| Ruby     | `rbenv + bundler`  |

Environments are cached in `.rustyhook/cache/` and versioned by `{tool}-{version}`. RustyHook uses hashes of config + tool version to determine cache freshness.

---

## 🔄 Shell Completions

RustyHook provides shell completion scripts for Bash, Zsh, Fish, and PowerShell. You can generate and install them as follows:

### Bash

```sh
# Generate and save completion script
rustyhook completions bash > ~/.bash_completion.d/rustyhook
# Or for the alias
rustyhook completions bash | sed 's/rustyhook/rh/g' > ~/.bash_completion.d/rh

# Source the completion script
source ~/.bash_completion.d/rustyhook
```

### Zsh

```sh
# Generate and save completion script
rustyhook completions zsh > ~/.zsh/completions/_rustyhook
# Or for the alias
rustyhook completions zsh | sed 's/rustyhook/rh/g' > ~/.zsh/completions/_rh

# Make sure ~/.zsh/completions is in your fpath
echo 'fpath=(~/.zsh/completions $fpath)' >> ~/.zshrc
echo 'autoload -U compinit && compinit' >> ~/.zshrc
```

### Fish

```sh
# Generate and save completion script
rustyhook completions fish > ~/.config/fish/completions/rustyhook.fish
# Or for the alias
rustyhook completions fish | sed 's/rustyhook/rh/g' > ~/.config/fish/completions/rh.fish
```

### PowerShell

```powershell
# Generate and save completion script
rustyhook completions powershell > $PROFILE.CurrentUserCurrentHost/rustyhook.ps1
# Or for the alias
rustyhook completions powershell | ForEach-Object { $_ -replace "rustyhook", "rh" } > $PROFILE.CurrentUserCurrentHost/rh.ps1

# Source the completion script
echo '. $PROFILE.CurrentUserCurrentHost/rustyhook.ps1' >> $PROFILE
```

---

## 👩‍💻 Contributing

### For LLM agents and developers:

* Read `Rustyhook Spec` (see `/specs`) for architecture and module layout
* All CLI commands are implemented using `clap`
* YAML parsing uses `serde_yaml`
* Environments are bootstrapped from scratch using shell-less Rust process invocations (`std::process::Command`)
* Code is modular under:

    * `toolchains/`: Python, Node, Ruby setup
    * `config/`: Config and compat parsing
    * `runner/`: Hook filtering and dispatch

### Dev Environment Setup

```sh
rustup override set stable
cargo check
cargo test
```

### To build:

```sh
cargo build --release
```

---

## 📜 License

MIT

---

## 🤝 Acknowledgments

* Inspired by `pre-commit`, `lefthook`, `moonrepo`, and the Rust community
* Shoutout to contributors and early testers helping shape this project

---

## 📣 TODO

* Add support for Go and Deno
* CI integrations for GitHub Actions
* Plugin architecture for third-party hook runners
