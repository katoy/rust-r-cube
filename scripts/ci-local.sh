#!/usr/bin/env bash
set -euo pipefail

# Local CI checks: format, clippy, fetch, build, test
# Usage: ./scripts/ci-local.sh

# プロジェクトのルートディレクトリに移動
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${ROOT_DIR}"

echo "== Local CI checks: fmt, clippy, build, test =="

# Use sccache if available
if command -v sccache >/dev/null 2>&1; then
  export RUSTC_WRAPPER="$(command -v sccache)"
  echo "Using sccache: $RUSTC_WRAPPER"
else
  echo "sccache not found: proceeding without compiler cache"
fi

PROJECTS=("2x2" "2x2-web" "3x3-web")

for proj in "${PROJECTS[@]}"; do
  echo "========================================"
  echo "Checking project: ${proj}"
  echo "========================================"
  
  (
    cd "${proj}"
    
    echo "-- cargo fmt --check --"
    cargo fmt --all -- --check
    
    echo "-- cargo clippy --"
    cargo clippy --all-targets --all-features -- -D warnings
    
    echo "-- cargo fetch --"
    cargo fetch --locked || true
    
    echo "-- cargo build --"
    cargo build --verbose
    
    echo "-- cargo test --"
    cargo test --verbose
  )
done

echo "== All checks completed successfully =="
