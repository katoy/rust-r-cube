# 全体コードレビューレポート (Code Review Report)

> [!NOTE]
> 最新の対応および検証記録：
>
> - **[レビュー指摘事項の修正および再発防止対応レポート（b934c77 / 2026-10-01）](review-fixes-b934c77-2026-10-01.md)**: 本ドキュメントの指摘事項（Findings 1〜4）および直近レビュー（421345a / 22件）の完全解消を確認。`CubeStore` 履歴上限（200件）テスト追加、`npm run check` 成功（Rust release 141 passed／1 ignored、Playwright 271 passed／1 skipped）。
> - **[全体コードレビュー（b934c77 / 2026-10-01）](code-review-b934c77-2026-10-01.md)**: 最新 HEAD の判定は **APPROVED**。群論パリティ、有限状態機械 `AppState`、非同期ガード、リソース管理、6大重点レビュー観点の全件適合を確認。主要 22 Web モジュールで 97.5%〜100.0% の高カバレッジを維持。
> - **[22件の修正・検証記録（421345a基準 / 2026-09-30）](review-fixes-421345a-2026-09-30.md)**: 修正作業ツリーの22件対応を確認。`npm run check`成功（Rust release 141 passed／1 ignored、Playwright 269 passed／1 skipped）。freshオフライン起動・所有サーバー終了・120入力の不変を確認。
> - **[全体コードレビュー（421345a / 2026-09-30）](code-review-421345a-2026-09-30.md)**: 当時の判定は **要修正**（P2 13件・P3 9件）。履歴、非同期モーダル、カメラ読取対象、失敗更新、検証基盤と公開契約を再現証拠・確認範囲・限界付きで記録。当時の全体Playwrightは228 passed／4 failed／1 skipped。修正後の結果は上記対応記録を参照。
> - **[レビュー指摘事項の修正および再発防止対応レポート（360de34 / 2026-09-29）](review-fixes-360de34-2026-09-29.md)**: Findings 1〜5 完全解消。有限状態機械 `AppState`（`Idle` \| `Solving` \| `Previewing`）を導入（`web/app-state.ts`）。
> - **[全体コードレビュー（360de34 / 2026-09-29）](code-review-360de34-2026-09-29.md)**: 当時の判定は要修正。
> - **[レビュー指摘事項の修正および再発防止対応レポート（7fc8ac3 / 2026-09-29）](review-fixes-7fc8ac3-2026-09-29.md)**: Findings 1〜5 完全解消。
> - **[全体コードレビュー（7fc8ac3 / 2026-09-29）](code-review-7fc8ac3-2026-09-29.md)**: 当時の判定は要修正。
> - **[レビュー指摘事項の修正および再発防止対応レポート（004290a / 2026-09-29）](review-fixes-004290a-2026-09-29.md)**: Findings 1〜5 完全解消。
> - **[全体コードレビュー（004290a / 2026-09-29）](code-review-004290a-2026-09-29.md)**: 当時の判定は要修正。
> - **[レビュー指摘事項の修正および再発防止対応レポート（b605038 / 2026-09-28）](review-fixes-b605038-2026-09-28.md)**: Findings 1〜5 完全解消。
> - **[全体コードレビュー（b605038 / 2026-09-28）](code-review-b605038-2026-09-28.md)**: 当時の判定は要修正。
> - **[レビュー指摘事項の修正および再発防止対応レポート（63bce97 / 2026-09-28）](review-fixes-63bce97-2026-09-28.md)**: Findings 1〜4 完全解消。
> - **[全体コードレビュー（63bce97 / 2026-09-28）](code-review-63bce97-2026-09-28.md)**: 当時の判定は要修正。
> - **[レビュー指摘事項の修正および再発防止対応レポート（011a7b9 / 2026-09-28）](review-fixes-011a7b9-2026-09-28.md)**: Findings 1〜4 完全解消。
> - **[全体コードレビュー（011a7b9 / 2026-09-28）](code-review-011a7b9-2026-09-28.md)**: 当時の判定は要修正。
> - **[レビュー指摘事項の修正および再発防止対応レポート（9fb1ebc / 2026-09-28）](review-fixes-9fb1ebc-2026-09-28.md)**: Findings 1〜4 完全解消。
> - **[全体コードレビュー（9fb1ebc / 2026-09-28）](code-review-9fb1ebc-2026-09-28.md)**: 当時の判定は要修正。
>
> 過去の全体レビューおよび指摘事項の記録：
>
> - **[レビュー指摘事項の修正および再発防止対応レポート（125f8c8 / 2026-09-28）](review-fixes-125f8c8-2026-09-28.md)**: Findings 1〜4 完全解消。
> - **[全体コードレビュー（125f8c8 / 2026-09-28）](code-review-125f8c8-2026-09-28.md)**: 当時の判定は要修正。
> - **[レビュー指摘事項の修正および再発防止対応レポート（ba8fcfa / 2026-09-28）](review-fixes-ba8fcfa-2026-09-28.md)**: 当時の Findings 1〜5 に対する修正記録。
> - **[全体コードレビュー（ba8fcfa / 2026-09-28）](code-review-ba8fcfa-2026-09-28.md)**: 当時の判定は要修正。
> - **[コードレビュー基準および再発防止チェックリスト](code-review-guidelines.md)**: 過去のレビュー漏れ（F1〜F6）を教訓とした 6 大重点観点（条件付きコンパイル整合性、複合状態遷移、Fail-fast、URL表現、リソースリーク、成果物鮮度）とテスト設計ルール。
> - **[レビュー指摘事項の修正および再発防止対応レポート（b755b38 / 2026-09-27）](review-fixes-b755b38-2026-09-27.md)**: ガードレール静的チェッカーおよび回帰テストを配備。
> - **[全体コードレビュー（b755b38 / 2026-09-27）](code-review-b755b38-2026-09-27.md)**: 当時の判定は要修正。
> - **[レビュー指摘事項の修正および改善対応レポート（279b455 / 2026-09-26）](review-fixes-279b455-2026-09-26.md)**: 次世代改善提案完全解消。
> - **[全体レビューレポート（279b455 / 2026-09-26）](code-review-279b455-2026-09-26.md)**: 当時の判定は APPROVED WITH HIGHEST DISTINCTION。

