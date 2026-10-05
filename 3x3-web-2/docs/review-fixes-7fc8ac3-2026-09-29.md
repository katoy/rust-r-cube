# レビュー指摘事項の修正および再発防止対応レポート（7fc8ac3 / 2026-09-29）

## 概要

[`docs/code-review-7fc8ac3-2026-09-29.md`](code-review-7fc8ac3-2026-09-29.md) で報告された全指摘事項（F1〜F5）への修正対応、品質ゲートの整備、および回帰テストによる検証を完了しました。
すべての指摘に対する再発防止テスト（[`tests/code-review-7fc8ac3-regression.spec.ts`](../tests/code-review-7fc8ac3-regression.spec.ts) 全 5 件）を新設し、すべて合格しています。

---

## 指摘事項別の対応詳細

### F1 (P2): 解法プレビューシーク中に「保存」（`#save`）を押すと、スクランブル局面ではなくプレビュー途中局面が保存される

- **問題:** 解法をシークバーでプレビューしている最中に「保存」をクリックして JSON ファイルをダウンロードすると、解く対象だった元のスクランブル局面ではなく、プレビュー途中の局面（または完成局面）が保存されていた。
- **原因:** [`web/main.ts`](../web/main.ts) の `save.onclick` が `store.getSnapshot()` を直接呼んでおり、プレビュー中も退避スナップショット（`store.getBaseSnapshot()`）を考慮していなかったため（F3 / 004290a で `#share-link` に施された修正の横展開漏れ）。
- **修正内容:**
  - [`web/main.ts`](../web/main.ts): `save.onclick` において、`const snapshot = store.getBaseSnapshot() ?? store.getSnapshot();` を用いて JSON Blob を生成するように改修。
- **検証結果:**
  - 回帰テスト `F1: 解法プレビューシーク中に『保存』を押した際、プレビュー途中局面ではなく元のスクランブル局面がファイル保存される` にて、プレビュー最終局面までシークした状態でも保存された JSON 内の state が元のスクランブル局面であることを確認。

---

### F2 (P2): 解法表示の「✕」（`#solution-close`）を押した際、プレビュー途中局面が実盤面に固定されて元のスクランブル局面が失われる

- **問題:** 解法をシークしてプレビューした状態で解法カードの「✕」ボタンを押して解法表示を閉じると、プレビューしていた中間局面（または完成局面）がキューブの実盤面に固定（コミット）されてしまい、元のスクランブル局面へ復元されなかった。
- **原因:** `solution-close.onclick` 内で `stop()` と `store.setSolution(undefined)` のみを呼び出しており、`store.restoreBaseSnapshot()` を呼び出していなかったため。
- **修正内容:**
  - [`web/main.ts`](../web/main.ts): `solution-close.onclick` において `stop()` 直後に `store.restoreBaseSnapshot();` を明示的に呼び出し、解法を閉じた際に元のスクランブル盤面へ安全に巻き戻すよう改修。
- **検証結果:**
  - 回帰テスト `F2: 解法プレビューシーク中に解法カードの閉じるボタン（✕）を押した際、プレビュー局面がコミットされず元のスクランブル局面に復元される` にて、✕ボタン押下後にキューブの局面が元のスクランブル状態に復元されることを確認。

---

### F3 (P2): プリセット非同期通信待機中に `solve()` 探索が開始された場合、遅延到着したプリセットデータで探索が中断・上書きされるレースコンディション

- **問題:** プリセットボタンをクリック後、ネットワーク通信（fetch）が遅延している最中にユーザーが「解法を探す」を押して探索を開始すると、遅れて到着したプリセットデータによって探索が `cancelSearch()` で破棄され、盤面がプリセットで強制上書きされていた。
- **原因:** `initializePresets` 内の非同期通信完了ガードにおいて `store.getRevision() !== initialRevision`（盤面変更）のみを検証しており、探索フラグ `solving` の状態を検証していなかったため。
- **修正内容:**
  - [`web/main.ts`](../web/main.ts):
    - プリセットクリックハンドラ冒頭に `if (!mainReady || solving) return;` ガードを追加。
    - fetch 完了時のガードに `if (solving || store.getRevision() !== initialRevision)` を追加し、探索が開始されている場合はプリセットの適用を拒否して操作優先メッセージを表示。
