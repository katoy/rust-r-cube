#!/bin/bash

# CI スクリプト: lint、テスト、カバレッジ計測を実行する
# ローカル開発環境でも GitHub Actions でも使用可能

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

echo "=========================================="
echo "CI チェック開始"
echo "=========================================="

# 1. Lint チェック
echo ""
echo "📝 rustfmt チェック..."
cargo fmt --check

echo "📝 clippy チェック..."
cargo clippy --all-targets --all-features -- -D warnings

# 2. ユニットテスト実行
echo ""
echo "🧪 ユニットテスト実行..."
cargo test --lib --no-fail-fast

# 3. 統合テスト実行
echo ""
echo "🧪 統合テスト実行..."
cargo test --test '*' --no-fail-fast

# 4. カバレッジ計測（GUI系除外、100%要求）
echo ""
echo "📊 カバレッジ計測（GUI系除外）..."

# tarpaulin をインストール
if ! cargo install --list | grep -q "cargo-tarpaulin"; then
    echo "⬇️  tarpaulin をインストール中..."
    cargo install cargo-tarpaulin
fi

# GUI ファイルを除外してカバレッジ計測
# - exclude-files で gui/ 配下のファイルを除外
# - desktop feature を無効化してテスト実行
cargo tarpaulin \
    --out Html \
    --output-dir coverage \
    --exclude-files 'src/gui/*' 'src/bin/*' \
    --timeout 300 \
    --skip-clean \
    --fail-under 100 \
    -- --test-threads=1

echo ""
echo "✅ カバレッジ計測完了"
echo "   詳細: coverage/index.html を参照"

# 5. ドキュメント生成
echo ""
echo "📚 ドキュメント生成..."
cargo doc --no-deps --all-features 2>&1 | grep -v "^warning:" | head -20 || true

echo ""
echo "=========================================="
echo "✅ CI チェック完了"
echo "=========================================="