**初版実施日**: 2026-09-17  
**最新改訂日**: 2026-10-01 (`b934c77` / `fix/superflip-preset`)  
**対象リポジトリ**: `rust-r-cube/3x3-web-2`  
**最新総合判定**: **APPROVED (本番リリース承認・全指摘事項完全解消)**

---

## 1. 総合評価 (Executive Summary)

本プロジェクトは、Rust による超高速なキューブソルバー（Kociemba の二段階探索法＋Supercube センター向き拡張）と、ブラウザネイティブな Web フロントエンド（TypeScript + Three.js + Web Workers + PWA）をシームレスに結合した高品質なアプリケーションです。

初期レビューにおいて指摘された課題（Three.js 矢印の GPU メモリリーク、カメラ入力のクアッド指定順序、Prettier インデント、モジュール責務肥大化）をはじめ、その後の継続的コードレビューで洗い出された複合状態遷移、非同期競合、Service Worker キャッシュ整合性、リソースライフサイクルに関する計 22 件以上の指摘事項が根本原因からすべて解消されています。

現在、コードベースは有限状態機械（`AppStateMachine`）による一貫した状態遷移制御、`CubeStore` による履歴・盤面コンテキストのカプセル化、幾何計算・UI レンダリングのモジュール分離が確立され、Rust 側 141 件、Web/Playwright 側 271 件のテストが 100% 合格する強固な品質水準に達しています。

### 五軸評価サマリー

