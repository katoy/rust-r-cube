export const SOLVED_2X2 = "UUUURRRRFFFFDDDDLLLLBBBB";

const VALID_FACE_SET = new Set(["U", "R", "F", "D", "L", "B"]);
const OPPOSITE_FACE: Record<string, string> = {
  U: "D",
  D: "U",
  R: "L",
  L: "R",
  F: "B",
  B: "F",
};

/**
 * 2x2 盤面（24文字）が空間回転を含めて6面すべて単色で揃っているかを判定します。
 */
export function isUniformSolved2x2(state: string): boolean {
  if (state.length !== 24) return false;
  const seenFaces = new Set<string>();
  for (let f = 0; f < 6; f++) {
    const base = f * 4;
    const c = state[base];
    if (!VALID_FACE_SET.has(c)) return false;
    if (
      state[base + 1] !== c ||
      state[base + 2] !== c ||
      state[base + 3] !== c
    ) {
      return false;
    }
    seenFaces.add(c);
  }
  return seenFaces.size === 6;
}

/**
 * 2x2 の DBL スロット（インデックス 14, 23, 18）にあるコーナーの色が
 * (D, B, L) になるように6面の色文字を再ラベリングした24文字盤面を返します。
 */
export function relabel2x2ForDbl(state: string): string {
  if (state.length !== 24) return state;
  const dCol = state[14];
  const bCol = state[23];
  const lCol = state[18];
  if (
    !VALID_FACE_SET.has(dCol) ||
    !VALID_FACE_SET.has(bCol) ||
    !VALID_FACE_SET.has(lCol)
  ) {
    return state;
  }
  const uCol = OPPOSITE_FACE[dCol];
  const fCol = OPPOSITE_FACE[bCol];
  const rCol = OPPOSITE_FACE[lCol];
  const map: Record<string, string> = {
    [dCol]: "D",
    [uCol]: "U",
    [bCol]: "B",
    [fCol]: "F",
    [lCol]: "L",
    [rCol]: "R",
  };
  if (Object.keys(map).length !== 6) return state;
  return [...state].map((ch) => map[ch] ?? ch).join("");
}

/**
 * 2x2 の8個のコーナースロットにある3枚のステッカーのインデックス
 * 順序: [U/D面, 他の面1, 他の面2]
 */
export const CORNERS_2X2 = [
  [3, 4, 9], // 0: UFR (U, R, F)
  [2, 8, 17], // 1: UFL (U, F, L)
  [0, 16, 21], // 2: ULB (U, L, B)
  [1, 20, 5], // 3: UBR (U, B, R)
  [13, 11, 6], // 4: DFR (D, F, R)
  [12, 19, 10], // 5: DLF (D, L, F)
  [14, 23, 18], // 6: DBL (D, B, L)
  [15, 7, 22], // 7: DRB (D, R, B)
];

export interface Preset2x2 {
  id: string;
  label: string;
  emoji: string;
  scramble?: string;
  state?: string;
  description: string;
}

export const PRESETS_2X2: Preset2x2[] = [
  {
    id: "solved",
    label: "完成状態",
    emoji: "✅",
    state: SOLVED_2X2,
    description: "6面すべてが揃った標準状態です。",
  },
  {
    id: "checkerboard",
    label: "チェッカー風",
    emoji: "🏁",
    scramble: "R2 F2 R2 U2",
    description: "対向する面の色が互い違いに入れ替わった幾何学的パターン。",
  },
  {
    id: "easy-6",
    label: "基本手順 (6手)",
    emoji: "🟢",
    scramble: "R U R' U' R U",
    description: "初心者でも覚えやすい短いスクランブル。",
  },
  {
    id: "gods-number-11",
    label: "最難関 (11手)",
    emoji: "⚡",
    scramble: "R U2 R' F2 R U2 R' U F' U R",
    description: "2x2 において最大手数を要する代表的な難関状態。",
  },
];
