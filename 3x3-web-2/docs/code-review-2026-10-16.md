# 全体コードレビューレポート（2026-10-16）

- **実施日**: 2026-10-16（JST）
- **対象リポジトリ**: `rust-r-cube/3x3-web-2`（直近の `e0a282e` 指摘修正反映後の作業ツリー全体）
- **対象範囲**: `src/`（`lib.rs`, `c3x3/`, `c2x2/`, `bin/`, `tests.rs`）、`build.rs`、`web/` 全 23 モジュール、`public/sw.js`、`scripts/`、`tests/`、`start.sh`、設定ファイル、ドキュメント（`README.md`, `CLAUDE.md`, `docs/`）
- **総合判定**: **要修正（CHANGES REQUESTED）** — High 1 件 / Medium 7 件 / Low 7 件

---

## 0. 検証方法と自動チェック結果

1. **全ソース精読と実行検証**:
   - Rust ソルバー（3x3 / 2x2）・WASM バインディング・TypeScript フロントエンド・Service Worker・ビルド／ガードレールスクリプト・Playwright テストスイートを精読し、Node.js からビルド済み WASM（`pkg/cube_studio_bg.wasm`）および各モジュールを直接駆動する検証プローブを実行して挙動と性能を実測しました。
   - 前回レビュー（[`docs/code-review-e0a282e-2026-10-09.md`](code-review-e0a282e-2026-10-09.md)）およびその修正レポート（[`docs/review-fixes-e0a282e-fixes-2026-10-09.md`](review-fixes-e0a282e-fixes-2026-10-09.md)）の主張と実際のコード実装を突合し、修正レポートの記述と実装が乖離している箇所や、修正に伴って新たに発生した退行（Regression）を洗い出しました。
2. **確証度の定義**:
   - **確認済み（実測・再現）**: WASM 実行またはスクリプト実行により具体的な入力と出力で再現を確認済み
   - **コード上確認**: 条件分岐・データフローから決定的に成立することを確認済み

### 0.1 自動チェック結果

| コマンド / 検証項目 | 結果 | 備考 |
| :--- | :--- | :--- |
| `npm run check:guardrails` | PASS | ただし §3.5 M7 のとおり検査抜けあり |
| `cargo fmt --check` | PASS | 差分 0 |
| `npm run typecheck` | PASS | エラー 0（ただし `tests/` は対象外：L6） |
| `npm run format:check` | PASS | Prettier 整合 |
| Release テストバイナリ実行 | PASS | 155 passed / 0 failed / 1 ignored |

---

## 1. 総括（Executive Summary）

### 良い点・改善が確認された点
- **ソルバーの基本正当性と最終検証**: 3x3（Kociemba, CFOP, Thistlethwaite, Korf）および 2x2（Optimal, LBL, Ortega）ともに、Release ビルドでも最終的な盤面完成検証（`verify_solution`）が無条件に機能しており、壊れた解を注入するフォールトインジェクションテストも [`src/tests.rs`](../src/tests.rs) に追加されています。
- **2x2 最短探索（Optimal）の高速化**: `include_orientation = false` 時に DBL コーナーを位置 6・向き 0 に固定して探索を 1 回に絞る改善（[`src/c2x2/search.rs`](../src/c2x2/search.rs)）により、11 手アンチポード局面の探索時間が約 9.7 秒から数十ミリ秒へと大幅に短縮されています。
- **カメラ入力のバイナリヘッダ解析**: [`web/camera.ts`](../web/camera.ts) の `readImageDimensions` がチャンク読み込みによる JPEG セグメント走査・WebP（VP8 / VP8L / VP8X）・GIF・BMP 解析に刷新され、巨大画像のデコード前 Fail-fast（原則 ③）が強化されています。

### 今回検出された主な問題点
一方で、`e0a282e` 修正時の変更によって **3x3 Kociemba（センター向き込み）の探索性能・手数が大きく退行している問題（H1）** をはじめ、**2x2 向き OFF 時の UI・WASM・LBL/Ortega の完成判定不整合（M1）**、**不正 URL パラメータによる既存保存データの破壊（M2）**、**Service Worker のインメモリ配列による上限管理の破綻（M3）**、および **修正レポート（`review-fixes-e0a282e`）の記載内容と実コードの乖離（M1, M5, M6, M7）** が確認されました。

---

## 2. 指摘一覧（サマリー）

