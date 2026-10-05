# レビュー指摘事項の修正および再発防止対応レポート（63bce97 / 2026-09-28）

## 概要

[`docs/code-review-63bce97-2026-09-28.md`](code-review-63bce97-2026-09-28.md) で報告された全指摘事項（F1〜F4）への修正対応、品質ゲートの整備、および回帰テストによる検証を完了しました。

---

## 指摘事項別の対応詳細

### F1 (P2): 解法再探索中のシーク・再生操作により完了した探索結果が無言で破棄される

- **問題:** 解法が表示されている状態で別のソルバー（CFOP等）を選択して再探索を開始すると、探索中も画面下部に古い解法のタイムラインや再生ボタン（`#next`, `#play`）が操作可能なまま残り、シーク操作によって `store.revision` が進んで完了した新探索結果が何のエラー表示もなく無言で破棄され、古い解法のまま取り残されていた。
- **原因:** [`web/main.ts`](../web/main.ts) の `solve()` 冒頭で以前の解法が失効されず、また [`refresh()`](../web/main.ts) において `solving` フラグが再生系コントロールの `disabled` 判定に含まれていなかった。
- **修正内容:**
  - [`web/main.ts`](../web/main.ts): `solve()` の開始時に直ちに `store.setSolution(undefined)` を呼び出し、古い解法表示を画面から失効させる。
  - [`web/main.ts`](../web/main.ts): `refresh()` において、再生ボタン（`play`）、ステップボタン（`first`, `prev`, `next`, `last`）、タイムラインスライダー（`timeline`）に `disabled = ... || solving` を追加し、探索中のシーク・再生操作を物理的に遮断。
  - [`web/main.ts`](../web/main.ts): `setupKeyboardShortcuts` の `onPlay` および `onSeek` に `if (solving) return;` の安全ガードを追加し、キーボードショートカット経由のシーク・再生も確実にブロック。
  - [`web/main.ts`](../web/main.ts): `solve()` 完了時、`if (!solving || store.getRevision() !== at) return;` のガードを追加し、探索中止後の不要な結果適用を防止。
- **検証結果:**
  - 新規回帰テスト [`tests/code-review-63bce97-regression.spec.ts`](../tests/code-review-63bce97-regression.spec.ts) にて、解法表示中に再探索を開始した際に古い解法が非表示になり、矢印キーやスペースキーを押してもシーク・再生が発火せず、探索完了後に新しいアルゴリズム（CFOP）の解法が確実に適用されることを確認。

### F2 (P2): カメラエディタ終了後、画像実体が破棄されても DOM カードが「読込完了」を表示し続ける

- **問題:** カメラエディタで画像を読み込んで「読込完了」になった後、Escape キーや閉じるボタンでダイアログを閉じると、内部の画像実体は解放され URL も破棄されるが、DOM の `#camera-status-a` は「読込完了」のまま残り、`#camera-drop-a` にも `has-file` クラスが付与されたままとなり、内部状態と表示が乖離していた。また、ファイル入力の `input.value` がクリアされず、同じファイルを再選択した際に `change` イベントが発火しなかった。
- **原因:** [`web/camera.ts`](../web/camera.ts) の `handleDialogClose` が画像実体と URL の破棄のみを行い、DOM カードのステータス更新、検出結果のリセット、およびファイル入力値のクリアを実行していなかった。
- **修正内容:**
  - [`web/camera.ts`](../web/camera.ts): `handleDialogClose()` 内で `this.clearViewResults("A")`, `this.clearViewResults("B")`, `this.points = []`, `this.centerPoint = undefined` を明示的に呼び出し、検出データをリセット。
  - ファイル入力要素 `#camera-file-a` および `#camera-file-b` の `.value` を `""` にクリア。
  - `this.updateCardStatus("A")`, `this.updateCardStatus("B")`, `this.renderResults()`, `this.update()` を呼び出し、ダイアログ終了直後に DOM カードが未選択状態へ戻り、クラスが除去されるように同期。
