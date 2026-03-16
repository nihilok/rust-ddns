# uninstall.ps1 — Windows uninstaller for rust-ddns

$ErrorActionPreference = "SilentlyContinue"

$InstallDir = "$env:LOCALAPPDATA\rust-ddns"
$TaskName = "rust-ddns"

Write-Host "Removing scheduled task '$TaskName'..."
schtasks /Delete /F /TN $TaskName 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Host "Scheduled task not found or already removed."
}

if (Test-Path $InstallDir) {
    Write-Host "Removing install directory $InstallDir..."
    Remove-Item -Recurse -Force $InstallDir
}

Write-Host "Uninstallation complete!"
