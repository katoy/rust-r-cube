# Rust 2x2 Rubik's Cube Project

Rustで実装された、高性能ソルバー搭載の2x2ルービックキューブプロジェクトです。デスクトップアプリとWebアプリの2つの形態で提供されています。

![CI](https://github.com/katoy/rust-r-cube/actions/workflows/build.yml/badge.svg)
![Core Coverage](https://img.shields.io/badge/core_coverage-100%25-brightgreen)
![Rust Version](https://img.shields.io/badge/rust-1.92%2B-blue)
![License](https://img.shields.io/badge/license-MIT-green)

## プロジェクト構成

本リポジトリは、以下の2つの主要なアプリケーションで構成されています。

### 1. [2x2 デスクトップ版](./2x2/)
ネイティブデスクトップ環境（Windows, macOS, Linux）で動作するGUIアプリケーションです。
- **特徴**: マルチスレッドによる並列化を最大限に活用した超高速探索。
- **技術詳細**: `eframe` (egui) を使用したモダンなUI。

### 2. [2x2-web Web版](./2x2-web/)
ブラウザ上で動作するWebAssemblyベースのアプリケーションです。
- **特徴**: インストール不要で、ブラウザから手軽にルービックキューブの操作と解決を体験できます。
- **技術詳細**: WebAssembly (WASM) にコンパイルされ、`trunk` を使用してビルドされます。

## 特徴 (共通)

- 🎮 **インタラクティブなUI**: 3Dビューによる直感的なキューブ操作。
- 🚀 **高性能ソルバー**: 双方向BFS（Breadth-First Search）を採用し、どのような状態からでも最短解（最大11手）を瞬時に探索。
- 📊 **リアルタイム表示**: 探索中の進捗状況をプログレスバーで表示。
- 📸 **6面スキャン入力**: 実物のキューブの状態を視覚的に入力・再現。
- 💾 **状態の保存・読込**: 現在の状態をテキストファイルとして保存し、後で復元可能。
- 🔄 **向きの可視化**: ステッカーの向き（矢印マーク）まで考慮した管理機能。

## クイックスタート

各ディレクトリに移動して、以下のコマンドで実行できます。

### デスクトップ版の実行
```bash
cd 2x2
cargo run --release
```

### Web版の起動 (開発サーバー)
```bash
cd 2x2-web
trunk serve --open
```
※ `trunk` のインストールが必要です: `cargo install trunk`

## 技術スタック

- **Language**: Rust
- **GUI Framework**: [egui](https://github.com/emilk/egui) / [eframe](https://github.com/emilk/egui/tree/master/crates/eframe)
- **Parallel Processing**: [Rayon](https://github.com/rayon-rs/rayon)
- **Math**: [glam](https://github.com/bitshifter/glam-rs)
- **WASM Support**: `wasm-bindgen`, `js-sys`, `web-sys`

## 開発とテスト

GitHub Actions により、すべてのプッシュとプルリクエストに対して自動的に以下のチェックが実行されます：
- `cargo test`: 全テストの実行
- `cargo clippy`: 静的解析
- `cargo fmt`: フォーマットチェック
- `cargo llvm-cov`: テストカバレッジの計測

CI の仕組み（簡単）
- マトリックスで複数プラットフォーム上で実行します: `ubuntu-latest`, `macos-latest`, `windows-latest`。
- 並列で各プラットフォームのビルド/テストを行い、プラットフォーム固有の問題を早期検出します。
- ビルド高速化のため `sccache` を使用しますが、ランナーに sccache が無ければ通常のビルドにフォールバックするようワークフローを構成しています。
- RUSTC_WRAPPER はランナー環境に依存するため、ワークフロー内で `CARGO_HOME` / `PATH` の設定と存在チェックを行っています。

詳細な開発手順やテスト方法については、各プロジェクトの `README.md` を参照してください。

## ライセンス

[MIT License](./LICENSE)
