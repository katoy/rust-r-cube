import type { CubeType } from "./model";

export interface ParsedUrlParams {
  solver?: string;
  state?: string;
  centers?: number[];
  alg?: string;
  cubeType?: CubeType;
  hasInvalidCenters?: boolean;
}

export function parseUrlParams(search: string): ParsedUrlParams {
  const params = new URLSearchParams(search);
  const result: ParsedUrlParams = {};

  const typeParam = params.get("type");
  if (typeParam === "2x2" || typeParam === "3x3") {
    result.cubeType = typeParam;
  }

  const solverParam = params.get("solver") || params.get("algorithm");
  if (
    solverParam &&
    ["kociemba", "cfop", "thistlethwaite", "korf", "optimal"].includes(
      solverParam,
    )
  ) {
    result.solver = solverParam;
  }

  const stateParam = params.get("state");
  if (stateParam && (stateParam.length === 54 || stateParam.length === 24)) {
    result.state = stateParam;
    result.cubeType = stateParam.length === 24 ? "2x2" : "3x3";
  }

  const centersParam = params.get("centers");
  if (centersParam) {
    const parts = centersParam.split(",");
    const parsed = parts.map((v) => Number(v.trim()));
    if (
      parts.length === 6 &&
      parsed.every((n) => Number.isInteger(n) && n >= 0 && n <= 3)
    ) {
      result.centers = parsed;
    } else {
      result.hasInvalidCenters = true;
    }
  }

  const algParam = params.get("alg");
  if (algParam) {
    result.alg = algParam;
  }

  return result;
}

export function buildShareUrl(
  baseHref: string,
  state: string,
  centerTurns?: number[],
  solver?: string,
  cubeType?: CubeType,
): string {
  const url = new URL(baseHref);
  url.searchParams.delete("alg");
  url.searchParams.delete("algorithm");
  url.searchParams.set("state", state);
  if (state.length === 24 || cubeType === "2x2") {
    url.searchParams.set("type", "2x2");
  } else {
    url.searchParams.delete("type");
  }
  if (centerTurns && centerTurns.some((t) => t !== 0)) {
    url.searchParams.set("centers", centerTurns.join(","));
  } else {
    url.searchParams.delete("centers");
  }
  if (
    solver &&
    solver !== "kociemba" &&
    ["cfop", "thistlethwaite", "korf", "optimal"].includes(solver)
  ) {
    url.searchParams.set("solver", solver);
  } else {
    url.searchParams.delete("solver");
  }
  return url.toString();
}