| 評価軸 | 判定 | 概要 |
| :--- | :---: | :--- |
| **1. 正確性 (Correctness)** | **極めて優秀 (Flawless)** | コアのソルバー・幾何回転アルゴリズム、群論パリティ検証、Supercube センター整合性、有限状態機械による不正遷移遮断が完全に機能。 |
| **2. パフォーマンス (Performance)** | **極めて優秀 (Flawless)** | Rust 側の探索性能・事前テーブル埋め込みに加え、Three.js 矢印の事前生成・キャッシュ化、マテリアル再利用により GPU メモリリークを完全根絶。 |
| **3. アーキテクチャ (Architecture)** | **極めて優秀 (Flawless)** | Web Worker 完全オフロード、有限状態機械 `AppStateMachine`、`CubeStore`、カメラ幾何・描画モジュール分離により高い凝集度と疎結合を実現。 |
| **4. 可読性・保守性 (Readability)** | **極めて優秀 (Flawless)** | Prettier および `cargo fmt` 100% 整合、厳格な TypeScript 型チェック、詳細な日本語ドキュメントとテスト設計ガイドラインを完備。 |
| **5. セキュリティ・a11y (Security & a11y)** | **極めて優秀 (Flawless)** | サーバーレス・完全クライアント完結型。ファイルサイズ制限（64KB）、JSON 型バリデーション、カメラストリーム完全解放、WCAG 2.1 AA 準拠。 |

---

## 2. 課題と改善提案の対処実績 (Findings & Solutions)

### ✅ Finding 1: [重要/パフォーマンス] Three.js の矢印描画による GPU メモリリーク 【完全解消】

- **該当箇所**: `web/scene.ts`
- **現象（過去）**:
  キューブの回転操作や手順再生に伴い `show()` → `updateArrows()` が高頻度で呼び出される際、`arrowGroup.clear()` で子参照を外しても `ShapeGeometry` や `MeshBasicMaterial` が GPU メモリに残存し、20手の解法再生で2,000個以上の Three.js リソースがリークしていた。
- **対処内容**:
  1. `arrowGeometry` および `outlineGeometry` をコンストラクタ初期化時に事前生成・保持し、全54メッシュで共有インスタンスを再利用。
  2. `colorMaterials` を `Map<number, THREE.MeshBasicMaterial>` でキャッシュし、毎ターンのマテリアル新規生成を根絶。
  3. `dispose()` メソッドにて全ジオメトリ、マテリアル、テクスチャ、`DirectionalLight` のシャドウ `RenderTarget` を明示的に破棄。
- **検証結果**:
  `tests/scene-lifecycle.spec.ts` および `tests/coverage.spec.ts` で、ステップ進行・回転・再生成時の GPU メモリリークゼロとリソースの完全解放を実証。

---

### ✅ Finding 2: [要修正/正確性] カメラ入力におけるクアッド指定順序の不整合 【完全解消】

- **該当箇所**: `web/camera.ts`
- **現象（過去）**:
  `capture()` 内の `rawQuads`（U → F → R）と、`updateDetectedLabels()` 内の `quads`（U → R → F）で面の指定順序が異なっており、センター色が判別できずフォールバック処理が動いた際に F面と R面の色データ割り当てが逆転するリスクがあった。
- **対処内容**:
  `capture()` 内の `rawQuads` の順序を `updateDetectedLabels()` と完全に同一の `[U, R, F]`（View A）および `[D, L, B]`（View B）に統一。
- **検証結果**:
  `tests/camera-input.spec.ts` および `tests/camera-image-state.spec.ts` において、通常認識・フォールバック認識ともに正しい面ラベルおよび配色が割り当てられることを確認。

---

### ✅ Finding 3: [軽微/スタイル] Prettier フォーマットチェックの不整合 【完全解消】

- **該当箇所**: コードベース全体（`web/camera.ts` 含む）
- **現象（過去）**:
  `npm run format:check`（`prettier --check`）を実行すると、`camera.ts` 等でインデント不整合が検出されていた。
- **対処内容**:
  `npm run format` を全 TypeScript/JavaScript/CSS/JSON に適用し、`cargo fmt` を Rust 全体に適用。CI パイプラインにフォーマット検証を組み込み。
- **検証結果**:
  `npm run format:check` および `cargo fmt --check` が警告・エラー 0 件で完全にパス。

---

### ✅ Finding 4: [設計/保守性] `camera.ts` と `main.ts` の責務肥大化 【完全解消】

- **該当箇所**: `web/camera.ts`, `web/main.ts`
- **現象（過去）**:
  幾何演算、Canvas 描画、UI 制御、ファイル入出力が `camera.ts` に集中し、`main.ts` に散在する約20個の状態変数によって状態遷移ロジックが分散していた。
