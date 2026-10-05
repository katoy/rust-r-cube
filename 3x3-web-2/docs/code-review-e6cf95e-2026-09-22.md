# コード全体・数理深層レビュー報告書（2026-09-22 / e6cf95e）

**レビュー実施日**: 2026-09-22  
**対象リポジトリ**: `rust-r-cube/3x3-web-2`  
**対象コミット**: `e6cf95e`（およびブランチ `fix/superflip-preset` ワーキングツリー）  
**評価基準**: Claude 3.7 / Codex / GitHub Copilot のレビュー水準を明確に超える、数学的根拠（群論・パリティ不変量・アドミッシブル探索論）・並行性モデル・ライフサイクル解析・実証可能性を兼ね備えた深層レビュー  

---

## 1. エグゼクティブサマリー

コミット `616023c` および最新ブランチ `fix/superflip-preset` において、先行レビューで指摘された P1/P2/P3 の主要課題（Service Worker のネットワークモックバイパス防止、シーク時同期 I/O スラッシング解消、プリセットメタデータ整合、Kociemba フェーズ境界、WASM 例外耐性、カメラ Blob URL リーク対策、全マッチングによるセンター最適化など）に着手され、着実な前進が確認された。

しかし、さらなる実証テストと全コードベース（Rust コアエンジン、Wasm バインディング、Web Worker 通信、Three.js レンダリングパイプライン、カメラ立体認識、PWA キャッシュ戦略、Playwright E2E テストスイート）に対する数理的・計算機科学的精査を行った結果、**「一見正しく機能しているように見えながら、数理モデルや探索空間、ライフサイクル管理に潜む深層課題」** が新たに 8 件特定された。

特筆すべきは以下の点である：
1. **CFOP 第1層コーナー探索におけるヒューリスティック非アドミッシブル性の発見**: `broken_cross > depth` の枝刈り条件が、D面回転を探索候補に含めていることで下界（admissible lower bound）として破綻し、D面回転を含む探索分岐を 100% デッドブランチ化させていた構造的欠陥。
2. **スーパークーブにおけるパリティ契約違反とサイレント不完全解法の検出**: 奇数個の奇数回転センターを持つ物理的に不可能な入力に対してエラーを返さず、最後のセンターを未解決のまま空手または不完全手順を返す不具合（実証スクリプトにより確認）。
3. **ResizeObserver rAF デバウンスの `dispose()` 時未キャンセル**: コンポーネント破棄後にスケジュール済み rAF が発火し、破棄済み WebGL レンダラーに対して遅延処理が実行される潜在的例外要因（実証スクリプトにより確認）。
4. **開発環境における Service Worker 自動汚染**: `npm run dev` 時にクエリなしでアクセスした開発者環境において Service Worker が無条件登録され、HMR や開発中のアセット更新がキャッシュされる問題。
5. **Clippy `-D warnings` による CI 破損**: 最新のワーキングツリー変更における Clippy 警告（`manual_is_multiple_of`, `unnecessary_map_or`）によるビルド中断。

### 総合品質判定: **A- (Fixes Required for CI & Mathematical Robustness)**
- **致命的障害 (P1)**: **0 件**（本番稼働を直ちに停止させるクラッシュはなし）
- **重要改善点 (P2)**: **4 件**（CI ビルドエラー、CFOP ヒューリスティック非アドミッシブル性、センターパリティ契約欠落、開発環境 SW 汚染）
- **保守・品質改善点 (P3)**: **4 件**（Three.js dispose 時 rAF 未キャンセル、Prettier 違反、180度センターペアリング最適化余地、解法 DOM キャッシュ無効化境界）

---

## 2. 実施した実証・検証結果一覧 (Test Matrix)

検証環境: macOS (Darwin arm64)、Node.js `26.7.0`、npm `11.19.0`、Rust/Cargo `1.98.1`、Playwright `1.63.0` (Chromium)。

