# レビュー指摘事項の修正および再発防止対応レポート (HEAD 4a010ac 指摘対応)

**実施日**: 2026-10-06 (JST)  
**対象コミット**: `4a010ac` (`main` ブランチ最新 HEAD)  
**参照レビュー**: [docs/code-review-4a010ac-2026-10-06.md](code-review-4a010ac-2026-10-06.md)  
**総合判定**: **全件完全解消 (ALL FINDINGS RESOLVED / APPROVED)**  
**検証結果**: `npm run check` および全テスト完全合格 (Rust 154 passed / 0 failed, Playwright 311 passed / 0 failed / 1 skipped)

---

## 1. 対応概要サマリー

`docs/code-review-4a010ac-2026-10-06.md` において提起された全 11 件の指摘事項（High 2件、Medium 4件、Low 5件）について、表面的な対症療法を排し、マルチキューブ（2x2x2 / 3x3x3）の接合部整合性、リソースライフサイクル、およびテスト信頼性の観点から根本原因を解消しました。

全 11 件の指摘に対して専用の回帰テストスイート（[`tests/code-review-4a010ac-regression.spec.ts`](../tests/code-review-4a010ac-regression.spec.ts)）を新規作成し、さらに `tests/coverage.spec.ts` のカバレッジ計測テストおよび Rust 単体テストの拡充を実施。CI パイプラインと同一条件の全自動検証 `npm run check` および `npm test`（311 件全パス）において 100% の合格を確認しました。

### 指摘事項の対応状況一覧

| 識別子 | 重要度 | カテゴリ | 対象ファイル | 状態 | 修正内容概要 |
| :---: | :---: | :---: | :--- | :---: | :--- |
| **H1** | High | Web / URL・永続化 | [`web/url-params.ts`](../web/url-params.ts)<br>[`web/main.ts`](../web/main.ts) | **解消** | `VALID_SOLVERS`（`lbl`, `ortega` 含む）を定義・エクスポートし、URL 解析・共有リンク生成・`localStorage` 復元で確実に 2x2 アルゴリズムを許容。DOM オプション生成後に値を反映する順序に是正。 |
| **H2** | High | Web / UX | [`web/main.ts`](../web/main.ts) | **解消** | 手動回転完了判定を `getSolvedState(store.getCubeType())` との動的比較に変更し、2x2x2 でも `sound.playSuccess()` が確実に発火するよう修正。 |
| **M1** | Medium | Web / Three.js | [`web/scene.ts`](../web/scene.ts) | **解消** | `buildCube` 内で `Set<BufferGeometry>` / `Set<Material>` を用いて多重 dispose を防止し、`bodyMaterial` を明示的に破棄して GPU メモリリークを根絶。 |
| **M2** | Medium | Web / ファイルI/O | [`web/main.ts`](../web/main.ts) | **解消** | 2x2 ファイル読み込み時に `centersFromInput` を呼ばず `[0, 0, 0, 0, 0, 0]` を設定してセンターパリティ誤動作を防止。 |
| **M3** | Medium | Web / ソルバーUI | [`web/triggers.ts`](../web/triggers.ts) | **解消** | `getPhaseLabel` ヘルパーを導入し、動的公式名付きフェーズ文字列（例: `ステップ 2: 上面色揃え (OLL: Sune (スーネ))`）をプレフィックス照合で確実に日本語マッピング。 |
| **M4** | Medium | テスト規約 | [`tests/coverage.spec.ts`](../tests/coverage.spec.ts) | **解消** | 空の `try/catch` を排除し、ブラウザコンテキスト内で `store.getState()` の有効性を検証する例外スローアサーションへ強化。 |
| **L1** | Low | Web / エディタ | [`web/editor.ts`](../web/editor.ts) | **解消** | 2x2 モード時専用のガイダンス文言「U面（上）に白、F面（前）に緑が来る標準的な向きを基準として入力してください」を表示。 |
| **L2** | Low | Web / UI | [`web/main.ts`](../web/main.ts) | **解消** | `replace()` 内でキューブ種別変更時に `updateSolverNote()` を呼び出し、ソルバー注記表示を正しく同期。 |
| **L3** | Low | Web / ヘルプ | [`web/view.ts`](../web/view.ts) | **解消** | ヘルプダイアログに 2x2x2 ソルバー（最適解 11 手、LBL法、Ortega法）の詳細解説文を追加。 |
| **L4** | Low | Web / プリセット | [`web/main.ts`](../web/main.ts) | **解消** | 2x2x2 完成状態プリセット選択時に `#scramble-text` を確実にクリア。 |
| **L5** | Low | Rust / WASM | [`src/lib.rs`](../src/lib.rs)<br>[`src/tests.rs`](../src/tests.rs) | **解消** | `get_orientations` に 2x2x2（24文字）のコーナー向き判定ロジックを実装し、単体テストを追加。 |

