# 0bd0d8c レビューの再現用コードと実行証跡

対象: `0bd0d8ccd27628b019834c13a8592abee3b7d2ad`。以下は `3x3-web-2` を作業ディレクトリにして実行する。

## 前提

プロジェクトの依存関係、Rust、wasm-pack、Playwright Chromiumが利用できること。

```sh
npm run build
npm run generate-test-images
cargo test --release
```

## 追加のネイティブ検証

本体ファイルを変更せず、ビルド済みのクレートへリンクする。

```sh
rustc --edition=2021 -O docs/review-evidence/0bd0d8c/probe_solvers.rs \
  --extern cube_studio=target/release/deps/libcube_studio.rlib \
  -L dependency=target/release/deps \
  -o /tmp/cube-review-0bd0d8c-solvers
/tmp/cube-review-0bd0d8c-solvers
```

- CFOP: seed 1〜1000の25手スクランブルを解き、適用後の状態を検証する。各探索の予算は1,000ms。
- センター補正: 4,096通りを列挙し、偶パリティの2,048通りはピースとセンターの両方が完成し、奇パリティの2,048通りは拒否されることをassertする。
- 記録: [solvers.log](solvers.log)。

## 追加のブラウザ検証

```sh
node docs/review-evidence/0bd0d8c/probe_browser.mjs
```

既定では専用のViteサーバーを `127.0.0.1:5183` に起動し、最後に終了する。ポート5183は空けておく。別の既存Viteサーバーを使う場合は `REVIEW_BASE_URL` を指定できる。本番ビルドのpreviewではなく、開発サーバーを使用する。

各ケースは別の一時的なChromiumコンテキストで実行する。通常利用中のブラウザプロファイルは使用せず、カメラはChromiumの偽デバイスを用いる。

出力 [browser.jsonl](browser.jsonl) の各 `name` は次の検証に対応する。

| name | 内容 |
| --- | --- |
| `playback_persistence` | R02: 解法を完了して閉じた後、表示・保存・復元状態を比較 |
| `unrelated_service_worker_unregistered` | R01: 別スコープの実SW登録が消えることを確認 |
| `undo_during_animation` | 低優先度観察: 即時undoでもアニメーションが残る。ストア状態破損とは判定しない |
| `camera_sticker_keyboard_focus` | R04: 認識済み画像で、セル補正後のフォーカスと次のTab移動先を確認 |
| `camera_palette_arrow_navigation` | 低優先度観察: ArrowRightでradio選択が動かない |
| `old_capture_overwrites_new_file` | R03: 実toBlobの配送だけを保留し、新しいファイル入力が古い撮影に上書きされることを確認 |

このプローブは観測結果をJSONで出力する。終了コード0はプローブが完走したことを意味し、アプリの不具合がないという意味ではない。修正後はレポートに記載した期待動作をassertする回帰テストへ変換する。

## 通常チェックの記録

- [rust-tests.log](rust-tests.log): `cargo test --release`
- [rust-fmt.log](rust-fmt.log): `cargo fmt --check`（exit 1）
- [clippy.log](clippy.log): `cargo clippy --all-targets -- -D warnings`
- [web-format.log](web-format.log): `npm run format:check`
- [build.log](build.log): `npm run build`
- [e2e.log](e2e.log): `npm test -- --output=/tmp/cube-review-0bd0d8c-e2e`
- [npm-audit.json](npm-audit.json): `npm audit --json`

Rustfmt以外の最終実行は成功。Rustのignored 1件とPlaywrightのskipped 1件は、長時間のKociemba/Workerベンチマーク用テストであり、この記録では実行していない。ログの実行時間・ポート・絶対パスは当日の環境に依存する。
