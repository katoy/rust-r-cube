# コード全体・README レビュー（2026-09-21 / 81cf84d）

## 結果

**要修正。指摘は10件（P2: 8件、P3: 2件）。P0/P1 相当の問題は今回確認していない。**

通常のビルド・テストは成功したが、追加検証で PWA 更新後の遅延ロード失敗、キャンセル後の再探索失敗、ネイティブ API の異常終了、カメラ起動の競合を再現した。README には実装・実測値と一致しない説明が残っている。

対象は `3x3-web-2/`、コミット **`81cf84dc5aa30da4c162b700ac40bee5d269639b`**。レビュー開始時の作業ツリーはクリーン。製品コードと README は変更せず、この報告と検証資料を追加した。既存レビューは上書きしていない。

重要度は P0＝緊急、P1＝優先的な修正が必要、P2＝特定条件での不具合・誤った契約説明のため修正が必要、P3＝影響の小さい表示・文書上の問題とした。

| ID | 優先度 | 指摘 | 主な対象 |
| --- | --- | --- | --- |
| R01 | P2 | SW 更新時に旧キャッシュを削除し、既存タブのオフライン動的 import が失敗 | `public/sw.js:60–87`, `web/pwa.ts:1–14` |
| R02 | P2 | キャンセルした同期探索が残り、次の探索を時間切れにする | `web/solver-client.ts:95–101` |
| R03 | P2 | ネイティブで不正入力を渡すと `JsValue::from_str` がプロセスを abort | `src/lib.rs:244–248` |
| R04 | P2 | 古い `video.play()` の完了処理が新しいカメラを停止 | `web/camera.ts:784–790` |
| R05 | P2 | Korf のフォールバック解も「IDA* 最短探索」と表示 | `src/lib.rs:134–145`, `src/korf.rs:75–84` |
| R06 | P2 | Superflip 比較ベンチの「逐次方式」も同時最適化を実行 | `src/bin/superflip_bench.rs:85–104` |
| D01 | P2 | README の平均手数・センター補正手数と実装の不一致 | `README.md:101, 116–129` |
| D02 | P2 | README の Korf PDB・最短保証の説明が実装と不一致 | `README.md:91–103` |
| R07 | P3 | Kociemba 後付け補正のフェーズ境界が表示されない | `web/triggers.ts:70–96`, `web/main.ts:171–178` |
| D03 | P3 | README の機能・バージョン・実行経路・構成図が古い | `README.md:8, 40–52, 145–174, 186, 239, 359` |

## 実施した検証

環境: macOS / arm64、Rust 1.98.1、Node.js 26.7.0、npm 11.19.0、Playwright Chromium。

| 検証 | 結果 |
| --- | --- |
| `cargo test --release` | **112 passed / 1 ignored**、13.69秒 |
| `cargo test` | **112 passed / 1 ignored**、136.25秒 |
| `cargo fmt --check` | 成功 |
| `cargo clippy --all-targets -- -D warnings` | 成功 |
| `npm run format:check` | 成功 |
| `npm run build` | WASM・型検査・Vite・SW プリキャッシュ生成が成功 |
| `npm test -- --output=/tmp/cube-review-81cf84d-e2e` | **152 passed / 1 skipped**、7.0分 |
| `npm audit --json` | 報告された npm 脆弱性 0件 |
| CFOP、seed 1〜100 の25手スクランブル | 100局面すべて完成を再適用で検証 |
| 追加再現 | R01〜R05 を実行確認、R06 は呼び出し経路を静的確認、R07 は生成メタデータと分類処理を確認 |

Rust の ignored は1000局面の長時間テスト、Playwright の skipped は `CUBE_BENCH=1` が必要な1000局面ベンチ。これらは今回実行していない。Three.js チャンクが500 kBを超える Vite 警告は出たが、ビルド失敗ではない。

最初のビルド・E2E・npm audit はサンドボックスによる実行・待受・通信の制限で失敗したため、許可された制限外実行で上表の結果を確認した。これらの環境制限は製品不具合に数えていない。

