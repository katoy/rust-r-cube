# 全体コードレビューレポート：速度最適化・セキュリティ重点評価（2026-11-21）

- **実施日**: 2026-11-21（JST）
- **対象リポジトリ**: `rust-r-cube/3x3-web-2`（Cube Studio）
- **重点観点**: **速度アップ（ロード時間・メモリ効率・探索速度・描画/UI 応答）** および **セキュリティ（脅威モデル・CSP/ヘッダ・グローバル情報漏洩・DoS 耐性・Service Worker 境界・入力検証・サプライチェーン）**
- **対象範囲**:
  - Rust ソルバー・座標/テーブル・WASM 境界（`src/lib.rs`, `src/c3x3/*`, `src/c2x2/*`, `build.rs`, `Cargo.toml`, `Cargo.lock`）
  - Web フロントエンド・Web Worker・3D 描画・カメラ認識・PWA（`index.html`, `web/*.ts`, `public/sw.js`, `public/manifest.webmanifest`）
  - ビルド・ガードレール・テスト・スクリプト（`vite.config.ts`, `package.json`, `package-lock.json`, `scripts/*`, `start.sh`, `tests/quality-contracts.spec.ts`）
- **総合判定**: **改善推奨（速度: High 3件 / Medium 5件 / Low 1件、セキュリティ: High 2件 / Medium 5件 / Low 2件）**
  - 自動静的検査（`npm run check:guardrails`, `npm run typecheck`, `npm run format:check`）はいずれも合格しており、基本的な入力検証・Fail-fast・オフライン整合性・依存最小化（本番 JS 依存は `three` のみ）は高水準です。
  - 一方で、本番 WASM（`pkg/cube_studio_bg.wasm`）の実測プロファイリングおよび全コード精読により、**初回ロード・メモリを圧迫する 6.36 MB WASM（3.48 MB gzip）と二重ロード構造（P1）**、**3x3 Kociemba の過剰ノード展開（平均 2,733 万〜1.05 億ノード / Superflip 4.8 億ノード、P2）**、**Worker 起動時間の 95.3% を占める 2x2 テーブル毎起動 BFS（P3）**、さらにセキュリティ面では **CSP / Referrer-Policy 未設定（S1）**、**本番ビルドへの `window.__lastCamera` 無条件漏洩（S2）**、**JPEG ヘッダ解析の無制限ループ DoS（S3）** など、大きな改善余地が特定されました。

---

## 1. 自動チェック・実測ベースライン

### 1.1 静的検査結果

| 検査項目 | コマンド | 結果 |
| :--- | :--- | :---: |
| レビューガードレール検証 | `npm run check:guardrails` | PASS |
| TypeScript 型検査 | `npm run typecheck` | PASS |
| Prettier フォーマット検査 | `npm run format:check` | PASS |

### 1.2 ビルド成果物サイズ（`dist/`・`pkg/` 実測）

| 成果物 | Raw サイズ | Gzip サイズ | 備考 |
| :--- | ---: | ---: | :--- |
| `dist/assets/cube_studio_bg-*.wasm` | 6,356,818 B (6.06 MiB) | 3,484,955 B (3.32 MiB) | うち埋め込み `tables.bin` が **6,111,572 B raw / 3,326,061 B gzip（95.4%）** |
| `dist/assets/three-*.js` | 577,983 B (564 KiB) | 142,709 B (139 KiB) | 本番唯一の外部 JS ライブラリ |
| `dist/assets/index-*.js` | 103,196 B (101 KiB) | 33,034 B (32 KiB) | メインスレッド JS＋WASM グルー |
| `dist/assets/camera-*.js` | 32,185 B (31 KiB) | 10,395 B (10 KiB) | 動的インポート |
| `dist/assets/index-*.css` | 29,067 B (28 KiB) | 6,697 B (6.5 KiB) | スタイルシート |
| `dist/assets/solver.worker-*.js` | 6,790 B (6.6 KiB) | 2,945 B (2.9 KiB) | 探索用 Web Worker |
| `dist/assets/editor-*.js` | 5,603 B (5.5 KiB) | 2,140 B (2.1 KiB) | 動的インポート |

#### 埋め込み `tables.bin`（6,111,572 B raw / 3,326,061 B gzip）の内訳

| セクション | Raw サイズ | Gzip サイズ | 4-bit 化後 Raw / Gzip | 最大値・未使用率 |
| :--- | ---: | ---: | ---: | :--- |
| `MoveTable.twist` (2,187×18) | 78,732 B | 53,614 B | — | 全 18 手使用 |
| `MoveTable.flip` (2,048×18) | 73,728 B | 45,178 B | — | 全 18 手使用 |
| `MoveTable.ud_slice` (495×18) | 17,820 B | 9,583 B | — | 全 18 手使用 |
| `MoveTable.cp` (40,320×18) | **1,451,520 B** | **1,289,916 B** | — | Phase 2 専用（18 手中 **8 手 = 44.4% が完全未使用**） |
| `MoveTable.ep8` (40,320×18) | **1,451,520 B** | **1,255,095 B** | — | Phase 2 専用（18 手中 **8 手 = 44.4% が完全未使用**） |
| `MoveTable.slice_p` (24×18) | 864 B | 353 B | — | Phase 2 専用（8 手未使用） |
| `PruningTable.twist_slice` | 561,330 B | 130,955 B | 280,665 B / 120,898 B | `maxVal = 9`（4 bit 格納可） |
| `PruningTable.flip_slice` | 522,720 B | 116,797 B | 261,360 B / 108,633 B | `maxVal = 9`（4 bit 格納可） |
| `PruningTable.cp_slice` | 967,680 B | 183,835 B | 483,840 B / 158,589 B | `maxVal = 14`（4 bit 格納可） |
| `PruningTable.ep8_slice` | 967,680 B | 229,644 B | 483,840 B / 198,933 B | `maxVal = 12`（4 bit 格納可） |

### 1.3 本番 WASM 実行速度ベースライン（Node.js v20 + `pkg/cube_studio_bg.wasm`）

- **WASM インスタンス化**: 7.7〜14.0 ms
- **`initialize()` 合計**: **約 178〜270 ms**
  - 内訳: 3x3 テーブルデコード＋FNV-1a 検証 **約 8.5 ms（4.7%）** / 2x2 4 グループ BFS 生成 **約 171.0 ms（95.3%）**

