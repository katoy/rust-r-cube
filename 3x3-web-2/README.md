# Cube Studio (3x3-web-2)

ブラウザ上で動作する、高速・高機能な 3×3×3 ルービックキューブ・ソルバー＆シミュレータです。  
Rust で実装された Kociemba 2段階探索エンジンを WebAssembly (WASM) にコンパイルし、Web Worker 上でバックグラウンド実行します。フロントエンドは TypeScript + Vite + Three.js で構築され、PWA による完全なオフライン動作に対応しています。

[![Live Demo](https://img.shields.io/badge/Cube_Studio-Demo-success)](https://katoy.github.io/rust-r-cube/3x3-v2/)
[![CI](https://github.com/katoy/rust-r-cube/actions/workflows/3x3-web-2.yml/badge.svg)](#)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.x-blue)](https://www.typescriptlang.org/)
[![Three.js](https://img.shields.io/badge/Three.js-r186-black)](https://threejs.org/)
[![Rust](https://img.shields.io/badge/Rust-1.80+-orange)](https://www.rust-lang.org/)

---

## 主な特徴

- 🧠 **4つの多彩な解法アルゴリズムを自由に選択可能**:
  - **Kociemba 2段階探索** (デフォルト): 速度と手数のバランスが最良（平均20手前後、10〜30ms）。
  - **CFOP / LBL**: 人間のスピードキューブ解法（Cross $\to$ F2L $\to$ OLL $\to$ PLL）。手順の意図が直感的に理解可能。
  - **Thistlethwaite**: 群論に基づく 4 段階部分群縮小法（$G_0 \to G_1 \to G_2 \to G_3 \to G_4$）。
  - **Korf (IDA*)**: 理論上の最短手（神の数字20手以内）を求める反復深化A*探索（タイムアウト時はKociemba準最適解へフォールバック）。
- ⚡ **Kociemba 2段階探索エンジン**:
  - どのような混ざった状態からでも、通常 20手前後（平均 10〜30ms）で高速に解法を導出。
  - **スーパーキューブ（センターパーツの向き解決）対応**: 各面センターの 90°/180° の回転も保持・解消する高精度モードを搭載。
- 🧵 **Web Worker による非同期探索**:
  - 重い探索処理をバックグラウンド Worker で実行するため、3D アニメーションやユーザー操作が一切カクつきません。
- 🧊 **Three.js による 3D レンダリング & 2D フォールバック**:
  - スムーズな回転アニメーション、視点操作、ズーム。
  - **視点プリセット**: 標準（斜め見下ろし / Iso）、正面（Front）、上面（Top）、右面（Right）をワンクリックで切り替え可能。
  - WebGL が利用できない環境では 2D 展開図モードへ自動的にフォールバック。
- 🧩 **解法フェーズ分解 & トリガームーブ注釈**:
  - Kociemba 解法を Phase 1（エッジ・コーナーの方向整理、G1 群へ遷移）と Phase 2（完成への遷移）にステップ分割してバッジ表示。
  - Sexy Move (`R U R' U'`) や Sune などの有名トリガームーブパターンを自動検出してアノテーションタグを表示。
- 🔊 **Web Audio API による効果音 & キーボード入力フィードバック**:
  - キューブ回転時のリアルなクリック音、リセット音、解法完了時の通知音を合成出力。
  - キーボード操作時に対応するボタンが瞬時に光る視覚的キーフィードバック。
- 📷 **カメラ入力（WebRTC リアルタイム映像 & 2方向静止画）**:
  - WebRTC (`getUserMedia`) によるカメラ映像プレビューとリアルタイムフレームキャプチャ。
  - 2方向からの斜め写真（上面・右面・前面 / 下面・左面・背面）から、外周 6 角を指定して 6 面の色を一度に自動認識・透視射影変換補正。
  - 認識結果の確認・微調整ができるインタラクティブ修正パレット。
- 🔗 **URL による状態・解法・センター向きの共有**:
  - 現在のキューブ状態、スクランブル手順、センター方位を URL パラメータ（`?state=` / `?alg=` / `?centers=`）として共有可能。
  - 「共有リンク」ボタンからワンクリックで URL をクリップボードにコピー。
- 💾 **設定と状態の自動永続化**:
  - テーマ、再生速度、各種表示設定を `localStorage` に自動保存。
  - プライベートブラウズやストレージ無効環境でも動作する安全なメモリフォールバック。
- ♿ **アクセシビリティ（a11y）対応 & 自動監査**:
  - WCAG 2.1 AA レベルのコントラスト比と WAI-ARIA 仕様に準拠（タブリスト、ラジオグループ、キーボードナビゲーション）。
  - `@axe-core/playwright` による a11y 自動監査テストを CI に組み込み、高品質な操作性を維持。
- 📱 **PWA (Progressive Web App) 対応 & 自動プリキャッシュ**:
  - Service Worker と Web App Manifest を備え、スマートフォンやデスクトップにアプリとしてインストール可能。
  - ビルド時に生成される動的プリキャッシュ一覧（`scripts/generate-sw-precache.js`）により、初回訪問後から完全オフラインで利用可能。
  - Service Worker の更新検知と自動リロード機能。
- 🎓 **初心者向けガイダンス**:
  - 回転記号（U, R, F, D, L, B, ′, 2）の日本語・英語対応ツールチップ。
  - ヘルプダイアログ内の記号早見表。
- ⏩ **充実した再生コントロール**:
  - 自動再生、1手送り/戻し、再生速度変更（0.5×, 1×, 2×）。
  - 最初/最後へのジャンプボタン、キーボードショートカット（Home, End, Space, ←, →）。
  - 再生位置に連動した手順リストの自動スムーズスクロール。
- 🚀 **バンドルサイズと読み込み最適化**:
  - Three.js の独立ベンダーチャンク化。
  - カラーエディタやカメラモジュールを初回起動時の動的インポート（Dynamic Import）にすることで、初期 JS を軽量化。

---

## 解法アルゴリズムの比較と特徴

Cube Studio では、目的に応じて 4 種類の異なる解法アルゴリズムを選択できます。画面の解法オプションからいつでもワンクリックで切り替えることが可能です。

### 1. Kociemba 2段階探索アルゴリズム (Two-Phase) [推奨・デフォルト]
- **アプローチ**: 群論に基づき、状態空間を Phase 1（エッジ・コーナーの向きを整え、$G_1 = \langle U, D, R2, L2, F2, B2 \rangle$ 群へ遷移）と Phase 2（完成状態 $G_2$ への遷移）の 2 段階に分割。
- **特徴**: 手数と計算速度のバランスが最も優れており、どのような状態からでも **通常 20手前後** の非常に短い手順を **10〜30ms** の瞬時に導出します。
- **スーパーキューブ対応**: センターパーツの向き（90°/180°）も同時に解消する高精度モードに対応しています。

### 2. CFOP / LBL (Layer-By-Layer: 階層解法)
- **アプローチ**: 人間のスピードキューバー（競技者）が広く用いる標準解法（Fridrich 法）。
- **特徴**: 下層十字（**Cross**） $\to$ 第1・第2層同時（**F2L**） $\to$ 上層の向き（**OLL**） $\to$ 上層の位置（**PLL**）の 4 ステップで進みます。
- **メリット**: 人間が覚える定石マクロ（Sexy Move、Sune、T-perm 等）で構成されているため、各ステップの意図が直感的にわかりやすく、**人間の練習・学習・ステップ解説に最適**です。

### 3. Thistlethwaite アルゴリズム (4段階群縮小解法)
- **アプローチ**: Morwen Thistlethwaite（1981年）によって発表された、数学的群論に基づく古典的コンピュータ解法。
- **特徴**: 許容される回転操作を段階的に制限し、入れ子状の部分群へと状態を縮小させていきます。
  - **Phase 1** ($G_0 \to G_1$): エッジの向き（EO）を解決 ($F, B$ を半回転に制限)
  - **Phase 2** ($G_1 \to G_2$): コーナーの向き（CO）と E スライスを解決 ($L, R$ を半回転に制限)
  - **Phase 3** ($G_2 \to G_3$): コーナー・エッジの各オービット位置を解決 (全回転を半回転 $180^\circ$ に制限)
  - **Phase 4** ($G_3 \to G_4$): 半回転のみですべての位置を揃えて完成
- **メリット**: 回転操作の制限が段階的に厳しくなり、最終的にすべて $180^\circ$ 回転だけで揃っていく群論の美しさを体感できます。

### 4. Korf アルゴリズム (IDA* 最適解探索)
- **アプローチ**: Richard Korf（1997年）によって開発された、神の数字（God's Algorithm）を追求する反復深化A*（IDA*）探索。
- **特徴**: 許容的ヒューリスティック（Admissible Heuristic）を用いて、理論上の **厳密最短手数**（通常 15〜20手以内）を探索します。
- **フォールバック**: 状態が深くなると探索ノード数が爆発するため、タイムアウト予算（1〜3秒）を超過した場合は自動的に高品質な Kociemba 準最適解にフォールバックして確実に解を出力します。

---

### 📊 アルゴリズム比較表

| アルゴリズム | 基本アプローチ | 通常の平均手数<br>(色のみ) | **セルの向きを揃える手数**<br>*(スーパーキューブ)* | **Superflip 解法手数**<br>*(色のみ / センター考慮)* | 探索速度 | スーパーキューブ対応 | 主な用途・特徴 |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| **Kociemba (Two-Phase)** <br>*(デフォルト)* | 2段階部分群探索<br>($G_0 \to G_1 \to G_2$) | 約 20〜22手 | **約 22〜28手**<br>(平均 24〜26手) | **20〜30手**<br>*(色のみの探索範囲。センター補正で増加する場合あり)* | 超高速<br>(10〜30ms) | ✅ **対応**<br>(センター方位保持) | 速度・手数のバランスが最良。日常の解法導出に最適。 |
| **CFOP (LBL)** | 階層別定石法<br>(Cross $\to$ F2L $\to$ OLL $\to$ PLL) | 約 50〜70手 | 非対応<br>*(色のみ解決)* | 定石による<br>*(修正後のプリセットは未計測)* | 即時<br>(< 5ms) | ❌ | 人間向け標準解法。各段階の意図が明確で学習向け。 |
| **Thistlethwaite** | 4段階部分群縮小<br>($G_0 \to \cdots \to G_4$) | 約 30〜45手 | 非対応<br>*(色のみ解決)* | 段階探索による<br>*(修正後のプリセットは未計測)* | 高速<br>(5〜50ms) | ❌ | 群論の古典的名作。回転制約が狭まる過程を観察可能。 |
| **Korf (IDA*)** | 反復深化A* 最短解探索<br>(準最適解フォールバック付) | **最短**<br>(15〜20手) | 非対応<br>*(色のみ解決)* | **最短20手**<br>*(最短探索成功時。時間切れ時はフォールバック)* | 浅い手は即時<br>深い手は予算内 | ❌ | 理論上の最短手（神の数字）を検証したい場合に最適。 |

> [!TIP]
> **セルの向き（センターの向き / スーパーキューブ）について**:  
> 通常の無地ルービックキューブは各面の中央のセル（センターパーツ）の回転角（90°/180°）が見た目に現れませんが、イラスト・ロゴ入りキューブやスーパーキューブではセンターセルの向きも正確に揃える必要があります。  
> - **Kociemba ソルバー**: センターセルの回転群を追跡し、色だけでなく 6 面すべてのセンターの向きを元の向き（0°）に復元する解法を導出します（通常の色のみに比べて **+2〜6手程度** 増加して約 22〜28手）。  
> - **その他のソルバー**: エッジ・コーナーの色のみを対象としており、センターセルの向きの調整には非対応です。

#### 🔄 色のみ解決した後、セルの向きを揃えるのには何手が必要か？

外周（エッジ・コーナー）の色を完成させた状態から、外周の配置を一切崩さずに**後からセルの向き（センターパーツの向き）だけを揃える**場合、群論のパリティ制約により **12手〜最大40手以上** が追加で必要になります。

##### 1. ズレのパターン別・後付け必要手数 (HTM)
ルービックキューブの力学上、**「単独の 1 つのセンターだけを 90° 回す」ことは数学的に不可能**です（全センターの四半回転の総和は偶数でなければならない）。そのため、以下の定石手順を適用して解消します。

| ズレているセルの状態 | 必要な手数 (HTM) | 代表的な定石手順 | 備考 |
| :--- | :---: | :--- | :--- |
| **1つのセンターを 180° 回転** | **12手** | `(U R L U2 R' L') × 2` | 外周を崩さず 1 面のセンターのみを半回転させる最短手順。 |
| **隣接する 2 つのセンターを 90° 相殺**<br>(例: U面 +90° / F面 -90°) | **16〜28手**<br>*(本アプリ定石: 28手)* | T-perm とスライスマクロの合成 | 90° 回転は必ず「時計回りと反時計回り」のペアで解消。 |
| **対向する 2 つのセンターを 90° 相殺**<br>(例: U面 +90° / D面 -90°) | **約 28〜36手** | 中間面を経由して隣接ペア定石を 2 回適用・合成 | 独立した定石の合成により手数が長くなります。 |
| **複数面が複合してズレている場合**<br>(例: 2面が90° + 1面が180°) | **約 30〜50手以上** | 上記定石を順番に適用し、隣接手をキャンセル | 定石の単純継ぎ足しにより手数が大幅に膨らみます。 |

##### 2. 「後付け補正方式」vs「同時最適化方式 (Cube Studio)」
| アプローチ | 解決の流れ | 解法全体の総手数 | 特徴 |
| :--- | :--- | :---: | :--- |
| **後付け補正方式**<br>*(逐次解決)* | ① 通常通り色だけを解く（約20手）<br>② 後からセンター回転定石を適用（**+12〜40手以上**） | **約 32〜60手以上** | 手順の意図は把握しやすいが、総手数が大幅に長くなる。 |
| **同時最適化方式**<br>*(Cube Studio の Kociemba)* | 探索時に「色完成かつセンター累積回転が0」となる解を IDA* で直接探索 | **約 22〜28手**<br>*(増加わずか **+2〜6手**)* | 最短に近い手数を瞬時に導出。最も効率的。 |

> [!NOTE]
> **Superflip（スーパーフリップ）について**:  
> すべてのコーナーが揃ったまま、12 個の全エッジパーツだけが反転した状態で、ルービックキューブにおいて「最短でも 20 手を要する」ことが数学的に証明されている代表的な最難関配置（God's Number = 20 の証明例）です。
> - 20手は **色のみの理論上の最短手数**（HTM: 180°回転も1手）です。探索ソルバーが常に20手の解法を返すという意味ではありません。根拠は [God's Number is 20](https://www.cube20.org/) を参照してください。
> - プリセットは `U R2 F B R B2 R U2 L B2 R U' D' R2 F R' L B2 U2 F2` で生成します。センターの向きもこの手順から引き継ぎます。
> - **Kociemba / Korf**: 探索予算やフォールバックによって手数が変わります。センター向きを含める場合も20手未満にはなりません。
> - **CFOP / Thistlethwaite**: それぞれの段階的な解法で解きます。手数は各段階で選ばれる手順によって変わります。

---

## アーキテクチャ

```mermaid
flowchart TD
    subgraph UI ["ブラウザ メインスレッド (TypeScript / Three.js)"]
        View["UI / View (DOM / ARIA)"]
        Scene["3D Scene (Three.js)"]
        Store["CubeStore (状態・Undo/Redo・永続化)"]
        Sound["Sound (Web Audio API)"]
        Triggers["Triggers (Phase/パターンの解析)"]
        View --> Store
        Scene --> Store
        Store --> View
        Store --> Sound
        Store --> Triggers
    end

    subgraph Dynamic ["遅延ロードモジュール (Dynamic Import)"]
        Editor["ColorEditor (6面カラーエディタ)"]
        Camera["TwoViewCamera (WebRTC / 静止画 2方向認識)"]
        Store -.->|利用時にロード| Editor
        Store -.->|利用時にロード| Camera
    end

    subgraph Worker ["Web Worker (WASM)"]
        Client["SolverClient"]
        WasmWorker["solver.worker.ts"]
        RustWasm["cube_studio.wasm"]
        Solver["4 Engines: Kociemba / CFOP / Thistlethwaite / Korf<br>& Supercube Extension"]
        Client <-->|postMessage| WasmWorker
        WasmWorker --> RustWasm
        RustWasm --> Solver
    end

    Store <--> Client
```

---

## 開発と実行

### 前提環境

- **Rust**: 1.80 以上 (`wasm32-unknown-unknown` ターゲット)
- **wasm-pack**: 最新版 (`cargo install wasm-pack` または `npm install -g wasm-pack`)
- **Node.js**: v20 以上
- **npm**: v10 以上

### コマンド一覧

```bash
# 依存関係のインストール
npm install

# 開発サーバーの起動 (Vite: http://127.0.0.1:5173)
npm run dev

# ヘルパースクリプトでの起動（WASM自動チェック/再ビルド付き）
./start.sh
./start.sh --offline   # PWAキャッシュ構築後、オフライン状態（ネットワーク遮断）で起動
./start.sh --preview   # 本番ビルドを作成しローカルプレビュー起動
./start.sh --build     # WASM を再コンパイルしてから起動

# オフライン動作検証モードの直接起動 (npm)
npm run offline

# WASM ビルドのみ
npm run wasm

# 本番ビルド (WASM ビルド + 型チェック + Vite ビルド + SW プリキャッシュ生成)
npm run build

# 本番ビルド成果物 (dist/) のローカルプレビュー
npm run preview

# TypeScript 型チェック
npm run typecheck

# コードフォーマットチェック / 自動整形 (Prettier)
npm run format:check
npm run format

# Playwright E2E テストの実行
npm test

# カバレッジ計測付き E2E テスト
npm run test:coverage

# カメラ認識テスト（テスト画像自動生成 + テスト実行）
npm run test:camera

# Rust ソルバーの性能ベンチマーク
npm run benchmark

# Web アプリケーションの性能ベンチマーク
npm run benchmark:web

# 全体チェック (Rust fmt/clippy/test + Web format/typecheck/build/test)
npm run check
```

---

## テスト & 品質管理

本プロジェクトでは、信頼性と使いやすさを担保するために多角的なテストを実施しています。

- **Playwright E2E テスト**:
  - キューブの基本操作、回転アニメーション、Undo/Redo。
  - Kociemba 解法の導出とステップ再生。
  - WebRTC ライブカメラおよび静止画アップロードによる色認識（透視射影変換・パレット修正）。
  - URL パラメータによる状態復元（`?state=`, `?alg=`, `?centers=`）。
  - 設定の localStorage 永続化とストレージ無効化環境でのフォールバック。
  - PWA インストール、オフラインキャッシュ、Service Worker 更新検知。
- **アクセシビリティ (a11y) 監査**:
  - `@axe-core/playwright` を使用し、全主要画面で WCAG 2.1 AA レベルへの適合を自動検証。
- **Rust ユニットテスト & ベンチマーク**:
  - コーナー/エッジ座標変換の正確性。
  - 枝刈りテーブル（Pruning Table）生成およびキャッシュ整合性。
  - スーパーキューブのセンター回転解消アルゴリズム。

---

## デプロイ (GitHub Pages / 静的ホスティング)

### 1. GitHub Actions による自動デプロイ

リポジトリの `main` ブランチに push またはマージされると、GitHub Actions ワークフロー（`.github/workflows/deploy-pages.yml`）が起動します。
本モジュールは `npm run build` により `dist/` が生成され、GitHub Pages の `/3x3-v2/` パスに自動配置されます。

- **公開 URL**: `https://katoy.github.io/rust-r-cube/3x3-v2/`

### 2. 手動・他の静的ホスティングへのデプロイ

Vite の設定で相対パスベース（`base: "./"`）にビルドされるため、生成された `dist/` フォルダをそのまま任意の静的ウェブサーバー（GitHub Pages, Cloudflare Pages, Vercel, Netlify, Nginx, S3 等）に配置するだけで動作します。

```bash
# 本番向け成果物の生成 (dist/ ディレクトリ)
npm run build

# 生成された dist/ をプレビュー確認
npm run preview
```

---

## ディレクトリ構成

```
3x3-web-2/
├── Cargo.toml               # Rust WASM パッケージ定義
├── src/                     # Rust ソルバー実装 (4種の探索エンジン)
│   ├── lib.rs               # WASM バインディング & アルゴリズムディスパッチ
│   ├── cube.rs              # キューブ物理状態表現・回転操作
│   ├── coord.rs             # 座標変換・Kociemba 座標系
│   ├── search.rs            # Kociemba 2段階探索アルゴリズム
│   ├── cfop.rs              # CFOP (Layer-By-Layer) 階層解法ソルバー
│   ├── thistlethwaite.rs    # Thistlethwaite 4段階群縮小ソルバー
│   ├── korf.rs              # Korf IDA* 最短解探索ソルバー
│   ├── supercube.rs         # スーパーキューブ（センター向き）探索
│   ├── tables.rs            # 移動テーブル・枝刈りテーブル
│   └── table_io.rs          # テーブルのバイナリ I/O
├── web/                     # TypeScript フロントエンド
│   ├── main.ts              # アプリケーションエントリポイント
│   ├── view.ts              # DOM 構築・ARIA 属性・イベント設定
│   ├── scene.ts             # Three.js 3D レンダリング & カメラ制御
│   ├── cube-store.ts        # 状態管理・Undo/Redo・localStorage 永続化
│   ├── sound.ts             # Web Audio API 効果音生成
│   ├── triggers.ts          # 解法フェーズ分割 & 有名トリガームーブ注釈
│   ├── solver-client.ts     # Web Worker 通信クライアント
│   ├── solver.worker.ts     # Web Worker スクリプト
│   ├── editor.ts            # 6面カラーエディタ (Dynamic Import)
│   ├── camera.ts            # カメラ入力・WebRTC キャプチャ (Dynamic Import)
│   ├── camera-geometry.ts   # 幾何計算・透視射影変換
│   ├── camera-ui-helper.ts  # 当たり判定・座標系ヘルパー
│   ├── camera-canvas-renderer.ts # ガイド枠・頂点描画
│   ├── camera-results-ui.ts # 読み取り展開図・修正パレット
│   ├── image-sampler.ts     # 画像色サンプリング・補正
│   ├── centers.ts           # センター方位データ構造
│   ├── model.ts             # データ型定義
│   └── pwa.ts               # Service Worker 登録・更新検知
├── scripts/                 # ビルド & テスト支援スクリプト
│   ├── launch-offline.js       # オフライン動作起動・検証スクリプト
│   ├── generate-sw-precache.js # SW プリキャッシュリスト自動生成
│   ├── generate-test-images.ts # カメラ認識テスト画像生成
│   └── generate-pwa-icons.js   # PWA アイコン生成
├── tests/                   # Playwright E2E テスト群
│   ├── accessibility.spec.ts   # axe-core a11y 監査
│   ├── app.spec.ts             # 基本 UI / 操作テスト
│   ├── camera-live.spec.ts     # WebRTC ライブカメラテスト
│   ├── camera-input.spec.ts    # 静止画色認識テスト
│   ├── solution-structure.spec.ts # フェーズ分割・トリガームーブ検証
│   ├── settings-persistence.spec.ts # 設定永続化テスト
│   ├── pwa.spec.ts             # PWA キャッシュ・オフライン動作
│   └── ...
├── public/                  # PWA アイコン・マニフェスト・Service Worker
├── index.html
├── vite.config.ts
└── playwright.config.ts
```

---

## ライセンス

[MIT License](../LICENSE)
