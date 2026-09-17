# 全体コードレビューレポート (Code Review Report)

**実施日**: 2026-09-17  
**対象リポジトリ**: `rust-r-cube/3x3-web-2`  
**主要技術スタック**: Rust (WebAssembly), TypeScript, Three.js, Vite, Playwright

---

## 1. 総合評価 (Executive Summary)

本プロジェクトは、Rustによる超高速なキューブソルバー（Kociembaの二段階探索法＋Supercubeセンター向き拡張）と、ブラウザネイティブなWebフロントエンド（TypeScript + Three.js + Web Workers）をシームレスに結合した高品質なアプリケーションです。

群論的パリティ検証やプルーニングテーブル事前生成など、数学的・計算機科学的に高度な設計が行われており、Rust単体テスト82件およびPlaywright E2Eテストによる強固なテスト基盤が整備されています。

一方で、Three.jsにおけるオブジェクト解放漏れ（GPUメモリリーク）や、カメラ認識モジュールにおける局所的な不整合・コード肥大化などの課題が確認されました。

### 五軸評価サマリー

| 評価軸 | 判定 | 概要 |
|:---|:---:|:---|
| **1. 正確性 (Correctness)** | **良 (Good)** | コアのソルバー・幾何回転アルゴリズムは非常に堅牢。カメラ入力のフォールバック時における面定義順序に一部不整合あり。 |
| **2. パフォーマンス (Performance)** | **要注意 (Needs Attention)** | Rust側の探索性能および事前テーブル埋め込みは秀逸。Three.jsの矢印描画でジオメトリ・マテリアルの破棄漏れによるGPUメモリリークが存在。 |
| **3. アーキテクチャ (Architecture)** | **良 (Good)** | Web Worker分離やWASM連携の設計は明快。`camera.ts` や `main.ts` への責務集中を分割する余地あり。 |
| **4. 可読性・単純性 (Readability)** | **良 (Good)** | 命名規則や日本語のガイド・解説コメントが充実。一部Prettierによるインデント不整合が残存。 |
| **5. セキュリティ (Security)** | **優 (Excellent)** | サーバーレス・完全クライアント完結型。ファイルサイズ制限、文字数制限、局面パリティ検証が徹底されている。 |

---

## 2. 課題と改善提案 (Findings & Recommendations)

### 🚨 Finding 1: [重要/パフォーマンス] Three.js の矢印描画による GPU メモリリーク
- **該当箇所**: `web/scene.ts`（260〜358行目付近）
- **現象**:
  キューブの回転操作や手順再生に伴い `show()` → `updateArrows()` が高頻度で呼び出されます。
  `this.arrowGroup.clear()` によって子オブジェクトの参照は外れるものの、Three.js の仕様上 **`ShapeGeometry` や `MeshBasicMaterial` は自動破棄されず GPU メモリに残存** します。
  全54セル × 2（カラー矢印 + 暗色アウトライン）= 108個のメッシュ・ジオメトリ・マテリアルが毎ステップ新規生成されるため、20手の解法を再生するだけで2,000個以上の Three.js リソースが解放されずに蓄積します。
- **改善案**:
  1. 静的形状である `arrowGeometry` と `outlineGeometry` をクラス単位（またはモジュール単位）でキャッシュして全矢印でインスタンスを共有する。
  2. マテリアルを再利用するか、または生成時に古いマテリアルを明示的に `dispose()` する。

```typescript
// 改善例 (web/scene.ts)
// ジオメトリを1度だけ作成して再利用
private static arrowGeomCache: THREE.ShapeGeometry | null = null;
private static outlineGeomCache: THREE.ShapeGeometry | null = null;

private getSharedGeometries() {
  if (!CubeScene.arrowGeomCache) {
    // arrowShape の作成
    CubeScene.arrowGeomCache = new THREE.ShapeGeometry(arrowShape);
    CubeScene.outlineGeomCache = new THREE.ShapeGeometry(outlineShape);
  }
  return {
    arrowGeom: CubeScene.arrowGeomCache,
    outlineGeom: CubeScene.outlineGeomCache,
  };
}
```

---

