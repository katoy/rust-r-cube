# 全体コードレビューレポート（2026-10-17）

- **実施日**: 2026-10-17（JST）
- **対象範囲**: リポジトリ全ソースコード（`src/` 配下の 3x3・2x2 ソルバーおよび WASM 境界、`build.rs`、`web/` 配下の全 TypeScript モジュール、`public/sw.js`、`scripts/`、`start.sh`、`tests/`、各種設定ファイル・ドキュメント）
- **前回レビューとの対応**: 前回レビュー（[docs/code-review-e0a282e-2026-10-09.md](code-review-e0a282e-2026-10-09.md)）およびその修正レポート（[docs/review-fixes-e0a282e-fixes-2026-10-09.md](review-fixes-e0a282e-fixes-2026-10-09.md)）の修正実態をコードと実行結果の双方から照合し、修正過程で新たに混入した退行・未実装項目・潜在バグを洗い出しました。
- **総合判定**: **要修正（CHANGES REQUESTED）** — High 0 件 / Medium 5 件 / Low 8 件

---

## 1. 自動チェック・実行検証の結果

| 検証項目 | 実行コマンド / 方法 | 結果 |
| :--- | :--- | :--- |
| **ガードレール静的検査** | `npm run check:guardrails` | PASS（ただし L5 のとおり検査ルールの抜け穴あり） |
| **Rust フォーマット** | `cargo fmt --check` | PASS |
| **TypeScript 型検査** | `npm run typecheck` (`tsc --noEmit`) | PASS（`web/` および `vite.config.ts`） |
| **Web フォーマット** | `npm run format:check` | PASS |
| **Rust Release テスト** | Release テストバイナリ実行 | 155 passed / 0 failed / 1 ignored |
| **WASM 実機プローブ検証** | Node + `pkg/cube_studio_bg.wasm` による境界値・性能実測 | M1・M2・M5・L7 の現象を再現・数値確認済み |

---

## 2. 総括（Executive Summary）

### 2.1 前回レビュー（e0a282e）から着実に改善された点
- **2x2 最短探索（向き OFF）の高速化**: `src/c2x2/search.rs:64-76` において DBL コーナーを位置 6・向き 0 に固定する 1 向き探索へ改善され、24 向き総当たりによる 11 手アンチポードの探索遅延（約 9.7 秒 → 1 秒未満）と探索ノード数のリセット問題が解消されました。
- **カメラ画像ヘッダ解析の強化**: `web/camera.ts:28-145` にチャンク単位のヘッダリーダー `createHeaderReader` が導入され、大きな APP セグメントを持つ JPEG や WebP（VP8 / VP8L / VP8X）、GIF、BMP の寸法をフルデコード前に判定できるようになりました。
- **プレビュー盤面と確定盤面の分離**: `web/main.ts:266` の `persist()` が `store.getBaseSnapshot() ?? store.getSnapshot()` を保存するよう修正され、シーク途中のリロードで未確定のプレビュー盤面が確定状態として上書きされる問題が解消されました。
- **2x2 カメラ認識の多色スクランブル E2E 検証**: `tests/camera-2x2.spec.ts` に多色スクランブル画像（`2x2-gods-number-11`）の 24 文字一致検証と手動色補正の伝播テストが追加されました。

### 2.2 今回検出された主要な問題点
一方で、前回修正（`fix/code-review-e0a282e`）の実装コードと修正レポート（`docs/review-fixes-e0a282e-fixes-2026-10-09.md`）を突き合わせた結果、**修正レポートに記載された内容が実際には実装されていない箇所**や、**修正によって新たに発生した探索性能・UI 整合性の退行**が確認されました。

