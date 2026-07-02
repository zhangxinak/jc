#!/usr/bin/env bash
# Rust 1.77.2 / Win7：用 stable 生成 lockfile 并降级 edition2024 依赖，再用 1.77.2 校验
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/src-tauri"
PINS="$ROOT/scripts/edition2021-pins.txt"

apply_pins() {
  rustup toolchain install stable --profile minimal 2>/dev/null || true
  while read -r line || [ -n "$line" ]; do
    line="${line%%#*}"
    line="$(echo "$line" | xargs)"
    [ -z "$line" ] && continue
    pkg="${line%% *}"
    ver="${line#* }"
    echo "  -> $pkg = $ver"
    cargo +stable update -p "$pkg" --precise "$ver"
  done < "$PINS"
  cargo +stable fetch --locked
}

validate_lock() {
  if grep -E 'name = "getrandom"' Cargo.lock -A2 | grep -qE 'version = "0\.4\.'; then
    echo "ERROR: Cargo.lock 仍含 getrandom 0.4.x（edition2024）"
    exit 1
  fi
  if grep -E 'name = "wit-bindgen"' Cargo.lock -A2 | grep -qE 'version = "0\.(5[0-9]|[6-9][0-9])'; then
    echo "ERROR: Cargo.lock 仍含 wit-bindgen 0.50+（edition2024）"
    exit 1
  fi
  if grep -E 'name = "idna_adapter"' Cargo.lock -A2 | grep -qE 'version = "1\.2\.[2-9]'; then
    echo "ERROR: Cargo.lock 仍含 idna_adapter 1.2.2+（edition2024 或 ICU 2.x 需 Rust 1.82+）"
    exit 1
  fi
  if grep -E 'name = "icu_normalizer_data"' Cargo.lock -A2 | grep -qE 'version = "2\.'; then
    echo "ERROR: Cargo.lock 仍含 icu 2.x 数据 crate（需 Rust 1.82+），请 pin idna_adapter=1.2.0"
    exit 1
  fi
  if grep -E 'name = "plist"' Cargo.lock -A2 | grep -qE 'version = "1\.8\.'; then
    echo "ERROR: Cargo.lock 仍含 plist 1.8+（需 Rust 1.81+），请 pin plist=1.7.1"
    exit 1
  fi
}

if [ -f Cargo.lock ]; then
  echo "=== 使用已提交的 Cargo.lock ==="
  validate_lock
else
  echo "=== [stable] 生成 Cargo.lock 并 pin edition2021 依赖 ==="
  cargo +stable generate-lockfile
  echo "=== 精确锁定（见 scripts/edition2021-pins.txt）==="
  apply_pins
  validate_lock
fi

echo "=== [1.77.2] cargo fetch --locked ==="
cargo fetch --locked

echo "=== 关键依赖版本 ==="
grep -E '^name = "(time|ignore|globset|indexmap|home|getrandom|uuid|tempfile|idna_adapter|wit-bindgen)"' -A1 Cargo.lock || true