| ID | 重大度 | 領域 | 概要 | 確証度 |
| :--- | :---: | :--- | :--- | :--- |
| **H1** | **High** | Rust 3x3 | センター向き込み Kociemba 探索で `set_max_total(seq_len)` と `self.max_total == 30` の不整合により Phase 2 が 12 手に制限され、重複探索の末に約半数の局面でタイムアウト・長手数の逐次解（49〜125手）へ退行する | 確認済み（実測） |
| **M1** | **Medium** | 2x2 / Web & Rust | 2x2「標準の向きに揃える」OFF 時に、空間回転された完成状態を UI（`#cube-status`・効果音）と WASM `is_solved`/`validate` が「未完成（スクランブル状態）」と誤判定する。また LBL / Ortega が 1 手以上の局面で向き OFF を無視する | 確認済み（実測） |
| **M2** | **Medium** | Web / URL・永続化 | URL の `state` が不正な場合でも、バリデーション前に `setCubeType` と `paramsApplied = true` が実行され、`localStorage` の既存保存状態が初期盤面で上書き破壊される。また `?centers=` 単独指定が無視される | 確認済み |
| **M3** | **Medium** | PWA / SW | `public/sw.js` の `queryKeysQueue` がインメモリ配列のため SW 再起動でリセットされ、同一 URL の再取得でも重複登録されて使用中のキャッシュ自身を誤削除する。また旧キャッシュ世代ソートがランダムハッシュ順になっている | 確認済み |
| **M4** | **Medium** | Rust 3x3 | 3x3 の CFOP（100%の局面）と Thistlethwaite（35%の局面）で、向き OFF 時やセンター補正 0 手時にフェーズ境界の連続同面回転（`U U2` 等）が相殺されず、空フェーズ（`start == end`）も残る | 確認済み（実測） |
| **M5** | **Medium** | Rust 2x2 | Ortega 法 `solve_first_face` の BFS 上限変更（L26）で `:90` のみ `>= 6` に変え、キュー追加条件 `:117` が `moves.len() + 1 < 5` のまま取り残されており、深さ 6 拡張がデッドコード化している | コード上確認 |
| **M6** | **Medium** | Web 状態管理 | 修正レポートに記載された `typeHistoryMap` が `web/cube-store.ts` に実装されておらず、種別ボタンの往復で作業盤面がリセットされ Redo 履歴も消える。またプレビュー中に「解法を閉じる」を押すと途中盤面が確定コミットされる | コード上確認 |
| **M7** | **Medium** | ガードレール・テスト | `check-review-guardrails.js` がコメントで謳う `tests/fixtures/guardrails/` が存在せず、SW の `.delete(` 検査や `.size` 検査が形骸化している。また `coverage.spec.ts` に依然としてモックサイズ引数（72B 文字列に `70000` 指定）が残存 | 確認済み |
| **L1** | **Low** | Web UI | `Undo` / `Redo` 実行時に `#scramble-text` がクリアされず古い手順が残る。また 2x2「完成状態」プリセット選択時に `#scramble-text` が空文字になり初期文言と不整合 | 確認済み |
| **L2** | **Low** | Web 状態管理 | `switchCubeType` が `store.setCubeType` のイベント通知（`store.subscribe`）と関数本体の両方で `updateCubeTypeUI` / `scene.setCubeType` / `persist` / `refresh` を二重実行している | コード上確認 |
| **L3** | **Low** | カメラ入力 | 2x2 カメラ結果の手動色補正で `?`（未入力）を選ぶと直前の整合性エラー表示が消えない。また `capture()` 内の `rawQuads` が未使用のデッドコード | コード上確認 |
| **L4** | **Low** | Rust 3x3 | Thistlethwaite の Phase 3 に枝刈りがなく、予算 3,000ms で 35〜40% が Kociemba フォールバックになる（継続課題） | 確認済み（実測） |
| **L5** | **Low** | Rust 全般 | `c2x2::tables::MoveTable`（約 1.5MB）および `c3x3::search::Search::with_max_total` が未使用のデッドコード | コード上確認 |
| **L6** | **Low** | 型検査・設定 | `tsconfig.json` の対象に `tests/` と `playwright.config.ts` が含まれておらず、`start.sh` の鮮度検査に `build.rs` が含まれていない | コード上確認 |
| **L7** | **Low** | リポジトリ衛生 | ルート `cubes/` と `public/cubes/` の重複、`coverage-tarpaulin*/` 等の生成物残存、`README.md` のテスト件数表記ズレ（154件 vs 155件） | 確認済み |

---

## 3. 指摘の詳細と修正案

### 3.1 Rust 3x3 ソルバー（`src/lib.rs`, `src/c3x3/`）

#### H1【High・確認済み（実測）】センター向き込み Kociemba 探索における Phase 2 深さ制限バグと重複探索による大幅な退行
- **該当箇所**:
  - [`src/lib.rs:253-256`](../src/lib.rs)
  - [`src/c3x3/search.rs:85-129, 193-195`](../src/c3x3/search.rs)