| 対象 / 条件 | 局面数 | 平均時間 | 中央値 / 最大時間 | 平均ノード数 | 平均手数 / 最大手数 | 備考 |
| :--- | :---: | ---: | ---: | ---: | :---: | :--- |
| 3x3 Kociemba（色のみ・20手保証） | 20 | **371.4 ms** | 315.6 / 769.6 ms | 27,332,345 | 19.6 / 20 | 予算 3,000 ms |
| 3x3 Kociemba（センター向き込み） | 20 | **1,158.0 ms** | 768.0 / 3,000.8 ms | 105,375,642 | 35.0 / 90 | **4/20（20%）が逐次解へ後退** |
| 3x3 Superflip（色のみ・20手保証） | 1 | **5,591.5 ms** | — | **480,765,952** | 20 | 予算超過して延長実行 |
| 3x3 Superflip（センター向き込み） | 1 | **3,129.9 ms** | — | 307,539,877 | 84 | タイムアウト後・逐次解 |
| 3x3 CFOP（色のみ） | 20 | 2.3 ms | 2.5 / 4.9 ms | 156,846 | 108.3 / 134 | 高速 |
| 3x3 Thistlethwaite（色のみ） | 20 | **941.1 ms** | 773.7 / 3,000.2 ms | 53,719,693 | 29.8 / 33 | **3/20（15%）がフォールバック** |
| 3x3 Korf IDA*（色のみ） | 20 | **885.2 ms** | 878.4 / 2,067.2 ms | 49,157,665 | 19.6 / 20 | **20/20（100%）がフォールバック** |
| 2x2 Antipode 11手（Optimal OFF/ON） | 1 | **310.7 / 310.2 ms** | — | 3,735,583 | 11 | `UUDBDLFUFRLLBDDFBRLUFRRB` |
| 2x2 ランダム 6 面（Optimal OFF/ON） | 50 | 6.2 / 6.2 ms | — / 118.4 ms | — | — | 6 面探索による枝分かれ増 |
| 2x2 ランダム 6 面（LBL） | 50 | **27.4 ms** | — / **150.0 ms** | 0 | — | `gods-number-11` で 88〜108 ms |
| 2x2 ランダム 6 面（Ortega） | 50 | 1.3 ms | — / 9.5 ms | 0 | — | 高速 |

---

## 2. 速度アップ（Performance）レビュー指摘事項

### 2.1 指摘サマリー（速度）

| ID | 重大度 | 領域 | 対象ファイル（行番号） | 概要と期待効果 |
| :--- | :---: | :--- | :--- | :--- |
| **P1** | **High** | 初期ロード / メモリ / WASM | [`src/lib.rs`](../src/lib.rs) (L13)<br>[`src/c3x3/table_io.rs`](../src/c3x3/table_io.rs) (L8-104)<br>[`src/c3x3/tables.rs`](../src/c3x3/tables.rs) (L10-48)<br>[`web/main.ts`](../web/main.ts) (L1571, L1714)<br>[`web/solver.worker.ts`](../web/solver.worker.ts) (L15-16)<br>[`Cargo.toml`](../Cargo.toml) (L23-25) | WASM が 6.36 MB raw / 3.48 MB gzip に肥大化（`cp`/`ep8` だけで gzip の 73%）。さらにメインスレッドと Worker で直列に二重ロード・ヒープ複製（計 ~18.6 MB）。4-bit パック＋`MoveTable` 縮小/生成＋二重ロード解消で **転送サイズ最大 78% 削減・メモリ約 75% 削減**。 |
| **P2** | **High** | 3x3 Kociemba 探索 | [`src/c3x3/search.rs`](../src/c3x3/search.rs) (L131-191, L328-488)<br>[`src/c3x3/tables.rs`](../src/c3x3/tables.rs) (L128-150) | 深さ 0..=12 の早期終了なし全探索、UD 軸二重走査、G1 早期到達枝刈り欠落、Phase 1→2 ループ内での座標 13〜19 回再計算、未インラインの 4 配列間接枝刈り参照。修正により **通常局面 10〜30 倍高速化（371 ms → ~15 ms）、Superflip 5.6 秒 → < 0.5 秒**。 |
| **P3** | **High** | Worker 起動 / 再起動 | [`src/lib.rs`](../src/lib.rs) (L532)<br>[`src/c2x2/tables.rs`](../src/c2x2/tables.rs) (L15-53)<br>[`web/solver-client.ts`](../web/solver-client.ts) (L128-133) | `initialize()` が毎回 2x2 用 4 グループ BFS（計 980 万回乗算、約 171 ms = 起動時間の **95.3%**）を実行。遅延初期化により **3x3 起動・キャンセル再起動を 179 ms → 8.5 ms（約 21 倍高速化）**。 |
| **P4** | **Medium** | 2x2 ソルバー | [`src/c2x2/search.rs`](../src/c2x2/search.rs) (L71-85, L120-163)<br>[`src/c2x2/coord.rs`](../src/c2x2/coord.rs) (L222-248)<br>[`src/c2x2/lbl.rs`](../src/c2x2/lbl.rs) (L78-187, L250, L302)<br>[`src/c2x2/ortega.rs`](../src/c2x2/ortega.rs) (L167, L239) | Optimal（向き OFF）が DBL 固定後も 6 面全探索（3 面 `<U,R,F>` 限定で **100 倍以上高速化**）。16 回線形探索の逆置換統合、LBL 第 1 層の `group_b` 距離表活用（150 ms → < 0.05 ms）、定石 `parse_moves` の `OnceLock` 化。 |
| **P5** | **Medium** | 3x3 Thistlethwaite / Korf | [`src/c3x3/thistlethwaite.rs`](../src/c3x3/thistlethwaite.rs) (L182-447)<br>[`src/c3x3/korf.rs`](../src/c3x3/korf.rs) (L58-167) | 両者とも既存の `MoveTable`/`PruningTable` を活かしきれず `RawCube::multiply` で探索。Thistlethwaite は Phase 3 枝刈り欠落（平均 941 ms）、Korf は 100% フォールバック前に平均 885 ms 浪費。 |
| **P6** | **Medium** | UI / 3D 描画ループ | [`web/main.ts`](../web/main.ts) (L368, L559-588, L685)<br>[`web/scene.ts`](../web/scene.ts) (L56-67, L456, L494, L588-624, L713-729) | 非表示 `#fallback-net` の毎手 66 DOM 再構築、`getCellArrowInfo` の 1 画面更新あたり二重計算＋毎手 300 個超の一時 Vector/Matrix 生成、`#solve`/`#play` の毎回 `innerHTML` 再パース、静止時 60fps ループ継続とステッカー 54 枚の不要シャドウ描画。 |
| **P7** | **Medium** | カメラ認識 UI | [`web/camera.ts`](../web/camera.ts) (L264-294, L749-790, L862, L1021-1065)<br>[`web/camera-ui-helper.ts`](../web/camera-ui-helper.ts) (L13, L55) | ホバー毎ピクセルでの Canvas 全再描画と `getBoundingClientRect()` 二重リフロー、`updateDetectedLabels` ごとの 7.7 MB `getImageData` 未キャッシュ読み戻し＋不要な 24 マス余剰サンプリング、画像リサイズ時の二重デコード。 |
| **P8** | **Medium** | 比較実行 / PWA キャッシュ | [`web/main.ts`](../web/main.ts) (L1073-1102)<br>[`public/sw.js`](../public/sw.js) (L95-139) | 「全解法を比較」の単一 Worker 直列実行による待機累積。また SW `activate` が旧ハッシュの 6.1 MB WASM 等を新キャッシュへ最大 15 件コピーし続けキャッシュ肥大化＆起動 I/O 悪化。 |
| **P9** | **Low** | WASM 境界 / バンドル | [`src/lib.rs`](../src/lib.rs) (L37-63, L476-484)<br>[`web/main.ts`](../web/main.ts) (L885, L1307, L1660, L1761, L1851) | `apply_moves` が呼び出し側で未使用の中間 `states` を全件 JSON 化（最大 125 KB）。また `three`（143 KB gzip）の名前付きインポート整理と WASM `<link rel="preload">` の未設定。 |