## 指摘の詳細

### R01 — SW 更新後に既存タブのカメラ・エディタが開けなくなる（P2）

**根拠:** [public/sw.js](../public/sw.js) の install は `skipWaiting()`、activate は旧バージョンのキャッシュ削除と `clients.claim()` を行う。一方、[web/pwa.ts](../web/pwa.ts) は登録だけで、既存タブの更新通知・リロード処理を持たない。[web/main.ts](../web/main.ts) の `getCamera()` / `getEditor()` は動的 import を使う。

**再現:** 本番ビルドを一時 HTTP サーバーで配信し、カメラ未使用のページを開く。次の配信を模してカメラチャンクのファイル名・参照・プリキャッシュを変更し、登録済み SW を更新する。そのタブを維持してオフラインにし、「2方向の画像から入力」を押す。

**実測:** 旧キャッシュ `cube-studio-root-fb710d6672` が新キャッシュ `cube-studio-root-decafbad42` に置き換わったが、document は旧版のままだった。カメラは開かず、旧チャンクへの `Failed to fetch dynamically imported module` が発生した。

**影響:** 初回訪問後のプリキャッシュが成功していても、更新をまたいだ既存タブでは完全オフライン利用が壊れる。オンラインでも配信元から旧チャンクがなくなれば同じ構造の問題が起きる。

**修正案:** ユーザーの状態を保存したうえで更新を案内し、安全に document を更新する、または旧クライアントが終了するまで旧資産を利用できる更新方式にする。遅延ロード失敗時のユーザー向けエラーも必要。README:52 の「自動リロード機能」は現状の説明として削除または訂正する。

**追加すべきテスト:** 旧ページを開いたまま SW を更新し、オフラインで未使用のカメラ・エディタを初めて開く。現在のキャッシュ更新テストや「リロード後のオフライン起動」だけではこのケースを検出できない。

### R02 — 探索キャンセル後に次の探索が詰まる（P2）

**根拠:** [web/solver-client.ts](../web/solver-client.ts):95–101 の `cancel()` は pending Promise と監視タイマーだけを破棄する。[web/solver.worker.ts](../web/solver.worker.ts):24 の WASM 呼び出しは同期実行であり、その探索は終了しない。`ready` は true のままなので、次の要求が同じ Worker に送られる。

**再現:** Superflip の Korf 探索を30秒予算で開始し、200 ms後に cancel。続けて完成状態を Kociemba・5秒予算で探索する。

**実測:** 完成状態への要求が **6501 ms** 後に失敗し、エンジンは `ready=false` になった。前の探索が Worker を占有し、次の要求は自分の探索を始める前に `budget + 1500` の監視タイマーに達する。さらに `fail()` も `disposeRequest()` を経由するため、呼び出し元には本来のエラーではなく `cancelled` が返る。

**修正案:** 現在の同期 WASM を維持するなら、実行中のキャンセルでは Worker を terminate し、再初期化完了まで ready を false にする。初期化コストの最適化はキャンセル成立を保って行う。協調キャンセルを採用する場合は探索自体が中断要求を観測できる設計にする。また、明示的キャンセルと失敗の reject 理由を分ける。

**テストの穴:** `tests/app.spec.ts:269` は `setTimeout` で返信する Worker モックを使い、古い返信が状態を上書きしないことを確認している。同期 WASM が処理キューを占有するケースは検証していない。

### R03 — ネイティブのエラー経路でプロセスが終了する（P2）

**根拠:** [src/lib.rs](../src/lib.rs):244–248 はターゲットを問わず `JsValue::from_str` を呼ぶ。クレートは `rlib` も生成し、ネイティブ単体テストでも公開バインディングを呼んでいる。

**再現:** release の rlib にリンクした小さな実行ファイルで `cube_studio::apply_moves(cube_studio::cube::SOLVED, "INVALID")` を呼ぶ。

**実測:** `Err` は返らず、`function not implemented on non-wasm32 targets` → `panic in a function that cannot unwind` → **終了コード134** になった。

