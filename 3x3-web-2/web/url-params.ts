export interface ParsedUrlParams {
  solver?: string;
  state?: string;
  centers?: number[];
  alg?: string;
  hasInvalidCenters?: boolean;
}

export function parseUrlParams(search: string): ParsedUrlParams {
  const params = new URLSearchParams(search);
  const result: ParsedUrlParams = {};

  const solverParam = params.get("solver") || params.get("algorithm");
  if (
    solverParam &&
    ["kociemba", "cfop", "thistlethwaite", "korf"].includes(solverParam)
  ) {
    result.solver = solverParam;
  }

  const stateParam = params.get("state");
  if (stateParam && stateParam.length === 54) {
    result.state = stateParam;
  }

  const centersParam = params.get("centers");
  if (centersParam) {
    try {
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
    } catch {
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
): string {
  const url = new URL(baseHref);
  url.searchParams.delete("alg");
  url.searchParams.delete("algorithm");
  url.searchParams.set("state", state);
  if (centerTurns && centerTurns.some((t) => t !== 0)) {
    url.searchParams.set("centers", centerTurns.join(","));
  } else {
    url.searchParams.delete("centers");
  }
  if (
    solver &&
    solver !== "kociemba" &&
    ["cfop", "thistlethwaite", "korf"].includes(solver)
  ) {
    url.searchParams.set("solver", solver);
  } else {
    url.searchParams.delete("solver");
  }
  return url.toString();
}
