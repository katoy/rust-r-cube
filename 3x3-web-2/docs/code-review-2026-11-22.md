# 全体コードレビューレポート：実行速度・コード構造・むやみな複雑さ除去（2026-11-22）

- **実施日**: 2026-11-22（JST）
- **対象リポジトリ**: `rust-r-cube/3x3-web-2`（Cube Studio）
- **重点観点**:
  1. **実行速度（Performance）**: 初期ロード・WASM メモリ・探索ノード数／時間・DOM/3D 描画・カメラ認識・SW キャッシュ
  2. **コードの構造（Code Structure / Architecture）**: モジュール責務分離・Single Source of Truth・WASM API 境界・テスト衛生
  3. **むやみな複雑さの除去（Removing Unnecessary Complexity）**: 過剰抽象・不要なワークアラウンド・対称テーブル逆転・重複ロジック・デッドコード
  4. **見落とし防止補完（Correctness & Remaining Hardening）**: 機能バグ・エッジケース・仕様不整合・2026-11-21 指摘のうち未着手のハードニング項目
- **対象範囲**:
  - Rust コア・ソルバー・WASM 境界（`src/lib.rs`, `src/c3x3/*`, `src/c2x2/*`, `build.rs`, `Cargo.toml`）
  - Web フロントエンド・Web Worker・3D・カメラ・PWA（`web/*.ts`, `web/style.css`, `public/sw.js`, `public/manifest.webmanifest`, `index.html`, `vite.config.ts`）
  - ビルド・ガードレール・スクリプト・テスト群（`package.json`, `scripts/*`, `start.sh`, `playwright.config.ts`, `tests/*.spec.ts`）

---

## 0. エグゼクティブサマリー

### 0.1 総合判定

| 観点 | High | Medium | Low | 合計 | 主な改善ポテンシャル |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **実行速度 (`PERF`)** | 4 | 5 | 1 | **10** | 3x3 探索 **10〜30 倍高速化**（Superflip 3.09s → &lt;0.3s）、Thistlethwaite **812ms → &lt;1ms**、WASM gzip **3.48MB → ~0.75MB（約 78% 削減）**、WASM 合計メモリ **~18.6MB → ~4MB**、毎手不要 DOM 再構築の撤廃 |
| **コード構造 (`ARCH`)** | 3 | 3 | 1 | **7** | `web/main.ts`（1,988 行）/`web/camera.ts`（1,444 行）分割、二重/三重状態管理と重複 `cubes/` 解消、WASM 境界 14 → 8 API 整理・メイン WASM 分離 |
| **複雑さ除去 (`SIMP`)** | 4 | 3 | 1 | **8** | 過剰 State パターン・SW 旧キャッシュ移行・Worker 2x2 迂回・対称マップ・Rust コピペ重複・デッドコード一掃で **約 800 行超を純削減** |
| **正確性・補完 (`CORR`)** | 2 | 3 | 2 | **7** | 新規発見バグ（`countPieceOrientations` 誤算出、`compareAllSolvers` 連鎖死、`depth >= max_total` off-by-one、`?centers=,,,,,` 誤受理 等）＋未着手ハードニング（S4/S6/S7/S8/S9） |
| **合計** | **13** | **14** | **5** | **32** | — |

### 0.2 2026-11-21 レビューからの進捗（既修正項目・誤指摘防止）

前回の [`docs/code-review-2026-11-21.md`](code-review-2026-11-21.md) 以降、以下の改善がすでに本番コードへ取り込まれており、本レポートでは**解決済み**として未修正項目から除外しています。

