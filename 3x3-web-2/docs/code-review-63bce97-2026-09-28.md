# 全体コードレビュー — `63bce97`（2026-09-28）

## 判定・対象・方法

**要修正。** `fix/superflip-preset` の `63bce97afe01563735d0258064a43d9dc260c2e3` を対象にした。レビュー開始時の作業ツリーはクリーン。製品コードは変更していない。前回のレビューや修正記録の「解消済み」という記述は、今回の判定根拠にしていない。

追跡対象は Rust の `src/` 14 ファイル、Web の `web/` 23 ファイル、テスト 47 ファイル、スクリプト・公開ファイル 20 ファイル、ビルド設定、および親リポジトリの Pages ワークフロー。合法局面の検査、4 探索方式（Kociemba, CFOP, Thistlethwaite, Korf）、センター向き、埋め込みテーブルと WASM 境界、Web のストア・Worker・再生・入力・カメラ・描画・保存、Service Worker とオフライン検証の経路を追った。既存テストを実行し、既存テストが通らない利用者操作の組合せを隔離ブラウザと実測スクリプトで再現した。

優先度は **P2 = 通常操作で誤った状態を適用し得る・探索結果を破棄し得るため公開前に修正**、**P3 = 限定条件下の表示・アクセシビリティ・二重非同期レースの不具合** とする。

## 指摘（優先順）

### F1 — P2: 解法再探索中のシーク・再生操作により完了した探索結果が無言で破棄される

**再現:** 12手のスクランブル局面 `B L2 D F2 R2 B2 U L2 D R2 F2 U'` で Kociemba 解法（18手）を表示させた後、ソルバーを CFOP に切り替えて再探索（`solve()`）を開始した。探索中（`solving = true`、キャンセルボタン表示中）にもかかわらず、画面下部には古い解法のタイムライン、再生ボタン、次へボタン（`#next`）、全18手の手順リストが有効なまま表示された。探索中にタイムラインで 1 手シークすると `store.revision` が 1 から 2 へ進み、探索完了時に `if (store.getRevision() !== at) return;` で新解法が破棄された。結果として `solutionAlgorithm` は CFOP に更新されず古い Kociemba のまま取り残された。[操作スクリプト](review-evidence/63bce97/probe-ui.mjs)と[状態記録](review-evidence/63bce97/probe-ui.json)を参照。

