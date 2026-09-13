/**
 * カメラ入力テスト用の画像を自動生成
 * 複数のキューブ状態に対応した PNG 画像を生成する
 */

import { createCanvas } from "canvas";
import * as fs from "fs";
import * as path from "path";

const COLORS: Record<string, string> = {
  U: "#eeeade", // 白
  R: "#e55649", // 赤
  F: "#74b89a", // 緑
  D: "#efce66", // 黄
  L: "#ec9851", // 橙
  B: "#6a9edb", // 青
  "?": "#454b49", // グレー
};

const FACES = "URFDLB";
const CELL_SIZE = 20;
const STICKERS_PER_FACE = 9;
const GRID_SIZE = 3;
const PADDING = 10;
const BORDER = 2;

interface CubeState {
  name: string;
  state: string;
}

/**
 * キューブ状態 - state は 54 文字で各面 9 ステッカー
 */
const CUBE_STATES: CubeState[] = [
  {
    name: "solved",
    state: "UUUUUUUUURRRRRRRRFFFFFFFDDDDDDDDDLLLLLLLLBBBBBBBB",
  },
  {
    name: "superflip",
    state: "UUUUUUUUURRRRRRRRFFFFFFFDDDDDDDDDLLLLLLLLBBBBBBBB", // 簡略版（実際はもっと複雑）
  },
  {
    name: "scrambled-1",
    state: "URFDLBURDLBFURDLBFURDLFBFURDLBURDLFBUDLBFURDLFBUR",
  },
  {
    name: "mixed-colors",
    state: "UUUURRRRFFFFDDDDDLLLLUUUURRRRFFFFDDDDDLLLLBBBBBBB",
  },
  {
    name: "partial",
    state: "UUUUUUUU???????U????????F????F??DDDDDDDD???????B",
  },
];

/**
 * 16 進数カラーコードを { r, g, b } オブジェクトに変換
 */
function hexToRgb(
  hex: string
): { r: number; g: number; b: number } {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  if (!result) throw new Error(`Invalid hex color: ${hex}`);
  return {
    r: parseInt(result[1], 16),
    g: parseInt(result[2], 16),
    b: parseInt(result[3], 16),
  };
}

/**
 * キューブ面を Canvas に描画
 */
function drawCubeFace(
  ctx: CanvasRenderingContext2D,
  startX: number,
  startY: number,
  faceIndex: number,
  state: string
) {
  const faceStart = faceIndex * STICKERS_PER_FACE;
  const faceColors = state.slice(faceStart, faceStart + STICKERS_PER_FACE);

  for (let row = 0; row < GRID_SIZE; row++) {
    for (let col = 0; col < GRID_SIZE; col++) {
      const cellIndex = row * GRID_SIZE + col;
      const color = faceColors[cellIndex];
      const hexColor = COLORS[color] || COLORS["?"];

      const x = startX + col * (CELL_SIZE + BORDER);
      const y = startY + row * (CELL_SIZE + BORDER);

      // セルを描画
      ctx.fillStyle = hexColor;
      ctx.fillRect(x, y, CELL_SIZE, CELL_SIZE);

      // 枠線を描画
      ctx.strokeStyle = "#333";
      ctx.lineWidth = 1;
      ctx.strokeRect(x, y, CELL_SIZE, CELL_SIZE);
    }
  }
}

/**
 * 2 ビュー（2 つの角度）からキューブを描画
 */
function generateTwoViewImage(state: string): {
  viewA: Buffer;
  viewB: Buffer;
} {
  const faceWidth = GRID_SIZE * CELL_SIZE + (GRID_SIZE - 1) * BORDER;
  const faceHeight = GRID_SIZE * CELL_SIZE + (GRID_SIZE - 1) * BORDER;

  // ビュー A: 3 面を横に並べる
  const canvasA = createCanvas(
    faceWidth * 3 + PADDING * 4,
    faceHeight + PADDING * 2
  );
  const ctxA = canvasA.getContext("2d");
  ctxA.fillStyle = "#f0f0f0";
  ctxA.fillRect(0, 0, canvasA.width, canvasA.height);

  // ビュー A: U, R, F 面
  const facesA = [0, 1, 2]; // U, R, F
  facesA.forEach((faceIndex, i) => {
    drawCubeFace(
      ctxA,
      PADDING + i * (faceWidth + PADDING),
      PADDING,
      faceIndex,
      state
    );
  });

  // ビュー B: D, L, B 面
  const canvasB = createCanvas(
    faceWidth * 3 + PADDING * 4,
    faceHeight + PADDING * 2
  );
  const ctxB = canvasB.getContext("2d");
  ctxB.fillStyle = "#f0f0f0";
  ctxB.fillRect(0, 0, canvasB.width, canvasB.height);

  const facesB = [3, 4, 5]; // D, L, B
  facesB.forEach((faceIndex, i) => {
    drawCubeFace(
      ctxB,
      PADDING + i * (faceWidth + PADDING),
      PADDING,
      faceIndex,
      state
    );
  });

  return {
    viewA: canvasA.toBuffer("image/png"),
    viewB: canvasB.toBuffer("image/png"),
  };
}

/**
 * テスト画像を生成して保存
 */
async function generateTestImages() {
  const outputDir = path.join(
    path.dirname(__filename),
    "../test-images"
  );

  // 出力ディレクトリを作成
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
  }

  console.log(`📸 カメラテスト画像生成開始`);
  console.log(`📁 出力先: ${outputDir}\n`);

  const imageManifest: Record<
    string,
    { viewA: string; viewB: string }
  > = {};

  for (const cubeState of CUBE_STATES) {
    console.log(`⚙️  生成中: ${cubeState.name}`);

    const { viewA, viewB } = generateTwoViewImage(cubeState.state);

    const viewAPath = path.join(
      outputDir,
      `${cubeState.name}-view-a.png`
    );
    const viewBPath = path.join(
      outputDir,
      `${cubeState.name}-view-b.png`
    );

    fs.writeFileSync(viewAPath, viewA);
    fs.writeFileSync(viewBPath, viewB);

    imageManifest[cubeState.name] = {
      viewA: `${cubeState.name}-view-a.png`,
      viewB: `${cubeState.name}-view-b.png`,
    };

    console.log(`   ✅ ${cubeState.name} 生成完了`);
  }

  // マニフェストファイルを生成
  const manifestPath = path.join(outputDir, "manifest.json");
  fs.writeFileSync(
    manifestPath,
    JSON.stringify(
      {
        generated: new Date().toISOString(),
        cellSize: CELL_SIZE,
        gridSize: GRID_SIZE,
        totalStates: CUBE_STATES.length,
        images: imageManifest,
      },
      null,
      2
    )
  );

  console.log(`\n✨ 画像生成完了！`);
  console.log(`📊 生成された画像:`);
  console.log(`   - 状態数: ${CUBE_STATES.length}`);
  console.log(`   - ビュー数: ${CUBE_STATES.length * 2}`);
  console.log(`   - マニフェスト: ${manifestPath}`);
}

generateTestImages().catch(console.error);
