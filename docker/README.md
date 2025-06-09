# RustyHook Development Docker Image

This directory contains a Dockerfile for setting up a development environment for RustyHook. Using this Docker image reduces the overhead of setting up a development environment by providing all the necessary dependencies pre-installed.

## Features

- Pre-installed Rust toolchain
- Git for version control
- Python and pip for Python-based hooks (for development and testing purposes)
- Node.js and npm for JavaScript-based hooks (for development and testing purposes)
- Dependency caching for faster builds
- Volume mounting for seamless code editing

Note: While RustyHook is designed to download and manage Python, Node.js, and other toolchains itself, having them pre-installed in the development environment can be useful for testing hooks that use these languages.

## Usage

### Building the Docker Image

```bash
docker build -t rustyhook-dev -f docker/Dockerfile .
```

### Running a Build

To compile the project:

```bash
docker run -v $(pwd):/app rustyhook-dev
```

### Interactive Development

For an interactive development session:

```bash
docker run -it -v $(pwd):/app rustyhook-dev bash
```

Once inside the container, you can run commands like:

```bash
cargo build
cargo test
cargo run -- <args>
```

### Running Tests

To run the test suite:

```bash
docker run -v $(pwd):/app rustyhook-dev cargo test
```

## Customization

You can customize the Docker image by modifying the Dockerfile. For example, you might want to:

- Add additional dependencies
- Change the Rust version
- Configure environment variables

## Troubleshooting

If you encounter permission issues with the mounted volume, you may need to adjust the permissions:

```bash
docker run -v $(pwd):/app -u $(id -u):$(id -g) rustyhook-dev
```

## CI/CD Integration

This Docker image can also be used in CI/CD pipelines to ensure consistent build environments.
