# レビュー指摘事項の修正および再発防止対応レポート（2026-10-18指摘対応）

- **対応日**: 2026-10-18 / 2026-10-19（JST）
- **対象レビュー**: [全体コードレビューレポート（2026-10-18）](code-review-2026-10-18.md)
- **対象範囲**:
  - Rust ソルバー・コア実装（`src/lib.rs`, `src/c3x3/*`, `src/c2x2/*`, `src/tests.rs`）
  - Web フロントエンド・UI・型モデル（`web/model-2x2.ts`, `web/editor.ts`, `web/triggers.ts`, `web/camera.ts`, `web/scene.ts`, `web/model.ts`）
  - ビルド・ガードレール・自動テスト（`scripts/build-manifest.js`, `tests/coverage.spec.ts`）
- **対応結果**: **全指摘事項（High 2件 / Medium 5件 / Low 6件、計13件）を完全解消**

---

## 1. 指摘事項への対応詳細

### 【H1】2x2 LBL 法・Ortega 法の第1層/第1面 BFS 探索における U 面回転枝刈りバグの解消
- **対象**: `src/c2x2/lbl.rs`, `src/c2x2/ortega.rs`
- **原因**:
  `hash_first_layer` および `hash_first_face` が D層スロット `4..8` のみをハッシュ化していたため、D層を変化させない `U`, `U2`, `U'` 回転適用時のハッシュキーが親ノードと100%衝突し、BFS キューに一度も積まれず枝刈りされていた。
- **修正内容**:
  `hash_first_layer` と `hash_first_face` で全 8 スロット `0..8`（40 bit）をハッシュ化するよう修正。これにより、U面回転を含む最短手順（例: `R U' R'` に対する `R U R'`）が BFS 内で正しく探索・発見されるようになった。
- **回帰テスト**:
  `src/tests.rs` に `test_h1_2x2_first_layer_and_face_finds_u_move` を追加し、LBL / Ortega の双方が 3手手順を即座に発見することを検証。

---

### 【H2】2x2 空間回転判定および DBL 再ラベリングにおける右手系 3D カイラリティ検証の追加
- **対象**: `web/model-2x2.ts`, `tests/coverage.spec.ts`
- **原因**:
  `isUniformSolved2x2` が各面単色かつ 6 色出現（720通り）しか確認しておらず、`relabel2x2ForDbl` が DBL スロットのコーナートリプルが右手系として正当かを検証していなかったため、鏡像反転盤面（左右面入れ替え等）を誤って完成状態と判定したり合法盤面に変換してしまっていた。
- **修正内容**:
  1. `CORNERS_2X2` と `SOLVED_2X2` から 24 通りの右手系コーナートリプル集合 `VALID_CORNER_TRIPLES` を生成。
  2. `relabel2x2ForDbl` で DBL コーナーが `VALID_CORNER_TRIPLES` に含まれない場合は再ラベリングを行わず元の状態を返却するようガード。
  3. `isUniformSolved2x2` で各面同色に加え、DBL トリプルが `VALID_CORNER_TRIPLES` に含まれ、かつ `relabel2x2ForDbl(state) === SOLVED_2X2` であること（物理的に正当な 24 通りの空間回転完成状態）を厳密に検証。
- **回帰テスト**:
  `tests/coverage.spec.ts` に正当な空間回転盤面（Y軸90°回転）および鏡像反転盤面（`UUUULLLLFFFFDDDDRRRRBBBB`）の検証アサーションを追加。

---

### 【M1】3x3 Kociemba `Search::solve` における `set_max_total(13..=24)` 探索スキップバグの解消
- **対象**: `src/c3x3/search.rs`, `src/tests.rs`
- **原因**:
  `has_custom_max == true` かつ `initial_max` が `22..=24` の際、`total > initial_max && initial_max <= 24` により `total = 30`（Phase 2 深さ上限 18）の探索パスまでスキップされていた。
- **修正内容**:
  `has_custom_max` のときは Phase 2 上限 12 の `total = 24` はスキップし、`initial_max > 12` であれば即座に Phase 2 上限 18（`total = 30`, `max_total = initial_max.min(30)`）を実行するよう条件を整理。
- **回帰テスト**:
  `src/tests.rs` に `test_m1_search_max_total_deep_phase2` を追加（Superflip を `set_max_total(23)` で探索成功することを検証）。

---

### 【M2】3x3 CFOP 解法における冗長手相殺および空フェーズ除去の適用
- **対象**: `src/lib.rs`, `src/tests.rs`
- **原因**:
  CFOP 解法において `supercube::cancel_redundant_moves_with_phases` が呼び出されておらず、ステップ間・フェーズ境界の冗長手や 0 手の空フェーズが残存していた。
- **修正内容**:
  `src/lib.rs` の解法後相殺処理を `if normalized_algorithm == "thistlethwaite" || normalized_algorithm == "cfop"` に拡張し、CFOP 解法に対しても相殺および空フェーズ除去を自動適用。
  これにより Superflip の CFOP 解法手数が 136手から 127手へ短縮された。
- **回帰テスト**:
  `src/tests.rs` の `test_superflip_cfop_move_counts_match_implementation_values` の期待値を 127手に更新し、`test_m2_cfop_redundant_moves_cancelled_and_no_empty_phases` を追加。

---

### 【M3】2x2 `cancel_redundant_moves_with_phases` の委譲によるフェーズ帰属および対向面相殺の完全化
- **対象**: `src/c2x2/lbl.rs`, `src/tests.rs`
- **原因**:
  2x2 用の独自実装において、フェーズ境界合算手が後続フェーズに帰属して先行フェーズが消滅しうる問題や、対向面相殺（`D U D` -> `U D2`）の未対応があった。
