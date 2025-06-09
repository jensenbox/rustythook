#!/bin/bash
set -e

echo "Building Docker image..."
docker build -t rustyhook-test -f docker/Dockerfile .

echo "Running a simple command in the Docker container..."
docker run --rm rustyhook-test cargo --version

echo "Testing if the Docker image can build the project..."
docker run --rm -v $(pwd):/app rustyhook-test cargo build

echo "All Docker tests passed!"