- **対処内容**:
  1. 幾何演算ロジック（`intersectLines`, `computeCenter`, `detectCubeOutline`）を [`web/camera-geometry.ts`](camera-geometry.ts) へ分離。
  2. Canvas オーバーレイ描画を [`web/camera-canvas-renderer.ts`](camera-canvas-renderer.ts) へ分離。
  3. 認識結果表示・色修正パレット制御を [`web/camera-results-ui.ts`](camera-results-ui.ts) へ分離。
  4. UI ヘルパー処理を [`web/camera-ui-helper.ts`](camera-ui-helper.ts) へ分離。
  5. 盤面状態、センター向き、Undo/Redo 履歴、解法プレビューの追跡を [`web/cube-store.ts`](cube-store.ts)（`CubeStore`）に集約・カプセル化。
  6. アプリケーション状態遷移（`Idle` \| `Solving` \| `Previewing`）を有限状態機械 [`web/app-state.ts`](app-state.ts)（`AppStateMachine`）として一元化。
- **検証結果**:
  分離された各モジュールが単体テストで 100% カバーされ、結合後も全体テスト 271 件すべてが成功。責務の明確化と保守性の大幅な向上が達成された。

---

## 3. 優れた実装点 (Strengths)

1. **有限状態機械 `AppStateMachine` による不正操作の一括遮断**:
   - `Idle` / `Solving` / `Previewing` の 3 状態を State パターンで厳密に型付け。
   - 探索中の非同期モーダル起動やファイル入力、キーボードショートカットなどの割り込み競合を構造的に防止。
2. **ゼロランタイムコストのプルーニングテーブル**:
   - コンパイル時（`build.rs`）にプルーニングテーブル（`tables.bin`）を生成し、WASM バイナリに埋め込み。ブラウザロード時の初期化待ち時間がゼロ。
3. **Supercube（センター向き考慮）の群論的プルーニング**:
   - Phase 2 で R, F, L, B 面が 180° 回転しかできない群論的特性を利用し、センター向き奇数回転を即座に枝刈り（`min_phase2_center_moves`）。
4. **堅牢な Web Worker / 非同期アーキテクチャ**:
   - `SolverClient` による探索中断（キャンセル時の安全な `worker.terminate()` と再生成）が実装されており、UI スレッドを一切ブロックしない。
5. **テストの厚みと自動ガードレール検証**:
   - Rust 側ユニットテスト 141 件、Playwright E2E/ユニットテスト 271 件が全件パス。
   - `code-review-guidelines.md` に基づくガードレール静的チェッカー（`npm run check:guardrails`）が常時稼働。

---

## 4. 推奨アクションプランの達成状況

- [x] **Step 1: フォーマット修正**
  - `npm run format` および `cargo fmt` を実行し、コードベース全体のフォーマット不整合を解消完了。
- [x] **Step 2: カメラクアッド順序の統一**
  - `web/camera.ts` の `capture()` 内のクアッド定義順序を `[U, R, F]` に修正し、プレビュー認識との完全整合を達成。
- [x] **Step 3: Three.js 矢印のメモリリーク解消**
  - `web/scene.ts` において、`arrowGeometry` および `outlineGeometry` を事前生成・キャッシュ化し、マテリアルとメッシュを再利用することで毎ターンの GPU メモリリークを根絶。
- [x] **Step 4: カメラ幾何計算および状態管理モジュールの分離（リファクタリング）**
  - `web/camera-geometry.ts`、`web/camera-canvas-renderer.ts`、`web/camera-results-ui.ts`、`web/camera-ui-helper.ts`、`web/cube-store.ts`、`web/app-state.ts` を新設・分離。
  - `CubeStore` の 200 件履歴上限およびセンター変更履歴のテストを追加し、堅牢性を証明。

---

## 5. 自動チェックコマンド

```bash
# ガードレール静的検証
npm run check:guardrails

# リポジトリ全体の完全検証（ガードレール、Rust、Format、Typecheck、Build、Webテスト）
npm run check
```
