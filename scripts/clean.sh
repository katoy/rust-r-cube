#!/usr/bin/env bash
set -euo pipefail

# プロジェクトのルートディレクトリに移動
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${ROOT_DIR}"

echo "== Cleaning 2x2 =="
if [ -d "2x2" ]; then
  (cd 2x2 && cargo clean && rm -f coverage.txt cube_state.txt *.profraw)
fi

echo "== Cleaning 2x2-web =="
if [ -d "2x2-web" ]; then
  (cd 2x2-web && cargo clean)
fi

echo "== Cleaning 3x3-web =="
if [ -d "3x3-web" ]; then
  # dist や一部の一時ファイルを削除 (Git管理されている coverage_report.txt や gen_cycles などは除外)
  (cd 3x3-web && cargo clean && rm -rf dist coverage_detailed.txt)
fi

echo "== Cleaning other temporary files =="
rm -f .DS_Store 2x2/.DS_Store 2x2-web/.DS_Store 3x3-web/.DS_Store

echo "== Clean completed successfully =="