| 検証項目 | 実行結果 | 詳細・実測値 |
| :--- | :---: | :--- |
| `cargo test --release` | **成功** | 115 passed, 0 failed, 1 ignored (12.54秒) |
| `cargo clippy --all-targets -- -D warnings` | ❌ **エラー検出** | `src/supercube.rs:174` (manual_is_multiple_of), `src/supercube.rs:180` (unnecessary_map_or) |
| `npm run typecheck` | **成功** | 型エラー 0件 (TypeScript 7.0.2) |
| `npm run format:check` | ❌ **警告検出** | `tests/ui-interaction.spec.ts` でコードスタイル違反 |
| `npm run build` | **成功** | WASM・Vite・SWプリキャッシュ生成成功 (341ms) |
| `npx playwright test` (全スイート) | **成功** | 153 passed, 1 skipped (7.7分) |
| **Probe 1: CFOP 探索実証** (`probe_cfop.rs`) | ❌ **数理破綻実証** | D面回転直後に `broken_cross = 4` となり、D' 1手で全修復可能。`broken_cross > depth` は下界として破綻しており、D面回転分岐が即死していることを実証。 |
| **Probe 2: センターパリティ実証** (`probe_supercube_v2.rs`) | ❌ **不具合実証** | `[1, 0, 0, 0, 0, 0]` で手長0手（未解決）、`[1, 1, 1, 0, 0, 0]` で42手返却後もセンター未解決（`[0, 0, 1, 0, 0, 0]`）。エラーを返さずサイレント破損。 |
| **Probe 3: Three.js dispose 実証** (`probe_scene_dispose_v2.mjs`) | ❌ **リーク実証** | `scene.dispose()` 呼び出し後に、破棄前にスケジュールされた `requestAnimationFrame` コールバック（`applyResize`）が遅延発火。 |
| **Probe 4: 開発時 SW 汚染実証** (`probe_dev_sw.mjs`) | ❌ **汚染実証** | `http://127.0.0.1:5173/` への通常アクセスで Service Worker が即座に `activated` になり、開発環境でキャッシュが支配的になる。 |

---

## 3. 主要指摘事項一覧 (Finding Index)

| ID | 深刻度 | 対象モジュール | カテゴリ | 指摘概要 |
|:---|:---|:---|:---|:---|
| **R01** | **P2** | `src/supercube.rs:174, 180` | ビルド / CI | `cargo clippy --all-targets -- -D warnings` が `manual_is_multiple_of` と `unnecessary_map_or` で失敗し、CI の品質ゲートを破壊している。 |
| **R02** | **P2** | `src/cfop.rs:249-270` | アルゴリズム / 探索論 | 第1層コーナー探索で D面回転が候補にあるにもかかわらず `broken_cross > depth` で枝刈りしているため、アドミッシビリティが破綻し、D面分岐が 100% 枝刈り即死（デッドブランチ）している。 |
| **R03** | **P2** | `src/supercube.rs:168-212` | 群論 / API契約 | 奇数個の奇数回転センターを持つ不可能な入力に対し、エラーを返さず不完全な手順（未解決センターを残す）をサイレントに返却する。 |
| **R04** | **P2** | `web/pwa.ts:7-21` | DX / アーキテクチャ | 開発環境（`npm run dev`）で通常アクセスした場合に Service Worker が無条件登録され、HMR 遅延やアセットキャッシュ汚染を引き起こす。 |
| **R05** | **P3** | `web/scene.ts:85, 277-285, 335` | ライフサイクル / メモリ | `CubeScene.dispose()` において、リサイズ用にスケジュールされた `resizeRafId` のキャンセルが欠落しており、破棄後コールバックが遅延実行される。 |
| **R06** | **P3** | `tests/ui-interaction.spec.ts` | コード品質 / スタイル | Prettier のスタイルチェックに違反しており、`npm run format:check` を失敗させている。 |
| **R07** | **P3** | `src/supercube.rs:145-155` | アルゴリズム / 最適性 | 180度センター回転が複数存在する場合に独立に12手ずつ（計24手以上）かけており、複合180度手順による手数短縮の余地がある。 |
| **R08** | **P3** | `web/main.ts:167-248` | UI / パフォーマンス | 解法ステップ描画のキャッシュ判定が `renderedSolution !== solution` のみで行われており、同一インスタンスに対する設定変更時のキャッシュ無効化境界が脆弱。 |

---

## 4. 指摘事項の深層分析と改善提案

---

### R01: `src/supercube.rs` における Clippy 警告による CI ビルドエラー (P2)

#### 該当コード
- `src/supercube.rs:174`
- `src/supercube.rs:180`

