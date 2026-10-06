# 全体コードレビューレポート（HEAD 4a010ac / 2026-10-06）

- **対象バージョン**: `4a010ac`（`main` ブランチ最新 HEAD、PR #19〜#22 マージ後）
- **実施日**: 2026-10-06 (JST)
- **対象範囲**: git 管理下の全ソースコード（`src/` 配下全 Rust ソース、`web/` 配下全 TypeScript、`public/`、`scripts/`、`tests/`、設定ファイル、README、CI ワークフロー）
- **総合判定**: **要修正 (CHANGES REQUESTED)** — High 2 件 / Medium 4 件 / Low 5 件

---

## 0. 本レビューの目的と過去のレビュー・PR マージに対する優位性

本プロジェクトでは直近において PR #19（2x2x2 ソルバー・マルチキューブ対応、LBL 法・Ortega 法の追加）およびそれに付随する PR #20〜#22 の改善・修正がマージされました。
ビルドは正常に通り、Rust 単体テスト（141件）および Playwright E2E テストスイート（約290件）はすべてパスしています。

しかし、「ビルドが通り、既存テストが成功していること」と「コード全体としての仕様整合性・UX・リソース管理が健全であること」は別問題です。
本レビューでは、最新 HEAD `4a010ac` の全コード（Rust コア、Three.js 描画層、有限状態機械、URL/ストレージ永続化、PWA、テストコード）を 1 行ずつ精査し、特に **2x2x2 マルチキューブ拡張に伴って生じた「機能拡張の接合部での脱落」や「3x3 前提コードの残存による不整合」** を徹底的に検証しました。

### 本レビューで明らかにした決定的事実

1. **新機能の永続化・共有からの完全脱落（H1）**: 2x2x2 向けに追加された画期的な解法アルゴリズムである `lbl`（LBL 法）および `ortega`（Ortega 法）が、`web/url-params.ts` の URL パラメータ解析および共有リンク生成ロジック、さらに `web/main.ts` の `localStorage` 復元ロジックのホワイトリストから完全に抜け落ちています。ユーザーがせっかく選択した 2x2 解法は、リロードや URL 共有時に一切反映されず消滅します。
2. **2x2 完成時の達成音不発（H2）**: `web/main.ts` の手動回転処理において、完成判定を 3x3 の 54 文字定数 `SOLVED` と固定比較しているため、24 文字の 2x2 キューブを解き終えても完成ファンファーレ（効果音 `sound.playSuccess()`）が絶対に鳴らない UX バグが存在します。
3. **Three.js マテリアルのリークとジオメトリ重複破棄（M1）**: 3x3 と 2x2 を切り替える際、`web/scene.ts` の `clearCube()` で `this.bodyMaterial` が破棄されずに GPU メモリを圧迫し続ける上、全ピースで共有されている本体ジオメトリ `body` に対してピース数分の重複 `dispose()` が実行されています。
4. **2x2 ファイル読込時のセンターパリティ誤処理リスク（M2）**: JSON 盤面読込時にキューブ種別の判定を行わず、センターのない 2x2 に対しても 3x3 前提の `centersFromInput()` が無条件で呼び出されています。
5. **フェーズ名日本語マッピングの形骸化（M3）**: `web/triggers.ts` に 2x2 用のフェーズ名マップ `PHASE_LABEL_MAP` が用意されたものの、Rust 側が返す実際のフェーズ文字列（公式名が付与された動的文字列）とキーが一致せず、常にルックアップに失敗して生の英語文字列が表示されるデッドコードになっています。
6. **カバレッジ偽装コードの残存（M4）**: `tests/coverage.spec.ts` 内に、アサーションを一切持たず例外を握りつぶして行実行率だけを稼ぐコードが依然として残存しており、プロジェクト品質規約（`CLAUDE.md` §4）に抵触しています。

---

## 1. 五軸総合評価 (Five-Axis Quality Evaluation)