**原因:** [`solve`](../web/main.ts#L459) は探索開始時に古い解法を失効させず、[`store.setSolution(undefined)`](../web/cube-store.ts#L181) を呼ばない。さらに [`refresh`](../web/main.ts#L255) は `solving` フラグを再生系ボタン（`play`, `first`, `prev`, `next`, `last`, `timeline`）の `disabled` 判定に含めていない。このため、探索中に利用者がシーク（[`updateAfterSeek`](../web/cube-store.ts#L194)）すると `this.revision++` が走り、探索完了時の [`at照合`](../web/main.ts#L490) で不一致となって探索結果が静かに捨てられる。

**影響・修正案:** 時間をかけて計算した探索結果が、単なる手順プレビュー操作によって失われ、画面には古い解法が残り続ける。`solve()` の冒頭で古い解法をクリアするか、あるいは探索中はタイムライン・再生コントロール・手順リスト・キーボードの左右矢印シークを無効化（`disabled`）する。探索中のシーク操作を行っても探索が安全に継続または明示的にキャンセルされる統合テストを追加する。

---

### F2 — P2: カメラエディタ終了後、画像実体が破棄されても DOM カードが「読込完了」を表示し続ける

**再現:** カメラエディタで画像 A（640×480）を選択し、ステータスが「読込完了 (640×480)」になった後、ダイアログの「閉じる」ボタン（または Escape キー）でダイアログを閉じた。内部では `imageA` が `undefined` になり Blob URL も解放されたが、DOM の `#camera-status-a` は「読込完了 (640×480)」のまま残り、`#camera-drop-a` にも `has-file` クラスが付与されたままとなった。[操作スクリプト](review-evidence/63bce97/probe-ui.mjs)と[状態記録](review-evidence/63bce97/probe-ui.json)（`f3AfterClose`）を参照。

**原因:** [`handleDialogClose`](../web/camera.ts#L1078) は `imageA` / `imageB` を `undefined` に戻し URL を破棄するが、[`updateCardStatus`](../web/camera.ts#L357)、[`renderResults`](../web/camera.ts#L614)、[`update`](../web/camera.ts#L800) を呼び出さない。また、`this.faces` や `this.detectedLabels`、ファイル入力の `inputA.value` / `inputB.value` も閉じる際にクリアされない。

**影響・修正案:** ダイアログを閉じた後、あるいは再オープン処理の合間に、画像が存在しないにもかかわらずカードが「読込完了」を表示する内部・表示乖離が生じる。またファイル入力値がリセットされないため、同じファイルを再度選択した際に `change` イベントが発火しない。`handleDialogClose` 内でカードステータス、検出ラベル、面データ、ファイル入力を一貫して初期状態へリセットし、UI 表示を同期させる。閉じる操作後の DOM 状態をアサートするテストを追加する。

---

### F3 — P3: 0手解法（完成状態）のタイムライン `aria-valuetext` が「完成 (0手)」ではなく「開始状態」と読まれる

**再現:** キューブが完成状態（SOLVED）のまま「解法を探す」を実行すると、0手で完了（`stepCount = "0 / 0"`、ガイダンス「6面が揃いました。おつかれさまでした。」）となる。しかしタイムラインの `aria-valuetext` は `"開始状態"` と設定された。[実画面の取得値](review-evidence/63bce97/probe-ui.json)（`f1Timeline`）を参照。

**原因:** [`refresh`](../web/main.ts#L265) の条件分岐において、`step === 0` が `step === solution.moves.length` より先に評価されている。0手解法では `step === 0` かつ `solution.moves.length === 0` の両方が成立するが、`step === 0` が優先され、次の手順（`nextMeta`）が存在しないため `"開始状態"` が返される。

```typescript
// web/main.ts L265-273
const valuetext =
  step === 0
    ? nextMeta
      ? `開始状態。次は1手目 ${nextMeta.move} (${nextMeta.phaseLabel})`
      : "開始状態" // ← 0手完成時もここに入ってしまう
    : step === solution.moves.length
      ? `完成 (${solution.moves.length}手)`
      : `${step}手完了。次は${step + 1}手目 ${nextMeta?.move || ""} (${nextMeta?.phaseLabel || ""})`;
```

**影響・修正案:** スクリーンリーダーの利用者に「0手で完成している」という事実が伝わらず、画面ガイダンスとタイムラインのスライダー読み上げが矛盾する。`solution.moves.length === 0`（または `step === solution.moves.length`）を優先判定し、0手完成時は `"完成 (0手)"` と読み上げるように分岐順序を是正する。

---

### F4 — P3: 探索中にプリセットを選択した際、探索が即時中断されずステータス表示が競合する

**再現:** CFOP 探索中にプリセット「簡単（3手）」ボタンをクリックした。プリセットの非同期 `fetch` 中に探索は中断されず、バックグラウンドで走ったままとなった。プリセットが成功して「✓ 簡単（3手） を読み込みました」と表示された直後に、古い探索の完了通知「53,668 ノードを探索 · 完成を検証」が `#solver-note` に書き込まれた。[実画面の取得値](review-evidence/63bce97/probe-ui.json)（`f4AfterPreset`）を参照。

**原因:** [`initializePresets`](../web/main.ts#L889) のクリックハンドラは `stop()` を呼ぶが、[`cancelSearch()`](../web/main.ts#L93) を呼んでいない。このため、プリセット取得のネットワーク往復中も Worker での探索が継続し、完了タイミングによっては `solver-note` や探索メッセージがプリセットの読み込み表示と入り乱れる。

**影響・修正案:** 新しいプリセット局面を選んだにもかかわらず、直前の探索結果のノード数やメッセージが表示され、利用者に混乱を与える。プリセットボタン押下時に即座に `cancelSearch()` を呼び出し、探索 Worker を確実に中断させる。探索中のプリセット押下で探索が即座に停止し、ステータス表示が汚染されないことをテストする。

---

## 既存テストで見逃す理由

| 指摘 | 現行テストが通る範囲 | 追加すべき境界 |
| --- | --- | --- |
| F1 | [再生テスト](../tests/playback-controls.spec.ts)は解法取得後に静止状態でシーク・再生を検証する。 | 探索処理（`solving = true`）が走っている最中にタイムラインや再生操作を行い、探索完了後の結果採否を確認すること。 |
| F2 | [カメラライフサイクルテスト](../tests/camera-lifecycle.spec.ts)はダイアログを開いている最中の挙動を検証する。 | ダイアログを閉じた（Escape/閉じるボタン）直後に DOM カードのステータスやクラス、ファイル入力値が初期化されていること。 |
| F3 | [アクセシビリティテスト](../tests/accessibility.spec.ts)および[前回回帰テスト](../tests/code-review-011a7b9-regression.spec.ts)は 2 手以上の解法で各ステップの読み上げを検証する。 | 手数が 0 手の解法（完成状態での解法生成）におけるタイムラインの `aria-valuetext`。 |
| F4 | [プリセットテスト](../tests/ui-interaction.spec.ts)はアイドル状態でのプリセット選択と手動回転の競合を検証する。 | 探索中にプリセットボタンを押下した際、即座に Worker がキャンセルされ探索ノートが残存しないこと。 |

---

## 横断確認

| 領域 | 現行コードで確かめた内容と境界 |
| --- | --- |
| Rust・WASM | [`parse_state`](../src/cube.rs#L33) の色数・センター・重複・向き・置換パリティを確認。4 探索方式は [`solve_state_with_algorithm`](../src/lib.rs#L85) に集約され、最終局面・センター向きの検証は release でも実行される。`cargo test --release` は 141 成功・1 ignored。 |
| 状態・入力 | [`CubeStore`](../web/cube-store.ts) の revision と Undo/Redo・プレビュー、Worker の ID・世代照合、JSON 64KB・画像 20MB の読込前ガード、共有 URL・保存キーを確認。F1 は `solve()` 中にプレビューシークを許容する状態ガードの漏れに起因する。 |
| 描画・PWA | [`CubeScene.dispose`](../web/scene.ts#L334) のジオメトリ・マテリアル・テクスチャとリスナー破棄、カメラのストリーム停止・デコード世代、SW のスコープ別キャッシュ削除・クエリ正規化・`waitUntil` を確認した。F2 はダイアログクローズ時の DOM 反映漏れである。 |
| 公開・依存 | [Pages ワークフロー](../../.github/workflows/deploy-pages.yml)の `build` は `test-3x3-web-2` を前提にする。利用者入力の主要な表示は `textContent`。`npm audit --json` の既知アドバイザリ数は 0。 |

---

## 検証と残る範囲

- `npm run check:guardrails`、`npm run typecheck`、`npm run format:check`、`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings` は全件成功。
- `cargo test --release`: **141 passed / 1 ignored**。
- `npm test`（Playwright / Chromium）: **200 passed / 1 skipped / 0 failed**。通過は F1〜F4 の不在を意味しない。再現条件と既存テストの差は上表の通り。
- ローカル環境は macOS、Node `v26.7.0`、Rust `1.98.1`、`wasm-pack 0.15.0`。
- F1〜F4 は開発サーバー上の実ブラウザ（Chromium）で再現し、`docs/review-evidence/63bce97/probe-ui.json` に実測値を記録した。
- 実機カメラデバイス、Chromium 以外のブラウザエンジン（WebKit, Firefox）、実ネットワーク遅延、長時間稼働でのヒープメモリプロファイルは今回の検証外。
- 本レビューで変更したのはこのレポート、再現プローブ、証跡データ、およびレビュー索引のみ。製品コードへの修正は未実施であり、F1〜F4 の回帰テストは既存テストにまだ含まれていない。
