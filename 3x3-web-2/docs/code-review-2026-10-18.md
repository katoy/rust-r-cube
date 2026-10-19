# 全体コードレビューレポート（2026-10-18）

- **実施日**: 2026-10-18（JST）
- **対象リポジトリ**: `rust-r-cube/3x3-web-2`
- **対象範囲**:
  - Rust ソルバー・コア実装（`src/lib.rs`, `src/c3x3/*`, `src/c2x2/*`, `build.rs`）
  - Web フロントエンド・Web Worker・3D 描画・カメラ認識・PWA（`web/*.ts`, `public/sw.js`, `index.html`, `web/style.css`）
  - ビルド・ガードレール・オフライン検証スクリプト・E2E テスト（`scripts/*`, `start.sh`, `tests/*.ts`, `package.json`）
- **総合判定**: **条件付き承認（要修正: High 2件 / Medium 5件 / Low 6件）**
  - 静的な型検査（`npm run typecheck`）、フォーマット検査（`npm run format:check`, `cargo fmt --check`）、およびレビューガードレール検査（`npm run check:guardrails`）はすべて合格しています。
  - 一方で、全ソースコードの行単位精読およびアルゴリズム・状態遷移のトレースにより、**2x2 LBL / Ortega の第1層・第1面 BFS における `U` 面回転の完全枝刈りバグ（H1）**、**2x2 の空間回転判定・DBL 再ラベリングにおける鏡像（カイラリティ反転）盤面の誤許容（H2）**、および **3x3 Kociemba の `set_max_total` 境界条件バグ（M1）** を含む計 13 件の改善・修正ポイントを特定しました。

---

## 1. 指摘事項サマリー