| 評価軸 | 判定 | 評価概要 |
| :--- | :---: | :--- |
| **1. 正確性 (Correctness)** | **良 (Good)** | 2x2 幅優先探索（最適解 11 手）および LBL/Ortega ヒューリスティックソルバーは群論的・手順的に正確。しかし、2x2 完成時の効果音不発や URL/共有パラメータの脱落など、マルチキューブ化に伴うフロントエンド接合部の不整合が存在する。 |
| **2. パフォーマンス (Performance)** | **良 (Good)** | 2x2 ソルバーの探索テーブルや WASM 連携は極めて高速。一方、キューブ種別切り替え時の Three.js マテリアル解放漏れによるメモリリーク懸念がある。 |
| **3. アーキテクチャ (Architecture)** | **優 (Very Good)** | 2x2 と 3x3 を共通の `CubeType` / `CubeStore` で抽象化し、Web Worker への委譲もクリーンに設計されている。URL 管理・ストレージ層のホワイトリスト更新が追従していない点のみ改善を要する。 |
| **4. 可読性・保守性 (Readability)** | **良 (Good)** | 厳格な TypeScript 型定義、Rust clippy/fmt、JSDoc を完備。ただし `triggers.ts` のマッピングキー不一致や、`tests/coverage.spec.ts` のアサーションなしテストコードが保守性を損ねている。 |
| **5. セキュリティ・a11y (Security & a11y)** | **優 (Very Good)** | サニタイズ、完全ローカル実行、WAI-ARIA ラジオグループ・ライブリージョンなどアクセシビリティ標準に高度に準拠。エディタの 2x2 ガイド文言に軽微な改善余地あり。 |

---

## 2. 重要指摘事項（High / Medium）の詳細検証

### H1【High・確認済み】`web/url-params.ts` & `web/main.ts`: 2x2x2 新アルゴリズム（`lbl` / `ortega`）の URL・共有・永続化からの脱落

