export interface ParsedCubeFile {
  state: string;
  centerTurns?: unknown;
}

export function validateAndParseCubeJson(
  content: string,
  fileSize: number,
): ParsedCubeFile {
  if (fileSize > 65536) {
    throw new Error("ファイルは64KB以内にしてください。");
  }
  const data: unknown = JSON.parse(content);
  if (
    !data ||
    typeof data !== "object" ||
    !("version" in data) ||
    data.version !== 1 ||
    !("state" in data) ||
    typeof data.state !== "string"
  ) {
    throw new Error("Cube Studio v1 のJSONファイルを選んでください。");
  }

  return {
    state: data.state,
    centerTurns: "centerTurns" in data ? data.centerTurns : undefined,
  };
}

export function createCubeJsonBlob(snapshot: object): Blob {
  return new Blob([JSON.stringify({ version: 1, ...snapshot }, null, 2)], {
    type: "application/json",
  });
}