- **原因分析**:
  1. `e0a282e` の修正で、[`src/lib.rs:254`](../src/lib.rs) にて同時最適化探索 `search_oriented` に逐次解の手数 `seq_len`（色解法約 20 手＋センター補正 12〜104 手＝通常 `32〜125` 手）を `search_oriented.set_max_total(seq_len)` で設定するようになりました。
  2. しかし [`src/c3x3/search.rs:194`](../src/c3x3/search.rs) の Phase 2 最大深さ計算が以下のようになっています：
     ```rust
     let max =
         (self.max_total - self.path.len()).min(if self.max_total == 30 { 18 } else { 12 });
     ```
     `self.max_total == 30` と等号判定しているため、`self.max_total` が `64` や `90` のとき Phase 2 の最大深さ `max` は **`12`** に制限されます。
  3. センター向きを考慮する Supercube 探索では、Phase 1 で R, F, L, B センターを偶数回転に揃えつつ Phase 2 で 6 面すべてのセンターを `0 mod 4` に戻す必要があるため、Phase 1 が 8〜10 手のときに Phase 2 で **13〜16 手** を要する局面が頻繁に現れます。Phase 2 が `12` 手に制限されるとこれらの解がすべて枝刈りされます。
  4. さらに第1ループ（`for depth in 0..=12`, `:85-103`）は Phase 1 の深さ `11, 12` という巨大分岐木を Phase 2 上限 `12` のまま最後まで探索し続けます。
  5. 第1ループがタイムアウトせずに終わった場合でも、続く段階緩和ループ（`:106-129`）の最初のイテレーション `total = 24` では `self.max_total == 30` が依然として偽（Phase 2 上限 `12`）であるため、**第1ループと 100% 同一の探索（Phase 1 `0..=12` × Phase 2 `<= 12`）をもう一度ゼロからやり直す**結果となっています。
  6. この二重探索によって `total = 30`（Phase 2 上限 `18`）に到達する前に時間予算を使い切り、多数の局面で同時最適化に失敗して 49〜125 手の逐次解へフォールバックしています。
- **実測データ（WASM Release ビルド、UI 既定の Kociemba・向き ON）**:
  - `seed = 1..=15`（予算 2,000ms）で計測した結果、**15 局面中 7 局面（47%）** がタイムアウトして逐次解に退行しました（`seed=2`: 64手、`seed=5`: 90手、`seed=8`: 71手、`seed=9`: 125手、`seed=12`: 49手、`seed=13`: 122手、`seed=15`: 88手）。
  - 予算 5,000ms（UI デフォルト）でも、`seed=2`（5,001ms・64手）や `seed=5`（5,001ms・90手）が 5 秒間フルにブロックした末に 64〜90 手の逐次解へ退行します。
- **付随する問題**:
  - [`src/c3x3/search.rs:106`](../src/c3x3/search.rs) の `if self.best_solution.is_none() && !self.timed_out && initial_max >= 22` により、呼び出し側が `set_max_total(22)` や `set_max_total(23)` で上限を指定していても、解が見つからないと勝手に `max_total = 24, 30` へ緩和して上限を超える解を返します。
  - [`src/lib.rs:239`](../src/lib.rs) で `seq_len` を計算する際、`cancel_redundant_moves_with_phases` による接合部相殺前の `cm.len() + fixes.len()` を使っているため、相殺後の逐次解より手数が長い同時最適化解を採用する可能性があります。また `fixes.is_empty()`（色解法だけでセンターも揃った場合）でも `seq_len = cm.len()` で再探索を行っています。
- **修正案**:
  1. [`src/c3x3/search.rs:194`](../src/c3x3/search.rs) を `if self.max_total >= 30 || self.target_centers.is_some() { 18 } else { 12 }` に修正する。
  2. `set_max_total` で上限が指定されている場合は `initial_max` を超える緩和を行わず、また `initial_max >= 24` のときに `total = 24` を再探索する重複ループを廃止する。
  3. [`src/lib.rs:235-256`](../src/lib.rs) では、逐次解 `seq_moves` を先に `cancel_redundant_moves_with_phases` で相殺した後の長さを `seq_len` とし、`search_oriented.set_max_total(seq_len.min(30))`（または `fixes.is_empty()` のときは `cm.len().saturating_sub(1)`）を設定する。

---

#### M4【Medium・確認済み（実測）】3x3 CFOP・Thistlethwaite におけるフェーズ境界の連続同面回転と空フェーズの残存
- **該当箇所**:
  - [`src/lib.rs:106-132, 173-204, 334-340`](../src/lib.rs)
  - [`src/c3x3/cfop.rs:51-118`](../src/c3x3/cfop.rs)
  - [`src/c3x3/thistlethwaite.rs:64-135`](../src/c3x3/thistlethwaite.rs)
- **問題**:
  - 2x2 の LBL / Ortega では常に `cancel_redundant_moves_with_phases` を呼んでフェーズ間の冗長手を相殺・空フェーズを除去していますが、3x3 では [`src/lib.rs:124`](../src/lib.rs) の `apply_center_orientation_fix` 内（`include_orientation == true` かつ `initial_centers` があり、さらに `!center_fixes.is_empty()` のとき）でしか `cancel_redundant_moves_with_phases` を呼んでいません。
  - **実測**:
    - `seed = 1..=20` を `include_orientation = false` で解くと、**CFOP では 20/20 局面（100%）**、**Thistlethwaite では 7/20 局面（35%）** で同一面の連続回転（例: `U` の直後に `U2` や `U'`）が手順に含まれます。
    - 具体例として、`U` 1 手のみを適用した盤面を Thistlethwaite（向き ON、センター `[1,0,0,0,0,0]`）で解くと、Phase 3 が `U`、Phase 4 が `U2` を出力し、`center_fixes` が空（0手）のため相殺が走らず、最終手順として **`U U2`（2手）** と空フェーズ `Phase 1 (0..0)`, `Phase 2 (0..0)` がそのまま返されます。
