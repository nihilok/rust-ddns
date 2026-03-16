# install.ps1 — Windows installer for rust-ddns

$ErrorActionPreference = "Stop"

$InstallDir = "$env:LOCALAPPDATA\rust-ddns"
$BinaryDest = "$InstallDir\rust-ddns.exe"
$WrapperDest = "$InstallDir\ddnsd.cmd"
$TaskName = "rust-ddns"

# Determine interval in seconds
$IntervalEnv = $env:RUST_DDNS_INTERVAL
if (-not $IntervalEnv) {
    $IntervalSecs = 300
} elseif ($IntervalEnv -match '^(\d+)min$') {
    $IntervalSecs = [int]$Matches[1] * 60
} elseif ($IntervalEnv -match '^(\d+)h$') {
    $IntervalSecs = [int]$Matches[1] * 3600
} else {
    $IntervalSecs = [int]$IntervalEnv
}

Write-Host "Compiling Rust application..."
cargo build --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "Creating install directory $InstallDir..."
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

Write-Host "Copying binary to $BinaryDest..."
Copy-Item -Force "target\release\rust-ddns.exe" $BinaryDest

Write-Host "Writing wrapper script to $WrapperDest..."
$LogFile = if ($env:RUST_DDNS_LOG_FILE) { $env:RUST_DDNS_LOG_FILE } else { "$env:USERPROFILE\.rust-ddns.log" }
$WrapperContent = @"
@echo off
set LOG_FILE=$LogFile
if not exist "%LOG_FILE%" type nul > "%LOG_FILE%"
"$BinaryDest" >> "%LOG_FILE%" 2>&1
powershell -Command "Get-Content '%LOG_FILE%' -Tail 200 | Set-Content '%LOG_FILE%'"
"@
Set-Content -Path $WrapperDest -Value $WrapperContent -Encoding ASCII

Write-Host "Registering scheduled task '$TaskName' (every $IntervalSecs seconds)..."
$RepeatInterval = "PT$($IntervalSecs)S"
schtasks /Create /F /TN $TaskName /TR "`"$WrapperDest`"" /SC MINUTE /MO ([Math]::Max(1, [Math]::Floor($IntervalSecs / 60))) | Out-Null
if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to register scheduled task."
    exit 1
}

Write-Host "Installation complete!"
Write-Host "The scheduled task '$TaskName' will run every $IntervalSecs seconds."