**影響範囲:** ネイティブからの公開バインディング呼び出し。不正な手順・局面・センター入力などでプロセスを終了させる。ブラウザ WASM 側で同じ abort が起きるという指摘ではない。通常テストの成功はネイティブの異常入力経路が安全であることを意味しない。

**修正案:** ネイティブで使う共通処理は `Result<_, String>` 等を返し、`JsValue` 化は wasm32 用の境界に限定する。バインディングをネイティブ非対応にするなら cfg で公開境界とテストを分離し、その契約を明示する。ゼロ初期化による旧実装へ戻すことは勧めない。

**過去レビューの訂正:** `docs/code-review-comprehensive-2026-09-21.md` の C02 改善案にある「`JsValue::from_str` はネイティブでも正しくサポート」は、このロック済み依存バージョンの実行結果と一致しない。

### R04 — 古いカメラ起動処理が新しいストリームを停止する（P2）

**根拠:** [web/camera.ts](../web/camera.ts):787–790 は `await video.play()` 後に要求世代の不一致を検出すると、インスタンス全体の `stopLiveStream()` を呼ぶ。その時点で `this.mediaStream` は次の要求のストリームになり得る。

**再現:** 1回目の `video.play()` を保留する。2回目の起動を完了させた後、1回目の Promise を解決する。実際の `MediaStream` オブジェクトを使い、getUserMedia と play の完了順だけを制御した。

**実測:** 2回目の起動直後は `isStreaming=true`、新ストリームの stop 回数0。古い処理の完了後は `isStreaming=false`、`video.srcObject=null`、新ストリームの stop 回数1になった。

**修正案:** stale な起動処理はその処理が取得したローカルの stream だけを解放する。世代が異なる場合、現在のインスタンス状態・映像・RAF に触れない。`video.play()` の resolve/reject と再起動の順序を入れ替えるテストを追加する。

### R05 — Kociemba の解を「IDA* 最短探索」と誤表示する（P2）

**根拠:** [src/korf.rs](../src/korf.rs):75–84 は Kociemba へのフォールバック結果をそのまま返す。[src/lib.rs](../src/lib.rs):134–145 は実際に使われた探索を判別せず、常に `IDA* 最短探索` フェーズを作る。

**実測:** 20手のプリセット手順から作った Superflip を Korf・1000 msで解くと、**21手** の結果に `algorithm="korf"`, `PhaseInfo { name: "IDA* 最短探索", start: 0, end: 21 }` が付いた。生成に使った20手の逆手順が解になるため、この21手が最短でないことは当該入力だけから確認できる。

**修正案:** 探索結果に実行アルゴリズム・フォールバック有無・最適性の保証有無を持たせ、UI に「Kociemba フォールバック」等を表示する。タイムアウトだけでなく深さ上限による切替も扱う。Thistlethwaite は既にフォールバック名を返しており、同様に区別できる。

### R06 — ベンチマークが同時最適化と逐次補正を比較していない（P2）

**根拠:** [src/bin/superflip_bench.rs](../src/bin/superflip_bench.rs):85–104 の「逐次方式」は `solve_state_with_centers(..., true, Some(initial_centers))` を呼ぶ。しかし [src/lib.rs](../src/lib.rs):149–158 はその引数でまずセンター同時探索を実行する。

**影響:** `npm run benchmark:superflip` の先頭の比較は、同時最適化を直接呼んだものと、同時最適化を内包する統合 API の比較になっている。後者は30秒予算のうち同時探索に最大25秒を使うため、純粋な「色を解く→センターだけ補正」の比較として時間・手数を解釈できない。この指摘は呼び出し経路の静的確認による。

**修正案:** 逐次側はセンター制約なしで色解法を求め、初期センターにその手順を適用し、残った向きに `solve_center_orientations` を明示的に追加する。両側とも完成局面と最終センター0を検証し、実装した計測方式を表示する。

### D01 — README の手数表が実装を過小に説明している（P2）

**対象:** [README.md](../README.md):101, 116–129、[src/supercube.rs](../src/supercube.rs)。