- **修正案**:
  - 3x3 でもアルゴリズムや `include_orientation` の有無にかかわらず、生成された `(moves, phase_infos)` に対して共通で `supercube::cancel_redundant_moves_with_phases(moves, phase_infos)` を適用する。

---

### 3.2 Rust 2x2 ソルバーと WASM 境界（`src/c2x2/`, `src/lib.rs`, `web/main.ts`）

#### M1【Medium・確認済み（実測）】2x2「標準の向きに揃える」OFF 時の完成状態判定不整合と LBL / Ortega の向き無視
- **該当箇所**:
  - [`web/main.ts:393-401, 420, 646`](../web/main.ts)
  - [`src/lib.rs:409-432`](../src/lib.rs)
  - [`src/c2x2/mod.rs:25-56`](../src/c2x2/mod.rs)
- **問題 1（UI と WASM の完成判定が標準向き固定）**:
  - 2x2 の Optimal ソルバーは、「標準の向きに揃える」が OFF（`include_orientation = false`）のとき、`DBL` ピースがスロット 6・向き 0 に位置する空間回転を目標完成状態とします。そのため、`D`, `L`, `B` 回転を含むスクランブル（例: `D` 1手や `D R U`）を Optimal（向き OFF）で解くと、解適用後の最終盤面は `UUUUFFFFLLLLDDDDBBBBRRRR` のような**空間回転された完成状態**（各面 4 マスはすべて単色だが `SOLVED_2X2 = "UUUURRRRFFFFDDDDLLLLBBBB"` とは異なる文字列）になります。
  - しかし [`web/main.ts:394`](../web/main.ts) は `const isSolved = state === solved;`（`SOLVED_2X2` との単純比較）で完成判定を行っているため、以下の不整合が発生します：
    1. 向き OFF で求めた最短解（例: `D` に対する `U'`）を手順入力で適用（`applyAlgorithm`）したり、手順再生の最終ステップで解法パネルを閉じたりすると、3D キューブ上は 6 面すべて綺麗に揃っているにもかかわらず、ステータスバッジ（`#cube-status`）が **「スクランブル状態」** と表示され、完成時の効果音（`sound.playSuccess()`）も鳴りません。
    2. その状態で「解法を探す」ボタンを押すと、ソルバーは「0 手・すでに6面が揃っています。」と返す一方、UI のステータス表示は「スクランブル状態」（または「0 / 0 手」）のまま食い違います。
    3. [`src/lib.rs:429`](../src/lib.rs) の `is_solved_core` / `validate_core` も `c2x2::is_solved(state, true)` と `true` をハードコードしているため、`wasm.is_solved("UUUUFFFFLLLLDDDDBBBBRRRR")` および `wasm.validate("UUUUFFFFLLLLDDDDBBBBRRRR")` が `false` を返します。
- **問題 2（LBL / Ortega が 1 手以上の局面で `include_orientation = false` を無視）**:
  - [`docs/review-fixes-e0a282e-fixes-2026-10-09.md`](review-fixes-e0a282e-fixes-2026-10-09.md) の L25 では「2x2 の LBL / Ortega において、DBL コーナー基準の向きを追従・正規化する仕様を明文化」と報告されていますが、実際の [`src/c2x2/mod.rs:25-56`](../src/c2x2/mod.rs) では `is_already_solved`（0 手判定）にしか向き OFF が反映されておらず、`lbl::solve_lbl(&rc)` と `ortega::solve_ortega(&rc)` には未正規化の `rc` がそのまま渡されています。
  - **実測**: 空間回転された完成状態 `FFFFUUUURRRRBBBBDDDDLLLL` に `U` を 1 手かけた盤面を `include_orientation = false` で解くと、`optimal` は `["U'"]`（1手）を返すのに対し、`lbl` は `["D", "L", "R'", "F2"]`（4手）、`ortega` は `["D'", "L'", "R", "U2", "D2"]`（5手）を返し、空間全体の持ち替えに相当する無駄な手順を生成します。
- **修正案**:
  1. `web/model-2x2.ts`（または WASM 境界）に「2x2 の各面 4 マスが単色で揃っているか（向き不問の完成判定）」を判定する関数を用意し、2x2 で `includeOrientation` が OFF のとき（または空間回転完成状態のとき）に `#cube-status`（「完成状態（回転向き）」または「完成状態」）および効果音判定が正しく機能するようにする。
  2. `src/c2x2/mod.rs` において、`include_orientation == false` のときは `lbl` / `ortega` でも `search.rs` と同じ `DBL` 固定の空間回転 `rot` を `rc` に乗じてから `solve_lbl` / `solve_ortega` を呼び出す。

---

