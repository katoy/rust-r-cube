# カメラ入力テストスイート

Playwright を使用したカメラ入力機能の自動テストスイートです。

## 概要

このテストスイートは、カメラ画像から Rubik's Cube の状態を認識する機能を検証します。

- **テストファイル**: `tests/camera-input.spec.ts`
- **テスト画像**: `test-images/` ディレクトリ（自動生成）
- **テストユーティリティ**: `tests/test-utils.ts`

## テスト実行

### カメラ入力テストを実行

```bash
npm run test:camera
```

このコマンドで以下の処理が実行されます：
1. テスト画像を自動生成
2. camera-input テストスイートを実行
3. center-input テストスイートを実行

### 特定のテストのみ実行

```bash
# カメラ入力テストのみ
npx playwright test tests/camera-input.spec.ts

# センター入力テストのみ
npx playwright test tests/center-input.spec.ts

# 特定のテストグループを実行
npx playwright test tests/camera-input.spec.ts --grep "画像処理テスト"
```

### テスト結果の表示

```bash
# テスト結果をレポート表示
npx playwright show-report

# 失敗時のスクリーンショットを確認
open test-results/
```

## テストの構成

### 1. 画像処理テスト（8 テスト）

基本的な UI 機能とファイル操作をテストします。

#### `マニフェストが存在し、すべての画像ファイルが揃っている`
- マニフェスト JSON が正しく生成されているか確認
- すべての画像ファイルが存在するか確認

#### `solved 状態の画像をアップロードできる`
- ビューA の画像をアップロード
- Canvas に画像が読み込まれることを確認

#### `ビューAとビューBの両方をアップロードできる`
- ビューA と ビューB の両方をアップロード
- 両画像が正常に処理されることを確認

#### `フェース選択が正常に動作する`
- U, R, F, D, L, B の 6 つの面を選択
- 各フェース選択が機能することを確認

#### `四隅をクリックしてキャプチャボタンを有効にできる`
- Canvas の四隅をクリック
- 4 点のクリック後、キャプチャボタンが有効になることを確認

#### `全 6 面をキャプチャしてから apply が有効になる`
- ビューA で 3 面（U, R, F）をキャプチャ
- ビューB で 3 面（D, L, B）をキャプチャ
- 各ステップで進捗が正しく更新されることを確認
- すべてキャプチャ後に apply ボタンが有効になることを確認

#### `キャプチャをキャンセルして別のビューに切り替えられる`
- 面選択時のポイント入力がリセットされることを確認
- 異なる面に切り替え時に UI が正しく更新されることを確認

#### `エラー表示領域が存在する`
- エラー表示要素が存在することを確認
- 正常な操作時にはエラーがクリアされることを確認

### 2. エンドツーエンドテスト（4 テスト）

実際のキューブ状態の認識と処理をテストします。

#### `solved 状態を画像から認識して apply できる`
- 完全に揃ったキューブ画像をアップロード
- すべての面をキャプチャ
- キューブ状態が更新されることを確認

#### `scrambled-1 状態を画像から認識できる`
- スクランブル状態の画像をアップロード
- 複雑に混ざったキューブを正しく処理できることを確認

#### `mixed-colors 状態を処理できる`
- 混合色状態の画像をアップロード
- 色認識が正しく機能することを確認

#### `partial 状態（未入力を含む）を処理できる`
- 一部の面が未入力（？）の画像をアップロード
- 部分的な入力を処理できることを確認

## テスト用画像について

テストは以下の 5 つのキューブ状態の画像を使用：

| 状態 | 説明 | 用途 |
|-----|------|------|
| solved | 完全に揃ったキューブ | 基本的な色認識 |
| superflip | スーパーフリップ状態 | 複雑なパターン |
| scrambled-1 | ランダムなスクランブル | ランダムな入力 |
| mixed-colors | 混合色状態 | 色認識精度 |
| partial | 部分入力状態 | 未入力の取扱い |

各状態について 2 つのビュー（ビューA と ビューB）が生成されます。

