$ErrorActionPreference = 'Stop'

$packageName = 'rustyhook'
$toolsDir = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"

# Remove any created files or registry entries if necessary
# For this package, Chocolatey will handle most of the uninstallation automatically

Write-Host "RustyHook has been uninstalled."