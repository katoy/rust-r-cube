# `ba8fcfa` レビューの運用系再現記録

実施日: 2026-09-28。作業ディレクトリ: `3x3-web-2/`。

## 成果物鮮度

1. `npm run build` 成功後、`touch vite.config.ts` で設定ファイルの更新時刻のみを進めた（内容は変更なし）。
2. `stat` の結果は `vite.config.ts: 2026-09-28 05:57:59`、`dist/index.html: 2026-09-28 04:55:32`。
3. `npm run offline -- --headless` は `⚡ 既存の最新ビルド成果物を使用します (4:55:32 AM)` と出力し、ビルドなしでオフライン起動成功。設定ファイルは鮮度判定対象外である。

## SIGINT

ポート待受が権限不足で失敗した後の起動待ち中に `Ctrl+C` を送ると、次の例外が出た。ポート権限の問題とは別に、シグナルの文字列が `process.exit` へ渡ることを示す。

```text
🧹 終了処理を実行中...
TypeError [ERR_INVALID_ARG_TYPE]: The "code" argument must be of type number. Received type string ('SIGINT')
    at process.cleanup (scripts/launch-offline.js:193:13)
```

## ビルドとテスト

- `cargo test --release`: 141 passed、1 ignored。
- `npm run build`: 成功。`dist/assets/cube_studio_bg-*.wasm` 6,289.62 kB、`dist/assets/three-*.js` 575.91 kB。
- `npm test`: 179 passed、1 skipped。
- `npm audit --json`: vulnerabilities total 0。
- 初回の `npm run check` はサンドボックス内での `wasm-bindgen` インストールに `Operation not permitted` が出てビルドで停止。権限を上げた `npm run build` と、その後の `npm test` は成功した。
