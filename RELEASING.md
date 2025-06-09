# Release Process for RustyHook

This document outlines the process for creating and deploying new releases of RustyHook.

## Prerequisites

- [cargo-release](https://github.com/crate-ci/cargo-release) installed:
  ```bash
  cargo install cargo-release
  ```
- Git access to the repository with push permissions
- Clean working directory (no uncommitted changes)

## Release Process

RustyHook uses [Semantic Versioning](https://semver.org/) for its releases:
- **MAJOR** version for incompatible API changes
- **MINOR** version for backwards-compatible functionality additions
- **PATCH** version for backwards-compatible bug fixes

### Steps to Create a New Release

1. Ensure all changes for the release are committed and pushed to the main branch.

2. Run the tests to make sure everything is working:
   ```bash
   cargo test
   ```

3. Create a new release using cargo-release:

   For a patch release (bug fixes):
   ```bash
   cargo release patch
   ```

   For a minor release (new features):
   ```bash
   cargo release minor
   ```

   For a major release (breaking changes):
   ```bash
   cargo release major
   ```

   This will:
   - Update the version in Cargo.toml
   - Create a git commit with the changes
   - Create a git tag for the release
   - Prepare for the next development iteration

4. Push the changes and tag to the remote repository:
   ```bash
   cargo release patch --push --tag
   ```

5. Create a GitHub release based on the new tag:
   - Go to the [GitHub Releases page](https://github.com/yourusername/rustyhook/releases)
   - Click "Draft a new release"
   - Select the tag you just created
   - Fill in the release title and description
   - Attach any binary artifacts if needed
   - Publish the release

## Release Automation

The release.toml file in the project root configures the behavior of cargo-release. The current configuration:

- Sets the default version bump to "patch"
- Disables automatic publishing to crates.io
- Disables automatic pushing to the remote repository
- Enables automatic tag creation
- Sets commit messages for pre-release and post-release
- Runs tests before release
- Restricts releases to the "main" branch

## GitHub Actions Integration

A GitHub Action workflow can be set up to automatically build and publish releases when a new tag is pushed. This would include:

1. Building the project
2. Running tests
3. Creating release artifacts
4. Publishing the release to GitHub Releases
5. Updating documentation

## Troubleshooting

If you encounter issues during the release process:

1. Ensure your working directory is clean
2. Verify you have the necessary permissions
3. Check that all tests are passing
4. Review the cargo-release documentation for advanced options