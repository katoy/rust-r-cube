# Rust Rubik's Cube Project (Rust ルービックキューブ プロジェクト)

Rustで実装された、高性能ソルバー搭載の2x2および3x3ルービックキューブプロジェクトです。ネイティブデスクトップアプリと、WebAssembly (WASM) ベースのWebアプリの両方の形態で提供されています。

[![2x2 Web Demo](https://img.shields.io/badge/2x2_demo-live-success)](https://katoy.github.io/rust-r-cube/)
[![3x3 Web Demo](https://img.shields.io/badge/3x3_demo-live-success)](https://katoy.github.io/rust-r-cube/3x3/)
[![Cube Studio (3x3 v2)](https://img.shields.io/badge/Cube_Studio_(3x3_v2)-live-success)](https://katoy.github.io/rust-r-cube/3x3-v2/)
[![Build & Test](https://github.com/katoy/rust-r-cube/actions/workflows/build.yml/badge.svg)](https://github.com/katoy/rust-r-cube/actions/workflows/build.yml)
[![Lint & Format](https://github.com/katoy/rust-r-cube/actions/workflows/lint.yml/badge.svg)](https://github.com/katoy/rust-r-cube/actions/workflows/lint.yml)
[![3x3-web-2 CI](https://github.com/katoy/rust-r-cube/actions/workflows/3x3-web-2.yml/badge.svg)](https://github.com/katoy/rust-r-cube/actions/workflows/3x3-web-2.yml)
[![Deploy to GitHub Pages](https://github.com/katoy/rust-r-cube/actions/workflows/deploy-pages.yml/badge.svg)](https://github.com/katoy/rust-r-cube/actions/workflows/deploy-pages.yml)
[![License](https://img.shields.io/badge/license-MIT-green)](./3x3-web-2/LICENSE)

---

## プロジェクト構成

本リポジトリは、以下の主要なアプリケーションで構成されています。

### 1. [2x2 デスクトップ版](./2x2/)
ネイティブデスクトップ環境（Windows, macOS, Linux）で動作するGUIアプリケーションです。
- **特徴**: 4グループ統合 Pruning Table（枝刈りテーブル）と IDA*（反復深化A*）探索による超高速な最短解探索（最大14手対応）。
- **技術詳細**: `eframe` (egui) を使用したモダンなUI。

### 2. [2x2-web Web版](./2x2-web/)
ブラウザ上で動作するWebAssemblyベースのアプリケーションです。
- **特徴**: インストール不要で、ブラウザから手軽に2x2ルービックキューブの操作と解決を体験できます。
- **技術詳細**: WebAssembly (WASM) にコンパイルされ、`trunk` を使用してビルドされます。

### 3. [3x3-web (デスクトップ / Web版)](./3x3-web/)
ブラウザ上（WebAssembly）およびデスクトップ環境で動作する3x3ルービックキューブのアプリケーションです。
- **特徴**: Kociembaの2段階アルゴリズム（Two-Phase Algorithm）を搭載し、どのような状態からでも瞬時（数ミリ秒〜200ミリ秒）に20手前後の解法を提示。さらに、スーパーキューブ（センター方位）の解決にも完全対応。
- **技術詳細**: `eframe` (egui) によるGUI、WASM/`trunk` によるWebサポート。

### 4. [3x3-web-2 (Cube Studio - 次世代Web/PWA版)](./3x3-web-2/)
Three.js + TypeScript + Rust (WASM WebWorker) によるモダンでリッチな 3×3×3 ＆ 2×2×2 ルービックキューブ アプリケーションです。
- **特徴**:
  - **3×3×3 ＆ 2×2×2 マルチキューブ対応**: ヘッダーのボタンから 3×3×3 と 2×2×2 を瞬時に切り替え可能。
  - **多彩なソルバーエンジン**:
    - **【3×3】** Kociemba 2段階探索（デフォルト）、CFOP (Fridrich)、Thistlethwaite 4段階群縮小、Korf (IDA* 最短探索)。
    - **【2×2】** LBL法（完全1層 $\to$ OLL $\to$ PLL）、Ortega法（下1面 $\to$ OLL $\to$ PBL）、Optimal 完全最短探索（神の数字11手以内保証）。
  - **全ソルバー一括比較＆ワンクリック切替再生**: 同一盤面に対して全アルゴリズム（3×3: 4種 / 2×2: 3種）の手数・計算時間・探索ノード数を一覧比較し、ワンクリックで再生手順を切り替え可能。
  - **超軽量 WASM & 高速ソルバー**: `CUBE0002` 形式の 4-bit パック直接枝刈りテーブル（ゼロコピー `Cow<'static, [u8]>`）と実行時 10 手 Phase 2 移動テーブルにより、WASM バイナリを gzip 約 914 KB（`dist/` 約 926 KB）・リニアメモリ約 4.9 MiB へ軽量化。3×3 Kociemba は深さ 0 からの 6 方向交互探索により色のみ全件 20 手以内保証（WASM 平均約 21 ms）・センター向き同時最適化平均約 203 ms、Thistlethwaite は座標ベース実装で < 0.1 ms、2×2 Optimal（`<U, R, F>` 3 面限定・11 手 HTM 以内）＆ Ortega（5,670 状態第 1 面距離表）も < 0.1 ms で解決。
  - Web Worker による完全非同期の探索エンジン実行（UIフリーズなし）。
  - Three.js による高品質3Dアニメーション、視点プリセット切替（標準・前・上・右）、WebGL非対応時の2D展開図自動フォールバック。
  - PWA による完全なオフライン起動＆ホーム画面インストール対応（初回訪問時に自動プリキャッシュ。スマホでの詳しい利用手順は [3x3-web-2 README](./3x3-web-2/README.md#-スマートフォンでのオフライン利用-pwa) を参照）。
  - 解法フェーズ分解（Phase 1 / Phase 2、2×2 OLL/PLL/PBL）およびトリガームーブ（Sexy Move、Sune 等）の自動注釈。
  - Web Audio API による効果音（回転音、達成ファンファーレ音）＆ キーボード入力時の視覚的キーフィードバック。
  - キューブ状態・キューブ種別・選択ソルバー・センター方位の URL 共有リンク機能（ワンクリックコピー）。
  - WCAG 2.1 AA 準拠のアクセシビリティ対応 & 自動監査テスト（axe-core）。
  - セキュリティ & 品質ガードレール: 本番ビルドへの厳格な `Content-Security-Policy` 自動注入、`no-referrer` 設定、JSON/画像ヘッダ事前検証、Service Worker のスコープ内 HTTP 200 限定キャッシュ、静的ガードレール検査（`npm run check:guardrails`）。
  - WebRTC ライブカメラ映像 & 2方向写真からのカラー一括スキャン＆補正パレット。
  - 初心者向けガイダンス（回転記号早見表・ツールチップ）、自動スクロール付き再生コントロール。
- **技術詳細**: Vite 8, TypeScript 7, Three.js r186, Rust 1.100.0+ (wasm-bindgen), Web Audio API, Playwright E2E。

---

## 特徴 (共通および各アルゴリズム)

- 🎮 **インタラクティブなUI**: 3D/2Dビューによる直感的なキューブ操作と、滑らかなイージングを伴う回転アニメーション。
- 📸 **6面スキャン入力**: 実物のキューブの状態を画面上のカラーパレットから視覚的に入力・再現。
- 💾 **状態の保存・読込**: 現在の状態をテキストファイルとして保存し、後で物理的な向きを含めて復元可能。
- 🔄 **物理的な整合性保証**: コーナーパズルの物理法則に基づく厳密なパリティチェックおよび向きの自動復元。
- ⚡ **高性能ソルバー**:
  - **2x2**: 4グループ統合 Pruning Table と IDA*（反復深化A*）探索を採用。メモリ消費を極小化しつつ、最難関状態からでも最短解（HTM基準で最大11手、QTM基準で最大14手）をミリ秒未満で瞬時に探索。
  - **3x3**: Kociembaの2段階アルゴリズム（Two-Phase Algorithm）とIDA\*探索を採用。`3x3-web` では X2全体回転対称性（Symmetry-reduction）の導入により枝刈りテーブルを半分近く（48.6%）に圧縮。`3x3-web-2` では `CUBE0002` 4-bit パック直接枝刈りテーブルのゼロコピー参照に加え、深さ 0 からの 6 方向交互探索により色のみ全件 20 手以内の解法を保証。
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

### 3x3-web-2 (Cube Studio) Web版の起動 (開発サーバー)
```bash
cd 3x3-web-2
npm install
npm run wasm
npm run dev
# または ./start.sh（WASM 未ビルド時は自動でビルド）
```

---

## デプロイ (GitHub Pages)

本リポジトリの `main` ブランチにプッシュされると、GitHub Actions ワークフロー（[`.github/workflows/deploy-pages.yml`](.github/workflows/deploy-pages.yml)）が自動実行され、GitHub Pages に一括デプロイされます。

- **ルート (`/`)**: 2x2 Web版
- **`/3x3/`**: 3x3 Web版 (Trunk / egui)
- **`/3x3-v2/`**: 3x3-web-2 (Cube Studio / Three.js + Vite)

各Webアプリは静的ファイル（HTML, JS, CSS, WASM）として生成されるため、GitHub Pages のほか、Cloudflare Pages、Vercel、Netlify 等の任意の静的ホスティング環境にもデプロイ可能です。

---

## 技術スタック

- **Core / Solver**: Rust (`rustc 1.100.0+`), `wasm-bindgen`, `js-sys`, `web-sys`
- **Desktop / egui Web (`2x2`, `2x2-web`, `3x3-web`)**: [egui](https://github.com/emilk/egui) / [eframe](https://github.com/emilk/egui/tree/master/crates/eframe), [glam](https://github.com/bitshifter/glam-rs), [rustc-hash](https://github.com/rust-lang/rustc-hash), [Rayon](https://github.com/rayon-rs/rayon)（`2x2-web` / `3x3-web`）, `trunk`
- **Cube Studio (`3x3-web-2`)**: TypeScript 7, Vite 8, Three.js r186, Web Worker, Web Audio API, Service Worker (PWA), Playwright (`@axe-core/playwright`)

---

## 開発とテスト

GitHub Actions により、プッシュおよびプルリクエストに対して以下のワークフローが自動実行されます：
- **Lint & Format** ([`lint.yml`](.github/workflows/lint.yml)): 全 4 クレートの `cargo fmt --all -- --check` / `cargo clippy --all-targets --all-features -- -D warnings`（`ubuntu-latest` / `macos-latest`）
- **Build & Test** ([`build.yml`](.github/workflows/build.yml)): 全 4 クレートのビルド＆テスト（`ubuntu-latest` / `macos-latest` / `windows-latest`）
- **3x3-web-2 CI** ([`3x3-web-2.yml`](.github/workflows/3x3-web-2.yml)): Rust 1.100.0 MSRV 互換性検証（native / `wasm32-unknown-unknown`）、Rust fmt/clippy/test、静的ガードレール検証（`npm run check:guardrails`）、Prettier フォーマットチェック、WASM & Web 本番ビルド、Playwright E2E・a11y（axe-core）テスト
- **Deploy to GitHub Pages** ([`deploy-pages.yml`](.github/workflows/deploy-pages.yml)): `3x3-web-2` テスト通過後、`2x2-web` / `3x3-web` / `3x3-web-2` をビルドし `main` ブランチから自動デプロイ

### ローカルでの一括検証
コミット前にリポジトリ全体の整合性（フォーマット、静的解析、ビルド、テスト）をまとめてローカルで検証するためのスクリプトを用意しています。
```bash
./scripts/ci-local.sh
```

### クリーンアップ（不要ファイルの削除）
各プロジェクトのビルド成果物（`target/`）や、Webビルド（`dist/`）、テスト等の実行時に生成された一時ファイル（`.DS_Store` や一部のカバレッジ関連データなど）をまとめて削除するためのスクリプトを用意しています。
```bash
./scripts/clean.sh
```

### テストカバレッジ / 品質指標
- **2x2 Core (`2x2` / `2x2-web`)**: 実質 **100.00%** カバレッジ達成（テスト 148 件 / 100 件）
- **3x3 Core (`3x3-web`)**: **99.59%** カバレッジ達成（テスト 216 件）
- **Cube Studio (`3x3-web-2`)**: Rust 単体テスト **178 passed**（+ 1 ignored）・Playwright E2E **343 passed**（+ 4 skipped）・Web 計測対象全 **26 モジュール** CDP 行カバレッジ基準達成（うち 21 モジュール 100.00%、`main.ts` 92.41%）

### CI の仕組み
- `ubuntu-latest`, `macos-latest`, `windows-latest` のマルチプラットフォームでテストを並列実行。
- ビルド高速化のため `sccache` を活用し、キャッシュが存在しない場合は通常のビルドに自動フォールバック。

詳細な開発手順やテスト方法については、各プロジェクト（[2x2](./2x2/README.md), [2x2-web](./2x2-web/README.md), [3x3-web](./3x3-web/README.md), [3x3-web-2](./3x3-web-2/README.md)）の `README.md` を参照してください。

---

## ライセンス

[MIT License](./3x3-web-2/LICENSE)
