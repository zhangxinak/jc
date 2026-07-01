#!/usr/bin/env bash
# Rust 1.77.2 / Win7：用 stable 生成 lockfile 并降级 edition2024 依赖，再用 1.77.2 fetch
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/src-tauri"

if [ -f Cargo.lock ]; then
  echo "=== 使用已有 Cargo.lock ==="
else
  echo "=== [stable] 生成 Cargo.lock ==="
  rustup toolchain install stable --profile minimal 2>/dev/null || true
  cargo +stable generate-lockfile
  pin() { echo "  -> $1 = $2"; cargo +stable update -p "$1" --precise "$2"; }
  pin time 0.3.41
  pin ignore 0.4.23
  pin globset 0.4.16
  pin home 0.5.11
  cargo +stable update -p 'indexmap@2' --precise 2.13.0
  cargo +stable fetch --locked
fi

echo "=== [1.77.2] cargo fetch --locked ==="
cargo fetch --locked
grep -E '^name = "(time|ignore|globset|indexmap|home|time-core)"' -A1 Cargo.lock || true