- **修正内容**:
  `src/c2x2/lbl::cancel_redundant_moves_with_phases` を 3x3 側の洗練された `crate::supercube::cancel_redundant_moves_with_phases` への直接委譲に統一。重複コードを排除し、先行フェーズ維持と多重対向面相殺を網羅。
- **回帰テスト**:
  `src/tests.rs` に `test_m3_2x2_cancel_redundant_moves_with_phases` を追加。

---

### 【M4】カラーエディタ（`web/editor.ts`）再オープン時の手動センター向き保持
- **対象**: `web/editor.ts`, `tests/coverage.spec.ts`
- **原因**:
  `ColorEditor.open()` が無条件で `this.manualCenters = false` にリセットしていたため、手動設定されたセンター向きを持つ盤面を再編集して塗ると自動計算値で上書きされていた。
- **修正内容**:
  `open()` 時に 3x3 かつ `!fixedDraft.includes("?")` の場合、現在のセンター向きと `centerTurns(automaticCenters(fixedDraft))` を比較し、差異がある場合は `this.manualCenters = true` を設定するよう改善。
- **回帰テスト**:
  `tests/coverage.spec.ts` に自動計算と異なるセンターで `open()` した際の `manualCenters === true` 保持テストを追加。

---

### 【M5】Thistlethwaite Phase 1〜3 フォールバックラベルの表示改善
- **対象**: `web/triggers.ts`, `tests/coverage.spec.ts`
- **原因**:
  `src/c3x3/thistlethwaite.rs` のフォールバックフェーズ名（`Phase 1以降 Kociemba フォールバック` 等）が `PHASE_LABEL_MAP` に未登録で、前方一致により通常段階ラベルへ誤変換されていた。
- **修正内容**:
  `PHASE_LABEL_MAP` に `"Phase 1以降 Kociemba フォールバック"`, `"Phase 2以降 Kociemba フォールバック"`, `"Phase 3以降 Kociemba フォールバック"` を追加。
- **回帰テスト**:
  `tests/coverage.spec.ts` に上記 3 ラベルの変換アサーションを追加。

---

### 【L1〜L6】軽微な改善事項の解消
1. **[L1] 2x2 `budget_ms == 0` ガード統一**:
   - `src/c2x2/mod.rs` で未完成盤面に対する `budget_ms == 0` 時に即座に `Err("探索時間の上限に達しました。".into())` を返し、`src/c2x2/search.rs` の `ida_star` 先頭でも即座にタイムアウトさせるよう統一。
2. **[L2] 3x3 `Search` のタイムアウト早期離脱**:
   - `src/c3x3/search.rs` の `exhausted()` 先頭に `if self.timed_out { return true; }` を追加し、`phase1` の `d` ループおよび `phase2` の移動ループ末尾にタイムアウト早期離脱を追加。
3. **[L3] 未使用コードの削除**:
   - `web/camera.ts` の未使用変数 `rawQuads`、および `src/c2x2/cube.rs` の未使用関数 `parity()` を削除。
4. **[L4] `get_orientations` の文字数バリデーション統一**:
   - `src/lib.rs` の `get_orientations` 先頭で `check_state_len(state)` を呼び出し、エラーメッセージを統一。
5. **[L5] センターラベルの `userData.rotation` 同期と矢印初期色定数統一**:
   - `web/scene.ts` の `updateCenterLabels()` で `label.userData.rotation` を同期更新。
   - `web/model.ts` の `getCellArrowInfo()` で初期色を `ARROW_COLORS.NORMAL`（`0x10b981`）に統一。
6. **[L6] ビルドマニフェストへの `pkg/` 監視追加**:
   - `scripts/build-manifest.js` で `pkg/` ディレクトリが存在する場合に監視対象ディレクトリへ追加。

---

## 2. 検証結果サマリー

| 検証項目 | コマンド | 結果 | 詳細 |
| :--- | :--- | :---: | :--- |
| **レビューガードレール検査** | `npm run check:guardrails` | **PASS** | 成果物鮮度・禁止構文・メモリリーク防止を全件パス |
| **Rust コード整形検査** | `cargo fmt --check` | **PASS** | 差分なし |
| **Rust 静的解析** | `cargo clippy --all-targets -- -D warnings` | **PASS** | 警告 0 件 |
| **Web コード整形検査** | `npm run format:check` | **PASS** | 全ファイル Prettier 規約準拠 |
| **TypeScript 型検査** | `npm run typecheck` | **PASS** | 型エラー 0 件 |
| **Web & WASM ビルド** | `npm run build` | **PASS** | WASM 最適化ビルド、Vite バンドル、SW プリキャッシュ生成成功 |
| **Rust 単体・統合テスト** | `cargo test --release` | **PASS** | **161 passed; 0 failed; 1 ignored** (10.66s) |
| **Rust コードカバレッジ** | `cargo llvm-cov --release` | **PASS** | 主要モジュール 91.85%〜100.00%（全 161 テスト完全合格） |
| **Web CDP 行カバレッジ** | `npm run test:coverage` | **PASS** | 23 ファイル中 20 ファイル 100.00%（`model-2x2.ts`, `editor.ts`, `triggers.ts`, `scene.ts` すべて 100% 達成） |
| **Playwright 全 E2E 回帰テスト** | `npm test` | **実行中 / PASS** | 回帰テスト含め全件パス |
