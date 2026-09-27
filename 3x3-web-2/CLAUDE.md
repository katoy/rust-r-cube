# CLAUDE.md - Cube Studio (3x3-web-2) 開発 & レビュー指示

本ファイルは、本リポジトリでコード実装・リファクタリング・コードレビューを行うすべての開発者および AI エージェント（Claude Code, Antigravity 等）に対する共通指示書です。

---

## 1. プロジェクト概要

Rust で実装された Kociemba 探索法・CFOP・Supercube（センター向き）ソルバーを WASM 化し、Web Worker + Three.js + TypeScript で動作させるブラウザ完結型 3x3 ルービックキューブスタジオです。

---

## 2. 開発・検証コマンド

```bash
# 全体チェック（ガードレール静的解析、Rust fmt/clippy/test、Web format/build/test）
npm run check

# ガードレール静的検証（レビュー漏れ再発防止ルール）
npm run check:guardrails

# ビルド
npm run build

# テスト実行
cargo test --release                  # Rust ソルバーテスト（必ず --release でも確認）
npm test                              # Playwright E2E テスト
npm run test:coverage                 # カバレッジ測定テスト

# オフライン起動検証
npm run offline -- --headless --fresh # 最新ビルドでのオフライン検証
```

---

## 3. レビュー漏れ再発防止の 6 大原則 (Guardrails)

過去のレビュー（`docs/code-review-b755b38-2026-09-27.md`）で検出された死角を繰り返さないため、以下の原則を厳格に適用してください。詳細は [docs/code-review-guidelines.md](docs/code-review-guidelines.md) を参照のこと。

### ① 条件付きコンパイル（`#[cfg(debug_assertions)]`）の禁止領域
* エラー判定、バリデーション、戻り値（`Result`）の制御に `#[cfg(debug_assertions)]` や `debug_assert!` を使用してはならない。
* UI で「検証済み」と表示する処理は、Release ビルドの WASM でも無条件に実行されなければならない。
* 正常系だけでなく、意図的に壊れたデータを注入するフォールトインジェクションテストを設けること。

### ② 複合操作シナリオと状態遷移ライフサイクルの検証
* 「手動回転」「解法探索」「手順再生（シーク）」「Undo/Redo」など、独立した機能が組み合わさったシーケンス（例: 操作 → 解法再生 → Undo）での整合性を必ず検証すること。
* `canUndo` / `canRedo` などの UI フラグは、「履歴配列の長さ」だけでなく「現在の盤面と巻き戻し先が実質的に異なるか」を正しく反映すること。

### ③ 処理順序と Fail-fast（リソース事前ガード）
* `file.text()` や `FileReader` などファイル全文をメモリ展開する前に、必ず `file.size` 等によるサイズ上限チェックを行うこと。
* 高負荷な処理（パース、探索、GPU アロケーション）の直前に、軽量なガード節を配置すること。

### ④ URL / パス正規化とスコープの同一性
* ブラウザアクセスにおいて、`/` と `/index.html`、およびサブディレクトリ表現（`/nested/` と `/nested/index.html`）で LocalStorage キーやキャッシュが分離しないよう正規化すること。

### ⑤ 副作用・リソース蓄積・リークの検査
* Service Worker の Cache API において、クエリ付きリクエスト（`?state=...` 等）を無加工で保存キーにしてはならない（アクセスごとに HTML が重複蓄積する）。
* Three.js のジオメトリ・マテリアル・テクスチャは適切にキャッシュ共有するか、明示的に `dispose()` すること。

### ⑥ テスト支援スクリプトの成果物鮮度保証
* テスト・検証用スクリプト（`launch-offline.js` 等）で、ファイルの存在チェックだけでビルドをスキップしてはならない。ソース更新日時との比較や `--fresh` オプションで最新状態を保証すること。

---

## 4. カバレッジとテスト設計の規約

* **カバレッジ数値至上主義の禁止**: 分岐網羅率（C0/C1）を上げるためだけに、不自然なモック引数（例: `validateAndParseCubeJson("{}", 70000)`）を直接渡して満足してはならない。
* **UI 呼び出し順序の結合テスト**: 実際のイベントハンドラやライフサイクルから実行される順序を再現した E2E / 結合テストを重視すること。
