# 9af69cc 指摘修正後の検証証跡

2026-09-23、[修正内容](../../review-fixes-9af69cc-2026-09-23.md) の検証。元の [9af69cc の証跡](../9af69cc/README.md) は修正前の記録として維持する。

アプリ本体の修正コミット:

- `cd0ec07`: カメラの表示・輪郭検出・手動調整保持
- `8e03ca2`: JSON最新選択の優先と古い要求の副作用抑止
- `18a009e`: CFOP探索ノード数の引継ぎ

この後に追加したCFOPブラウザ表示検証（`d27bed7`）、Superflipテストのコメント・名称修正（`6801dff`）も含めたソースを検証した。`environment.json` に検証対象コミット `d27bed7`、ツールのバージョン、主要ソースのSHA-256を記録している。後続の変更は検証記録のみ。

| ファイル | 内容 |
| --- | --- |
| `rust-tests.log` | Rust全体: 120成功、1 ignored |
| `build-resumed.log` | WASM・型チェック・Web・Service Worker生成に成功 |
| `e2e-resumed.log` | Chromium全体: 171成功、1 skipped、7.9分 |
| `build.log` | 初回のサンドボックス制限によるビルド失敗 |
| `e2e.log` | 初回のローカルサーバー待受制限によるE2E開始失敗 |
| `environment.json` | 検証対象ソースのコミット・ハッシュと実行環境 |

fmt、Clippy、Prettierは端末出力で成功を確認した。再実行はプロジェクトルートで以下を実行する。

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --release
npm run format:check
npm run build
npm test -- --output=/tmp/cube-fixes-9af69cc-e2e
```

集中的な回帰確認は以下。CFOP表示のテストでは更新後WASMが必要なため、先にビルドする。

```sh
npm test -- tests/camera-image-state.spec.ts tests/json-load-race.spec.ts
npm test -- tests/app.spec.ts -g 'solver algorithm selection'
cargo test --release cfop_reports_visited_nodes_in_solution_statistics
```

RED確認: カメラ6失敗、JSON3失敗＋手動操作保護1成功、Rust CFOP統計1失敗。更新前WASMを使ったCFOPブラウザ表示検証も1失敗。修正後のカメラ・JSON集中検証は10成功、Rust CFOP統計は1成功。これらのテストは実際の画像・JSONデコード・ソルバーを使い、非同期完了の境界だけを制御している。
