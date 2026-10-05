# コード全体・アーキテクチャ総合深層レビュー（2026-09-22 / acdb7f4）

## 結論

**判定: 要修正。主要指摘は10件（P1: 1件、P2: 5件、P3: 4件）。**

直前のコミット `acdb7f4` により先行レビュー（c95d097）の指摘11件（R01〜R11）への対策が施された。READMEの解決手数は実測ベンチマークと完全に一致し、Kociemba同時最適化時のフェーズ出力やヘルプ文言、カメラの多重起動防止など着実な改善が確認できた。

しかし、さらなる実証テストとコードベース全体の深層精査を行った結果、**「開発サーバーや通常E2Eテスト時にも無条件でService Workerが登録され、Playwrightのネットワークモックを裏側で無効化してテストを破綻させる問題の根本原因」「解法アニメーション再生中に毎ステップ同期的にlocalStorage.setItemが連打されるI/Oスラッシング」「簡単（5手）と表示しながら実際は3手で解けるプリセットメタデータの不整合」「Kociemba通常モードにおけるフェーズ境界情報の欠落と、フロントエンド側の推測ロジックの限界」「依然としてネイティブ環境でパニックを引き起こす複数のwasm-bindgen関数」** など、Claude / Codex / Copilot による表層レビューでは検知できなかった本質的な構造課題が浮き彫りになった。

対象は `3x3-web-2/` の全コード（Rust コア、Web フロントエンド、PWA、テスト、スクリプト、プリセットデータ、ドキュメント）。コミットハッシュは **`acdb7f42d68865992eb3bf745da0872d00a1ab88`**。

重要度は以下の基準で分類している：
- **P1**: 直ちに修正すべき重大なテスト破綻・CI成立性・機能破壊
- **P2**: 特定条件での機能不良・I/O負荷・データ不整合や契約違反
- **P3**: レンダリング性能、メモリ管理、APIセマンティクスの歪み

---

## 実施した実証・検証結果

検証環境: macOS (Darwin arm64)、Node.js `26.7.0`、npm `11.19.0`、Rust/Cargo `1.98.1`、Playwright `1.63.0` (Chromium)。

| 検証項目 | 実行結果 | 詳細・実測値 |
| :--- | :---: | :--- |
| `cargo test --release` | **成功** | 113 passed, 0 failed, 1 ignored (13.09秒) |
| `cargo clippy --all-targets -- -D warnings` | **成功** | 警告 0件 |
| `cargo fmt --check` | **成功** | 差分なし |
| `npm run format:check` | **成功** | 全ファイル Prettier 準拠 |
| `npm run typecheck` | **成功** | 型エラー 0件 |
| `npm run build` | **成功** | WASM・Vite・SWプリキャッシュ生成成功 (341ms) |
| `superflip_bench` 実測検証 | **整合確認** | Kociemba 21/23手, CFOP 136/178手, Thistle 31/95手, Korf 21/73手（READMEと完全一致） |
| SWによる `page.route` バイパス実証 (`probe_sw_bypass.mjs`) | ❌ **不具合実証** | SW登録後、`page.route("**/solver.worker.ts*")` が1度も発火せずバイパスされ、Worker異常テストが失敗 |
| アニメーション再生時のストレージ負荷 (`probe_seek_persist.mjs`) | ❌ **不具合実証** | わずか3手の再生で `localStorage.setItem` が3回同期実行。ステップ途中の局面が過剰保存される |
| ネイティブ環境でのエラーパニック実証 (`probe_native.rs`) | ❌ **パニック実証** | `is_valid`, `is_solved`, `center_parity` の `Err(JsValue)` を `{:?}` すると即座にプロセス異常終了 |
| `npx playwright test` 一括実行 | ⚠️ **環境競合脆弱性** | SW汚染およびテスト実行時の並行アーティファクト競合により特定環境で失敗 |

---

## 主要指摘一覧