**実測との差:**

| 内容 | README | 今回の実装測定 |
| --- | --- | --- |
| CFOP の通常手数 | 約50〜70手 | seed 1〜100、25手スクランブルの色解法は **75〜155手、中央値120手** |
| U +90° / F −90° の補正 | 本アプリ定石28手 | `[1,0,3,0,0,0]` の補正は **30手** |
| U +90° / D −90° の補正 | 約28〜36手 | `[1,0,0,3,0,0]` の補正は **60手** |
| 90° の向きの組合せ | 必ず時計回り・反時計回りのペア | `[1,1,0,0,0,0]` も適法で、実装は **42手** で補正する |

最後のケースは向きの総和の偶奇条件を満たす。同じ向きの90°が2面あってもよく、実装は相殺ペアのマクロと180°補正を組み合わせて解決する。補正手順を完成キューブへ再適用し、外周ピースが完成状態のままであることを検証した。

**修正案:** 一般的な人間の CFOP 手数と本実装の LBL 手数、理論最短値と採用マクロの手数を分ける。実測表に入力集合・探索予算・向き設定・実行環境を付ける。なお速度の測定は同時実行中の他テストの影響を含むため、今回の性能保証には使わない。

### D02 — README の Korf 説明に未実装の PDB と過剰な保証がある（P2）

**根拠:** [README.md](../README.md):91 はコーナー・エッジの PDB を使用すると説明するが、[src/korf.rs](../src/korf.rs):43–55 の heuristic は誤配置・向き不一致の個数を4で割った下界の最大値だけであり、PDB を参照していない。

また比較表:103 の「最短（深さ12手以内）」は、予算内に IDA* が完了した場合という条件を欠く。探索深さが12以下の局面でも時間予算を先に使い切れば Kociemba へ切り替わる。残り予算0なら `None` を返すため、:92 の「確実に解を出力」も API の保証としては成立しない。

**修正案:** 現在の下界計算をそのまま記述し、「IDA* が予算内に完了した場合は最短。上限到達時は残余予算内で Kociemba を試み、失敗時はエラー」とする。一般的な Korf 法の説明と、この実装の機能を区別する。R05 の表示修正も併せて行う。

### R07 — センター後付け補正のフェーズ見出しが表示されない（P3）

**根拠:** Kociemba は色解法のフェーズ情報を返さず、後付け補正がある場合だけセンターフェーズを返す。例えば今回の小予算プローブは31手を返し、`phases` は `{ name: "センター向き解決", start: 1, end: 31 }` だけだった。

[web/triggers.ts](../web/triggers.ts):70–96 はフェーズ未割当の手を `phase=1`、配列の最初の明示フェーズも `phase=1` とする。[web/main.ts](../web/main.ts):171–178 は phase 番号が変わったときだけ見出しを追加するので、色解法とセンター補正の境界に見出しが出ない。`analyzeMoves` に未割当2手＋最初の明示フェーズ1手を渡しても、3手とも `phase=1` になることを確認した。

**修正案:** バックエンドから全区間を覆うフェーズを返すか、未割当区間に別の番号を与える。補正を伴う Kociemba 結果でセンター見出しが出ることをテストする。

### D03 — README の周辺説明・セットアップ説明を現状に揃える（P3）

- **バージョン:** :8 は TypeScript 5.x、:186 は Vite 6 だが、`package.json` は **TypeScript 7.0.2 / Vite 8.3.0**。
- **設定保存:** :40–43 のテーマ・各種表示設定の保存という説明に対し、`persist()` が保存するのは状態・センター・動きを減らす・速度・アルゴリズム、音声設定は別保存。テーマ切替・テーマ保存の実装は見当たらない。
- **E2E の対象:** :239 は E2E が本番 `dist/` を参照すると一括して説明するが、`playwright.config.ts` の通常 baseURL は **5173 の開発サーバー**。4173 の本番プレビューは PWA・サブディレクトリ等の一部テストで使う。ビルドの事前実行は引き続き必要だが、通常 UI テストがすべて本番バンドルを検証するわけではない。
- **画像生成スクリプト:** :359 は `.ts` を案内するが、npm scripts と fixture ヘルパーは **`scripts/generate-test-images.js`** を実行する。両ファイルは内容も分岐している。
- **構成図と関数名:** アーキテクチャ図は Store が動的 import・Sound・SolverClient を調停するように描いているが、実際の調停・永続化は `main.ts`。:109 の `apply_supercube_centers_if_needed` という関数も存在せず、現在は `solve_state_with_algorithm` 内で補正する。

