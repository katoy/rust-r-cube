# Rust Rubik's Cube Project (Rust ルービックキューブ プロジェクト)

Rustで実装された、高性能ソルバー搭載の2x2および3x3ルービックキューブプロジェクトです。ネイティブデスクトップアプリと、WebAssembly (WASM) ベースのWebアプリの両方の形態で提供されています。

[![2x2 Web Demo](https://img.shields.io/badge/2x2_demo-live-success)](https://katoy.github.io/rust-r-cube/)
[![3x3 Web Demo](https://img.shields.io/badge/3x3_demo-live-success)](https://katoy.github.io/rust-r-cube/3x3/)
[![CI](https://github.com/katoy/rust-r-cube/actions/workflows/build.yml/badge.svg)](#)
[![License](https://img.shields.io/badge/license-MIT-green)](#)

---

## プロジェクト構成

本リポジトリは、以下の主要なアプリケーションで構成されています。

### 1. [2x2 デスクトップ版](./2x2/)
ネイティブデスクトップ環境（Windows, macOS, Linux）で動作するGUIアプリケーションです。
- **特徴**: マルチスレッドによる並列化を最大限に活用した超高速探索（双方向BFSソルバー）。
- **技術詳細**: `eframe` (egui) を使用したモダンなUI。

### 2. [2x2-web Web版](./2x2-web/)
ブラウザ上で動作するWebAssemblyベースのアプリケーションです。
- **特徴**: インストール不要で、ブラウザから手軽に2x2ルービックキューブの操作と解決を体験できます。
- **技術詳細**: WebAssembly (WASM) にコンパイルされ、`trunk` を使用してビルドされます。

### 3. [3x3-web (デスクトップ / Web版)](./3x3-web/)
ブラウザ上（WebAssembly）およびデスクトップ環境で動作する3x3ルービックキューブのアプリケーションです。
- **特徴**: Kociembaの2段階アルゴリズム（Two-Phase Algorithm）を搭載し、どのような状態からでも瞬時（数ミリ秒〜200ミリ秒）に20手前後の解法を提示。さらに、スーパーキューブ（センター方位）の解決にも完全対応。
- **技術詳細**: `eframe` (egui) によるGUI、WASM/`trunk` によるWebサポート。

---

## 特徴 (共通および各アルゴリズム)

- 🎮 **インタラクティブなUI**: 3D/2Dビューによる直感的なキューブ操作と、滑らかなイージングを伴う回転アニメーション。
- 📸 **6面スキャン入力**: 実物のキューブの状態を画面上のカラーパレットから視覚的に入力・再現。
- 💾 **状態の保存・読込**: 現在の状態をテキストファイルとして保存し、後で物理的な向きを含めて復元可能。
- 🔄 **物理的な整合性保証**: コーナーパズルの物理法則に基づく厳密なパリティチェックおよび向きの自動復元。
- ⚡ **高性能ソルバー**:
  - **2x2**: 双方向BFS（Breadth-First Search）を採用。キューサイズに応じた探索方向の動的切り替えや、Rayon/シングルスレッドのハイブリッド並列展開により、最短解（最大11手）を瞬時に探索。
  - **3x3**: Kociembaの2段階アルゴリズム（Two-Phase Algorithm）とIDA\*探索を採用。X2全体回転対称性（Symmetry-reduction）の導入により枝刈りテーブルを半分近く（48.6%）に圧縮し、メモリ効率を高めつつ高速に動作。
- 🎯 **スーパーキューブ（センター方位）対応**: ステッカーの向き（矢印マーク）を考慮し、他のパーツの配置を崩さない「真に色保存的」なアルゴリズムでセンターパーツの向き（90度/180度）も正確に解決（3x3にて対応）。

---

## クイックスタート

各プロジェクトのディレクトリに移動して、以下のコマンドで実行できます。

### 2x2 デスクトップ版の実行
```bash
cd 2x2
cargo run --release
```

### 2x2 Web版の起動 (開発サーバー)
```bash
cd 2x2-web
trunk serve --open
```

### 3x3 デスクトップ版の実行
```bash
cd 3x3-web
cargo run --release --bin rubiks-cube-3x3
```

### 3x3 Web版の起動 (開発サーバー)
```bash
cd 3x3-web
trunk serve --open
```
※ Web版の起動には `trunk` のインストールが必要です: `cargo install trunk`

---

## 技術スタック

- **Language**: Rust
- **GUI Framework**: [egui](https://github.com/emilk/egui) / [eframe](https://github.com/emilk/egui/tree/master/crates/eframe)
- **Parallel Processing**: [Rayon](https://github.com/rayon-rs/rayon)
- **Math**: [glam](https://github.com/bitshifter/glam-rs)
- **Hashing**: [rustc-hash](https://github.com/rust-lang/rustc-hash) (FxHashによる高速なハッシュマップ操作)
- **WASM Support**: `wasm-bindgen`, `js-sys`, `web-sys`, `trunk`

---

## 開発とテスト

GitHub Actions により、すべてのプッシュとプルリクエストに対して自動的に以下のチェックが実行されます：
- `cargo test`: 全テストの実行
- `cargo clippy`: 静的解析
- `cargo fmt`: フォーマットチェック
- `cargo llvm-cov`: テストカバレッジの計測

### テストカバレッジ (Core Logic)
- **2x2 Core**: 実質 **100.00%** カバレッジ達成 ✨
- **3x3 Core**: **99.59%** カバレッジ達成 🎉

### CI の仕組み
- `ubuntu-latest`, `macos-latest`, `windows-latest` のマルチプラットフォームでテストを並列実行。
- ビルド高速化のため `sccache` を活用し、キャッシュが存在しない場合は通常のビルドに自動フォールバック。

詳細な開発手順やテスト方法については、各プロジェクト（[2x2](./2x2/README.md), [2x2-web](./2x2-web/README.md), [3x3-web](./3x3-web/README.md)）の `README.md` を参照してください。

---

## ライセンス

[MIT License](./LICENSE)