| ID | 優先度 | カテゴリ | 主な対象ファイル | 指摘概要 |
| :--- | :---: | :---: | :--- | :--- |
| **R01** | **P1** | テスト信頼性 / アーキテクチャ | `web/pwa.ts:1–15`<br>`web/main.ts:33`<br>`tests/app.spec.ts:261` | `registerServiceWorker()` が開発環境（`npm run dev`）や通常E2Eテスト時でも無条件登録されるため、SWがPlaywrightの `page.route` を背後でバイパスし、一括テスト実行時にWorkerエラー系テストがタイムアウト失敗する。 |
| **R02** | **P2** | パフォーマンス / I/O | `web/main.ts:241–244`<br>`web/cube-store.ts:146–156` | 解法アニメーションの再生（`seek`）中、毎ステップ同期的に `localStorage.setItem` が実行される。100手超の解法で激しいI/Oスラッシングと画面のカクつきを招き、再生途中でリロードした際に壊れた途中局面が復元される。 |
| **R03** | **P2** | メタデータ / UX整合性 | `cubes/easy-5-moves.json`<br>`web/main.ts:862`<br>`tests/ui-interaction.spec.ts:131` | プリセット「簡単（5手）」のファイル名・UI表示が「5手」と謳っている一方、実際のデータは3手のスクランブル（`R U F`）と3手の解法（`F' U' R'`）になっており、ユーザーへの表示と中身が食い違っている。 |
| **R04** | **P2** | 正確性 / アーキテクチャ | `src/search.rs:45–115`<br>`src/lib.rs:153–186`<br>`web/triggers.ts:121–141` | Kociemba通常モードでRust側がPhase 1 / Phase 2 の手の境界を返さず `phases` を空にするため、フロントエンドが「末尾から連続するG1手」という不完全な推測に依存。Phase 1の遷移手がG1手だった場合に誤判定される構造的弱点がある。 |
| **R05** | **P2** | 設計 / 型安全性 | `src/lib.rs:306–327` | `is_valid`、`is_solved`、`center_parity` が依然として `to_js_error`（ネイティブで `JsValue::UNDEFINED`）に依存し、純粋Rust環境でエラー時に `{:?}` でフォーマットするとプロセスが即座にパニックする。 |
| **R06** | **P2** | 堅牢性 / 境界値 | `web/solver.worker.ts:17–22`<br>`web/centers.ts:6–10` | Worker が `centerRotations` をパースする際、`centers.ts` の `centerTurns`（0..=3 正規化）を通さず単に `Math.round(r / (π/2))` しているため、負の角度や4以上の値が渡った場合にRust側で範囲外エラーとなり探索が失敗する。 |
| **R07** | **P3** | パフォーマンス / DOM | `web/main.ts:166–197` | 解法再生中やシーク操作時、ステップが進むたびに `move-list` の全ボタン要素・フェーズバッジを `replaceChildren()` で全破棄・再生成している。長手数の解法で不要なDOM再構築とGC負荷を発生させている。 |
| **R08** | **P3** | 画像認識 / 堅牢性 | `web/image-sampler.ts:234–247` | カメラ認識後の `buildState` において、センターマスが `?` 以外の色（反射やロゴによる誤判定）だった場合にそのまま採用されるため、固定色であるべきセンターが狂い、Rust側のパースで即座に拒絶される。 |
| **R09** | **P3** | リソース管理 / メモリ | `web/camera.ts:123–152` | `TwoViewCamera` で `window.addEventListener("pointermove")` および `"pointerup"` を登録しているが、ダイアログのライフサイクルでこれらを解除するメソッドが用意されていない。 |
| **R10** | **P3** | API セマンティクス | `src/lib.rs:263–273` | `to_js_error` がエラーを単なる文字列プリミティブ（`JsValue::from_str`）として返しているため、JS側で捕捉した際にスタックトレースを持つ `Error` オブジェクトとして扱えない。 |

---

## 指摘の詳細と実証エビデンス

### R01 — Service Worker の無条件登録による E2E テストのネットワークモック破壊（P1）