R01 に関連する自動リロードの記述も修正対象。これらは新機能の追加要求ではなく、現行版の説明を正確にするための文書修正である。

## レビュー範囲と残る検証上の限界

Rust の全探索エンジン、座標変換・合法性検証・枝刈りテーブル・I/O・WASM 境界、Web の状態管理・再生・3D描画・カメラ/透視変換・保存/URL復元、PWA、ビルド/起動/画像生成/ベンチスクリプト、テスト設定と主要テスト、README・関連仕様を対象にした。README のデプロイ説明の確認に、親リポジトリの `.github/workflows/3x3-web-2.yml` と `deploy-pages.yml` も読み取った。

生成済みカバレッジ HTML や `node_modules/` を製品ソースの指摘対象にはしていない。npm audit は npm 依存に対する結果であり、Rust 依存の脆弱性監査結果ではない。最低対応バージョンの Rust/Node、Safari/Firefox、実カメラ端末での新たな実機検証、GitHub 上の CI 実行・公開サイトの配信状態の確認は行っていない。全キューブ状態の解法成功や厳密な性能上限を証明するレビューでもない。

直前のレビューが述べた debug テストの失敗は今回再現せず、112件が成功した。センター最終検証、局面の合法性検査、世代/リビジョンによる古い探索結果の排除、保存失敗時の継続動作、初回訪問後の通常オフライン起動などは既存コードとテストで確認できた。過去の指摘をそのまま現行版の未解決事項として転記していない。

## 保存した再現資料

- [実行結果 JSON](review-evidence/81cf84d/results.json)
- [Rust プローブ](review-evidence/81cf84d/probe.rs): native-error / korf-metadata / センター補正・CFOP100局面
- [ブラウザプローブ](review-evidence/81cf84d/browser.mjs): 探索キャンセル・カメラ起動競合
- [PWA 更新プローブ](review-evidence/81cf84d/pwa-update.mjs): 本番ビルドのチャンク変更を模擬した更新とオフライン動的 import
- [センターフェーズプローブ](review-evidence/81cf84d/center-phase.rs)

プローブは不具合を修正するものではなく、観測結果を出力する再現用コード。ブラウザのカメラ競合検証では完了順を制御するため API をモックしている。PWA 検証は本番 `dist/` のアセット名と参照をメモリ上で変更して更新を模擬し、配布ファイル自体は書き換えない。

プロジェクトルートからの実行例:

```sh
cargo test --release
rustc --edition=2021 docs/review-evidence/81cf84d/probe.rs \
  --extern cube_studio=target/release/deps/libcube_studio.rlib \
  -L dependency=target/release/deps -o /tmp/cube-review-probe
/tmp/cube-review-probe
/tmp/cube-review-probe korf-metadata
# 次は現行版では意図的に異常終了する再現コマンド
/tmp/cube-review-probe native-error

# browser.mjs は別ターミナルで npm run dev を起動した状態で実行
node docs/review-evidence/81cf84d/browser.mjs

# PWA プローブはビルド後、独自の一時サーバーを起動して終了時に閉じる
npm run build
node docs/review-evidence/81cf84d/pwa-update.mjs
```

**修正順の提案:** R01/R02 の更新・キャンセルのライフサイクル → R03/R04 の異常系と競合 → R05/R06 の結果表示・検証方法 → README の整合性と R07。修正時には通常テストの再実行に加え、今回再現した操作順序を回帰テストへ取り込む。
