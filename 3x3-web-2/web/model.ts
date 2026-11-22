export const FACES = "URFDLB";
import { CORNERS_2X2, SOLVED_2X2 } from "./model-2x2";
export { SOLVED_2X2 } from "./model-2x2";

export const SOLVED_3X3 = [...FACES].map((f) => f.repeat(9)).join("");
export const SOLVED = SOLVED_3X3;

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
export const FACE_NAMES_EN: Record<string, string> = {
  U: "上面 (Up)",
  R: "右面 (Right)",
  F: "前面 (Front)",
  D: "下面 (Down)",
  L: "左面 (Left)",
  B: "背面 (Back)",
};

export type CubeType = "2x2" | "3x3";

export function getSolvedState(type: CubeType): string {
  return type === "2x2" ? SOLVED_2X2 : SOLVED_3X3;
}

export function getStickerCount(type: CubeType): number {
  return type === "2x2" ? 24 : 54;
}

export function getGridSize(type: CubeType): number {
  return type === "2x2" ? 2 : 3;
}

export type SolverAlgorithm =
  | "kociemba"
  | "cfop"
  | "thistlethwaite"
  | "korf"
  | "optimal"
  | "lbl"
  | "ortega";

export interface PhaseInfo {
  name: string;
  start: number;
  end: number;
}

