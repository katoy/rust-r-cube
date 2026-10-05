export const SOLVED_2X2 = "UUUURRRRFFFFDDDDLLLLBBBB";

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