### ⚠️ Finding 2: [要修正/正確性] カメラ入力におけるクアッド指定順序の不整合
- **該当箇所**: `web/camera.ts`（543〜551行目、および 782〜792行目）
- **現象**:
  - `capture()` 内の `rawQuads`（画像A）:
    - 0: `defaultFace: "U"` (`[p1, p2, center, p6]`)
    - 1: `defaultFace: "F"` (`[p6, center, p4, p5]`)
    - 2: `defaultFace: "R"` (`[center, p2, p3, p4]`)
  - 一方、`updateDetectedLabels()` 内の `quads`（画像A）:
    - 0: `[p1, p2, center, p6]` (U面)
    - 1: `[center, p2, p3, p4]` (R面)
    - 2: `[p6, center, p4, p5]` (F面)
  - `keys` は `["U", "R", "F"]` と定義されているため、`updateDetectedLabels()` では 1番目が R、2番目が F ですが、`capture()` では 1番目が F、2番目が R になっています。
- **影響**:
  画像認識でセンター色が判別できずフォールバック処理が動いた場合、F面とR面の色データの割り当てが逆転するリスクがあります。
- **改善案**:
  `capture()` 側の `rawQuads` の順序を、U → R → F に統一してください。

---

### ⚠️ Finding 3: [軽微/スタイル] Prettier フォーマットチェックの不整合
- **該当箇所**: `web/camera.ts`（900〜924行目付近）
- **現象**:
  `npm run format:check`（`prettier --check`）を実行すると、`camera.ts` で不自然な深いインデントが検出され、CIチェックが失敗します。
- **改善案**:
  `npm run format` を実行してフォーマットを自動修正してください。

---

### 💡 Finding 4: [設計/保守性] `camera.ts` と `main.ts` の責務肥大化
- **該当箇所**: 
  - `web/camera.ts` (1,038行)
  - `web/main.ts` (737行)
- **現状**:
  - `camera.ts` に幾何演算（直線交点、中心計算、6角形輪郭検出）、Canvasドラッグハンドル描画、UI制御、ファイル入出力が1つのクラスに集中しています。
  - `main.ts` のトップレベルに約20個の状態変数（`state`, `centerRotations`, `history`, `future`, `solving`, `inMotion` 等）が散在し、状態遷移ロジックが各イベントハンドラに分散しています。
- **改善案**:
  1. 幾何演算ロジック（`intersectLines`, `computeCenter`, `detectCubeOutline`）を `web/camera-geometry.ts` へ分離。
  2. `main.ts` の状態変数を `CubeAppState` 等のコンテキストオブジェクトに集約し、状態更新処理の一元化を検討。

---

## 3. 優れた実装点 (Strengths)

1. **ゼロランタイムコストのプルーニングテーブル**
   - `build.rs` によりコンパイル時に約数MBのプルーニングテーブルをバイナリ（`tables.bin`）として生成し、WASMバイナリに直接埋め込み。
   - ブラウザロード時のテーブル生成待ち時間がゼロ。
2. **高速な直接探索（Direct Solve）ハイブリッド機構**
   - `src/search.rs` で深さ1〜5手までの直接探索を先行して実施。5手以下の簡単な局面や完成に近い状態であれば、数ミリ秒で即座に最短解を出力。
3. **Supercube（センター向き考慮）の群論的プルーニング**
   - Phase 2 で R, F, L, B 面が180°回転しかできない特性を利用し、奇数回転が残っている状態を即座に枝刈り（`min_phase2_center_moves`）。
4. **堅牢な Web Worker / 非同期アーキテクチャ**
   - `SolverClient` による探索中断（キャンセル時の安全な `worker.terminate()` と再生成）が実装されており、UIスレッドを一切ブロックしない。
5. **テストの厚みと自動検証**
   - Rust側のユニットテスト82件（可逆性、群論パリティ、対称性、既知局面テスト）がすべてパス。
   - Playwright によるE2Eおよび各モジュールのユニットテストが整備されている。

---

## 4. 推奨アクションプラン

- [x] **Step 1: フォーマット修正**
  - `npm run format` および `cargo fmt` を実行し、コードベース全体のフォーマット不整合を解消。
- [x] **Step 2: カメラクアッド順序の統一**
  - `web/camera.ts` の `capture()` 内のクアッド定義順序を `[U, R, F]` に修正し、プレビュー認識との不整合を解消。
- [x] **Step 3: Three.js 矢印のメモリリーク解消**
  - `web/scene.ts` において、`arrowGeometry` および `outlineGeometry` を事前生成・キャッシュ化し、マテリアルとメッシュを再利用することで毎ターンのGPUメモリリークを根絶。
- [x] **Step 4: カメラ幾何計算モジュールの分離（リファクタリング）**
  - `web/camera-geometry.ts` を新設し、純粋幾何関数群（`intersectLines`, `computeCenter`, `detectCubeOutline`）を分離。既存コードおよびテストとの互換性を完全に維持。