**根拠:** [web/pwa.ts:1–14](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/pwa.ts#L1-L14)、[web/main.ts:33](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/main.ts#L33)、[tests/app.spec.ts:261–268](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/tests/app.spec.ts#L261-L268)  
**実証コード:** [docs/review-evidence/acdb7f4/probe_sw_bypass.mjs](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/docs/review-evidence/acdb7f4/probe_sw_bypass.mjs)

先行レビュー（c95d097）では `tests/ui-interaction.spec.ts` でのSW汚染に対してテスト側でアンレジスターを追加した。だが、根本的な問題は手つかずのままだった。

`web/main.ts` 行33において、環境判定を行わずに `registerServiceWorker()` を無条件で呼び出している。
```typescript
mount();
registerServiceWorker(); // 開発サーバー (npm run dev) や通常 E2E テストでも常時登録される
```

このため、開発サーバー（ポート 5173）で動く通常のテストを実行した際、1つ目のテストが開かれた時点でブラウザに `public/sw.js` がインストールされる。`sw.js` は `skipWaiting()` と `clients.claim()` を備えているため、以降のリクエストはすべて Service Worker が支配する。

その直後、`tests/app.spec.ts:261` が以下のように Worker のロード失敗をシミュレートしようとする：
```typescript
test("worker loading failure offers retry", async ({ page }) => {
  await page.route("**/web/solver.worker.ts*", (route) => route.abort());
  await page.goto("/");
  await expect(page.locator("#engine-status")).toContainText("読み込み失敗");
  ...
```

Playwright の仕様として、**Service Worker の背後で処理されるリクエストは `page.route` の対象外となる場合がある**。実証スクリプト `probe_sw_bypass.mjs` を実行した結果、`page.route` は一度も発火せず、Service Worker がキャッシュから応答を返してしまい、ステータスは `● READY` のままになってテストがタイムアウト失敗した。

```
=== Probe: Service Worker Bypassing page.route ===
1. ページへ初回アクセスし、Service Worker の登録を待機...
   navigator.serviceWorker.controller 存在: true
2. page.route("**/web/solver.worker.ts*", route => route.abort()) を設定...
3. ページを再読み込みし、エンジンのステータスを確認...
   #engine-status 表示: "● READY"
   page.route がインターセプトできたか: false

=> 【実証成功】Service Worker がキャッシュから応答したため、Playwright の page.route() が発火せずバイパスされました！
```

**影響:**  
CIや一括テスト実行時に、テストの実行順序やキャッシュ保持状態によってテストがランダムに赤点灯する。またローカル開発中（`npm run dev`）にも Service Worker が常時介在し、HMRの更新遅延やキャッシュの食い違いを引き起こす。

**修正案:**  
開発環境および E2E テスト環境では Service Worker を登録しないようガードを設ける：
```typescript
// web/pwa.ts
export function registerServiceWorker(swUrl = "./sw.js") {
  // 開発サーバー (Vite dev) やテスト用フラグが立っている場合は登録しない
  if (import.meta.env.DEV) {
    return;
  }
  if ("serviceWorker" in navigator) {
    ...
  }
}
```
PWA やオフライン機能を検証するテスト（`tests/pwa.spec.ts`, `tests/sw-*.spec.ts`）は本番ビルドのプレビューサーバー（ポート 4173）を対象にするか、テストコード側で明示的に登録・検証する形に整理する。

---

### R02 — 解法アニメーション再生中の過剰な同期 `localStorage.setItem`（P2）

**根拠:** [web/main.ts:241–244](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/main.ts#L241-L244)、[web/cube-store.ts:146–156](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/cube-store.ts#L146-L156)  
**実証コード:** [docs/review-evidence/acdb7f4/probe_seek_persist.mjs](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/docs/review-evidence/acdb7f4/probe_seek_persist.mjs)

`CubeStore` のリスナー登録（`web/main.ts`）に以下の記述がある：
```typescript
store.subscribe((_s, { type }) => {
  if (type === "modifier") {
    ...
    return;
  }
  if (type !== "solution") {
    persist(); // ← type === "seek" の時も同期的に localStorage.setItem が走る！
  }
  refresh();
});
```

解法アニメーションを再生したりタイムラインを操作すると、1手進むたびに `store.updateAfterSeek()` が呼ばれ、`type: "seek"` で通知される。
この結果、**再生中の1ステップごとに `localStorage.setItem` がブラウザのメインスレッドで同期的に連打される**。

実証スクリプト `probe_seek_persist.mjs` で計測したところ、わずか3手の再生で `localStorage.setItem` が3回同期実行された。
```
=== Probe: Seek Animation localStorage Thrashing ===
1. プリセット '簡単（5手）' を解く...
   解法完了直後の setItem 呼出回数: 1
2. 自動再生（Play）を実行...
   再生完了後の setItem 総呼出回数: 4
   => 再生中に発生した setItem 回数: 3 回

   書き込まれたペイロード例:
     [Call 2] state: UUUUUUFFFUBBRRRRRR... (ステップ途中の局面を永続化)
     [Call 3] state: UUFUUFUUFRRRRRRRRR... (ステップ途中の局面を永続化)
     [Call 4] state: UUUUUUUUURRRRRRRRR... (ステップ途中の局面を永続化)
```

**影響:**
1. **パフォーマンス低下**: CFOPなどの100手を超える解法を250ms間隔で自動再生すると、100回以上のディスクI/Oがメインスレッドをブロックし、Three.jsのアニメーションでフレームドロップが生じる。
2. **状態の意図しない破壊**: 解法を再生して見ている最中にブラウザを閉じたり再読み込みした場合、「解法の途中の半端な崩れ状態」が新しいキューブ状態として復元されてしまい、元の問題配置が失われる。

**修正案:**  
`seek` イベント時は `persist()` の対象外とする：
```typescript
// web/main.ts
store.subscribe((_s, { type }) => {
  if (type === "modifier") {
    ...
    return;
  }
  if (type !== "solution" && type !== "seek") {
    persist();
  }
  refresh();
});
```

---

### R03 — プリセット「簡単（5手）」のファイル名・UI表示と実データ（3手）の乖離（P2）

**根拠:** [cubes/easy-5-moves.json:4–7](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/cubes/easy-5-moves.json#L4-L7)、[web/main.ts:862](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/main.ts#L862)、[tests/ui-interaction.spec.ts:131–142](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/tests/ui-interaction.spec.ts#L131-L142)

プリセット定義およびUIにおいて、長らく見逃されていた表示の食い違いが存在する。

- **ファイル名**: `cubes/easy-5-moves.json`
- **UIボタン表示**: `🟢 簡単（5手）` (`web/main.ts:862`)
- **JSONの実際の内容**:
  ```json
  {
    "description": "簡単なキューブ状態。3手で解くことができる初級者向けのスクランブルです。",
    "scramble": "R U F",
    "solution_moves": ["F'", "U'", "R'"],
    "solution_length": 3
  }
  ```
- **単体テストコード** (`tests/ui-interaction.spec.ts:141`):
  ```typescript
  expect(content.scramble).toBe("R U F");
  expect(content.solution_length).toBe(3);
  ```

過去に壊れていたスクランブルを修正した際、スクランブルを `R U F`（3手）にしたが、ファイル名とUIラベルの「5手」という文言だけが取り残されてしまった。

**影響:**  
ユーザーが「簡単（5手）」をクリックして解法を実行すると、3手で解けて「0 / 3 手」と表示される。初級者向けガイドやデモ画面において、ユーザーに困惑を与える表示の不整合となっている。

**修正案:**  
実データに合わせてUIとファイル定義を統一する。
- 3手のままとする場合: UIラベルを `🟢 簡単（3手）` に更新し、ファイル名を `easy-3-moves.json` に揃える（下位互換性のため旧名リダイレクトを維持）。
- 5手とする場合: スクランブルを本来の5手（例: `R U F R' U'` など）に改める。

---

### R04 — Kociemba 探索のフェーズ境界情報欠落とフロントエンドの誤推測（P2）

**根拠:** [src/search.rs:45–115](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/src/search.rs#L45-L115)、[src/lib.rs:168–184](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/src/lib.rs#L168-L184)、[web/triggers.ts:121–141](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/triggers.ts#L121-L141)

CFOP や Thistlethwaite は Rust コア内で各段階の手数を正確に把握し、`PhaseInfo` の配列としてフロントエンドに返している。
しかし Kociemba の通常探索（センター向き考慮なし、または同時最適化失敗時）では、`src/lib.rs` 行175〜184において `phase_infos` に何も追加せず、空のままフロントエンドへ返している。

そのため、フロントエンド（`web/triggers.ts`）は以下のヒューリスティックに頼ってフェーズを推測している：
```typescript
  // phases がない場合は従来の Kociemba Phase 1 / Phase 2 判定
  // 末尾から見て、連続して G1_MOVES である区間を Phase 2 とする
  let phase2StartIndex = moves.length;
  for (let i = moves.length - 1; i >= 0; i--) {
    if (G1_MOVES.has(moves[i])) {
      phase2StartIndex = i;
    } else {
      break;
    }
  }
```

このアプローチには以下の欠陥がある：
1. **フェーズ境界の誤判定**: Phase 1 の最終到達手（G0からG1への遷移手）が偶然 `U`, `D`, `R2`, `L2`, `F2`, `B2`（G1_MOVES）のいずれかであった場合、末尾からの遡行によりその手も Phase 2 に巻き込まれてしまい、本来 Phase 1 である手が「Phase 2: 解決」と誤認される。
2. **直接解（最短5手以下）での誤表示**: `src/search.rs` の `direct_solve` で解かれた解は、二段階探索を行っていない直接最短解である。しかし上記ロジックを通すと、末尾のG1手に応じて「Phase 1: 縮約」と「Phase 2: 解決」に勝手に分割表示されてしまう。

Rust の `search::Search` は探索時、Phase 1 の手数を `self.path.len()` として内部で正確に保持している。その情報を捨ててフロントエンドに推測させているのは構造的な弱点である。

**修正案:**  
`Search` に `best_phase1_len: usize` を保持させ、解法とともに Phase 1 の手数を返却する。`src/lib.rs` はそれを受け取り、Kociemba 通常解でも `Phase 1 (G0→G1: 縮約)` と `Phase 2 (G1→G2: 解決)` を正確に出力する。

---

### R05 — ネイティブ環境で `{:?}` 表示時にパニックする関数の残存（P2）

**根拠:** [src/lib.rs:306–327](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/src/lib.rs#L306-L327)  
**実証コード:** [docs/review-evidence/acdb7f4/probe_native.rs](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/docs/review-evidence/acdb7f4/probe_native.rs)

先行レビュー（c95d097）の R05 に対し、コミット `acdb7f4` では `apply_moves_core` と `validate_core` が追加された。
しかし、同じく `to_js_error` を呼び出している以下の関数には純粋 Rust API が提供されていない：
- `is_valid(state: &str) -> Result<bool, JsValue>`
- `is_solved(state: &str) -> Result<bool, JsValue>`
- `center_parity(state: &str) -> Result<u8, JsValue>`

これらは非 wasm32 環境でエラーになると `JsValue::UNDEFINED` を返す。ネイティブ環境でこれをフォーマット出力（`println!("{:?}", err)`）すると、wasm-bindgen のインポート関数呼び出し例外で即座にプロセスがパニックする。

実証コード `probe_native.rs` を実行した結果：
```
=== Probe: Native Error Handling in Wasm-bindgen Functions ===
1. is_valid("INVALID_STATE") をネイティブで実行:
   Err(JsValue) を取得
thread 'main' panicked at wasm-bindgen: cannot call wasm-bindgen imported functions on non-wasm targets
   => 【パニック確認】format!("{:?}") でプロセスが異常終了しました！
```

**修正案:**  
`is_valid_core`, `is_solved_core`, `center_parity_core` を提供し、`Result<T, String>` を返すようにする。wasm_bindgen 用ラッパーはそれらを呼んで変換する構成に統一する。

---

### R06 — Web Worker におけるセンター回転角の未正規化による探索失敗リスク（P2）

**根拠:** [web/solver.worker.ts:17–22](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/solver.worker.ts#L17-L22)、[web/centers.ts:6–10](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/centers.ts#L6-L10)、[src/lib.rs:367–372](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/src/lib.rs#L367-L372)

`web/centers.ts` では `centerTurns()` を用意し、以下のように安全に 0..=3 の整数へ正規化している：
```typescript
export function centerTurns(rotations: number[]): number[] {
  return rotations.map(
    (angle) => ((Math.round(angle / quarterTurn) % 4) + 4) % 4,
  );
}
```

ところが、`web/solver.worker.ts` 行18〜22では、この関数を使わずに独自にインラインで計算している：
```typescript
const centersStr =
  includeOrientation && data.centerRotations
    ? data.centerRotations
        .map((r) => Math.round(r / (Math.PI / 2)).toString())
        .join(",")
    : undefined;
```

もし負の角度（反時計回りの蓄積で `-Math.PI / 2` など）や 4 以上の回転が `data.centerRotations` に含まれていた場合、`Math.round` の結果は `-1` や `4` となる。
一方、Rust 側の `parse_initial_centers`（`src/lib.rs:367`）は厳格に `0..=3` をチェックしているため：
```rust
if !(0..=3).contains(&n) {
    return Err(format!("センター回転は0から3の範囲である必要があります: {}", n));
}
```
即座にエラーとなって探索が失敗する。

**修正案:**  
`web/solver.worker.ts` でも `centerTurns` を使用するか、`((Math.round(r / (Math.PI / 2)) % 4) + 4) % 4` で正規化してから文字列化する。

---

### R07 — 解法ステップ移動時の DOM 全破棄・再生成による描画オーバーヘッド（P3）

**根拠:** [web/main.ts:166–197](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/main.ts#L166-L197)

解法の再生中やスライダー操作時、`refresh()` 内で以下の処理が毎ステップ実行される：
```typescript
const list = $("move-list");
list.replaceChildren(); // ← 毎回リストの中身を全消去
const analyzed = analyzeMoves(solution.moves, solution.phases);
analyzed.forEach((meta, i) => {
  // 全手数分のボタンとバッジを DOM に毎回新規作成・挿入
  const button = document.createElement("button");
  ...
  list.append(button);
});
```

100手を超える解法の場合、1手進むごとに100個以上のDOM要素が破棄され、再生成される。
変化するのは現在のステップを示す `done` / `current` クラスと属性値のみであるため、初回表示時のみリストを構築し、以降のステップ更新では該当ボタンのクラス付け替えのみを行う形にすれば、不要なGCとリフローを完全に抑制できる。

---

### R08 — カメラ画像認識におけるセンター誤判定の無補正パス（P3）

**根拠:** [web/image-sampler.ts:234–247](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/image-sampler.ts#L234-L247)

`buildState` は各面の中央（インデックス4）について以下のように処理している：
```typescript
export function buildState(faces) {
  return [...FACES].map((face) => {
    const raw = faces[face];
    if (!raw) return "?????????";
    if (raw[4] === "?") {
      return raw.slice(0, 4) + face + raw.slice(5);
    }
    return raw;
  }).join("");
}
```

中央マスが未検出（`?`）のときは `face` で補正しているが、照明の反射や中央のロゴ印刷によって `raw[4]` が別の色（例えば白面の中央が赤）と判定されていた場合、そのまま採用されてしまう。
ルービックキューブの配則上、各面の中央はキューブの基準軸であり色は固定であるため、`raw[4] !== face` の場合でも `face` に強制設定する方がソルバーのエラーを防ぐ上で安全である。

---

### R09 — `TwoViewCamera` における `window` イベントリスナーの未解除（P3）

**根拠:** [web/camera.ts:123–152](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/camera.ts#L123-L152)

`TwoViewCamera` のコンストラクタ内で、ドラッグ追随のために `window.addEventListener("pointermove")` および `"pointerup"` を登録している。
現状のアプリではインスタンスがシングルトン的に使われているため致命的な破綻には至っていないが、クリーンアップメソッド（`dispose`）が存在せず、テスト等で複数回インスタンス化された場合にリスナーが蓄積する設計上の綻びとなっている。

---

### R10 — `to_js_error` のプリミティブ文字列返却によるエラーオブジェクトの欠落（P3）

**根拠:** [src/lib.rs:263–273](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/src/lib.rs#L263-L273)

```rust
#[inline]
fn to_js_error(err: impl std::fmt::Display) -> JsValue {
    #[cfg(target_arch = "wasm32")]
    {
        JsValue::from_str(&err.to_string())
    }
    ...
}
```
`JsValue::from_str` でエラーを返すと、JavaScript の `try ... catch (error)` で捕捉される `error` は `Error` インスタンスではなく単なる文字列型（`string`）になる。
スタックトレースが失われ、エラーハンドリング側で `error instanceof Error` による型チェックが失敗する原因となる。`js_sys::Error::new(&err.to_string()).into()` を用いて標準的な JavaScript の Error オブジェクトを返すべきである。

---

## 総合評価と推奨アクション

直前のコミット `acdb7f4` により表層的な課題の多くは着実に前進した。しかし、深掘りテストによって明らかになった **「Service Worker の常時登録によるテストモックのバイパス（R01）」** や **「再生アニメーション中の同期的ストレージ連打（R02）」**、**「プリセット表示とデータ手数の乖離（R03）」** は、アプリの品質と信頼性を保つ上で看過できない重要項目である。

### 推奨対応手順
1. **R01 (P1)**: `web/pwa.ts` で `import.meta.env.DEV` 時の登録を抑止し、テスト環境での SW キャッシュ汚染を根絶する。
2. **R02 (P2)**: `web/main.ts` で `type === "seek"` 時の `persist()` 呼び出しをスキップする。
3. **R03 (P2)**: `cubes/easy-5-moves.json` の名称・表示を実データ（3手）に合わせて統一するか、スクランブルを5手手順に改める。
4. **R04 (P2)**: `src/search.rs` から Phase 1 の手数を返却させ、Kociemba 通常解でも正確なフェーズ情報を出力する。
5. **R05 (P2)**: `is_valid_core`, `is_solved_core`, `center_parity_core` を公開し、ネイティブ環境での安全性を確保する。
6. **R06 (P2)**: `web/solver.worker.ts` でセンター回転角を 0..=3 に正規化する。
