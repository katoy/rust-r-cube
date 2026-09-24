export interface ParsedUrlParams {
  solver?: string;
  state?: string;
  centers?: number[];
  alg?: string;
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
      const parsed = centersParam.split(",").map((v) => Number(v));
      if (parsed.length === 6 && parsed.every((n) => !Number.isNaN(n))) {
        result.centers = parsed;
      }
    } catch {
      // 不正な centers は無視
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
): string {
  const url = new URL(baseHref);
  url.searchParams.delete("alg");
  url.searchParams.set("state", state);
  if (centerTurns && centerTurns.some((t) => t !== 0)) {
    url.searchParams.set("centers", centerTurns.join(","));
  } else {
    url.searchParams.delete("centers");
  }
  return url.toString();
}
