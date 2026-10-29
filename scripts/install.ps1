# Install WallShift next to this script: bin\wallshift.exe and config\config.toml
$ErrorActionPreference = "Stop"

if (-not ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
        [Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw "Run this script as Administrator."
}

$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$exeSrc = Join-Path $root "wallshift.exe"
if (-not (Test-Path $exeSrc)) {
    $exeSrc = Join-Path $root "bin\wallshift.exe"
}
if (-not (Test-Path $exeSrc)) {
    throw "wallshift.exe not found next to install.ps1 or in bin\"
}

$binDir = Join-Path $root "bin"
$configDir = Join-Path $root "config"
New-Item -ItemType Directory -Force -Path $binDir | Out-Null
New-Item -ItemType Directory -Force -Path $configDir | Out-Null
$exeDst = Join-Path $binDir "wallshift.exe"

$svc = Get-Service -Name WallShift -ErrorAction SilentlyContinue
if ($svc -and $svc.Status -ne "Stopped") {
    Stop-Service WallShift -Force
    Start-Sleep -Seconds 2
}

Copy-Item -Force $exeSrc $exeDst

$configSrc = Join-Path $root "config.example.toml"
$configDst = Join-Path $configDir "config.toml"
if (-not (Test-Path $configDst)) {
    if (Test-Path $configSrc) {
        Copy-Item $configSrc $configDst
    }
}

& $exeDst install
if ($LASTEXITCODE -ne 0) {
    throw "service install failed"
}
Start-Service WallShift
Get-Service WallShift
Write-Host "Installed to $exeDst"
Write-Host "Config: $configDst"
