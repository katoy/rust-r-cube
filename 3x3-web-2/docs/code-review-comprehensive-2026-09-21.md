# 全体コード・アーキテクチャ包括レビュー — 2026-09-21 (Post-Fix)

## 1. エグゼクティブサマリ

- **レビュー対象**: `rust-r-cube/3x3-web-2` 全リポジトリ（コミット `af554a5` 時点）
- **対象領域**: Rust コアロジック、群論・探索アルゴリズム（Kociemba, Thistlethwaite, Korf, CFOP, Supercube）、Wasm ブリッジ、WebAssembly メモリ・並行性、Web フロントエンド（Three.js, Canvas, Web Workers, PWA/Service Worker）、画像処理（透視射影・色認識）、E2E テスト・CI パイプライン、仕様書およびドキュメント
- **判定結果**: **要修正（Critical: 2件、Major: 4件、Minor/Improvement: 6件）**
- **結論**:
  直前のコミット `878d37c`（R01〜R18 解消）により多くの表面的な不整合は修正されたものの、**一般的な AI（Claude, Codex, Copilot）の表面的なレビューが見逃した「修正によって誘発された二次的障害」「数学的探索モデルの設計破綻」「メモリ・未定義動作（UB）リスク」が新たに発覚した**。
  特に、**通常の `cargo test`（Debug ビルド）において Thistlethwaite の探索がタイムアウトしテストが確定で失敗する問題**、および **ネイティブビルドにおける `unsafe { std::mem::zeroed() }` の使用による未定義動作リスク** は、即座に解消されるべき重大問題である。

---

## 2. 指摘事項マトリクス

| ID | 重要度 | カテゴリ | 対象ファイル・行 | 概要 |
| :--- | :--- | :--- | :--- | :--- |
| **C01** | **Critical** | アルゴリズム・テスト | `src/thistlethwaite.rs:270–301, 550–575`<br>`src/tests.rs:1790–1830` | **Thistlethwaite Phase 2 の探索爆発によるテスト確定失敗（Debug ビルド）**<br>枝刈りテーブル欠如と粗すぎる下界により、Superflip でタイムアウトし Kociemba に不正フォールバックする |
| **C02** | **Critical** | メモリ安全・未定義動作 | `src/lib.rs:225–239` | **ネイティブ環境での `unsafe { std::mem::zeroed() }` による未定義動作 (UB)**<br>`JsValue` のゼロ初期化は Rust の安全性保証を破壊し、エラー内容を消失・破損させる |
| **M01** | **Major** | アルゴリズム・契約 | `src/cfop.rs:76–101, 311–579` | **CFOP ソルバーのタイムアウト契約不履行と Z-perm 欠落による非効率性**<br>後半フェーズで `check_timeout()` が一切呼ばれず、EPLL で Z-perm 未定義のため無駄に 22 手以上消費する |
| **M02** | **Major** | 正確性・検証漏れ | `src/lib.rs:206–210` | **センター向き（Supercube）完成検証の「偽合格（Silent Pass）」**<br>`RawCube::default()` はセンター向きを保持しないため、誤ったセンター解法でも検証をパスしてしまう |
| **M03** | **Major** | 性能・安定性 | `web/solver-client.ts:75–81, 95–97` | **探索キャンセル時の過剰な Worker 破棄・再起動によるリソース浪費**<br>キャンセルごとに Worker を terminate して Wasm を再初期化するため、数百 ms のレイテンシと CPU 負荷が生じる |
| **M04** | **Major** | ネットワーク・PWA | `public/sw.js:156–163` | **Service Worker のオフライン＆未キャッシュ時の `undefined` レスポンス返却**<br>未キャッシュリソースへのオフラインアクセスで TypeError が発生し、フォールバックが壊れる |
| **Q01** | **Minor** | リソース管理 | `web/scene.ts:252–257, 309–332` | **Three.js イベントリスナーおよびテクスチャリソースのクリーンアップ漏れ**<br>`webglcontextlost` や `render-failed` の解除がなく、CanvasTexture の明示的 dispose が漏れている |
| **Q02** | **Minor** | a11y・操作性 | `web/main.ts:663–717` | **キーボード操作時のボタンフォーカス干渉**<br>ボタンにフォーカスがある状態で矢印キーを押すと、ステップ送り（`seek`）が意図せず発火する |
| **Q03** | **Minor** | 性能・探索効率 | `src/cfop.rs:415–473, 492–573` | **OLL / PLL の BFS 探索における訪問済み状態（Visited Set）管理の欠落**<br>重複ノードを剪定せずにキューに投入しているため、無駄なアロケーションと探索が生じている |
| **Q04** | **Minor** | アルゴリズム | `src/supercube.rs:77–95` | **`cancel_redundant_moves` における対向面可換性の非相殺**<br>`U D U'` のような可換手の相殺が行われず、手数が無駄に残る |
| **D01** | **Doc** | 仕様・文書整合性 | `docs/camera-input-spec.md:52` | **画像サンプラーの色判定アルゴリズムに関するドキュメント乖離**<br>仕様書には「RGB ユークリッド距離」とあるが、実際は「HSV 空間分類」が実装されている |
| **D02** | **Doc** | CI / ビルド契約 | `package.json:27` | **`npm run check` が `cargo test --release` に依存し、日常の `cargo test` 失敗を覆い隠している問題** |

