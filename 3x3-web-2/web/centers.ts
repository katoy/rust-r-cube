import { center_parity } from "../pkg/cube_studio";
import { FACES } from "./model";

const quarterTurn = Math.PI / 2;

export function centerTurns(rotations: number[]): number[] {
  return rotations.map(
    (angle) => ((Math.round(angle / quarterTurn) % 4) + 4) % 4,
  );
}

export function turnsToCenters(turns: number[]): number[] {
  return turns.map((t) => (((t % 4) + 4) % 4) * quarterTurn);
}

export function automaticCenters(state: string): number[] {
  // 面の90°回転はコーナー置換パリティとセンター回転総和パリティの双方を反転させるため、
  // パリティ整合を満たす有効な向きの代表値（U面の回転）を選択
  return [center_parity(state) * quarterTurn, 0, 0, 0, 0, 0];
}

export function centersFromInput(state: string, turns: unknown): number[] {
  if (turns === undefined) return automaticCenters(state);
  if (
    !Array.isArray(turns) ||
    turns.length !== 6 ||
    !turns.every((t) => Number.isInteger(t) && t >= 0 && t <= 3)
  ) {
    throw new Error(
      "センターの向きは6面それぞれ0°・90°・180°・270°で指定してください。",
    );
  }
  if (turns.reduce((sum, t) => sum + t, 0) % 2 !== center_parity(state)) {
    throw new Error(
      "センターの向きと配色が整合しません。実物の向きを確認するか、自動設定してください。",
    );
  }
  return turns.map((t) => t * quarterTurn);
}

export function rotateCenters(rotations: number[], moves: string[]): number[] {
  const turns = centerTurns(rotations);
  for (const move of moves) {
    const face = FACES.indexOf(move[0]);
    const delta = move.endsWith("2") ? 2 : move.endsWith("'") ? 3 : 1;
    turns[face] = (turns[face] + delta) % 4;
  }
  return turns.map((t) => t * quarterTurn);
}
