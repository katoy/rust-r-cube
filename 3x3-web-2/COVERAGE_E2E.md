# E2E Coverage Measurement with Browser DevTools Protocol

ブラウザの Chrome DevTools Protocol（CDP）と UI 操作ベースのトレーシングを使用して、E2E テスト実行時の JavaScript/TypeScript/WASM カバレッジを計測します。

## 📊 計測方法

### 方法1：基本的な CDP カバレッジ計測（`coverage.spec.ts`）

- **計測ツール**: Playwright + Browser DevTools Protocol
- **対象コード**: ブラウザで実行される JavaScript / TypeScript
- **計測単位**: 関数、行（V8 エンジンレベル）
- **WASM 呼び出し記録**: UI 操作経由での間接的なトレーシング

### 方法2：詳細な操作トレーシング（`coverage-advanced.spec.ts`）

- **計測ツール**: Playwright パフォーマンスタイミング
- **対象**: UI 操作ごとの実行時間計測
- **特徴**:
  - スクランブル、解法、再生などの各操作の実行時間を記録
  - 向きモード（orientation）有効/無効の比較分析
  - コンソールログキャプチャ
  - 詳細なパフォーマンス統計

### 計測対象外

- **Rust コアロジック**: `cargo llvm-cov` で別途計測が必要
- **WASM バイナリコード**: JIT コンパイルされるため、行単位の詳細は不完全

## 🚀 実行方法

### 1. WASM ビルド（必須）

```bash
npm run wasm
```

### 2a. 基本的な CDP カバレッジテスト（推奨）

```bash
npm run test:coverage
```

**特徴**:
- ✅ シンプルで高速（4 秒で完了）
- ✅ 15 個の JS/TS ファイルをスキャン
- ✅ UI 操作経由で WASM を呼び出し

**生成ファイル**:
- `coverage-e2e/index.html` - HTML レポート
- `coverage-e2e/coverage.json` - JSON データ

### 2b. 詳細カバレッジテスト（パフォーマンス分析）

```bash
# 全テストを実行
npm run test -- tests/coverage-advanced.spec.ts

# または特定のテストのみ
npm run test -- tests/coverage-advanced.spec.ts -g "UI 操作ベース"
```

**テスト1: UI 操作ベースの WASM 呼び出し追跡**
- ページ読み込み、スクランブル、解法、再生の実行時間を計測
- コンソールログをキャプチャ
- 詳細なパフォーマンス統計を生成

**テスト2: 向きモード有効時のカバレッジ計測**
- 向きモード（orientation）有効/無効を比較
- 各モードでの解法手数と実行時間を比較
- パフォーマンス差分を可視化

**生成ファイル**:
- `coverage-e2e-advanced/index.html` - 操作トレースレポート
- `coverage-e2e-advanced/operations.json` - JSON データ
- `coverage-e2e-advanced/orientation-comparison.html` - 向きモード比較レポート

### 3. レポート確認

```bash
# 基本的なレポート
open coverage-e2e/index.html

# 詳細レポート
open coverage-e2e-advanced/index.html
open coverage-e2e-advanced/orientation-comparison.html

# JSON で詳細を確認
cat coverage-e2e/coverage.json
cat coverage-e2e-advanced/operations.json
```

## 📈 レポート内容

### HTML レポート（`coverage-e2e/index.html`）

| 項目 | 説明 |
|------|------|
| **対象ファイル** | ブラウザで実行された JS/TS ファイル一覧 |
| **カバレッジ率** | ファイルごとのカバレッジ率（高/中/低で色分け） |
| **WASM 呼び出し履歴** | テスト実行時に呼び出された WASM 関数の順序 |

### JSON レポート（`coverage-e2e/coverage.json`）

```json
{
  "timestamp": "2026-09-12T...",
  "method": "Browser DevTools Protocol (CDP)",
  "files": [
    {
      "url": "http://127.0.0.1:5173/web/solver.worker.ts",
      "covered": 150,
      "total": 200,
      "percentage": "75.00"
    },
    ...
  ],
  "wasmCallLog": [
    "initialize",
    "scramble",
    "apply_moves",
    "validate",
    "solve"
  ]
}
```