- **検証結果:**
  - 回帰テスト `F3: プリセット通信待機中に探索が開始された場合、遅延到着したプリセットデータで探索が中断・上書きされない` にて、遅延到着したプリセットが安全に棄却され、探索が継続されることを確認。

---

### F4 (P3): 空アルゴリズムまたは空白のみで「手順を適用」を押した際、表示中の解法が破棄され無意味な回転音が鳴る

- **問題:** 手順テキスト入力が空または空白のみの状態で「手順を適用」ボタンを押すと、盤面は一切回転しないにもかかわらず、現在表示されている解法カードが破棄され、無駄な回転 SE（Play move sound）が再生されていた。
- **原因:** [`web/main.ts`](../web/main.ts) の `applyAlgorithm` が空文字列の早期リターンを行わず、Wasm 呼び出し `apply_moves` に渡して `stop()`, `cancelSearch()`, `store.setSolution(undefined)`, `playMoveSound()` を無条件で実行していたため。
- **修正内容:**
  - [`web/main.ts`](../web/main.ts): `applyAlgorithm` 冒頭に `const trimmed = algorithm.trim(); if (!trimmed) return;` を追加し、無効な空適用を即座に破棄。
- **検証結果:**
  - 回帰テスト `F4: 空入力または空白のみで『手順を適用』をクリックした際、表示中の解法が破棄されない` にて、空入力時に解法カードが消えず、不要な副作用が発生しないことを確認。

---

### F5 (P3): 探索中（`solving = true`）における修飾子ボタン（`#prime`, `#double`）の無効化漏れとキーボード押下時のアクティブスタイル付与漏れ

- **問題:** 探索中に `#prime`, `#double` ボタンが `disabled` にならずクリック可能に見えていた。また、キーボードで面キー（`U`, `R` 等）を押した際、探索ガードによって回転はしないものの、画面上の回転ボタンやプライムボタンに `.active-press` スタイルが付与されて点滅していた。
- **原因:**
  - [`web/main.ts`](../web/main.ts) の `refresh()` 無効化セレクタに `#prime,#double` が含まれていなかった。
  - [`web/keyboard-shortcuts.ts`](../web/keyboard-shortcuts.ts) の `onKeyDown` 内で、ボタン要素へのクラス付与（`btn.classList.add("active-press")`）が `options.isReady()` ガードの手前に記述されており、探索中状態（`solving`）の確認がショートカットマネージャ側に渡されていなかった。
- **修正内容:**
  - [`web/main.ts`](../web/main.ts): `refresh()` の無効化対象セレクタに `#prime,#double` を追加。
  - [`web/keyboard-shortcuts.ts`](../web/keyboard-shortcuts.ts): `KeyboardShortcutsOptions` に `isSolving?: () => boolean` を追加し、`onKeyDown` 冒頭で `options.isSolving?.()` をチェックして早期リターン。
- **検証結果:**
  - 回帰テスト `F5: 探索中に修飾子ボタンが disabled に無効化され、キーボード押下時にも回転ボタンに active-press が付与されない` にて、探索中にボタンが無効化され、キーストロークによる不要な UI フィードバックが発生しないことを確認。

---

## 全体検証結果

| 検証項目 | コマンド | 結果 |
| :--- | :--- | :--- |
| ガードレール静的検証 | `npm run check:guardrails` | ✅ PASS |
| Rust フォーマット | `cargo fmt --check` | ✅ PASS |
| Rust 静的解析 (Clippy) | `cargo clippy --all-targets -- -D warnings` | ✅ PASS |
| Rust ユニット・統合テスト | `cargo test --release` | ✅ 141 passed; 0 failed |
| Web コードスタイル (Prettier) | `npm run format:check` | ✅ All matched files use Prettier code style! |
| Web ビルド & 型検査 | `npm run build` (`wasm` + `tsc` + `vite`) | ✅ PASS |
| 今回の回帰テスト | `npx playwright test tests/code-review-7fc8ac3-regression.spec.ts` | ✅ 5 passed |
| 直近レビュー回帰テスト統合 | `npx playwright test tests/code-review-*.spec.ts` (一部抜粋) | ✅ 14 passed |
