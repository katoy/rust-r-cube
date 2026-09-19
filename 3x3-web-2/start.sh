#!/usr/bin/env bash
set -euo pipefail

# プロジェクトルートに移動
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

MODE="dev"
REBUILD_WASM=false

for arg in "$@"; do
  case "$arg" in
    --preview|-p)
      MODE="preview"
      ;;
    --offline|-o)
      MODE="offline"
      ;;
    --build|-b)
      REBUILD_WASM=true
      ;;
    --help|-h)
      echo "使い方: $0 [オプション]"
      echo ""
      echo "オプション:"
      echo "  -o, --offline   PWAキャッシュ構築後、オフラインモード（ネットワーク切断）でブラウザを起動する"
      echo "  -p, --preview   プロダクション用にビルドしてプレビューサーバーを起動する"
      echo "  -b, --build     起動前に WebAssembly (wasm) を再ビルドする"
      echo "  -h, --help      このヘルプを表示する"
      exit 0
      ;;
    *)
      echo "不明なオプション: $arg"
      echo "使用可能なオプションは -h または --help を参照してください。"
      exit 1
      ;;
  esac
done

# node_modules の存在確認
if [ ! -d "node_modules" ]; then
  echo "📦 node_modules が見つかりません。npm ci を実行しています..."
  npm ci
fi

# wasm 成果物の存在確認または再ビルド要求時
if [ "$REBUILD_WASM" = true ] || [ ! -f "pkg/cube_studio_bg.wasm" ]; then
  echo "🦀 WebAssembly (pkg/) をビルドしています..."
  npm run wasm
fi

if [ "$MODE" = "offline" ]; then
  echo "⚡ オフライン動作検証モードを起動します..."
  node scripts/launch-offline.js
elif [ "$MODE" = "preview" ]; then
  echo "🚀 プロダクションビルドを実行しています..."
  npm run build
  echo "🌐 プレビューサーバーを起動します..."
  npm run preview
else
  echo "🌐 開発サーバーを起動します (http://127.0.0.1:5173)..."
  npm run dev
fi
