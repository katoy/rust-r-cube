#!/usr/bin/env bash
set -euo pipefail

# Local CI checks: format, clippy, fetch, build, test
# Usage: ./scripts/ci-local.sh

echo "== Local CI checks: fmt, clippy, build, test =="

# Use sccache if available
if command -v sccache >/dev/null 2>&1; then
  export RUSTC_WRAPPER="$(command -v sccache)"
  echo "Using sccache: $RUSTC_WRAPPER"
else
  echo "sccache not found: proceeding without compiler cache"
fi

echo "-- cargo fmt --check --"
cargo fmt --all -- --check

echo "-- cargo clippy --"
cargo clippy --all-targets --all-features -- -D warnings

echo "-- cargo fetch --"
cargo fetch --locked || true

echo "-- cargo build --"
cargo build --workspace --verbose

echo "-- cargo test --"
cargo test --workspace --verbose

echo "== All checks completed successfully =="
