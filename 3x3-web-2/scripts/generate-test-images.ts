/**
 * カメラ入力テスト用の画像を自動生成
 * 実写写真（黒フレーム、角丸ステッカー、八角形センター、光沢、接地影）に準拠した画像を生成する
 */

import { createCanvas } from "canvas";
import * as fs from "fs";
import * as path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const COLORS: Record<string, string> = {
  U: "#eeeade", // 白
  R: "#e55649", // 赤
  F: "#74b89a", // 緑
  D: "#efce66", // 黄
  L: "#ec9851", // 橙
  B: "#6a9edb", // 青
  "?": "#454b49", // グレー
};

const WIDTH = 640;
const HEIGHT = 480;
const cx = 320;
const cy = 240;
const L = 160;

// アイソメトリック投影の3軸ベクトル（上から見下ろす対角アングル）
const vL = { x: -L * Math.cos(Math.PI / 6), y: -L * Math.sin(Math.PI / 6) };
const vR = { x: L * Math.cos(Math.PI / 6), y: -L * Math.sin(Math.PI / 6) };
const vD = { x: 0, y: L };

const O = { x: cx, y: cy };

export interface Point {
  x: number;
  y: number;
}

// ビューA: 上面=U, 前面左=F, 前面右=R
const CORNERS_VIEW_A: Record<string, Point[]> = {
  U: [
    { x: Math.round(O.x + vL.x + vR.x), y: Math.round(O.y + vL.y + vR.y) }, // topLeft: B-L-U
    { x: Math.round(O.x + vR.x), y: Math.round(O.y + vR.y) }, // topRight: B-R-U
    { x: Math.round(O.x), y: Math.round(O.y) }, // bottomRight: F-R-U
    { x: Math.round(O.x + vL.x), y: Math.round(O.y + vL.y) }, // bottomLeft: F-L-U
  ],
  F: [
    { x: Math.round(O.x + vL.x), y: Math.round(O.y + vL.y) }, // topLeft: F-L-U
    { x: Math.round(O.x), y: Math.round(O.y) }, // topRight: F-R-U
    { x: Math.round(O.x + vD.x), y: Math.round(O.y + vD.y) }, // bottomRight: F-R-D
    { x: Math.round(O.x + vL.x + vD.x), y: Math.round(O.y + vL.y + vD.y) }, // bottomLeft: F-L-D
  ],
  R: [
    { x: Math.round(O.x), y: Math.round(O.y) }, // topLeft: F-R-U
    { x: Math.round(O.x + vR.x), y: Math.round(O.y + vR.y) }, // topRight: B-R-U
    { x: Math.round(O.x + vR.x + vD.x), y: Math.round(O.y + vR.y + vD.y) }, // bottomRight: B-R-D
    { x: Math.round(O.x + vD.x), y: Math.round(O.y + vD.y) }, // bottomLeft: F-R-D
  ],
};

// ビューB: 上面=D, 前面左=L, 前面右=B
const CORNERS_VIEW_B: Record<string, Point[]> = {
  D: [
    { x: Math.round(O.x + vL.x), y: Math.round(O.y + vL.y) }, // topLeft: F-L-D
    { x: Math.round(O.x + vL.x + vR.x), y: Math.round(O.y + vL.y + vR.y) }, // topRight: F-R-D
    { x: Math.round(O.x + vR.x), y: Math.round(O.y + vR.y) }, // bottomRight: B-R-D
    { x: Math.round(O.x), y: Math.round(O.y) }, // bottomLeft: B-L-D
  ],
  L: [
    { x: Math.round(O.x + vD.x), y: Math.round(O.y + vD.y) }, // topLeft: B-L-U
    { x: Math.round(O.x + vL.x + vD.x), y: Math.round(O.y + vL.y + vD.y) }, // topRight: F-L-U
    { x: Math.round(O.x + vL.x), y: Math.round(O.y + vL.y) }, // bottomRight: F-L-D
    { x: Math.round(O.x), y: Math.round(O.y) }, // bottomLeft: B-L-D
  ],
  B: [
    { x: Math.round(O.x + vR.x + vD.x), y: Math.round(O.y + vR.y + vD.y) }, // topLeft: B-R-U
    { x: Math.round(O.x + vD.x), y: Math.round(O.y + vD.y) }, // topRight: B-L-U
    { x: Math.round(O.x), y: Math.round(O.y) }, // bottomRight: B-L-D
    { x: Math.round(O.x + vR.x), y: Math.round(O.y + vR.y) }, // bottomLeft: B-R-D
  ],
};

interface CubeState {
  name: string;
  state: string;
}