#### M5【Medium・コード上確認】2x2 Ortega 法 `solve_first_face` の BFS 深さ上限拡張漏れ
- **該当箇所**: [`src/c2x2/ortega.rs:90, 117`](../src/c2x2/ortega.rs)
- **問題**:
  ```rust
  while let Some((cur, moves)) = q.pop_front() {
      if moves.len() >= 6 { // ← :90 は 5 から 6 に変更された
          break;
      }
      ...
              if moves.len() + 1 < 5 { // ← :117 が < 5 のまま！
                  let key = next_cube.hash_first_face();
                  if visited.insert(key) {
                      let mut next_moves = moves.clone();
                      next_moves.push(m);
                      q.push_back((next_cube, next_moves));
                  }
              }
  ```
  `:117` の `if moves.len() + 1 < 5` により、長さが 4 以下の `next_moves` しかキュー `q` に積まれないため、`q.pop_front()` で取り出される `moves.len()` は最大でも 4 にしかなりません。結果として `:90` の `if moves.len() >= 6` は到達不能なデッドコードであり、BFS の実質的な探索深さ上限は 5 手のまま変わっていません。
- **修正案**:
  - `:117` を `if moves.len() + 1 < 6` に修正する。

---

### 3.3 Web 状態管理・URL パラメータ・永続化（`web/main.ts`, `web/cube-store.ts`, `web/url-params.ts`）

#### M2【Medium・確認済み】不正な URL `state` パラメータによる既存保存状態の破壊と `?centers=` 単独指定の無視
- **該当箇所**:
  - [`web/main.ts:1275-1356`](../web/main.ts)
  - [`web/url-params.ts:27-60`](../web/url-params.ts)
- **問題**:
  1. **バリデーション前の `setCubeType` と `persist()` による保存状態破壊**:
     - `parseUrlParams` は `state` パラメータが 24 文字または 54 文字であれば、内容が不正（例: `?state=UUUUUUUUUUUUUUUUUUUUUUUU`）でも `result.cubeType = "2x2"` を設定します。
     - [`web/main.ts:1282-1287`](../web/main.ts) は `validate(parsedParams.state)` を呼ぶ**前**に `if (targetType)` で `store.setCubeType(targetType)` を実行し、`paramsApplied = true` にします。
     - そのため、`localStorage` に 3x3 の作業盤面を保存していたユーザーが、不正な 24 文字の `?state=...` を含む URL を開くと、`store.setCubeType("2x2")` によって復元済みの 3x3 盤面が `SOLVED_2X2` にリセットされ、直後の `validate(parsedParams.state)` が例外を投げて「URLパラメータで指定された盤面または手順が無効です。」と表示されるにもかかわらず、`paramsApplied === true` のため `:1354` の `persist()` が実行され、**`localStorage` の保存データが `SOLVED_2X2` で上書き消去**されます。
  2. **不正な文字数の `state`・不正な `type` / `solver` の黙殺**:
     - `?state=UUUU`（24/54 文字以外）や `?solver=unknown`、`?type=4x4` が指定された場合、`parseUrlParams` はエラーフラグを立てずに無視するため、エラーメッセージも表示されず、アドレスバーからのクエリ除去も行われません。
  3. **`?centers=` 単独指定（および `?alg=...&centers=...`）の無視**:
     - [`README.md:190`](../README.md) の共有 URL の例には `?centers=0,1,0,3,0,0` が掲載されていますが、[`web/main.ts:1302`](../web/main.ts) は `if (parsedParams.state)` のブロック内でしか `parsedParams.centers` を参照していません。そのため、`?centers=0,1,0,3,0,0` や `?alg=R_U&centers=...` を開いてもセンター向きが一切反映されません。
- **修正案**:
  - `parsedParams.state` が指定されている場合は、`validate(parsedParams.state)` および `centersFromInput` の検証が通ってから `store.setCubeType` と `store.replace` を実行する。
  - `parseUrlParams` で未知の `type` / `solver` や 24/54 文字以外の `state` に対しても不正パラメータフラグを設定する。
  - `parsedParams.state` が無く `parsedParams.centers` が指定されている場合も、現在の 3x3 盤面に対して `centersFromInput(store.getState(), parsedParams.centers)` を適用する。

---

#### M6【Medium・コード上確認】キューブ種別切替とプレビュー終了時の履歴・盤面仕様の不整合（修正レポートとの乖離）
- **該当箇所**:
  - [`web/cube-store.ts:49-66, 145-152, 242-248`](../web/cube-store.ts)
  - [`web/main.ts:207-219, 986-996`](../web/main.ts)
