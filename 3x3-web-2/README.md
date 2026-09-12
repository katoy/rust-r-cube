# Cube Studio

Rustで解き、3Dでたどる、ブラウザ完結の3×3ルービックキューブsolverです。
日本語UI、実物の色入力、回転アニメーション、手順の前後再生を備えています。

## 起動

必要環境：Rust（動作確認 1.93）、`wasm32-unknown-unknown`、wasm-pack、Node.js（22.12以上）、npm。

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --locked
npm ci
npm run wasm
npm run dev
```

表示されたローカルURLを開いてください。通常は http://127.0.0.1:5173 です。
Rustを変更した場合は `npm run wasm` を再実行します。初回だけビルド用ツールの取得が必要です。

```sh
npm run build       # Rust/WASM・型検査・Webビルド → dist/
npm run preview     # 配布物の動作確認
```

`dist/` の内容を静的ホスティングへ配置できます。相対URLでアセットを解決するため、`/nested/cube/` のようなサブディレクトリにも対応します。`.wasm` のMIME型は `application/wasm` に設定してください。ブラウザの `file://` ではなくHTTP(S)で配信します。

## できること

- 25手のスクランブル、18種類の面回転、手順の一括入力、Undo／Redo。
- 3Dの視点回転・ズームと、WebGLを利用できない場合の2D展開図。
- 6面の色入力。1面ずつの持ち方ガイド、色の残数、未入力状態に対応。
- ピースの重複・欠落、エッジの反転、コーナーのねじれ、置換パリティの検査。
- 専用Workerで解法探索。通常5秒／延長30秒の上限、中止と再試行。
- 自動再生、一時停止、前後1手、任意ステップへの移動、再生速度、次の面の強調。
- 状態の自動保存、JSONの保存・読込、プリセット状態の読み込み、解法のコピー。
- スマートフォン、キーボード、色名の読み上げ、動きを減らす設定。

`U R F D L B` キーで回転、Shiftで逆回転。画面の `′`・`2` は回転方向の切替です。Spaceで再生／停止、矢印キーで手順を前後します。入力欄やダイアログ内ではショートカットを無効にしています。

実物の入力時は **上が白、前が緑、右が赤**。回転方向は、対象の面を正面から見た向きです。画面のカメラだけを回しても `U/R/F` の基準は変わりません。

通常の6色3×3専用です。センターの矢印、スライス回転、カメラ認識、旧版のテキスト形式は対象外です。スクランブルはランダムな回転列であり、競技用の一様ランダム状態生成ではありません。最短解・20手以内は保証しません。

## 構成

| 部分 | 役割 |
| --- | --- |
| `src/cube.rs` | URFDLBの54ステッカーとピース表現の相互変換、物理的検証、手順解析 |
| `src/coord.rs` / `src/search.rs` | 座標化、枝刈りを使うKociembaの2段階探索 |
| `build.rs` / `src/tables.rs` | ホスト側で探索テーブルを生成。ブラウザではバージョン・チェックサムを検査して読み込む |
| `src/lib.rs` | Rustの解法検証、再生状態生成、WASM境界 |
| `web/solver.worker.ts` / `web/solver-client.ts` | UIと探索の分離、中止、タイムアウト、古い応答の破棄 |
| `web/scene.ts` | Three.jsによる描画と回転アニメーション。静止時は再描画しない |
| `web/editor.ts` / `web/main.ts` | 色入力とアプリの操作・再生・保存 |

論理的な回転・各ステップの状態はRustが生成し、TypeScriptは描画と操作を担当します。解法は元の状態に適用して完成を確認してから返します。手動操作や状態の読込で既存の解法を無効化し、探索中ならWorkerを終了します。中止後はWorkerを再生成するので、SharedArrayBufferや特別なHTTPヘッダーは不要です。

座標・テーブル生成の実装は `../3x3-web/src/kociemba` を参考にしています。取り込み時にコーナーの向きをURFDLBの標準的な面表現に合わせ、全18手を独立した幾何学計算で検証しました。旧版のソース・未コミット変更には手を加えていません。

### プリセット状態

Web UI の「キューブを準備」セクションに「プリセット」タブがあり、有名なキューブ状態をワンクリックで読み込めます。各プリセットは JSON 形式で `cubes/` に保存されています：