| ID | 重大度 | カテゴリ | 対象ファイル（行番号） | 概要 |
| :--- | :---: | :--- | :--- | :--- |
| **H1** | **High** | 正確性 / アルゴリズム | [`src/c2x2/lbl.rs`](../src/c2x2/lbl.rs) (L88-174)<br>[`src/c2x2/ortega.rs`](../src/c2x2/ortega.rs) (L76-134) | `hash_first_layer` / `hash_first_face` が D層スロット `4..8` のみをハッシュ化しているため、D層を変化させない `U`, `U2`, `U'` 回転が親ノードと 100% ハッシュ衝突し、BFS 全体で `U` 面回転が 1 手もキューに積まれない。 |
| **H2** | **High** | 正確性 / バリデーション | [`web/model-2x2.ts`](../web/model-2x2.ts) (L16-63)<br>[`web/solver.worker.ts`](../web/solver.worker.ts) (L22-70) | `isUniformSolved2x2` が 6 面単色の全 $6! = 720$ 通り（うち 696 通りは物理的に不可能）を完成と判定し、`relabel2x2ForDbl` が鏡像反転（左右面交換等）した 24 通りの不正配色を合法盤面へ変換してしまう。 |
| **M1** | **Medium** | 正確性 / アルゴリズム | [`src/c3x3/search.rs`](../src/c3x3/search.rs) (L122-136) | `set_max_total(initial_max)` で `initial_max` が `13..=24` の際、`total > initial_max && initial_max <= 24` の条件により `total = 30`（`allow_deep_phase2 = true`、Phase 2 上限 18）の探索パスまでスキップされる。 |
| **M2** | **Medium** | 正確性 / アルゴリズム | [`src/lib.rs`](../src/lib.rs) (L199-213, L326-331)<br>[`src/c3x3/cfop.rs`](../src/c3x3/cfop.rs) (L56-105) | 3x3 CFOP 解法において、`include_orientation = false` またはセンター補正不要時に `cancel_redundant_moves_with_phases` が呼ばれず、サブステップ・フェーズ境界の冗長手（`U U'` 等）や 0 手の空フェーズが残存する。 |
| **M3** | **Medium** | 正確性 / フェーズ境界 | [`src/c2x2/lbl.rs`](../src/c2x2/lbl.rs) (L318-390) | 2x2 用 `cancel_redundant_moves_with_phases` がフェーズ境界の合算手（例: `R` + `R` → `R2`）を後続フェーズに帰属させるため、1 手で完了した先行フェーズが消滅しうる。また対向面可換相殺（`D U D` → `U D2`）が未対応。 |
| **M4** | **Medium** | UI / 状態整合性 | [`web/editor.ts`](../web/editor.ts) (L81-96, L119-136) | `ColorEditor.open()` が常に `this.manualCenters = false` にリセットするため、手動設定済みのセンター向きを持つ盤面を再編集して 1 マス塗ると、全 6 面のセンター向きが自動計算値で上書きされる。 |
| **M5** | **Medium** | UI / 表示 | [`web/triggers.ts`](../web/triggers.ts) (L62-98)<br>[`src/c3x3/thistlethwaite.rs`](../src/c3x3/thistlethwaite.rs) (L146-151) | Thistlethwaite の Phase 1〜3 タイムアウト時に返される `"Phase 1以降 Kociemba フォールバック"` 等が、`getPhaseLabel()` の前方一致（`startsWith("Phase 1")`）により通常の第1〜第3段階ラベルへ誤変換される。 |
| **L1** | Low | API契約 / 境界値 | [`src/c2x2/mod.rs`](../src/c2x2/mod.rs) (L36-103)<br>[`src/lib.rs`](../src/lib.rs) (L424-437) | 2x2 `c2x2::solve` に `budget_ms == 0` の事前ガードがなく、未完成盤面でも `lbl` / `ortega` や 4,095 ノード未満の `optimal` が予算 0 で成功を返す（3x3 と不一致）。 |
| **L2** | Low | パフォーマンス / 探索 | [`src/c3x3/search.rs`](../src/c3x3/search.rs) (L199-208, L255-268, L311-340) | `exhausted()` の先頭に `if self.timed_out { return true; }` がなく、`phase2` 再帰ループおよび `phase1` の `for d in min_d..=max_d` に `self.timed_out` 早期離脱がないため、タイムアウト後も残り分岐を走査して `nodes` が増加する。 |
| **L3** | Low | 可読性 / デッドコード | [`web/camera.ts`](../web/camera.ts) (L649-659)<br>[`src/c2x2/cube.rs`](../src/c2x2/cube.rs) (L77-82) | `web/camera.ts` の `capture()` 内に参照されない `rawQuads` 定義が残存。また `src/c2x2/cube.rs` の `parity()` 関数が未使用。 |
| **L4** | Low | API契約 / メッセージ | [`src/lib.rs`](../src/lib.rs) (L495-522) | WASM 公開関数 `get_orientations(state)` のみ `check_state_len(state)` を呼んでおらず、24/54文字以外の入力時に 3x3 専用のエラーメッセージ（`"54マスすべての色を入力してください。"`）を返す。 |
| **L5** | Low | 3D描画 / 定数整合 | [`web/scene.ts`](../web/scene.ts) (L484-493)<br>[`web/model.ts`](../web/model.ts) (L204) | `updateCenterLabels()` が `label.userData.rotation` を更新していない。また `getCellArrowInfo` の初期色 `0x2ecc71` が `ARROW_COLORS.NORMAL`（`0x10b981`）と不一致。 |
| **L6** | Low | ビルド鮮度管理 | [`scripts/build-manifest.js`](../scripts/build-manifest.js) (L35-38) | `collectInputFiles()` が `pkg/`（WASM 成果物）を監視対象に含めておらず、`src/` 変更後に WASM 未ビルドで Web ビルドのみ実行した場合に古い WASM を検知できない。 |

---

## 2. 詳細解説と改善案

### [H1] 2x2 LBL 法・Ortega 法の第1層/第1面 BFS 探索で `U` 面回転が 100% 枝刈りされるバグ

- **該当箇所**:
  - [`src/c2x2/lbl.rs`](../src/c2x2/lbl.rs): 88〜136行目（`solve_first_layer`）、165〜174行目（`hash_first_layer`）
  - [`src/c2x2/ortega.rs`](../src/c2x2/ortega.rs): 76〜122行目（`solve_first_face`）、124〜134行目（`hash_first_face`）