1. **3x3 既定ソルバー（センター向き込み Kociemba）の探索退行（M1）**: `set_max_total(seq_len)` の導入により `self.max_total == 30` の分岐条件が外れ、Phase 2 の深さ上限が 12 手に縛られたまま Phase 1 深さ 12 まで全探索し、さらに `total = 24` で同一探索木を重複走査する退行が発生しています。これによりランダムスクランブルの約半数で予算を使い切って 49〜125 手の逐次解へ後退しています。
2. **2x2 向き OFF 解法後の完成判定破綻（M2）**: 2x2 で「標準の向きに揃える」を OFF にして Optimal で解くと、23/24 の確率で空間回転された完成状態（6 面すべて単色だが `SOLVED_2X2` とは文字列表現が異なる状態）に到達しますが、Web UI が `state === SOLVED_2X2` でしか完成判定をしていないため、**6 面が揃っているのに「スクランブル状態」と表示され、完成効果音も鳴らない**不整合が起きます。
3. **種別切替時の盤面退避未実装と Redo 消失（M3）**: 修正レポートで「`typeHistoryMap` により種別ごとの盤面・履歴を退避・復元」と報告された機能は `web/cube-store.ts` に実装されておらず、`3x3 → 2x2 → 3x3` の切り替えで盤面が完成状態にリセットされ、Redo 履歴も消滅します。
4. **Service Worker クエリキャッシュ管理の欠陥（M4）**: `queryKeysQueue` が揮発性のインメモリ配列であるため SW 再起動で上限管理がリセットされるほか、同一 URL の再取得で重複 push されキャッシュが自己破棄されます。
5. **3x3 CFOP / Thistlethwaite の連続同面回転（M5）**: センター補正が走らない経路（向き OFF 等）で `cancel_redundant_moves_with_phases` が呼ばれず、CFOP の 100%（20/20 局面）・Thistlethwaite の 35%（7/20 局面）で `U U2` や `U' U'` などの連続同面回転と空フェーズが出力されます。

---

## 3. 指摘一覧（サマリー）

| ID | 重大度 | 領域 | 概要 | 確証度 |
| :--- | :---: | :--- | :--- | :---: |
| **M1** | **Medium** | Rust 3x3 | センター向き込み Kociemba で `max_total == 30` 判定が外れ、Phase 2 上限 12 固定＋重複走査により約半数がタイムアウト・逐次解（49〜125手）に後退する | 確認済み |
| **M2** | **Medium** | Web / 2x2 | 2x2 向き OFF で揃えた空間回転完成状態が UI と WASM `is_solved` で「未完成（スクランブル状態）」と扱われ、LBL/Ortega も向き OFF を無視する | 確認済み |
| **M3** | **Medium** | Web 状態 | `CubeStore` に種別ごとの盤面退避（`typeHistoryMap`）が実装されておらず、種別往復で盤面が初期化され Redo 履歴も消える | 確認済み |
| **M4** | **Medium** | PWA / SW | `public/sw.js` の `queryKeysQueue` がインメモリ配列のため SW 再起動で無効化され、同一 URL の重複登録で自己削除が起きる | コード上確認 |
| **M5** | **Medium** | Rust 3x3 | CFOP / Thistlethwaite でセンター補正を伴わない場合に冗長手相殺が行われず、`U U2` 等の連続同面回転と空フェーズが残る | 確認済み |
| **L1** | Low | Rust 2x2 | Ortega 法 `solve_first_face` の BFS 深さ上限変更が片側（`:90`）のみで、キュー追加条件（`:117` の `< 5`）が未修正のため機能していない | コード上確認 |
| **L2** | Low | Web / a11y | カメラ入力から「色入力へ反映」でエディタへ遷移すると、`lastTriggerElement` が閉じたダイアログ内の要素で上書きされフォーカス復元が失われる | コード上確認 |
| **L3** | Low | カメラ 2x2 | 2x2 の手動色補正で `?`（未入力）に戻した際、直前の整合性エラー表示（`#camera-error`）がクリアされず残る | コード上確認 |
| **L4** | Low | カメラ | `readImageDimensions` の先頭 30 バイト制限により、25〜29 バイトの最小有効 WebP (VP8L) が拒否される | コード上確認 |
| **L5** | Low | ガードレール | `check-review-guardrails.js` が記載している `tests/fixtures/guardrails/` が存在せず、F3・F5・`build.rs` 鮮度検査に抜け穴がある | 確認済み |
| **L6** | Low | テスト | `tests/coverage.spec.ts:607` に `CLAUDE.md` §4 が禁じる不自然なサイズ引数（72B 文字列に `70000` を指定）が残存している | コード上確認 |
| **L7** | Low | Rust 3x3 | Thistlethwaite の Phase 3 に枝刈りがなく、約 4 割の局面で予算の 70% を消費してから Kociemba にフォールバックする（継続課題） | 確認済み |
| **L8** | Low | 衛生 / 契約 | `c2x2::tables::MoveTable` の未使用コード残存、3x3 未知アルゴリズム名の暗黙フォールバック、生成物ファイル重複、README テスト件数表記ズレ | 確認済み |

---

## 4. 指摘事項の詳細と修正案

### 4.1 Rust 3x3・2x2 ソルバーおよび WASM 境界

