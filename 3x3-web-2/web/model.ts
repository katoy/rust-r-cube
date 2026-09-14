export const FACES = "URFDLB";
export const SOLVED = [...FACES].map((f) => f.repeat(9)).join("");
export const COLORS: Record<string, string> = {
  U: "#eeeade",
  R: "#e55649",
  F: "#74b89a",
  D: "#efce66",
  L: "#ec9851",
  B: "#6a9edb",
  "?": "#454b49",
};
export const NAMES: Record<string, string> = {
  U: "白",
  R: "赤",
  F: "緑",
  D: "黄",
  L: "橙",
  B: "青",
  "?": "未入力",
};
export const FACE_NAMES: Record<string, string> = {
  U: "上面",
  R: "右面",
  F: "前面",
  D: "下面",
  L: "左面",
  B: "背面",
};
export interface ResultData {
  state: string;
  moves: string[];
  states: string[];
  elapsed_ms: number;
  nodes: number;
}
export interface Request {
  id: number;
  revision: number;
  kind: "solve";
  state: string;
  budget: number;
  includeOrientation?: boolean;
  centerRotations?: number[];
}

export type Reply =
  | { kind: "ready"; elapsed: number }
  | { kind: "init-error"; error: string }
  | {
      kind: "result";
      id: number;
      revision: number;
      result?: ResultData;
      error?: string;
    };
export function inverse(move: string) {
  return move.endsWith("2") ? move : move.endsWith("'") ? move[0] : `${move}'`;
}
export function instruction(move: string) {
  return `${FACE_NAMES[move[0]]}を、その面から見て${move.endsWith("2") ? "180°" : move.endsWith("'") ? "反時計回りに90°" : "時計回りに90°"}回す`;
}

export const CORNERS = [
  [8, 9, 20],
  [6, 18, 38],
  [0, 36, 47],
  [2, 45, 11],
  [29, 26, 15],
  [27, 44, 24],
  [33, 53, 42],
  [35, 17, 51],
];

export const EDGES = [
  [5, 10],
  [7, 19],
  [3, 37],
  [1, 46],
  [32, 16],
  [28, 25],
  [30, 43],
  [34, 52],
  [23, 12],
  [21, 41],
  [50, 39],
  [48, 14],
];

export function getErrorIndices(message?: string): number[] {
  if (!message) return [];
  const edgeMatch = message.match(/エッジ\s*(\d+)/);
  if (edgeMatch) {
    const slot = parseInt(edgeMatch[1], 10) - 1;
    if (slot >= 0 && slot < EDGES.length) {
      return [...EDGES[slot]];
    }
  }
  const cornerMatch = message.match(/コーナー\s*(\d+)/);
  if (cornerMatch) {
    const slot = parseInt(cornerMatch[1], 10) - 1;
    if (slot >= 0 && slot < CORNERS.length) {
      return [...CORNERS[slot]];
    }
  }
  return [];
}

export const ARROW_COLORS = {
  NORMAL: 0x10b981, // エメラルドグリーン
  CORNER_TWIST: 0xf43f5e, // ローズピンク（赤・橙ステッカーと被らず高コントラスト）
  EDGE_FLIP: 0x06b6d4, // エレクトリックシアン（青ステッカーと被らず高輝度）
  CENTER_ROTATE: 0xf59e0b, // アンバーゴールド
  OUTLINE: 0x141a18, // 暗色アウトライン（全6面の地色から矢印をくっきり分離）
};

export interface CellArrowInfo {
  angles: number[]; // 54セルの基準上方向からの回転角（rad, 時計回り正）
  colors: number[]; // 54セルの色
  kinds: ("corner" | "edge" | "center")[];
  pieceIndices: number[];
}

type Vec3 = [number, number, number];
const dot = (a: Vec3, b: Vec3) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
const cross = (a: Vec3, b: Vec3): Vec3 => [
  a[1] * b[2] - a[2] * b[1],
  a[2] * b[0] - a[0] * b[2],
  a[0] * b[1] - a[1] * b[0],
];

const NORMALS: Vec3[] = [
  [0, 1, 0], // U
  [1, 0, 0], // R
  [0, 0, 1], // F
  [0, -1, 0], // D
  [-1, 0, 0], // L
  [0, 0, -1], // B
];