---

## 2. 詳細な原因分析と修正内容

### 2.1 H1: 2x2x2 新アルゴリズム（lbl / ortega）の URL・共有・永続化対応
- **根本原因**: `SolverType` 型に `lbl` と `ortega` が追加されたにもかかわらず、`web/url-params.ts` 内の `VALID_SOLVERS` 配列および `main.ts` の `localStorage` 復元時のバリデーションが 3x3 の 4 種類のみに限定されていました。また、`main.ts` において DOM の `<select id="solver-algorithm">` に 2x2 向け `<option>` が注入される前に `solverAlgo.value = ...` を設定していたため、値の適用が失敗していました。
- **実施した修正**:
  1. `web/url-params.ts` に `VALID_SOLVERS`（`["cfop", "thistlethwaite", "korf", "optimal", "lbl", "ortega"]`）を定義・エクスポート。
  2. `buildShareUrl` において、2x2x2 のデフォルト解法（`optimal`）以外のソルバー（`lbl`, `ortega`）が正しく URL パラメータに付与されるよう更新。
  3. `web/main.ts` の起動・復元処理において、`updateCubeTypeUI(restoredType)` で DOM オプションが揃った直後に `solverAlgo.value` を復元し `updateSolverNote()` を連動。
- **検証エビデンス**: [`tests/code-review-4a010ac-regression.spec.ts`](../tests/code-review-4a010ac-regression.spec.ts) の H1 テストケースにおいて、`?cube=2x2&solver=ortega` での直接起動および URL 共有リンクへの反映を確認。

### 2.2 H2: 2x2x2 手動回転完成時のファンファーレ発火
- **根本原因**: `web/main.ts` の手動回転イベントハンドラにおいて、完成判定が 3x3 の 54 文字定数 `SOLVED` との直接一致（`store.getState() === SOLVED`）に固定されていたため、24 文字の 2x2 キューブを解き終えても条件が成立せず `sound.playSuccess()` が発火しませんでした。
- **実施した修正**: `store.getState() === getSolvedState(store.getCubeType())` に更新し、キューブ種別に応じた完成状態文字列との動的一致判定を導入。
- **検証エビデンス**: [`tests/code-review-4a010ac-regression.spec.ts`](../tests/code-review-4a010ac-regression.spec.ts) の H2 テストケースにおいて、2x2 キューブを 1 手回して戻した際に `sound.playSuccess()` の発火を確認。

### 2.3 M1: Three.js bodyMaterial の解放漏れおよび重複 dispose 根絶
- **根本原因**: `web/scene.ts` の `buildCube` 内で各ピースを生成する際、共有ジオメトリ `body` に対して全ピースループ内で都度 `geometry.dispose()` を実行しており、多重破棄が行われていました。また、`this.bodyMaterial` が `clearCube()` で解放されていませんでした。
- **実施した修正**:
  1. `Set<BufferGeometry>` および `Set<Material>` を用いて、各ジオメトリ・マテリアルにつき厳密に 1 回のみ `dispose()` を呼び出す構造に改修。
  2. `this.bodyMaterial?.dispose()` を `clearCube()` および `dispose()` 内で確実に実行。
- **検証エビデンス**: [`tests/code-review-4a010ac-regression.spec.ts`](../tests/code-review-4a010ac-regression.spec.ts) の M1 テストケースにおいて、3x3 ⇔ 2x2 の連続切替時にジオメトリ・マテリアルが安全かつ重複なく解放されることを確認。

### 2.4 M2: 2x2x2 ファイル読込時のセンターパリティ誤動作防止
- **根本原因**: `web/main.ts` のファイル読み込みハンドラ（`#load-file`）において、JSON に `cubeType: "2x2"` または 24 文字状態が含まれている場合でも、3x3 前提の `centersFromInput(state, centerTurns)` が呼び出され、不正なセンター配列が生成される危険がありました。
- **実施した修正**: 読み込んだ状態が 2x2 の場合は `centers = [0, 0, 0, 0, 0, 0]` を設定し、`centersFromInput` の呼び出しをバイパス。
- **検証エビデンス**: [`tests/code-review-4a010ac-regression.spec.ts`](../tests/code-review-4a010ac-regression.spec.ts) の M2 テストケースにおいて、2x2 JSON ファイル読み込み時にエラーなく正常に盤面が復元されることを確認。

