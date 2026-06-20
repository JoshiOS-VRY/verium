# Build + bundle the Qt desktop wallet on Windows (PowerShell).
# Uses the Cargo-driven build, then windeployqt to gather Qt DLLs + QML, then
# NSIS for the installer.
$ErrorActionPreference = "Stop"

$QtPrefix = $env:QT_PREFIX
if (-not $QtPrefix) { throw "Set QT_PREFIX to your Qt 6 install (e.g. C:\Qt\6.11.1\msvc2022_64)" }
$env:PATH = "$QtPrefix\bin;$env:PATH"
$env:CMAKE_PREFIX_PATH = "$QtPrefix;$env:CMAKE_PREFIX_PATH"

$Root = Split-Path -Parent $PSScriptRoot
$Dist = Join-Path $Root "dist"
$BinName = "verium-qt.exe"

Write-Host "==> Building release binary"
Push-Location $Root
cargo build --release
Pop-Location

New-Item -ItemType Directory -Force -Path $Dist | Out-Null
Copy-Item (Join-Path $Root "target\release\$BinName") $Dist -Force

Write-Host "==> Running windeployqt"
& "$QtPrefix\bin\windeployqt.exe" --qmldir (Join-Path $Root "qml") (Join-Path $Dist $BinName)

# Reuse the verium-app daemon sidecars.
$Sidecars = Join-Path $Root "..\verium-app\src-tauri\binaries"
if (Test-Path $Sidecars) { Copy-Item "$Sidecars\*" $Dist -Recurse -Force }

Write-Host "==> Building NSIS installer (requires makensis on PATH)"
if (Get-Command makensis -ErrorAction SilentlyContinue) {
    makensis (Join-Path $PSScriptRoot "installer.nsi")
} else {
    Write-Host "    (makensis not found — skipping installer step)"
}

Write-Host "==> Done: $Dist"