const CUBE_STATES: CubeState[] = [
  {
    name: "solved",
    state: "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB",
  },
  {
    name: "superflip",
    state: "RFUUUUDDFUBBRRFFLFRRRFFBFFDDDRDDUDDUURBLLLLLLLRBUBBLBB",
  },
  {
    name: "scrambled-1",
    state: "DRBUULUBRDBLDRLFLFBDFUFLDRRRBUBDFUDLBFRDLRLUBUFLUBRDFF",
  },
  {
    name: "mixed-colors",
    state: "UDUDUDUDURLRLRLRLRFBFBFBFBFDUDUDUDUDLRLRLRLRLBFBFBFBFB",
  },
  {
    name: "partial",
    state: "UUUU?UUUURRRR?RRRR????F????????D????LLLL?LLLL????B????",
    // 固定センターモデルのため、エディタ反映時の期待値は未認識センターが面の色に補正される
    expectedState: "UUUUUUUUURRRRRRRRR????F????????D????LLLLLLLLL????B????",
  },
];

function interpolate(p1: Point, p2: Point, t: number): Point {
  return {
    x: p1.x + (p2.x - p1.x) * t,
    y: p1.y + (p2.y - p1.y) * t,
  };
}

function getPoint(corners: Point[], u: number, v: number): Point {
  const [topLeft, topRight, bottomRight, bottomLeft] = corners;
  const top = interpolate(topLeft, topRight, u);
  const bottom = interpolate(bottomLeft, bottomRight, u);
  return interpolate(top, bottom, v);
}

function drawRoundedQuad(
  ctx: ReturnType<ReturnType<typeof createCanvas>["getContext"]>,
  p0: Point,
  p1: Point,
  p2: Point,
  p3: Point,
  roundness = 0.2,
) {
  const pts = [p0, p1, p2, p3];
  ctx.beginPath();
  for (let i = 0; i < 4; i++) {
    const prev = pts[(i + 3) % 4];
    const curr = pts[i];
    const next = pts[(i + 1) % 4];
    const start = interpolate(curr, prev, roundness);
    const end = interpolate(curr, next, roundness);
    if (i === 0) {
      ctx.moveTo(start.x, start.y);
    } else {
      ctx.lineTo(start.x, start.y);
    }
    ctx.quadraticCurveTo(curr.x, curr.y, end.x, end.y);
  }
  ctx.closePath();
}

function renderPhotoRealisticCube(
  ctx: ReturnType<ReturnType<typeof createCanvas>["getContext"]>,
  cornersView: Record<string, Point[]>,
  face1: string,
  face2: string,
  face3: string,
  colors1: string,
  colors2: string,
  colors3: string,
) {
  // 1. 背景
  const bg = ctx.createLinearGradient(0, 0, WIDTH, HEIGHT);
  bg.addColorStop(0, "#eae6de");
  bg.addColorStop(0.35, "#dad4c7");
  bg.addColorStop(1, "#bcb5a6");
  ctx.fillStyle = bg;
  ctx.fillRect(0, 0, WIDTH, HEIGHT);

  ctx.save();
  ctx.strokeStyle = "rgba(70, 60, 50, 0.025)";
  ctx.lineWidth = 1;
  for (let y = 0; y < HEIGHT; y += 3) {
    ctx.beginPath();
    ctx.moveTo(0, y);
    ctx.lineTo(WIDTH, y + Math.sin(y * 0.03) * 2);
    ctx.stroke();
  }
  ctx.restore();

  // 2. 接地影
  ctx.save();
  const shadowGrad = ctx.createRadialGradient(
    O.x,
    O.y + vD.y + 12,
    15,
    O.x,
    O.y + vD.y + 15,
    195,
  );
  shadowGrad.addColorStop(0, "rgba(20, 18, 22, 0.65)");
  shadowGrad.addColorStop(0.3, "rgba(40, 38, 45, 0.38)");
  shadowGrad.addColorStop(0.7, "rgba(80, 75, 85, 0.12)");
  shadowGrad.addColorStop(1, "rgba(100, 95, 105, 0)");
  ctx.fillStyle = shadowGrad;
  ctx.beginPath();
  ctx.ellipse(O.x, O.y + vD.y + 14, 190, 64, 0, 0, Math.PI * 2);
  ctx.fill();
  ctx.restore();

  // 3. キューブ本体（黒プラスチックフレーム・角丸外枠）
  const drawFaceBase = (corners: Point[], shade: string) => {
    ctx.save();
    ctx.fillStyle = shade;
    drawRoundedQuad(ctx, corners[0], corners[1], corners[2], corners[3], 0.05);
    ctx.fill();
    ctx.strokeStyle = "#08080a";
    ctx.lineWidth = 2;
    ctx.stroke();
    ctx.restore();
  };

  drawFaceBase(cornersView[face1], "#222528");
  drawFaceBase(cornersView[face2], "#18191c");
  drawFaceBase(cornersView[face3], "#111214");

  // 4. 実写風ステッカー描画（写真通りの角丸＆センター八角形）
  const drawStickers = (corners: Point[], faceColors: string) => {
    const margin = 0.07;
    for (let r = 0; r < 3; r++) {
      for (let c = 0; c < 3; c++) {
        const colorChar = faceColors[r * 3 + c] || "?";
        const hex = COLORS[colorChar] || COLORS["?"];

        const u0 = c / 3 + margin / 3;
        const u1 = (c + 1) / 3 - margin / 3;
        const v0 = r / 3 + margin / 3;
        const v1 = (r + 1) / 3 - margin / 3;

        const p00 = getPoint(corners, u0, v0);
        const p10 = getPoint(corners, u1, v0);
        const p11 = getPoint(corners, u1, v1);
        const p01 = getPoint(corners, u0, v1);

        const isCenter = r === 1 && c === 1;
        const roundness = isCenter ? 0.38 : 0.22;

        ctx.save();
        drawRoundedQuad(ctx, p00, p10, p11, p01, roundness);

        ctx.fillStyle = hex;
        ctx.fill();

        ctx.strokeStyle = "rgba(0, 0, 0, 0.4)";
        ctx.lineWidth = 1.2;
        ctx.stroke();

        const gloss = ctx.createLinearGradient(p00.x, p00.y, p11.x, p11.y);
        gloss.addColorStop(0, "rgba(255, 255, 255, 0.08)");
        gloss.addColorStop(0.4, "rgba(255, 255, 255, 0)");
        gloss.addColorStop(1, "rgba(0, 0, 0, 0.05)");
        ctx.fillStyle = gloss;
        ctx.fill();

        ctx.restore();
      }
    }
  };

  drawStickers(cornersView[face1], colors1);
  drawStickers(cornersView[face2], colors2);
  drawStickers(cornersView[face3], colors3);

  // 5. ヴィネット
  ctx.save();
  const vignette = ctx.createRadialGradient(
    WIDTH / 2,
    HEIGHT / 2,
    WIDTH * 0.35,
    WIDTH / 2,
    HEIGHT / 2,
    WIDTH * 0.65,
  );
  vignette.addColorStop(0, "rgba(0, 0, 0, 0)");
  vignette.addColorStop(1, "rgba(25, 20, 15, 0.22)");
  ctx.fillStyle = vignette;
  ctx.fillRect(0, 0, WIDTH, HEIGHT);
  ctx.restore();
}