const FACE_UPS: Vec3[] = [
  [0, 0, -1], // U: 奥方向
  [0, 1, 0], // R: 上方向
  [0, 1, 0], // F: 上方向
  [0, 0, 1], // D: 手前方向
  [0, 1, 0], // L: 上方向
  [0, 1, 0], // B: 上方向
];

const FACE_RIGHTS: Vec3[] = NORMALS.map((n, i) => cross(FACE_UPS[i], n));

export function getCellArrowInfo(
  state: string,
  centerRotations: number[] = [0, 0, 0, 0, 0, 0],
): CellArrowInfo {
  const angles = new Array<number>(54).fill(0);
  const colors = new Array<number>(54).fill(0x2ecc71);
  const kinds = new Array<"corner" | "edge" | "center">(54).fill("center");
  const pieceIndices = new Array<number>(54).fill(0);

  if (!state || state.length < 54) {
    return { angles, colors, kinds, pieceIndices };
  }

  // 1. コーナーピース（8ピース）
  CORNERS.forEach((slotStickers, slot) => {
    // 現在のスロットにある3つのステッカーの色
    const currentCols = slotStickers.map((idx) => state[idx]);
    const mNormals = slotStickers.map((idx) => NORMALS[Math.floor(idx / 9)]);

    // 元のどのコーナーピースか探す（色の一致）
    let origPiece = -1;
    for (let p = 0; p < CORNERS.length; p++) {
      const origCols = CORNERS[p].map((idx) => SOLVED[idx]);
      if (
        origCols.every((c) => currentCols.includes(c)) &&
        currentCols.every((c) => origCols.includes(c))
      ) {
        origPiece = p;
        break;
      }
    }

    if (origPiece === -1) {
      // 不正な状態または未入力の場合
      slotStickers.forEach((idx) => {
        kinds[idx] = "corner";
        pieceIndices[idx] = slot;
      });
      return;
    }

    const origStickers = CORNERS[origPiece];
    const nNormals = origStickers.map((idx) => NORMALS[Math.floor(idx / 9)]);
    const vUps = origStickers.map((idx) => FACE_UPS[Math.floor(idx / 9)]);

    // 元のステッカー k が現在のスロットのどの位置 sigma[k] にあるか
    const sigma: number[] = [];
    for (let k = 0; k < 3; k++) {
      const origCol = SOLVED[origStickers[k]];
      const curPos = currentCols.indexOf(origCol);
      sigma.push(curPos);
    }

    // 剛体回転行列 R: R[row][col] = sum_k m[sigma[k]][row] * n[k][col]
    const R: [Vec3, Vec3, Vec3] = [
      [0, 0, 0],
      [0, 0, 0],
      [0, 0, 0],
    ];
    for (let row = 0; row < 3; row++) {
      for (let col = 0; col < 3; col++) {
        let sum = 0;
        for (let k = 0; k < 3; k++) {
          sum += mNormals[sigma[k]][row] * nNormals[k][col];
        }
        R[row][col] = sum;
      }
    }

    // 各ステッカーの回転角を計算
    for (let k = 0; k < 3; k++) {
      const curSlotStickerIdx = slotStickers[sigma[k]];
      const curFace = Math.floor(curSlotStickerIdx / 9);
      const vOrig = vUps[k];

      // 回転後の矢印ベクトル w = R * vOrig
      const w: Vec3 = [
        R[0][0] * vOrig[0] + R[0][1] * vOrig[1] + R[0][2] * vOrig[2],
        R[1][0] * vOrig[0] + R[1][1] * vOrig[1] + R[1][2] * vOrig[2],
        R[2][0] * vOrig[0] + R[2][1] * vOrig[1] + R[2][2] * vOrig[2],
      ];

      const uCur = FACE_UPS[curFace];
      const rCur = FACE_RIGHTS[curFace];
      const cosVal = dot(w, uCur);
      const sinVal = dot(w, rCur);
      let angle = Math.atan2(sinVal, cosVal);
      // -π を +π に正規化
      if (Math.abs(angle + Math.PI) < 1e-4) angle = Math.PI;

      angles[curSlotStickerIdx] = angle;
      colors[curSlotStickerIdx] =
        Math.abs(angle) < 1e-4
          ? ARROW_COLORS.NORMAL
          : ARROW_COLORS.CORNER_TWIST;
      kinds[curSlotStickerIdx] = "corner";
      pieceIndices[curSlotStickerIdx] = origPiece;
    }
  });

  // 2. エッジピース（12ピース）
  EDGES.forEach((slotStickers, slot) => {
    const currentCols = slotStickers.map((idx) => state[idx]);
    const m0 = NORMALS[Math.floor(slotStickers[0] / 9)];
    const m1 = NORMALS[Math.floor(slotStickers[1] / 9)];

    let origPiece = -1;
    for (let p = 0; p < EDGES.length; p++) {
      const origCols = EDGES[p].map((idx) => SOLVED[idx]);
      if (
        origCols.every((c) => currentCols.includes(c)) &&
        currentCols.every((c) => origCols.includes(c))
      ) {
        origPiece = p;
        break;
      }
    }

    if (origPiece === -1) {
      slotStickers.forEach((idx) => {
        kinds[idx] = "edge";
        pieceIndices[idx] = slot;
      });
      return;
    }

    const origStickers = EDGES[origPiece];
    const n0 = NORMALS[Math.floor(origStickers[0] / 9)];
    const n1 = NORMALS[Math.floor(origStickers[1] / 9)];
    const n2 = cross(n0, n1);
    const vUps = origStickers.map((idx) => FACE_UPS[Math.floor(idx / 9)]);

    const sigma: number[] = [];
    for (let k = 0; k < 2; k++) {
      const origCol = SOLVED[origStickers[k]];
      sigma.push(currentCols.indexOf(origCol));
    }

    const curN0 = sigma[0] === 0 ? m0 : m1;
    const curN1 = sigma[1] === 0 ? m0 : m1;
    const curN2 = cross(curN0, curN1);

    // 回転行列 R
    const R: [Vec3, Vec3, Vec3] = [
      [0, 0, 0],
      [0, 0, 0],
      [0, 0, 0],
    ];
    for (let row = 0; row < 3; row++) {
      for (let col = 0; col < 3; col++) {
        R[row][col] =
          curN0[row] * n0[col] + curN1[row] * n1[col] + curN2[row] * n2[col];
      }
    }

    for (let k = 0; k < 2; k++) {
      const curSlotStickerIdx = slotStickers[sigma[k]];
      const curFace = Math.floor(curSlotStickerIdx / 9);
      const vOrig = vUps[k];

      const w: Vec3 = [
        R[0][0] * vOrig[0] + R[0][1] * vOrig[1] + R[0][2] * vOrig[2],
        R[1][0] * vOrig[0] + R[1][1] * vOrig[1] + R[1][2] * vOrig[2],
        R[2][0] * vOrig[0] + R[2][1] * vOrig[1] + R[2][2] * vOrig[2],
      ];

      const uCur = FACE_UPS[curFace];
      const rCur = FACE_RIGHTS[curFace];
      const cosVal = dot(w, uCur);
      const sinVal = dot(w, rCur);
      let angle = Math.atan2(sinVal, cosVal);
      if (Math.abs(angle + Math.PI) < 1e-4) angle = Math.PI;

      angles[curSlotStickerIdx] = angle;
      colors[curSlotStickerIdx] =
        Math.abs(angle) < 1e-4 ? ARROW_COLORS.NORMAL : ARROW_COLORS.EDGE_FLIP;
      kinds[curSlotStickerIdx] = "edge";
      pieceIndices[curSlotStickerIdx] = origPiece;
    }
  });

  // 3. センターピース（6面）
  for (let f = 0; f < 6; f++) {
    const idx = f * 9 + 4;
    let angle = (centerRotations[f] ?? 0) % (2 * Math.PI);
    if (angle > Math.PI) angle -= 2 * Math.PI;
    if (angle <= -Math.PI) angle += 2 * Math.PI;

    angles[idx] = angle;
    colors[idx] =
      Math.abs(angle) < 1e-4 ? ARROW_COLORS.NORMAL : ARROW_COLORS.CENTER_ROTATE;
    kinds[idx] = "center";
    pieceIndices[idx] = f;
  }

  return { angles, colors, kinds, pieceIndices };
}
