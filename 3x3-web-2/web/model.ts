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