## 🔧 テストコードのカスタマイズ

### 追加の WASM 関数をテストする

`tests/coverage.spec.ts` の `page.evaluate()` セクションで、新しい関数呼び出しを追加：

```typescript
// WASM 関数をラップしてログに記録
wasm.solve_with_orientation = function (...args: any[]) {
  callLog.push("solve_with_orientation");
  return originalSolveWithOrientation.apply(this, args);
};

// テスト実行
await page.evaluate(async () => {
  const wasm = (window as any).cube_studio;
  wasm.solve_with_orientation(state, true, 5000);
});
```

### より詳細な WASM トレーシング

呼び出し引数やリターン値をログに記録：

```typescript
wasm.solve = function (state: string, budget: number) {
  const startTime = performance.now();
  callLog.push({
    function: "solve",
    args: { stateLength: state.length, budget },
    timestamp: startTime,
  });
  const result = originalSolve.apply(this, [state, budget]);
  const elapsed = performance.now() - startTime;
  callLog[callLog.length - 1].elapsed = elapsed;
  return result;
};
```

## ⚙️ パフォーマンス考慮事項

| 項目 | 特性 |
|------|------|
| **計測オーバーヘッド** | V8 Coverage: 5-10% 程度 |
| **メモリ使用量** | JavaScript ヒープ: +5-20MB |
| **テスト実行時間** | +3-5 秒（通常の E2E テスト比） |

大規模テストスイートの場合は、カバレッジ計測の有効/無効を制御できます：

```bash
# カバレッジ計測なし（高速）
npm run test

# カバレッジ計測あり（詳細）
npm run test:coverage
```

## 🔗 関連

- **Rust カバレッジ**: `cargo llvm-cov test --open`
- **統合レポート**: [[test_coverage_100_percent]] 参照
- **既存 E2E テスト**: `tests/app.spec.ts`、`tests/performance.spec.ts`

## 📝 ログ例

### 基本テスト（coverage.spec.ts）

```
✓ WASM テスト実行完了: UI 操作経由
✓ JS カバレッジ対象: 15 ファイル
✓ 記録された操作: 4 個
✅ カバレッジレポート生成完了:
   📄 HTML: /path/to/coverage-e2e/index.html
   📋 JSON: /path/to/coverage-e2e/coverage.json
```

### 詳細テスト（coverage-advanced.spec.ts）

```
📍 ページを開く...
✓ ページ読み込み完了 (1322ms)
🎮 スクランブル実行...
✓ スクランブル完了 - 状態: URBDUBDLFDURURLLUBFD...
🔍 解法探索実行...
✓ 解法探索完了 - 手数: 22 手 (280ms)
▶️  解法再生開始...
✓ 解法再生完了 (676ms)
🔄 リセット実行...
✓ リセット完了 (351ms)
✅ 詳細レポート生成完了:
   📄 HTML: /path/to/coverage-e2e-advanced/index.html
   📋 JSON: /path/to/coverage-e2e-advanced/operations.json
✅ テスト完了: 5 個の操作をトレース
```

## 🔄 統合カバレッジ（Rust + JavaScript）

| ツール | 対象 | コマンド |
|--------|------|---------|
| `cargo llvm-cov` | Rust コード | `cargo llvm-cov test --open` |
| `tests/coverage.spec.ts` | JavaScript / TypeScript | `npm run test:coverage` |
| `tests/coverage-advanced.spec.ts` | パフォーマンス / 操作トレース | `npm run test -- tests/coverage-advanced.spec.ts` |

完全なテストカバレッジを得るには、これら 3 つを組み合わせて実行してください。

## 📚 参考

- [[test_coverage_100_percent]] - 2026-09-11 のテストカバレッジ改善記録
- Playwright カバレッジ API: https://playwright.dev/docs/coverage
- Chrome DevTools Protocol: https://chromedevtools.github.io/devtools-protocol/
