import type { PhaseInfo } from "./model";

export interface MoveMeta {
  move: string;
  phase: number;
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
  // 2x2 PBL / OLL 公式
  { pattern: ["R2", "F2", "R2"], name: "PBL ダブル対角" },
  { pattern: ["R", "U", "R'", "U", "R", "U2", "R'"], name: "スーネ (Sune)" },
  { pattern: ["R", "U2", "R'", "U'", "R", "U'", "R'"], name: "アンチスーネ" },
  { pattern: ["R2", "U2", "R", "U2", "R2"], name: "H型 OLL" },
  // 共通トリガー
  { pattern: ["R", "U", "R'", "U'"], name: "セクシームーブ" },
  { pattern: ["L'", "U'", "L", "U"], name: "レフトセクシー" },
  { pattern: ["R'", "F", "R", "F'"], name: "スレッジハンマー" },
  { pattern: ["R", "U", "R'"], name: "インサート" },
  { pattern: ["L'", "U'", "L"], name: "レフトインサート" },
];

const PHASE_LABEL_MAP: Record<string, string> = {
  // 2x2 LBL (Rust: c2x2/lbl.rs)
  "ステップ 1: 完全1層 (First Layer)": "ステップ 1: 完全1層 (First Layer)",
  "ステップ 2: 上面色揃え (OLL)": "ステップ 2: 上面色揃え (OLL)",
  "ステップ 3: 上面位置揃え (PLL)": "ステップ 3: 上面位置揃え (PLL)",

  // 2x2 Ortega (Rust: c2x2/ortega.rs)
  "ステップ 1: 最初の1面 (First Face)": "ステップ 1: 最初の1面 (First Face)",
  "ステップ 2: 反対面色揃え (OLL)": "ステップ 2: 反対面色揃え (OLL)",
  "ステップ 3: 両層同時配置 (PBL)": "ステップ 3: 両層同時配置 (PBL)",

  // 2x2 Optimal
  "Optimal 最短探索 (IDA*)": "Optimal 最短手順探索 (IDA*)",

  // CFOP (Rust: cfop.rs)
  "Cross (クロス)": "ステップ 1: 底面クロス (Cross)",
  "First Layer (第1層コーナー)": "ステップ 2: 完全1層 (First Layer)",
  "Second Layer (中層エッジ)": "ステップ 3: 中層エッジ (Second Layer)",
  "OLL (ラストレイヤー向き)": "ステップ 4: 上面色揃え (OLL)",
  "PLL (ラストレイヤー配置)": "ステップ 5: 上面完全配置 (PLL)",
  Cross: "ステップ 1: 底面クロス (Cross)",
  "First Layer": "ステップ 2: 完全1層 (First Layer)",
  "Second Layer": "ステップ 3: 中層エッジ (Second Layer)",
  OLL: "ステップ 4: 上面色揃え (OLL)",
  PLL: "ステップ 5: 上面完全配置 (PLL)",

  // Thistlethwaite (Rust: thistlethwaite.rs)
  "Phase 1 (G0→G1: エッジ向き)": "第1段階: G0→G1 (エッジ反転解消)",
  "Phase 2 (G1→G2: コーナー向き&Eスライス)":
    "第2段階: G1→G2 (コーナー向き+中層)",
  "Phase 3 (G2→G3: オービット分離)": "第3段階: G2→G3 (角・辺軌道限定)",
  "Phase 4 (G3→G4: 最終解決)": "第4段階: G3→G4 (最終揃え)",
  "Phase 4 (最終解決フォールバック)": "第4段階: 最終揃え (フォールバック)",
  "Phase 1": "第1段階: G0→G1 (エッジ反転解消)",
  "Phase 2": "第2段階: G1→G2 (コーナー向き+中層)",
  "Phase 3": "第3段階: G2→G3 (角・辺軌道限定)",
  "Phase 4": "第4段階: G3→G4 (最終揃え)",

  // Korf (Rust: korf.rs)
  "Korf 最短探索 (IDA*)": "IDA* 最短手順探索",
  "Korf IDA*": "IDA* 最短手順探索",
  "Kociemba フォールバック": "Kociemba フォールバック解",

  // センター向き解決・Kociemba (Rust: lib.rs)
  "同時最適化 (色＆センター)": "同時最適化: 色＆センター向き",
  "色解法 (Kociemba)": "ステップ 1: 色解法 (Kociemba)",
  "Kociemba Phase 1 (G1縮約)": "Phase 1: G1部分群縮約",
  "Kociemba Phase 2 (群解決)": "Phase 2: 解決手順",
  "Kociemba 直接解決": "直接探索解決",
  センター向き解決: "ステップ 2: センター向き解決",
};

export function analyzeMoves(
  moves: string[],
  phases?: PhaseInfo[],
): MoveMeta[] {
  if (moves.length === 0) return [];

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

  // phases 情報が与えられている場合はそれを使用
  if (phases && phases.length > 0) {
    return moves.map((move, idx) => {
      let matchedPhaseIdx = -1;
      let phaseLabel = "解決手順";

      for (let p = 0; p < phases.length; p++) {
        const ph = phases[p];
        if (idx >= ph.start && idx < ph.end) {
          matchedPhaseIdx = p;
          phaseLabel = PHASE_LABEL_MAP[ph.name] || ph.name;
          break;
        }
      }

      return {
        move,
        phase: matchedPhaseIdx >= 0 ? matchedPhaseIdx + 1 : 0,
        phaseLabel,
        trigger: triggerMap.get(idx),
      };
    });
  }

  // phases がない場合は従来の Kociemba Phase 1 / Phase 2 判定
  // 末尾から見て、連続して G1_MOVES である区間を Phase 2 とする
  let phase2StartIndex = moves.length;
  for (let i = moves.length - 1; i >= 0; i--) {
    if (G1_MOVES.has(moves[i])) {
      phase2StartIndex = i;
    } else {
      break;
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
