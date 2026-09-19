/**
 * V8 Coverage データから行カバレッジを計算（単一または複数エントリの統合）
 */
export function computeMergedLineCoverage(entries: any[]): {
  covered: number;
  total: number;
  percentage: string;
  uncoveredLines: number[];
} {
  if (!entries || entries.length === 0) {
    return { covered: 0, total: 0, percentage: "0", uncoveredLines: [] };
  }

  const firstWithText = entries.find((e) => e && (e.text || e.source));
  const text = firstWithText?.text || firstWithText?.source || "";
  if (!text) {
    return { covered: 0, total: 0, percentage: "0", uncoveredLines: [] };
  }

  const isExecuted = new Uint8Array(text.length);

  for (const entry of entries) {
    if (!entry) continue;
    const entryExecuted = new Uint8Array(text.length);

    if (entry.functions && Array.isArray(entry.functions)) {
      interface FlatRange {
        start: number;
        end: number;
        count: number;
        length: number;
      }
      const allRanges: FlatRange[] = [];
      for (const fn of entry.functions) {
        if (!fn.ranges) continue;
        for (const r of fn.ranges) {
          const start = r.start ?? r.startOffset ?? 0;
          const end = r.end ?? r.endOffset ?? 0;
          allRanges.push({
            start,
            end,
            count: r.count ?? 0,
            length: Math.max(0, end - start),
          });
        }
      }
      // スパン長が大きい順（外側のスコープから内側のスコープの順）に適用
      allRanges.sort((a, b) => b.length - a.length);
      for (const r of allRanges) {
        const s = Math.max(0, Math.min(r.start, text.length));
        const e = Math.max(0, Math.min(r.end, text.length));
        if (e > s) {
          entryExecuted.fill(r.count > 0 ? 1 : 0, s, e);
        }
      }
    } else if (entry.ranges && Array.isArray(entry.ranges)) {
      // Playwright 形式の ranges: [{ start, end }]
      for (const r of entry.ranges) {
        const start = r.start ?? r.startOffset ?? 0;
        const end = r.end ?? r.endOffset ?? 0;
        const s = Math.max(0, Math.min(start, text.length));
        const e = Math.max(0, Math.min(end, text.length));
        if (e > s) {
          entryExecuted.fill(1, s, e);
        }
      }
    }

    // 複数エントリの実行結果を OR 合成
    for (let pos = 0; pos < text.length; pos++) {
      if (entryExecuted[pos] === 1) {
        isExecuted[pos] = 1;
      }
    }
  }

  const lines = text.split("\n");
  let lineStartOffset = 0;
  let coveredCount = 0;
  let totalLines = 0;
  const uncoveredLines: number[] = [];

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const lineLen = line.length;
    const lineEndOffset = lineStartOffset + lineLen;

    const trimmed = line.trim();
    if (trimmed.length > 0) {
      totalLines++;
      let hasExecutedChar = false;
      for (let pos = lineStartOffset; pos < lineEndOffset; pos++) {
        const ch = text.charCodeAt(pos);
        if (ch > 32 && isExecuted[pos] === 1) {
          hasExecutedChar = true;
          break;
        }
      }
      if (hasExecutedChar) {
        coveredCount++;
      } else {
        uncoveredLines.push(i + 1);
      }
    }

    lineStartOffset = lineEndOffset + 1; // +1 for '\n'
  }

  return {
    covered: coveredCount,
    total: totalLines,
    percentage:
      totalLines > 0 ? ((coveredCount / totalLines) * 100).toFixed(2) : "0",
    uncoveredLines,
  };
}

/**
 * 単一エントリの行カバレッジ計算（後方互換性用）
 */
export function computeLineCoverage(entry: any): {
  covered: number;
  total: number;
  percentage: string;
  uncoveredLines?: number[];
} {
  return computeMergedLineCoverage([entry]);
}