- **問題**:
  1. **修正レポートの記述と実装の乖離**:
     - [`docs/review-fixes-e0a282e-fixes-2026-10-09.md`](review-fixes-e0a282e-fixes-2026-10-09.md) §2.1 には「`CubeStore` に種別ごとの盤面・履歴退避マップ（`typeHistoryMap`）を実装。種別切替時に現在の盤面と履歴を保存し、切替先の退避履歴を安全にリストア」と記載されていますが、実際の [`web/cube-store.ts`](../web/cube-store.ts) には `typeHistoryMap` は存在しません。
     - 実装では `setCubeType(type, true)` が単一の `history` スタックに切替前のスナップショットを `pushHistory` し、盤面を `getSolvedState(type)`（完成状態）にリセットしています。そのため、ヘッダーの「3×3×3」「2×2×2」ボタンで行き来すると、切替先の盤面は毎回完成状態にリセットされ、切替前に持っていた Redo 履歴（`future`）も `pushHistory` によって失われます。
  2. **プレビュー中に「解法を閉じる」（`#solution-close`）を押した際の状態コミット**:
     - 解法シーク中（`step > 0`）の盤面について、`persist()`・「保存」・「共有リンク」・再探索（`solve()`）はいずれも `baseSnapshot`（探索開始時の確定盤面）を正として扱います。
     - しかし `#solution-close`（[`web/main.ts:986-996`](../web/main.ts)）は `store.setSolution(undefined)` を呼び、その内部で `commitBaseSnapshotIfPreviewing()` が走るため、「解法を閉じる」を押した瞬間に**シーク途中のプレビュー盤面が確定盤面として履歴にコミットされ、Redo 履歴も消去される**という非対称な挙動になっています。
- **修正案**:
  - `CubeStore` に種別ごとの盤面・履歴退避マップを実装して種別ボタンの切り替えで作業中の盤面と Undo/Redo 履歴を維持できるようにするか、単一履歴スタック方式とする場合はドキュメントを実装に合わせて修正する。
  - `#solution-close` 押下時は `store.restoreBaseSnapshot()` で探索前の確定盤面に戻すか、UI 上で「現在のステップの盤面をそのまま残す／元の盤面に戻す」の整合性を取る。

---

### 3.4 PWA / Service Worker（`public/sw.js`）

#### M3【Medium・確認済み】Service Worker のクエリ付きキャッシュ上限管理の破綻と旧キャッシュ世代ソートの不具合
- **該当箇所**: [`public/sw.js:47-48, 68-71, 200-209`](../public/sw.js)
- **問題**:
  1. **Service Worker 再起動による `queryKeysQueue` の揮発**:
     ```javascript
     const MAX_QUERY_ENTRIES = 50;
     const queryKeysQueue = [];
     ```
     Service Worker はイベント駆動のワーカーであり、ブラウザによって数十秒のアイドル後に終了・再起動されます。再起動するとメモリ上の `queryKeysQueue` は空配列 `[]` に戻る一方、ディスク上の `CacheStorage`（`ownCache`）には過去のクエリ付きエントリが残り続けるため、上限 50 件の制限が再起動を跨いで機能しません。
  2. **同一 URL の重複 push による自己削除バグ**:
     ```javascript
     if (url.search) {
       queryKeysQueue.push(url.href);
       if (queryKeysQueue.length > MAX_QUERY_ENTRIES) {
         const oldest = queryKeysQueue.shift();
         if (oldest) await ownCache.delete(oldest);
       }
     }
     const boundedCacheKey = url.search ? url.href : canonicalKey;
     await ownCache.put(boundedCacheKey, clone);
     ```
     `queryKeysQueue` にすでに同じ `url.href` が含まれているかをチェックせずに毎回 `push` しているため、たとえば少数のクエリ付きアセット（`cube_studio_bg.wasm?url` 等）が繰り返しフェッチされると、`queryKeysQueue` が同一 URL で埋め尽くされます。その結果、実質 2〜3 種類の URL しかキャッシュしていないのに 51 回目のアクセスで `ownCache.delete(oldest)` が発火し、既存の有効なキャッシュエントリを次々に誤削除します。
  3. **SHA-256 ハッシュ文字列に対する辞書順ソート**:
     `activate` ハンドラ（`:68-71`）で直近 1 世代のキャッシュを選ぶ際、`a.localeCompare(b, undefined, { numeric: true })` としていますが、本番ビルドの `CACHE_VERSION` は `15ffacd280` のような SHA-256 ハッシュ先頭 10 文字です。ハッシュ文字列の大小はビルド順序と無関係であるため、旧キャッシュが複数ある場合に直近世代を正しく特定できません。
- **修正案**:
  - グローバル配列 `queryKeysQueue` を廃止し、クエリ付きリクエストを `ownCache.put` した後に `const reqs = (await ownCache.keys()).filter((r) => new URL(r.url).search);` を取得し、`reqs.length > MAX_QUERY_ENTRIES` の分だけ先頭から `await ownCache.delete(reqs[i])` する（Cache Storage の `keys()` は挿入順を保持するため、再起動後も正確に FIFO 削除が可能）。
  - `generate-sw-precache.js` で生成する `CACHE_VERSION` を `${ timestamp }-${ contentHash }` 形式にするか、旧キャッシュからのマイグレーション処理を整理する。

---

### 3.5 ガードレール・ビルドスクリプト・テスト基盤（`scripts/`, `tests/`, `start.sh`, `playwright.config.ts`）

