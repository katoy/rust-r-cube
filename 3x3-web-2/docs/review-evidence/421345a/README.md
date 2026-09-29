# 421345a 全体レビューの再現証拠

対応報告：[code-review-421345a-2026-09-30.md](../../code-review-421345a-2026-09-30.md)。

対象は `421345a4726cd93552e3bc0a242083939d5bbd5f`。保存されたJSON／txtは今回の実測であり、性能・欠陥不存在の保証ではない。**成功するプローブは現状の問題が再現することをassertする。修正後には失敗し得る。** 製品コード・既存テストの修正は含まない。

## Web／カメラ

cwdは `3x3-web-2`、依存導入済み、WASM生成済みが前提。実行環境はNode26.7.0、Playwright1.63.0。infraのTypeScript関数抽出はNodeの `stripTypeScriptTypes` を使うため、この証拠コード自体のNode20互換は保証しない。

別ターミナルで開発サーバーを起動する：

```bash
npm run dev -- --port 5173 --strictPort
```

```bash
node docs/review-evidence/421345a/probe-ui.mjs
node docs/review-evidence/421345a/probe-scene.mjs
node docs/review-evidence/421345a/wasm-validation-contract-probe.mjs
```

UIプローブは `REVIEW_BASE_URL` で変更可。`probe-ui.json`の `redoDuringPreview`／`delayedEditorDuringSolve`／`invalidCenterTypeAccepted`／`failedMutationChangesFsm` がF01／F02／F03／F12。`validateContract`は完成状態の公開API契約だけを測定する。R局面・不正入力との比較は `wasm-validation-contract-output.txt`。

UIの状態観測はdev debug hookを使用する。通常UI操作と通信保留で経路を再現したが、これを本番bundle上のFSM hook計測とは呼ばない。`probe-scene.json`はF13の実shadow RenderTargetと破棄イベントを記録する。

カメラは別ターミナルで独立port：

```bash
npm run dev -- --port 5197 --strictPort
node docs/review-evidence/421345a/camera-review-probes.mjs
```

`camera-review-probes.json`はF04／F05／F14／F15／F16。偽カメラと配送保留でライフサイクルを検証する。188 bytesの4000×1画像はスクリプトが生成する実入力である。実機の認識精度測定ではない。

回帰テストの現行契約は独立port5198で確認した：

```bash
npm run dev -- --host 127.0.0.1 --port 5198 --strictPort
node docs/review-evidence/421345a/probe-test-contracts.mjs
```

`probe-test-contracts.json`はF21／F22。通常UIの解法取得後、solver-client応答に制御した完了barrierを置く。探索中の直接handler拒否、disabledプリセットクリックの待機、完了後のクリックを観測する。これはタイミングを固定した統合確認で、実ソルバーの性能評価ではない。

## SW／検証基盤／CI

以下は独立fixture・所有サーバーを使用し、終了時に自分のブラウザ／サーバー／scratchを片付ける。`infra-ci-probe.mjs`から呼ばれるfixtureのexit 1は意図的な失敗で、親プローブは結果を確認してexit 0。

```bash
node docs/review-evidence/421345a/infra-probe.mjs
node docs/review-evidence/421345a/infra-browser-probe.mjs
node docs/review-evidence/421345a/infra-cleanup-probe.mjs
node docs/review-evidence/421345a/infra-ci-probe.mjs
```

| ファイル                      | 根拠                                                                       |
| ----------------------------- | -------------------------------------------------------------------------- |
| `infra-probe-results.json`    | F06／F07／F17／F18。実SWハンドラ・実関数の抽出／障害注入                   |
| `infra-browser-results.json`  | F06／F08。実Chromium SWと局所HTTP配信。成功更新後の旧lazy import制約も記録 |
| `infra-cleanup-results.json`  | 所有previewサーバーの停止を確認。漏れの新規指摘は棄却                      |
| `infra-ci-results.json`       | F11／F19。event条件評価と意図的失敗fixture                                 |
| `infra-ci-fixture-output.txt` | list reporterの生成先と期待exit 1                                          |

`coverage-inventory.json`は今回の全体suiteの測定を24 Webファイルの棚卸しと照合した結果。Workerが欠落していた。再生成は `node docs/review-evidence/421345a/coverage-inventory.mjs`。CSS／WASMのURL捕捉を、その内部の意味的カバレッジとは扱わない。

## Rust

release rlibを事前に生成する。以下のバイナリ出力先はsession／一時ディレクトリなど、追跡外の明示した場所へ変更できる。実行後は作成したそのバイナリだけを削除する。

```bash
cargo build --release --lib
rustc --edition=2021 --crate-name rust_review_probe -O \
  docs/review-evidence/421345a/rust-review-probe.rs \
  --extern cube_studio=target/release/deps/libcube_studio.rlib \
  -L dependency=target/release/deps -o /tmp/cube-review-421345a-wide
/tmp/cube-review-421345a-wide
rustc --edition=2021 --crate-name rust_center_bound_probe -O \
  docs/review-evidence/421345a/rust-center-bound-probe.rs \
  --extern cube_studio=target/release/deps/libcube_studio.rlib \
  -L dependency=target/release/deps -o /tmp/cube-review-421345a-center
/tmp/cube-review-421345a-center
rustc --edition=2021 --crate-name rust_superflip_fallback_probe -O \
  docs/review-evidence/421345a/rust-superflip-fallback-probe.rs \
  --extern cube_studio=target/release/deps/libcube_studio.rlib \
  -L dependency=target/release/deps -o /tmp/cube-review-421345a-superflip
/tmp/cube-review-421345a-superflip
```

`rust-review-probe-output.txt`は座標全値・センター2048構成・CFOP等の追加検証結果。原本 `rust-review-probe.rs` も保存し、整形後の再コンパイル・実行でも同じ検査項目の成功を確認した。最小124手再現と失敗snapshotの72手を探索せず検証するソースも保存した。測定に使用したrelease rlibはRust最終変更から対象HEADまでソース差分なしであることを確認した。経過時間は環境依存。

`rust-msrv-api-versions.txt`は使用APIの安定化版の根拠と、1.80実ビルド未実行という限界を記録する。

## 基準チェックと入力棚卸し

`full-check.log`／`offline.log`／`superflip-isolated.log`／`superflip-repeat.log`／`validate-test.log`／`lifecycle-regressions.log`は実行ログ。Superflip失敗の全手順はRustプローブにも固定して保存した。`lifecycle-fast-cfop-snapshot.txt`は全体suiteのF1失敗時snapshotの抜粋で、新CFOPの5ms完成を確認できる。全体結果は228 passed／4 failed／1 skipped、品質ゲートexit 1。分離再実行では63bce97の2件は通過、b605038 F3は再失敗。

`inventory.mjs`はGit管理対象の分類・bytes・SHA-256を生成する。`inventory.json`はレビュー開始時の対象入力。これは古い文書・画像・生成物の全文精読を意味しない。新しく作成した未追跡のレビュー証拠は当初の棚卸しに含まれない。

```bash
node docs/review-evidence/421345a/inventory.mjs
```

既存チェックのrawログに書かれたローカルsessionのtrace／screenshotパスは元の環境の補助出力であり、このディレクトリにはバイナリtraceを複製していない。JSON、失敗ログ、固定手順と再実行コードを永続証拠とする。
