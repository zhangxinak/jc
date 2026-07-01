# Rust 1.77.2 / Win7：在 Windows 上生成并锁定 Cargo.lock
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Tauri = Join-Path $Root "src-tauri"
$PinsFile = Join-Path $Root "scripts\edition2021-pins.txt"
$Cargo = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
$Rustup = Join-Path $env:USERPROFILE ".cargo\bin\rustup.exe"

& $Rustup toolchain install stable --profile minimal 2>$null
& $Rustup toolchain install 1.77.2 --profile minimal 2>$null

Push-Location $Tauri
try {
    if (-not (Test-Path "Cargo.lock")) {
        Write-Host "=== [stable] generate-lockfile ==="
        & $Cargo +stable generate-lockfile
    } else {
        Write-Host "=== 使用已有 Cargo.lock，重新 pin ==="
    }

    Write-Host "=== 精确锁定 edition2021 依赖 ==="
    Get-Content $PinsFile | ForEach-Object {
        $line = ($_ -replace '#.*', '').Trim()
        if (-not $line) { return }
        $parts = $line -split '\s+', 2
        $pkg = $parts[0]
        $ver = $parts[1]
        Write-Host "  -> $pkg = $ver"
        & $Cargo +stable update -p $pkg --precise $ver
    }

    & $Cargo +stable fetch --locked

    $lock = Get-Content Cargo.lock -Raw
    if ($lock -match 'name = "getrandom"[\s\S]*?version = "0\.4\.') {
        Write-Error "Cargo.lock 仍含 getrandom 0.4.x，请检查 scripts/edition2021-pins.txt"
    }

    Write-Host "=== [1.77.2] cargo fetch --locked ==="
    & $Cargo fetch --locked
    Write-Host "=== 完成，请提交 src-tauri/Cargo.lock ==="
} finally {
    Pop-Location
}