#### M1【Medium・確認済み】センター向き込み Kociemba 探索における Phase 2 深さ上限の誤判定と重複走査
- **該当箇所**:
  - [`src/lib.rs:236-260`](../src/lib.rs#L236-L260)
  - [`src/c3x3/search.rs:85-130`](../src/c3x3/search.rs#L85-L130), [`src/c3x3/search.rs:193-195`](../src/c3x3/search.rs#L193-L195)
- **現象と根本原因**:
  1. `src/lib.rs:254` では、まず色のみの解 `color_moves` とセンター補正手順 `fixes` から逐次解の手数 `seq_len = cm.len() + fixes.len()`（通常 32〜125 手）を求め、`search_oriented.set_max_total(seq_len)` を呼んでいます。
  2. ところが `src/c3x3/search.rs:194` の Phase 2 最大深さ計算が以下のようになっています：
     ```rust
     let max =
         (self.max_total - self.path.len()).min(if self.max_total == 30 { 18 } else { 12 });
     ```
     条件が `self.max_total >= 30` ではなく `self.max_total == 30` であるため、`self.max_total` に `seq_len`（例: `64`）が設定されると Phase 2 の最大深さが `18` ではなく **`12` に制限**されます。
  3. センター向きを考慮する Supercube 探索では、Phase 1（深さ 8〜10）のあとに Phase 2 で 13〜16 手を要する局面が多数存在します。しかし 1 回目のループ（`for depth in 0..=12`、`search.rs:85-103`）では Phase 2 が 12 手以下に制限されているため解が見つからず、分岐数が爆発する Phase 1 深さ 11〜12 まで全探索して 1.2〜5.0 秒を費やします。
  4. さらに 1 回目のループでタイムアウトしなかった場合でも、直後の緩和ループ（`search.rs:106-129`）：
     ```rust
     if self.best_solution.is_none() && !self.timed_out && initial_max >= 22 {
         for &total in &[24, 30] {
             self.max_total = total;
             for depth in 0..=12 { ... }
     ```
     において、最初の `total = 24` でも `self.max_total == 30` は偽（Phase 2 上限 12）となるため、**1 回目のループと完全に同一の探索空間（Phase 1: `0..=12`, Phase 2: `<= 12`, 合計 `<= 24`）をゼロからもう一度重複走査**します。
  5. その結果、`total = 30`（Phase 2 上限 18）に到達する前に時間予算を使い切る局面が多発しています。
- **実測エビデンス（WASM Release ビルド）**:
  - 既定予算 5,000ms で `seed = 1..=5` を解いた結果：
    - `seed = 1`: 1,184ms（24手・同時最適化）— `total = 24` の重複走査後に `total = 30` で発見
    - `seed = 2`: **5,001ms 消費 → タイムアウトし 64 手の逐次解に後退**
    - `seed = 3`: 1,330ms（26手・同時最適化）
    - `seed = 4`: 1,285ms（24手・同時最適化）
    - `seed = 5`: **5,001ms 消費 → タイムアウトし 90 手の逐次解に後退**
  - 予算 2,000ms で `seed = 1..=15` を解いた結果：**15 局面中 7 局面（47%）が同時最適化に失敗して 49〜125 手の逐次解に後退**し、平均所要時間は 1,780ms でした。
- **付随する問題**:
  - `src/lib.rs:239` の `seq_len` は `supercube::cancel_redundant_moves_with_phases` を通す前の手数（`cm.len() + fixes.len()`）で計算されているため、接合部相殺後の逐次解より手数が長い同時最適化解を採用してしまうケースがあります。
  - `fixes.is_empty()`（色のみの解でセンターも偶然揃っていた場合）でも無条件に `search_oriented` を実行しています。
  - `search.rs:106` の `initial_max >= 22` は、外部から `set_max_total(22)` や `set_max_total(23)` が明示指定された場合にも勝手に上限を `24`・`30` へ引き上げてしまいます。
- **修正案**:
  1. `src/c3x3/search.rs:194` を `if self.max_total >= 30 { 18 } else { 12 }`（または `target_centers.is_some()` の場合は最初から Phase 2 上限を `18.min(self.max_total - self.path.len())`）に修正する。
  2. `src/lib.rs:236-246` で先に `cancel_redundant_moves_with_phases` を適用して正確な `seq_len` を算出し、`search_oriented` には `seq_len.min(30)` を上限として渡す（30 手を超える同時最適化解は探索空間上も実用的にも不要なため）。また `fixes.is_empty()` の場合は `search_oriented` をスキップするか上限を `cm.len() - 1` に絞る。
  3. `search.rs:106` の自動緩和ループは、`max_total` が外部から明示指定されていない場合のみ実行し、1 回目のループと同じ `(max_total, max_p2)` の組み合わせを再走査しないようにする。

---

#### M2【Medium・確認済み】2x2 の「標準の向きに揃える」OFF 時における完成状態判定の不整合と LBL / Ortega の向き無視
- **該当箇所**:
  - [`web/main.ts:392-401`](../web/main.ts#L392-L401), [`web/main.ts:420`](../web/main.ts#L420), [`web/main.ts:646`](../web/main.ts#L646)
  - [`src/lib.rs:409-411`](../src/lib.rs#L409-L411), [`src/lib.rs:427-432`](../src/lib.rs#L427-L432)
  - [`src/c2x2/mod.rs:34-56`](../src/c2x2/mod.rs#L34-L56)
- **現象と根本原因**:
  1. 2x2 の Optimal 探索（`include_orientation = false`）は、DBL コーナーが現在ある位置・向きを基準（位置 6・向き 0）に空間回転した目標状態へ最短手順で揃えます（`src/c2x2/search.rs:64-76`）。そのため、`D`・`L`・`B` のいずれかを含むスクランブル（またはランダム状態の 23/24）では、解法の最終状態 `solution.state` は標準向きの `UUUURRRRFFFFDDDDLLLLBBBB`（`SOLVED_2X2`）ではなく、**空間回転された完成状態**（例: `D` に対して `U'` で揃えた `UUUUFFFFLLLLDDDDBBBBRRRR`）になります。
  2. しかし `web/main.ts:394` の `refresh()` では：
     ```ts
     const solved = getSolvedState(is2x2 ? "2x2" : "3x3");
     const isSolved = state === solved;
     ```
     と `SOLVED_2X2` との文字列一致のみで完成判定を行っています。`applyAlgorithm()`（`web/main.ts:646`）の成功効果音判定も同様です。
  3. その結果、向き OFF の解法手順を最後まで進めて解法パネルを閉じたとき、あるいは手動で回転して 6 面すべてを単色に揃えたときに、**画面上の 2x2 キューブは 6 面すべて揃っているにもかかわらず、ステータスバッジ（`#cube-status`）は「スクランブル状態」と表示され、`#solve` ボタンも「完成状態を確認」ではなく「解法を探す」となり、手動完成時の成功効果音も鳴りません**。
  4. さらにその状態で「解法を探す」（向き OFF）を押すと、ソルバーは 0 手を返して「すでに6面が揃っています。」と表示するため、同じ画面上で「スクランブル状態」と「すでに6面が揃っています」が矛盾して表示されます。
  5. また、WASM 公開 API の `is_solved` / `validate`（`src/lib.rs:409, 427`）は `c2x2::is_solved(state, true)` を呼んでおり、空間回転された 2x2 完成状態に対して常に `false` を返します。
  6. 加えて、2x2 で 1 手以上離れた状態（例: 回転完成状態 `FFFFUUUURRRRBBBBDDDDLLLL` に `U` を 1 手適用した状態）を `include_orientation = false` で解いた場合、`optimal` は `U'`（1 手）を返すのに対し、`lbl` は `D L R' F2`（4 手）、`ortega` は `D' L' R U2 D2`（5 手）を返し、常に標準向き（白上・緑前）へ揃えようとします（修正レポート L25 では「DBL コーナー基準の向きを追従・正規化する仕様」と記載されていますが、`c2x2/mod.rs` では未実装です）。
- **修正案**:
  1. 2x2 において各面 4 マスが同一色（かつ合法盤面）であれば空間回転された完成状態と判定するヘルパーを導入し、`web/main.ts` の `refresh()`（ステータス表示・ボタン文言）と `applyAlgorithm()`（完成効果音）に反映する（標準向きでない場合は「完成状態（回転あり）」または `includeOrientation.checked` に応じた表示にする）。
  2. `src/c2x2/mod.rs` の `lbl` / `ortega` でも、`include_orientation == false` のときは `search.rs` と同じ DBL 固定の空間回転 `rot` で `rc` を正規化してから解き、逆変換した手順を返すか、または LBL / Ortega は常に標準向き（白底/黄上）を基準とする旨を UI・仕様に明記する。

---

#### M3【Medium・確認済み】キューブ種別切替で盤面・履歴の退避が行われず、Redo 履歴も失われる（修正レポート M1 との乖離）
- **該当箇所**: [`web/cube-store.ts:49-66`](../web/cube-store.ts#L49-L66), [`web/main.ts:198-209`](../web/main.ts#L198-L209)
- **現象と根本原因**:
  - `docs/review-fixes-e0a282e-fixes-2026-10-09.md`（§1 M1 および §2.1）には「`CubeStore` に種別ごとの盤面・履歴退避マップ（`typeHistoryMap`）を実装。種別切替時に現在の盤面と履歴を保存し、切替先の退避履歴を安全にリストア」と記載されています。
  - しかし実際の [`web/cube-store.ts:49-66`](../web/cube-store.ts#L49-L66) には `typeHistoryMap` は存在せず、以下の実装になっています：
    ```ts
    setCubeType(type: CubeType, record = false): void {
      if (this.cubeType === type) return;
      if (record) {
        this.commitBaseSnapshotIfPreviewing();
        this.pushHistory(this.getSnapshot());
      } else {
        this.history = [];
        this.future = [];
      }
      this.cubeType = type;
      this.state = getSolvedState(type);
      ...
    ```
  - このため、以下の 2 つの問題が発生します：
    1. **種別ボタンの往復で盤面が初期化される**: 3x3 でスクランブルや手順入力を行った後、「2×2×2」ボタンを押してから再度「3×3×3」ボタンを押すと、元の 3x3 盤面は復元されず **完成状態（`SOLVED_3X3`）にリセット**されます（2x2 側で作業してから 3x3 → 2x2 と戻した場合も同様に `SOLVED_2X2` にリセットされます）。
    2. **Redo 履歴の消失**: `this.pushHistory(...)` が無条件に `this.future = []` にするため、Undo 後に種別を切り替えると Redo 履歴がすべて失われます。
- **修正案**:
  - 修正レポートの設計どおり、`CubeStore` に種別（`"2x2"` / `"3x3"`）ごとの盤面・`history`・`future` を退避・復元するマップを実装するか、または種別切替時にも切替先の直近スナップショットを復元できるようにする。

---

#### M4【Medium・コード上確認】Service Worker のクエリ付きキャッシュ上限管理が Worker 再起動でリセットされ、重複登録で自己削除が起きる
- **該当箇所**: [`public/sw.js:47-48`](../public/sw.js#L47-L48), [`public/sw.js:68-71`](../public/sw.js#L68-L71), [`public/sw.js:200-209`](../public/sw.js#L200-L209)
- **現象と根本原因**:
  1. **SW 再起動による上限管理の消失**: `public/sw.js:48` の `const queryKeysQueue = [];` は Service Worker スレッド内のインメモリ変数ですが、`ownCache`（Cache Storage）はブラウザに永続化されます。Service Worker はアイドル状態が続くとブラウザによって終了・再生成されるため、再起動のたびに `queryKeysQueue` は空配列 `[]` にリセットされ、過去のセッションで保存されたクエリ付きエントリは永久に `ownCache.delete()` されません（原則 ⑤ 違反）。
  2. **重複 push によるキャッシュの自己削除**: `public/sw.js:200-206` では：
     ```js
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
     と毎回 `url.href` を重複チェックなしで `push` しています。同一のクエリ付き URL（例: Vite の `?url` アセット）が繰り返しリクエストされると、`queryKeysQueue` が同一 URL で埋め尽くされ、51 回目以降は古い別 URL を誤って削除するか、重複した同一 URL を削除し続ける挙動になります。
  3. **旧キャッシュ移行時の世代順序の誤り**: `public/sw.js:68-70` で `oldKeys` を `a.localeCompare(b, undefined, { numeric: true })` でソートし `pop()` で「直近 1 世代」を取得していますが、`scripts/generate-sw-precache.js:61` が注入する `CACHE_VERSION` は **SHA-256 ハッシュの先頭 10 文字（16進文字列）**です。ハッシュ文字列の辞書順はビルドの時系列と無関係であるため、複数世代のキャッシュが残っている場合に直近の世代ではなくランダムな世代が選択されます。
- **修正案**:
  1. インメモリ配列 `queryKeysQueue` を廃止し、`const keys = await ownCache.keys(); const queryReqs = keys.filter((r) => new URL(r.url).search !== "");` のように永続キャッシュ自身のキー一覧（Cache API は挿入順を保持）から超過分を `ownCache.delete()` する。
  2. `generate-sw-precache.js` で注入する `CACHE_VERSION` を `${Date.now()}-${contentHash}` のように単調増加プレフィックス付きにするか、`activate` 時はプリキャッシュに含まれる資産が既に揃っているため旧キャッシュからのマイグレーション処理自体を簡素化する。

---

#### M5【Medium・確認済み】3x3 の CFOP / Thistlethwaite で冗長手相殺が行われず、連続同面回転と空フェーズが出力される
- **該当箇所**:
  - [`src/lib.rs:116-143`](../src/lib.rs#L116-L143), [`src/lib.rs:173-201`](../src/lib.rs#L173-L201), [`src/lib.rs:334-340`](../src/lib.rs#L334-L340)
- **現象と根本原因**:
  - 2x2 の `lbl.rs` および `ortega.rs` では、解法の最後で必ず `cancel_redundant_moves_with_phases` を呼び出し、フェーズ境界やマクロ間の連続同面回転（`U` + `U2` → `U'`、`U` + `U'` → 消去）を相殺するとともに、手数 0 の空フェーズ（`start == end`）を除去しています。
  - 一方、3x3 の `src/lib.rs` では、`supercube::cancel_redundant_moves_with_phases` が呼ばれるのは **`include_orientation == true` かつ `initial_centers == Some(...)` かつ `!center_fixes.is_empty()` のとき（または Kociemba の逐次解フォールバック時）のみ**です。
  - そのため、「向きも含めて揃える」を OFF（`include_orientation = false`）にした場合や、色解法終了時点でたまたまセンター向きがすべて 0 だった場合（`center_fixes.is_empty()`）、および `solve_state`（`initial_centers = None`）を呼んだ場合には、冗長手相殺が一切実行されません。
- **実測エビデンス**:
  - `seed = 1..=20` のスクランブルに対し `include_orientation = false` で検証した結果：
    - **CFOP**: **20 局面中 20 局面（100%）**で連続する同一面の回転（`U' U'`、`U U'`、`U U2` 等）が解法に含まれました。
    - **Thistlethwaite**: **20 局面中 7 局面（35%）**でフェーズ境界の連続同面回転が含まれました（例: `"U"` 1 手のみのスクランブルに対して、Phase 3 が `U`、Phase 4 が `U2` を返し、最終手順として **`U U2` の 2 手**が出力されます）。
    - また、`"U"` 1 手スクランブルを CFOP で解くと、`start: 0, end: 0` の空フェーズが 4 個（Cross, First Layer, Second Layer, OLL）そのまま返されます。
- **修正案**:
  - `src/lib.rs` の `solve_state_with_algorithm` において、`apply_center_orientation_fix` の後に（`include_orientation` や `center_fixes` の有無を問わず）無条件で `supercube::cancel_redundant_moves_with_phases(moves, phase_infos)` を実行する。

---

### 4.2 Low 指摘事項（L1〜L8）

#### L1【Low・コード上確認】2x2 Ortega 法 `solve_first_face` の BFS 深さ上限引き上げが未完で機能していない
- **該当箇所**: [`src/c2x2/ortega.rs:90`](../src/c2x2/ortega.rs#L90), [`src/c2x2/ortega.rs:117`](../src/c2x2/ortega.rs#L117)
- **詳細**:
  - 前回修正（L26）で `:90` のループ終了条件が `if moves.len() >= 6 { break; }` に変更されましたが、`:117` のキュー追加条件が `if moves.len() + 1 < 5` のまま残っています。
  - `moves.len() + 1 < 5` では長さ 4 までの手順しか `q` に積まれないため、`q.pop_front()` から取り出される `moves.len()` は最大 4 となり、`:90` の `moves.len() >= 6` は決して成立しないデッドコードになっています（実質の上限は深さ 5 のままです）。
- **修正案**: `:117` を `if moves.len() + 1 < 6` に修正する。

#### L2【Low・コード上確認】カメラ入力から色入力エディタへ遷移した際、モーダル終了時のフォーカス復元先（`lastTriggerElement`）が失われる
- **該当箇所**: [`web/camera.ts:288-297`](../web/camera.ts#L288-L297), [`web/main.ts:703-723`](../web/main.ts#L703-L723), [`web/main.ts:753-760`](../web/main.ts#L753-L760), [`web/main.ts:996-1002`](../web/main.ts#L996-L1002)
- **詳細**:
  - `#camera-colors` ボタンから `#camera-editor` を開いた時点では、`lastTriggerElement` に `#camera-colors` が保存されます。
  - しかし `#camera-apply`（「色入力へ反映」）をクリックすると、`this.close()` の直後に `this.apply(state)` → `openModal(getEditor, ...)` が呼ばれ、`openModal` 内の `const trigger = document.activeElement`（この時点では閉じられた `#camera-editor` 内の `#camera-apply` または `body`）によって `lastTriggerElement` が上書きされます。
  - その結果、ユーザーが色入力エディタ（`#editor`）を閉じた際、`lastTriggerElement.focus()` は非表示の `<dialog id="camera-editor">` 内の `#camera-apply` をフォーカスしようとして失敗し、フォーカスが失われます。
- **修正案**: `openModal` において、`trigger` が `dialog` 要素の内部（特に `!trigger.closest("dialog")?.open`）にある場合は `lastTriggerElement` を上書きしないようにする。

#### L3【Low・コード上確認】2x2 カメラの手動色補正で「未入力 (`?`)」を選ぶと、直前の整合性エラー表示が消えずに残る
- **該当箇所**: [`web/camera.ts:956-969`](../web/camera.ts#L956-L969)
- **詳細**:
  - `validate2x2Faces()` は 6 面キャプチャ後に不正な配色を検知すると `#camera-error` に警告を表示しますが、その後にユーザーがパレットの `?`（未入力）でセルを書き換えると、`if (state.includes("?")) return;`（`:958`）で早期リターンするため、古いエラーメッセージがクリアされずに残ります。
- **修正案**: `if (state.includes("?")) { this.error(""); return; }` として未確定状態の古い整合性エラーを消去する。

#### L4【Low・コード上確認】`readImageDimensions` の 30 バイト制限により最小 WebP (`VP8L`) が拒否される
- **該当箇所**: [`web/camera.ts:105-128`](../web/camera.ts#L105-L128)
- **詳細**:
  - WebP の分岐先頭で `n >= 30` を要求していますが、ロスレス WebP（`VP8L`）は 25 バイト目までで幅・高さが確定し、1×1 等の極小 WebP 画像はファイル全体で 28 バイトになりえます。
- **修正案**: WebP 共通ヘッダ判定を `n >= 20` とし、`VP8X` / `VP8` は `n >= 30`、`VP8L` は `n >= 25` で判定する。

#### L5【Low・確認済み】ガードレールチェッカーのフィクスチャ不在と検査ルールの抜け穴
- **該当箇所**: [`scripts/check-review-guardrails.js:13`](../scripts/check-review-guardrails.js#L13), [`:173`](../scripts/check-review-guardrails.js#L173), [`:240`](../scripts/check-review-guardrails.js#L240), [`start.sh:56`](../start.sh#L56), [`playwright.config.ts:41`](../playwright.config.ts#L41)
- **詳細**:
  1. `scripts/check-review-guardrails.js:13` のコメントに「`すり抜けパターンは tests/fixtures/guardrails/ に置き、tests/scripts-regression.spec.ts で検出を回帰テストする`」と記載されていますが、`tests/fixtures/` ディレクトリ自体が存在せず、`tests/scripts-regression.spec.ts` でも F1 の単純な 1 パターンしか検証されていません。
  2. `checkServiceWorkerCache`（`:240`）は `boundedCacheKey` 使用時に `!content.includes(".delete(")` を検査していますが、`public/sw.js` の `activate` イベント内に `caches.delete(mostRecentOldKey)` が存在するため、`fetch` ハンドラ内のキャッシュ上限削除（`ownCache.delete`）を削除してもエラーになりません。
  3. `checkFileReadOrder`（`:173`）は直前 15 行に `.size` という文字列があるかしか見ていないため、`capturedFaces.size` 等の無関係な `.size` が近くにあると `file.size` 未検査でもパスします。
  4. `start.sh:56` の `needs_wasm_build()` は `Cargo.toml`・`Cargo.lock`・`src/` のみを検査しており、`build.rs` の更新を検知しません。また `playwright.config.ts:41` の preview サーバー（ポート 4173）は `npm test` 単体実行時に `dist/` の鮮度を検証しません。
- **修正案**:
  - F1〜F7 の各違反パターンに対する回帰テストを `tests/scripts-regression.spec.ts` に追加する。
  - `checkServiceWorkerCache` では `ownCache.delete(` が `MAX_QUERY_ENTRIES` の処理ブロック内に存在することを検査する。
  - `start.sh` の `needs_wasm_build()` に `[ "build.rs" -nt "pkg/cube_studio_bg.wasm" ]` を追加する。

#### L6【Low・コード上確認】`tests/coverage.spec.ts` に `CLAUDE.md` §4 が禁じる不自然なサイズ引数が残存している
- **該当箇所**: [`tests/coverage.spec.ts:604-609`](../tests/coverage.spec.ts#L604-L609)
- **詳細**:
  - `CLAUDE.md` §4 で名指しで禁止されている「短い JSON 文字列に `70000` というモックサイズ引数を直接渡す呼び出し」が、`validateAndParseCubeJson("{}", 70000)` から `validateAndParseCubeJson(validJson, 70000)`（`validJson` は約 72 バイト）に変わっただけで、実データ長と乖離したモック引数のまま残っています（修正レポート §2.5 の「70,000 bytes に肥大化させたペイロードを渡し」という記述とも食い違っています）。
- **修正案**: 実際に 65,536 バイトを超える文字列を生成し、`validateAndParseCubeJson(hugeContent, hugeContent.length)` として検証する。

#### L7【Low・確認済み】Thistlethwaite の Phase 3 に枝刈りがなく、約 4 割が予算の 70% を待ってから Kociemba にフォールバックする（継続課題）
- **該当箇所**: [`src/c3x3/thistlethwaite.rs:55`](../src/c3x3/thistlethwaite.rs#L55), [`src/c3x3/thistlethwaite.rs:363-392`](../src/c3x3/thistlethwaite.rs#L363-L392)
- **詳細**:
  - `seed = 1..=20`（予算 3,000ms）で実測したところ、20 局面中 8 局面（40%）が Phase 3 で時間切れとなり Kociemba フォールバックに移行しました。フォールバック発生時は常に予算の 70% を消費するため、体感待ち時間が長くなります。
- **修正案**: $G_2 \to G_3$ 用の軽量な枝刈りテーブル（コーナー軌道＋エッジスライス）を導入する。

#### L8【Low・確認済み】未使用コード・契約の非対称性・リポジトリ衛生の課題
- **該当箇所**:
  - [`src/c2x2/tables.rs:8-28`](../src/c2x2/tables.rs#L8-L28): `c2x2::tables::MoveTable` は `initialize()` から呼ばれなくなったものの、構造体と生成関数が未使用のまま残っています。
  - [`src/lib.rs:165-170`](../src/lib.rs#L165-L170): 2x2 では未知のアルゴリズム名を渡すと `Err` を返すよう改善されましたが、3x3 では未知のアルゴリズム名を渡すと黙って `"kociemba"` にフォールバックする非対称な契約になっています。
  - ルート直下の生成物（`coverage.lcov`, `coverage*.txt`, `coverage-tarpaulin*/`, `coverage-e2e*/`）および `cubes/` と `public/cubes/` の重複が引き続き残っています。
  - [`README.md`](../README.md) 内の Rust 単体テスト件数表記が「154 件」（実際は 155 passed / 1 ignored）のままです。

---

## 5. 推奨する修正の優先順位

1. **M1（3x3 センター込み Kociemba の探索退行修正）**: `src/c3x3/search.rs:194` の `self.max_total >= 30` 修正と重複ループの排除、および `src/lib.rs` の `seq_len` 算出順序の修正。数行の修正で 3x3 既定ソルバーのタイムアウト後退（約 47%）と待ち時間を劇的に改善できます。
2. **M2（2x2 向き OFF 完成判定と LBL/Ortega の整合性）**: `web/main.ts`・`src/lib.rs`・`src/c2x2/mod.rs` において空間回転された 2x2 完成状態を正しく完成扱いとし、UI 表示・効果音・ソルバー挙動を一致させます。
3. **M5 & L1（3x3 冗長手相殺の常時適用・2x2 Ortega BFS 条件修正）**: `src/lib.rs` で全アルゴリズムに `cancel_redundant_moves_with_phases` を適用し、`src/c2x2/ortega.rs:117` を `< 6` に修正します。
4. **M3 & M4（種別切替時の盤面退避・SW クエリキャッシュの永続上限管理）**: `web/cube-store.ts` の種別ごと退避マップ実装と、`public/sw.js` の `ownCache.keys()` ベースの上限管理への移行を行います。
5. **L2〜L8（フォーカス復元、カメラ微修正、ガードレール・テスト・衛生面の改善）**。