- **根本原因**:
  ```rust
  impl RawCube {
      #[inline]
      fn hash_first_layer(&self) -> u64 {
          let mut h = 0u64;
          for i in 4..8 {
              h = (h << 3) | (self.cp[i] as u64);
              h = (h << 2) | (self.co[i] as u64);
          }
          h
      }
  }
  ```
  - `self.cp[i]`・`self.co[i]` は「スロット `i` に現在入っているピースとその向き」を表します。
  - `for i in 4..8` は D 層（下面の 4 スロット `4..8`）に入っているピースしか見ておらず、U 層（上面の 4 スロット `0..4`）にあるピースの位置・向きを一切ハッシュに含めていません。
  - ここで `U`, `U2`, `U'`（`face == 0`）を適用すると、U 層（`0..4`）のみが回転し、D 層（`4..8`）はまったく変化しません。
  - したがって、BFS キューから取り出した任意の盤面 `cur` に対して `U`, `U2`, `U'` を適用した `next_cube` は、常に `next_cube.hash_first_layer() == cur.hash_first_layer()`（`ortega` の `hash_first_face` も同一）となり、親ノード `cur` 自身が登録済みの `visited` キーと 100% 衝突して `visited.insert(key)` が常に `false` を返します。
  - また `U` 回転だけで D 層が完成することもないため、**BFS 探索において `U`, `U2`, `U'` は深さ 1 から深さ上限に至るまで一度もキューに積まれません**。
- **再現例**:
  - 完成状態に `R U' R'`（3手）を適用した盤面では、第1層（および第1面）を揃える最短手順は基本トリガーの `R U R'`（3手）です。
  - しかし BFS で 1 手目に `R` を選んだ後、2 手目の `U` が `visited.insert(key) == false` により棄却されるため、`R U R'` を発見できず、`R, F, D, L, B` のみを使った遠回りな手順を探すか、`lbl` では BFS 失敗後に IDA* フォールバック（`solve_first_layer_ida`）へ落ちる挙動となっています。
- **修正案**:
  - 2x2 の全 8 スロット `0..8` の状態は `(3 + 2) * 8 = 40 bit` で `u64` に余裕をもって収まるため、`for i in 0..8` をハッシュ化するか、または [`src/c2x2/coord.rs`](../src/c2x2/coord.rs) の `get_group_b_idx(&next_cube)`（D 面の 4 コーナーピース `DFR, DLF, DBL, DRB` が全 8 スロット `0..8` のどこにどの向きで存在するかを一意に符号化した `0..136,080` の整数）を `visited` キーとして使用します。

---

### [H2] `isUniformSolved2x2` および `relabel2x2ForDbl` における右手系 3D 回転（カイラリティ）未検証

- **該当箇所**:
  - [`web/model-2x2.ts`](../web/model-2x2.ts): 16〜63行目
  - [`web/solver.worker.ts`](../web/solver.worker.ts): 22〜70行目
- **根本原因**:
  1. `isUniformSolved2x2(state)` は各面 4 マスが同色で 6 色すべてが登場すること（$6! = 720$ 通り）しか確認していません。標準配色 `SOLVED_2X2` を 3D 空間で回転させて到達できる正当な完成状態は 24 通りのみであり、残りの 696 通り（例: U面の裏である D面が赤 `R` になっている盤面や、R面と L面だけを入れ替えた鏡像反転盤面 `UUUULLLLFFFFDDDDRRRRBBBB`）は物理的に不可能な不正盤面です。
  2. `relabel2x2ForDbl(state)` は DBL コーナーの 3 マス `(state[14], state[23], state[18])` とその対向面色から 6 色の変換マップを作っていますが、`(dCol, bCol, lCol)` が右手系の正当なコーナー配色（8 コーナー × 3 向き = 24 通り）であるかを検証していません。そのため、左右反転した鏡像コーナー（例: `(D, B, R)`、計 24 通り）も `(D, B, L)` に変換してしまい、本来 WASM の `c2x2::cube::parse_state` で「スロット N のコーナー配色が物理的に不正です」と弾かれるべき鏡像反転盤面が、`includeOrientation = false` かつ `lbl` / `ortega` のときに合法盤面へ化けてしまいます。
- **修正案**:
  - `CORNERS_2X2` と `SOLVED_2X2` から正当な 24 通りの右手系コーナートリプル集合 `VALID_CORNER_TRIPLES`（例: `"DBL"`, `"BLD"`, `"LDB"`, `"UFR"`, ...）を生成し、`relabel2x2ForDbl(state)` で `VALID_CORNER_TRIPLES.has(dCol + bCol + lCol)` を満たさない場合は変換を行わないようにします。
  - `isUniformSolved2x2(state)` は `state.length === 24 && VALID_CORNER_TRIPLES.has(state[14] + state[23] + state[18]) && relabel2x2ForDbl(state) === SOLVED_2X2` で厳密に 24 通りのみを判定します。