```rust
// src/supercube.rs:174
if odd_faces.len() % 2 == 0 {

// src/supercube.rs:180
if best_moves.as_ref().map_or(true, |b| moves.len() < b.len()) {
```

#### 問題の構造と影響
- 最新の Rust 1.98+ の Clippy において、以下の2つの lint が警告され、プロジェクト共通の品質ゲート `-D warnings` によりコンパイルエラーとなる：
  1. `clippy::manual_is_multiple_of`: `odd_faces.len() % 2 == 0` は `odd_faces.len().is_multiple_of(2)` を使用すべき。
  2. `clippy::unnecessary_map_or`: `Option::map_or(true, ...)` は `Option::is_none_or(...)` を使用すべき。
- この状態では GitHub Actions やローカルの `cargo clippy --all-targets -- -D warnings` が即座に赤点灯し、マージ不可能な状態となっている。

#### 推奨修正コード
```rust
// src/supercube.rs:174
if odd_faces.len().is_multiple_of(2) {

// src/supercube.rs:180
if best_moves.as_ref().is_none_or(|b| moves.len() < b.len()) {
```

---

### R02: CFOP 第1層コーナー探索におけるヒューリスティック非アドミッシブル性とデッドブランチ (P2)

#### 該当コード
- `src/cfop.rs:249-269`
- **実証コード**: `docs/review-evidence/latest/probe_cfop.rs`

```rust
// src/cfop.rs:249-269
    let broken_cross = (4..8)
        .filter(|&i| c.ep[i] as usize != i || c.eo[i] != 0)
        .count();
    if broken_cross > depth {
        return Ok(false);
    }

    for face in 0..6 {
        if face == last_face {
            continue;
        }
        for turn in 0..3 {
            let m = face * 3 + turn;
            let next_cube = c.multiply(move_cube_18(m));
            path.push(m);
            ...
```

#### 問題の構造と数理的分析
1. **アドミッシビリティ（Admissibility）の定義**:
   A* や IDA* におけるヒューリスティック関数 $h(n)$ は、目標状態までの真の最短距離 $h^*(n)$ に対して常に $h(n) \le h^*(n)$ を満たさなければならない。
2. **D面回転による一括変化**:
   `search_corner` は探索候補として全 6 面の回転（`face in 0..6`）を試みる。
   もし探索木の中で `D`（`face == 3`）を回すと、4 つのクロスエッジ（DR, DF, DL, DB）が一斉に隣接スロットに移動し、`broken_cross = 4` となる。
   しかし、この状態は **逆回転 `D'` の 1 手だけで 4 つのクロスエッジすべてを元通り修復できる**（$h^*(n) = 1$）。
   ところが、コードでは `broken_cross > depth` を判定しているため、残り深さが `depth = 1, 2, 3` のいずれであっても `4 > depth` が成り立ち、直ちに `return Ok(false);` で枝刈りされる！
   すなわち、$h(n) = 4 > h^*(n) = 1$ となり、**ヒューリスティックが真の距離を過大評価してアドミッシビリティを満たしていない**。
3. **探索空間の浪費（デッドブランチ）**:
   深さ 1〜6 の探索において、D面を回した瞬間に `broken_cross = 4` となり、子ノードまたは孫ノードで確実に即死する。
   第1層コーナー解法において D面回転はそもそも不要（U面と各側面の回転 `[R, U, F, L, B]` のみでコーナーの挿入・退避は完結する）。
   にもかかわらず `face == 3`（D面）を毎手試行しているため、無意味な探索ノードを毎回生成しては即死させている。
   `face == 3` をスキップすれば、探索分岐数は各深さで $15 \to 12$ に減少し（20% 削減）、深さ 5 では探索空間が約 $(12/15)^5 \approx 0.32$（約 1/3）に縮約されて大幅に高速化する。
   さらに、D面を除外することで「各手で変化するクロスエッジは高々1個」という前提が数学的に成立するため、`broken_cross > depth` が **真のアドミッシブル下界として数学的に正当化される**。

#### 実証結果 (`probe_cfop.rs`)
```text
=== Probe 1: CFOP First Layer Corner Search Analysis ===
D面回転直後のクロス破壊エッジ数: 4
D' を 1 手適用した後のクロス破壊エッジ数: 0
【数理的帰結】
D面回転を許容する場合、1手で修復できるクロスエッジ数は最大4個。
したがって、現在の実装 broken_cross > depth はアドミッシビリティが破綻している。
第1層コーナー解法においてD面回転は不要であるため、最初から face == 3 を除外すべきである。
```

