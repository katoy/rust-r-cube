# `125f8c8` 全体レビューの運用系再現

実施日: 2026-09-28。作業ディレクトリ: `3x3-web-2/`。

## 削除した公開ファイルとオフライン検証

1. `public/__review_freshness_probe.txt` を一時追加し、`npm run build` に成功。`generate-sw-precache.js` は **20 assets / version 60e5f9b5cc** を出力した。
2. 元ファイルだけ削除。`dist/__review_freshness_probe.txt` は残存した。
3. `node scripts/launch-offline.js --headless` を実行。先頭で **`⚡ 既存の最新ビルド成果物を使用します`** と表示し、ビルドしなかった。Service Worker は **20 個のアセット**をキャッシュし、終了コード 0 で **`ヘッドレスモードでのオフライン起動検証が正常に完了しました`** と報告した。
4. 一時ファイルを含まない `npm run build` を再実行。**19 assets / version a7b1788955** となり、`dist/__review_freshness_probe.txt` が存在しないことを確認した。一時ファイルはソースにも残していない。

原因は [`launch-offline.js`](../../../scripts/launch-offline.js#L18) が現在存在するファイルの最終更新時刻だけを計算し、削除された入力を検出できないため。`--fresh` 指定時はビルドするが、既定モードの「最新」判定はこの条件で誤る。
