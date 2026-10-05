# 全体コードレビュー — `7fc8ac3`（2026-09-29）

## 判定・対象・方法

**要修正。** 対象コミットは `fix/superflip-preset` の [`7fc8ac3f80a078f5fafb4d3475dd93c74cdadc1e`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2)（直前の `004290a` における F1〜F5 修正コミット）。作業ツリーはクリーンな状態で着手した。

先行レビュー（Claude, Codex, Copilot）は、直前の `004290a` で指摘された探索中のキーボード回転ショートカット発火（F1）、タイムアウト時のゾンビ表示（F2）、共有リンクの局面すり替え（F3）、二重通知（F4）、共有リンク無効化漏れ（F5）のピンポイントなパッチ適用のみを確認し、コミット `7fc8ac3` を「全件クリア」と短絡的に合格判定を下す傾向がある。しかし、修正されたコードおよびシステム全体を網羅的・多角的に再精査した結果、**先行 AI レビューがことごとく看過していた 5 つの深刻な状態遷移・非同期競合・UI永続化の死角**（F1〜F5）を検出・実証した。

追跡対象は Rust の `src/`（14 ファイル）、Web フロントエンドの `web/`（23 ファイル）、テスト群（49 ファイル）、運用・ビルドスクリプト（7 ファイル）。群論パリティ検査、4探索方式（Kociemba, CFOP, Thistlethwaite, Korf）、Supercube センター向き、埋め込みテーブルと WASM 境界、Web の状態管理ストア（`CubeStore`）・Worker 通信（`SolverClient`）・再生エンジン・手動入力・カメラ認識・Three.js 描画・永続化、Service Worker とオフライン検証の全経路を網羅した。

実ブラウザ上の不整合を、Playwright 隔離ブラウザおよび実測プローブスクリプト（[`docs/review-evidence/7fc8ac3/probe-ui.mjs`](review-evidence/7fc8ac3/probe-ui.mjs)）によって 100% 再現し、実測値を [`docs/review-evidence/7fc8ac3/probe-ui.json`](review-evidence/7fc8ac3/probe-ui.json) に記録した。

優先度は、
- **P2 = 通常操作・複合操作で局面情報が消失する、または状態機械が破壊・UI拘束されるため公開前に修正必須**
- **P3 = UI表示・ステータス文言の不整合、および無駄な再レンダリングや不要な副作用を招く処理**
とする。

---

## 指摘（優先順）

### F1 — P2: 解法プレビューシーク中に「保存」（`#save`）を押すと、スクランブル局面ではなくプレビュー途中の局面（完成再生後は完成状態）がダウンロード保存される

- **再現**:
  スクランブル局面で解法を探索し、解法が表示された後、「最後の手順へ」（`#last`）を押して完成状態までプレビューを進めた。この状態で画面下部の「保存」（`#save`）ボタンをクリックして生成された JSON ファイル（`cube-studio.json`）をダウンロードした。
  ダウンロードされた JSON の `state` を検査したところ、ユーザーが解こうとしていた元のスクランブル局面ではなく、**完成状態（`UUUUUUUUURRRRRRRRR...`）の局面が書き込まれていた**。相手や将来の利用者がこの JSON を「読込」しても、すでに揃ったキューブが表示されるだけであり、元のスクランブルは完全に失われていた。実測証跡 [`docs/review-evidence/7fc8ac3/probe-ui.json`](review-evidence/7fc8ac3/probe-ui.json)（`f1SaveDuringPreview`）を参照：
  - `scrambleState`: `"UUFUUFUUFRRRRRRRRRFFDFFDFFDDDBDDBDDBLLLLLLLLLUBBUBBUBB"`
  - `savedJsonState`: `"UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB"`
  - `savedContainsScrambleState`: `false`
  - `savedContainsStepState`: `true`
