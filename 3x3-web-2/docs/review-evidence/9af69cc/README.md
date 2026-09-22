# 9af69cc レビューの検証証跡

対象: `9af69cc8176ce63b55704f4af6d7b59b2b35150f`。レポートは [code-review-9af69cc-2026-09-23.md](../../code-review-9af69cc-2026-09-23.md)。

2026-09-22に開始し、23日に再開した。`environment.json` にバージョンと主要ファイルのSHA-256を保存した。アプリ本体は変更していない。

## 結果とファイル

| ファイル | 内容 |
| --- | --- |
| `rust-tests.log` | 22日に完了: Rust 119 passed / 1 ignored |
| `rust-fmt.log` | 22日に完了: 整形差分なし、空ログ |
| `web-format.log` | 22日に完了: Prettier成功 |
| `clippy.log` | 23日に完了: 警告をエラーにした静的検査が成功 |
| `build-resumed.log` | 23日に再実行: WASM・型検査・Web・SW生成が成功 |
| `npm-audit-resumed.json` | 23日に再実行: npm既知脆弱性0件 |
| `e2e-resumed.log` | 23日に完了: Chromium 161 passed / 1 skipped、7.3分 |
| `browser.jsonl` / `probe_browser.mjs` | 新規R01〜R03の再現。偽カメラ、実画像、ポインター操作 |
| `live-after-file.png` | ファイル選択後も表示されるライブ映像。読み取り対象は別のPNG |
| `main.jsonl` / `probe_main.mjs` | 新規R04の再現と、前回R02の途中停止・ページ離脱後の保存確認 |
| `wasm.jsonl` / `probe_wasm.mjs` | 幾何モデルとの3,000回の回転照合、162,000個の矢印照合、4ソルバー×2モード×25局面 |
| `browser.stderr`, `main.stderr`, `wasm.stderr` | 成功した追加プローブの標準エラー。いずれも空 |

`build.log` と `npm-audit.json` は22日のサンドボックス制限による失敗ログ、`e2e.log` は23日のサーバー待受制限による失敗ログ。最終結果には `*-resumed.*` を使う。Chromium起動も同様の制限があり、追加プローブは通常環境で実行した。

## 通常チェック

プロジェクトルートで実行する。依存関係・RustのWASMターゲット・wasm-pack・Playwright Chromiumが必要。

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --release
npm run format:check
npm run build
npm test -- --output=/tmp/cube-review-9af69cc-e2e
npm audit --json
```

`npm run check` と同じ品質ゲートを個別実行した。全体テストの出力先は `/tmp`、保存したテキストログはこのディレクトリ。

## 追加プローブ

画像フィクスチャがなければ、先に `npm run generate-test-images` を実行する。

カメラプローブは、別ターミナルで開発サーバーを起動してから実行する。

```sh
npm run dev -- --port 5173 --strictPort
```

```sh
node docs/review-evidence/9af69cc/probe_browser.mjs
```

`REVIEW_BASE_URL` で別の開発サーバーURLを指定可能。各ケースは独立した使い捨てブラウザコンテキストを使用する。撮影・輪郭検出・色認識関数は書き換えない。キャンバスに関する対照実験のみ、元画像を別のキャンバスへ描いて同じ検出関数を呼ぶ。

JSON・保存プローブは、ポート5185に専用Viteを起動し、終了時にそのプロセスを終了する。

```sh
node docs/review-evidence/9af69cc/probe_main.mjs
```

JSONの競合実験では、元の `File.text()` で得たデータのPromise解決だけを保留し、完了順序を制御する。state・revision・読込ハンドラは書き換えない。

WASM照合はブラウザ・サーバー不要。先にWASMをビルドする。今回のNode.js 26.7.0では型を除去して `web/model.ts` を直接読み込むため、同等のTypeScript実行機能を持つNode.jsを使用する。

```sh
node docs/review-evidence/9af69cc/probe_wasm.mjs
```

幾何モデルのseedは20260923、移動回数3,000。ソルバーはseed 1〜25、各500ms予算、4方式×色のみ／センター込み。後者ではスクランブル由来の向きにさらに1面の180°を加える。全200件成功した。実際の手順・途中状態・フェーズ境界・必要時のセンター完成をassertしている。

JSONLは各プローブの標準出力を保存したもの。ファイルの更新時刻だけでなく、対象コミットと `environment.json` のハッシュも確認すること。古いコミット向けの再現スクリプトを修正後のコードで動かす場合、元の不具合をassertするケースは失敗し得る。
