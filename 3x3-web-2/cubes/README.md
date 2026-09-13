# キューブ状態サンプル集

このディレクトリには、3×3×3 ルービックキューブのさまざまな状態をJSON形式で保存しています。

## ファイル形式

各ファイルは以下の構造を持つJSON形式です：

```json
{
  "version": 1,
  "state": "UUUUUUUUU...",  // 54文字のキューブ状態文字列
  "description": "...",        // 状態の説明
  "scramble": "R U F ...",     // スクランブルシーケンス（R/U/F/D/L/B記法）
  "solution_moves": [...],     // ソリューション手順
  "solution_length": 20,       // 解くために必要な手数
  "notes": "..."              // 追加情報
}
```

### 状態文字列（state）

54文字の文字列で、キューブの全ステッカーを表します：

```
U面(9) + R面(9) + F面(9) + D面(9) + L面(9) + B面(9) = 54文字
UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB
```

各面の文字は U(上)/R(右)/F(前)/D(下)/L(左)/B(奥) を表します。

## サンプルファイル

### solved.json - 完成状態 ✅

完全に解いたキューブの状態です。すべてのステッカーが正しい位置にあります。

```
解くために必要な手数: 0
用途: ソルバーのテスト基準値
```

**使用例:**
```typescript
const solved = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
const result = wasm.validate(solved);  // true を返す
```

---

### superflip.json - スーパーフリップ ⚡

**特徴**: すべてのエッジピース（12個）が反転している特殊な状態

- **神の数**: 20手（最難度のキューブ状態の1つ）
- **特異性**: コーナーピースは正しい位置だが、すべてのエッジが反転
- **生成方法**: 
  ```
  R U' R U R U R U' R' U' R2 U R U' R' U' R2 U
  ```

**数学的意義**：
- キューブソルバーのベンチマークテスト
- 最悪ケースの性能測定
- God's Number (20) を実証する状態

**使用例:**
```typescript
// スーパーフリップを生成
const superflip = wasm.apply_moves(solved, "R U' R U R U R U' R' U' R 2 U R U' R' U' R 2 U");

// ソルバーの能力確認
const solution = wasm.solve(state, 5000);  // 最大5秒で解く
if (solution.moves.length <= 20) {
  console.log("✅ God's Number を達成");
}
```

**参考資料**:
- [SuperFlip - Rubik's Cube Wiki](https://www.speedcubing.org)
- [God's Number - 20](https://en.wikipedia.org/wiki/Rubik%27s_cube#Optimal_solutions)

---

### easy-5-moves.json - 簡単な状態（5手） 🟢

初級者向けの簡単なスクランブルです。

- **解くために必要な手数**: 5手
- **スクランブル**: `R U F`
- **用途**: UI テスト、ベンチマーク、デモンストレーション

**使用例:**
```typescript
const state = wasm.apply_moves(solved, "R U F");
const solution = wasm.solve(state, 1000);
console.log(`解く時間: ${solution.elapsed_ms}ms`);  // 非常に高速
```

---

### t-perm.json - T-Permutation パターン 🔄

速解きキューブ（Speedcubing）で一般的なパターン。OLL/PLL ステップで頻出します。

- **パターン名**: T-Perm (T-Permutation)
- **出現度**: 速解きの最後の層で頻繁に現れる
- **解法手数**: 約6手
- **用途**: 速解きアルゴリズムテスト

**参考資料**:
- [PLL Algorithm Set](https://www.speedcubing.org)
- [CubingChronicles](https://www.youtube.com/c/CubingChronicles)

---

### seed-1-scramble.json - シード値ベースのスクランブル 🎲

WASM の `scramble(seed)` 関数で生成される擬似乱数ベースのスクランブル。

- **シード値**: 1
- **特徴**: 同じシード値なら同じスクランブルが得られる（再現可能）
- **用途**: テスト、デバッグ、ベンチマーク

**使用例:**
```typescript
// 同じシード値で同じスクランブルを再生成
const scramble1 = wasm.scramble(1);
const scramble2 = wasm.scramble(1);
console.log(scramble1 === scramble2);  // true（再現可能）

// 複数のシード値でテスト
for (let seed = 1; seed <= 100; seed++) {
  const scramble = wasm.scramble(seed);
  const state = wasm.apply_moves(solved, scramble);
  const solution = wasm.solve(state, 5000);
}
```

---

## WASM との統合

### UI での使用（web/main.ts）

```typescript
import cubes from '../cubes/solved.json';

// ファイルから状態を読み込む
const loadCubeState = async (filename: string) => {
  const response = await fetch(`/cubes/${filename}.json`);
  const data = await response.json();
  return data.state || wasm.apply_moves(solved, data.scramble);
};

// ボタンクリック時に状態を読み込み
document.getElementById("load-superflip")?.addEventListener("click", async () => {
  const state = await loadCubeState("superflip");
  await solve(state);
});
```

### テスト（src/tests.rs）

```rust
#[test]
fn superflip_solvable_in_20_moves() {
    // スーパーフリップのスクランブル実行
    let scramble = vec![/* R, U', R, U, ... */];
    let cube = cube::apply(&cube::parse_state(SOLVED).unwrap(), &scramble);
    
    // 解法試行
    let mut search = search::Search::new(5000);
    let moves = search.solve(&cube);
    
    // 20手以内で解けるか確認
    assert!(moves.unwrap_or_default().len() <= 20);
}
```

---

## キューブ表記法

### Face Notation（面記号）

- **U**: Up（上面）- 時計回り
- **D**: Down（下面）- 時計回り
- **R**: Right（右面）- 時計回り
- **L**: Left（左面）- 時計回り
- **F**: Front（前面）- 時計回り
- **B**: Back（奥面）- 時計回り

### Modifiers（修飾子）

- **無記号**: 時計回り 90° （例: R）
- **'**: 反時計回り 90° （例: R'）
- **2**: 180° 回転 （例: R 2）

### 例

```
R U R' U' R' F R 2 U' R' U' R U R' F'
= 右、上、右反、上反、右反、前、右2、上反、右反、上反、右、上、右反、前反
```

---

## 参考資料

### キューブ理論

- [Rubik's Cube - Wikipedia](https://en.wikipedia.org/wiki/Rubik%27s_cube)
- [God's Number = 20](https://en.wikipedia.org/wiki/Rubik%27s_cube#Optimal_solutions)
- [Kociemba's Algorithm](https://en.wikipedia.org/wiki/Rubik%27s_cube#Kociemba's_algorithm)

### 速解きコミュニティ

- [speedcubing.org](https://www.speedcubing.org)
- [CubingChronicles](https://www.youtube.com/c/CubingChronicles)
- [JPerm.net](https://www.jperm.net)

### ウェブアプリケーション

- [Rubik's Cube Solver - このプロジェクト](http://127.0.0.1:5173/)
- [Rubik's Cube Online](https://www.rubiks.com)

---

## ファイルの追加方法

新しいキューブ状態を追加する場合：

1. **JSON ファイルを作成**
   ```json
   {
     "version": 1,
     "state": "...",
     "description": "...",
     "scramble": "...",
     "solution_length": N
   }
   ```

2. **README.md に説明セクションを追加**

3. **cubes/ ディレクトリにファイルを保存**

4. **UI から参照可能にするか、テストケースに追加**

---

## ライセンス

このサンプル集は、Rubik's Cube の学習・研究・速解き練習を目的として提供されています。

すべてのキューブパターンは一般的なナレッジのため、著作権の対象ではありません。

---

**最終更新**: 2026-09-12
**バージョン**: 1.0