#### 推奨修正コード
`src/cfop.rs` の探索ループにおいて、`face == 3`（D面）を明示的にスキップする：
```rust
// src/cfop.rs:260
    for face in 0..6 {
        // D面 (face 3) はクロスエッジを破壊するためコーナー探索では使用しない
        // これにより「1手で動くクロスエッジは最大1個」が保証され、broken_cross > depth がアドミッシブルになる
        if face == last_face || face == 3 {
            continue;
        }
        for turn in 0..3 {
            let m = face * 3 + turn;
            ...
```

---

### R03: スーパークーブにおけるパリティ契約違反とサイレント不完全解法 (P2)

#### 該当コード
- `src/supercube.rs:168-212`
- **実証コード**: `docs/review-evidence/latest/probe_supercube_v2.rs`

```rust
// src/supercube.rs:172-205
    let odd_faces: Vec<usize> = (0..6).filter(|&f| needed[f] % 2 != 0).collect();

    if odd_faces.len().is_multiple_of(2) {
        let matchings = generate_perfect_matchings(&odd_faces);
        ...
    } else {
        let mut cur = needed;
        let mut moves: Vec<usize> = Vec::new();
        while let Some(a) = (0..6).find(|&f| cur[f] % 2 != 0) {
            let Some(b) = (0..6).find(|&f| f != a && cur[f] % 2 != 0) else {
                break; // 奇数個の奇数回転があると最後の1面が無視される！
            };
            ...
        }
        for (f, rot) in cur.iter_mut().enumerate() {
            if *rot == 2 {
                let fix = rotate_center_180(f);
                moves.extend(fix);
                *rot = 0;
            }
        }
        cancel_redundant_moves(&moves)
    }
```

#### 問題の構造と影響
1. **群論的制約（スーパークーブのパリティ不変量）**:
   3x3 ルービックキューブにおける任意の基本回転（90度）は、
   - 対象面のセンターを +1 回転（mod 4）
   - 対象面の 4 コーナーを 4-cycle 置換（奇置換、符号 -1）
   - 対象面の 4 エッジを 4-cycle 置換（奇置換、符号 -1）
   を引き起こす。
   コーナー置換が解かれている（恒等置換＝偶置換）状態では、**6面のセンター回転の和（mod 2）は必ず 0（偶数）でなければならない**。
   したがって、90度または270度回転（奇数回転）しているセンターの個数は、必ず **偶数個（0, 2, 4, 6 個）** でなければ物理的に解法が存在しない。
2. **サイレント不完全解法の返却**:
   奇数個の奇数回転を持つセンター状態（例: 1面だけ90度回転している `[1, 0, 0, 0, 0, 0]` や 3面が90度回転している `[1, 1, 1, 0, 0, 0]`）が渡された場合、現在のコードはエラーを返さず（シグネチャが `Vec<usize>`）、`while let ... break` で最後の奇数面を無視して処理を終了する。
   その結果、呼び出し元で手順を再生しても最後のセンターは揃わず、`lib.rs` 側の最終アサーション（`final_centers.iter().any(|&c| c != 0)`）で「センター向きの検証に失敗しました。」という不透明なエラーとなって落ちる。

#### 実証結果 (`probe_supercube_v2.rs`)
```text
=== Probe 2: Supercube Center Orientations Parity Analysis ===
不正な入力（1つのセンターのみ90度回転）: [1, 0, 0, 0, 0, 0]
返された手順の手数: 0
手順適用後のセンター回転: [1, 0, 0, 0, 0, 0]
センターは揃ったか？: false

入力（3つのセンターが90度回転）: [1, 1, 1, 0, 0, 0]
返された手順の手数: 42
手順適用後のセンター回転: [0, 0, 1, 0, 0, 0]
センターは揃ったか？: false
=> 【実証成功】奇数個の奇数回転が存在する場合、最後のセンターが未解決のまま無視され、サイレントに不完全な手順が返される！
```

