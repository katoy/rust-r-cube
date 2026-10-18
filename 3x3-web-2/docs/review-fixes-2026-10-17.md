# レビュー指摘事項の修正および再発防止対応レポート（2026-10-17）

- **実施日**: 2026-10-17（JST）
- **参照レビュー**: [docs/code-review-2026-10-17.md](code-review-2026-10-17.md)
- **対象範囲**: Rust ソルバー（`src/c3x3/search.rs`, `src/lib.rs`, `src/c2x2/mod.rs`, `src/c2x2/ortega.rs`, `src/c2x2/tables.rs`）、Web フロントエンド（`web/main.ts`, `web/model-2x2.ts`, `web/solver.worker.ts`, `web/camera.ts`）、PWA Service Worker（`public/sw.js`）、ガードレール・起動スクリプト（`scripts/check-review-guardrails.js`, `start.sh`）、テスト（`tests/coverage.spec.ts`）

---

## 1. 対応状況サマリー

| ID | 重大度 | 対象ファイル | 状態 | 修正内容 |
| :--- | :---: | :--- | :---: | :--- |
| **M1** | **Medium** | [`src/c3x3/search.rs`](../src/c3x3/search.rs)<br>[`src/lib.rs`](../src/lib.rs) | **解消** | `allow_deep_phase2` フラグと `has_custom_max` を導入し、`initial_max >= 24` の際に `total == 24`（Phase 2 上限 12）の重複探索をスキップして即座に Phase 2 上限 18 の探索へ進むよう修正。`lib.rs` では接合部相殺後の手数を `seq_len` とし、`fixes.is_empty()` 時は不要な再探索を回避。 |
| **M2** | **Medium** | [`web/model-2x2.ts`](../web/model-2x2.ts)<br>[`web/main.ts`](../web/main.ts)<br>[`web/solver.worker.ts`](../web/solver.worker.ts)<br>[`src/c2x2/mod.rs`](../src/c2x2/mod.rs) | **解消** | 空間回転された 2x2 完成状態（6面すべて単色）を判定する `isUniformSolved2x2` を実装し、向き OFF 時の完成表示・効果音発火・向き ON 時の「6面完成（標準の向きと異なります）」表示に対応。また LBL / Ortega でも向き OFF 時は DBL コーナー基準の向きに正規化して解くよう統一。 |
| **M4** | **Medium** | [`public/sw.js`](../public/sw.js) | **解消** | クエリ付きランタイムキャッシュの `queryKeysQueue` への重複登録を排除して自己削除を防止するとともに、永続キャッシュ（`ownCache.keys()`）のクエリ付きエントリ数を直接照合して `MAX_QUERY_ENTRIES`（50件）を超えた古いエントリを破棄するよう修正。 |
| **M5** | **Medium** | [`src/lib.rs`](../src/lib.rs) | **解消** | Thistlethwaite 解法においてもセンター補正の有無を問わず `supercube::cancel_redundant_moves_with_phases` を適用し、フェーズ境界の連続同面回転（`U U2` 等）と空フェーズを相殺・除去。 |
| **L1** | Low | [`src/c2x2/ortega.rs`](../src/c2x2/ortega.rs) | **解消** | `solve_first_face` の BFS キュー追加条件を `moves.len() + 1 < 5` から `< 6` に修正し、深さ 6 までの探索が実際に機能するよう是正。 |
| **L2** | Low | [`web/main.ts`](../web/main.ts) | **解消** | カメラ入力から色入力エディタへ連続遷移する際、閉じた `<dialog>` 内の要素で `lastTriggerElement` を上書きしないよう保護し、別のモーダルが開いている間は `close` イベントでフォーカスを奪わないよう修正。 |
| **L3** | Low | [`web/camera.ts`](../web/camera.ts) | **解消** | `validate2x2Faces()` で未入力（`?`）に戻された場合は古い整合性エラー表示をクリアするよう修正。 |
| **L4** | Low | [`web/camera.ts`](../web/camera.ts) | **解消** | `readImageDimensions()` の WebP 判定で 25〜29 バイトの最小有効ロスレス WebP（`VP8L`）を許容するよう修正。 |
| **L5** | Low | [`scripts/check-review-guardrails.js`](../scripts/check-review-guardrails.js)<br>[`start.sh`](../start.sh) | **解消** | SW の `boundedCacheKey` 検査で `activate` 内の `caches.delete` ではなく `ownCache.delete(` の存在を必須化。`start.sh` の WASM 鮮度検査に `build.rs` を追加。 |
| **L6** | Low | [`tests/coverage.spec.ts`](../tests/coverage.spec.ts) | **解消** | 72 バイトの JSON に `70000` のモックサイズ引数を渡していた箇所を、実際に 64KB を超える JSON ペイロード（`hugeJson`）と実サイズ（`hugeJson.length`）による検証へ置換。 |
| **L8** | Low | [`src/c2x2/tables.rs`](../src/c2x2/tables.rs) | **解消** | 2x2 探索で参照されていなかった未使用の `c2x2::tables::MoveTable` コードを削除。 |

