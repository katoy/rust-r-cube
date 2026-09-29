import { test, expect } from "@playwright/test";
import { assertCoverageInventory, computeLineCoverage } from "./coverage-calc";

test.describe("Coverage inventory gate", () => {
  const expected = ["/web/main.ts", "/web/model.ts"];
  const measured = expected.map((name) => ({
    url: `http://127.0.0.1:5173${name}`,
    total: 10,
  }));

  test("accepts every intended module", () => {
    expect(() => assertCoverageInventory(measured, expected)).not.toThrow();
  });

  test("rejects an empty measurement", () => {
    expect(() => assertCoverageInventory([], expected)).toThrow(
      "Missing coverage modules",
    );
  });

  test("rejects a missing module even when measured modules pass thresholds", () => {
    expect(() => assertCoverageInventory([measured[0]], expected)).toThrow(
      "Missing coverage modules: /web/model.ts",
    );
  });

  test("rejects a module with no measured lines", () => {
    expect(() =>
      assertCoverageInventory(
        [measured[0], { ...measured[1], total: 0 }],
        expected,
      ),
    ).toThrow("Empty coverage measurement: /web/model.ts");
  });

  test("rejects duplicate measurements, including query variants", () => {
    expect(() =>
      assertCoverageInventory(
        [...measured, { ...measured[0], url: `${measured[0].url}?t=1` }],
        expected,
      ),
    ).toThrow("Duplicate measured coverage module");
  });

  test("rejects unknown modules and invalid expected inventories", () => {
    expect(() =>
      assertCoverageInventory(
        [...measured, { url: "/web/unknown.ts", total: 1 }],
        expected,
      ),
    ).toThrow("Unexpected coverage module");
    expect(() => assertCoverageInventory([], [])).toThrow(
      "Coverage inventory must not be empty",
    );
    expect(() =>
      assertCoverageInventory(measured, [...expected, expected[0]]),
    ).toThrow("Duplicate expected coverage module");
  });
});

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
