$ErrorActionPreference = 'Stop'

$packageName = 'rustyhook'
$toolsDir = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$url64 = 'https://github.com/your-org/rustyhook/releases/download/v0.1.0/rustyhook-v0.1.0-x86_64-pc-windows-msvc.zip'
$checksum64 = 'REPLACE_WITH_ACTUAL_CHECKSUM_AFTER_RELEASE'
$checksumType64 = 'sha256'

Install-ChocolateyZipPackage -PackageName $packageName `
                             -Url64bit $url64 `
                             -UnzipLocation $toolsDir `
                             -Checksum64 $checksum64 `
                             -ChecksumType64 $checksumType64

# Create a shim for the executable
$files = Get-ChildItem $toolsDir -Include "rh.exe" -Recurse
foreach ($file in $files) {
  New-Item "$file.ignore" -Type File -Force | Out-Null
  Set-Content -Path "$file.gui" -Value "" -Force
}

Write-Host "RustyHook has been installed. You can now use 'rh' from the command line."