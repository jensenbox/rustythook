# Migration from pre-commit

RustyHook is designed to be a drop-in replacement for [pre-commit](https://pre-commit.com/), making migration as seamless as possible. This guide will help you transition from pre-commit to RustyHook.

## Compatibility Mode

The easiest way to start using RustyHook is with compatibility mode, which allows RustyHook to use your existing `.pre-commit-config.yaml` file without any changes:

```sh
# Run hooks using your existing pre-commit configuration
rh compat
```

This command will:
1. Read your `.pre-commit-config.yaml` file
2. Set up the necessary environments
3. Run the hooks as defined in your pre-commit configuration

## Converting Your Configuration

While compatibility mode works well, you'll get the best performance and features by converting to RustyHook's native configuration format:

```sh
# Convert pre-commit config to RustyHook config
rh convert --from-precommit > .rustyhook/config.yaml
```

This will create a new `.rustyhook/config.yaml` file based on your existing pre-commit configuration.

## Configuration Differences

Here's how pre-commit and RustyHook configurations compare:

### pre-commit Configuration

```yaml
# .pre-commit-config.yaml
repos:
  - repo: https://github.com/charliermarsh/ruff-pre-commit
    rev: 'v0.0.262'
    hooks:
      - id: ruff
        args: [--fix]
  
  - repo: https://github.com/pre-commit/mirrors-eslint
    rev: v8.38.0
    hooks:
      - id: eslint
        files: \.(js|jsx|ts|tsx)$
        types: [file]
        additional_dependencies:
          - eslint@8.38.0
          - eslint-config-standard@17.0.0
```

### Equivalent RustyHook Configuration

```yaml
# .rustyhook/config.yaml
hooks:
  - id: ruff
    language: python
    version: "==0.0.262"
    entry: "ruff"
    args: ["--fix"]
    files: "\\.py$"
  
  - id: eslint
    language: node
    version: "^8.38.0"
    entry: "eslint"
    files: "\\.(js|jsx|ts|tsx)$"
    args: []
    dependencies:
      - "eslint-config-standard@17.0.0"
```

## Key Differences

1. **Repository References**: RustyHook doesn't use the `repos` structure. Instead, it directly defines hooks with their language and version.

2. **Version Specification**: 
   - pre-commit uses Git revisions (`rev`)
   - RustyHook uses package version specifiers (`version`)

3. **Dependencies**: 
   - pre-commit uses `additional_dependencies`
   - RustyHook uses `dependencies`

4. **Entry Point**: 
   - pre-commit infers the entry point from the hook ID
   - RustyHook requires an explicit `entry` field

5. **File Patterns**: 
   - Both use similar regex patterns, but RustyHook always requires the `files` field if you want to filter by file type

## Migrating Git Hooks

If you've installed pre-commit as a Git hook, you'll need to uninstall it and install RustyHook instead:

```sh
# Uninstall pre-commit hooks
pre-commit uninstall

# Install RustyHook hooks
rh install
```

## Handling Custom Hooks

If you have custom hooks defined in your pre-commit configuration, you'll need to adapt them for RustyHook:

### pre-commit Custom Hook

```yaml
repos:
  - repo: local
    hooks:
      - id: custom-script
        name: Custom Script
        entry: ./scripts/custom.sh
        language: script
        files: \.txt$
```

### RustyHook Custom Hook

```yaml
hooks:
  - id: custom-script
    language: system
    entry: "./scripts/custom.sh"
    files: "\\.txt$"
```

## Handling Local Hooks

RustyHook treats all hooks as "local" by default. There's no need for a special `local` repository designation.

## Troubleshooting Migration Issues

### Common Issues

1. **Missing Dependencies**: If your pre-commit hooks had implicit dependencies, you might need to add them explicitly in RustyHook.

2. **Path Differences**: RustyHook executes hooks from the Git root by default. Use `working_dir` if you need to change this.

3. **Hook Execution Order**: RustyHook executes hooks in the order they're defined in the configuration file.

### Debugging

If you encounter issues during migration, use the `--verbose` flag to get more information:

```sh
rh run --verbose
```

You can also use the `doctor` command to diagnose issues:

```sh
rh doctor
```

## Next Steps

After migrating to RustyHook, you might want to:

- Explore [Configuration](configuration.md) options to take advantage of RustyHook-specific features
- Learn about [CLI Usage](cli-usage.md) to optimize your workflow
- Set up [Shell Completions](shell-completions.md) for a better command-line experience