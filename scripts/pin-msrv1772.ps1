# 在 Rust 1.77.2 下反复 build，按 MSRV 报错自动降级（仅处理 "requires rustc X" 类错误）
$ErrorActionPreference = "Continue"
Set-Location "$PSScriptRoot\..\src-tauri"
$cargo = "$env:USERPROFILE\.cargo\bin\cargo.exe"

function Get-MsrvFailure([string]$Output) {
    if ($Output -match 'package ``([^`]+) v([^`]+)`` cannot be built because it requires rustc ([0-9.]+)') {
        return @{ Package = $Matches[1]; Version = $Matches[2]; Need = $Matches[3] }
    }
    if ($Output -match 'package `([^`]+) v([^`]+)` cannot be built because it requires rustc ([0-9.]+)') {
        return @{ Package = $Matches[1]; Version = $Matches[2]; Need = $Matches[3] }
    }
    return $null
}

function Try-Downgrade([string]$Package, [string]$Current) {
    $pinsFile = "$PSScriptRoot\msrv1772-overrides.txt"
    if (Test-Path $pinsFile) {
        Get-Content $pinsFile | ForEach-Object {
            $line = ($_ -replace '#.*','').Trim()
            if (-not $line) { return }
            $parts = $line -split '\s+', 2
            if ($parts[0] -eq $Package) { return $parts[1] }
        }
    }
    return $null
}

$max = 30
for ($i = 1; $i -le $max; $i++) {
    Write-Host "=== build attempt $i ==="
    $out = & $cargo +1.77.2 build --locked --release 2>&1 | Out-String
    if ($LASTEXITCODE -eq 0) {
        Write-Host "BUILD SUCCESS"
        exit 0
    }
    $fail = Get-MsrvFailure $out
    if (-not $fail) {
        Write-Host $out
        exit 1
    }
    Write-Host "MSRV: $($fail.Package) $($fail.Version) needs $($fail.Need)"
    $target = Try-Downgrade $fail.Package $fail.Version
    if (-not $target) {
        Write-Host "No override for $($fail.Package) in msrv1772-overrides.txt"
        Write-Host ($out -split "`n" | Select-Object -Last 6) -join "`n"
        exit 1
    }
    Write-Host "  -> pinning $($fail.Package) = $target"
    & $cargo +1.77.2 update -p $fail.Package --precise $target 2>&1 | Out-Null
}
Write-Host "Exceeded max attempts"
exit 1
