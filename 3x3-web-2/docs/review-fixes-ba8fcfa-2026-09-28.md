# レビュー指摘事項の修正および再発防止対応レポート（ba8fcfa / 2026-09-28）

## 概要

[`docs/code-review-ba8fcfa-2026-09-28.md`](code-review-ba8fcfa-2026-09-28.md) で報告された全指摘事項（F1〜F5）への修正対応、品質ゲート整備、および自動テストによる検証を完了しました。

---

## 指摘事項別の対応詳細

### F1 (P2): 解法表示を閉じると、再生した局面を Undo できなくなる

- **問題:** 解法を再生（シーク）した後に解法表示を閉じると、`setSolution(undefined)` によって開始時のスナップショット `baseSnapshot` が破棄され、シーク後の局面に対する Undo 履歴が消失して回転前の局面に戻れなくなっていた。
- **修正内容:**
  - [`web/cube-store.ts`](../web/cube-store.ts): `setSolution(solution)` の冒頭で、`this.baseSnapshot` が存在し現在の局面と異なる場合、シークによる局面変更を1つの確定履歴として `this.history.push(this.baseSnapshot)` を実行し、`this.future = []` にリセットするよう改修。
  - これにより、解法を閉じた後も Undo ボタンが有効となり、解法適用前の局面へ正確に戻すことが可能。さらに Redo でシーク後局面へ進むことも可能となった。
- **追加テスト:**
  - [`tests/cube-store.spec.ts`](../tests/cube-store.spec.ts): 単体テスト `closing solution after seek preserves undo history to base snapshot and supports redo` を追加。
  - [`tests/playback-controls.spec.ts`](../tests/playback-controls.spec.ts): E2E テスト `closing solution after seek allows undoing back to pre-solution state and redoing to solved`（回転 → 解法最終手 → 閉じる → Undo/Redo）を追加。

### F2 (P2): Pages デプロイは CI のテスト結果を待たずに公開できる

- **問題:** GitHub Pages デプロイワークフロー（`deploy-pages.yml`）が独立して起動し、テストワークフロー（`3x3-web-2.yml`）の成功を待たずに `npm run build` だけで公開されていた。
- **修正内容:**
  - [`.github/workflows/deploy-pages.yml`](../../.github/workflows/deploy-pages.yml): `test-3x3-web-2` ジョブを新規追加し、Rust のフォーマット・clippy・単体テスト、ガードレール静的検査、Web フォーマット検査、ビルド、Playwright テスト全件を実行。
  - `build` ジョブに `needs: [test-3x3-web-2]` を設定し、テストがすべて合格しなければビルド・デプロイが実行されないよう公開ゲートを配備。

### F3 (P2): `check:guardrails` は検査違反を警告しても成功終了する

- **問題:** `scripts/check-review-guardrails.js` で違反検出時も `reportWarning` のみを呼び出し、末尾で `process.exit(0)` していたため、ゲートとして機能していなかった。
- **修正内容:**
  - [`scripts/check-review-guardrails.js`](../scripts/check-review-guardrails.js): 各検査ルール（F1-RELEASE-VERIFICATION, F3-FILE-SIZE-FIRST, F4-URL-NORMALIZATION, F5-SW-QUERY-CACHE）で違反検出時に `reportError`（exit 1）を呼ぶよう改修。
  - 助言モードとしての `--warn-only` オプションおよび検査対象ディレクトリの引数指定をサポート。
- **追加テスト:**
  - [`tests/scripts-regression.spec.ts`](../tests/scripts-regression.spec.ts): 一時フィクスチャに違反コードを配置して実行し、exit code 1 で失敗すること、および `--warn-only` で exit code 0 となることを検証するテストを追加。

### F4 (P3): オフライン検証は設定変更後も古い `dist` を「最新」と判定する

- **問題:** `scripts/launch-offline.js` の鮮度判定 `ensureBuild` が `src/`, `web/`, `public/` の mtime しか比較しておらず、`vite.config.ts` などの設定変更が再ビルドの契機になっていなかった。
- **修正内容:**
  - [`scripts/launch-offline.js`](../scripts/launch-offline.js): `vite.config.ts`, `Cargo.toml`, `Cargo.lock`, `build.rs`, `package.json`, `package-lock.json`, `scripts/generate-sw-precache.js`, `index.html` などのルート設定ファイル群の mtime を比較対象に追加。
- **検証:**
  - `touch vite.config.ts` を実行した後に `node scripts/launch-offline.js --headless` を実行し、自動的に再ビルドが発火して最新成果物でオフライン起動することを確認。

### F5 (P3): オフライン検証を Ctrl+C で止めると終了処理が例外になる

- **問題:** `process.on("SIGINT", cleanup)` でシグナル名（文字列）が渡り、`process.exit("SIGINT")` で `ERR_INVALID_ARG_TYPE` 例外が発生していた。
- **修正内容:**
  - [`scripts/launch-offline.js`](../scripts/launch-offline.js): `process.on("SIGINT", () => cleanup(130))`、`process.on("SIGTERM", () => cleanup(143))` に修正。さらに `cleanup` 関数内で `typeof exitCode === "number"` による型ガードを実装。
- **追加テスト:**
  - [`tests/scripts-regression.spec.ts`](../tests/scripts-regression.spec.ts): ヘッドレス起動中のプロセスに SIGINT を送信し、例外なく exit code 130 または 0 でクリーンに終了することを検証するテストを追加。

---

## 総合テスト・検証結果

1. **Rust 単体テスト・品質検査:**
   - `cargo fmt --check`: 成功
   - `cargo clippy --all-targets -- -D warnings`: 成功（警告 0 件）
   - `cargo test --release`: 141 passed, 1 ignored（スーパーフリップ 1000 回探索のみ ignored 設定）
2. **静的ガードレール検査:**
   - `npm run check:guardrails`: 成功（エラー 0 件）
3. **Web コードフォーマット検査:**
   - `npm run format:check`: 成功
4. **本番ビルド & SW プリキャッシュ生成:**
   - `npm run build`: 成功（WASM 生成、Vite バンドル、SW プリキャッシュ注入完了）
5. **E2E / Playwright テスト:**
   - `npx playwright test`: **183 passed, 1 skipped**（全件合格）
6. **オフライン起動検証:**
   - `node scripts/launch-offline.js --headless`: ヘッドレスオフライン起動およびキャッシュ検証成功