export interface ResultData {
  state: string;
  moves: string[];
  states: string[];
  elapsed_ms: number;
  nodes: number;
  algorithm?: string;
  phases?: PhaseInfo[];
}
export interface Request {
  id: number;
  revision: number;
  kind: "solve";
  state: string;
  budget: number;
  cubeType?: CubeType;
  includeOrientation?: boolean;
  centerRotations?: number[];
  algorithm?: SolverAlgorithm;
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

export function getErrorIndices(
  message?: string,
  cubeType: CubeType = "3x3",
): number[] {
  if (!message) return [];
  if (cubeType === "2x2") {
    // 2x2 のエラー文は「スロット N のコーナー配色…」形式（N は 1 始まり）
    const slotMatch = message.match(/スロット\s*(\d+)\s*のコーナー/);
    if (slotMatch) {
      const slot = parseInt(slotMatch[1], 10) - 1;
      if (slot >= 0 && slot < CORNERS_2X2.length) {
        return [...CORNERS_2X2[slot]];
      }
    }
    return [];
  }
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

const CHAR_ID = (() => {
  const table = new Uint8Array(128).fill(7);
  for (let i = 0; i < FACES.length; i++) {
    table[FACES.charCodeAt(i)] = i;
  }
  return table;
})();

const CORNER_PERMS: readonly [number, number, number][] = [
  [0, 1, 2],
  [0, 2, 1],
  [1, 0, 2],
  [1, 2, 0],
  [2, 0, 1],
  [2, 1, 0],
];

const CORNER_ENTRY_ID = new Int16Array(512).fill(-1);
const CORNER_ORIG_PIECE = new Int8Array(48);
// [slot(8)][entryId(48)][k(3)] のステッカー基準上方向からの回転角キャッシュ
const CORNER_ANGLES_LUT = new Float64Array(8 * 48 * 3);

const EDGE_ENTRY_ID = new Int16Array(64).fill(-1);
const EDGE_ORIG_PIECE = new Int8Array(24);
// [slot(12)][entryId(24)][k(2)] のステッカー基準上方向からの回転角キャッシュ
const EDGE_ANGLES_LUT = new Float64Array(12 * 24 * 2);

function normalizeAnglePi(angle: number): number {
  return Math.abs(angle + Math.PI) < 1e-4 ? Math.PI : angle;
}

(function initArrowTables() {
  let cornerEntryId = 0;
  for (let p = 0; p < CORNERS.length; p++) {
    const origCols = CORNERS[p].map((idx) => CHAR_ID[SOLVED.charCodeAt(idx)]);
    for (const perm of CORNER_PERMS) {
      const code =
        (origCols[perm[0]] << 6) | (origCols[perm[1]] << 3) | origCols[perm[2]];
      CORNER_ENTRY_ID[code] = cornerEntryId;
      CORNER_ORIG_PIECE[cornerEntryId] = p;

      // perm[slotPos] = origIdx の逆写像 sigma[origIdx] = slotPos
      const sigma = [0, 0, 0];
      for (let pos = 0; pos < 3; pos++) {
        sigma[perm[pos]] = pos;
      }

      const origStickers = CORNERS[p];
      const nNormals = origStickers.map((idx) => NORMALS[Math.floor(idx / 9)]);
      const vUps = origStickers.map((idx) => FACE_UPS[Math.floor(idx / 9)]);

      for (let slot = 0; slot < CORNERS.length; slot++) {
        const slotStickers = CORNERS[slot];
        const mNormals = slotStickers.map(
          (idx) => NORMALS[Math.floor(idx / 9)],
        );
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
        const base = (slot * 48 + cornerEntryId) * 3;
        for (let k = 0; k < 3; k++) {
          const curFace = Math.floor(slotStickers[sigma[k]] / 9);
          const vOrig = vUps[k];
          const w: Vec3 = [
            R[0][0] * vOrig[0] + R[0][1] * vOrig[1] + R[0][2] * vOrig[2],
            R[1][0] * vOrig[0] + R[1][1] * vOrig[1] + R[1][2] * vOrig[2],
            R[2][0] * vOrig[0] + R[2][1] * vOrig[1] + R[2][2] * vOrig[2],
          ];
          const cosVal = dot(w, FACE_UPS[curFace]);
          const sinVal = dot(w, FACE_RIGHTS[curFace]);
          CORNER_ANGLES_LUT[base + sigma[k]] = normalizeAnglePi(
            Math.atan2(sinVal, cosVal),
          );
        }
      }
      cornerEntryId++;
    }
  }

  let edgeEntryId = 0;
  for (let p = 0; p < EDGES.length; p++) {
    const origCols = EDGES[p].map((idx) => CHAR_ID[SOLVED.charCodeAt(idx)]);
    const origStickers = EDGES[p];
    const n0 = NORMALS[Math.floor(origStickers[0] / 9)];
    const n1 = NORMALS[Math.floor(origStickers[1] / 9)];
    const n2 = cross(n0, n1);
    const vUps = origStickers.map((idx) => FACE_UPS[Math.floor(idx / 9)]);

    for (const perm of [
      [0, 1],
      [1, 0],
    ] as const) {
      const code = (origCols[perm[0]] << 3) | origCols[perm[1]];
      EDGE_ENTRY_ID[code] = edgeEntryId;
      EDGE_ORIG_PIECE[edgeEntryId] = p;

      const sigma0 = perm[0] === 0 ? 0 : 1;
      const sigma1 = perm[1] === 0 ? 0 : 1;

      for (let slot = 0; slot < EDGES.length; slot++) {
        const slotStickers = EDGES[slot];
        const m0 = NORMALS[Math.floor(slotStickers[0] / 9)];
        const m1 = NORMALS[Math.floor(slotStickers[1] / 9)];
        const curN0 = sigma0 === 0 ? m0 : m1;
        const curN1 = sigma1 === 0 ? m0 : m1;
        const curN2 = cross(curN0, curN1);

        const R: [Vec3, Vec3, Vec3] = [
          [0, 0, 0],
          [0, 0, 0],
          [0, 0, 0],
        ];
        for (let row = 0; row < 3; row++) {
          for (let col = 0; col < 3; col++) {
            R[row][col] =
              curN0[row] * n0[col] +
              curN1[row] * n1[col] +
              curN2[row] * n2[col];
          }
        }
        const base = (slot * 24 + edgeEntryId) * 2;
        const sigmas = [sigma0, sigma1];
        for (let k = 0; k < 2; k++) {
          const curFace = Math.floor(slotStickers[sigmas[k]] / 9);
          const vOrig = vUps[k];
          const w: Vec3 = [
            R[0][0] * vOrig[0] + R[0][1] * vOrig[1] + R[0][2] * vOrig[2],
            R[1][0] * vOrig[0] + R[1][1] * vOrig[1] + R[1][2] * vOrig[2],
            R[2][0] * vOrig[0] + R[2][1] * vOrig[1] + R[2][2] * vOrig[2],
          ];
          const cosVal = dot(w, FACE_UPS[curFace]);
          const sinVal = dot(w, FACE_RIGHTS[curFace]);
          EDGE_ANGLES_LUT[base + sigmas[k]] = normalizeAnglePi(
            Math.atan2(sinVal, cosVal),
          );
        }
      }
      edgeEntryId++;
    }
  }
})();

export function getCellArrowInfo(
  state: string,
  centerRotations: number[] = [0, 0, 0, 0, 0, 0],
): CellArrowInfo {
  const angles = new Array<number>(54).fill(0);
  const colors = new Array<number>(54).fill(ARROW_COLORS.NORMAL);
  const kinds = new Array<"corner" | "edge" | "center">(54).fill("center");
  const pieceIndices = new Array<number>(54).fill(0);

  if (!state || state.length < 54) {
    return { angles, colors, kinds, pieceIndices };
  }

  // 1. コーナーピース（8ピース）
  for (let slot = 0; slot < 8; slot++) {
    const slotStickers = CORNERS[slot];
    const s0 = slotStickers[0];
    const s1 = slotStickers[1];
    const s2 = slotStickers[2];
    const c0 = state.charCodeAt(s0);
    const c1 = state.charCodeAt(s1);
    const c2 = state.charCodeAt(s2);
    const ci0 = c0 < 128 ? CHAR_ID[c0] : 7;
    const ci1 = c1 < 128 ? CHAR_ID[c1] : 7;
    const ci2 = c2 < 128 ? CHAR_ID[c2] : 7;
    const entryId =
      ci0 <= 5 && ci1 <= 5 && ci2 <= 5
        ? CORNER_ENTRY_ID[(ci0 << 6) | (ci1 << 3) | ci2]
        : -1;

    if (entryId < 0) {
      kinds[s0] = "corner";
      kinds[s1] = "corner";
      kinds[s2] = "corner";
      pieceIndices[s0] = slot;
      pieceIndices[s1] = slot;
      pieceIndices[s2] = slot;
      continue;
    }

    const origPiece = CORNER_ORIG_PIECE[entryId];
    const base = (slot * 48 + entryId) * 3;
    const a0 = CORNER_ANGLES_LUT[base];
    const a1 = CORNER_ANGLES_LUT[base + 1];
    const a2 = CORNER_ANGLES_LUT[base + 2];

    angles[s0] = a0;
    angles[s1] = a1;
    angles[s2] = a2;
    colors[s0] =
      Math.abs(a0) < 1e-4 ? ARROW_COLORS.NORMAL : ARROW_COLORS.CORNER_TWIST;
    colors[s1] =
      Math.abs(a1) < 1e-4 ? ARROW_COLORS.NORMAL : ARROW_COLORS.CORNER_TWIST;
    colors[s2] =
      Math.abs(a2) < 1e-4 ? ARROW_COLORS.NORMAL : ARROW_COLORS.CORNER_TWIST;
    kinds[s0] = "corner";
    kinds[s1] = "corner";
    kinds[s2] = "corner";
    pieceIndices[s0] = origPiece;
    pieceIndices[s1] = origPiece;
    pieceIndices[s2] = origPiece;
  }

  // 2. エッジピース（12ピース）
  for (let slot = 0; slot < 12; slot++) {
    const slotStickers = EDGES[slot];
    const s0 = slotStickers[0];
    const s1 = slotStickers[1];
    const c0 = state.charCodeAt(s0);
    const c1 = state.charCodeAt(s1);
    const ci0 = c0 < 128 ? CHAR_ID[c0] : 7;
    const ci1 = c1 < 128 ? CHAR_ID[c1] : 7;
    const entryId = ci0 <= 5 && ci1 <= 5 ? EDGE_ENTRY_ID[(ci0 << 3) | ci1] : -1;

    if (entryId < 0) {
      kinds[s0] = "edge";
      kinds[s1] = "edge";
      pieceIndices[s0] = slot;
      pieceIndices[s1] = slot;
      continue;
    }

    const origPiece = EDGE_ORIG_PIECE[entryId];
    const base = (slot * 24 + entryId) * 2;
    const a0 = EDGE_ANGLES_LUT[base];
    const a1 = EDGE_ANGLES_LUT[base + 1];

    angles[s0] = a0;
    angles[s1] = a1;
    colors[s0] =
      Math.abs(a0) < 1e-4 ? ARROW_COLORS.NORMAL : ARROW_COLORS.EDGE_FLIP;
    colors[s1] =
      Math.abs(a1) < 1e-4 ? ARROW_COLORS.NORMAL : ARROW_COLORS.EDGE_FLIP;
    kinds[s0] = "edge";
    kinds[s1] = "edge";
    pieceIndices[s0] = origPiece;
    pieceIndices[s1] = origPiece;
  }

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

export interface PieceOrientationCounts {
  normal: number;
  cornerTwist: number;
  edgeFlip: number;
  centerRotate: number;
}

export function countPieceOrientations(
  arrowInfo: CellArrowInfo,
): PieceOrientationCounts {
  let normal = 0;
  let cornerMask = 0;
  let edgeMask = 0;
  let centerRotate = 0;

  for (let i = 0; i < 54; i++) {
    const c = arrowInfo.colors[i];
    if (c === ARROW_COLORS.NORMAL) {
      normal++;
    } else if (c === ARROW_COLORS.CORNER_TWIST) {
      cornerMask |= 1 << arrowInfo.pieceIndices[i];
    } else if (c === ARROW_COLORS.EDGE_FLIP) {
      edgeMask |= 1 << arrowInfo.pieceIndices[i];
    } else if (c === ARROW_COLORS.CENTER_ROTATE) {
      centerRotate++;
    }
  }

  let cornerTwist = 0;
  while (cornerMask) {
    cornerMask &= cornerMask - 1;
    cornerTwist++;
  }
  let edgeFlip = 0;
  while (edgeMask) {
    edgeMask &= edgeMask - 1;
    edgeFlip++;
  }

  return {
    normal,
    cornerTwist,
    edgeFlip,
    centerRotate,
  };
}

export interface SolverMeta {
  id: SolverAlgorithm;
  label: string;
  shortDesc: string;
}

export const SOLVER_META_BY_TYPE: Record<CubeType, readonly SolverMeta[]> = {
  "3x3": [
    { id: "kociemba", label: "Kociemba", shortDesc: "2段階探索" },
    { id: "cfop", label: "LBL / CFOP", shortDesc: "人間向け階層解法" },
    { id: "thistlethwaite", label: "Thistlethwaite", shortDesc: "4段階群縮小" },
    { id: "korf", label: "Korf (IDA*)", shortDesc: "最短探索+フォールバック" },
  ],
  "2x2": [
    { id: "optimal", label: "Optimal (IDA*)", shortDesc: "完全最短 (≤11手)" },
    { id: "ortega", label: "Ortega法", shortDesc: "スピード3ステップ" },
    { id: "lbl", label: "LBL法", shortDesc: "初心者向け3ステップ" },
  ],
};