---

### [M1] 3x3 Kociemba `Search::solve` で `set_max_total(13..=24)` の際に Phase 2 深さ上限 18 の探索がスキップされるバグ

- **該当箇所**: [`src/c3x3/search.rs`](../src/c3x3/search.rs): 122〜136行目
- **根本原因**:
  ```rust
  if self.best_solution.is_none() && !self.timed_out && initial_max >= 22 {
      for &total in &[24, 30] {
          if self.has_custom_max && total > initial_max && initial_max <= 24 {
              continue;
          }
          if total == 24 && initial_max >= 24 {
              continue;
          }
          if total == 30 {
              self.allow_deep_phase2 = true;
          }
  ```
  - `has_custom_max == true` かつ `initial_max` が `22, 23, 24` のとき、`total = 30` のループで `total (30) > initial_max (22..=24)` かつ `initial_max <= 24` が成立して `continue` されるため、`self.allow_deep_phase2 = true`（Phase 2 深さ上限 18）の探索が一度も実行されません。
  - また `has_custom_max == true` の場合、1 回目のループですでに `self.max_total = initial_max`（Phase 2 上限 12）の探索は完了しているため、`total = 24`（Phase 2 上限 12）は `initial_max` の値にかかわらず常に冗長です。
- **修正案**:
  - `has_custom_max == true` のときは `total = 24` を常にスキップし、`initial_max > 12` であれば `total = 30`（`allow_deep_phase2 = true`, `self.max_total = initial_max.min(30)`）を実行するよう条件を整理します。

---

### [M2] 3x3 CFOP 解法における冗長手相殺・空フェーズ除去の未適用

- **該当箇所**: [`src/lib.rs`](../src/lib.rs): 199〜213行目、326〜331行目
- **根本原因**:
  - `src/lib.rs` の 326行目では `if normalized_algorithm == "thistlethwaite"` のみ `supercube::cancel_redundant_moves_with_phases(moves, phase_infos)` を呼び出しています。
  - `cfop::solve` は Cross（4 エッジ順次探索）、First Layer（4 コーナー順次探索）、Second Layer（4 エッジ順次探索）、OLL（エッジ＋コーナー）、PLL（コーナー＋エッジ）を単純連結するため、ステップ内およびステップ間で `U U'`、`U U2`、`D D'` 等が発生しうるほか、初期状態で既に揃っているステップ（例: Cross が 0 手）が空フェーズ（`start == end`）のまま返されます。
- **修正案**:
  - `if normalized_algorithm == "thistlethwaite" || normalized_algorithm == "cfop"` で `supercube::cancel_redundant_moves_with_phases` を適用し、冗長手の相殺と空フェーズの除去を統一します。

---

### [M3] 2x2 `lbl::cancel_redundant_moves_with_phases` におけるフェーズ境界合算時の先行フェーズ消滅と対向面相殺の欠落

- **該当箇所**: [`src/c2x2/lbl.rs`](../src/c2x2/lbl.rs): 318〜390行目
- **根本原因**:
  - 366〜369行目で同一面の 2 手が合算（`total_t > 0`）された際、`out_phases.pop()` で先行フェーズのインデックスを破棄し、`out_phases.push(p_idx)` で後続フェーズのインデックスを付与しています。
  - これにより、ステップ 1 が 1 手（例: `R`）でステップ 2（OLL）の先頭手が `R` の場合、合算された `R2` がステップ 2 に帰属し、ステップ 1 のフェーズが空になって `new_phases` から消滅します。
  - また、3x3 の `supercube::cancel_redundant_moves_with_phases` と異なり、対向面（`U` と `D` 等）を跨いだ相殺（Ortega の PBL 前後で頻出する `D U D` → `U D2` 等）が行われません。
- **修正案**:
  - 2x2 と 3x3 の回転インデックス（`0..18`）は完全に同一仕様であるため、`lbl::cancel_redundant_moves_with_phases` の内部実装を `crate::supercube::cancel_redundant_moves_with_phases(moves, phases)` への委譲に統一します。

