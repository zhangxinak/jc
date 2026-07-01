#!/usr/bin/env bash
# Rust 1.77.2 / Win7：锁定 edition2021 依赖，避免 cargo 解析到 edition2024 新版本
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/src-tauri"

echo "=== 清理可能含 edition2024 manifest 的缓存 ==="
shopt -s nullglob
for dir in "$HOME/.cargo/registry/src/"index.crates.io-*; do
  rm -rf "$dir"/time-0.3.5* \
         "$dir"/time-core-0.1.[5-9]* \
         "$dir"/ignore-0.4.2[4-9]* \
         "$dir"/ignore-0.4.3* \
         "$dir"/globset-0.4.2[4-9]* \
         "$dir"/globset-0.4.3* \
         "$dir"/indexmap-2.1[4-9]* \
         2>/dev/null || true
done

echo "=== 生成 Cargo.lock ==="
cargo generate-lockfile

echo "=== 精确锁定已知问题依赖 ==="
PRECISE_PINS=(
  "time 0.3.41"
  "ignore 0.4.23"
  "globset 0.4.16"
)
# indexmap =2.13.0 已在 Cargo.toml 直接锁定，勿 cargo update -p indexmap（1.x/2.x 并存会歧义）

for pin in "${PRECISE_PINS[@]}"; do
  set -- $pin
  echo "  -> $1 = $2"
  cargo update -p "$1" --precise "$2"
done

cargo fetch --locked

echo "=== 锁定结果 ==="
grep -E '^name = "(time|ignore|globset|indexmap|time-core|time-macros)"' -A1 Cargo.lock || true