---

## 2. 主要修正の詳細

### 2.1 3x3 センター向き込み Kociemba 探索の重複走査排除と Phase 2 深さ上限の適正化 (M1)
- [`src/c3x3/search.rs`](../src/c3x3/search.rs):
  - `set_max_total(seq_len)` により `initial_max >= 24` が設定された場合、1 回目のループ（Phase 2 上限 12）で既に深さ 24 以下の探索空間が網羅されているにもかかわらず、緩和ループの `total = 24` で同一探索木を再走査していた問題を解消しました。
  - 緩和ループの `total = 30` では `allow_deep_phase2 = true` を有効化し、`self.max_total` が `seq_len` で頭打ちされている場合でも Phase 2 深さ上限 18 が確実に適用されるようにしました。
- [`src/lib.rs`](../src/lib.rs):
  - 色のみの解 `cm` とセンター解決手順 `fixes` を結合した直後に `supercube::cancel_redundant_moves_with_phases` を適用し、相殺後の実質手数を `seq_len` として同時最適化探索の上限に設定しました。
  - `fixes.is_empty()`（色解法のみでセンターも揃っている場合）は不要な再探索を行わず即座に採用するようにしました。

### 2.2 2x2 の空間回転完成状態の判定と LBL / Ortega の向き OFF 正規化 (M2)
- [`web/model-2x2.ts`](../web/model-2x2.ts):
  - 24 文字の 2x2 盤面で 6 面すべてが単色かつ 6 色揃っているかを判定する `isUniformSolved2x2(state)`、および DBL コーナーの色を `(D, B, L)` に写像する `relabel2x2ForDbl(state)` を実装しました。
- [`web/main.ts`](../web/main.ts):
  - 空間回転された 2x2 完成状態において、「標準の向きに揃える」が OFF の場合は「完成状態」として扱い（ボタン文言「完成状態を確認」および手動回転完成時の効果音発火）、ON の場合は「6面完成（標準の向きと異なります）」と明示するよう改善しました。
- [`src/c2x2/mod.rs`](../src/c2x2/mod.rs) / [`web/solver.worker.ts`](../web/solver.worker.ts):
  - `include_orientation = false` で LBL 法または Ortega 法を実行した際、Optimal と同様に DBL コーナー基準の空間向きへ正規化してから解くように統一しました。

### 2.3 Service Worker クエリキャッシュの永続上限管理 (M4)
- [`public/sw.js`](../public/sw.js):
  - `queryKeysQueue` への同一 URL 重複追加を排除し、自身のキャッシュエントリを誤削除する問題を解消しました。
  - さらに `ownCache.keys()` からクエリ付きリクエスト一覧を取得して `MAX_QUERY_ENTRIES`（50 件）を超過した古いエントリを削除することで、Service Worker の再起動を跨いでも上限管理が維持されるようにしました。
