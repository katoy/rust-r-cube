# `9fb1ebc` レビュー証跡

- `probe-current-head.mjs`: 一時ディレクトリ内でビルド鮮度とガードレールの偽陽性を確認し、Chromium で非表示側カメラ画像のカード表示を確認する。製品コードを変更しない。
- `probe-current-head.json`: 上記プローブの実行結果。
- `sw-test.log`: `npx playwright test tests/sw-update.spec.ts -g 'actual precache generator' --reporter=line` の失敗ログ。`ERR_MODULE_NOT_FOUND` を記録する。

プローブを再実行する場合は、別のターミナルで `npm run dev -- --port 5173 --strictPort` を起動し、リポジトリの `3x3-web-2` ディレクトリから `node docs/review-evidence/9fb1ebc/probe-current-head.mjs` を実行する。Chromium と WASM ビルド済みの `pkg/` が必要。