- **該当箇所**:
  - [`web/url-params.ts:22-29`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/url-params.ts#L22-L29)
  - [`web/url-params.ts:80-86`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/url-params.ts#L80-L86)
  - [`web/main.ts:1233-1238`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/main.ts#L1233-L1238)
- **確証度**: **確認済み**（コード追跡および型定義照合）
- **現象**:
  PR #19 で `SolverType` 型に `"lbl" | "ortega"` が追加され、2x2x2 の解法として LBL 法および Ortega 法が選択可能になりました。
  しかし、URL クエリパラメータの解析および共有 URL 生成、さらにはブラウザリロード時の `localStorage` 復元処理において、許可されるソルバーのホワイトリストが 3x3 の 4 種（`cfop`, `thistlethwaite`, `korf`, `optimal`）のまま更新されていません。

  ```typescript
  // web/url-params.ts:22-29
  const VALID_SOLVERS: SolverType[] = [
    "cfop",
    "thistlethwaite",
    "korf",
    "optimal",
    // ← "lbl", "ortega" が欠落している！
  ];
  ```

  ```typescript
  // web/url-params.ts:80-86 (buildShareUrl)
  if (
    state.solver &&
    state.solver !== "cfop" &&
    VALID_SOLVERS.includes(state.solver) // ← "lbl", "ortega" は除外される
  ) {
    params.set("solver", state.solver);
  }
  ```

  ```typescript
  // web/main.ts:1233-1244 (localStorage 復元処理)
  if (
    ["kociemba", "cfop", "thistlethwaite", "korf", "optimal"].includes(
      data.solverAlgorithm,
    )
  )
    solverAlgo.value = data.solverAlgorithm;
  const restoredType: CubeType =
    data.cubeType === "2x2" || (data.state && data.state.length === 24)
      ? "2x2"
      : "3x3";
  store.setCubeType(restoredType);
  ```

  さらに深刻な点として、**復元実行順序と DOM のライフサイクルの不整合** が存在します。
  `main.ts` では、`store.setCubeType(restoredType)`（および `updateCubeTypeUI` による `<select id="solver-algorithm">` の `<option>` 要素再構築）が実行される **前** に `solverAlgo.value = data.solverAlgorithm` を代入しています。
  初期状態の DOM では `<select>` 内に 3x3 用の選択肢（kociemba, cfop, etc.）しか存在しないため、仮にホワイトリスト検査を通過させたとしても、DOM 上に `<option value="ortega">` が存在しない時点で代入が行われるため、ブラウザによって代入値が破棄され、その後の `updateCubeTypeUI` によってデフォルトの `lbl` や `kociemba` に上書きされてしまいます。
  URL パラメータの適用処理（Line 1261-1269）でも同様に、`parsedParams.solver` を適用する前に `<select>` の選択肢を 2x2 用に同期させる必要があります。
- **影響**:
  1. ユーザーが 2x2 キューブで「Ortega 法」または「LBL 法」を選択して「共有」ボタンを押しても、生成される共有 URL には `?solver=ortega` が付与されず、相手側で解法設定が共有されません。
  2. URL に `?solver=ortega` や `?solver=lbl` を直接指定してアクセスしても、バリデーションで弾かれ、デフォルトの `cfop` に強制変更されてしまいます。
  3. ページをリロードした際、`localStorage` に保存されていた `ortega` / `lbl` が無視され、復元されません。
  4. 仮にホワイトリストのみを修正した場合でも、DOM オプション再構築前の先行代入によって選択状態がリセットされるため、復元順序の修正が不可欠です。
- **推奨修正案**:
  1. `VALID_SOLVERS` に `"lbl"` と `"ortega"` を追加し、ホワイトリストを統一します。
     ```typescript
     // web/url-params.ts
     export const VALID_SOLVERS = [
       "kociemba",
       "cfop",
       "thistlethwaite",
       "korf",
       "optimal",
       "lbl",
       "ortega",
     ] as const;
     ```
  2. `main.ts` の `localStorage` および URL パラメータ復元処理において、先に `restoredType` を決定して `updateCubeTypeUI(restoredType)`（または `store.setCubeType`）を実行し、DOM の `<option>` 要素が 2x2 用に切り替わった **後** に `solverAlgo.value = ...` を設定するように処理順序を是正します。

---

### H2【High・確認済み】`web/main.ts`: 2x2x2 完成時の達成効果音（`sound.playSuccess()`）未発火バグ

- **該当箇所**: [`web/main.ts:739-741`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/main.ts#L739-L741)
- **確証度**: **確認済み**（コード追跡および状態定数照合）
- **現象**:
  キューブの手動回転（キーボードまたは 3D ビュー上のドラッグ操作）が行われた際、回転後の盤面が「完成状態」であるかを判定してファンファーレ効果音を鳴らす処理があります。
  ```typescript
  // web/main.ts:739-741
  if (result.state === SOLVED) {
    sound.playSuccess();
  }
  ```
  ここで使用されている `SOLVED` は `web/main.ts:40` で定義された 3x3 用の 54 文字定数（`"UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB"`）です。
  一方、2x2x2 キューブの完成状態は 24 文字（`"UUUURRRRFFFFDDDDLLLLBBBB"`）です。
  したがって、2x2 キューブをどれだけ手動で回して完成させても、`result.state === SOLVED` は絶対に `true` にならず、達成効果音が一切再生されません。
- **影響**:
  2x2x2 モードにおいて、パズルを自力で解いた際の最も重要なフィードバック（達成感を与える成功サウンド）が完全に失われており、UX 上重大なバグとなっています。
- **推奨修正案**:
  `store.getCubeType()` または `cube-store.ts` の `getSolvedState()` を用いて、現在のキューブ種別に応じた完成状態と比較します。
  ```typescript
  // web/main.ts:739-741
  const solvedState = getSolvedState(store.getCubeType());
  if (result.state === solvedState) {
    sound.playSuccess();
  }
  ```

---

### M1【Medium・確認済み】`web/scene.ts`: キューブ切り替え時における `bodyMaterial` の破棄漏れと共有ジオメトリの重複 `dispose()`

- **該当箇所**:
  - [`web/scene.ts:373-424`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/scene.ts#L373-L424)
  - [`web/scene.ts:448-462`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/scene.ts#L448-L462)
- **確証度**: **確認済み**（Three.js オブジェクトライフサイクル追跡）
- **現象**:
  3x3 と 2x2 の表示切り替え時（`buildCube(cubeType)`）には、まず `clearCube()` が呼ばれて既存のメッシュ群が破棄されます。
  しかし、以下の 2 点のリソース管理上の問題が存在します。
  1. `buildCube()` の先頭（Line 375）で `this.bodyMaterial = new THREE.MeshStandardMaterial(...)` が毎回新しくインスタンス化されていますが、`clearCube()` では `this.bodyMaterial.dispose()` が呼ばれていません。そのため、キューブ種別を切り替えるたびに古い `MeshStandardMaterial` が GPU メモリ上にリークします。
  2. `buildCube()` 内で全ピース共通の角丸立方体ジオメトリ `const body = createRoundedBox(...)` を 1 つ生成し、全ピース（3x3 なら 26 個、2x2 なら 8 個）のメッシュに共有させています。ところが `clearCube()` 内では以下のように各ピースに対して無条件で `dispose()` を呼んでいます：
     ```typescript
     // web/scene.ts:452-458
     for (const p of this.pieces) {
       p.geometry.dispose(); // ← 同一の body ジオメトリに対して 26 回または 8 回連続で dispose() が走る！
       // ...
     }
     ```
- **影響**:
  Three.js において同一ジオメトリに対する多重 `dispose()` は無駄な内部クリーンアップ処理を誘発し、将来の Three.js バージョンアップ時に警告や予期せぬ例外の原因となります。また、`this.bodyMaterial` の破棄漏れは長時間の操作やタブ切り替えにおいて GPU メモリを徐々に浪費します。
- **推奨修正案**:
  `clearCube()` で `this.bodyMaterial` を適切に破棄し、ジオメトリの破棄は重複を排除して 1 度だけ行うように修正します。

---

### M2【Medium・確認済み】`web/main.ts`: 2x2x2 JSON ファイル読込時の無条件 `centersFromInput` 呼び出しとパリティ誤判定リスク

- **該当箇所**: [`web/main.ts:1173-1178`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/main.ts#L1173-L1178)
- **確証度**: **確認済み**（インポート処理の追跡）
- **現象**:
  JSON ファイルからキューブ状態を復元する `loadFromFile` 処理において、次のように書かれています：
  ```typescript
  // web/main.ts:1173-1178
  const centerData = centersFromInput(imported.state);
  if (centerData) {
    store.setCenters(centerData.centers);
    store.setCenterParity(centerData.parity);
  }
  ```
  インポートされたデータが 2x2x2（24 文字）である場合でも、無条件に `centersFromInput` が呼ばれています。
  2x2 にはセンターパーツが存在せず、センターの向きやパリティという概念自体が存在しません。現状の `centersFromInput` は 54 文字以外で `null` を返すため致命的な例外には至っていませんが、もし 2x2 のインポート時にストアに残存していた 3x3 のセンター情報がクリアされず引き継がれる潜在リスクがあります。
- **影響**:
  2x2 と 3x3 の状態管理の境界が曖昧になり、ファイルインポート時の整合性が損なわれるリスクがあります。
- **推奨修正案**:
  `if (store.getCubeType() === "3x3")` または `imported.cubeType === "3x3"` の場合のみセンター処理を実行し、2x2 の場合は確実にセンター状態をリセットします。

---

### M3【Medium・確認済み】`web/triggers.ts`: `PHASE_LABEL_MAP` のキー不一致による 2x2 フェーズ名マッピングの形骸化

- **該当箇所**: [`web/triggers.ts:39-48`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/triggers.ts#L39-L48)
- **確証度**: **確認済み**（Rust 側ソルバー出力と TypeScript 定数の突合）
- **現象**:
  `web/triggers.ts` では、ソルバーの各ステップに対応する日本語表示名として `PHASE_LABEL_MAP` を定義しています。
  ```typescript
  // web/triggers.ts:39-48
  export const PHASE_LABEL_MAP: Record<string, string> = {
    // 2x2 LBL (Rust: c2x2/lbl.rs)
    "ステップ 1: 完全1層 (First Layer)": "ステップ 1: 完全1層 (First Layer)",
    "ステップ 2: 上面色揃え (OLL)": "ステップ 2: 上面色揃え (OLL)",
    "ステップ 3: 上面位置揃え (PLL)": "ステップ 3: 上面位置揃え (PLL)",

    // 2x2 Ortega (Rust: c2x2/ortega.rs)
    "ステップ 1: 最初の1面 (First Face)": "ステップ 1: 最初の1面 (First Face)",
    "ステップ 2: 反対面色揃え (OLL)": "ステップ 2: 反対面色揃え (OLL)",
    "ステップ 3: 両層同時配置 (PBL)": "ステップ 3: 両層同時配置 (PBL)",
    // ...
  };
  ```
  しかし、Rust 側のソルバー（`src/c2x2/lbl.rs`, `src/c2x2/ortega.rs`）が実際に生成するフェーズ名文字列は、適用された公式名をカッコ付きで付与した以下のような動的文字列です：
  - `ステップ 2: 上面色揃え (OLL: Sune (スーネ))`
  - `ステップ 3: 上面位置揃え (PLL: T-perm (隣接交換))`
  - `ステップ 2: 反対面色揃え (OLL: Anti-Sune (アンチスーネ))`
  - `ステップ 3: 両層同時配置 (PBL: Adj-Adj (隣接-隣接))`
  そのため、`triggers.ts` 内での `PHASE_LABEL_MAP[phase]` による単純な等値検索は常に `undefined` となり、せっかく定義した日本語ラベルマップが全く利用されず、UI には英語の公式名がカッコに入った文字列がそのまま出力されてしまいます。
- **影響**:
  2x2x2 解法ステップの表示において、マップによる正規化や表示統一処理が完全に形骸化（デッドコード化）しています。
- **推奨修正案**:
  完全一致検索ではなく、プレフィックス判定（`phase.startsWith(key)`）を行うか、公式名のカッコ手前部分でマッチングするように `getPhaseLabel` ヘルパーを導入します。

---

### M4【Medium・確認済み】`tests/coverage.spec.ts`: アサーションを伴わない空 try/catch コードの残存

- **該当箇所**: [`tests/coverage.spec.ts:2634-2663`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/tests/coverage.spec.ts#L2634-L2663)
- **確証度**: **確認済み**（テストコード精査）
- **現象**:
  プロジェクト規約（`CLAUDE.md` §4）では「カバレッジの数値だけを上げるための、アサーションのないテストの追加は禁止する」と明記されています。
  しかし、`coverage.spec.ts` の末尾付近において、関数を呼び出して例外を空の `catch` ブロックで握りつぶし、何のアサーションも行わないコードが依然として残存しています。
- **影響**:
  テストスイートの信頼性を損ね、将来の不具合発生時に何も検知できない無意味な実行コストとなっています。
- **推奨修正案**:
  不要なダミー実行を削除するか、期待される戻り値や例外の型・メッセージを検証する明示的な `expect(...)` アサーションを追加します。

---

## 3. 軽微な指摘事項（Low）の詳細検証

### L1【Low】`web/editor.ts:6-13, 171`: 2x2 展開図エディタでの 3x3 センター前提ガイド文言
- **該当箇所**: [`web/editor.ts:6-13`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/editor.ts#L6-L13), [`:171`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/editor.ts#L171)
- **現象**: 2x2 エディタを開いた際、画面上部に「U面（上）に白、F面（前）に緑センターが来る向きです」と表示される。2x2 にはセンターが存在しないため、初見のユーザーが混乱する。
- **改善案**: 2x2 モード時は「U面（上）に白、F面（前）に緑が来る標準的な向きを基準として入力してください」等の適切な案内に切り替える。

### L2【Low】`web/main.ts:286`: `replace()` 実行時の `#solver-note` 3x3 固定上書き
- **該当箇所**: [`web/main.ts:286`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/main.ts#L286)
- **現象**: 解法アルゴリズムを切り替えた際、`#solver-note` が 3x3 前提の「通常5秒以内」で固定更新される。2x2 は全探索でも瞬時（ミリ秒単位）に完了するため、2x2 に適した説明文を設定すべきである。

### L3【Low】`web/view.ts:94`: ヘルプモーダルにおける 2x2 手数表記（QTM vs HTM）および LBL/Ortega 記載漏れ
- **該当箇所**: [`web/view.ts:94`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/view.ts#L94)
- **現象**: ヘルプダイアログで 2x2 の神の数字が「最大14手」(QTM) と記載されているが、本アプリのソルバーは半回転を 1 手と数える HTM（神の数字は最大11手）を採用しているため不整合がある。また、追加された LBL 法と Ortega 法の解説がヘルプに反映されていない。

### L4【Low】`web/main.ts:1464-1473`: 2x2 完成プリセット読込時の `#scramble-text` 未クリア
- **該当箇所**: [`web/main.ts:1464-1473`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/main.ts#L1464-L1473)
- **現象**: 2x2 の完成状態プリセットを選択した際、直前のスクランブル文字列が `#scramble-text` に残存したままになる。

### L5【Low】`src/lib.rs:596`: `get_orientations` の 2x2 入力時エラーメッセージの不親切さ
- **該当箇所**: [`src/lib.rs:596`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/src/lib.rs#L596)
- **現象**: `get_orientations` に 2x2（24 文字）が渡された際、「State must be 54 chars」とエラーが返る。マルチキューブ対応の API として「2x2 cube does not have center orientations」等の適切なメッセージを返すのが望ましい。

---

## 4. 結論および推奨アクションプラン

本レビューにより、最新コードベース（HEAD `4a010ac`）は高精度な群論アルゴリズムと強固な自動テストを備えつつも、**2x2x2 マルチキューブ化に伴うフロントエンドの接合部（URL パラメータ、共有リンク、永続化、効果音、3D リソース解放、フェーズ表示）において複数の重要な見落としが存在すること** が明らかになりました。

### 次期推奨アクション（優先度順）

1. **Phase 1: High 指摘事項の即時改修**
   - `web/url-params.ts` および `web/main.ts` で `lbl` / `ortega` をホワイトリストに追加し、URL・共有リンク・localStorage の同期を回復。
   - `web/main.ts` の手動回転完成判定を `getSolvedState(store.getCubeType())` に変更し、2x2 完成時の効果音再生を復旧。
2. **Phase 2: Medium 指摘事項の改修**
   - `web/scene.ts` で `clearCube()` 時に `this.bodyMaterial.dispose()` を実行し、共通ジオメトリの重複 `dispose()` を解消。
   - `web/main.ts` のファイル読込時に 2x2 のセンター処理をガード。
   - `web/triggers.ts` にプレフィックス対応の `getPhaseLabel` を導入し、2x2 の日本語フェーズ表示を復旧。
   - `tests/coverage.spec.ts` のアサーションなしコードを是正。
3. **Phase 3: Low 指摘事項の改修**
   - 2x2 エディタの文言、ヘルプダイアログの表記（HTM 11手、LBL/Ortega解説）、プリセット読込時の表示クリーンアップを実施。