#### M7【Medium・確認済み】ガードレールチェッカーの検査抜け・フィクスチャ未整備と `coverage.spec.ts` のモック引数残存
- **該当箇所**:
  - [`scripts/check-review-guardrails.js:13, 173-176, 228-238, 282-313`](../scripts/check-review-guardrails.js)
  - [`tests/scripts-regression.spec.ts:120-189`](../tests/scripts-regression.spec.ts)
  - [`tests/coverage.spec.ts:604-609`](../tests/coverage.spec.ts)
  - [`start.sh:51-62`](../start.sh)
  - [`playwright.config.ts:41-47`](../playwright.config.ts)
- **問題**:
  1. **コメントで宣言されたフィクスチャの不在**:
     `scripts/check-review-guardrails.js:13` には「`すり抜けパターンは tests/fixtures/guardrails/ に置き、tests/scripts-regression.spec.ts で検出を回帰テストする`」と明記されていますが、`tests/fixtures/` ディレクトリは存在せず、`tests/scripts-regression.spec.ts` も F1 の単純な 1 ケースしかテストしていません。
  2. **F5（Service Worker 上限管理）検査の形骸化**:
     `checkServiceWorkerCache`（`:228-238`）は `!content.includes("MAX_QUERY_ENTRIES") || !content.includes(".delete(")` とファイル全体の部分文字列しか検査していません。`public/sw.js` には `activate` イベント内に `caches.delete(...)` が常にあるため、ランタイムキャッシュの `ownCache.delete(oldest)` を完全に削除してもガードレール検査をパスしてしまいます。
  3. **F3（ファイルサイズ事前検査）の緩い判定**:
     `checkFileReadOrder`（`:173-176`）は直前 15 行に `.size` という文字列が含まれるかしか見ていないため、サイズ上限によるガード条件でなくても通過します。
  4. **F7（成果物鮮度）の対象漏れ**:
     - `start.sh:51-62` の `needs_wasm_build()` は `Cargo.toml`, `Cargo.lock`, `src/` のタイムスタンプしか比較しておらず、`build.rs` の更新を検知しません。
     - `playwright.config.ts:41-47` の `webServer[1]`（`npm run preview -- --port 4173 --strictPort --base /nested/cube/`）は、`npm test` を単独実行した際に `dist/` の鮮度検証（`checkInputsFreshness`）を行わずに古い `dist/` を配信します（`.ts` ファイルのため `checkArtifactFreshness` の走査からも漏れています）。
  5. **`coverage.spec.ts` におけるモックサイズ引数の残存**:
     [`docs/review-fixes-e0a282e-fixes-2026-10-09.md`](review-fixes-e0a282e-fixes-2026-10-09.md) §2.5 では「有効な JSON 構造を保持したままパディングして 70,000 bytes に肥大化させたペイロードを渡し」と記載されていますが、実際の [`tests/coverage.spec.ts:604-609`](../tests/coverage.spec.ts) は以下のとおりです：
     ```typescript
     const validJson = JSON.stringify({ version: 1, state: solved });
     // 有効な JSON でもサイズ上限 (64KB) を超える場合は確実に 64KB エラーとなること
     assertThrows(
       () => fileIo.validateAndParseCubeJson(validJson, 70000),
       /64KB/,
     );
     ```
     `validJson` は 72 バイトのままであり、第 2 引数にだけ `70000` という実ファイルサイズと乖離したモック数値を渡しています（`CLAUDE.md` §4 が禁止するパターン）。
- **修正案**:
  - `tests/scripts-regression.spec.ts` に F1, F3, F4, F5, F6, F7 すべての違反パターンに対する検出テストを追加する。
  - `checkServiceWorkerCache` では `fetch` ハンドラ内の `boundedCacheKey` の直近スコープで `ownCache.delete` が呼ばれていることを検証する。
  - `start.sh` の `needs_wasm_build()` に `[ "build.rs" -nt "pkg/cube_studio_bg.wasm" ]` を追加する。
  - `tests/coverage.spec.ts:604-609` では実際に 65,536 バイトを超える文字列（または `File` オブジェクト）を生成して `file.size` と共に検証する。

---

### 3.6 Low 指摘事項（L1〜L7）

- **L1【Low・確認済み】`Undo` / `Redo` 実行時および 2x2 プリセット適用時の `#scramble-text` 表示残り**
  - **箇所**: [`web/main.ts:912-926, 1475`](../web/main.ts)
  - **内容**: スクランブルや 2x2 プリセット適用で `#scramble-text` が書き換わった後、`#undo` / `#redo` を押して完成状態（または別種別）に戻しても `#scramble-text` が `DEFAULT_SCRAMBLE_TEXT` に戻らず、直前のスクランブル手順が表示されたままになります。また `:1475` で 2x2 の「完成状態」プリセットを押した際は `$("scramble-text").textContent = "";` と空文字にしており、`replace()` が設定した `DEFAULT_SCRAMBLE_TEXT` を消してしまっています。
  - **修正案**: `undo` / `redo` 時も `#scramble-text` を `DEFAULT_SCRAMBLE_TEXT` にリセットし、`:1475` の空文字代入を削除する。

