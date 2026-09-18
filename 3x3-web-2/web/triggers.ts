export interface MoveMeta {
  move: string;
  phase: 1 | 2;
  phaseLabel: string;
  trigger?: string;
}

// Kociemba Phase 2 (G1群) で許可される移動: U, D, R2, L2, F2, B2
const G1_MOVES = new Set([
  "U",
  "U'",
  "U2",
  "D",
  "D'",
  "D2",
  "R2",
  "L2",
  "F2",
  "B2",
]);

const TRIGGERS: { pattern: string[]; name: string }[] = [
  { pattern: ["R", "U", "R'", "U'"], name: "セクシームーブ" },
  { pattern: ["L'", "U'", "L", "U"], name: "レフトセクシー" },
  { pattern: ["R'", "F", "R", "F'"], name: "スレッジハンマー" },
  { pattern: ["R", "U", "R'"], name: "インサート" },
  { pattern: ["L'", "U'", "L"], name: "レフトインサート" },
];

export function analyzeMoves(moves: string[]): MoveMeta[] {
  if (moves.length === 0) return [];

  // 末尾から見て、連続して G1_MOVES である区間を Phase 2 とする
  let phase2StartIndex = moves.length;
  for (let i = moves.length - 1; i >= 0; i--) {
    if (G1_MOVES.has(moves[i])) {
      phase2StartIndex = i;
    } else {
      break;
    }
  }

  // トリガーの検出
  const triggerMap = new Map<number, string>();
  for (let i = 0; i < moves.length; i++) {
    for (const trig of TRIGGERS) {
      if (i + trig.pattern.length <= moves.length) {
        let match = true;
        for (let j = 0; j < trig.pattern.length; j++) {
          if (moves[i + j] !== trig.pattern[j]) {
            match = false;
            break;
          }
        }
        if (match) {
          for (let j = 0; j < trig.pattern.length; j++) {
            if (!triggerMap.has(i + j)) {
              triggerMap.set(i + j, trig.name);
            }
          }
        }
      }
    }
  }

  return moves.map((move, idx) => {
    const isPhase2 = idx >= phase2StartIndex;
    return {
      move,
      phase: isPhase2 ? 2 : 1,
      phaseLabel: isPhase2 ? "Phase 2: 解決" : "Phase 1: 縮約",
      trigger: triggerMap.get(idx),
    };
  });
}
