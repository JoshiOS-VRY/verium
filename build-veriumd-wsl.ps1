# Build veriumd.exe via WSL and install into the wallet sidecar directory.
$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$veriumRoot = (Resolve-Path $scriptDir).Path
$walletDir = Join-Path $veriumRoot "desktop\verium-app"
$builtExe = Join-Path $veriumRoot "src\veriumd.exe"

function To-WslPath([string]$WinPath) {
    $resolved = (Resolve-Path $WinPath).Path -replace '\\', '/'
    if ($resolved -match '^([A-Za-z]):(.*)$') {
        return "/mnt/$($matches[1].ToLower())$($matches[2])"
    }
    return $resolved
}

$wslVerium = To-WslPath $veriumRoot
$wslLog = "~/vericonomy-build-verium/build-veriumd.log"

Write-Host "==> Building veriumd in WSL" -ForegroundColor Cyan
Write-Host "    Windows path: $veriumRoot"
Write-Host "    WSL path:     $wslVerium"
Write-Host "    Log:          $wslLog"
Write-Host ""

$buildCmd = "cd '$wslVerium' && sed -i 's/\r$//' ./build-veriumd-wsl.sh 2>/dev/null || true && chmod +x ./build-veriumd-wsl.sh && ./build-veriumd-wsl.sh"

wsl bash -lc $buildCmd
if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed (exit $LASTEXITCODE)." -ForegroundColor Red
    wsl bash -lc "tail -50 '$wslLog' 2>/dev/null || true"
    exit $LASTEXITCODE
}

if (-not (Test-Path $builtExe)) {
    Write-Host "Build reported success but $builtExe was not found." -ForegroundColor Red
    exit 1
}

$sizeMb = [math]::Round((Get-Item $builtExe).Length / 1MB, 1)
Write-Host "==> Built veriumd.exe ($sizeMb MB)" -ForegroundColor Green

Get-Process veriumd -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

Write-Host "==> Installing into wallet sidecar directory" -ForegroundColor Cyan
Push-Location $walletDir
$env:VERIUMD_LOCAL = $builtExe
$env:VERIUMD_FORCE = "1"
npm run fetch:veriumd
$exitCode = $LASTEXITCODE
Pop-Location

if ($exitCode -ne 0) {
    exit $exitCode
}

$sidecar = Join-Path $walletDir "src-tauri\binaries\veriumd-x86_64-pc-windows-msvc.exe"
if (Test-Path $sidecar) {
    $sidecarMb = [math]::Round((Get-Item $sidecar).Length / 1MB, 1)
    Write-Host "==> Sidecar installed: $sidecar ($sidecarMb MB)" -ForegroundColor Green
    Write-Host "Restart the Verium node in the wallet (or quit and reopen) to use native pool mining." -ForegroundColor Yellow
} else {
    Write-Host "WARNING: Expected sidecar not found at $sidecar" -ForegroundColor Yellow
}
