# RustyHook Cross-Platform Packaging

This directory contains scripts and configurations for building RustyHook packages for various operating systems.

## Quick Start

To build packages for all supported platforms:

```bash
chmod +x packaging/installers/build.sh
./packaging/installers/build.sh all
```

To build for a specific platform:

```bash
./packaging/installers/build.sh linux
./packaging/installers/build.sh macos
./packaging/installers/build.sh windows
```

## Package Types

The build script creates the following package types by default:

- **Linux**: `.tar.gz` archives
- **macOS**: `.tar.gz` archives
- **Windows**: `.zip` archives

## Advanced Packaging

For more advanced packaging formats, additional tools are required:

### Debian/Ubuntu (.deb)

To create `.deb` packages, install `cargo-deb`:

```bash
cargo install cargo-deb
```

Then uncomment the relevant lines in `build.sh` or run:

```bash
cargo deb
```

### Windows (.msi)

To create `.msi` installers, you need the WiX Toolset:

1. Install WiX Toolset: https://wixtoolset.org/
2. Install `cargo-wix`:

```bash
cargo install cargo-wix
```

Then run:

```bash
cargo wix
```

### macOS (.pkg)

To create `.pkg` installers, you need macOS developer tools:

1. Install Xcode Command Line Tools:

```bash
xcode-select --install
```

2. Use `pkgbuild` to create the package:

```bash
pkgbuild --root <path-to-binaries> --identifier com.rustyhook --version <version> rustyhook.pkg
```

## Cross-Compilation Setup

To build for multiple platforms, you need to set up cross-compilation:

### For Linux targets:

```bash
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-unknown-linux-musl
```

### For macOS targets:

```bash
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
```

### For Windows targets:

```bash
rustup target add x86_64-pc-windows-msvc
```

## CI/CD Integration

These scripts are designed to be used in CI/CD pipelines. See the GitHub Actions workflows in the `.github/workflows` directory for examples of how to automate the packaging process.

## Package Structure

Each package follows this structure:

```
rustyhook-<version>-<target>/
├── bin/
│   └── rh (or rh.exe for Windows)
├── LICENSE
└── README.md
```

## Customization

To customize the packaging process, edit the `build.sh` script. You can modify:

- Output directory
- Package name
- Included files
- Target platforms
- Package formats