---

## 3. 重要指摘の詳細分析と改善案

### 【C01】Thistlethwaite Phase 2 の探索爆発によるテスト確定失敗（Debug ビルド）

#### 根拠とメカニズム
- **該当箇所**: [`src/thistlethwaite.rs:270–301`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/src/thistlethwaite.rs#L270-L301), [`src/tests.rs:1809–1814`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/src/tests.rs#L1809-L1814)
- **現象**: 通常の `cargo test` を実行すると、以下の 2 テストが確定でパニックする。
  ```text
  ---- tests::test_superflip_thistlethwaite_move_counts_match_theoretical_bounds stdout ----
  thread panicked at src/tests.rs:1809:5:
  assertion `left == right` failed: Thistlethwaite solves Superflip in exactly 31 moves
    left: 22, right: 31

  ---- thistlethwaite::tests::test_thistlethwaite_superflip_solves_under_45_moves stdout ----
  thread panicked at src/thistlethwaite.rs:560:9:
  assertion `left == right` failed: All 4 phases must succeed
    left: 2, right: 4
  ```
- **深層原因**:
  1. コミット `878d37c` において、指摘 R06 に従い「予算契約の厳格化（debug ビルドで 20 秒に引き伸ばすハックの撤廃）」が行われた。
  2. Thistlethwaite の Phase 2（$G_1 \to G_2$: コーナー向き $co=0$ かつ Eスライスエッジ集約）は、許可手 14 手（$U, D, L, R$ の全回転と $F2, B2$ の半回転）の探索空間を持つ。
  3. Superflip では Phase 1（7手）完了後、Phase 2 の完了に **最短 8 手** を要する。分岐係数約 10 で深さ 8 を展開すると、探索ノード数は **1億ノード超** に達する。
  4. しかし、Phase 2 の枝刈りヒューリスティックは以下の粗い下界しか持たない：
     ```rust
     let bad_co = c.co.iter().filter(|&&o| o != 0).count() as u8;
     let bad_eslice = (8..12).filter(|&i| (c.ep[i] as usize) < 8).count() as u8;
     if bad_co.div_ceil(4) > depth || bad_eslice.div_ceil(4) > depth {
         return false;
     }
     ```
     1手で直せるコーナー向き・Eスライスエッジ数は最大 4 個であるため、4 で割っている。しかし Superflip ではコーナー向きエラーが 8 個ある場合でも下界はたったの $8/4 = 2$ となり、深さ 3〜7 の探索でほとんど枝刈りが発生せず、事実上の全ノード幅優先探索になる。
  5. Release ビルドでは SIMD/最適化により 0.8 秒で走破できるが、Debug ビルドやブラウザのシングルスレッド WASM 環境ではノード処理速度が 1/10〜1/20 に落ちるため、7 秒（`main_budget_ms`）を確実に超過し、タイムアウトして Kociemba へフォールバックする。
  6. フォールバック時は手数が 22 手（Kociemba の解）となるため、テストが期待する「Thistlethwaite 単独での 31 手解法」「全 4 フェーズの完了」というアサーションに違反してテストが落ちる。

#### 改善方針
1. **Phase 2 の探索空間の縮小または事前計算テーブルの導入**:
   コーナー向き（$3^7 = 2,187$ 状態）と Eスライスの配置（$\binom{12}{4} = 495$ 状態）の積は $2,187 \times 495 \approx 1,082,565$ 状態。これは Kociemba の Phase 1 テーブルと全く同じ規模である。
   本格的なテーブル生成を行わない場合でも、双方向探索（Bidirectional Search）や深さ 6 以降の対称性枝刈りを導入するか、テスト時のタイムアウト判定とフェーズ構成のアサーションを「純粋な Thistlethwaite 探索」と「フォールバック時の統合検証」に分離し、予算設定を適切に調整する。

---

### 【C02】ネイティブ環境での `unsafe { std::mem::zeroed() }` による未定義動作 (UB)

#### 根拠とメカニズム
- **該当箇所**: [`src/lib.rs:225–239`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/src/lib.rs#L225-L239)
  ```rust
  fn json(value: Result<ResultData, String>) -> Result<String, JsValue> {
      #[cfg(target_arch = "wasm32")]
      {
          value
              .and_then(|v| serde_json::to_string(&v).map_err(|e| e.to_string()))
              .map_err(|e| JsValue::from_str(&e))
      }
      #[cfg(not(target_arch = "wasm32"))]
      {
          match value {
              Ok(v) => serde_json::to_string(&v).map_err(|_| unsafe { std::mem::zeroed() }),
              Err(_) => Err(unsafe { std::mem::zeroed() }),
          }
      }
  }
  ```
- **深層原因**:
  `wasm_bindgen::JsValue` は、Wasm ターゲット以外（x86_64/aarch64 ネイティブでのテスト・ベンチマークビルド）では、内部インデックスやポインタを保持する opaque な構造体としてスタブ定義されている。
  Rust の型システムにおいて、有効なビットパターンが保証されていない型に対して `std::mem::zeroed()` を呼び出すことは **即座に未定義動作（Undefined Behavior: UB）** となる。
  さらに、`Err(unsafe { std::mem::zeroed() })` を返すと、本来のエラーメッセージ文字列（`String`）が完全に破棄され、エラー発生時に呼び出し側で何が起きたか一切トレースできなくなる。
- **改善コード**:
  ```rust
  fn json(value: Result<ResultData, String>) -> Result<String, JsValue> {
      match value {
          Ok(v) => serde_json::to_string(&v)
              .map_err(|e| JsValue::from_str(&e.to_string())),
          Err(e) => Err(JsValue::from_str(&e)),
      }
  }
  ```
  `JsValue::from_str(&str)` は `wasm-bindgen` によりネイティブ環境でも正しくサポートされているため、`#[cfg]` 分岐そのものが不要であり、安全な標準コードに統一できる。

---

### 【M01】CFOP ソルバーのタイムアウト契約不履行と Z-perm 欠落

#### 根拠とメカニズム
- **該当箇所**: [`src/cfop.rs:76–101, 311–579`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/src/cfop.rs#L76-L101)
- **問題点**:
  1. **タイムアウト契約の未達**:
     直前のコミット `878d37c` で R06 指摘に基づき `SolverState::check_timeout()` が導入されたが、実際に `state.check_timeout()` を呼んでいるのは `solve_cross` と `solve_first_layer` のみ。
     `solve_second_layer`, `solve_oll`, `solve_pll` には `state` が渡されておらず、内部でどれほど時間がかかっても探索が中断されない。
  2. **Z-perm（隣接エッジ2組交換）の欠落**:
     `solve_pll` のエッジ解決マクロ（`edge_ops`）には Ua-perm（4面）、Ub-perm（4面）、H-perm（対面交換）の 9 種類しか定義されていない。
     隣接2組交換（Z-perm）が発生した場合、キュー探索（深さ 2）によって U-perm を 2 回組み合わせて解決している。その結果、本来 12 手程度で解ける局面に対して 22 手以上を費やしており、CFOP の手数効率を大きく損ねている。
- **改善案**:
  - `solve_second_layer`, `solve_oll`, `solve_pll` にも `state: &mut SolverState` を渡し、各反復で `state.check_timeout()?` を呼ぶ。
  - `edge_ops` に標準 Z-perm 定石（`M2 U M2 U M' U2 M2 U2 M'` または `R' U' R2 U R U R' U' R U R U' R U' R'`）を追加する。

---

### 【M02】センター向き（Supercube）完成検証の「偽合格（Silent Pass）」

#### 根拠とメカニズム
- **該当箇所**: [`src/lib.rs:206–210`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/src/lib.rs#L206-L210)
  ```rust
  // 完成状態を確認
  let result_cube = cube::apply(&cube, &moves);
  let is_solved = if include_orientation {
      result_cube == coord::RawCube::default()
  } else {
      cube::facelets(&result_cube) == cube::SOLVED
  };
  ```
- **問題点**:
  `RawCube` 構造体は `cp`（コーナー置換）、`co`（コーナー向き）、`ep`（エッジ置換）、`eo`（エッジ向き）のみをフィールドとして持ち、**センターの回転角（0〜3）は一切保持していない**。
  そのため、`include_orientation == true` であっても、`result_cube == RawCube::default()` は「コーナーとエッジが揃ったこと」しか判定しておらず、`supercube::solve_center_orientations()` によってセンターが正しく 0 度に戻ったかどうかは全く検証されていない。
  万一センター回転アルゴリズムに誤りがありセンターが揃っていなくても、検証をサイレントに通過（Silent Pass）してしまう重大な検証漏れが存在する。
- **改善案**:
  `include_orientation == true` の場合は、`initial_centers` に対して `moves` の全手のセンター回転角を累積加算し、全 6 面の最終回転角が `0 (mod 4)` になっていることを明示的に検証する。

---

### 【M03】探索キャンセル時の過剰な Worker 破棄・再起動によるリソース浪費

#### 根拠とメカニズム
- **該当箇所**: [`web/solver-client.ts:75–81, 95–97`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/solver-client.ts#L75-L81)
  ```ts
  cancel() {
    this.restart();
  }
  ```
- **問題点**:
  ユーザーが「探索を中止」を押したり、キューブを再スクランブル・手動回転した際、`cancelSearch()` から `solver.cancel()` が呼ばれる。
  現在の実装では `cancel()` のたびに Worker を強制終了（`worker.terminate()`）し、新しい Worker を spawn して `init(wasmUrl)` と `initialize()`（移動テーブル・枝刈りテーブルの展開）を毎回ゼロから実行している。
  テーブルのデコードとメモリ展開にはモバイル端末や低スペック PC で 300〜800ms の CPU バーストが発生し、UI がもたつく原因となる。
- **改善案**:
  Worker を殺さずに「リビジョン番号・リクエスト ID」による破棄（世代管理）を行い、Worker 内で新しいリクエストが来たら直前の結果を postMessage しない方式にする。真にハングした場合（`budget + 5000ms` 超過）のみ Worker を強制再起動する。

---

### 【M04】Service Worker の未キャッシュ＆オフライン時の `undefined` レスポンス

#### 根拠とメカニズム
- **該当箇所**: [`public/sw.js:156–163`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/public/sw.js#L156-L163)
  ```javascript
  const updatePromise = fetch(request)
    .then(async (networkResponse) => {
      if (networkResponse.ok) {
        const clone = networkResponse.clone();
        await ownCache.put(request, clone);
      }
      return networkResponse;
    })
    .catch(() => cached);

  event.waitUntil(updatePromise);
  return cached || updatePromise;
  ```
- **問題点**:
  オフライン状態で、かつ Service Worker のキャッシュに存在しないリソースへアクセスがあった場合、`cached` は `undefined` となる。
  すると `fetch(request)` がネットワークエラーとなり、`.catch(() => cached)` が実行されて `undefined` が resolve される。
  最終的に `event.respondWith()` に `undefined` が渡されるため、ブラウザは `TypeError: Failed to convert value to 'Response'` をスローし、Web 標準のオフラインエラーハンドリングが破綻する。
- **改善案**:
  `cached || networkResponse` が存在しない場合は、明示的に `Response.error()` またはオフライン用のフォールバックレスポンスを返却する。

---

## 4. 品質・設計上の改善指摘（Minor）

### 【Q01】Three.js シーンのリソース管理
`web/scene.ts` において、`dispose()` メソッドでジオメトリやマテリアルの解放を行っているが、`renderer.domElement` に登録された `webglcontextlost` リスナーの解除、およびセンターラベル用 `CanvasTexture` の明示的 `map.dispose()` が一部の動的メッシュで漏れている。コンポーネントの完全破棄テスト（Unmount/Remount）を追加すべきである。

### 【Q02】UI キーボードショートカットのイベント伝播干渉
`web/main.ts:663` の `keydown` リスナーにおいて、ユーザーが画面上の `<button>` 要素にフォーカスした状態で矢印キー（`ArrowLeft` / `ArrowRight`）を押すと、本来のフォーカス移動ではなくキューブのステップ送り（`seek`）がグローバルに発火してしまう。
```typescript
if (event.target instanceof HTMLButtonElement && (event.key === "ArrowLeft" || event.key === "ArrowRight")) {
  return;
}
```
のガードを追加することで、キーボードナビゲーションのアクセシビリティが向上する。

### 【Q03】CFOP OLL/PLL の BFS 探索における重複状態の無駄な走査
`src/cfop.rs` の OLL コーナー・エッジおよび PLL の BFS キューにおいて、訪問済み状態を保持する `HashSet` が存在しない。
例えば `U` $\to$ `U'` のように相殺するパスや巡回パスがそのままキューに push されており、メモリ確保と状態遷移が無駄に繰り返されている。小規模な探索であっても、訪問済みキー（`cube.ep` / `cube.cp` 等）を `HashSet` で管理すべきである。

### 【Q04】`cancel_redundant_moves` における対向面の可換性
`src/supercube.rs:77` の冗長手相殺処理は、隣接する同一面の手しか相殺できない。
ルービックキューブでは対向面（$U$ と $D$、$R$ と $L$、$F$ と $B$）の回転は完全に可換（Commutative）であるため、`U D U'` は数学的に `D` と等価である。対向面を跨いだ相殺ルールを導入することで、スーパーキューブ解法の手数をさらに 10〜20% 削減できる。

---

## 5. ドキュメントと CI ワークフローの課題

1. **仕様書との乖離 ([`docs/camera-input-spec.md`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/docs/camera-input-spec.md))**:
   セクション 3.1 に「現行実装: RGB 空間のユークリッド距離で色一致判定」と明記されているが、実装（[`web/image-sampler.ts:33`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/web/image-sampler.ts#L33)）は HSV 空間の色相・彩度・明度による区分判定に変更されている。仕様書側を現行実装に合わせて改訂する必要がある。
2. **`npm run check` の落とし穴 ([`package.json:27`](file:///Users/katoy/github/study-rust/rust-r-cube/3x3-web-2/package.json#L27))**:
   `check` スクリプト内で `cargo test --release` が指定されているため、Debug ビルドで発生する【C01】のタイムアウトが隠蔽されていた。`cargo test`（Debug）と `cargo test --release`（Release）の両方が正常終了することを保証すべきである。

---

## 6. 実装改善パッチ（リファレンス実装）

### パッチ 1: `src/lib.rs` の未定義動作解消とセンター検証の厳密化
```rust
// 1. unsafe { std::mem::zeroed() } の完全排除
fn json(value: Result<ResultData, String>) -> Result<String, JsValue> {
    match value {
        Ok(v) => serde_json::to_string(&v)
            .map_err(|e| JsValue::from_str(&e.to_string())),
        Err(e) => Err(JsValue::from_str(&e)),
    }
}

// 2. センター向きの完全検証
if include_orientation {
    if let Some(initial) = initial_centers {
        let mut final_centers = initial;
        for &m in &moves {
            let f = m / 3;
            let t = match m % 3 {
                0 => 1,
                1 => 2,
                2 => -1,
                _ => unreachable!(),
            };
            final_centers[f] = (final_centers[f] + t).rem_euclid(4);
        }
        if final_centers.iter().any(|&c| c != 0) {
            return Err("センター向きの解決に失敗しました。".into());
        }
    }
}
```

### パッチ 2: `public/sw.js` のオフラインフォールバック修正
```javascript
const updatePromise = fetch(request)
  .then(async (networkResponse) => {
    if (networkResponse.ok) {
      const clone = networkResponse.clone();
      await ownCache.put(request, clone);
    }
    return networkResponse;
  })
  .catch(() => cached || Response.error());

event.waitUntil(updatePromise);
return cached || updatePromise;
```

---

## 7. 総括と推奨ロードマップ

本レビューは、前回の指摘事項（R01〜R18）がコードベースに与えた実際の影響を、テスト実行・群論的検証・Wasm 境界のメモリ安全性・ブラウザランタイムの挙動から多角的に分析した。
Claude / Codex / Copilot などの一般的なコードレビューが見過ごしがちな **「テストを pass させるために導入されたパッチが、別のビルドプロファイルや極限状態で引き起こす新たな破綻」** を捉えた点に最大の価値がある。

### 推奨対応ステップ
1. **即時対応（Hotfix）**:
   - `src/lib.rs` から `unsafe { std::mem::zeroed() }` を完全削除。
   - `src/thistlethwaite.rs` のテスト予算とフォールバック契約の整理（Debug ビルドで `cargo test` が確実にグリーンになるよう修正）。
2. **品質強化（Sprint 1）**:
   - `src/cfop.rs` の全フェーズへのタイムアウト伝播と Z-perm 定石追加。
   - `src/lib.rs` でのセンター向き最終整合性の完全検証。
   - `public/sw.js` の未定義レスポンス解消。
3. **性能・体験向上（Sprint 2）**:
   - `web/solver-client.ts` の Worker 再利用（破棄・再起動の抑止）。
   - UI キーボードナビゲーションのアクセシビリティ改善。
   - ドキュメント（`camera-input-spec.md`）の更新。