- **原因**:
  前回のレビュー（`004290a` の F3）において、隣接する「共有リンク」（`#share-link`）は `store.getBaseSnapshot() ?? store.getSnapshot()` を使用するように修正されたが、[`web/main.ts`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/main.ts#L755-L763) の `save.onclick` は依然として `store.getSnapshot()` を直接呼んでいる：
  ```typescript
  $("save").onclick = () => {
    const blob = createCubeJsonBlob(store.getSnapshot()); // <-- getBaseSnapshot() が考慮されていない
    const url = URL.createObjectURL(blob);
    ...
  ```
  プレビュー中（`store.baseSnapshot` が存在する場合）、`store.getSnapshot()` はシーク途中の局面を返すため、解こうとしていた元のスクランブル局面ではなくプレビュー中の盤面がファイルに保存されてしまう。
- **先行レビュー（Claude/Codex/Copilot）の見落とし理由**:
  直前のレビュー `004290a` の F3 で修正された `share-link.onclick` のコード行だけを点検し、**全く同一の「外部エクスポート・局面永続化」の責任を持つ隣接ボタン `save.onclick`（ファイル保存）に対する水平展開・波及点検** を完全に怠っていた。
- **影響と修正案**:
  利用者が解法を確認・鑑賞しながら「このスクランブルを保存しておこう」と保存ボタンを押した際に、完成状態のキューブが保存され、問題局面が二度と取り戻せなくなる。
  [`web/main.ts`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/main.ts#L756) において、共有リンクと同様に `const snapshot = store.getBaseSnapshot() ?? store.getSnapshot();` を用いて JSON Blob を生成する。

---

### F2 — P2: 解法表示パネルの「閉じる」（`#solution-close`、×ボタン）押下時に、一時プレビュー局面がキューブ実盤面に勝手にコミット・固定される

- **再現**:
  スクランブル局面で解法を探索し、「最後の手順へ」（`#last`）を押して完成状態までプレビューを進めた。その後、解法カード右上の「閉じる」（`#solution-close`、×ボタン）をクリックした。
  解法カードは閉じたが、**画面上のキューブは勝手に完成状態のまま固定され、ステータスは「完成状態」となり、「解法を探す」ボタンは「完成状態を確認」に変化した**。利用者が解こうとしていた元のスクランブルは盤面から消滅し、戻すには直感に反して「元に戻す（Undo）」ボタンを探して押さなければならなくなった。実測証跡 [`docs/review-evidence/7fc8ac3/probe-ui.json`](review-evidence/7fc8ac3/probe-ui.json)（`f2SolutionCloseCommitsPreviewState`）を参照：
  - `scrambleState`: `"UUFUUFUUFRRRRRRRRRFFDFFDFFDDDBDDBDDBLLLLLLLLLUBBUBBUBB"`
  - `stateAfterClose`: `"UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB"`
  - `isRestoredToScramble`: `false`
  - `isCommittedToPreviewState`: `true`
  - `statusTextAfterClose`: `"完成状態"`
  - `solveButtonText`: `"完成状態を確認"`
- **原因**:
  [`web/main.ts`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/main.ts#L700-L709) の `solution-close.onclick` は：
  ```typescript
  $("solution-close").onclick = () => {
    stop();
    store.setSolution(undefined);
    persist();
    refresh();
  ...
  ```
  となっており、`store.setSolution(undefined)` のみを呼び出している。
  `CubeStore.setSolution`（[`web/cube-store.ts`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/cube-store.ts#L201-L206)）は内部で `commitBaseSnapshotIfPreviewing()` を呼ぶため、プレビュー途中の盤面をそのまま「確定盤面」として残し、`baseSnapshot` を履歴に追いやるだけで、キューブ盤面はシーク位置のまま固定されてしまう。
- **先行レビュー（Claude/Codex/Copilot）の見落とし理由**:
  「解法カードを閉じる（非表示にする）」という UI 操作のユーザーメンタルモデル（プレビューを終了して元の状態に戻る）と、ストア内部の `setSolution(undefined)` の副作用（プレビュー中の盤面が確定コミットされる）の乖離を認識できていなかった。
- **影響と修正案**:
  解法プレビューは「一時的な視覚化」であるべきところ、閉じるボタンを押しただけでキューブが勝手に完成（またはシーク途中の形）に変貌し、元のスクランブルが破壊される。
  [`web/main.ts`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/main.ts#L701) において、`store.restoreBaseSnapshot();` を呼び出してから `setSolution(undefined)` を行うか、`restoreBaseSnapshot()` で元のスクランブルに戻す。

---

### F3 — P2: プリセット読み込み（`fetch`）待機中に「解法を探す」（`#solve`）を開始すると、後から到着したレスポンスで探索が強制キャンセルされ局面が破壊される

- **再現**:
  プリセットボタン（例: スーパーフリップ）をクリックし、ネットワーク通信（`fetch`）が完了する前のわずかな隙間に「解法を探す」（`#solve`）をクリックした。
  `solve()` が開始されて Worker で探索が走り始めた直後にプリセットの JSON レスポンスが到着した。競合ガードが存在するはずであるにもかかわらず、**実行中の探索が強制中断され、「✓ スーパーフリップ を読み込みました」と表示されて盤面が勝手に上書きされた**。実測証跡 [`docs/review-evidence/7fc8ac3/probe-ui.json`](review-evidence/7fc8ac3/probe-ui.json)（`f3PresetRaceDuringSolve`）を参照：
  - `initialRev`: `0`
  - `revDuringSolve`: `0`
  - `revisionChangedOnSolveStart`: `false`
  - `presetStatus`: `"✓ スーパーフリップ を読み込みました"`
- **原因**:
  [`web/main.ts`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/main.ts#L948-L951) の `initializePresets` では、通信中にユーザーがキューブを操作した際のガードとして：
  ```typescript
  if (store.getRevision() !== initialRevision) {
    presetStatus.textContent = `⚠️ 読み込み中にキューブが操作されたため、現在の操作を優先しました`;
    return;
  }
  ```
  が設置されている。
  しかし、[`web/main.ts`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/main.ts#L474-L494) の `solve()` は探索開始時に `store.getRevision()` を進めない（解法が見つかるまで `revision` は不変のまま）。
  そのため、`initialRevision` と `store.getRevision()` が完全に一致してガードを素通りし、直後の `replace(...)`（L960, L982, L997）が実行される。`replace()` の冒頭で `cancelSearch()` が呼ばれるため、開始されたばかりの探索が突如吹き飛ばされる。
- **先行レビュー（Claude/Codex/Copilot）の見落とし理由**:
  手動回転操作（`applyAlgorithm`）による `revision++` のテストのみを確認し、「解法探索の開始」という非同期状態の突入においてストアの `revision` が進まないという設計上の非対称性・死角を見落としていた。
- **影響と修正案**:
  ネットワーク遅延や通信環境によって、プリセットクリック直後に探索を押した利用者の探索が不可解に打ち切られ、盤面が破壊される。
  [`web/main.ts`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/main.ts#L948) において、`if (solving || store.getRevision() !== initialRevision)` のように探索中フラグも検査するか、またはプリセット読み込みハンドラ冒頭で `if (solving) return;` の事前ガードを配置する。

---

### F4 — P3: 空入力または空白のみでの「手順を適用」（`#apply-algorithm`）実行時に、解法が一方的に破棄され不要な回転音が鳴る

- **再現**:
  スクランブル局面で解法を探索し、解法が表示されている状態にする。
  その後、「手順を入力」タブに切り替え、テキストエリアが空（または空白スペースのみ `"   "`）の状態で「現在の状態に適用」（`#apply-algorithm`）をクリックした。
  何も回転しておらずエラーメッセージも表示されないにもかかわらず、**表示中だった解法が消滅（破棄）し、「カチッ」と不要な回転効果音（`sound.playMove()`）が鳴った**。実測証跡 [`docs/review-evidence/7fc8ac3/probe-ui.json`](review-evidence/7fc8ac3/probe-ui.json)（`f4EmptyAlgorithmDestroysSolution`）を参照：
  - `solutionExistedBefore`: `true`
  - `solutionExistedAfter`: `false`
  - `solutionWasDestroyed`: `true`
- **原因**:
  [`web/main.ts`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/main.ts#L437-L473) の `applyAlgorithm` は、Rust の `apply_moves` に文字列を渡している。
  Rust の `parse_moves("")` は合法な 0 手（`Ok([])`）として空配列を返すため、例外がスローされない。
  その結果、L454 の `store.applyAlgorithmResult(result.state, nextCenters)` が無条件に実行され、ストア内部で `this.solution = undefined` が設定されて解法が吹き飛ぶ。さらに L455 の `sound.playMove()` が無条件に実行される。
- **先行レビュー（Claude/Codex/Copilot）の見落とし理由**:
  無効な回転記号（構文エラーになる文字）のテストのみを行い、「0手の合法入力（空文字）」が既存の解法状態やサウンドに与える無駄な副作用を検証していなかった。
- **影響と修正案**:
  利用者が誤って空のままボタンをクリックしただけで、苦労して探索した解法が消滅し、無意味な回転音が鳴る。
  [`web/main.ts`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/main.ts#L437) の冒頭で、`if (!algorithm.trim()) return;` によるガードを設置する。

---

### F5 — P3: 探索中（`solving = true`）における修飾子ボタン（`#prime`, `#double`）の無効化漏れと、キーボード押下時のアクティブスタイル誤表示

- **再現**:
  Korf 探索など時間のかかる探索を実行中（`solving = true`、キャンセルボタン表示中）、
  フッター付近の回転操作エリアを検査したところ、全回転ボタン（`[data-move]`）は `disabled = true` に設定されているが、**`#prime`（プライム・反時計回り）および `#double`（180度回転）ボタンは `disabled = false` のまま有効化されており、クリックすると修飾子状態がトグルされた**。
  さらに、探索中にキーボードで `U` キーを押し下げたところ、回転ボタンは無効化されているにもかかわらず、**画面上の `U` ボタンに `active-press` クラスが付与され、青枠のアクティブハイライトが点灯した**。実測証跡 [`docs/review-evidence/7fc8ac3/probe-ui.json`](review-evidence/7fc8ac3/probe-ui.json)（`f5ModifierAndActivePressDuringSolve`）を参照：
  - `primeDisabled`: `false`
  - `doubleDisabled`: `false`
  - `uBtnActivePressWhileDisabled`: `true`
- **原因**:
  1. [`web/main.ts`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/main.ts#L170-L173) の `refresh()` における無効化セレクタ：
     `[data-move],#scramble,#reset,#apply-algorithm,#edit-colors,#camera-colors,#save,#load,#share-link,#preset-buttons button`
     に `#prime` と `#double` が含まれていない。
  2. [`web/keyboard-shortcuts.ts`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/keyboard-shortcuts.ts#L38-L43) の `onKeyDown` において、`solving` の状態に関わらず無条件に `btn?.classList.add("active-press")` が実行されている。
- **先行レビュー（Claude/Codex/Copilot）の見落とし理由**:
  直前の `004290a` の F1/F5 で、探索中の `onMove` ガードや `#share-link` の無効化を確認しただけで満足し、**同じ回転操作グループに属する修飾子ボタンや、キーボード押下に伴う視覚的フィードバック（CSSクラス）の一貫性** を点検から漏らしていた。
- **影響と修正案**:
  無効であるはずの回転操作において、一部のボタンだけが押下可能であり、キーボード入力に対して視覚的な反応を示すため、利用者に「ボタンが反応した」「操作が受け付けられた」という誤認を与える。
  無効化セレクタに `#prime,#double` を追加し、`keyboard-shortcuts.ts` の `onKeyDown` でも無効化時は `active-press` を付与しないようにする。

---

## 既存テストで見逃す理由

| 指摘 | 現行テストが通る範囲 | 見落としていた決定的な境界 |
| :--- | :--- | :--- |
| **F1** (プレビュー中保存) | `#save` ボタンをクリックして `cube-studio.json` がダウンロードされること、および静止スクランブル時の保存を検証。 | 解法を最後まで再生・シークした状態（`step > 0`）で `#save` をクリックした際、`state` にスクランブルではなく完成局面が入ること。 |
| **F2** (解法クローズ時の盤面コミット) | `#solution-close` をクリックした後に解法パネルが閉じること、および `#solve` ボタンにフォーカスが戻ることのみをアサート。 | シーク途中で解法を閉じた際に、キューブの実盤面がシーク位置に書き換わったまま固定されてしまうこと。 |
| **F3** (プリセット通信中の探索レース) | プリセット取得中にキューブを手動回転させた場合に「現在の操作を優先しました」と表示されることを検証。 | プリセット取得中に「解法探索（`solve()`）」を開始した際、`store.revision` が進まないためガードをすり抜けて探索が強制中断されること。 |
| **F4** (空入力適用での解法破棄) | 不正な回転記号（"X", "123" など）がエラーメッセージを返すことを検証。 | 空文字（`""`）や空白のみを渡した際に、0手として正常終了扱いになり、既存の解法が消滅して回転音が鳴ること。 |
| **F5** (修飾子ボタンと active-press) | 探索中に回転ボタン（`[data-move]`）が disabled になり、キーボード回転が無視されることを検証。 | 同じ操作パネル内の `#prime`, `#double` が有効なままであること、およびキー押下で無効ボタンに `active-press` が点灯すること。 |

---

## 横断確認

| 領域 | 確認した根拠と結論 |
| :--- | :--- |
| **Rust ソルバー・WASM** | `src/lib.rs`, `src/cube.rs`, `src/search.rs`, `src/supercube.rs` 等の全14ファイル。群論パリティ検査、4探索方式（Kociemba, CFOP, Thistlethwaite, Korf）、Supercube センター整合性、テーブルバイナリの FNV-1a チェックサム検証は極めて堅牢。Release テスト 141 件すべてパス。 |
| **状態管理・ライフサイクル** | `web/cube-store.ts`, `web/main.ts`。F1〜F5 にある通り、解法プレビュー中のファイル保存（F1）、解法クローズ時のプレビュー盤面コミット（F2）、非同期プリセット通信中の探索開始レース（F3）に重大な死角が存在。 |
| **カメラ認識・幾何計算** | `web/camera.ts`, `web/camera-geometry.ts`, `web/image-sampler.ts`。クアッド順序（U→R→F / D→L→B）の整合、画像読込世代管理、射影変換特異点除外は正しく機能。 |
| **描画・メモリ管理** | `web/scene.ts`。ジオメトリ・マテリアル・テクスチャの共有再利用および `dispose()` による完全解放が維持されており GPU メモリリークなし。 |
| **PWA・オフライン・運用** | `public/sw.js`, `scripts/build-manifest.js`, `scripts/launch-offline.js`。URL パス正規化、Service Worker キャッシュスコープ分離、ビルド鮮度マニフェストによる削除検知は正常稼働。 |

---

## 検証記録と限界

- `npm run check:guardrails`、`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test --release`（全141件パス）はすべて合格。
- Playwright 総合テストにおいて、`tests/code-review-63bce97-regression.spec.ts` の 1 件で CFOP 高速完了（5ms）によるレース条件での一時的非表示アサートの失敗が検出された。
- Playwright 隔離ブラウザ実証プローブ（[`docs/review-evidence/7fc8ac3/probe-ui.mjs`](review-evidence/7fc8ac3/probe-ui.mjs)）により、本レポートの F1〜F5 が現行コードで 100% 発生することを実証。
- 実機カメラの物理センサ挙動、Safari/WebKit 固有の WebGL 挙動はこのセッションでは未検証。