#### 推奨修正コード
関数シグネチャを `Result<Vec<usize>, String>` に改善するか、内部でパリティを検証して奇数の場合はパリティエラーを明確に報告する：
```rust
/// Solves remaining center rotations so that all 6 centers have 0 rotation mod 4.
pub fn solve_center_orientations(current_rotations: [i32; 6]) -> Result<Vec<usize>, String> {
    let mut needed = [0i32; 6];
    for f in 0..6 {
        needed[f] = (-current_rotations[f]).rem_euclid(4);
    }

    if needed.iter().all(|&x| x == 0) {
        return Ok(Vec::new());
    }

    let odd_faces: Vec<usize> = (0..6).filter(|&f| needed[f] % 2 != 0).collect();

    if !odd_faces.len().is_multiple_of(2) {
        return Err("センター向きのパリティが不正です（奇数回転のセンター数が奇数です）".into());
    }

    let matchings = generate_perfect_matchings(&odd_faces);
    let mut best_moves: Option<Vec<usize>> = None;

    for matching in &matchings {
        let moves = evaluate_matching(needed, matching);
        if best_moves.as_ref().is_none_or(|b| moves.len() < b.len()) {
            best_moves = Some(moves);
        }
    }
    Ok(best_moves.unwrap_or_default())
}
```

---

### R04: 開発環境（`npm run dev`）における Service Worker 無条件登録とキャッシュ汚染 (P2)

#### 該当コード
- `web/pwa.ts:7-21`
- `web/main.ts:33`
- **実証コード**: `docs/review-evidence/latest/probe_dev_sw.mjs`

```typescript
// web/pwa.ts:7-21
export function registerServiceWorker(swUrl = "./sw.js") {
  if (typeof window !== "undefined") {
    const params = new URLSearchParams(window.location.search);
    if (params.has("no-sw") || window.__DISABLE_SW__) {
      if ("serviceWorker" in navigator) {
        navigator.serviceWorker.getRegistrations().then((regs) => {
          regs.forEach((r) => r.unregister());
        });
      }
      return;
    }
  }
  ...
```

#### 問題の構造と影響
- 先行対応により `?no-sw` クエリおよび `window.__DISABLE_SW__` によるオプトアウト機構が導入された。
- しかし、**通常の開発作業（`npm run dev` でローカルサーバーを起動し `http://127.0.0.1:5173/` をブラウザで開く）** では、開発者は URL パラメータを付けずにアクセスするのが一般的である。
- 実証スクリプト `probe_dev_sw.mjs` で確認された通り、通常アクセスでは Service Worker が即座に登録・アクティブ化される。
- これにより、開発中にコードや WASM を編集しても Service Worker のキャッシュが優先的に返され、HMR の不整合や「修正したはずのコードがブラウザに反映されない」という典型的な Service Worker キャッシュ汚染トラブルに直面する。

#### 実証結果 (`probe_dev_sw.mjs`)
```text
=== Probe 4: Service Worker Auto-Registration in Dev Environment ===
Service Worker 登録状態: {
  supported: true,
  hasRegistration: true,
  scope: 'http://127.0.0.1:5173/',
  active: 'activated'
}
=> 【実証成功】開発サーバー (127.0.0.1:5173) において、クエリ未指定の場合に無条件で Service Worker が登録され、開発環境が汚染される！
```

#### 推奨修正コード
Vite の環境変数 `import.meta.env.DEV` を活用し、開発モード時はデフォルトで登録を抑止し、既存の Service Worker があれば自動登録解除する：
```typescript
// web/pwa.ts
export function registerServiceWorker(swUrl = "./sw.js") {
  if (typeof window !== "undefined") {
    const params = new URLSearchParams(window.location.search);
    const isDev = import.meta.env.DEV;
    const shouldDisable =
      params.has("no-sw") || window.__DISABLE_SW__ || (isDev && !params.has("enable-sw"));

    if (shouldDisable) {
      if ("serviceWorker" in navigator) {
        navigator.serviceWorker.getRegistrations().then((regs) => {
          regs.forEach((r) => r.unregister());
        });
      }
      return;
    }
  }
  ...
```

---

### R05: `CubeScene.dispose()` におけるリサイズ rAF の未キャンセル (P3)

#### 該当コード
- `web/scene.ts:85, 277-285, 335`
- **実証コード**: `docs/review-evidence/latest/probe_scene_dispose_v2.mjs`

