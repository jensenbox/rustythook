# Project Guidelines for RustyHook

## Project Overview
RustyHook is a blazing-fast, Rust-native Git hook runner designed to be a modern, drop-in replacement for `pre-commit`. It is language-agnostic, monorepo-friendly, and automatically provisions environments for linters and checkers written in Python, Node.js, Ruby, and more.

## Project Structure
- **src/**: Main source code
  - **bin/**: Binary executables (rh.rs, rustyhook_hooks.rs)
  - **cache/**: Caching functionality
  - **config/**: Configuration parsing and handling
  - **hooks/**: Hook implementations
  - **runner/**: Hook filtering and dispatch
  - **toolchains/**: Python, Node, Ruby setup
  - **lib.rs**: Library code
  - **logging.rs**: Logging functionality
  - **main.rs**: Main entry point
- **tests/**: Test files
  - **cli_tests.rs**: Tests for CLI functionality
  - **compat_tests.rs**: Tests for compatibility with pre-commit
  - **config_tests.rs**: Tests for configuration parsing
  - **hook_execution_tests.rs**: Tests for hook execution
  - **precommit_config_tests.rs**: Tests for pre-commit config handling
- **.rustyhook/**: Project configuration
  - **cache/**: Cache for tool installations
  - **config.yaml**: Main configuration file
  - **venvs/**: Python virtual environments
- **docs/**: Documentation
- **packaging/**: Packaging for different platforms

## Building and Testing
Before submitting any changes, Junie should:

1. **Build the project**:
   ```bash
   cargo build
   ```

2. **Run the application without arguments to check for errors**:
   ```bash
   cargo run
   ```

3. **Run the test suite**:
   ```bash
   cargo test
   ```

4. **Check code style and linting**:
   ```bash
   cargo fmt -- --check
   cargo clippy -- -D warnings
   ```

## Rust Best Practices

### Code Style
- Follow the Rust style guide as enforced by `rustfmt`
- Use `cargo fmt` to format code before committing
- Use `cargo clippy` to catch common mistakes and improve code quality

### Error Handling
- Use `Result` and `Option` types for error handling
- Avoid panics in library code; propagate errors to the caller
- Use the `?` operator for concise error propagation
- Provide meaningful error messages

### Performance
- Prefer iterators over explicit loops when appropriate
- Use `&str` instead of `String` for function parameters when possible
- Avoid unnecessary cloning of data
- Use async/await for I/O-bound operations

### Testing
- Write unit tests for all public functions
- Use integration tests for testing the CLI
- Mock external dependencies in tests
- Test edge cases and error conditions

### Documentation
- Document all public APIs with rustdoc comments
- Include examples in documentation where appropriate
- Keep documentation up-to-date with code changes

### Dependencies
- Minimize dependencies to reduce build times and potential security issues
- Keep dependencies up-to-date
- Prefer well-maintained crates with good documentation

### Safety
- Avoid `unsafe` code unless absolutely necessary
- If `unsafe` is used, document why it's necessary and how safety is maintained
- Use the `#[deny(unsafe_code)]` attribute when possible

## Feature Implementation Guidelines
When implementing new features:

1. Start by writing tests that define the expected behavior
2. Implement the feature with clean, well-documented code
3. Ensure the feature works with existing functionality
4. Update documentation to reflect the new feature
5. Run the full test suite to ensure nothing was broken

## Pull Request Guidelines
Pull requests should:

1. Address a single concern
2. Include tests for new functionality
3. Pass all existing tests
4. Follow the code style guidelines
5. Include documentation updates if necessary
