#!/usr/bin/env bash
set -euo pipefail

# Run from repository root (script directory)
cd "$(dirname "$0")"

echo "Adding rustup components (clippy, rustfmt) if missing (non-fatal)"
rustup component add clippy rustfmt || true

echo "1) cargo fmt --check"
cargo fmt --all -- --check

echo "2) cargo check --all-targets --all-features"
cargo check --all-targets --all-features

echo "3) cargo test --all-features --verbose"
cargo test --all-features --verbose

echo "4) cargo clippy --all-targets --all-features (deny warnings)"
cargo clippy --all-targets --all-features -- -D warnings

# cargo-audit (install if missing)
if ! command -v cargo-audit >/dev/null 2>&1; then
  echo "cargo-audit not found; attempting to install (this may take a while)"
  cargo install --locked cargo-audit || echo "cargo-audit install failed; skipping audit"
fi
if command -v cargo-audit >/dev/null 2>&1; then
  echo "5) cargo audit"
  cargo audit || true
fi

# cargo-llvm-cov (coverage) - optional
if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
  echo "cargo-llvm-cov not found; attempting to install (may take a while)"
  cargo install --locked cargo-llvm-cov || echo "cargo-llvm-cov install failed; skipping coverage"
fi
if command -v cargo-llvm-cov >/dev/null 2>&1; then
  echo "6) cargo llvm-cov (generate report)"
  cargo llvm-cov --all-features --workspace || true
  cargo llvm-cov report --all-features --workspace || true
fi

echo "Local CI checks finished."