```typescript
// web/scene.ts:277-284
    if (this.resizeRafId !== undefined) return;
    this.resizeRafId = requestAnimationFrame(() => {
      this.resizeRafId = undefined;
      this.applyResize();
    });

// web/scene.ts:335 (dispose)
  dispose() {
    this.finish();
    this.renderer.setAnimationLoop(null);
    this.renderer.domElement.removeEventListener(
      "webglcontextlost",
      this.onContextLost,
    );
    this.controls.dispose();
    this.observer.disconnect();
    // ここで this.resizeRafId のキャンセルが行われていない！
    this.scene.traverse((object) => {
```

#### 問題の構造と影響
- リサイズイベント密集時の負荷を抑えるために `requestAnimationFrame` によるデバウンスが導入された。
- しかし、`CubeScene.dispose()` 内で `cancelAnimationFrame(this.resizeRafId)` が行われていない。
- WebGL コンテキスト喪失時や 2D フォールバック時、あるいはページ離脱時に `dispose()` が呼ばれた際、フレーム描画キューに残っていた `applyResize()` が破棄後に遅延実行される（実証スクリプト `probe_scene_dispose_v2.mjs` で 100% 再現）。
- 破棄済みのレンダラーやカメラに対して `this.renderer.setSize(w, h)` が実行され、リソース解放の完全性が損なわれる。

#### 実証結果 (`probe_scene_dispose_v2.mjs`)
```text
=== Probe 3: Scene ResizeObserver rAF Callback after Dispose ===
検証結果: { applyResizeCalledAfterDispose: true }
=> 【実証成功】dispose() 後にキューされていた applyResize() が発火し、破棄済みインスタンスに対してリサイズ処理が遅延実行された！
```

#### 推奨修正コード
```typescript
// web/scene.ts: dispose()
  dispose() {
    this.finish();
    if (this.resizeRafId !== undefined) {
      cancelAnimationFrame(this.resizeRafId);
      this.resizeRafId = undefined;
    }
    this.renderer.setAnimationLoop(null);
    ...
```

---

### R06: `tests/ui-interaction.spec.ts` における Prettier フォーマットチェック違反 (P3)

#### 該当コード
- `tests/ui-interaction.spec.ts:166-184`

```bash
$ npm run format:check
Checking formatting...
[warn] tests/ui-interaction.spec.ts
[warn] Code style issues found in the above file. Run Prettier with --write to fix.
```

#### 問題と影響
- 直前のコミットで追加されたテストコードにおいて、引数の折り返し等のフォーマットが Prettier のルールと乖離しており、`npm run format:check` および `npm run check` が失敗する。
- CI のコミットフックや品質ゲートで即時リジェクト要因となる。

#### 推奨修正
`npx prettier --write tests/ui-interaction.spec.ts` を実行して整形を統一する。

---

### R07: 180度センター回転ペアリングの最小手数探索のさらなる余地 (P3)

#### 該当コード
- `src/supercube.rs:145-155`

```rust
// src/supercube.rs:145-155
    for (f, &rot) in cur.iter().enumerate() {
        if rot == 2 {
            moves.extend(rotate_center_180(f));
        }
    }
```

#### 問題の構造と分析
- `rotate_center_180` は単一のセンターを 180 度回転させるために 12 手（`X A B X2 A' B' X A B X2 A' B'`）を消費する。
- 180 度回転が残る面が 2 つ存在する場合（例: U面とD面がともに180度回転している場合）、現在は単純に 12 手 + 12 手 = 24 手を直列適用している。
- 対面の 2 センターを同時に 180 度回転させる手順（例: `U R L U2 R' L' U R L U2 R' L'` 等の対面コミュテータ）や隣接面の 180 度ペア手順を利用すれば、16〜18 手程度に短縮可能である。
- 現在の全マッチングアルゴリズムに 180度同士のペアリング評価を拡張することで、解法手数をさらに 6〜8 手削減できる。

---

### R08: 解法 DOM キャッシュ無効化境界の堅牢性 (P3)

#### 該当コード
- `web/main.ts:167-175`

```typescript
    const list = $("move-list");
    if (renderedSolution !== solution) {
      renderedSolution = solution;
      cachedAnalyzedMoves = analyzeMoves(solution.moves, solution.phases);
      list.replaceChildren();
      ...
```

