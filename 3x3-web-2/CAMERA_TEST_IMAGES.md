# カメラ入力テスト画像

このドキュメントはカメラ入力機能のテストに使用する自動生成画像について説明しています。

## 概要

`scripts/generate-test-images.js` スクリプトで複数のキューブ状態をシミュレートした画像を自動生成できます。

生成された画像は以下の特徴があります：
- **2 ビューシステム** - 各状態について ビューA と ビューB の 2 種類の角度から撮影
- **正確な RGB 色** - `COLORS` で定義された標準色を使用
- **3×3 グリッド** - 各ステッカーを 20×20 ピクセルで描画
- **複数状態** - solved、scrambled など異なるキューブ状態の画像セット

## ファイル構成

```
test-images/
├── manifest.json              # 画像メタデータ
├── solved-view-a.png         # Solved 状態（ビューA）
├── solved-view-b.png         # Solved 状態（ビューB）
├── superflip-view-a.png      # スーパーフリップ状態
├── superflip-view-b.png
├── scrambled-1-view-a.png    # スクランブル状態
├── scrambled-1-view-b.png
├── mixed-colors-view-a.png   # 混合色状態
├── mixed-colors-view-b.png
├── partial-view-a.png        # 部分入力状態
└── partial-view-b.png
```

## 使い方

### 1. テスト画像を生成

```bash
npm run generate-test-images
```

生成後 `test-images/` ディレクトリに PNG 画像とマニフェストが作成されます。

### 2. テストコードで使用

```typescript
import { openCameraEditor, uploadTestImage, captureFaceOnTestImage } from "./test-utils";

test("カメラ入力でキューブ状態を認識", async ({ page }) => {
  await openCameraEditor(page);
  
  // ビューA（U, R, F 面）をアップロード
  await uploadTestImage(page, "solved", "A");
  
  // 各面をキャプチャ
  for (const face of ["U", "R", "F"]) {
    await captureFaceOnTestImage(page, face);
  }
  
  // ビューB（D, L, B 面）をアップロード
  await uploadTestImage(page, "solved", "B");
  
  for (const face of ["D", "L", "B"]) {
    await captureFaceOnTestImage(page, face);
  }
  
  // Apply して完了
  await page.locator("#camera-apply").click();
});
```

### 3. テスト用ユーティリティ

`tests/test-utils.ts` には以下のヘルパー関数が提供されています：

- `loadImageManifest()` - マニフェストを読み込む
- `getTestImagePath(imageName, view)` - 画像ファイルパスを取得
- `openCameraEditor(page)` - カメラエディタを開く
- `uploadTestImage(page, imageName, view)` - テスト画像をアップロード
- `captureFaceOnTestImage(page, face)` - 面をキャプチャ
- `completeImageCapture(page, imageName)` - 全面をキャプチャして完了

## 生成される画像の状態

### 1. solved
完全に揃ったキューブの状態。各面がすべて同じ色。

### 2. superflip
スーパーフリップ状態。コーナーはすべて正しく、エッジがすべて反転している状態。

### 3. scrambled-1
ランダムにスクランブルされた状態。複雑に混ざっているキューブ。

### 4. mixed-colors
各面が異なる色で混ざった状態。色認識の精度をテストできます。

### 5. partial
一部の面だけが入力された状態。`?` 記号で未入力を表現。

## カスタマイズ

### 新しい状態を追加

`scripts/generate-test-images.js` の `CUBE_STATES` 配列に追加：

```javascript
const CUBE_STATES = [
  // ... 既存状態
  {
    name: "custom-state",
    state: "UUUUUUUUURRRRRRRRFFFFFFFDDDDDDDDDLLLLLLLLBBBBBBBB",
  },
];
```

### 画像のサイズを変更

`scripts/generate-test-images.js` の定数を編集：

```javascript
const CELL_SIZE = 20;      // セル 1 個あたりのサイズ
const PADDING = 10;        // 画像の余白
const BORDER = 2;          // セル間の枠線幅
```

### 画像の仕様

- **サイズ**: 640 × 480 ピクセル
- **投影**: 等角投影（アイソメトリック）による3Dキューブ表示
- **ビューA**: U（上面）, R（右面）, F（前面）の3面立体ビュー
- **ビューB**: D（下面）, L（左面）, B（背面）の3面立体ビュー
- **背景**: グレー（#e5e7e6）
- **セル間**: 2 ピクセルの境界線（#1b1e1d）

## マニフェストの形式

```json
{
  "generated": "2026-09-13T03:37:58.061Z",
  "imageWidth": 640,
  "imageHeight": 480,
  "totalStates": 5,
  "views": {
    "A": {
      "faces": ["U", "R", "F"],
      "corners": {
        "U": [{ "x": 320, "y": 80 }, ...],
        "F": [...],
        "R": [...]
      }
    },
    "B": {
      "faces": ["D", "L", "B"],
      "corners": { ... }
    }
  },
  "images": {
    "solved": {
      "viewA": "solved-view-a.png",
      "viewB": "solved-view-b.png",
      "expectedState": "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB"
    }
  }
}
```

## テストの実行

### カメラテストを実行

```bash
npm run test:camera
```

このコマンドで画像を生成してからカメラ入力テストを実行します。

### すべてのテストを実行

```bash
npm test
```

## 技術詳細

### 画像生成パイプライン

1. Node.js `canvas` ライブラリで Canvas を作成
2. キューブ状態の文字列から各ステッカーの色を抽出
3. RGB 値を 20×20 ピクセルのセルに描画
4. 3×3 グリッドで一つの面を構成
5. ビューA（3 面）と ビューB（3 面）に分けて PNG 出力

### RGB 色マッピング

生成される画像は以下の標準色を使用：

| 面   | 色       | RGB       |
|------|---------|-----------|
| U    | 白     | #eeeade  |
| R    | 赤     | #e55649  |
| F    | 緑     | #74b89a  |
| D    | 黄     | #efce66  |
| L    | 橙     | #ec9851  |
| B    | 青     | #6a9edb  |
| ?    | グレー | #454b49  |

### 色認識アルゴリズム

`image-sampler.ts` の `classify()` 関数では：

1. 各ピクセルの周辺（3×3 領域）を取得
2. 標準色との RGB 距離を計算
3. 最小距離の色を選択

このため、セルサイズは最低でも 7×7 ピクセル以上が推奨されます。

## トラブルシューティング

### 画像が生成されない

1. Canvas ライブラリがインストールされているか確認
   ```bash
   npm install canvas
   ```

2. Node.js バージョンを確認（推奨: v16 以上）
   ```bash
   node --version
   ```

### テスト画像がアップロードされない

1. `test-images/` ディレクトリが存在するか確認
2. Playwright の設定で画像ファイルアクセスが許可されているか確認

### 色認識がうまくいかない

1. セルサイズが大きすぎないか確認（推奨: 15-30 ピクセル）
2. Canvas の描画品質を確認
3. RGB 値が正確に設定されているか確認（マニフェストで確認可能）

## 関連ファイル

- `scripts/generate-test-images.js` - 画像生成スクリプト
- `tests/test-utils.ts` - テスト用ユーティリティ
- `web/camera.ts` - カメラ入力 UI
- `web/image-sampler.ts` - 画像サンプリング実装