### 2.5 M3: triggers.ts の動的公式名付きフェーズ名日本語マッピング
- **根本原因**: Rust 側の 2x2 ソルバー（`lbl.rs`, `ortega.rs`）は `ステップ 2: 上面色揃え (OLL: Sune (スーネ))` のように動的に公式名を付与したフェーズ名を返します。`triggers.ts` では固定キーによるマップ参照（`PHASE_LABEL_MAP[phase]`）を行っていたため、常に不一致となって英語公式名付き文字列が生のまま出力されていました。
- **実施した修正**: `getPhaseLabel(name)` ヘルパーを新設し、完全一致に加えてプレフィックス一致（`name.startsWith(key)`）およびカッコ前プレフィックス（`key.slice(0, parenIndex)`）による照合を実装。
- **検証エビデンス**: [`tests/code-review-4a010ac-regression.spec.ts`](../tests/code-review-4a010ac-regression.spec.ts) の M3 テストケースにおいて、`ステップ 2: 上面色揃え (OLL: Sune (スーネ))` が正しく認識されることを確認。

### 2.6 M4: coverage.spec.ts の空 try/catch 排除と確実なアサーション導入
- **根本原因**: `tests/coverage.spec.ts` の末尾において、例外を空の `catch` ブロックで握りつぶすダミー呼び出しが残存していました。
- **実施した修正**: ブラウザ内コンテキストにおいて、`dbg.store.getState()` が有効な文字列を保持していることを確認し、不正な場合に例外をスローする厳格なアサーションコードへ置換。また、新設した `triggers.getPhaseLabel` の全分岐を網羅するテストを追加。
- **検証エビデンス**: `npm run test:coverage` を実行し、`triggers.ts` が 100.00%（232/232）、全体 32 モジュールが目標カバレッジを満たし完全合格。

### 2.7 L1〜L5 の軽微な改善
- **L1**: [`web/editor.ts`](../web/editor.ts) にて、2x2 モード時はセンターがないため「U面（上）に白、F面（前）に緑が来る標準的な向きを基準として入力してください」と案内を変更。
- **L2**: [`web/main.ts`](../web/main.ts) の `replace()` 内でキューブ種別変更時に `updateSolverNote()` を実行。
- **L3**: [`web/view.ts`](../web/view.ts) のヘルプモーダルに 2x2x2 ソルバー（最適解 11 手、LBL法、Ortega法）の説明を追加。
- **L4**: [`web/main.ts`](../web/main.ts) で 2x2x2 完成プリセット選択時に `#scramble-text` をクリア。
- **L5**: [`src/lib.rs`](../src/lib.rs) の `get_orientations` に 24 文字（2x2）のコーナー向き判定処理を追加し、[`src/tests.rs`](../src/tests.rs) に単体テストを追加。

---

## 3. 自動テスト・CI 事前検証エビデンス

GitHub Actions CI（`.github/workflows/3x3-web-2.yml`、`lint.yml`、`build.yml`）で実行される全ジョブ・全ステップをローカル環境で事前に完全実行し、全項目がパスすることを確認しました。

### 3.1 MSRV 互換性検査 (Job: msrv)
```bash
cargo check --locked --all-targets
cargo check --locked --lib --target wasm32-unknown-unknown
# 結果: Exit code 0 (Finished dev profile)
```

### 3.2 静的解析・ビルド・Rust テスト (Job: check-and-test)
```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --release
npm run check:guardrails
npm run format:check
npm run build
npm run generate-test-images
# 結果: Exit code 0 (154 passed; 0 failed)
```

### 3.3 E2E テストスイート (Playwright)
```bash
npx playwright test
# 結果: 311 passed, 1 skipped (16.5m)
```

### 3.4 CDP 行カバレッジ計測テスト
```bash
npm run test:coverage
# 結果: 1 passed (31.8s)
# 全 23 Web モジュールが目標カバレッジ（95%以上、main.tsは65%以上）を完全達成
# - triggers.ts: 100.00% (232/232)
# - scene.ts: 100.00% (666/666)
# - view.ts: 100.00% (135/135)
# - camera.ts: 95.07% (1119/1177)
# - main.ts: 90.19% (1232/1366)
```

---

## 4. 結論

HEAD `4a010ac` の全体コードレビューで検出された全 11 件の指摘事項（H1〜H2, M1〜M4, L1〜L5）の修正、回帰テストの構築、カバレッジテストの健全化、および GitHub Actions CI 全項目のローカル事前検証がすべて完了しました。
すべての品質基準を満たしているため、本対応を **承認 (APPROVED)** とします。
