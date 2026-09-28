export function getScopedStorageKey(
  baseKey: string,
  customPath?: string,
): string {
  const p =
    customPath !== undefined
      ? customPath
      : typeof window !== "undefined"
        ? window.location.pathname
        : "/";
  const withoutIndex = p.replace(/\/index\.html$/i, "");
  const normalized = withoutIndex.replace(/\/+$/, "") || "/";
  return normalized !== "/" ? `${baseKey}:${normalized}` : baseKey;
}