- **L2【Low・コード上確認】`switchCubeType` における UI 更新・3D 再構築・永続化の二重実行**
  - **箇所**: [`web/main.ts:207-219, 533-545`](../web/main.ts)
  - **内容**: `switchCubeType` 内で `store.setCubeType(type, true)` を呼ぶと、`store.subscribe` の `"cube-type"` リスナーが同期的に発火して `updateCubeTypeUI`・`scene?.setCubeType`・`persist`・`refresh` を実行しますが、その直後に `switchCubeType` 本体でも同じ 4 つの処理を再度呼び出しています。
  - **修正案**: `store.subscribe` 側に一本化し、`switchCubeType` 内の重複呼び出しを削除する。

- **L3【Low・コード上確認】カメラ 2x2 手動補正時の `?` 選択でエラーが消えない問題と `rawQuads` デッドコード**
  - **箇所**: [`web/camera.ts:892-903, 957-971`](../web/camera.ts)
  - **内容**:
    1. `validate2x2Faces()` で整合性エラーが表示された後、ユーザーがパレットで `?`（未入力）を選んでセルを修正しようとすると、`:960` の `if (state.includes("?")) return;` で早期リターンするため、古いエラーメッセージが画面に残り続けます（`this.error("")` でクリアすべき）。
    2. `capture()` 内の `rawQuads`（`:892-903`）はどこからも参照されていないデッドコードです（直後の `faceDefs` と重複）。

- **L4【Low・確認済み】3x3 Thistlethwaite の Phase 3 枝刈り不在によるフォールバック率（継続課題）**
  - **箇所**: [`src/c3x3/thistlethwaite.rs:55, 363-392`](../src/c3x3/thistlethwaite.rs)
  - **内容**: Phase 3（$G_2 \to G_3$）にヒューリスティック枝刈りがないため、ランダム 20 局面（予算 3,000ms）のうち 8 局面（40%）で予算の 70% を消費した後に Kociemba フォールバックとなります。G2→G3 のコーナー・エッジ軌道に関する小規模な枝刈り表（またはパリティ・テトラッド不一致数の下界）を入れることで純粋な Thistlethwaite 解の成功率と応答速度を大きく改善できます。

- **L5【Low・コード上確認】Rust 側の未使用コード（`c2x2::tables::MoveTable` ほか）**
  - **箇所**: [`src/c2x2/tables.rs:8-28`](../src/c2x2/tables.rs)、[`src/c3x3/search.rs:52`](../src/c3x3/search.rs)
  - **内容**: `c2x2::tables::MoveTable`（CP 40,320 × 18 と Twist 2,187 × 18）はどこからも呼ばれていません（`initialize()` からは外されていますが、構造体と生成関数が残っています）。また `c3x3::search::Search::with_max_total` も未使用です。

- **L6【Low・コード上確認】`tsconfig.json` の型チェック対象範囲**
  - **箇所**: [`tsconfig.json:12`](../tsconfig.json)
  - **内容**: `"include": ["web", "vite.config.ts"]` となっているため、`tests/*.ts` や `playwright.config.ts` の型エラーが `npm run typecheck` で検出されません。

- **L7【Low・確認済み】リポジトリ衛生面の整理項目**
  - **箇所**: ルートディレクトリ、[`README.md:315`](../README.md)
  - **内容**:
    - ルートの `cubes/` ディレクトリと `public/cubes/` ディレクトリが同一内容で二重管理されています。
    - `coverage-tarpaulin/`, `coverage-tarpaulin-new/`, `coverage-tarpaulin-final/`, `coverage.lcov`, `coverage*.txt`, `COVERAGE_ANALYSIS.md`, `COVERAGE_E2E.md`, `CAMERA_INPUT_TESTS.md`, `CAMERA_TEST_IMAGES.md` など、初期検証時の生成物・古いレポートがルートに残っています。
    - `README.md:315` の Rust 単体テスト件数が `154 件`（実際は `155 件`）となっています。

---

## 4. 推奨する修正の優先順位

1. **最優先（H1）**: [`src/c3x3/search.rs:194`](../src/c3x3/search.rs) の `self.max_total == 30` を `self.max_total >= 30 || self.target_centers.is_some()` に修正し、重複する `total = 24` ループと `src/lib.rs` の相殺前 `seq_len` を是正する（わずか数行の修正で 3x3 センター込み探索の 47% フォールバック退行と数秒の待ち時間が解消します）。
2. **高優先（M1, M5）**: 2x2 の空間回転された完成状態の判定統一（UI・WASM・LBL/Ortega の DBL 基準正規化）と、Ortega `solve_first_face` `:117` の `< 6` 修正。
3. **中優先（M2, M3, M4, M6）**: URL パラメータのバリデーション順序修正と `?centers=` 単独指定対応、Service Worker の `ownCache.keys()` ベースの上限管理への移行、3x3 全ソルバーへの `cancel_redundant_moves_with_phases` 適用、`CubeStore` の種別切替・プレビュー終了仕様の統一。
4. **品質基盤・衛生（M7, L1〜L7）**: ガードレール回帰テストの拡充、`start.sh` の `build.rs` 鮮度チェック追加、軽微な UI・デッドコード整理。
