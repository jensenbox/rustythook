FROM rust:latest

LABEL maintainer="RustyHook Team"
LABEL description="Docker image for compiling and developing RustyHook"

# Install additional dependencies
# Note: Python, Node.js, and npm are included for development and testing purposes
# RustyHook itself can download and manage these toolchains, but having them pre-installed
# in the development environment can be useful for testing hooks that use these languages
RUN apt-get update && apt-get install -y \
    git \
    python3 \
    python3-pip \
    nodejs \
    npm \
    && rm -rf /var/lib/apt/lists/*

# Set up working directory
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files to cache dependencies
COPY Cargo.toml Cargo.lock* ./

# Create a dummy main.rs to build dependencies
RUN mkdir -p src && \
    echo "fn main() {println!(\"Dummy\");}" > src/main.rs && \
    cargo build && \
    rm -rf src

# Set up environment variables
ENV RUST_BACKTRACE=1

# Add a volume for the project code
VOLUME ["/app"]

# Default command
CMD ["cargo", "build"]

# Usage instructions as a comment
# To use this image:
# 1. Build: docker build -t rustyhook-dev -f docker/Dockerfile .
# 2. Run: docker run -v $(pwd):/app rustyhook-dev
# 3. For development: docker run -it -v $(pwd):/app rustyhook-dev bash
