import { test, expect } from "@playwright/test";
import { computeLineCoverage } from "./coverage-calc";

test.describe("R13: Coverage calculation with uncalled functions", () => {
  test("does not report 100% coverage when uncalled functions exist", () => {
    const code = [
      "window.reviewUsed = 1;",
      "function reviewNeverCalled() {",
      "  window.reviewNeverRan = 42;",
      "  window.reviewNeverRanAgain = 43;",
      "}",
    ].join("\n");

    // V8 coverage データ: スクリプト全体は count: 1, reviewNeverCalled は count: 0
    const funcStart = code.indexOf("function reviewNeverCalled");
    const funcEnd = code.length;

    const entry = {
      text: code,
      functions: [
        {
          functionName: "",
          isBlockCoverage: true,
          ranges: [{ startOffset: 0, endOffset: code.length, count: 1 }],
        },
        {
          functionName: "reviewNeverCalled",
          isBlockCoverage: true,
          ranges: [{ startOffset: funcStart, endOffset: funcEnd, count: 0 }],
        },
      ],
    };

    const result = computeLineCoverage(entry);
    // コードには5行あり、2行目〜4行目(または3行目〜4行目)は実行されていないため、
    // covered は 5 未満でなければならず、percentage は 100% 未満であるべき。
    expect(parseFloat(result.percentage)).toBeLessThan(100.0);
    expect(result.covered).toBeLessThan(result.total);
  });
});
