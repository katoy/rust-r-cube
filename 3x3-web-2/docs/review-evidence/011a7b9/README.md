# `011a7b9` レビュー再現証跡

- [`probe-ui.mjs`](probe-ui.mjs): Chromium で画像差し替え後の旧読取結果、プリセットの逆順失敗、タイムラインの読み上げ文を再現し、[`probe-ui.json`](probe-ui.json)へ結果を保存する。
- [`probe-manifest.mjs`](probe-manifest.mjs): 一時ディレクトリの破損マニフェストを検査し、[`probe-manifest.json`](probe-manifest.json)へ結果を保存する。製品の `dist` は変更しない。

再実行には依存関係・WASM・テスト画像を用意し、別ターミナルで `npm run dev -- --port 5173 --strictPort` を起動する。続けてプロジェクトルートから次を実行する。

```sh
node docs/review-evidence/011a7b9/probe-ui.mjs
node docs/review-evidence/011a7b9/probe-manifest.mjs
```

カメラ画像は `npm run generate-test-images` で生成できる。ブラウザの一時 `blob:` URL は再実行ごとに変わる。