- **検証結果:**
  - 新規回帰テストにて、閉じるボタンおよび Escape キー押下直後にステータスカードが「未選択」に戻り、クラスが除去され、`input.value` が空になり、同じファイルを再選択した際に正常に再読み込みできることを確認。

### F3 (P3): 0手解法（完成状態）のタイムライン `aria-valuetext` が「完成 (0手)」ではなく「開始状態」と読まれる

- **問題:** 完成状態（SOLVED）のまま解法探索を実行して 0 手解法が生成された際、手数は 0/0 手で案内文は「6面が揃いました」と表示される一方、タイムラインの `aria-valuetext` は `"開始状態"` と設定され、スクリーンリーダーに完成状態が伝わらなかった。
- **原因:** [`web/main.ts`](../web/main.ts) の `refresh()` における三項演算子で、`step === 0` が `step === solution.moves.length` より先に評価されており、0手解法では `nextMeta` が未定義のため `"開始状態"` が返されていた。
- **修正内容:**
  - [`web/main.ts`](../web/main.ts): `solution.moves.length === 0` を最優先で評価し、手数が 0 手の解法では直ちに `"完成 (0手)"` を返すように分岐順序を是正。
- **検証結果:**
  - 新規回帰テストにて、完成状態での解法生成時に `aria-valuetext` が `"完成 (0手)"` になること、および 2手解法での開始位置・中間位置・完成位置の読み上げ文がすべて正確に動作することを確認。

### F4 (P3): 探索中にプリセットを選択した際、探索が即時中断されずステータス表示が競合する

- **問題:** 重い探索が走っている最中にプリセットボタンをクリックした際、`cancelSearch()` が呼ばれずバックグラウンド探索が継続し、プリセット読み込み完了表示の直後に古い探索の完了通知が `#solver-note` を上書きしていた。また局面置換後も古い探索ノード数が画面に残存していた。
- **原因:** [`web/main.ts`](../web/main.ts) の `initializePresets` で `cancelSearch()` が呼ばれておらず、さらに `replace()` において `solver-note` が初期ステータスにリセットされていなかった。
- **修正内容:**
  - [`web/main.ts`](../web/main.ts): プリセットボタンのクリックハンドラ冒頭で直ちに `cancelSearch()` を呼び出し、実行中の探索 Worker を即時停止。
  - [`web/main.ts`](../web/main.ts): `cancelSearch()` 内で `$("solver-note").textContent = "探索を中止しました"` を設定。
  - [`web/main.ts`](../web/main.ts): `replace()` 内で `$("solver-note").textContent = "ブラウザ内で計算 · 通常5秒以内"` を設定し、プリセット読み込みやスクランブルによる局面置き換え時に古い探索ノード数が残存しないよう初期化。
- **検証結果:**
  - 新規回帰テストにて、探索中にプリセットを選択した際に即座に Worker が中断され、プリセット完了後に旧探索のノード数が `#solver-note` を上書きしないことを確認。

---

## 総合テスト・検証結果

1. **Rust 単体テスト・静的検査:**
   - `cargo fmt --check`: 成功
   - `cargo clippy --all-targets -- -D warnings`: 成功（警告 0 件）
   - `cargo test --release`: **141 passed / 1 ignored**

2. **Web フロントエンド静的検査:**
   - `npm run check:guardrails`: 成功
   - `npm run typecheck`: 成功（TypeScript 型エラー 0 件）
   - `npm run format:check`: 成功（Prettier 不整合 0 件）

3. **Playwright E2E & ユニットテスト:**
   - 全体テストスイート: **204 passed / 1 skipped / 0 failed**（新設回帰テスト 4 件を含む全件合格）
   - 新規回帰テスト `tests/code-review-63bce97-regression.spec.ts`: 全 4 件パス

4. **本番ビルド・PWA・オフライン起動検証:**
   - `npm run build`: WASM 最適化、Vite バンドル、SW プリキャッシュ（19 アセット）生成成功
   - `node scripts/launch-offline.js --headless`: キャッシュ書き込み後の完全オフライン起動検証に成功（終了コード 0）