- **解決済み（速度系）**:
  - `initialize()` からの 2x2 `PruningTable` 遅延初期化（[`src/lib.rs:532`](../src/lib.rs#L532)）→ `initialize()` 起動時間は **179.5 ms → 8.57 ms** へ短縮済み。
  - 2x2 Optimal（向き OFF）の DBL 固定＋`<U, R, F>` 3 面限定（[`src/c2x2/search.rs:84`](../src/c2x2/search.rs#L84)）→ Antipode 11 手が **310.7 ms → 1.62 ms（191 倍高速化）**。
  - 2x2 `get_all_group_indices` による 16 回線形探索の逆置換統合（[`src/c2x2/coord.rs:255`](../src/c2x2/coord.rs#L255)）。
  - 2x2 LBL 第 1 層の `PruningTable::group_b()` 活用（[`src/c2x2/lbl.rs:84-118`](../src/c2x2/lbl.rs#L84-L118)）→ ランダム 50 局面平均 **27.4 ms → 0.009 ms（約 3,000 倍高速化）**。
  - 2x2 LBL / Ortega の定石 `OnceLock` 事前パース（[`src/c2x2/lbl.rs:171,237`](../src/c2x2/lbl.rs#L171), [`src/c2x2/ortega.rs:156,240`](../src/c2x2/ortega.rs#L156)）。
  - 3x3 Kociemba の `has_acceptable_solution()` 早期終了、`cp/ep/sp` の `for d` ループ外化＋`start_d = min_d.max(h2)`、UD `depth == 1` の一部枝刈り、`#[inline(always)]`（[`src/c3x3/search.rs:86,390-425`](../src/c3x3/search.rs#L86), [`src/c3x3/tables.rs:128,141`](../src/c3x3/tables.rs#L128)）→ 3x3 Kociemba 色のみ平均が **371.4 ms → 159.98 ms** へ改善済み。
  - `refresh()` の非表示 `#fallback-net` スキップ・`lastSolveLabel` / `lastPlayIcon` キャッシュ（[`web/main.ts:579,602,703`](../web/main.ts#L579)）。
  - `scene.ts` のスクラッチベクトル化・ステッカー `castShadow = false`・`dirty` フラグによる描画スキップ（[`web/scene.ts:57-74,463,501,761`](../web/scene.ts#L57-L74)）。
- **解決済み（セキュリティ系）**:
  - 本番ビルド CSP メタタグ（[`vite.config.ts:3-38`](../vite.config.ts#L3-L38)）および `Referrer-Policy: no-referrer`（[`index.html:6`](../index.html#L6)）。
  - `window.__lastCamera`・`window.cube_store`・`window.__cube_main_debug__` の `import.meta.env.DEV` 限定化（[`web/camera.ts:500`](../web/camera.ts#L500), [`web/main.ts:49-55,808,1595,1935`](../web/main.ts#L49-L55)）。
  - `readJpegDimensions` のマーカー数・走査バイト・連続 `0xFF` 上限ガード（[`web/camera.ts:39-117`](../web/camera.ts#L39-L117)）。
  - Service Worker の `scopePrefix` 検証・`status === 200` / `type === "basic" | "default"` 制限・`?no-sw` の開発/自動テスト限定化（[`public/sw.js:191-230`](../public/sw.js#L191-L230), [`web/pwa.ts:17-22`](../web/pwa.ts#L17-L22)）。

### 0.3 今回新たに発見した重大な死角（2026-11-21 レビューで未検出）

1. **`direct_solve` の無枝刈り 67.5 万ノード空回し（`PERF-01`）**: 距離 6 以上のほぼ全局面（および T-perm 等の G1 局面）で深さ 1..=5 を枝刈りゼロで全探索し、毎回 **674,746〜676,449 ノード（~8.3 ms）**を浪費（T-perm 全体の 99.99% が無駄）。
2. **`refresh()` 毎のプリセット DOM 全破棄再構築＆アニメ 1 手で `getCellArrowInfo` 3 重実行（`PERF-04`）**: `store.subscribe` が毎手 `updateCubeTypeUI()` → `renderPresetButtons()` を呼び出し、1 手のシーク／アニメで `getCellArrowInfo` が 3 回走る。
3. **`countPieceOrientations` の数学的誤算出バグ（`CORR-01`）**: `U` 1 手回転（真値：コーナーねじれ 0・エッジ反転 0）で「角ねじれ 1・辺反転 2」と誤表示する。
4. **`compareAllSolvers` タイムアウト後の連鎖失敗バグ（`CORR-02`）**: 1 アルゴリズムのタイムアウト時に `SolverClient.fail()` が Worker を破棄したまま次のアルゴリズムが走り、以降すべて即エラーになる。
5. **`search.rs` `depth >= self.max_total` の off-by-one（`CORR-03`）**: `max_total == 20` 時に深さ 20 を探索せずスキップし、Doc コメントの完全性保証と矛盾。
6. **`url-params.ts` の `?centers=,,,,,` 誤受理（`CORR-04`）**: `Number("") === 0` により空要素が `0` として通る。
7. **`solver.worker.ts` 50 行の 2x2 迂回重複（`SIMP-03`）** と **`web/app-state.ts` 302 行の過剰 State パターン（`SIMP-01`）**・**`tables.rs` x2 対称マップの構造逆転（`SIMP-02`）**。

---

## 1. 現行ベースライン実測データ（2026-11-22 時点）

### 1.1 静的検査・ビルド状態

| 検査項目 | コマンド | 結果 | 備考 |
| :--- | :--- | :---: | :--- |
| レビューガードレール | `npm run check:guardrails` | **PASS** | F1〜F7 静的検査すべて合格 |
| TypeScript 型検査 | `npm run typecheck` | **PASS** | `tsc --noEmit`（TS 7.0.2）エラー 0 |
| Prettier フォーマット | `npm run format:check` | **PASS** | 全対象ファイル適合 |
| Rust Clippy / Test | `cargo clippy --offline ...` | **環境制約** | サンドボックス内の Cargo インデックス参照制限により未実行（ソースは全行精読） |

### 1.2 ビルド成果物サイズ（`pkg/`・`dist/` 実測）

| 成果物 | Raw サイズ | Gzip サイズ | 備考 |
| :--- | ---: | ---: | :--- |
| `pkg/cube_studio_bg.wasm` | **6,357,460 B** (6.06 MiB) | **3,485,474 B** (3.32 MiB) | メイン・Worker で二重ロード |
| `dist/assets/cube_studio_bg-*.wasm` | **6,357,460 B** (6.06 MiB) | **3,485,483 B** (3.32 MiB) | うち `tables.bin`（約 6.11 MB）が約 95% |
| `dist/assets/three-*.js` | 577,991 B (564 KiB) | — | 本番唯一の外部 JS 依存 |
| `dist/assets/index-*.js` | 103,774 B (101 KiB) | — | メインスレッド JS＋WASM グルー |
| `dist/assets/camera-*.js` | 32,261 B (31.5 KiB) | — | 動的インポート |
| `dist/assets/index-*.css` | 29,067 B (28.4 KiB) | — | `web/style.css`（2,054 行） |
| `dist/assets/solver.worker-*.js` | 6,790 B (6.6 KiB) | — | Web Worker |
| `dist/assets/editor-*.js` | 5,603 B (5.5 KiB) | — | 動的インポート |

### 1.3 本番 WASM ソルバー実測（Node.js v20.20.2 + `pkg/cube_studio_bg.wasm`）

- **WASM インスタンス化**: **13.86 ms**
- **`initialize()`**: **8.57 ms**（3x3 テーブルデコード＋6.11 MB FNV-1a 検証）

| 対象 / 条件 | 局面数 | 平均時間 | 中央値 / 最大時間 | 平均ノード数 | 平均手数 / 最大手数 | 備考 |
| :--- | :---: | ---: | ---: | ---: | :---: | :--- |
| 3x3 Kociemba（色のみ・予算 3s） | 20 | **159.98 ms** | 106.78 / 687.31 ms | 12,733,462 | 19.8 / 20 | エラー 0・20 手達成 |
| 3x3 Kociemba（センター向き込み・予算 3s） | 20 | **906.55 ms** | 430.63 / **3,000.95 ms** | **85,371,854** | 35.1 / **90** | **4/20（20%）が逐次解へフォールバック** |
| 3x3 Superflip（色のみ） | 1 | **3,089.11 ms** | — | **279,551,516** | 20 | `gods_guarantee` 延長完走 |
| 3x3 Superflip（センター向き込み） | 1 | **1,591.58 ms** | — | **147,797,421** | 23 | 同時最適化 |
| 3x3 CFOP（色のみ） | 20 | 2.26 ms | 2.54 / 4.92 ms | 156,846 | 108.3 / 134 | 高速 |
| 3x3 Thistlethwaite（色のみ・予算 3s） | 20 | **811.99 ms** | 811.47 / 2,117.79 ms | **45,969,409** | 29.8 / 33 | **3/20（15%）がフォールバック** |
| 3x3 Korf IDA*（色のみ・予算 3s） | 20 | **684.99 ms** | 646.03 / 1,378.27 ms | **34,558,782** | 19.8 / 20 | **20/20（100%）がフォールバック** |
| `direct_solve` 段差：4 手 (`R U R' F'`) | 1 | 0.49 ms | — | 21,624 | 4 | Kociemba 直接解決 |
| `direct_solve` 段差：5 手 (`R U R' F' L`) | 1 | **7.48 ms** | — | **559,240** | 5 | 無枝刈りで 25 倍増 |
| `direct_solve` 段差：6 手 (`R U R' F' L D`) | 1 | **8.48 ms** | — | **676,449** | 13 | うち **~675k が空振り** |
| `direct_solve` 段差：T-perm（G1 14 手） | 1 | **8.27 ms** | — | **674,746** | 11 | **99.99% が空振り** |
| 2x2 Antipode 11 手（初回 BFS 込み） | 1 | **119.13 ms** | — | 55,582 | 11 | 遅延 BFS が ~117.5 ms |
| 2x2 Antipode 11 手（ウォーム・向き OFF） | 1 | 1.62 ms | — | 55,582 | 11 | `<U, R, F>` 限定が奏功 |
| 2x2 Antipode 11 手（ウォーム・向き ON） | 1 | **106.95 ms** | — | **3,735,583** | 11 | 6 面探索で 67 倍ノード |
| 2x2 Optimal（50 局面ウォーム・向き OFF） | 50 | 0.058 ms | — / 0.184 ms | — | 8.3 | 高速 |
| 2x2 LBL（50 局面ウォーム・向き OFF） | 50 | 0.009 ms | — / 0.028 ms | — | 23.2 | `group_b` により爆速 |
| 2x2 Ortega（50 局面ウォーム・向き OFF） | 50 | **1.361 ms** | — / **6.336 ms** | — | 20.3 | **LBL の 150 倍低速** |

### 1.4 コード規模と肥大化箇所

生産コード・補助スクリプト合計 **24,921 行**（`src/tests.rs` 2,723 行含む）＋ E2E テスト **65 ファイル / 17,520 行**：
- 最大ファイル集中: [`web/style.css`](../web/style.css)（2,054 行）、[`web/main.ts`](../web/main.ts)（1,988 行）、[`web/camera.ts`](../web/camera.ts)（1,444 行）、[`src/c3x3/cfop.rs`](../src/c3x3/cfop.rs)（817 行）、[`src/c3x3/search.rs`](../src/c3x3/search.rs)（816 行）、[`web/scene.ts`](../web/scene.ts)（766 行）、[`src/lib.rs`](../src/lib.rs)（709 行）、[`tests/coverage.spec.ts`](../tests/coverage.spec.ts)（3,348 行）。

---

## 2. 実行速度（Performance）レビュー指摘事項

### 2.1 指摘サマリー（実行速度）

| ID | 重大度 | 領域 | 対象ファイル（行番号） | 要約と期待効果 |
| :--- | :---: | :--- | :--- | :--- |
| **PERF-01** | **High** | `direct_solve` 全探索 | [`src/c3x3/search.rs`](../src/c3x3/search.rs) (L126-138, L261-305) | 距離 ≥ 6 の全局面で枝刈りなし深さ 1..=5 全探索を実行し毎回 **67.5 万ノード（~8.3 ms）**浪費。座標下界枝刈りで **~0 ms** 化。 |
| **PERF-02** | **High** | Kociemba 探索本体 | [`src/c3x3/search.rs`](../src/c3x3/search.rs) (L140-204, L306-520)<br>[`src/c3x3/tables.rs`](../src/c3x3/tables.rs) (L128-153)<br>[`src/lib.rs`](../src/lib.rs) (L282-365) | UD 単独深さ 12 先行、センター向き時 6 方向探索未適用、途中 G1 枝刈り不足、`exhausted()` 毎ノード判定、`RawCube::multiply` 全手再生、4 配列間接参照。**10〜30 倍高速化（Superflip 3.09s → &lt;0.3s、センター向きタイムアウトゼロ化）**。 |
| **PERF-03** | **High** | WASM サイズ＆二重ロード | [`src/lib.rs`](../src/lib.rs) (L13)<br>[`src/c3x3/tables.rs`](../src/c3x3/tables.rs) (L10-48)<br>[`src/c3x3/table_io.rs`](../src/c3x3/table_io.rs) (L8-104)<br>[`src/c3x3/coord.rs`](../src/c3x3/coord.rs) (L343-402)<br>[`web/main.ts`](../web/main.ts) (L1594, L1737)<br>[`Cargo.toml`](../Cargo.toml) (L23-25) | 6.36 MB WASM（gzip 3.48 MB）の直列二重ロード（メモリ ~18.6 MB）、Phase 2 テーブル未使用 44.4%、8-bit 枝刈り表、FNV-1a 全走査、`set_cp`/`set_ep8` の 4 万回ヒープ確保。**転送最大 78% 減・メモリ 75% 減**。 |
| **PERF-04** | **High** | UI 更新・`getCellArrowInfo` | [`web/main.ts`](../web/main.ts) (L214, L217-230, L571-584, L610-614, L664, L775-784)<br>[`web/scene.ts`](../web/scene.ts) (L703)<br>[`web/model.ts`](../web/model.ts) (L212-408) | 毎手 `renderPresetButtons()` DOM 全破棄再生成、アニメ 1 手で `getCellArrowInfo` 3 重実行＆`scene.show` 2 重実行、`getCellArrowInfo` の 200+ 一時配列生成、毎 `refresh()` 広範 DOM 走査。 |
| **PERF-05** | **Medium** | Thistlethwaite / Korf | [`src/c3x3/thistlethwaite.rs`](../src/c3x3/thistlethwaite.rs) (L182-447)<br>[`src/c3x3/korf.rs`](../src/c3x3/korf.rs) (L58-167) | Thistlethwaite（平均 812 ms / 15% 失敗）を `MoveTable`＋小型テーブルで **&lt; 1 ms** 化。Korf（100% 失敗前 685 ms 浪費）を 3 軸初期下界＋座標探索化。 |
| **PERF-06** | **Medium** | 2x2 ソルバー群 | [`src/c2x2/ortega.rs`](../src/c2x2/ortega.rs) (L78-142, L262-301)<br>[`src/c2x2/search.rs`](../src/c2x2/search.rs) (L58-60)<br>[`src/c2x2/tables.rs`](../src/c2x2/tables.rs) (L35-60)<br>[`src/c2x2/mod.rs`](../src/c2x2/mod.rs) (L73-98) | Ortega `solve_pbl` 9,216 回動的 `Vec`＆`solve_first_face` BFS（LBL の 150 倍遅い）、Optimal 向き ON（107 ms）、初回 2x2 BFS（117.5 ms）、24 向き二重検索。 |
| **PERF-07** | **Medium** | Three.js 描画ループ | [`web/scene.ts`](../web/scene.ts) (L226, L431-539, L715-765) | 静止時も 60fps `setAnimationLoop`＋`controls.update()` 継続、`finish()` の全 194 メッシュ無条件再親付け、195 ドローコール/フレーム。 |
| **PERF-08** | **Medium** | カメラ＆色編集 UI | [`web/camera.ts`](../web/camera.ts) (L276-296, L751-815, L836-845, L876, L1013-1082, L1430-1436)<br>[`web/camera-ui-helper.ts`](../web/camera-ui-helper.ts) (L13, L55)<br>[`web/editor.ts`](../web/editor.ts) (L140-253) | ホバー 1px 毎 Canvas 全再描画＆二重 `getBoundingClientRect()`、未キャッシュ 7.7 MB `getImageData` 3 重読み出し、ラベル推定 27 マス過剰走査、1600px 超二重デコード、`editor` 毎クリック 110 DOM 再生成。 |
| **PERF-09** | **Medium** | SW キャッシュ移行 / 比較直列 | [`public/sw.js`](../public/sw.js) (L95-128)<br>[`web/main.ts`](../web/main.ts) (L1096-1125) | SW `activate` で旧 6.36 MB WASM 等最大 15 件をコピー蓄積し起動 I/O 悪化。`compareAllSolvers` 単一 Worker 直列待機。 |
| **PERF-10** | **Low** | WASM 境界 JSON / バンドル | [`src/lib.rs`](../src/lib.rs) (L37-63, L476-484)<br>[`web/main.ts`](../web/main.ts) (L909, L1330, L1684, L1784, L1875)<br>[`web/scene.ts`](../web/scene.ts) (L1) | `apply_moves` が未使用 `states` 配列（最大 125 KB）を毎回 JSON 化。`import * as THREE` の名前付き整理余地。 |

---

### 2.2 実行速度 詳細と改善案

#### [PERF-01] `direct_solve` の無枝刈り深さ 1..=5 全探索（毎回 67.5 万ノード / 8.3 ms 浪費）

- **該当箇所**: [`src/c3x3/search.rs:126-138, 261-305`](../src/c3x3/search.rs#L126-L138)
- **原因と実測根拠**:
  - `Search::solve` は本探索の前に深さ `1..=5` を `direct_solve` で探索します。`direct_solve` は `RawCube::multiply` を毎回呼び、座標テーブル・枝刈りテーブルを一切参照しません（センターの非ゼロ数チェック L274-279 のみ）。
  - このため距離 6 以上の局面（ランダム 25 手スクランブルのほぼ 100%、T-perm 等）では毎回深さ 1..=5 を枝刈りゼロで完走します。
  - 実測（[`src/c3x3/search.rs`](../src/c3x3/search.rs)）で 4 手は `21,624 nodes / 0.49 ms` なのに、5 手で `559,240 nodes / 7.48 ms`、6 手で `676,449 nodes / 8.48 ms` に急増します。T-perm（G1 部分群で Phase 2 直接解決できるため本来の探索は数百ノード）でも **674,746 ノード / 8.27 ms** かかり、**探索コストの 99.99% が `direct_solve` の空振り**です。
  - さらにセンター向き指定時は [`src/lib.rs:285,348`](../src/lib.rs#L285) で色探索と同時最適化の 2 回 `solve()` が走るため、合計 **~135 万ノード（~16.5 ms）**を無条件に消費します。
- **改善案**:
  1. `direct_solve` を `twist / flip / slice`（＋Phase 1 枝刈り `get_twist_slice` / `get_flip_slice`）および `min_moves_from_cube` 下界で枝刈りする、あるいは `twist == 0 && flip == 0 && slice == 0` を判定する `Phase 1 (depth 0..=5)` へ統合する。
  2. G1 到達可能な深さ 5 以下の解は、そもそも Phase 1 の深さ `0..=5`（`phase1` + `phase2`）でもすべて発見可能であり、`direct_solve` 専用の無枝刈り木は不要です（Phase 1 の深さ 0..=5＋Phase 2 の方が圧倒的に高速）。

#### [PERF-02] Kociemba 探索の軸偏重・センター向き 6 方向未適用・Phase 1→2 再計算

- **該当箇所**: [`src/c3x3/search.rs:140-204, 306-520`](../src/c3x3/search.rs#L140-L204), [`src/c3x3/tables.rs:128-153`](../src/c3x3/tables.rs#L128-L153), [`src/lib.rs:282-365`](../src/lib.rs#L282-L365)
- **ボトルネック分析**:
  1. **UD 軸単独での深さ 12 完走先行（L140-158）**: `depth = 0..=12` を UD 軸（順方向）だけで完走してから 6 方向インターリーブ（L176-200）に入るため、UD 軸の Phase 1 深さが 11〜12 で他軸（RL / FB や逆状態）に 7〜8 手の良い G1 がある局面でも UD 軸の深い木を先に探索してしまいます。さらに Superflip（20 手）では Loop 1（`depth = 0..=12`）だけで数千万〜億ノードを消費した後に 6 方向ループへ突入し、計 **2.80 億ノード / 3.09 秒**かかっています。
  2. **センター向き指定時の 6 方向探索未適用（`search.rs:164`）**: 「センター回転の追跡を変換できないため対象外」という前提で UD 軸固定になっているため、20 局面中 **4/20（20%）が 3 秒でタイムアウトし最長 90 手の逐次解へ後退**、平均 **906.55 ms / 8,537 万ノード**を要しています。しかし `S_URF3` は右手系の純粋な空間 120° 回転であるため、6 面の添字置換（`axis = 1, 2` に応じた `[U,R,F,D,L,B]` の置換）だけで `target_centers` を共役変換でき、逆状態も `inv_centers[f] = (4 - centers[f]) & 3` で完全に同一条件が成立します（面の向きは右手系回転で時計回り＝時計回りのまま保たれます）。
  3. **Phase 1 での途中 G1 枝刈り不足（L416-425）**: `depth == 1` のときしか既に G1 へ到達済み（`twist == 0 && flip == 0 && slice == 0`）を弾いておらず、`depth >= 2` で途中から G1 に入った後に Phase 2 手だけを組み合わせて戻る経路が深さごとに重複展開されます。
  4. **`phase1` 葉での `RawCube::multiply` 全手再生（L370-373）**: Phase 1 深さ 10 の葉に到達するたびに `self.path` 全 10 手を `RawCube::multiply` で先頭から畳み込んでいます。`cp` 座標（または現在の `RawCube` / Phase 2 座標）を探索引数で更新するか、親ノードからインクリメンタルに受け渡せば再計算が消滅します。
  5. **`exhausted()` による毎ノード関数呼出＆分岐（L306-324）**: Color-only モード（`gods_guarantee = true`）では最初の `depth = 0..=20` 中 `extending == true` なのに全 2.8 億ノードで毎ノード分岐評価されます。外側のループ／深さごと、または `nodes & 4095 == 0` のチェックに絞り、`gods_guarantee` 時も安全ハード上限（後述 `CORR-07`）のみチェックすべきです。
  6. **`get_twist_slice` / `get_flip_slice` の間接参照（`tables.rs:128-153`）**: 毎ノード 2 回呼ばれ、毎回 `twist_class` → `twist_self_sym` → `twist_sym` → `ud_slice_x2` → `twist_slice` の最大 5 配列参照と 2 分岐を踏みます。
  7. **`lib.rs:282-365` のセンター向き 2 段階予算配分**: 色解を求めて `seq_len` を設定してから同時最適化に入りますが、色解後のセンター補正は最大 60 手近くなるため `seq_len` 上限が非常に緩く、同時最適化で深い木を無制限に探索して 3 秒を使い切るケースが多発します。
- **改善案**:
  1. **6 方向インターリーブ統合**: Loop 1 の `0..=12` 単独走査を廃止し、最初から深さ `0..` で `directions`（UD/RL/FB × 順・逆）を交互探索する。
  2. **4-bit 直接枝刈り表の導入**: 対称マップを廃し `twist_slice`（541 KB）＋`flip_slice`（507 KB）＋`cp_slice`（484 KB）＋`ep8_slice`（484 KB）を 4-bit ニブル格納（合計 **2.02 MB**）にして 1 回のアラインド配列参照＋シフトで距離を取得する。
  3. **センター向きの 6 方向共役対応**: `axis` ごとの面置換マップと `inv_centers` を導入し、センター向き込みでも 6 方向インターリーブを解禁する。
  4. **Phase 1 出口のインクリメンタル化と子ノード先読み枝刈り**: `phase1` / `phase2` で子ノード再帰前に `distance > depth - 1` なら再帰自体をスキップし、`cp` 遷移をテーブルで 1 命令追跡する。

#### [PERF-03] WASM バイナリ肥大化（6.36 MB）と直列二重ロード・二重メモリ消費

- **該当箇所**: [`src/lib.rs:13`](../src/lib.rs#L13), [`src/c3x3/tables.rs:10-48`](../src/c3x3/tables.rs#L10-L48), [`src/c3x3/table_io.rs:8-104`](../src/c3x3/table_io.rs#L8-L104), [`src/c3x3/coord.rs:343-402`](../src/c3x3/coord.rs#L343-L402), [`web/main.ts:1594,1737`](../web/main.ts#L1594), [`web/solver.worker.ts:15-16`](../web/solver.worker.ts#L15-L16), [`Cargo.toml:23-25`](../Cargo.toml#L23-L25)
- **原因と定量根拠**:
  1. **`MoveTable.cp` / `ep8` が gzip サイズの大半を占有**: `cp`（40,320×18×2 = 1.45 MB）と `ep8`（1.45 MB）の 2 表はエントロピーが高く gzip 後も合計 **~2.54 MB（WASM gzip 3.48 MB の約 73%）**を占めます。しかも Phase 2 で使うのは 18 手中 10 手のみで、残り 8 列（44.4% = 1.29 MB raw）は完全デッドデータです。
  2. **4-bit 格納＋ゼロコピー未適用**: 3x3 枝刈りテーブル 4 表の最大値は 9〜14 であり全要素 4-bit に収まりますが、現在 8-bit 格納（3.02 MB）かつ `decode_pruning`（`table_io.rs:76-88,94-103`）で `include_bytes!` の静的データ（6.11 MB）からさらにヒープへ約 6.1 MB をコピー複製しています。さらに起動ごとに `ensure_table_verified` が 6.11 MB 全体に FNV-1a チェックサムを走らせます。
  3. **`set_cp` / `set_ep8` のヒープ確保**: `coord.rs:344,384` が `(0..8).collect::<Vec<u8>>()` と `.remove()` を使用しており、Phase 2 移動表生成時に 80,640 回ヒープ確保が発生します（一方 `c2x2/coord.rs:91-99` はスタック配列 `[0..7]` で実装済み）。
  4. **メインスレッド＋Worker の直列二重ロード**: `main.ts:1594` がメインスレッドで 6.36 MB WASM をロード・コンパイル・インスタンス化し（使用するのは `apply_moves`, `validate`, `scramble`, `scramble_2x2`, `center_parity`, `is_valid` のみで `tables.bin` は完全不使用）、その完了後に `L1737` で Worker を起動して Worker が同じ 6.36 MB WASM を再ロードします。両スレッドの WASM 線形メモリ＋テーブルで計 **~18.6 MB** を消費しています。
  5. **ビルド設定とプリロード欠如**: `Cargo.toml:23-25` に `codegen-units = 1`, `panic = "abort"`, `strip = true` がなく、`index.html` / `vite.config.ts` に WASM プリロードもありません。
- **改善案**:
  1. `PruningTable`（対称マップ廃止後の 4-bit 直接表 **2.02 MB**）を `tables.bin` に格納し、WASM ではヒープ複製せず `TABLE_BYTES` スライスから **ゼロコピー参照**する（ヒープ複製 6 MB 削減＋`ensure_table_verified` 撤去または必要最小限化）。
  2. `MoveTable.cp` / `ep8` / `slice_p` は Phase 2 の 10 手のみに限定して `tables.bin` から外し、`set_cp` / `set_ep8` をスタック配列化した上で Worker 起動時に ~10 ms でオンメモリ生成する（あるいは 10 手化表を格納）。これで **WASM バイナリを 6.36 MB (3.48 MB gzip) → 約 1.7〜2.2 MB (~750 KB gzip、最大 78% 削減)** へ縮小できます。
  3. 短期的には `main.ts` で `WebAssembly.compileStreaming` して `WebAssembly.Module` を Worker に転送共有し、中期的にはメインスレッド用軽量関数（盤面検証・回転・スクランブル・パリティ）を TS 化またはテーブル非同梱 WASM にしてメインスレッドの 6 MB ロード自体を撤廃します。
  4. `[profile.release]` に `codegen-units = 1`, `panic = "abort"`, `strip = true` を追加します。

#### [PERF-04] 毎手 `renderPresetButtons()` DOM 再構築・`getCellArrowInfo` 3 重実行・広範 `querySelectorAll`

- **該当箇所**: [`web/main.ts:214, 217-230, 571-584, 610-614, 664, 775-784`](../web/main.ts#L762-L786), [`web/scene.ts:703`](../web/scene.ts#L703), [`web/model.ts:212-408`](../web/model.ts#L212-L408)
- **原因と分析**:
  1. **毎手プリセットボタン DOM 全破棄・再作成**: `store.subscribe`（`main.ts:775-784`）が `replace`, `undo`, `redo`, `algorithm` のたびに無条件で `updateCubeTypeUI(store.getCubeType())` を呼び出し、その中の `L214` で `renderPresetButtons(type)` を毎回実行しています。結果、ユーザーが手動で `R` ボタンを 1 回押すたびに `#preset-buttons` 配下の DOM ノードが全削除・再生成されます（種別が変わっていない時はスキップすべき）。
  2. **`switchCubeType` での二重実行（`main.ts:217-230` vs `L769-773`）**: `store.setCubeType(type, true)` が `notify("cube-type")` 経由で `updateCubeTypeUI`・`scene.setCubeType`・`persist`・`refresh` を実行した後、`switchCubeType` のコールバック本体でも同じ 4 処理をもう一度実行しています。
  3. **アニメーション 1 手あたりの `getCellArrowInfo` 3 重実行と `scene.show` 2 重実行**:
     - `seek()` の開始時 `store.updateAfterSeek()` → `refresh()` で `getCellArrowInfo`（1 回目）
     - `scene.turn()` 完了時 [`scene.ts:703`](../web/scene.ts#L703) の `this.show(state, this.next)`（`precomputedArrowInfo` なし）で `getCellArrowInfo`（2 回目）＋ 54 メッシュ更新
     - `seek()` 完了時 `main.ts:855` の `refresh()` で `getCellArrowInfo`（3 回目）＋ `scene.show`（2 回目）
  4. **`getCellArrowInfo` 自体の高コスト構造（`model.ts:212-408`）**: 毎回の呼び出しで `CORNERS`（8 スロット × 最大 8 ピース探索 × `.map()` / `.includes()`）・`EDGES`（12 スロット × 最大 12 ピース探索 × `.map()` / `.includes()`）を実行し、1 回あたり **200 個以上の小配列・`Vec3`・3×3 行列と 108 回の `Math.atan2`** を生成しています。しかも `CORNERS` 8 箇所・`EDGES` 12 箇所の各向き（コーナー 3 通り、エッジ 2 通り）に対して各ステッカー角度は完全に決定的（`0, ±π/2, π` の 4 値のみ）なため、事前計算ルックアップ表に置き換えれば **行列計算・`atan2`・配列確保がゼロになり ~20 倍高速化**できます。
  5. **毎 `refresh()` の DOM セレクタ走査**: `L610-614` が毎回 14 個のセレクタで `querySelectorAll` を走査して `disabled` を代入し、`L664` も毎ステップ `.solution-move` を全検索しています（キャッシュ参照で O(1) 化可能）。

#### [PERF-05] 3x3 Thistlethwaite（平均 812 ms / 15% 失敗）と Korf（100% 失敗前 685 ms 浪費）

- **該当箇所**: [`src/c3x3/thistlethwaite.rs:182-447`](../src/c3x3/thistlethwaite.rs#L182-L447), [`src/c3x3/korf.rs:58-167`](../src/c3x3/korf.rs#L58-L167)
- **詳細**:
  - **Thistlethwaite**: 全 4 フェーズが未だ `RawCube::multiply` を毎回実行しており、さらに Phase 3（`G2 → G3`）には枝刈りヒューリスティックが一切ありません（`search_g3` L363-392）。実測で平均 **811.99 ms / 4,597 万ノード、3/20（15%）が Kociemba へフォールバック**しています。Phase 1 は `MoveTable.flip` + 2 KB 距離表（`OnceLock` 生成 &lt; 0.1 ms）、Phase 2 は既存の `twist` + `ud_slice` + `PruningTable::get_twist_slice`、Phase 3 は `|G2/G3| = 29,400` 通りの 29 KB 完全距離表（生成 &lt; 2 ms）、Phase 4 は `cp` / `ep8` / `slice_p` テーブル＋`cp_slice`/`ep8_slice` を活用することで **平均 812 ms → 1 ms 未満、フォールバック 0%** になります。
  - **Korf (IDA*)**: 25 手スクランブルでは実測 **20/20（100%）が `max_depth = 12` で解けず、Kociemba フォールバック前に平均 684.99 ms（3,456 万ノード）を浪費**しています。`search()` も `RawCube::multiply` を毎回実行しています。3 軸共役（UD/RL/FB）の下界最大値 `h3` を初期判定に使うと深さ 12 超過の局面の大半を探索開始前に ~0 ms で Kociemba へ即フォールバックでき、さらに `max_depth` 内も座標ベース化で高速化できます。

#### [PERF-06] 2x2 Ortega 150 倍遅延・Optimal 向き ON・初回 117.5 ms BFS・24 向き二重探索

- **該当箇所**: [`src/c2x2/ortega.rs:78-142, 262-301`](../src/c2x2/ortega.rs#L78-L142), [`src/c2x2/search.rs:58-86, 111-175`](../src/c2x2/search.rs#L58-L86), [`src/c2x2/tables.rs:35-60`](../src/c2x2/tables.rs#L35-L60), [`src/c2x2/mod.rs:73-98`](../src/c2x2/mod.rs#L73-L98)
- **詳細**:
  1. **Ortega `solve_pbl`（`ortega.rs:262-301`）の 9,216 回 6 重ループ**: `pre_u(4) * pre_d(4) * alg(9) * y_rot(4) * post_u(4) * post_d(4) = 9,216` 通りのループ内部で毎回 `Vec::with_capacity` を確保し `cube::apply` を呼んでいます。しかも「スキップ」や対称アルゴを 4 回転分回しているため 4 倍重複しています。結果、`ortega` の実測平均は **1.361 ms / 最大 6.336 ms** で、**同じ 3 ステップ解法の `lbl`（平均 0.009 ms / 最大 0.028 ms）より約 150 倍、`optimal`（0.058 ms）より約 23 倍遅い**状態です。
  2. **Ortega `solve_first_face`（`ortega.rs:78-142`）**: `hash_first_face` が U 層 4 コーナーの配置・向きまで全部含めてハッシュ化しているため状態数が爆増し、`Vec` クローン付き BFS になっています。D 面 4 ピース（U 層は無視して「D 面ステッカーが D 面スロットにあれば 1、それ以外は 0」の 4 スロット位置ビット＋D 面ピース向きのみで済む）だけを見れば状態数は極小、あるいは `group_b` 型の軽量距離表で &lt; 0.01 ms で解けます。
  3. **Optimal 向き ON（`c2x2/search.rs:58-60`）**: 向き OFF（`1.62 ms / 55,582 nodes`）に対し、向き ON は 6 面探索のため Antipode で **106.95 ms / 3,735,583 nodes** かかります。`ida_star` で子ノード再帰前に `h == 0` / 先読み枝刈りし、A↔B の鏡像対称を利用できます。
  4. **初回 2x2 探索の 117.5 ms 遅延（`c2x2/tables.rs:35-60`）**: `generate_group_pruning_table` は毎回 `rc.multiply` + `get_group_*_idx` を 980 万回走らせるため、2x2 初回解決だけ **119.13 ms** ブロックされます。Group A↔B・C↔D は上下対称写像で一方から O(N) コピーでき、さらに 4-bit パック済み 272 KB テーブルを `tables.bin` に同梱すれば **~1 ms** になります。
  5. **`c2x2::solve`（`c2x2/mod.rs:73-98`）の無駄計算**: Optimal 選択時、`L76-86` で計算した `target_rc` を一切使わず `searcher.solve(&rc, include_orientation)` に生 `rc` を渡して `c2x2/search.rs:61-82` で同じ 24 向き検索をやり直しています。

#### [PERF-07] Three.js の常時 60fps ループ、`finish()` 全 194 メッシュ走査、195 ドローコール

- **該当箇所**: [`web/scene.ts:226, 431-539, 715-765`](../web/scene.ts#L226)
- **詳細**:
  1. `setAnimationLoop((time) => this.frame(time))` が静止時も毎フレーム `this.controls.update()`（`OrbitControls` ダンピング計算）を実行し続けています（`controls.update()` が `false` を返し `this.active === undefined` かつ `!this.dirty` なら描画不要で、操作・回転開始時にだけループを回すオンデマンド駆動にすれば静止時 CPU 消費をゼロ化できます）。
  2. `finish()`（L715-748）は 1 層の回転完了のたびに `pieces`（26 個）、`outlineMeshes`（54 個）、`arrowMeshes`（54 個）、`centerLabels`（6 個）の **全 140 個（+`stickers` 重複含め最大 194 回ループ）** に対して無条件に `root.add` / `arrowGroup.add` と行列コピーを行っています。`turn()` で `turnLayer` へ移したメッシュ配列だけを保持しておけば **約 37 個**のみの復元に減ります。
  3. ステッカー 54 枚は同じ `RoundedBoxGeometry` を共有しつつもそれぞれ個別の `MeshStandardMaterial` を持ち、矢印 54 + アウトライン 54 + ボディ 26 + センターラベル 6 + 床 1 で **1 フレーム最大約 195 ドローコール**を発行しています。ボディ 26 個と矢印・アウトラインは `InstancedMesh` へ統合可能です。

#### [PERF-08] カメラモーダルの毎ピクセル再描画・7.7 MB readback 3 重複・色編集の過剰再生成

- **該当箇所**: [`web/camera.ts:276-296, 751-815, 836-845, 876, 1013-1082, 1430-1436`](../web/camera.ts#L276-L296), [`web/camera-ui-helper.ts:13, 55`](../web/camera-ui-helper.ts#L13), [`web/editor.ts:140-146, 152-253`](../web/editor.ts#L140-L146)
- **詳細**:
  1. `handlePointerMove`（`camera.ts:276-296`）がホバー中、`prevHover === hit` でもマウス 1px 移動ごとに背景画像ごと `this.draw()` を全面再描画しています。さらに `toCanvasCoords` と `findHitTarget` が毎イベントで `getBoundingClientRect()` を **2 回**呼んでレイアウト強制再計算（リフロー）を誘発しています。ドラッグ中（`L289`）の毎フレーム `this.update()` も文字列比較なしで DOM を叩いています。
  2. `ImageData`（1600×1200 で 7.68 MB の GPU→CPU readback）がキャッシュされておらず、`autoDetectOutline`（L836-842）、`updateDetectedLabels`（L1063）、`capture`（L876）で毎回新規 `<canvas>` 生成＋`drawImage`＋`getImageData` を繰り返します。しかも `updateDetectedLabels`（L1064-1066）はセンター 3 マスの色しか使わないのに `sampleFaceFromPixels` で 27 マスすべてをサンプリング・分類しています。画像読込時に `ImageData` を 1 回キャッシュし、ラベル推定は `(0.5, 0.5)` 3 点のみに絞るべきです。
  3. `processFile`（L751-815）の 1600px 超リサイズは `new Image()` フルデコード → Canvas 縮小 → `toBlob("image/jpeg", 0.92)` 再エンコード → `new Image()` 再デコードを行います。`createImageBitmap(file, { resizeWidth, resizeHeight })` を使えば二重デコードと JPEG 圧縮劣化を回避できます。
  4. `close()`（`camera.ts:1430-1436`）が `dialog.close()` を呼んで `close` イベント経由で `handleDialogClose()` を発火させた直後に L1435 でも直接 `this.handleDialogClose()` を呼んでおり、毎回クローズ処理が 2 重実行されます。
  5. `editor.ts:140-146, 152-253`: 1 マス塗るたびに `paint()` が `automaticCenters(this.draft)`（WASM `center_parity`）を呼んで未完成盤面で毎回例外を throw/catch し、さらに `render()` がパレット 7 ボタン＋展開図 54 セル＋ガイド 9 セル＋センター 6 セレクト（24 オプション）の **約 110 DOM ノード**を毎回全破棄・再作成しています。

#### [PERF-09] Service Worker 旧巨大 WASM 移行による肥大化＆比較モード直列実行

- **該当箇所**: [`public/sw.js:95-128`](../public/sw.js#L95-L128), [`web/main.ts:1096-1125`](../web/main.ts#L1096-L1125)
- **詳細**:
  1. `public/sw.js:95-127` は `install` 時点で新しい全ビルド資産を `PRECACHE_ASSETS` に保存済みであるにもかかわらず、`activate` 時に旧キャッシュからハッシュ名が異なる旧資産（**旧 6.36 MB WASM や旧 578 KB `three.js` 等を含む最大 15 件**）を新キャッシュへコピーし続けます。デプロイのたびに使われない旧版 WASM が蓄積し続け、ストレージとキャッシュ検索コストを悪化させます。さらに `sortedOldKeys`（L96-99）はランダム 10 桁 SHA-256 ハッシュを数値ソートしているため「直近世代の選択」自体が機能していません。
  2. `compareAllSolvers`（`main.ts:1096-1125`）は単一 Worker で 4 アルゴリズムを直列実行するため合計 2〜5 秒待機します。PERF-02/05 解消後でも必要なら最速順実行または 2 Worker 並列化を検討できます。

#### [PERF-10] `apply_moves` の未使用中間状態 JSON 生成と Three.js インポート整理

- **該当箇所**: [`src/lib.rs:37-63, 476-484`](../src/lib.rs#L37-L63), [`web/main.ts:909, 1330, 1684, 1784, 1875`](../web/main.ts#L909), [`web/scene.ts:1`](../web/scene.ts#L1)
- **詳細**:
  - `web/main.ts` の `apply_moves` 呼び出し 5 箇所はいずれも `result.state` と `result.moves` のみを使い `result.states` を破棄していますが、最大 2,048 手（4,095 B）入力時には 2,049 盤面（約 125 KB）の文字列・JSON が無駄に生成されます。`states` を省いた `apply_moves_final` を用意するか、`states` 配列生成を必要なときのみに限定できます。
  - `web/scene.ts:1` の `import * as THREE from "three"` を名前付きインポートへ切り替え、`vite.config.ts` とあわせてツリーシェイキング余地を確認できます。

---

## 3. コードの構造（Code Structure / Architecture）レビュー指摘事項

### 3.1 指摘サマリー（コードの構造）

| ID | 重大度 | 領域 | 対象ファイル（行番号） | 構造課題と改善方向 |
| :--- | :---: | :--- | :--- | :--- |
| **ARCH-01** | **High** | フロントエンド God File | [`web/main.ts`](../web/main.ts) (L1-1988, 特に L547-761 `refresh`)<br>[`web/view.ts`](../web/view.ts) (L32-96) | 1,988 行・13 個以上の独立責務＋215 行の `refresh()` が単一ファイルに集中（および `view.ts` の 1 行 3,000 文字超 HTML）。コントローラ／レンダラ分割が必須。 |
| **ARCH-02** | **High** | カメラ God Class | [`web/camera.ts`](../web/camera.ts) (L34-218, L220-1444) | 1,444 行にバイナリヘッダ解析、ライブストリーム制御、画像パイプライン、UI、3 面クワッド重複定義が同居。 |
| **ARCH-03** | **High** | Single Source of Truth 欠如 | [`web/main.ts`](../web/main.ts), [`web/app-state.ts`](../web/app-state.ts), [`web/cube-store.ts`](../web/cube-store.ts), [`web/model.ts`](../web/model.ts), [`web/url-params.ts`](../web/url-params.ts), `cubes/` vs `public/cubes/` | 状態の 3 重管理、ソルバー定義 5 箇所分散、2x2 デフォルト不整合（`"lbl"` vs `"optimal"`）、同一 JSON 群の二重配置。 |
| **ARCH-04** | **Medium** | Rust モジュール構成 | [`src/lib.rs`](../src/lib.rs) (L152-446), [`src/c3x3/tables.rs`](../src/c3x3/tables.rs) (L3), [`build.rs`](../build.rs) (L1-6) | 295 行の `solve_state_with_algorithm`、`include!("table_io.rs")`＋`#[path]` の非標準結合、3x3/2x2 共通基盤モジュール不在。 |
| **ARCH-05** | **Medium** | WASM 境界＆脆い文字列結合 | [`src/lib.rs`](../src/lib.rs) (L525-706), [`web/model.ts`](../web/model.ts) (L136-167), [`web/triggers.ts`](../web/triggers.ts) (L38-111), [`web/main.ts`](../web/main.ts) (L2, L1594) | 14 WASM API 中 6 個が本番未使用、`validate` が `is_solved` 別名、エラー箇所特定・フェーズ名解決が日本語文の文字列/正規表現に依存。 |
| **ARCH-06** | **Medium** | 3D ビルダーの重複構造 | [`web/scene.ts`](../web/scene.ts) (L31-55, L427-539) | 2x2 と 3x3 でキュービー・ステッカー生成ループが 110 行ほぼ二重化。サイズパラメータ化で統合可能。 |
| **ARCH-07** | **Low** | テスト・リポジトリ衛生 | `tests/` (65 files / 17,520 行), ルート `coverage*`, `docs/` (44 files) | 3,348 行巨大テスト、15 個の時系列テストの `__cube_main_debug__` 密結合、ルート残留カバレッジ生成物。 |

---

### 3.2 コードの構造 詳細指摘

#### [ARCH-01] `web/main.ts`（1,988 行）God File と 215 行 `refresh()` の責務集中

- **現状**: [`web/main.ts`](../web/main.ts) 1 ファイルに以下がすべて混在し（`L61-62` のファイル中盤 `import`/`export` を含め）、可読性と単体テスト性を大きく損ねています。また [`web/view.ts:32-96`](../web/view.ts#L32-L96) は最大 1 行 ~3,000 文字の改行なし HTML テンプレートを抱えており差分レビューを困難にしています。
  1. WASM 初期化・起動シーケンス・URL パラメータ復元・`localStorage` 永続化
  2. `CubeStore` と `AppStateMachine` の接続・二重フラグ同期
  3. 単体探索（`solve`）と比較探索（`compareAllSolvers`）の制御
  4. 手順再生・アニメーション・シーク（`play`, `stop`, `seek`）
  5. プリセット管理（3x3 `fetch` ＋ 2x2 インメモリ切替）
  6. モーダル制御（`openModal`, `getEditor`, `getCamera`）
  7. ファイル I/O・共有リンク・キーボード・矢印凡例・215 行の巨大 `refresh()`（L547-761）
- **推奨分割構造**:
  - `web/bootstrap.ts`: 起動・`localStorage`・URL パラメータ初期化・サービスワーカー登録
  - `web/solve-controller.ts`: `solve` / `compareAllSolvers` / `cancelSearch` / 比較結果状態
  - `web/playback-controller.ts`: `play` / `stop` / `seek` / `animation` トークン管理
  - `web/preset-controller.ts`: 3x3 / 2x2 プリセット一覧・読込レース制御
  - `web/ui-renderer.ts`: `refresh()`（盤面ステータス・コントロール活性制御・手順リスト・比較表描画を各専用関数へ分割）

#### [ARCH-02] `web/camera.ts`（1,444 行）のバイナリ解析・カメラ制御・UI 一体化

- **現状**: [`web/camera.ts`](../web/camera.ts) に、すでに切り出された `camera-geometry.ts` / `camera-canvas-renderer.ts` / `camera-results-ui.ts` / `camera-ui-helper.ts` の残りすべて（画像ヘッダパーサ、ライブカメラ `MediaStream` 制御、6 点入力・ドラッグ制御、画像リサイズ、サンプリング制御、結果確認・2x2 検証）が 1,444 行のまま残っています。また A 面 `U/R/F`・B 面 `D/L/B` のクワッド対応定義が [`L881-891`](../web/camera.ts#L881-L891) と [`L1050-1061`](../web/camera.ts#L1050-L1061) に重複しています。
- **推奨構造**:
  - `web/image-header.ts` へ `readImageDimensions` / `readJpegDimensions` / `checkImagePixelCount` を分離（DOM 非依存の純粋 Blob パーサとして単体テストも容易化）。
  - `web/live-stream.ts` へ `MediaStream` ライフサイクルを分離。
  - `VIEW_QUADS` 定数ヘルパーで 3 面クワッド定義を一元化。

#### [ARCH-03] Single Source of Truth（単一信頼源）の複数箇所における崩壊

1. **アプリ状態の 3 重管理**: [`web/cube-store.ts`](../web/cube-store.ts)（`solution`, `baseSnapshot`, `step`）、[`web/app-state.ts`](../web/app-state.ts)（`idle | solving | previewing`）、[`web/main.ts:69`](../web/main.ts#L69)（モジュール変数 `let solving = false`）の 3 箇所が同じ状態を別々に同期しており、さらに `mainReady`, `engineError`, `playing`, `inMotion` もモジュール変数に散在しています。
2. **ソルバーメタデータ・選択肢の 5 箇所分散**:
   - [`web/view.ts:82`](../web/view.ts#L82)（初期 HTML `<select>`）
   - [`web/main.ts:86-93`](../web/main.ts#L86-L93)（`SOLVERS_BY_TYPE`, `DEFAULT_SOLVER`）
   - [`web/main.ts:169-191`](../web/main.ts#L169-L191)（`updateCubeTypeUI` 内の `<option>` ハードコード再生成）
   - [`web/model.ts:452-464`](../web/model.ts#L452-L464)（`SOLVER_META_BY_TYPE`）
   - [`web/url-params.ts:12-20`](../web/url-params.ts#L12-L20)（`VALID_SOLVERS`）
   → `SOLVER_META_BY_TYPE` 1 箇所から `<option>` 生成・バリデーション・説明文を導出すべきです。
3. **2x2 デフォルトソルバーの仕様不整合**:
   - [`web/main.ts:91`](../web/main.ts#L91) と [`web/url-params.ts:91`](../web/url-params.ts#L91) は `"lbl"` を既定にする一方、[`src/lib.rs:662`](../src/lib.rs#L662) と [`web/solver.worker.ts:21`](../web/solver.worker.ts#L21) は `"optimal"` を既定にしています。
4. **`cubes/` と `public/cubes/` の完全重複**:
   - `diff -rq cubes/ public/cubes/` の結果が**完全一致（全 6 ファイル同一内容）**でありながら、`tests/*` と `src/*` は `cubes/` を参照し、Vite 配信は `public/cubes/` をコピーします。`public/cubes/` のみ（またはスクリプト／テスト参照の統一）に一本化しないとプリセット修正時に片側だけ古いままになります。

#### [ARCH-04] Rust 側の肥大関数・非標準モジュール結合・3x3/2x2 分断

1. **`solve_state_with_algorithm`（[`src/lib.rs:152-446`](../src/lib.rs#L152-L446)）が 295 行**: `2x2` 分岐・4 アルゴリズム分岐・センター向き 2 段階最適化・フォールバック・`PhaseInfo` 構築がすべて 1 関数にベタ書きされています。`src/c3x3/mod.rs` へ 3x3 ディスパッチを移し、`lib.rs` は薄い WASM / FFI アダプタ層に徹するべきです。
2. **`include!("table_io.rs")` ＋ `build.rs` `#[path]` ハック（[`src/c3x3/tables.rs:3`](../src/c3x3/tables.rs#L3), [`build.rs:1-7`](../build.rs#L1-L7)）**: `table_io.rs` が独立した Rust モジュールではなくテキスト `include!` で結合されており、IDE 補完・lint・可視性境界が不明瞭です。独立モジュール化または共通サブモジュール化が推奨されます。
3. **3x3 と 2x2 の基本操作重複**: `parse_moves`、`notation`、`redundant`、`apply_moves_core`、`ResultData` 組み立て（`states` 生成ループが [`src/lib.rs:46-52`](../src/lib.rs#L46-L52)、[`src/c2x2/mod.rs:120-126`](../src/c2x2/mod.rs#L120-L126)、[`src/c2x2/mod.rs:162-168`](../src/c2x2/mod.rs#L162-L168)、[`src/bin/serialization_bench.rs:13-19`](../src/bin/serialization_bench.rs#L13-L19) に 4 重複）を共通ヘルパーへ集約すべきです。

#### [ARCH-05] WASM API 境界の肥大（14 → 8 API）と脆い文字列結合

- `#[wasm_bindgen]` エクスポートは **14 個**ありますが、本番コード（`web/*.ts`）が実際に使用するのは **8 個**（`initialize`, `scramble_2x2`, `validate`, `is_valid`, `center_parity`, `apply_moves`, `scramble`, `solve_with_algorithm`）のみで、残り **6 個**（`solve_2x2`, `solve_2x2_with_algorithm`, `is_solved`, `solve`, `solve_with_orientation`, `get_orientations`）は本番ゼロ使用です。
- さらに [`src/lib.rs:489-491`](../src/lib.rs#L489-L491) の `validate` は内部で `is_solved_core` を呼んでおり、名前と実装がズレています（呼び出し側の `main.ts` も戻り値 `bool` を一切見ず例外が出ないこと＝合法判定としてのみ使用＝`is_valid` と完全重複）。
- [`web/model.ts:136-167`](../web/model.ts#L136-L167) の `getErrorIndices` が Rust の日本語エラー文字列（`/エッジ\s*(\d+)/`, `/コーナー\s*(\d+)/`, `/スロット\s*(\d+)\s*のコーナー/`）を正規表現で逆パースして色エディタの赤枠インデックスを復元しているため、Rust 側の文言を変えるだけで UI ハイライトがサイレントに壊れます。同様に [`web/triggers.ts:38-111`](../web/triggers.ts#L38-L111) の `PHASE_LABEL_MAP`（38 キー）と `getPhaseLabel()` も Rust 側の日本語フェーズ文字列に完全依存（しかも `L102-108` は括弧前プレフィックスが一致すると翻訳せず生 `name` を返す妙なフォールバック付き）しており、構造化 ID（enum/code）ベースの受け渡しへ移行すべきです。

#### [ARCH-06] `web/scene.ts` の 3x3 / 2x2 ビルダー二重実装

- [`web/scene.ts:427-539`](../web/scene.ts#L427-L539) で `"2x2"` と `"3x3"` の `buildCube` が約 110 行にわたりほぼ同じキュービー・ステッカー生成ループを複製しています。`gridSize`（2 or 3）・`offset`（0.5 or 1.0）・`stickerPlaneDist` をパラメータ化すれば 1 本の共通ビルダーに統一できます。また `position3x3` / `position2x2` も同数式で統合可能です。

#### [ARCH-07] テスト肥大とリポジトリ直下のレガシー成果物残留

- `tests/` が **65 ファイル・17,520 行**に達し、特に [`tests/coverage.spec.ts`](../tests/coverage.spec.ts) が **3,348 行**、時系列パッチごとに作られた `tests/code-review-*-regression.spec.ts` が **15 ファイル**存在します。多くが `window.__cube_main_debug__`（[`web/main.ts:1940-1987`](../web/main.ts#L1940-L1987)）の内部関数差し替えに依存しているため、`main.ts` をリファクタリングする際の足枷になります。
- ルート直下に古い解析成果物（`coverage_improved.txt`, `coverage_new.txt`, `coverage.txt`, `coverage.lcov`, `coverage-tarpaulin*`, `COVERAGE_*.md` 等）が多数残置され、`docs/` にも 44 件の過去レビューが残っています。機能単位で E2E テストを集約し、古い生成物を整理／アーカイブすることで保守性が向上します。

---

## 4. むやみな複雑さの除去（Removing Unnecessary Complexity）

### 4.1 削減サマリー

| ID | 重大度 | 対象箇所 | 不要な複雑さの内容 | 推定削減量・副次効果 |
| :--- | :---: | :--- | :--- | :--- |
| **SIMP-01** | **High** | [`web/app-state.ts`](../web/app-state.ts) (L1-302)<br>[`web/main.ts`](../web/main.ts) (L57, L69) | 本番で `IdleState` ≡ `PreviewingState` なのに 3 クラス + `AppState` IF + 未使用 7 メソッド/引数で 302 行。 | **約 260 行削減**＋`solving` 三重同期解消 |
| **SIMP-02** | **High** | [`src/c3x3/tables.rs`](../src/c3x3/tables.rs) (L40-58, L128-153, L251-495)<br>[`src/c3x3/table_io.rs`](../src/c3x3/table_io.rs) (L21-26, L99-100) | x2 対称マップ 7 配列・分岐計算で**4-bit 直接表（1.05 MB）より実メモリが大きく、かつ 5 回参照で低速**という逆転。 | **約 150 行削減**＋探索高速化＋`tables.bin` 簡素化 |
| **SIMP-03** | **High** | [`web/solver.worker.ts`](../web/solver.worker.ts) (L23-72)<br>[`web/model-2x2.ts`](../web/model-2x2.ts) (L74-102) | Rust `c2x2::solve` が既に実装済みの 2x2 向き OFF 正規化を Worker で重複実装＋2 回目 WASM 呼出。 | **約 50〜79 行削減**＋2 回目 WASM 呼出廃止 |
| **SIMP-04** | **High** | [`public/sw.js`](../public/sw.js) (L69-70, L81-136, L236-263) | 逆効果の旧キャッシュ資産移行（55 行）＋メモリ配列とストレージキーの二重 LRU 管理。 | **約 75 行削減**＋SW キャッシュ肥大化解消 |
| **SIMP-05** | **Medium** | [`src/c3x3/`](../src/c3x3/)・[`src/c2x2/`](../src/c2x2/) 全体 | `redundant` 7 重複、MoveTable 6 関数、Pruning 2 関数、2x2 OLL 2 重複、センター計算 5 重複、二重検証、冗長 `.min()`。 | **約 250 行削減**＋バグ温床一掃 |
| **SIMP-06** | **Medium** | [`src/c3x3/coord.rs`](../src/c3x3/coord.rs), [`src/c2x2/coord.rs`](../src/c2x2/coord.rs), [`src/c3x3/search.rs`](../src/c3x3/search.rs), [`web/image-sampler.ts`](../web/image-sampler.ts), [`web/camera.ts`](../web/camera.ts), [`web/camera-geometry.ts`](../web/camera-geometry.ts) | テスト以外で未使用の型・メソッド・引数（`CoordCube`, `c2x2::RawCube::get_cp` 等, `with_max_total`, `rgbToHsv`, `sampleFace` import）。 | **約 120 行削減**＋API 面明確化 |
| **SIMP-07** | **Medium** | [`web/cube-store.ts`](../web/cube-store.ts) (L70-77, L170-177, L186-193, L199-206, L222-229, L250-257, L303-310)<br>[`web/camera-results-ui.ts`](../web/camera-results-ui.ts) (L128-132, L182-186) | 6 箇所に散らばる盤面リセット処理のコピペ、48 個ボタンへの毎回 `.onclick` クロージャ再代入。 | **約 60 行削減**＋DOM 更新バグ予防 |
| **SIMP-08** | **Low** | [`src/c3x3/supercube.rs`](../src/c3x3/supercube.rs) (L91-210) | `remove`+`insert` による手置換、毎フェーズの `Vec` 確保、2 回走査固定点ループ。 | **約 30 行削減**＋O(N) 化 |

---

### 4.2 むやみな複雑さの除去 詳細指摘

#### [SIMP-01] `web/app-state.ts`（302 行）の過剰 State パターンと死にメソッド群

- **該当箇所**: [`web/app-state.ts:1-302`](../web/app-state.ts#L1-L302), [`web/main.ts:57,69`](../web/main.ts#L57)
- **詳細**:
  - 3 つの状態（`"idle" | "solving" | "previewing"`）を表すためだけに、`AppState` インターフェース＋`IdleState`・`SolvingState`・`PreviewingState`・`AppStateMachine` で **302 行**が費やされています。
  - しかし両方の実装を比較すると、**本番コード（`web/`）において `IdleState` と `PreviewingState` の実挙動は完全に同一**（`canModifyCube() = true`, `canStartSolve() = true`）であり、区別するためだけに用意された `canCancelSolve()`, `canUndoRedo()`, `canSeekSolution()`, `isIdle()`, `isPreviewing()`, `subscribe()`, `runCubeMutation` の第 2 引数 `onExitPreview` は**本番コードで一度も呼ばれていません**（`tests/app-state.spec.ts` と `tests/coverage.spec.ts` のみが呼んでいます）。
  - さらに `transitionTo`（L211-221）は `prevState.kind === nextState.kind` で同種別遷移を無視し、`runCubeMutation`（L240-248）はコールバック内で `replace()` → `resetToIdle()` が走ると外部状態と矛盾しないよう `this.currentState === previousState` の迂回チェックまで抱えています。
- **簡素化案**:
  - `"idle" | "solving" | "previewing"` の単純な型エイリアスと小さなヘルパー関数（または `CubeStore` への `solving: boolean` 統合、`previewing` は `solution !== undefined` から導出）に置き換えれば **約 35 行（~260 行削減）** で同等の状態遷移ガードを明快に表現できます。

#### [SIMP-02] `src/c3x3/tables.rs` の x2 対称圧縮マップ（労力・複雑さ・メモリ・速度がすべて逆転）

- **該当箇所**: [`src/c3x3/tables.rs:40-58, 128-153, 251-495`](../src/c3x3/tables.rs#L251-L495), [`src/c3x3/table_io.rs:21-26, 99-100`](../src/c3x3/table_io.rs#L99-L100)
- **詳細**:
  - `twist_slice` / `flip_slice` を半減するためだけに `generate_x2_maps()` と 7 個の補助配列（`twist_class`, `twist_sym`, `twist_self_sym`, `flip_class`, `flip_sym`, `flip_self_sym`, `ud_slice_x2`）＋対称対応版 BFS 2 関数（約 150 行）を実装しています。
  - しかし現行の 8-bit 対称圧縮は `twist_slice`（561 KB）＋ `flip_slice`（523 KB）＋ 補助 7 配列（18 KB）＝ **1,102 KB** を消費しています。一方、**対称圧縮を一切使わない 4-bit 直接テーブル**は `(2,187 * 495) / 2`（541 KB）＋ `(2,048 * 495) / 2`（507 KB）＝ **1,048 KB** と**むしろ 54 KB 小さく**、かつ探索最内ループから **4 配列の間接参照と 2 回の条件分岐を完全に消滅**させられます。対称マップ約 150 行を丸ごと削除するのが、容量・速度・単純さの全軸で優位です。

#### [SIMP-03] `web/solver.worker.ts` の 2x2 迂回コード 50 行と `relabel2x2ForDbl`

- **該当箇所**: [`web/solver.worker.ts:23-72`](../web/solver.worker.ts#L23-L72), [`web/model-2x2.ts:74-102`](../web/model-2x2.ts#L74-L102)
- **詳細**:
  - `solver.worker.ts:23-72` は 2x2 の向き OFF 時に JS 側で完成判定し、さらに `lbl` / `ortega` では `relabel2x2ForDbl` → `solve_with_algorithm(..., true)` → 結果手順で再度 `apply_moves(data.state, ...)` を呼ぶ 50 行のワークアラウンドを持っています。
  - しかし Rust 側の [`src/c2x2/mod.rs:45-95,116`](../src/c2x2/mod.rs#L45-L95) は既に `!include_orientation` での完成判定・DBL 固定空間回転 `target_rc` の算出・`lbl`/`ortega` 実行・元 `rc` 基準での解法検証および `states` 生成をすべて完結させています。Worker は `solve_with_algorithm(data.state, data.budget, includeOrientation, centersStr, algo)` の 1 経路に統一するだけで 50 行が消え、`relabel2x2ForDbl` の単独用途もなくなります（なお `isUniformSolved2x2` は UI の完成ステータス表示用なので、6 面単色チェック＋WASM 判定か簡易トリプル検証のみで簡潔に保てます）。

#### [SIMP-04] `public/sw.js` の逆効果な旧キャッシュ移行＆二重キュー管理

- **該当箇所**: [`public/sw.js:69-70, 81-136, 236-263`](../public/sw.js#L81-L136)
- **詳細**:
  - `activate` ハンドラ（L81-136）は、新キャッシュへ全最新ビルド資産を `PRECACHE_ASSETS` でプリキャッシュ済みなのに、旧ハッシュ付き資産を最大 15 件移し替える約 40 行の複雑コードを抱えています（`PERF-09` の肥大化原因）。旧バージョンのキャッシュは単純に `caches.delete(oldKey)` で全削除する 5 行で十分であり安全です。
  - また `queryKeysQueue`（インメモリ配列）と `ownCache.keys()` の永続チェック（L239-262）を二重で維持するより、`query` 付きキャッシュが必要なのは開発サーバー（Vite dev）のみなので、本番（`PRECACHE_ASSETS` が `./` + 全静的ファイルを含む構成）ではクエリ付き URL のランタイムキャッシュ自体を行わない、もしくは `ownCache.keys()` 一本に統一できます。

#### [SIMP-05] Rust クレート内の大量コピペ重複の一掃（約 250 行削減）

1. **`redundant(face, last)` / 対向面順序チェックの 7 重複**: [`c3x3/search.rs:523`](../src/c3x3/search.rs#L523), [`c3x3/cfop.rs:159`](../src/c3x3/cfop.rs#L159), [`c3x3/thistlethwaite.rs:450`](../src/c3x3/thistlethwaite.rs#L450), [`c3x3/korf.rs:179`](../src/c3x3/korf.rs#L179), [`c2x2/search.rs:147-157`](../src/c2x2/search.rs#L147-L157), [`c2x2/lbl.rs:93-101`](../src/c2x2/lbl.rs#L93-L101), [`c2x2/ortega.rs:97-105`](../src/c2x2/ortega.rs#L97-L105) に同じ論理（しかも `c3x3` は `face + 3 == last` すなわち `D->U` 禁止・`c2x2` は `last_face + 3 == face` すなわち `U->D` 禁止で逆向き！）が 7 回コピペされています。
2. **`generate_*_move_table` 6 関数（[`src/c3x3/tables.rs:155-249`](../src/c3x3/tables.rs#L155-L249)）**: `set_*` と `get_*` が違うだけで 95 行が 6 回同一ループ。ジェネリック関数 `generate_move_table<const N: usize>` 1 つで約 75 行削減。
3. **`generate_cp_slice_pruning_table` / `generate_ep8_slice_pruning_table`（[`tables.rs:497-573`](../src/c3x3/tables.rs#L497-L573)）**: `mt.cp` vs `mt.ep8` 以外は完全同一の 77 行 → 1 関数で約 35 行削減。
4. **`c2x2/lbl.rs:135-213` vs `c2x2/ortega.rs:146-198`**: OLL 7 アルゴリズム定義・`OnceLock` パース・`solve_oll` ロジックが完全重複（条件 `is_first_layer_solved` vs `is_first_face_solved` のみ差異）。約 60 行統合可能。
5. **センター回転計算（`f = m / 3; match m % 3 { 0 => 1, 1 => 2, _ => -1 }`）の 5 重複**: [`src/lib.rs:108-114`](../src/lib.rs#L108-L114), [`src/c3x3/supercube.rs:156-167, 321-327`](../src/c3x3/supercube.rs#L156-L167), [`src/bin/superflip_bench.rs:15-22, 54-62, 100-108, 121-129`](../src/bin/superflip_bench.rs#L15-L22) でベタ書き重複。`supercube::apply_moves_to_centers` 1 箇所に統一。
6. **二重 `cube::apply` 検証の整理**: `cfop::solve` / `thistlethwaite::solve` / `lbl::solve_lbl` / `ortega::solve_ortega` の各内部で `cube::apply` 検証した直後に、親の `lib.rs:435` / `c2x2/mod.rs:116` でもう一度同じ `cube::apply` 検証を実行しています（内部検証を親層へ一本化すれば十分な箇所あり）。
7. **`search.rs:374-386` & `L464-475` の冗長ガード**: `max` が既に `self.max_total - self.path.len()` の `min` を取っているのに `max_d = max.min(self.max_total.saturating_sub(self.path.len()))` を再実行。また `phase2` 中のセンター奇数チェック（L465-471）は Phase 2 に 90° R/F/L/B 手が存在しないため Phase 1 出口で保証済みの不変条件です。

#### [SIMP-06] デッドコード・テスト専用残骸の本番コード混在除去

- [`src/c3x3/coord.rs:44-51`](../src/c3x3/coord.rs#L44-L51): `pub struct CoordCube` — 本番で一切構築されず `src/tests.rs:736,1034` のみが構築。
- [`src/c2x2/coord.rs:75-122`](../src/c2x2/coord.rs#L75-L122): `c2x2::RawCube` の `get_cp`, `set_cp`, `get_twist`, `set_twist` — 2x2 ソルバー本体から一切呼ばれないデッドメソッド。
- [`src/c3x3/search.rs:93-97`](../src/c3x3/search.rs#L93-L97): `Search::with_max_total` — 全ファイル検索で呼び出し 0 件（`set_max_total` と重複）。
- [`web/image-sampler.ts:11-37`](../web/image-sampler.ts#L11-L37): `rgbToHsv` — 本番ゼロ呼出（`classifyColor` 内に同じ計算がインライン実装されているため二重管理状態。テストからのみ呼出）。
- [`web/camera.ts:15`](../web/camera.ts#L15): 未使用 import `sampleFace`。[`web/camera.ts:506`](../web/camera.ts#L506): `setupDropZone` の未使用引数 `input`。[`web/camera-geometry.ts:62`](../web/camera-geometry.ts#L62): `detectCubeOutline` の未使用引数 `_image`。
- [`web/model.ts:180-181`](../web/model.ts#L180-L181): `CellArrowInfo` の `kinds` / `pieceIndices` は `main.ts` やテストの `arrow-direction` でも不使用（`scene.ts:645-646` で `userData` に格納されるのみ、ただし `CORR-01` 修正で活用しない限り不要）。

#### [SIMP-07] `CubeStore` 状態遷移の重複集約＆`camera-results-ui` イベント委譲

- [`web/cube-store.ts:70-77, 170-177, 186-193, 199-206, 222-229, 250-257, 303-310`](../web/cube-store.ts#L70-L77) で、`this.cubeType = ...; this.state = ...; this.centerRotations = ...; this.revision++; this.solution = undefined; this.baseSnapshot = undefined; this.step = 0;` の 7 行ブロックが 6〜7 箇所にベタ書きされています。`private applySnapshotState(snapshot, type?, notifyType)` に抽出すれば大幅に見通しが良くなります。
- [`web/camera-results-ui.ts:128-132, 182-186`](../web/camera-results-ui.ts#L128-L132) は再描画のたびに 48 個（6 面×8 セル）のボタンへ新規クロージャ `cell.onclick` を再代入しています。親コンテナ `#camera-result-faces` に `onclick` イベント委譲を 1 回貼ればクロージャ再生成が消えます。

#### [SIMP-08] `src/c3x3/supercube.rs` のデータ構造簡素化

- [`src/c3x3/supercube.rs:91-140`](../src/c3x3/supercube.rs#L91-L140): `cancel_tagged_moves_single_pass` が `reduced.remove(i - 1)` + `reduced.insert(i - 1, ...)`（`Vec` 中間シフト）を繰り返し、外側の `cancel_tagged_moves` が毎回 `to_vec()` と全走査やり直しを行います。`cancel_tagged_moves_single_pass` の `i` 走査は対向面ブロック（直前高々 1〜2 手）までしか遡らないため、単純なスワップ更新または `reduced.pop()`/直近 2 スロット比較で O(N) 化でき、`cancel_redundant_moves_with_phases`（L191-207）の毎フェーズ `indices: Vec<usize>` 動的確保も `(Option<usize>, Option<usize>)` 変数 2 つでゼロアロケーション化できます。

---

## 5. 見落とし防止補完：正確性バグ・エッジケース・残存ハードニング

### 5.1 サマリー

| ID | 重大度 | 種別 | 対象ファイル（行番号） | 内容 |
| :--- | :---: | :--- | :--- | :--- |
| **CORR-01** | **High** | **実バグ** | [`web/model.ts`](../web/model.ts) (L417-444) | `countPieceOrientations` が `U` 1 手回転を「角ねじれ 1・辺反転 2」と誤報告。 |
| **CORR-02** | **High** | **実バグ** | [`web/main.ts`](../web/main.ts) (L1096-1125)<br>[`web/solver-client.ts`](../web/solver-client.ts) (L139-149) | `compareAllSolvers` で 1 解法がタイムアウトすると Worker 死亡のまま残り全解法が連鎖失敗。 |
| **CORR-03** | **Medium** | **実バグ** | [`src/c3x3/search.rs`](../src/c3x3/search.rs) (L73, L141, L178, L235) | `depth >= self.max_total` の off-by-one により `max_total == 20` 時に `depth == 20` が探索されない。 |
| **CORR-04** | **Medium** | **実バグ** | [`web/url-params.ts`](../web/url-params.ts) (L50) | `Number("") === 0` により `?centers=,,,,,` が `[0,0,0,0,0,0]` として誤受理される。 |
| **CORR-05** | **Low** | **実バグ** | [`web/scene.ts`](../web/scene.ts) (L753)<br>[`web/cube-store.ts`](../web/cube-store.ts) (L298) | `scene.ts` アニメ進捗 `p` の負値クランプ欠落・`cube-store.ts` の浮動小数点ラジアン直接比較。 |
| **CORR-06** | **Low** | **仕様齟齬** | [`web/main.ts`](../web/main.ts) (L91, L127)<br>[`src/c3x3/korf.rs`](../src/c3x3/korf.rs) (L86)<br>[`src/lib.rs`](../src/lib.rs) (L662) | Korf UI「最大20手」vs 実 `max_depth = 12`（100% フォールバック）。2x2 既定 `"lbl"` vs `"optimal"`。 |
| **CORR-07** | **Medium** | **残存S4/S6/S7/S8/S9** | `src/c3x3/search.rs`, `src/c2x2/*`, `web/camera.ts`, `web/url-params.ts`, `web/file-io.ts`, `tests/quality-contracts.spec.ts`, `public/sw.js`, `package.json`, `Cargo.toml` | 2026-11-21 レビューのうち未着手だったセキュリティ／DoS 防御項目。 |

---

### 5.2 詳細分析と修正案

#### [CORR-01] `countPieceOrientations` のねじれ・反転カウント誤算出バグ

- **該当箇所**: [`web/model.ts:417-444`](../web/model.ts#L417-L444)
- **実測再現結果**（Node 上で `web/model.ts` ロジックと WASM `get_orientations` を照合）：

| 操作 | WASM 真値 (ねじれ角 / 反転辺) | `getCellArrowInfo` 色分布 (正常 / 角 / 辺 / 中) | UI 表示 (`countPieceOrientations`) |
| :--- | :---: | :---: | :---: |
| **`U` 1 手** | **0** コーナー / **0** エッジ | 45 / **4** / **4** / 1 | **角ねじれ 1 / 辺反転 2 / 中回転 1**（誤） |
| **`R` 1 手** | **4** コーナー / **0** エッジ | 39 / **8** / **6** / 1 | **角ねじれ 2 / 辺反転 3 / 中回転 1**（誤） |
| **`F` 1 手** | **4** コーナー / **4** エッジ | 33 / 12 / 8 / 1 | 角ねじれ 4 / 辺反転 4 / 中回転 1 |
| **`U2` 1 手** | **0** コーナー / **0** エッジ | 45 / **4** / **4** / 1 | **角ねじれ 1 / 辺反転 2 / 中回転 1**（誤） |

- **原因**: `getCellArrowInfo` は「各ステッカー矢印が現在面の基準上方向から何 rad 回転しているか」を計算するため、`U` 面内の移動では `U` ステッカーだけが ±90°/180° 回転（`CORNER_TWIST`/`EDGE_FLIP` 色）になり、同じピースの側面ステッカーは 0°（`NORMAL` 緑）になります。そのため 1 つのコーナーから 1 ステッカー、1 つのエッジから 1 ステッカーだけしかカウントされず、`Math.floor(cornerTwist / 3)` と `Math.floor(edgeFlip / 2)` が群論的なコーナーねじれ数／エッジ反転数と完全に食い違います。
- **修正案**: ピース単位で「3 枚（または 2 枚）のうち少なくとも 1 枚でも基準から外れているピース数」または「Kociemba 定義の U/D 基準向き（`co != 0` / `eo != 0`）＋同一ピース重複排除（`pieceIndices` 活用）」で数え直すか、セル単位の矢印ズレ数としてラベル整合を取る必要があります。

#### [CORR-02] `compareAllSolvers` で 1 解法タイムアウト後に残り全解法が連鎖失敗するバグ

- **該当箇所**: [`web/main.ts:1096-1125`](../web/main.ts#L1096-L1125), [`web/solver-client.ts:106-113, 139-149`](../web/solver-client.ts#L139-L149)
- **原因**: `SolverClient.solve()` のクライアント監視タイマー（`solver-client.ts:106-110`）が発火すると `this.fail(...)` が実行され、`this.worker?.terminate()`・`this.ready = false`・`this.readyPromise = undefined` になります。`compareAllSolvers`（`main.ts:1113-1122`）はそのエラーを `"cancelled"` 以外として `compareEntries` に記録して次ループへ進みますが、`SolverClient` を `restart()` していないため、残りのアルゴリズムはすべて `solver-client.ts:89` で即座に `"エンジンの準備完了をお待ちください。"` エラーになり、しかも比較完了後も `solver.ready === false` のまま操作不能になります。
- **修正案**: `compareAllSolvers` の `catch` 内で `!solver.ready` を検知したら `await solver.restart()` で Worker を復旧（かつタイムアウト時は `status("error", ...)` による `promptReloadForUpdate` 誤発火を避ける）するか、`SolverClient` 側でタイムアウト時に自動復旧可能にする。

#### [CORR-03] `src/c3x3/search.rs` の `depth >= self.max_total` off-by-one バグ

- **該当箇所**: [`src/c3x3/search.rs:73, 141, 178, 235`](../src/c3x3/search.rs#L141)
- **原因**: `L73` の Doc コメントでは「Phase 1 を深さ 20 まで探索すれば…」、また `L356` の `phase1(depth == 0)` では `if self.path.len() > self.max_total { return false; }`（すなわち Phase 1 長さ `== max_total`・Phase 2 長さ `0` の 20 手解を許可）と判定しています。ところが外側ループ 3 箇所（`L141`, `L178`, `L235`）が `if depth >= self.max_total { break; }` となっているため、`max_total == 20`（あるいは `set_max_total(12)`）のときに `depth == max_total` が一切探索されません。`depth > self.max_total` に修正すべきです。

#### [CORR-04] `web/url-params.ts` の `?centers=,,,,,` 誤受理バグ

- **該当箇所**: [`web/url-params.ts:50`](../web/url-params.ts#L50)
- **原因**: `"?centers=,,,,,".split(",")` は長さ 6 の配列 `["", "", "", "", "", ""]` を返し、JavaScript では `Number("".trim())` が `0` と評価されるため、`Number.isInteger(0) && 0 >= 0 && 0 <= 3` をパスして `[0, 0, 0, 0, 0, 0]` として誤受理されます（不正指定警告 `hasInvalidCenters` が立たない）。`/^[0-3]$/` の正規表現チェック、または `trimmed !== ""` 条件を加えるべきです。

#### [CORR-05] `scene.ts` アニメ進捗 `p` 未クランプと `cube-store.ts` 浮動小数点比較

- **該当箇所**: [`web/scene.ts:753`](../web/scene.ts#L753), [`web/cube-store.ts:298`](../web/cube-store.ts#L298)
- **詳細**:
  - `scene.ts:700` で `start = performance.now()` を記録する一方、`setAnimationLoop` のコールバック引数 `time` が別タイムラインや過去タイムスタンプになった場合、`(time - start) / duration` が負になり `Math.min(1, p)` だけでは負のクォータニオン回転（一瞬逆回転する跳ね）が起き得ます。`Math.max(0, Math.min(1, ...))` にすべきです。
  - `cube-store.ts:298` は `nextCenters.some((angle, i) => angle !== this.centerRotations[i])` と浮動小数点（`Math.PI / 2` の整数倍）を直接比較しており、`centerTurns`（整数 `0..3`）で比較する `replace()`（`L165`）と不整合です。

#### [CORR-06] UI 説明と実挙動の乖離（Korf・2x2 既定アルゴリズム）

- [`web/main.ts:127`](../web/main.ts#L127) は Korf の説明に `"IDA* 最短探索 (最大20手)"` と表示しますが、[`src/c3x3/korf.rs:86`](../src/c3x3/korf.rs#L86) の実装上限は `max_depth = 12` であり、ランダム局面は 100% Kociemba フォールバックになります（`"最短探索 (≤12手 + Kociemba補完)"` のように正確化すべき）。
- 2x2 の既定アルゴリズムが [`web/main.ts:91`](../web/main.ts#L91)・[`web/url-params.ts:91`](../web/url-params.ts#L91) では `"lbl"` なのに対し、[`src/lib.rs:662`](../src/lib.rs#L662)・[`web/solver.worker.ts:21`](../web/solver.worker.ts#L21) では `"optimal"` です。

#### [CORR-07] 2026-11-21 レビューの残存セキュリティ／DoS 防御項目

以下は前回レビュー（`docs/code-review-2026-11-21.md`）で指摘されたうち、現時点でまだ反映されていない項目です：

1. **[S4] 計算量上限ガード欠如**:
   - [`src/c3x3/search.rs:311-319`](../src/c3x3/search.rs#L311-L319) と [`web/solver-client.ts:101-113`](../web/solver-client.ts#L101-L113) で `gods_guarantee` 時に Rust 時間予算も Worker タイマーも完全無効（ハード上限ゼロ）。60 秒または上限ノード等のフェイルセーフを設けること（`PERF-02` により通常局面は 1 秒未満で収束するため実用影響なし）。
   - [`src/c2x2/mod.rs:88-95`](../src/c2x2/mod.rs#L88-L95) で 2x2 `lbl` / `ortega` が `budget_ms` を完全に無視している（`budget_ms == 0` 以外でも上限なし）。
2. **[S6] 境界入力検証の残り**:
   - [`web/camera.ts:35`](../web/camera.ts#L35): `MAX_IMAGE_PIXELS = 50_000_000`（展開時 200 MB/枚、2 枚で 400 MB）がモバイル環境で OOM リスク。20〜25 MP 見直し＋`createImageBitmap` 併用を推奨。
   - [`web/camera.ts:130-137`](../web/camera.ts#L130-L137): PNG 判定で 12〜15 バイト目の `IHDR`（`0x49484452`）未検証。
   - [`web/url-params.ts:61-64`](../web/url-params.ts#L61-L64): `?alg=` クエリの長さ上限なし（`algParam.length <= 4096` ガード追加推奨）。
   - [`web/file-io.ts:10, 17-20`](../web/file-io.ts#L10): `fileSize` 引数のみで `content.length > 65536` が未検査、および `in` 演算子の代わりに `Object.hasOwn` を使用すること。
3. **[S5 残り] SW キャッシュの `Cache-Control` 検査**: [`public/sw.js:227-230`](../public/sw.js#L227-L230) で `Cache-Control: no-store` / `private` 応答の除外チェックを追加すること。
4. **[S7] CI テストの動的実行**: [`tests/quality-contracts.spec.ts:20-23`](../tests/quality-contracts.spec.ts#L20-L23) で YAML 式を `new Function(...)` で直接実行している箇所を、安全な式パーサへ置換すること。
5. **[S8] DOM シンク＆エラー詳細露出**: [`web/main.ts:169,186,249,604,705`](../web/main.ts#L604) の `innerHTML` 置換（静的定数のみだが DOM API 化により XSS 攻撃面ゼロ化＋`PERF-04` 改善）、[`web/solver.worker.ts:109`](../web/solver.worker.ts#L109) / [`web/main.ts:1832`](../web/main.ts#L1832) の生エラー・URL の UI 表示抑制、[`Cargo.toml:18,21`](../Cargo.toml#L18-L21) 本番での `console_error_panic_hook` 任意化。
6. **[S9] サプライチェーンピン留め**: [`package.json:37`](../package.json#L37) の `"canvas": "^3.2.3"`（ネイティブ `install` スクリプト付き dev 依存）の完全ピン留め、および CI での `npm audit` / `cargo audit` 導入。

---

## 6. ガードレール 6 大原則（F1〜F6）適合チェック

[`CLAUDE.md`](../CLAUDE.md) および [`docs/code-review-guidelines.md`](code-review-guidelines.md) への準拠状況：

| 原則 | 現行状態 | 指摘との関連・注意点 |
| :--- | :---: | :--- |
| **① [F1] `debug_assertions` 禁止** | **適合** | `check:guardrails` PASS。`src/lib.rs:435`・`src/c2x2/mod.rs:116` で Release 無条件検証実施済み（`SIMP-05` で内部重複のみ整理推奨）。 |
| **② [F2] 複合操作＆ライフサイクル** | **要改善** | `AppStateMachine` / `CubeStore` / `solving` の三重管理（`ARCH-03`, `SIMP-01`）および `compareAllSolvers` タイムアウト後復旧漏れ（`CORR-02`）あり。 |
| **③ [F3] 処理順序と Fail-fast** | **一部要改善** | `file.size` チェックは良好。ただし `file-io.ts` `content.length`、`?alg=` 上限、50 MP リサイズ前フルデコード（`PERF-08`, `CORR-07`）に改善余地。 |
| **④ [F4] URL / パス正規化** | **適合** | `web/storage-key.ts` および `public/sw.js` で `/`, `/index.html`, `/nested/cube/` を統一管理済み。 |
| **⑤ [F5] リソース蓄積・リーク** | **要改善** | Three.js `dispose()` は良好。一方 `public/sw.js` の旧 WASM 移行（`PERF-09`, `SIMP-04`）と `camera.ts` 未キャッシュ `getImageData`（`PERF-08`）を是正すべき。 |
| **⑥ [F6] ビルド成果物鮮度** | **適合** | `scripts/build-manifest.js`（SHA-256 ハッシュ比較）＋`launch-offline.js` で保証済み。 |

---

## 7. 優先度付き実行ロードマップと検証基準

### 7.1 推奨実行フェーズ

| フェーズ | 対象 ID | 主な変更内容 | 期待成果（定量目標） |
| :--- | :--- | :--- | :--- |
| **Phase A**<br>（即効バグ修正＋最大 ROI 高速化＋安全な簡素化） | **CORR-01, CORR-02, CORR-03, CORR-04**<br>**PERF-01, PERF-04, PERF-06(Ortega)**<br>**SIMP-03, SIMP-04, SIMP-06** | - `countPieceOrientations` 修正、`compareAllSolvers` Worker 自動復旧、`depth > max_total` 修正、`?centers=,,,,,` 拒否<br>- `direct_solve` 廃止または座標枝刈り導入<br>- `renderPresetButtons()` 種別変化時限定、アニメ 1 手 `getCellArrowInfo` 1 回化＋テーブル化、`switchCubeType` 二重実行撤去<br>- `solver.worker.ts` 50 行迂回撤去＋SW 旧キャッシュ移行廃止＋`ortega` `solve_pbl`/`solve_first_face` 最適化＋デッドコード削除 | - バグ 4 件解消<br>- 毎手 UI 更新コスト 90% 減<br>- T-perm 等短 G1 解 **8.3 ms → &lt; 0.1 ms**<br>- 2x2 Ortega **1.36 ms → &lt; 0.05 ms**<br>- SW 肥大化停止＋約 **250 行純削減** |
| **Phase B**<br>（コア探索・WASM 削減＋アーキテクチャ分割） | **PERF-02, PERF-03**<br>**SIMP-01, SIMP-02, SIMP-05**<br>**ARCH-01, ARCH-03, ARCH-04** | - 3x3 Kociemba 6 方向インターリーブ統合（センター向き共役含む）＋4-bit 直接表化（対称マップ撤廃）＋`RawCube::multiply` 再生除去<br>- `tables.bin` 4-bit ゼロコピー参照＋Phase 2 10 手生成化＋`WebAssembly.Module` 共有／メイン WASM 分離＋`Cargo.toml` リリース最適化<br>- `web/app-state.ts` 簡素化＋`web/main.ts` コントローラ分割＋`cubes/` 単一元化＋Rust 共通ヘルパー抽出 | - WASM gzip **3.48 MB → ~0.75 MB（約 78% 削減）**<br>- WASM 総メモリ **~18.6 MB → ~4 MB**<br>- 3x3 Kociemba 平均 **160 ms → &lt; 15 ms**、Superflip **3.09 s → &lt; 0.3 s**<br>- センター向き **907 ms (20%失敗) → &lt; 100 ms (0%失敗)**<br>- さらに約 **550 行純削減** |
| **Phase C**<br>（副ソルバー・3D・カメラ・テスト衛生・残存ハードニング） | **PERF-05, PERF-07, PERF-08, PERF-10**<br>**ARCH-02, ARCH-05, ARCH-06, ARCH-07**<br>**SIMP-07, SIMP-08, CORR-05, CORR-06, CORR-07** | - Thistlethwaite 座標＋29 KB 表導入、Korf 3 軸初期下界導入<br>- Three.js オンデマンド描画・`finish()` 対象限定・3x3/2x2 ビルダー統合<br>- `camera.ts` 分割・`ImageData` キャッシュ・ホバー再描画抑止・`createImageBitmap`・`editor.ts` 差分更新<br>- `gods_guarantee` ハード上限・`new Function` 撤去・構造化エラー・残存ハードニング（S4/S6/S7/S8/S9）＋テスト再編 | - Thistlethwaite **812 ms → &lt; 1 ms（失敗 0%）**<br>- Korf 無駄待機 685 ms → ~0 ms<br>- 静止時 GPU/CPU 負荷ゼロ化<br>- カメラ操作 60fps 化・OOM 対策・完全ハードニング |

### 7.2 各フェーズ完了時の必須検証コマンド

```bash
# 1. ガードレール・Rust 品質・Release テスト
npm run check:guardrails
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --release

# 2. Web 型検査・フォーマット・本番ビルド・全 E2E テスト
npm run format:check
npm run typecheck
npm run build
npm test

# 3. オフライン起動・サイズ/速度ベンチマーク確認
npm run offline -- --headless --fresh
npm run benchmark
```
