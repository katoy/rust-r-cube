# 全体コードレビュー — `9fb1ebc`（2026-09-28）

## 判定と対象

**要修正。** 現行 `fix/superflip-preset` の `9fb1ebcb304a096db7bc84a895778eacd9c914d9` を対象とする。最優先は Service Worker 更新テスト 17 件の失敗で、Pages 公開ワークフローの検証ジョブも通過できない。加えて、ブラウザ表示の不整合と、ビルド鮮度・静的ガードレールの偽陽性を再現した。レビュー開始時の作業ツリーはクリーンで、製品コードとテストコードは変更していない。

`src/` の合法局面・座標・4探索方式・センター補正・テーブルシリアライズ・WASM 境界、`web/` のストア・Worker・入力・画像処理・3D 描画・保存、`public/sw.js`、ビルド/検証スクリプト、Playwright テストと親リポジトリの Pages ワークフローを確認した。生成物 `pkg/`・`dist/` は本番ビルドとオフライン起動で確認した。過去のレビュー記述は現行コードの証明として扱っていない。

## 指摘（優先順）

### F1 — P1: 新しい生成スクリプト依存が隔離テストへ渡されず、17 件が失敗する

[`generate-sw-precache.js`](../scripts/generate-sw-precache.js#L5) は新たに `./build-manifest.js` を import する。一方、[`sw-update.spec.ts`](../tests/sw-update.spec.ts#L53) の `build` フィクスチャは生成スクリプトだけを一時ディレクトリへコピーして実行する。依存ファイルがないため、実行時に `ERR_MODULE_NOT_FOUND` で止まる。

`npm test` は **172 passed / 17 failed / 1 skipped**。失敗 17 件はすべて `sw-update.spec.ts` で、保存した[代表テストの生ログ](review-evidence/9fb1ebc/sw-test.log)にも同じ import エラーが出る。これは Service Worker 実装の成否を判定する前の、フィクスチャ構築段階での失敗である。親リポジトリの [Pages ワークフロー](../../.github/workflows/deploy-pages.yml) は `test-3x3-web-2` を `build` の前提にしているため、公開ゲートも通過できない。

**修正案:** フィクスチャへ `build-manifest.js` と ESM 設定を含めるか、生成ロジックを依存込みで呼べるようにする。代表テストと `npm test` 全件を再実行し、生成内容の比較に到達することを確認する。

### F2 — P2: 入力マニフェストに `tsconfig.json` とマニフェスト生成器自身がない

[`collectInputFiles`](../scripts/build-manifest.js#L36) は `src/`・`web/`・`public/` と一部設定を記録するが、`tsconfig.json` と `scripts/build-manifest.js` を含めない。`npm run build` は `tsc --noEmit` を実行するので `tsconfig.json` は検証結果を変える入力であり、後者はビルド時に書き出すマニフェストの生成ロジックそのもの。オフライン検証は[`checkInputsFreshness`](../scripts/launch-offline.js#L43)の `fresh` に従い、再ビルドを省略する。

[隔離した再現プローブ](review-evidence/9fb1ebc/probe-current-head.mjs)で両ファイルを変更しても `fresh: true` だった（[結果 JSON](review-evidence/9fb1ebc/probe-current-head.json)）。これらの変更後に `npm run offline -- --headless` を実行すると、古い `dist` で検証に成功し得る。今回の現在の `dist` を使ったオフライン起動自体は成功したが、この鮮度判定の保証にはならない。

**修正案:** ビルド・型検査・マニフェスト生成に影響する入力を一覧へ追加する。設定変更後に `ensureBuild` が再ビルドする統合テストを設ける。

### F3 — P3: 非表示側へ画像を読み込むと、画像カードが「未選択」のまま残る

画像Aのデコード中に利用者がビューBを選ぶと、[`onImageLoaded`](../web/camera.ts#L357) は `imageA` を保存するが、`currentView !== view` の経路では `renderImage`・`autoDetectOutline`・`update` のいずれも呼ばない。[`update`](../web/camera.ts#L820) が更新する画像Aカードの状態・文言が古いままになる。

ブラウザの[再現結果](review-evidence/9fb1ebc/probe-current-head.json)では `imageA` が存在するのにカードは `未選択（クリックまたはドロップ）`、`has-file` クラスもなかった。Aへ切り替えた直後に `読込完了 (20×20)` に変わる。前回修正した「遅延完了でビューBをAへ戻す」問題は再発していないが、読込結果の表示が操作中の事実と食い違う。

**修正案:** 非表示側への保存後も、そのカードとステータスだけを更新する。ビューと選択中の6点は変更せず、A/B逆順完了時の表示をテストする。

### F4 — P3: パス正規化の静的ガードレールが現在の実装ファイルを見ていない

パス正規化は [`storage-key.ts`](../web/storage-key.ts#L1) に移されたが、[`checkPathNormalization`](../scripts/check-review-guardrails.js#L100) は `web/main.ts` だけを走査する。`main.ts` の import には `pathname` がないため、実装を壊しても F4 検査が実行されない。

隔離フィクスチャで `getScopedStorageKey` を `pathname` を正規化せずキーへ連結する関数に替えた。この関数は `/` と `/index.html` を別キーにするが、ガードレールは終了コード **0** と「検査をパスしました」を返した（[プローブ](review-evidence/9fb1ebc/probe-current-head.mjs)と[結果](review-evidence/9fb1ebc/probe-current-head.json)）。現行の実装自体は両URLを正規化し、既存の E2E テストもある。指摘は静的チェックが保証しているように見える範囲に限定する。

**修正案:** 実装ファイルを検査対象にする。文字列走査の通過を仕様保証とせず、別名URLに対する動作テストを品質ゲートに残す。

## 横断確認

| 領域 | 現行コードと実行結果から確認したこと |
| --- | --- |
| Rust / WASM | [`parse_state`](../src/cube.rs#L33) は色数、センター、重複、向き、パリティを検査する。4方式の解法は [`solve_state_with_algorithm`](../src/lib.rs#L85) に集まり、最終局面とセンター向きの検証は release にも残る（同ファイル L240–267）。探索・座標・テーブルの release テストは 141 件成功。1000局面テストは ignored。 |
| 状態・入力 | [`CubeStore`](../web/cube-store.ts) のプレビュー確定、Undo/Redo と Worker の revision 照合を確認。JSON は全文読込前に 64KB を検査し、カメラ画像は読込前に 20MB を検査する。前回指摘の履歴・URL・遅延ビュー選択の回帰テストは今回の成功群に含まれる。 |
| 描画・リソース | [`CubeScene.dispose`](../web/scene.ts#L334) はジオメトリ・マテリアル・テクスチャ・リスナーを破棄する。カメラ終了時はストリーム停止と読込世代の無効化を行う。非表示画像カードだけ F3 の表示遅延がある。 |
| PWA・公開 | [`sw.js`](../public/sw.js) はナビゲーションのクエリを外してキャッシュし、同一スコープの旧キャッシュだけを削除する。実際の本番ビルドは19アセットをキャッシュしてオフラインで起動した。ただし更新シナリオ17件は F1 のため実行前に停止。Pages の `build` は同一ワークフローのテストジョブを `needs` とする。 |
| セキュリティ・性能 | 主要な利用者入力表示は `textContent` を使い、`npm audit --json` は現時点で既知アドバイザリ0件。本番出力の WASM は約6.29 MB、Three.js チャンクは約576 KB。実回線での転送時間・端末別メモリは今回測定していない。 |

## 検証と限界

- `npm run check:guardrails`、`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test --release`、`npm run format:check` は成功。Rust は **141 passed / 1 ignored**。
- `npm run check` は WASM ツールのサンドボックス権限でビルド段階に停止した。同じ `npm run build` を権限付きで再実行すると WASM・型検査・Vite・プリキャッシュ生成まで成功。
- `npm test` は **172 passed / 17 failed / 1 skipped**（Chromium）。17件は F1 と同じ `ERR_MODULE_NOT_FOUND`。一件を単独再実行し、[生ログ](review-evidence/9fb1ebc/sw-test.log)を保存した。緑のテストだけで Service Worker 更新の正しさは主張しない。
- `node scripts/launch-offline.js --headless` は19アセットの初回キャッシュとオフライン再読込を通過。F2 の対象外入力を変えた場合の鮮度は別の隔離プローブで検証。
- `npm audit --json` は既知アドバイザリ0件。`cargo audit` は未導入。実機カメラ、Chromium 以外のブラウザ、長時間1000局面探索、実回線・低性能端末は今回未検証。
- 過去資料の「全件成功」「完全解消」「100%カバレッジ」は現在の証拠に置き換えていない。今回の製品変更はなく、指摘と再現証跡のみを保存した。