| プリセット | ファイル | 説明 |
| --- | --- | --- |
| ✅ 完成状態 | `solved.json` | 完全に揃ったキューブ（テスト基準） |
| ⚡ スーパーフリップ | `superflip.json` | 全エッジが反転。神の数 20 手の最難問 |
| 🟢 簡単（5手） | `easy-5-moves.json` | 5 手で解ける初級者向けテスト用 |
| 🔄 T-Permutation | `t-perm.json` | 速解き（Speedcubing）の一般的なパターン |
| 🎲 ランダム（seed=1） | `seed-1-scramble.json` | 再現可能なシード値ベーススクランブル |

詳細は `cubes/README.md` を参照してください。

### WASM APIと保存形式

- `initialize()`：探索テーブルを読み込む。Worker起動時に実行。
- `validate(state)`：不正なら例外。正常なら完成状態かどうかを返す。
- `apply_moves(state, notation)`：回転後の状態・手順・各ステップの状態をJSONで返す。
- `scramble(seed)`：固定seedから25手の回転列を生成。
- `solve(state, budget_ms)`：検証済みの解法をJSONで返す。タイムアウト・入力不正は例外。

`state` は面順 `URFDLB`、各面は正面から見て左上から行優先の54文字です。色は対応するセンターの面記号で表現します。解法の戻り値は `{ state, moves, states, elapsed_ms, nodes }`。`states[0]` は開始状態です。Workerの要求・応答はリクエストIDと状態リビジョンを持ちます。

```json
{
  "version": 1,
  "state": "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB"
}
```

自動保存はこのブラウザのlocalStorageを使用します。ネットワーク送信はしません。ブラウザデータを消すと自動保存も消えるため、残したい状態はJSONで保存してください。解法や途中の入力ダイアログは再読み込み後に復元されません。

## 検証

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --release
npm run format:check
npm run build
npx playwright install chromium
npm test
```

Rustテストは、全18手と独立した3D座標回転との一致、逆回転、配色の往復変換、不正状態、既知状態、Superflip、固定seed 1〜1000のスクランブル、時間制限、テーブルのシリアライズ往復を検証します。

ブラウザテストは、入力→探索→完成、54マスの手入力、保存復元、アニメーションの中断、探索の中止・古い結果、初期化失敗と再試行、2Dフォールバック、サブディレクトリ配信を確認します。320・768・1024・1440pxで横にはみ出さないことと、axeによる通常画面・入力ダイアログの検査も含みます。スクリーンショットは `test-results/studio-desktop.png` と `test-results/studio-mobile.png` に出力します。

ブラウザテスト中はソースの編集やWASMの再ビルドをしないでください。開発サーバーの自動再読み込みが操作を中断します。

## 性能測定

```sh
npm run benchmark                 # 新作のRustコア、1000状態
node scripts/compare-legacy.mjs    # 旧版の探索コアを一時ディレクトリでビルドして比較
npm run benchmark:web             # 実ブラウザのWASM Worker、1000状態
```

比較は同じ端末、同じxorshift seed 1〜1000、同じ25手の回転列で行います。旧版はKociembaコアの `Search::solve(..., 128)` と既定の2,000万ノード上限を使用し、GUIやセンター方位の補正、上位solverの追加探索は含めません。各エンジン固有のピース表現から同じ物理的回転列を適用します。比較スクリプトは参照ソースのハッシュと一時ディレクトリを表示し、元ファイルは変更しません。

計測結果は初期化とウォーム探索を分けます。初期化時間にネットワーク取得・WASMコンパイルは含まれないため、ページの初回表示時間とは異なります。端末・ブラウザ・負荷で変動します。

配布WASMは事前生成テーブル込みで約6.18MB（gzip約3.36MB）。初回はこのダウンロードが必要です。JavaScriptはThree.js込みで約586KB（gzip約151KB）です。配信時のgzip／Brotliとアセットキャッシュを推奨します。ログイン、バックエンドAPI、外部フォント、解析サービスは使いません。

## ライセンス

MIT。参照元のKociemba座標・テーブル生成はkatoyによる `rust-r-cube/3x3-web` の実装です。外部依存のライセンスは各パッケージに従います。
