# Stop and remove the WallShift Windows service.
$ErrorActionPreference = "Stop"

if (-not ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
        [Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw "Run this script as Administrator."
}

$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$exe = Join-Path $root "bin\wallshift.exe"
if (-not (Test-Path $exe)) {
    $exe = Join-Path $root "wallshift.exe"
}
if (-not (Test-Path $exe)) {
    throw "wallshift.exe not found in bin\ or next to this script"
}

& $exe uninstall
if ($LASTEXITCODE -ne 0) {
    throw "uninstall failed"
}
