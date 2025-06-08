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

RustyHook can run `.pre-commit-config.yaml` directly:

```sh
rh compat
```

To migrate to native format:

```sh
rh convert --from-precommit > .rustyhook/config.yaml
```

---

## 🧰 Supported Environments

| Language | Setup Method       |
| -------- | ------------------ |
| Python   | `virtualenv + pip` |
| Node.js  | `npm` or `pnpm`    |
| Ruby     | `rbenv + bundler`  |

Environments are cached in `.rustyhook/cache/` and versioned by `{tool}-{version}`. RustyHook uses hashes of config + tool version to determine cache freshness.

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
