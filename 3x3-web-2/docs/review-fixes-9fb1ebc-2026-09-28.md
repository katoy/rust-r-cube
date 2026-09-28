# レビュー指摘事項の修正および再発防止対応レポート（9fb1ebc / 2026-09-28）

## 概要

[`docs/code-review-9fb1ebc-2026-09-28.md`](code-review-9fb1ebc-2026-09-28.md) で報告された全指摘事項（F1〜F4）への修正対応、品質ゲートの整備、および回帰テストによる検証を完了しました。

---

## 指摘事項別の対応詳細

### F1 (P1): 新しい生成スクリプト依存が隔離テストへ渡されず、17 件が失敗する

- **問題:** [`generate-sw-precache.js`](../scripts/generate-sw-precache.js) が新たに `./build-manifest.js` をインポートする構成に変更されたが、[`tests/sw-update.spec.ts`](../tests/sw-update.spec.ts) の `build` フィクスチャは生成スクリプトのみを一時ディレクトリへコピーしていたため、実行時に `ERR_MODULE_NOT_FOUND` が発生し 17 件のテストが停止していた。
- **原因:** 一時ディレクトリ内に依存モジュール `build-manifest.js` および ESM 解決のための `package.json`（`"type": "module"`）が配置されていなかった。
- **修正内容:**
  - [`tests/sw-update.spec.ts`](../tests/sw-update.spec.ts): `build` フィクスチャ内で `package.json`（`{"type": "module"}`）を書き出し、`scripts/build-manifest.js` を一時スクリプトディレクトリへコピーするように改修。
  - 実スクリプトを `.js` のまま実行し、依存解決を正しく行えるように整備。
- **検証結果:**
  - `tests/sw-update.spec.ts` の全 17 テストが成功。代表テストおよび全自動テストでアセット更新検知・キャッシュ比較まで到達することを確認。

### F2 (P2): 入力マニフェストに `tsconfig.json` とマニフェスト生成器自身がない

- **問題:** [`scripts/build-manifest.js`](../scripts/build-manifest.js) の `collectInputFiles` が `tsconfig.json` と `scripts/build-manifest.js` を収集対象に含めていなかった。そのため、型チェック設定やマニフェスト生成ロジックを変更しても `checkInputsFreshness` が `fresh: true` を返し、オフライン検証で古い `dist` を再利用してしまうリスクがあった。
- **原因:** `collectInputFiles` の `configFiles` 配列に対象ファイルが含まれていなかった。
- **修正内容:**
  - [`scripts/build-manifest.js`](../scripts/build-manifest.js): `configFiles` 配列に `"tsconfig.json"` と `"scripts/build-manifest.js"` を追加。
- **検証結果:**
  - 両ファイルの変更時に `checkInputsFreshness` が確実に `fresh: false`（`ファイルが変更されました: tsconfig.json` 等）を返すことを確認。
  - 再現プローブでも変更検知が正常に機能することを確認。

### F3 (P3): 非表示側へ画像を読み込むと、画像カードが「未選択」のまま残る

- **問題:** カメラ画面で画像Aのデコード保留中に利用者がビューBへ切り替えた際、画像Aの読み込みが完了しても、`currentView !== view` の経路でカード表示やステータス文言の更新処理が呼ばれず、画像Aカードが「未選択（クリックまたはドロップ）」のまま残っていた。
- **原因:** [`web/camera.ts`](../web/camera.ts) の `onImageLoaded` で、非表示側の完了時にビュー復帰を防ぎつつも、カードUI（`camera-status-*`, `camera-drop-*`）を更新する処理が欠落していたため。
- **修正内容:**
  - [`web/camera.ts`](../web/camera.ts): カードのファイル有無クラスおよびステータス文言を更新する `updateCardStatus(view: "A" | "B")` を新設。
  - `onImageLoaded` 内で、画像オブジェクトの保存直後に `this.updateCardStatus(view)` を呼び出すよう改修。
  - 現在の表示ビュー（B）や選択中の6点は一切変更せず、非表示側のカード状態のみを即座に「読込完了 (W×H)」へ更新。
- **検証結果:**
  - 実ブラウザプローブおよび自動テストにおいて、画像Aが非表示側で完了した直後にカードAが `has-file` クラス付きの「読込完了 (20×20)」になり、ビューBの選択状態が破壊されないことを確認。

### F4 (P3): パス正規化の静的ガードレールが現在の実装ファイルを見ていない

- **問題:** パス正規化処理が [`web/storage-key.ts`](../web/storage-key.ts) に集約されたが、[`scripts/check-review-guardrails.js`](../scripts/check-review-guardrails.js) の `checkPathNormalization` は `web/main.ts` のみを走査していたため、正規化実装を壊しても静的検査を通過（exit code 0）してしまっていた。
- **原因:** 静的ガードレールチェッカーの走査対象ファイル一覧に `web/storage-key.ts` が含まれていなかった。
- **修正内容:**
  - [`scripts/check-review-guardrails.js`](../scripts/check-review-guardrails.js): `checkPathNormalization` の走査対象に `web/storage-key.ts` を追加。
- **検証結果:**
  - `web/storage-key.ts` に `index.html` の考慮がない実装を配置した隔離テストで、静的チェッカーが exit code 1 とエラーメッセージ `[ERROR][F4-URL-NORMALIZATION]` を出力することを確認。
  - 現行コードでの検査合格、および別名URL（`/` と `/index.html`、`/nested/cube/` と `/nested/cube/index.html`）でのキー同一性テストをパス。

---

## 総合テスト・検証結果

1. **Rust 単体テスト・静的検査:**
   - `cargo fmt --check`: 成功
   - `cargo clippy --all-targets -- -D warnings`: 成功（警告 0 件）
   - `cargo test --release`: **141 passed / 1 ignored**

2. **静的品質ゲート・コードスタイル:**
   - `npm run check:guardrails`: **PASS**（全ルール合格）
   - `npm run format:check`: **PASS**（全ファイル Prettier 準拠）
   - `npm run typecheck`: **PASS**（型エラー 0 件）

3. **ビルドおよびオフライン検証:**
   - `npm run build`: 成功（19 アセット、WASM 最適化、プリキャッシュ注入完了）
   - `node scripts/launch-offline.js --headless`: **PASS**（Service Worker 19 アセットキャッシュおよび完全オフライン起動を確認）

4. **Playwright 自動テスト:**
   - 新規回帰テストスイート [`tests/code-review-9fb1ebc-regression.spec.ts`](../tests/code-review-9fb1ebc-regression.spec.ts): **5 passed / 0 failed**
   - 既存更新テスト [`tests/sw-update.spec.ts`](../tests/sw-update.spec.ts): **17 passed / 0 failed**（17件の失敗を解消）
   - 全自動テストスイート (`npm test`): **194 passed / 1 skipped / 0 failed**