function generateTwoViewImage(state: string) {
  const faceU = state.slice(0, 9);
  const faceR = state.slice(9, 18);
  const faceF = state.slice(18, 27);
  const faceD = state.slice(27, 36);
  const faceL = state.slice(36, 45);
  const faceB = state.slice(45, 54);

  const canvasA = createCanvas(WIDTH, HEIGHT);
  const ctxA = canvasA.getContext("2d");
  renderPhotoRealisticCube(
    ctxA,
    CORNERS_VIEW_A,
    "U",
    "F",
    "R",
    faceU,
    faceF,
    faceR,
  );

  const canvasB = createCanvas(WIDTH, HEIGHT);
  const ctxB = canvasB.getContext("2d");
  renderPhotoRealisticCube(
    ctxB,
    CORNERS_VIEW_B,
    "D",
    "L",
    "B",
    faceD,
    faceL,
    faceB,
  );

  return {
    viewA: canvasA.toBuffer("image/png"),
    viewB: canvasB.toBuffer("image/png"),
  };
}

export async function generateTestImages() {
  const outputDir = path.join(__dirname, "../test-images");

  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
  }

  const imageManifest: Record<
    string,
    { viewA: string; viewB: string; expectedState: string }
  > = {};

  for (const cubeState of CUBE_STATES) {
    const { viewA, viewB } = generateTwoViewImage(cubeState.state);

    const viewAPath = path.join(outputDir, `${cubeState.name}-view-a.png`);
    const viewBPath = path.join(outputDir, `${cubeState.name}-view-b.png`);

    fs.writeFileSync(viewAPath, viewA);
    fs.writeFileSync(viewBPath, viewB);

    imageManifest[cubeState.name] = {
      viewA: `${cubeState.name}-view-a.png`,
      viewB: `${cubeState.name}-view-b.png`,
      expectedState: cubeState.expectedState ?? cubeState.state,
    };
  }

  const manifestPath = path.join(outputDir, "manifest.json");
  fs.writeFileSync(
    manifestPath,
    JSON.stringify(
      {
        generated: new Date().toISOString(),
        imageWidth: WIDTH,
        imageHeight: HEIGHT,
        totalStates: CUBE_STATES.length,
        views: {
          A: {
            faces: ["U", "R", "F"],
            corners: CORNERS_VIEW_A,
          },
          B: {
            faces: ["D", "L", "B"],
            corners: CORNERS_VIEW_B,
          },
        },
        images: imageManifest,
      },
      null,
      2,
    ),
  );
}
