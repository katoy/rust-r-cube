# 全体コードレビュー — `ba8fcfa`（2026-09-28）

## 対象と判定

対象は `fix/superflip-preset` の `ba8fcfaf1fc0e34002bc0356211dd740b2b98afd`。作業ツリーがクリーンな状態から、`src/` の局面表現・4種の探索・WASM 境界、`web/` の入力・状態管理・再生・カメラ・3D 描画、Service Worker、ビルド・オフライン検証スクリプト、主要テストと GitHub Actions を確認した。以前のレビューの「修正済み」という記述は、現行コードに照らして再判定した。

**判定: 要修正。** 利用者の Undo 経路に再現可能な欠陥が1件ある。さらに、デプロイと再発防止チェックがテスト失敗・検査警告を止められない。優先度は P2（次の公開前に対処を推奨）、P3（運用・検証品質の改善）とする。

## 指摘（影響の大きい順）

### F1 — P2: 解法表示を閉じると、再生した局面を Undo できなくなる

- **再現:** `R` を回す → 解法を求める → 「最後の手順へ」 → 解法を閉じる。閉じる前は完成局面で `canUndo=true`、Undo ボタン有効。閉じた後も局面は完成のままだが `canUndo=false`、ボタン無効となり、`R` を回した局面に戻れない。実ブラウザで確認し、[再現スクリプト](review-evidence/ba8fcfa/probe-close-undo.mjs)と[出力](review-evidence/ba8fcfa/probe-close-undo.json)を保存した。
- **原因:** [`CubeStore.setSolution`](../web/cube-store.ts#L166) は `undefined` を受けると `baseSnapshot` を捨てる。シークは [`updateAfterSeek`](../web/cube-store.ts#L178) で現在局面だけを変え、履歴には積まない。閉じる UI は [`main.ts`](../web/main.ts#L682) から `setSolution(undefined)` を呼ぶため、[`canUndo`](../web/cube-store.ts#L73) はシーク前の局面を参照できない。以前の F2 修正は**解法が開いている間**の Undo を扱うが、この遷移は対象外だった。
- **影響:** 解法を閉じる通常操作だけで取り消し可能な履歴が失われる。閉じた後に保存・再読込すると、元のスクランブルはさらに復元できなくなる。
- **修正案:** 解法を閉じる際、現在局面と `baseSnapshot` が異なれば、シークによる変更を一つの履歴遷移として確定するか、開始局面に戻してから閉じる。どちらを選ぶか UI 文言と揃え、`回転 → 解法最終手 → 閉じる → Undo/Redo` を実画面テストに追加する。

### F2 — P2: Pages デプロイは CI のテスト結果を待たずに公開できる

- **根拠:** [CI ワークフロー](../../.github/workflows/3x3-web-2.yml) は Rust テスト・Playwright・ビルドを実行する。一方 [Pages ワークフロー](../../.github/workflows/deploy-pages.yml) は `main` への push で独立に開始し、`deploy` は同ワークフロー内の `build` のみを `needs` としている。Pages 側の `build` は `npm run build` を実行するが、Rust/Playwright のテスト結果を確認しない。
- **影響:** `main` の変更がビルド可能で、振る舞いのテストだけ失敗した場合でも、CI の結果が出る前または失敗後に公開され得る。今回の F1 のような、ビルド・既存テストを通る不具合に対する公開ゲートがない。
- **修正案:** デプロイ前に同じコミットの CI 成功を必須にする。例えば Pages のビルド前に検査ジョブを再利用するか、検査成功後のワークフロー連携にし、公開するコミット SHA の一致を確認する。ブランチ保護だけに依存する場合も、直接 push と自動実行時の条件を明記する。

### F3 — P2: `check:guardrails` は検査違反を警告しても成功終了する

- **根拠:** [`check-review-guardrails.js`](../scripts/check-review-guardrails.js#L23) の実際の4検査は `reportWarning` を呼ぶ。末尾の [`hasWarnings` 分岐](../scripts/check-review-guardrails.js#L152) は `process.exit(0)`。`reportError` を呼ぶ箇所はない。したがって検出した違反でも `npm run check` と CI の [`Run review guardrail checks`](../../.github/workflows/3x3-web-2.yml) は成功する。
- **影響:** 既存の[レビュー基準](code-review-guidelines.md)は必須検査としているが、自動ゲートとしては働かない。例えば release 検証を再び `#[cfg(debug_assertions)]` に入れても、このチェック自体は CI を止めない。
- **修正案:** 確定的なルール違反を非ゼロ終了にし、誤検出の可能性がある警告は別の助言コマンドへ分ける。違反を埋め込んだ一時フィクスチャで終了コードもテストする。現行の文字列・近傍行走査は構文理解を持たないため、通過を「安全性の証明」と扱わない。

### F4 — P3: オフライン検証は設定変更後も古い `dist` を「最新」と判定する

- **再現:** 本番ビルド後、`vite.config.ts` の更新時刻だけを `touch` で `dist/index.html` より新しくし、`npm run offline -- --headless` を実行した。出力は `⚡ 既存の最新ビルド成果物を使用します` で、再ビルドせずオフライン起動は成功した。ファイル内容は変更していないが、実際にビルド設定の内容を変えた場合も同じ判定となる。[出力記録](review-evidence/ba8fcfa/operational-probes.md)を参照。
- **原因:** [`ensureBuild`](../scripts/launch-offline.js#L40) は `src/`・`web/`・`public/` の最終更新時刻だけを比較する。`vite.config.ts`、`Cargo.toml`、`build.rs`、`package.json`/`package-lock.json`、`scripts/generate-sw-precache.js`、削除された入力ファイルは判定に入らない。以前の F6 修正記録の「成果物鮮度保証」はこの範囲に限られる。
- **影響:** バンドル設定、依存関係、WASM 生成条件、プリキャッシュ生成方法の変更を、古い成果物で検証して成功と報告し得る。
- **修正案:** 検証モードでは毎回 `npm run build` するか、ビルド入力全体の内容ハッシュを成果物に記録して比較する。mtime を使い続けるなら、ルート設定・生成スクリプト・削除も対象にする。

### F5 — P3: オフライン検証を Ctrl+C で止めると終了処理が例外になる

- **再現:** `npm run offline -- --headless` の起動待ち中に SIGINT を送ると、`TypeError [ERR_INVALID_ARG_TYPE]: The "code" argument must be of type number. Received type string ('SIGINT')` が発生した。[出力記録](review-evidence/ba8fcfa/operational-probes.md)を参照。
- **原因:** [`process.on("SIGINT", cleanup)`](../scripts/launch-offline.js#L196) は signal 名を `cleanup(exitCode = 0)` に渡す。[`cleanup` 内の `process.exit(exitCode)`](../scripts/launch-offline.js#L182) が文字列を終了コードとして受け取る。
- **影響:** ヘルプに記載された Ctrl+C の終了経路でスタックトレースが出る。後続の終了処理は `process.exit` 前に走るが、意図した正常終了にはならない。
- **修正案:** シグナルのコールバックで `cleanup(130)` のように数値を渡す。`SIGTERM` も同様に扱い、子プロセス・ブラウザが残らないことを検証する。

## 横断確認

| 領域 | 確認した根拠と結論 |
| --- | --- |
| Rust 局面・テーブル | [`parse_state`](../src/cube.rs#L33) は色数、センター、重複、向き、置換パリティを検査する。テーブルのマジック・チェックサム検証は [`table_io.rs`](../src/table_io.rs#L34) にある。release テストは通過。任意に破損した埋め込みバイナリの復旧動作までは検証していない。 |
| 探索・WASM | Kociemba、CFOP、Thistlethwaite、Korf とセンター補正を確認。共通の最終完成検証は [`lib.rs`](../src/lib.rs#L240) で release にも存在する。141件の release テストが成功。1000局面回帰テストは `ignored` なので、この実行結果から1000局面の保証は主張しない。 |
| Web 状態・非同期 | Worker の世代・要求 ID・revision 照合、ファイル・プリセットの古い応答排除を確認。解法を閉じる状態遷移だけは F1 の通り失敗する。 |
| 入力・描画 | JSON は全文読込前に 64KB を検査。カメラ画像は MIME と 20MB を事前検査し、後続の読込世代も管理する。3D シーンはジオメトリ・マテリアルを再利用・破棄する。実機カメラの画質・端末差は未検証。 |
| PWA・運用 | ナビゲーションの HTML はクエリを外したキーでキャッシュされる。オフライン起動自体は実行成功。ただし F4 の鮮度判定と F5 の終了処理に問題がある。 |
| セキュリティ・依存 | 利用者の局面・手順は Rust 境界で検証され、主要 UI は `textContent` で表示する。`npm audit --json` は本日時点で本番・開発依存とも既知アドバイザリ0件。これは未報告の問題がない証明ではない。 |

## 検証記録と限界

- `npm run check:guardrails`、`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test --release`、`npm run format:check` は成功。Rust は **141 passed / 1 ignored**。
- `npm run build` は成功。最初の `npm run check` はサンドボックスの `wasm-bindgen` インストール権限でビルド段階に停止したため、権限を上げて同じ本番ビルドを再実行した。`npm test` は **179 passed / 1 skipped**。失敗したアプリ検査はない。
- `npm run offline -- --headless` はオフラインで READY まで起動成功。ただし F4 の条件では古い成果物を用いた成功となる。
- `npm audit --json` は **0 vulnerabilities**。Rust 依存アドバイザリの最新照会、実機カメラ、他ブラウザ、1000局面の ignored テストはこの回に実施していない。
- テスト成功・高カバレッジ・過去の「完全解消」記録を承認の代わりにせず、機能をまたぐ UI 操作と検証ツールの終了コードを確認した。F1〜F5 は現行 `HEAD` に対する指摘であり、過去の修正済み事象をそのまま再掲したものではない。