---

### [M4] カラーエディタ（`web/editor.ts`）再オープン時の手動センター向きリセット

- **該当箇所**: [`web/editor.ts`](../web/editor.ts): 81〜96行目、119〜136行目
- **根本原因**:
  - `open(state, centers)` で `this.turns = centerTurns(centers)` を設定した直後に無条件で `this.manualCenters = false` としているため、手動設定されたセンター向きを持つ盤面を再度エディタで開いて 1 マスでも塗ると、`paint()` 内の `if (!is2x2 && !this.manualCenters)` 分岐により `automaticCenters(this.draft)` で全センター向きが上書きされます。
- **修正案**:
  - `open(state, centers)` において、3x3 かつ `!fixedDraft.includes("?")` のときに `centerTurns(automaticCenters(fixedDraft))` と `this.turns` を比較し、差異がある場合は `this.manualCenters = true` に設定します。

---

### [M5] Thistlethwaite の Phase 1〜3 フォールバックラベルの誤変換

- **該当箇所**: [`web/triggers.ts`](../web/triggers.ts): 62〜98行目
- **根本原因**:
  - `src/c3x3/thistlethwaite.rs` の `fallback_solve` が生成する `"Phase 1以降 Kociemba フォールバック"`、`"Phase 2以降 Kociemba フォールバック"`、`"Phase 3以降 Kociemba フォールバック"` が `PHASE_LABEL_MAP` に未登録であり、`getPhaseLabel()` の前方一致ループで `"Phase 1"` / `"Phase 2"` / `"Phase 3"` にマッチして通常の段階名に変換されてしまいます。
- **修正案**:
  - `PHASE_LABEL_MAP` に上記 3 件のフォールバックフェーズ名を明示的に追加します。

---

### [L1〜L6] 軽微な改善事項

1. **[L1] 2x2 `budget_ms == 0` の事前チェックと WASM 関数の予算統一**（[`src/c2x2/mod.rs`](../src/c2x2/mod.rs), [`src/c2x2/search.rs`](../src/c2x2/search.rs)）:
   - `c2x2::solve` で未完成盤面に対して `budget_ms == 0` が渡された場合は 3x3 と同じく即座に `Err("探索時間の上限に達しました。".into())` を返し、`c2x2::search::Search::ida_star` でも `budget_ms == Some(0)` の場合は即座にタイムアウトさせます。
2. **[L2] 3x3 `Search` のタイムアウト早期離脱**（[`src/c3x3/search.rs`](../src/c3x3/search.rs)）:
   - `exhausted()` の先頭に `if self.timed_out { return true; }` を追加し、`phase1` の `for d in min_d..=max_d` および `phase2` の移動ループ末尾に `if self.timed_out { break / return false; }` を追加します。
3. **[L3] 未使用コードの整理**（[`web/camera.ts`](../web/camera.ts), [`src/c2x2/cube.rs`](../src/c2x2/cube.rs)）:
   - `web/camera.ts` の `capture()` 内の未使用変数 `rawQuads`、および `src/c2x2/cube.rs` の未使用関数 `parity()` を削除します。
4. **[L4] `get_orientations` の文字数チェック統一**（[`src/lib.rs`](../src/lib.rs)）:
   - `get_orientations(state)` の先頭で `check_state_len(state).map_err(to_js_error)?;` を呼び出し、エラーメッセージを他の公開 API と統一します。
5. **[L5] `updateCenterLabels` の `userData.rotation` 同期と `getCellArrowInfo` の初期色定数統一**（[`web/scene.ts`](../web/scene.ts), [`web/model.ts`](../web/model.ts)）:
   - `web/scene.ts` の `updateCenterLabels()` 内で `if (label.userData) label.userData.rotation = label.quaternion.clone();` を更新し、`web/model.ts` の `getCellArrowInfo()` の `colors` 初期値を `ARROW_COLORS.NORMAL` に統一します。
6. **[L6] ビルドマニフェストへの `pkg/` 追加**（[`scripts/build-manifest.js`](../scripts/build-manifest.js)）:
   - `collectInputFiles()` の監視対象に `pkg/cube_studio_bg.wasm` および `pkg/cube_studio.js` を追加し、WASM 再ビルドが行われた場合や欠落している場合の不整合を検知できるようにします。