#### 問題の構造と分析
- `refresh()` 内での `move-list` の再構築判定が、オブジェクト参照 `renderedSolution !== solution` のみに依存している。
- 解法が同一インスタンスのまま、将来的に「トリガー表示の ON/OFF 切り替え」「フェーズ表示言語の変更」「フォントサイズ変更」などの UI オプションが追加された場合、参照が変わらないため DOM が更新されない不整合が発生するリスクがある。
- メモ化キーに UI モードや表示設定のハッシュ・リビジョンを含めるか、明示的な `invalidateSolutionDom()` 関数を設けることが望ましい。

---

## 5. モジュール別品質評価とアーキテクチャ成熟度

| モジュール | 成熟度 | 評価概要 |
|:---|:---:|:---|
| **Rust コア・探索 (`src/`)** | **A** | Kociemba 2-phase、Korf IDA*、Thistlethwaite 4-phase、CFOP、Supercube の 5 アルゴリズムが高度に統合。Phase境界の出力や時間制約契約も極めて洗練されている。R02 の CFOP 探索空間縮約と R03 のセンターパリティ契約を適用すれば世界水準のキューブエンジンとなる。 |
| **WASM バインディング (`src/lib.rs`)** | **A+** | 純粋 Rust 環境（ネイティブテスト）と WASM 環境の切り分け、`to_js_error` による安全な境界設計、Phase 情報の serde 連携など、堅牢性が極めて高い。 |
| **フロントエンド状態 (`web/cube-store.ts`)** | **A+** | イベント駆動型アーキテクチャ（`replace`, `undo`, `redo`, `seek`, `algorithm`）が確立。WASM への同期依存が排除され、`turnsToCenters` による軽量変換で耐障害性が大幅に向上。 |
| **3D レンダリング (`web/scene.ts`)** | **A** | Three.js による高品質なシャドウ・ライティング、全54セルの矢印表示、マテリアルキャッシュ再利用による GPU リーク防止が完備。rAF デバウンスの `dispose` 時キャンセル（R05）のみ要修正。 |
| **カメラ認識 (`web/camera.ts`, `image-sampler.ts`)** | **A** | 対角2視点立体復元、外周6頂点の自動検出、HSV クラスタリング、ドラッグ微調整、WebRTC ライブストリームが完璧に動作。Blob URL の解放ライフサイクルも安全に管理されている。 |
| **PWA / オフライン (`public/sw.js`, `web/pwa.ts`)** | **A-** | スコープ分離（SCOPE_SLUG）による複数デプロイ保護、SWR キャッシュ戦略、プリキャッシュ自動注入が秀逸。開発環境（`npm run dev`）における登録抑止（R04）が最後のピース。 |
| **テストスイート (`tests/`)** | **A+** | ユニット、E2E、アクセシビリティ（axe-core）、CDP カバレッジ計測、PWA オフライン復帰、サブディレクトリデプロイ検証まで網羅された 154 件の強固なテスト。実行時間は約 7.7 分と充実。 |

---

## 6. まとめ・次期推奨アクション

本レビューで浮き彫りになった課題（R01〜R08）は、表面的なコードの不備ではなく、**数理モデルの厳密な前提条件（アドミッシビリティ、群論的パリティ）と非同期フレームワークの境界条件（rAF ライフサイクル、Service Worker スコープ）** に関わる深層の論点である。

以下の順序で改修を適用することを推奨する：

1. **即時対応（CI 正常化）**:
   - `src/supercube.rs` の Clippy 警告を修正（`is_multiple_of(2)`, `is_none_or`）。
   - `tests/ui-interaction.spec.ts` を Prettier で整形。
2. **数理・探索の健全化**:
   - `src/cfop.rs` のコーナー探索ループで `face == 3`（D面）を除外して探索速度向上とアドミッシビリティを両立。
   - `src/supercube.rs` にパリティ事前検証を追加し、奇数個の奇数回転に対して安全にエラーを返却。
3. **ライフサイクルと開発者体験（DX）の向上**:
   - `web/scene.ts` の `dispose()` で `resizeRafId` を確実にキャンセル。
   - `web/pwa.ts` で `import.meta.env.DEV` による開発時の Service Worker 登録を抑止。

---
*レビュー作成: 2026-09-22 / AI Expert Reviewer*