---

### 2.2 速度アップ詳細と改善案

#### [P1] WASM バイナリ肥大化（6.36 MB raw / 3.48 MB gzip）と二重ロード・メモリ重複

- **該当箇所**: [`src/lib.rs:13`](../src/lib.rs#L13), [`src/c3x3/tables.rs:10-48`](../src/c3x3/tables.rs#L10-L48), [`src/c3x3/table_io.rs:8-104`](../src/c3x3/table_io.rs#L8-L104), [`src/c3x3/coord.rs:343-402`](../src/c3x3/coord.rs#L343-L402), [`web/main.ts:1571,1714`](../web/main.ts#L1571), [`web/solver.worker.ts:15-16`](../web/solver.worker.ts#L15-L16), [`Cargo.toml:23-25`](../Cargo.toml#L23-L25)
- **ボトルネック分析**:
  1. **ネットワーク転送量の 73% が Phase 2 用の圧縮困難な `MoveTable.cp` / `ep8`**:
     - `cp`（40,320×18×2 B = 1.45 MB / gzip 1.29 MB）と `ep8`（1.45 MB / gzip 1.26 MB）の 2 表だけで gzip 済み WASM 全体の **73.0%（2.54 MB）** を占めます。
     - しかも `cp`・`ep8`・`slice_p` は Phase 2 の 10 手（`U, U2, U', R2, F2, D, D2, D', L2, B2`）でしか参照されず、残りの 8 列（44.4% = 1.29 MB raw）は完全なデッドデータです。
  2. **枝刈りテーブルの 8-bit 格納による 2 倍の容量・キャッシュ消費**:
     - 実測のとおり 4 表の最大距離は `twist_slice=9`, `flip_slice=9`, `cp_slice=14`, `ep8_slice=12` であり、すべて 4 bit（`0..=14`, `15=未到達`）に収まります。4-bit パック化で **3.02 MB → 1.51 MB raw（キャッシュヒット率も向上）** になります。
  3. **静的セグメント＋ヒープ複製の二重メモリ消費と、メイン/Worker の直列二重ロード**:
     - `include_bytes!` の 6.11 MB が WASM 静的メモリに常駐したまま、`decode_moves` / `decode_pruning` がさらに 6.11 MB をヒープへ複製します（Worker 単体で 12.2 MB）。加えて `ensure_table_verified` が静的バイナリ自身に対して起動ごとに 6.11 MB の FNV-1a チェックサムを走査します。
     - さらに `web/main.ts:1571` がメインスレッドで 6.36 MB WASM を `await init(...)` でロード・インスタンス化し（実際に使うのは `apply_moves`, `validate`, `scramble`, `center_parity`, `is_valid` のみで `tables.bin` は一切不使用）、その完了を待ってから `L1714` で `SolverClient` を生成して Worker が同じ 6.36 MB WASM を再度フェッチ・コンパイル・インスタンス化します（合計 WASM メモリ約 **18.6 MB**）。
  4. **Release プロファイル最適化余地**: `Cargo.toml:23-25` に `codegen-units = 1`, `panic = "abort"`, `strip = true` がなく、`dist/index.html` に WASM の `<link rel="preload">` もありません。
- **改善案**:
  1. **`tables.bin` のスリム化**:
     - `PruningTable` 4 表を 4-bit パック化し、WASM 静的スライス（`&'static [u8]`）から **ゼロコピー参照** する（ヒープ複製 3.02 MB をゼロ化）。
     - `MoveTable.cp` / `ep8` / `slice_p` は Phase 2 の 10 手のみに限定し、さらに `coord.rs:344,384` の `set_cp` / `set_ep8` から `Vec::remove` のヒープ確保を排除してスタック配列化すれば、Worker 起動時生成でも約 10〜15 ms で構築でき、**WASM バイナリ全体を 6.36 MB (3.48 MB gzip) → 約 1.75 MB (~750 KB gzip、約 78% 削減)** へ圧縮可能です。
  2. **二重ロード＆直列ウォーターフォール解消**:
     - 短期的には `main.ts` で `WebAssembly.compileStreaming(fetch(wasmUrl))` を実行し、コンパイル済み `WebAssembly.Module` を Worker に `postMessage` 転送して 2 回目のフェッチ・コンパイルを排除する。
     - 中期的にはメインスレッドで使う軽量操作（盤面検証・回転適用・スクランブル・パリティ計算）を TypeScript 化または `tables.bin` 非同梱の軽量 WASM ターゲットに分離し、メインスレッドの 6.4 MB WASM 読み込み自体を撤廃する。
  3. **`Cargo.toml` リリース最適化**: `[profile.release]` に `codegen-units = 1`, `panic = "abort"`, `strip = true` を追加し、`wasm-opt -O3` の適用を確認する。

#### [P2] 3x3 Kociemba 探索の過剰ノード展開（平均 2,733 万〜1.05 億ノード、Superflip 4.8 億ノード）

- **該当箇所**: [`src/c3x3/search.rs:131-191,328-488`](../src/c3x3/search.rs#L131-L191), [`src/c3x3/tables.rs:128-150`](../src/c3x3/tables.rs#L128-L150)
- **ボトルネック分析**:
  1. **Loop 1（`search.rs:131-149`）の早期終了欠落・UD 軸偏重・重複探索**:
     - `for depth in 0..=12` が `self.has_gods_solution()` で終了しないため、深さ 8〜9 で 20 手以内の解を見つけても深さ 10・11・12 を UD 軸だけで全走査し続けます（平均 2,733 万ノード / 371 ms）。センター向き込みでも 24 手の解を見つけた後に深さ 10〜12 を全走査し平均 1.05 億ノード / 1.16 秒消費、20% がタイムアウトします。
     - さらに 20 手以内の解が見つからなかった場合、Loop 2（`search.rs:155-191`）が 6 方向リストに `(axis=0, inverse=false)` を再度含めて深さ 0 から重複探索しています。
  2. **Phase 1 での G1 早期到達の未枝刈り（`search.rs:328-341`）**:
     - `depth > 0 && twist == 0 && flip == 0 && slice == 0` を棄却していないため、すでに G1 に到達したノードから Phase 2 の手を交えて再び G1 へ戻る大量の重複葉が展開されます。
  3. **Phase 1→2 境界での重複座標計算と無駄ループ（`search.rs:357-391`）**:
     - Phase 1 の全葉で `self.path` 全手を `RawCube::multiply` でリプレイした上、以下のように `for d in min_d..=max_d` **ループの内側で** `cube.get_cp()`, `cube.get_ep8()`, `cube.get_slice_p()` を最大 13〜19 回も再計算しています。
     ```rust
     for d in min_d..=max_d {
         if self.phase2(
             cube.get_cp(),
             cube.get_ep8(),
             cube.get_slice_p(),
             d as u8,
             last,
         ) { ... }
     }
     ```
     - さらに `min_d` が `0` から始まるため、Phase 2 枝刈り下界 `h2 = max(cp_slice, ep8_slice)` が `max_d` を超えて即棄却される葉でも `d = 0..=max_d` の全反復で座標再計算＋`phase2` 呼び出しが空回りしています。
  4. **`get_twist_slice` / `get_flip_slice` の 4 配列間接参照と未インライン（`tables.rs:128-150`）**:
     - 1 探索あたり数千万〜数億回呼ばれる最ホット関数に `#[inline(always)]` がなく、毎回 `twist_class` → `twist_self_sym` → `twist_sym` → `ud_slice_x2` → `twist_slice` の最大 4〜5 回のメモリ参照と 2 回の分岐が発生しています。
     - 4-bit 直接テーブル（対称圧縮なし: `2187×495/2 = 541 KB` + `2048×495/2 = 507 KB` = **1.05 MB**）なら現行の対称テーブル（1.08 MB）より小さく、**分岐ゼロ・メモリ参照 1 回**で取得できます。
- **改善案**:
  1. `search.rs:376-383` の `cp`, `ep`, `sp` 算出をループ外へ 1 回だけ計算するよう移動し、`let h2 = self.pt.cp_slice[...].max(self.pt.ep8_slice[...]) as usize; if h2 > max_d { return false; }` で即座に枝刈り、`for d in min_d.max(h2)..=max_d` とする（`cp` は Phase 1 中も `MoveTable.cp` で O(1) 追跡すれば `RawCube::multiply` 前に事前枝刈り可能）。
  2. `target_centers.is_none()` 時は `depth > 0 && twist == 0 && flip == 0 && slice == 0` を即枝刈りし、Phase 1 の最後の手が Phase 2 許可手かつすでに G1 にいた経路を排除する。
  3. 6 方向探索（UD/RL/FB × 順・逆）を深さ 0 からインターリーブ実行し、色のみ時は `has_gods_solution()` 到達時点で（あるいは短い改善猶予予算後）即時終了する。
  4. `get_twist_slice` / `get_flip_slice` を `#[inline(always)]`＋4-bit 直接参照化し、`phase1` / `phase2` の子ノード展開前に `distance > depth - 1` で先読み枝刈りする。

#### [P3] Worker `initialize()` の 95.3%（約 171 ms）を占める 2x2 PruningTable 毎起動 BFS

- **該当箇所**: [`src/lib.rs:532`](../src/lib.rs#L532), [`src/c2x2/tables.rs:15-53`](../src/c2x2/tables.rs#L15-L53), [`web/solver-client.ts:128-133`](../web/solver-client.ts#L128-L133)
- **ボトルネック分析**:
  - `initialize()` が無条件に `c2x2::tables::PruningTable::get()` を呼び出し、136,080 状態 × 4 グループ × 18 手（約 980 万回の `RawCube::multiply` と `get_group_idx`）を毎回実行しています。
  - 実測で `initialize()` 全体 **179.5 ms** のうち 3x3 復元はわずか **8.5 ms**、残り **171.0 ms（95.3%）** が 2x2 テーブル生成です。3x3 しか使わないユーザーや、探索キャンセルによる Worker 再起動（`solver-client.ts:131`）のたびに 171 ms ブロックされます。
- **改善案**:
  1. `initialize()` から `c2x2::tables::PruningTable::get()` を外し、2x2 Optimal（または LBL）の初回呼び出し時に `OnceLock` で遅延初期化する。
  2. さらに Group A↔B・Group C↔D の対称性利用または 4-bit パック済み 2x2 テーブル（計 272 KB raw / ~60 KB gzip）の `tables.bin` 同梱を検討する。

#### [P4] 2x2 ソルバーの探索効率（Optimal 向き OFF 6 面探索・16 回線形探索・LBL 150 ms 遅延・定石再パース）

- **該当箇所**: [`src/c2x2/search.rs:71-85,120-163`](../src/c2x2/search.rs#L120-L163), [`src/c2x2/coord.rs:222-248`](../src/c2x2/coord.rs#L222-L248), [`src/c2x2/lbl.rs:78-187,250,302`](../src/c2x2/lbl.rs#L78-L187), [`src/c2x2/ortega.rs:167,239`](../src/c2x2/ortega.rs#L167)
- **ボトルネック分析**:
  1. **Optimal（向き OFF）の不要な 6 面探索**: `c2x2/search.rs:71-84` で DBL コーナーを位置 6・向き 0 に固定したにもかかわらず、`ida_star`（`L139`）が `for face in 0..6` で D/L/B も探索しています。DBL 固定後の最短解は `<U, R, F>`（3 面 9 手、分岐係数 6）だけで数学的に完全同一（最大 11 手）であり、6 面探索（分岐係数 ~13.3）により 11 手アンチポードで **310.7 ms / 373.6 万ノード** かかっています。
  2. **ノードあたり 16 回の線形探索**: `get_group_a/b/c/d_idx` が毎回 `rc.cp.iter().position(...)` を 4 回ずつ計 16 回実行しています。`let mut inv = [0usize; 8]; for i in 0..8 { inv[rc.cp[i] as usize] = i; }` を 1 回だけ計算すれば 4 グループすべての位置が O(1) で取得できます。
  3. **2x2 LBL 第 1 層の BFS 状態空間爆発**: `lbl.rs:179` の `hash_first_layer` が全 8 スロットをハッシュ化するため U 層の配置違いがすべて別状態扱いとなり、`Vec::clone` を伴う大量のキュー展開が発生（実測平均 **27.4 ms・最大 150.0 ms**）。また深さ 7 以上のフォールバック `solve_first_layer_ida`（`L135-175`）は深さ 1 から枝刈りなし・タイムアウトなしで再探索します。既に存在する `get_group_b_idx`（136,080 状態）と `PruningTable::get().group_b`（完全 1 層までの厳密最短距離表）を使えば **< 0.05 ms** で即解できます。
  4. **定石文字列のループ内再パース**: `lbl.rs:250,302` および `ortega.rs:167,239` で毎回 `cube::parse_moves(alg_str)` を実行（Ortega PBL では最大 128 回/解）。`OnceLock` で事前パース済み配列を持つべきです。

#### [P5] 3x3 Thistlethwaite（平均 941 ms）と Korf（100% フォールバック / 平均 885 ms）の高速化

- **該当箇所**: [`src/c3x3/thistlethwaite.rs:182-447`](../src/c3x3/thistlethwaite.rs#L182-L447), [`src/c3x3/korf.rs:58-167`](../src/c3x3/korf.rs#L58-L167)
- **ボトルネック分析**:
  - **Thistlethwaite**: 全 4 フェーズが `RawCube::multiply` に依存し、Phase 3（`G2→G3`）は枝刈りゼロのため実測平均 **941.1 ms**（最大 3,000.2 ms、3/20 がフォールバック）。既に常駐する `MoveTable`（`flip`, `twist`, `ud_slice`, `cp`, `ep8`, `slice_p`）と `PruningTable` を Phase 1・2・4 に適用し、Phase 3 に小型の軌道分離距離表を導入することで数 ms〜数十 ms へ短縮できます。
  - **Korf (IDA*)**: ランダム 25 手スクランブルでは 20/20（100%）が `max_depth = 12` 内で解けず、平均 **885.2 ms**（4,916 万ノード）を浪費してから Kociemba にフォールバックしています。3 軸共役（UD/RL/FB）の `twist_slice`/`flip_slice` 最大値をヒューリスティックに採用し、初期下界が `max_depth` を超えた場合は即座に Kociemba へ移行するだけでも無駄な待機時間を大幅に削減できます。

#### [P6] `refresh()` の過剰 DOM 再構築・`getCellArrowInfo` 二重計算・3D 描画負荷

- **該当箇所**: [`web/main.ts:368,559-588,685`](../web/main.ts#L557-L588), [`web/scene.ts:56-67,456,494,588-624,713-729`](../web/scene.ts#L588-L624), [`web/model.ts:212-408`](../web/model.ts#L212-L408)
- **ボトルネック分析**:
  1. **非表示 `#fallback-net` の毎回再生成**: `web/main.ts:562-571` は WebGL 3D が有効で `$("fallback").hidden === true` のときでも、毎手・毎ステップ 66 DOM 要素を破棄・再作成しています。`if (!inMotion && !$("fallback").hidden)` の 1 条件で回避可能です。
  2. **`getCellArrowInfo` の二重実行と一時オブジェクト大量生成**: 1 回の `refresh()` 内で `scene.show()`（`scene.ts:588`）と `updateArrowLegend()`（`main.ts:368`）が同じ `getCellArrowInfo(state, centerRotations)` を 2 回計算しています。さらに `scene.ts:56-67,601-623` が 54 セル＋6 センター更新のたびに約 300 個以上の `THREE.Vector3` / `Matrix4` / `Quaternion` を新規確保しています。結果を 1 回だけ計算して共有し、作業用 `Vector3`/`Matrix4` を使い回すことで GC 負荷を劇的に削減できます。
  3. **`innerHTML` の毎回再パース**: `main.ts:587`（`#solve`）と `main.ts:685`（`#play`）が状態変化の有無にかかわらず毎回 `innerHTML` を書き換えて SVG DOM を再生成しています。
  4. **3D レンダリング最適化**: `scene.ts:456,494` で薄いステッカー 54 枚すべてに `castShadow = true` が設定されておりシャドウパスで 54 ドローコール増えています（本体キュービーのみで十分）。また `setAnimationLoop`（`L219, L713-729`）が静止時も毎フレーム `controls.update()` を回し続けるため、操作・アニメーション終了時にループを止めるオンデマンド描画化でバッテリー消費を削減できます。

#### [P7] カメラ入力モーダルのホバー再描画・強制リフロー・7.7 MB `getImageData` 未キャッシュ

- **該当箇所**: [`web/camera.ts:264-294,749-790,862,1021-1065`](../web/camera.ts#L264-L294), [`web/camera-ui-helper.ts:13,55`](../web/camera-ui-helper.ts#L13)
- **ボトルネック分析**:
  1. `handlePointerMove`（`camera.ts:279-283`）が、`hoverIndex` が変化していない通常ホバー時でも 1 ピクセル動くたびに背景画像込みで `this.draw()` を全面再描画し、`toCanvasCoords` と `findHitTarget` で `getBoundingClientRect()` を 1 イベントあたり 2 回呼んでいます。`prevHover !== hit` のときのみ再描画し、`rect` を 1 回だけ取得するよう修正すべきです。
  2. `updateDetectedLabels()`（`camera.ts:1049-1052`）が頂点ドラッグ完了・枠回転のたびに新規 `<canvas>` を生成し `getImageData()`（1600×1200 で約 7.7 MB の GPU→CPU 転送）を実行し、さらにセンター 3 マスの色しか使わないのに 27 マスすべてをサンプリングしています。画像読込時に `ImageData` を 1 回キャッシュし、ラベル推定は `(0.5, 0.5)` の 3 点のみサンプリングすべきです。
  3. 1600px 超画像の縮小処理（`camera.ts:749-790`）が `new Image()` フルデコード → Canvas 縮小 → `toBlob("image/jpeg")` 再エンコード → `new Image()` 再デコードを行っています。`createImageBitmap(file, { resizeWidth, resizeHeight })` の活用で二重デコードとメモリピークを回避できます。

#### [P8] `compareAllSolvers` の直列実行と Service Worker の旧 6.1 MB WASM キャッシュ移行肥大化

- **該当箇所**: [`web/main.ts:1073-1102`](../web/main.ts#L1073-L1102), [`public/sw.js:95-139`](../public/sw.js#L95-L139)
- **ボトルネック分析**:
  1. `compareAllSolvers` が全アルゴリズムを単一 Worker で直列実行するため、3x3 比較の総待ち時間が 4 アルゴリズムの合算（現状 3〜6 秒）になります。P2・P5 の探索高速化とあわせ、高速な CFOP / Kociemba を先行表示し、必要なら複数 Worker 並列化を検討する価値があります。
  2. `public/sw.js:101-139` の `activate` 時キャッシュ移行処理は、新バージョンが既に全資産を `PRECACHE_ASSETS` にプリキャッシュ済みにもかかわらず、ハッシュ名の異なる旧資産（**旧 6.1 MB WASM・旧 564 KB `three.js` を含む**）を最大 15 件まで新キャッシュへ複製し続けます。結果としてデプロイを重ねるごとに数十 MB の不要 WASM が永続蓄積し、ストレージ消費とキャッシュ探索コストが増大します。また `sortedOldKeys` はランダムな SHA-256 10 桁を辞書順ソートしているため「直近世代」の選択自体が機能していません。旧キャッシュはそのまま削除するか、どうしても移行が必要な場合も `.wasm` のような巨大ファイル・2 世代以上前の資産を除外すべきです。

#### [P9] `apply_moves` の未使用中間状態 JSON 生成とアセットプリロード

- **該当箇所**: [`src/lib.rs:37-63,476-484`](../src/lib.rs#L37-L63), [`web/main.ts:885,1307,1660,1761,1851`](../web/main.ts#L885), [`index.html`](../index.html)
- **詳細**:
  - `web/main.ts` の `apply_moves` 呼び出し 5 箇所はいずれも `result.state` と `result.moves` のみ使用し `result.states` を破棄していますが、4,095 B（2,048 手）の手順適用では 2,049 局面を含む **125,131 B** の JSON が毎回生成・シリアライズ・パースされます。最終局面のみ返す `apply_moves_final` を用意すれば不要なアロケーションを削減できます。
  - `vite.config.ts` / ビルド出力 HTML に WASM の `<link rel="preload" as="fetch" crossorigin>` を付与すると、JS 実行を待たずに WASM ダウンロードを先行開始でき初期表示が短縮します。

---

## 3. セキュリティ（Security & Hardening）レビュー指摘事項

### 3.1 脅威モデル（STRIDE / 信頼境界）

本アプリはバックエンド API・DB・認証を持たないブラウザ完結型 SPA（GitHub Pages 等の静的ホスティングおよびサブディレクトリ配信対応）です。主な資産と外部信頼境界は以下のとおりです。

- **保護対象資産**: ユーザーのライブカメラ映像（`getUserMedia`）・アップロード写真、ブラウザ保存データ（`localStorage` / Cache Storage）、タブの可用性（CPU/メモリ）、同一オリジン上の他アプリ（GitHub Pages 共有オリジン配下の別リポジトリ等）。
- **外部信頼境界**: (1) 共有 URL クエリパラメータ（`?state=`, `?alg=`, `?centers=`, `?solver=`, `?no-sw`）、(2) 読込 JSON ファイル、(3) カメラ／ドロップ画像ファイル、(4) Service Worker キャッシュとブラウザ実行環境（CSP・グローバルスコープ）、(5) CI 実行環境・依存パッケージ。

### 3.2 指摘サマリー（セキュリティ）

| ID | 重大度 | STRIDE / 分類 | 対象ファイル（行番号） | 概要 |
| :--- | :---: | :--- | :--- | :--- |
| **S1** | **High** | T / I / EoP<br>セキュリティ設定 | [`index.html`](../index.html) (L1-20) | Content Security Policy (CSP) および `Referrer-Policy` が完全未設定。静的ホスティングで唯一の多層防御手段が欠如し、共有 URL クエリ漏洩や将来 XSS 時の外部データ流出を遮断できない。 |
| **S2** | **High** | I / EoP<br>権限・情報漏洩 | [`web/camera.ts`](../web/camera.ts) (L488)<br>[`web/main.ts`](../web/main.ts) (L48-57) | `window.__lastCamera = this` が本番ビルドで無条件公開（ライブカメラ `MediaStream` や撮影画像・操作メソッドがグローバル参照可能）。`__TEST_ENV__` 判定も本番バンドルに残存。 |
| **S3** | **Medium** | D (DoS)<br>入力解析 | [`web/camera.ts`](../web/camera.ts) (L67-105) | `readJpegDimensions` の `for (;;)` に反復上限がなく、20 MB の `0xFF` 細工ファイルで約 2,000 万回の非同期反復（実測 1 MB で 80.8 ms → 20 MB で ~1.6 秒以上・モバイルでフリーズ）。 |
| **S4** | **Medium** | D (DoS)<br>計算量無制限 | [`web/solver-client.ts`](../web/solver-client.ts) (L101-113)<br>[`src/c3x3/search.rs`](../src/c3x3/search.rs) (L297-310)<br>[`src/c2x2/mod.rs`](../src/c2x2/mod.rs) (L88-95)<br>[`src/c2x2/lbl.rs`](../src/c2x2/lbl.rs) (L135-175) | 3x3 `gods_guarantee` で Rust 予算もクライアント監視タイマーも完全無効化（ハード上限なし）。2x2 LBL/Ortega も `budget_ms` 未参照かつ `solve_first_layer_ida` が上限なし探索。 |
| **S5** | **Medium** | D / T / I<br>PWA / SW 境界 | [`public/sw.js`](../public/sw.js) (L183-241)<br>[`web/pwa.ts`](../web/pwa.ts) (L17-33) | SW がスコープ外（共有オリジン上の別パス）や `206`/`no-store` 応答・任意クエリ URL をキャッシュし得る。また本番で `?no-sw` 外部リンクを踏むと SW が無断解除される。 |
| **S6** | **Medium** | D (DoS)<br>境界バリデーション | [`web/camera.ts`](../web/camera.ts) (L35, L119-125, L725-797)<br>[`web/url-params.ts`](../web/url-params.ts) (L61-64)<br>[`web/file-io.ts`](../web/file-io.ts) (L10) | 50 MP 画像上限（展開時 200 MB/枚）によるモバイル OOM リスク、PNG `IHDR` 型未確認、`?alg=` の長さ無制限コピー、`validateAndParseCubeJson` の `content.length` 未検査。 |
| **S7** | **Medium** | T / EoP<br>CI 動的コード実行 | [`tests/quality-contracts.spec.ts`](../tests/quality-contracts.spec.ts) (L17-23) | GitHub Actions ワークフロー YAML から抽出した文字列を `new Function(...)` で動的実行しており、設定ファイル改変時のコード実行経路となっている。 |
| **S8** | **Low** | T / I<br>DOM シンク / エラー露出 | [`web/main.ts`](../web/main.ts) (L171, L188, L251, L587, L685, L1809)<br>[`web/solver.worker.ts`](../web/solver.worker.ts) (L109)<br>[`Cargo.toml`](../Cargo.toml) (L18, L21) | `innerHTML` 代入箇所の残存、UI エラー文への生例外・URL 露出、`console_error_panic_hook` の本番デフォルト有効化による内部ソースパス露出。 |
| **S9** | **Low / Info** | サプライチェーン / 防御衛生 | [`package.json`](../package.json) (L30-41)<br>[`web/file-io.ts`](../web/file-io.ts) (L17-20) | `canvas: "^3.2.3"` の範囲指定（ネイティブ `install` スクリプトあり）、`file-io.ts` の `in` 演算子、CI での定期 `npm audit` / `cargo audit` 未構成。 |

---

### 3.3 セキュリティ詳細と対策案

#### [S1] Content Security Policy (CSP) および `Referrer-Policy` の完全欠落

- **該当箇所**: [`index.html:1-20`](../index.html#L1-L20)（ビルド後 `dist/index.html`）
- **リスク**:
  - 本アプリは GitHub Pages 等の静的配信を前提としており HTTP レスポンスヘッダをサーバー側で制御できないケースが多いため、`<meta http-equiv="Content-Security-Policy">` がブラウザで強制可能な唯一の CSP となります。
  - 現状は CSP が未設定のため、万一 DOM XSS や拡張機能・共有オリジン経由のコード混入が発生した際、外部サーバーへの盤面・カメラ画像データ送信（`fetch` / `img` / `WebSocket`）や外部スクリプト読み込みをブラウザ側で抑止できません。
  - また `Referrer-Policy` 未設定のため、共有 URL（`?state=...&centers=...`）で開いた直後（`history.replaceState` 実行前や WASM 読込失敗時）に外部リンクをクリックするとクエリを含む URL が `Referer` に送信され得ます。
- **対策案**:
  - `index.html` の `<head>` に以下を追加する（本アプリは外部 CDN 未使用・WASM 利用・`data:` SVG favicon・`blob:` 画像利用のため最小権限で厳格に記述可能）：
    ```html
    <meta
      http-equiv="Content-Security-Policy"
      content="default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self' data: blob:; worker-src 'self'; connect-src 'self'; media-src 'self'; object-src 'none'; base-uri 'self'; form-action 'none';"
    />
    <meta name="referrer" content="no-referrer" />
    ```
  - 補足: `frame-ancestors 'none'`（クリックジャッキング対策）や `Permissions-Policy: camera=(self), microphone=(), geolocation=()` は `<meta>` では効かないため、リバースプロキシや Cloudflare Pages / Netlify 等で配信する場合は HTTP レスポンスヘッダで付与する。開発時（Vite HMR の `ws:`）や `scripts/launch-offline.js` のバナー（インライン `style` 属性）は必要に応じてテスト/ビルド環境で調整する。

#### [S2] 本番ビルドへのデバッグ用グローバル参照漏洩（`window.__lastCamera`・`__TEST_ENV__`）

- **該当箇所**: [`web/camera.ts:488`](../web/camera.ts#L488), [`web/main.ts:48-57`](../web/main.ts#L48-L57)
- **リスク**:
  - `web/camera.ts:488` が環境判定なしで `(window as any).__lastCamera = this;` を実行しているため、本番環境でも `TwoViewCamera` インスタンス全体が `window` に露出しています。このオブジェクトにはライブカメラの `mediaStream`、撮影メソッド（`startLiveStream` / `captureLiveFrame`）、アップロード画像参照が含まれます。
  - `web/main.ts:48-57` も `Boolean((window as any).__TEST_ENV__) || Boolean(navigator.webdriver)` が本番ビルドに残り、DOM Clobbering（`id="__TEST_ENV__"`）や外部スクリプトによる `window.__TEST_ENV__ = true` 設定で内部ストア・WASM API が本番でも露出します。
- **対策案**:
  - `web/camera.ts:488` および `web/main.ts:48-57` のグローバル公開を `if (import.meta.env.DEV)`（あるいは E2E 専用ビルドフラグ `import.meta.env.VITE_TEST_HOOKS === "1"`）に限定し、本番ビルドでは Vite の Dead Code Elimination で完全に除去されるようにする。

#### [S3] `readJpegDimensions` の無制限ループによるメインスレッド凍結 DoS

- **該当箇所**: [`web/camera.ts:67-105`](../web/camera.ts#L67-L105)
- **リスク**:
  - `readJpegDimensions` の `for (;;)` ループに最大反復回数・最大走査オフセットの上限がありません。
  - JPEG マジックバイト `FF D8` に続けて `0xFF`（1 バイト前進）を 20 MB 敷き詰めたファイル、または空セグメント `FF E0 00 02`（4 バイト前進）を並べたファイルを投入すると、最大約 2,000 万回の非同期 `await read(...)` 反復と `DataView` 生成が発生します。
  - 実測（Node v20）で 1 MB の `0xFF` ファイルに **80.8 ms / 999,998 回反復**（20 MB 上限で ~1.6 秒超、モバイル端末では数秒〜十数秒 UI 凍結）を確認しました。
- **対策案**:
  - `const MAX_JPEG_MARKERS = 1024;` および `const MAX_JPEG_HEADER_SCAN_BYTES = 2 * 1024 * 1024;` を導入し、連続 `0xFF` フィルバイトはチャンク内ループでスキップしつつ上限超過時は即 `return null` で拒否する。

#### [S4] 無制限 CPU 消費 DoS（`gods_guarantee` ハード上限欠落・2x2 LBL/Ortega 予算無視）

- **該当箇所**: [`web/solver-client.ts:101-113`](../web/solver-client.ts#L101-L113), [`src/c3x3/search.rs:297-310`](../src/c3x3/search.rs#L297-L310), [`src/c2x2/mod.rs:88-95`](../src/c2x2/mod.rs#L88-L95), [`src/c2x2/lbl.rs:135-175`](../src/c2x2/lbl.rs#L135-L175)
- **リスク**:
  - 3x3 Kociemba・向き OFF では `gods_guarantee` が Rust 側の時間予算チェックを無効化し、`solver-client.ts:105` もクライアント側タイムアウトを完全無効化しています。Superflip 単体でもデスクトップで **5.59 秒 / 4.8 億ノード** 要しており、より困難な距離 20 盤面を共有リンク（`?state=...`）で開かせると低スペック端末で長時間 CPU 100% 張り付きを誘発できます。
  - 2x2 の `lbl` / `ortega` は `budget_ms` を受け取らず、`lbl::solve_first_layer_ida` は深さ 10 までの無枝刈り・無制限探索となっています。
- **対策案**:
  - `gods_guarantee` モードにも絶対安全上限（例: 60 秒または 5 億ノード等のハードリミット）と Worker 監視タイマーの上限を設け、超過時はそれまでに得た最良解（21〜22 手等）または明確なタイムアウトエラーを返す（P2 の探索高速化と併用すれば実質的に全局面が 1 秒未満で 20 手以内に収束）。
  - 2x2 LBL / Ortega にも `budget_ms` とノード上限を伝播する（かつ P4 により LBL 第 1 層を定数時間化）。

#### [S5] Service Worker のスコープ境界・レスポンス条件・`?no-sw` による本番 SW 強制解除

- **該当箇所**: [`public/sw.js:183-241`](../public/sw.js#L183-L241), [`web/pwa.ts:17-33`](../web/pwa.ts#L17-L33)
- **リスク**:
  1. `public/sw.js:189` は同一オリジン（`url.origin !== selfOrigin`）のみを検査しており、`self.registration.scope` 配下であるかを検証していません。GitHub Pages（`https://<user>.github.io/<repo>/`）のような共有オリジン配信では同一オリジン上の別アプリのパスまでキャッシュ対象になり得ます。
  2. `networkResponse.ok`（`L209`）のみでは `206 Partial Content`（`cache.put` が例外送出）や `Cache-Control: no-store` / `private` 応答もキャッシュを試みます。また本番ビルドでは全配信資産が `PRECACHE_ASSETS` に含まれるため、任意クエリ付き URL のランタイムキャッシュは不要なキャッシュ汚染面となります。
  3. `web/pwa.ts:17` が本番でも `?no-sw` クエリを受け付けて既存 Service Worker を `unregister()` するため、外部リンク経由でユーザーのオフライン機能を無断無効化できます。
- **対策案**:
  - `public/sw.js` で `const scopePrefix = self.registration.scope;` を用いて `if (!url.href.startsWith(scopePrefix)) return;` を追加し、`networkResponse.status === 200 && networkResponse.type === "basic"` かつ `!(networkResponse.headers.get("Cache-Control") || "").includes("no-store")` のみキャッシュ対象にする。また本番ではクエリ付き URL のランタイムキャッシュを無効化する。
  - `web/pwa.ts:17` の `params.has("no-sw")` / `window.__DISABLE_SW__` を `import.meta.env.DEV || isAutomatedTest` に限定する。

#### [S6] 50 MP 画像デコードのメモリ DoS・PNG `IHDR` 未検証・`?alg=` / `file-io` 境界検証

- **該当箇所**: [`web/camera.ts:35,119-125,725-797`](../web/camera.ts#L35), [`web/url-params.ts:61-64`](../web/url-params.ts#L61-L64), [`web/file-io.ts:10`](../web/file-io.ts#L10)
- **リスク**:
  - `MAX_IMAGE_PIXELS = 50_000_000` は RGBA 展開時に 1 枚 **200 MB**（画像 A/B で **400 MB 以上**）を消費するため、縮小前に `new Image()` フルデコードを行う現行実装ではモバイル Safari 等でタブクラッシュ（OOM DoS）を招きます。
  - PNG 寸法読み取り（`camera.ts:120-124`）で 12〜15 バイト目のチャンク種別 `IHDR`（`0x49484452`）を検査していません。
  - `web/url-params.ts:61-64` に `alg` の長さ上限がなく、巨大な `?alg=` クエリ文字列が `main.ts:1659` で正規表現置換・コピーされてから WASM へ渡されます。また `web/file-io.ts:10` は `fileSize` 引数のみを検査し `content.length` を検査していません。
- **対策案**:
  - 画像縮小に `createImageBitmap(file, { resizeWidth, resizeHeight })` を優先採用し、`MAX_IMAGE_PIXELS` も 20〜25 MP 程度へ見直す。PNG 判定に `head.getUint32(12) === 0x49484452` を追加。
  - `url-params.ts` で `algParam.length <= 4096` を強制し、超過時は無効パラメータ扱いとする。`file-io.ts` でも `fileSize > 65536 || content.length > 65536` を検査する。

#### [S7] テストコード内の `new Function()` によるワークフロー YAML 動的実行

- **該当箇所**: [`tests/quality-contracts.spec.ts:17-23`](../tests/quality-contracts.spec.ts#L17-L23)
- **リスク**:
  - `../.github/workflows/deploy-pages.yml` の `if:` 行を正規表現抽出し、`new Function("github", \`return (${expression});\`)` で JS として直接実行しています。設定ファイル由来の文字列を動的評価する実装は、PR 等でワークフロー YAML が改変された際にテスト実行環境（CI ランナー）上で任意コード実行を許すアンチパターンです。
- **対策案**:
  - `new Function` を撤去し、`github.ref == '...'`・`github.event_name == '...'`・`&&`・`||`・括弧のみを受理する小さな再帰下降パーサで安全に評価する。

#### [S8] `innerHTML` シンク削減・エラー詳細露出抑制・`console_error_panic_hook` の本番扱い

- **該当箇所**: [`web/main.ts:171,188,251,587,685,1809`](../web/main.ts#L587), [`web/solver.worker.ts:109`](../web/solver.worker.ts#L109), [`Cargo.toml:18,21`](../Cargo.toml#L18-L21)
- **詳細**:
  - 現在の `innerHTML` 使用箇所（`web/view.ts:33`, `web/main.ts` 5 箇所）はすべて静的定数・アイコン定義のみであり現時点で XSS は成立しませんが、`main.ts` の動的更新箇所（`#solve`, `#play`, `#sound-toggle`, `#solver-algorithm`）を DOM API ベースへ置き換えることで将来の誤用リスクを根絶でき、P6 の再描画コスト削減も同時に達成できます。
  - `solver.worker.ts:109` の `${String(error)}` や `main.ts:1809` の `${response.url}` は画面表示を簡潔な定型エラー文に留め、詳細ログは `console.error` 側に分離することを推奨します。
  - `Cargo.toml` の `console_error_panic_hook` はデフォルト有効で本番 WASM にもパニック時の Rust ファイルパス/行番号出力とサイズ増を含みます。リリースビルドでは `--no-default-features` 選択またはサイズ優先時の無効化を検討できます。

#### [S9] サプライチェーンおよび良好な既存セキュリティ実装の評価

- **該当箇所**: [`package.json:30-41`](../package.json#L30-L41), [`web/file-io.ts:17-20`](../web/file-io.ts#L17-L20)
- **良好な点（維持すべき強み）**:
  - 本番ランタイム JS 依存が `three@0.186.0` の 1 パッケージのみ（完全ピン留め）で、外部 CDN・外部フォント・トラッカー読み込みがゼロ。
  - 外部リンク（GitHub）に `rel="noopener noreferrer"` 設定済み、`localStorage` 全アクセスが `try/catch` 保護済み。
  - JSON（64 KB 上限）・画像（20 MB 上限＋マジックバイト検査により SVG 等のスクリプト含有形式をデコード前に自動遮断）の Fail-fast 検証が整備済み。
  - `scripts/launch-offline.js` は `127.0.0.1` 限定バインド・`spawn`（非シェル）・SHA-256 配信物照合を実装。
- **小さな強化提案**:
  - `package.json:37` の `canvas: "^3.2.3"`（`hasInstallScript: true` の dev 依存）を完全バージョンピン留め（`"3.2.3"`）に統一する。
  - `web/file-io.ts:17-20` の `in` 演算子を `Object.hasOwn(data, "version")` 等に置換し、プロトタイプチェーン参照を完全に断つ。
  - CI ワークフローに `npm audit --omit=dev` および `cargo audit` の定期・PR チェックを追加する。

---

## 4. 優先度付き改善ロードマップ

| フェーズ | 対象 ID | 主な作業内容 | 期待効果 |
| :--- | :--- | :--- | :--- |
| **Phase 1（即効・高 ROI）** | **S2, S1, S3, S5**<br>**P3, P2(一部), P4, P6** | - `window.__lastCamera` / `__TEST_ENV__` の本番除去、`index.html` へ CSP・`Referrer-Policy` 追加<br>- `readJpegDimensions` 反復上限、`?no-sw` 開発限定化、SW スコープ検証<br>- `initialize()` から 2x2 BFS を遅延化（起動 **179 ms → 8.5 ms**）<br>- Kociemba Phase 1→2 座標ループ外移動＋G1 枝刈り＋早期終了<br>- 2x2 Optimal（向き OFF）の 3 面限定＋LBL `group_b` 活用＋定石 `OnceLock`<br>- 非表示 `#fallback-net` スキップ＋`getCellArrowInfo` 二重計算解消 | 初回 Worker 起動 **21 倍高速化**、3x3/2x2 探索 **10〜100 倍高速化**、主要な High セキュリティ課題（CSP・カメラ参照漏洩・JPEG DoS）を一掃 |
| **Phase 2（構造最適化）** | **P1, P2(完全), P8** | - `PruningTable` 4-bit パック＋ゼロコピー参照、`MoveTable` の Phase 2 10 手縮小または起動時生成<br>- WASM 並列コンパイル／転送またはメインスレッド WASM 分離<br>- Kociemba 6 方向インターリーブ探索＋4-bit 直接枝刈り表＋先読み枝刈り<br>- SW `activate` の旧巨大 WASM 移行廃止 | WASM 転送サイズ **3.48 MB → ~750 KB gzip（78% 削減）**、メモリ **~18.6 MB → ~4 MB**、Superflip 含む全局面 < 0.5 秒、SW キャッシュ肥大化解消 |
| **Phase 3（継続強化）** | **P5, P7, P9**<br>**S4, S6, S7, S8, S9** | - Thistlethwaite / Korf の `MoveTable`/`PruningTable`・3 軸下界活用<br>- カメラ `ImageData` キャッシュ・ホバー再描画抑止・`createImageBitmap` リサイズ<br>- `gods_guarantee` ハード上限、`?alg=` 上限、`new Function` 撤去、`innerHTML` DOM 化、`npm/cargo audit` 導入 | 全アルゴリズムの体感待機ゼロ化、カメラ操作の 60fps 化、DoS・CI・サプライチェーンの完全ハードニング |
