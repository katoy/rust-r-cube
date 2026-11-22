export interface ParsedCubeFile {
  state: string;
  centerTurns?: unknown;
}

export function validateAndParseCubeJson(
  content: string,
  fileSize: number,
): ParsedCubeFile {
  if (fileSize > 65536 || content.length > 65536) {
    throw new Error("ファイルは64KB以内にしてください。");
  }
  const data: unknown = JSON.parse(content);
  if (!data || typeof data !== "object" || Array.isArray(data)) {
    throw new Error("Cube Studio v1 のJSONファイルを選んでください。");
  }
  const obj = data as Record<string, unknown>;
  if (
    !Object.hasOwn(obj, "version") ||
    obj.version !== 1 ||
    !Object.hasOwn(obj, "state") ||
    typeof obj.state !== "string"
  ) {
    throw new Error("Cube Studio v1 のJSONファイルを選んでください。");
  }

  return {
    state: obj.state,
    centerTurns: Object.hasOwn(obj, "centerTurns")
      ? obj.centerTurns
      : undefined,
  };
}

export function createCubeJsonBlob(snapshot: object): Blob {
  return new Blob([JSON.stringify({ ...snapshot, version: 1 }, null, 2)], {
    type: "application/json",
  });
}
