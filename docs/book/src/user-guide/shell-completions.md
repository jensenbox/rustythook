# Shell Completions

RustyHook provides shell completion scripts for Bash, Zsh, Fish, and PowerShell. This guide explains how to generate and install these completion scripts to enhance your command-line experience.

## Generating Completion Scripts

RustyHook can generate completion scripts for various shells using the `completions` command:

```sh
rustyhook completions <SHELL>
```

Replace `<SHELL>` with one of the following:
- `bash`: For Bash shell
- `zsh`: For Zsh shell
- `fish`: For Fish shell
- `powershell`: For PowerShell

## Installing Completion Scripts

### Bash

```sh
# Create completion directory if it doesn't exist
mkdir -p ~/.bash_completion.d

# Generate and save completion script
rustyhook completions bash > ~/.bash_completion.d/rustyhook

# For the alias (rh)
rustyhook completions bash | sed 's/rustyhook/rh/g' > ~/.bash_completion.d/rh

# Source the completion script
echo 'source ~/.bash_completion.d/rustyhook' >> ~/.bashrc
echo 'source ~/.bash_completion.d/rh' >> ~/.bashrc

# Apply changes
source ~/.bashrc
```

### Zsh

```sh
# Create completion directory if it doesn't exist
mkdir -p ~/.zsh/completions

# Generate and save completion script
rustyhook completions zsh > ~/.zsh/completions/_rustyhook

# For the alias (rh)
rustyhook completions zsh | sed 's/rustyhook/rh/g' > ~/.zsh/completions/_rh

# Make sure ~/.zsh/completions is in your fpath
echo 'fpath=(~/.zsh/completions $fpath)' >> ~/.zshrc
echo 'autoload -U compinit && compinit' >> ~/.zshrc

# Apply changes
source ~/.zshrc
```

### Fish

```sh
# Create completion directory if it doesn't exist
mkdir -p ~/.config/fish/completions

# Generate and save completion script
rustyhook completions fish > ~/.config/fish/completions/rustyhook.fish

# For the alias (rh)
rustyhook completions fish | sed 's/rustyhook/rh/g' > ~/.config/fish/completions/rh.fish

# Fish automatically loads completions from this directory
```

### PowerShell

```powershell
# Create a directory for the completion script if it doesn't exist
if (-not (Test-Path -Path "$PROFILE\.." -PathType Container)) {
    New-Item -Path "$PROFILE\.." -ItemType Directory -Force
}

# Generate and save completion script
rustyhook completions powershell > $PROFILE.CurrentUserCurrentHost/rustyhook.ps1

# For the alias (rh)
rustyhook completions powershell | ForEach-Object { $_ -replace "rustyhook", "rh" } > $PROFILE.CurrentUserCurrentHost/rh.ps1

# Source the completion script
echo '. $PROFILE.CurrentUserCurrentHost/rustyhook.ps1' >> $PROFILE
echo '. $PROFILE.CurrentUserCurrentHost/rh.ps1' >> $PROFILE

# Apply changes
. $PROFILE
```

## Verifying Completions

After installing the completion scripts, you can verify they're working by typing `rustyhook` or `rh` followed by a space and pressing the Tab key. You should see available commands and options.

## Troubleshooting

### Completions Not Working

If completions aren't working after installation:

1. Make sure you've sourced your shell configuration file (`.bashrc`, `.zshrc`, etc.)
2. Verify that the completion script was generated correctly
3. Check that the completion script is in the correct location
4. Ensure your shell is configured to use completions

### Bash Completions

If Bash completions aren't working:

```sh
# Add this to your .bashrc if it's not already there
if [ -d ~/.bash_completion.d ]; then
  for file in ~/.bash_completion.d/*; do
    . "$file"
  done
fi
```

### Zsh Completions

If Zsh completions aren't working:

```sh
# Make sure compinit is loaded
autoload -Uz compinit && compinit
```

## Next Steps

- Learn about [CLI Usage](cli-usage.md)
- Explore [Configuration](configuration.md) options
- Check out the [Migration Guide](migration.md) if you're coming from pre-commit