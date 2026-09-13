import { center_parity } from "../pkg/cube_studio";
import { FACES } from "./model";

const quarterTurn = Math.PI / 2;

export function centerTurns(rotations: number[]): number[] {
  return rotations.map(
    (angle) => ((Math.round(angle / quarterTurn) % 4) + 4) % 4,
  );
}

export function automaticCenters(state: string): number[] {
  // Any legal quarter turn changes both corner permutation parity and the
  // parity of the center-turn sum. This picks one compatible orientation.
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
