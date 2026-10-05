# 421345a 修正後の検証証拠

対応記録：[review-fixes-421345a-2026-09-30.md](../../review-fixes-421345a-2026-09-30.md)。

元のレビュー証拠は [review-evidence/421345a/](../../review-evidence/421345a/README.md) に変更せず保存。ここには修正後のコマンド出力を保存する。UIの初回失敗・再実行も残し、失敗を合格に読み替えない。

恒久的な回帰は `tests/code-review-421345a-regression.spec.ts`、`camera-async-regressions.spec.ts`、`infra-offline-regression.spec.ts`、`quality-contracts.spec.ts`、既存FSM／coverage／回帰テスト、およびRust testsにある。

`tests/solver-barrier.ts` は本物のsolver結果を受け取った後に配送を保留する。状態検査を短い探索時間に依存させない。72手UI回帰は記録済みの合法fallbackを実WASM `apply_moves` で再構成し、solver応答として配送する。探索性能をmockで証明するものではない。

ログ内の元のtest-results／session traceパスは元の実行環境の補助出力で、バイナリtraceの複製は含めない。

## 最終検証

- [full-check-final.log](full-check-final.log)：`npm run check`終了0。Rust release 141 passed／1 ignored、Playwright 269 passed／1 skipped。
- [ui-regressions-final.log](ui-regressions-final.log)：新UI回帰9件成功。同一previewへのRedoケースも独立登録して実行。
- [scripts-regressions.log](scripts-regressions.log)：SIGINT・cleanupを含む運用回帰5件成功。
- [msrv-native.log](msrv-native.log)／[msrv-wasm.log](msrv-wasm.log)：Rust1.87のlocked native／WASMチェック成功。
- [offline-fresh.log](offline-fresh.log)：固有5199でfreshビルド、資源照合、19資源cache、ネットワーク遮断後の起動成功。
- [offline-cleanup.log](offline-cleanup.log)：所有listener5199の解放確認。共有4173は停止しない。
- [verified-inputs.json](verified-inputs.json)：最終ゲート開始前の120入力のSHA-256。ゲート後・fresh後に `node docs/fix-evidence/421345a/verified-inputs.mjs --verify` で不変を確認。最終出力は [inputs-unchanged.log](inputs-unchanged.log)。
- [browser-smoke.json](browser-smoke.json)：隔離ChromeのDOM click複合操作とconsole確認。

## 不合格・中断の履歴

[full-check-interrupted.log](full-check-interrupted.log) は追加Redo境界の補強前の中断。[full-check-first-complete.log](full-check-first-complete.log) は265 passed／3 failed／1 skippedの初回完遂で、合格証拠ではない。[verified-inputs-first.json](verified-inputs-first.json) はその実行時の入力hash。

初回完遂の失敗は、新F01の誤った入れ子登録でF02が2件失敗し、SIGINTテストが共有nested previewの資源照合で終了1になったもの。登録位置と固有portによる隔離を修正し、全体を再実行した。`history-tests.log` だけでは誤登録の追加F01を実行した証明にならないため、最終UIログの独立ケースを根拠とする。