### 画像の仕様

- **セルサイズ**: 20 × 20 ピクセル
- **グリッド**: 3 × 3 セル（1 面あたり）
- **ビューA**: U（上）, R（右）, F（前）面を横に並べ
- **ビューB**: D（下）, L（左）, B（背）面を横に並べ
- **背景**: グレー（#f0f0f0）
- **セル間**: 2 ピクセルの黒い枠線

### RGB 色の定義

```javascript
U: "#eeeade" // 白
R: "#e55649" // 赤
F: "#74b89a" // 緑
D: "#efce66" // 黄
L: "#ec9851" // 橙
B: "#6a9edb" // 青
?: "#454b49" // グレー（未入力）
```

## テスト用ユーティリティ

`tests/test-utils.ts` から以下の関数が提供されています：

### `loadImageManifest(): Record<string, { viewA: string; viewB: string }>`
マニフェスト JSON を読み込みます。

```typescript
const manifest = loadImageManifest();
console.log(manifest.solved.viewA); // "solved-view-a.png"
```

### `getTestImagePath(imageName: string, view: "A" | "B"): string`
テスト画像のファイルパスを取得します。

```typescript
const path = getTestImagePath("solved", "A");
```

### `openCameraEditor(page: Page): Promise<void>`
カメラエディタを開きます。

```typescript
await openCameraEditor(page);
await expect(page.locator("#camera-editor")).toBeVisible();
```

### `uploadTestImage(page: Page, imageName: string, view: "A" | "B"): Promise<void>`
テスト画像をカメラエディタにアップロードします。

```typescript
await uploadTestImage(page, "solved", "A");
```

## テスト結果の確認

### コマンドラインでの確認

```bash
npm run test:camera
```

PASS: ✓（チェックマーク）
FAIL: ✗（バツマーク）

### GUI でのレポート確認

```bash
npx playwright show-report
```

### 失敗時のトラブルシューティング

1. **スクリーンショット**: `test-results/` に失敗時のスクリーンショットが保存される
2. **トレース**: `test-results/` にプレイバック可能なトレースが保存される
3. **詳細レポート**: `npx playwright show-report` でインタラクティブに確認

## CI/CD への統合

### GitHub Actions の例

```yaml
name: Camera Input Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '20'
      - run: npm ci
      - run: npm run test:camera
      - uses: actions/upload-artifact@v3
        if: always()
        with:
          name: test-results
          path: test-results/
```

## パフォーマンス

- **テスト実行時間**: 約 35-45 秒（12 テスト）
- **画像生成時間**: 約 1-2 秒（5 状態 × 2 ビュー）
- **タイムアウト**: 45 秒/テスト（Playwright のデフォルト）

## トラブルシューティング

### テストが見つからない

```bash
# テストファイルが存在するか確認
ls -la tests/camera-input.spec.ts

# TypeScript のコンパイルエラーを確認
npm run typecheck
```

### 画像が生成されない

```bash
# 手動で画像を生成
npm run generate-test-images

# 生成されたファイルを確認
ls -la test-images/
```

### テストがタイムアウト

- 開発サーバーが起動しているか確認
- ネットワーク速度を確認
- Playwright の timeout を増加させる

```typescript
test.setTimeout(60000); // 60 秒に設定
```

### 色認識がうまくいかない

1. 生成される画像の RGB 値を確認
   ```bash
   # canvas デバッグ用にスクリーンショットを取得
   npx playwright test tests/camera-input.spec.ts --debug
   ```

2. image-sampler.ts の classify() 関数を確認
3. セルサイズが十分か確認（推奨: 20+ ピクセル）

## 関連ファイル

- `scripts/generate-test-images.js` - テスト画像生成スクリプト
- `web/camera.ts` - カメラ入力 UI コンポーネント
- `web/image-sampler.ts` - 画像サンプリング実装
- `CAMERA_TEST_IMAGES.md` - テスト画像の詳細ドキュメント
- `playwright.config.ts` - Playwright 設定
