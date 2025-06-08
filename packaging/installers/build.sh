#!/bin/bash
set -e

# Configuration
VERSION=$(grep '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
NAME="rustyhook"
BINARY_NAME="rh"
OUTPUT_DIR="target/packages"

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo "Building packages for rustyhook v$VERSION..."

# Function to build for a specific target
build_for_target() {
    local target=$1
    local package_type=$2
    local extension=$3
    
    echo "Building for $target..."
    
    # Build the binary
    cargo build --release --target "$target"
    
    # Create package directory
    local package_dir="$OUTPUT_DIR/$NAME-$VERSION-$target"
    mkdir -p "$package_dir/bin"
    
    # Copy binary and license
    if [[ "$target" == *"windows"* ]]; then
        cp "target/$target/release/$BINARY_NAME.exe" "$package_dir/bin/"
    else
        cp "target/$target/release/$BINARY_NAME" "$package_dir/bin/"
    fi
    cp LICENSE "$package_dir/"
    cp README.md "$package_dir/"
    
    # Create package
    echo "Creating $package_type package..."
    case "$package_type" in
        "zip")
            (cd "$OUTPUT_DIR" && zip -r "$NAME-$VERSION-$target.zip" "$(basename "$package_dir")")
            ;;
        "tar")
            (cd "$OUTPUT_DIR" && tar -czf "$NAME-$VERSION-$target.tar.gz" "$(basename "$package_dir")")
            ;;
        "deb")
            # For .deb packages, we'd use a tool like fpm or cargo-deb
            echo "DEB packaging requires additional tools. See packaging/installers/README.md"
            ;;
        "msi")
            # For .msi packages, we'd use a tool like WiX
            echo "MSI packaging requires additional tools. See packaging/installers/README.md"
            ;;
        "pkg")
            # For .pkg packages, we'd use macOS tools
            echo "PKG packaging requires additional tools. See packaging/installers/README.md"
            ;;
    esac
    
    echo "Package created at $OUTPUT_DIR/$NAME-$VERSION-$target$extension"
}

# Build for various platforms
if [[ "$1" == "all" || "$1" == "linux" ]]; then
    build_for_target "x86_64-unknown-linux-gnu" "tar" ".tar.gz"
    build_for_target "x86_64-unknown-linux-musl" "tar" ".tar.gz"
    # Uncomment to build .deb packages
    # build_for_target "x86_64-unknown-linux-gnu" "deb" ".deb"
fi

if [[ "$1" == "all" || "$1" == "macos" ]]; then
    build_for_target "x86_64-apple-darwin" "tar" ".tar.gz"
    build_for_target "aarch64-apple-darwin" "tar" ".tar.gz"
    # Uncomment to build .pkg packages
    # build_for_target "x86_64-apple-darwin" "pkg" ".pkg"
    # build_for_target "aarch64-apple-darwin" "pkg" ".pkg"
fi

if [[ "$1" == "all" || "$1" == "windows" ]]; then
    build_for_target "x86_64-pc-windows-msvc" "zip" ".zip"
    # Uncomment to build .msi packages
    # build_for_target "x86_64-pc-windows-msvc" "msi" ".msi"
fi

echo "All packages built successfully!"