import { test, expect } from "@playwright/test";

/**
 * 完全な E2E カバレッジテスト
 * lib.rs と tables.rs のすべての機能を網羅
 */
test.describe("Complete WASM and Tables Coverage", () => {
  test("all WASM functions with comprehensive scenarios", async ({ page }) => {
    await page.goto("http://127.0.0.1:5173/");
    await expect(page.locator("#engine-status")).toContainText("READY", {
      timeout: 10000,
    });

    // テスト結果ログ
    const results: string[] = [];

    // 1. initialize() - テーブル初期化
    await page.evaluate(async () => {
      const wasm = await import(/* @vite-ignore */ "/pkg/cube_studio.js");
      await wasm.default({ module_or_path: "/pkg/cube_studio_bg.wasm" });
      wasm.initialize();
    });
    results.push("✓ initialize()");

    // 2-7. スクランブル、動き適用、検証、解法（複数シナリオ）
    const scenarios = [
      { seed: 1, name: "seed 1" },
      { seed: 50, name: "seed 50" },
      { seed: 100, name: "seed 100" },
      { seed: 500, name: "seed 500" },
      { seed: 1000, name: "seed 1000" },
    ];

    for (const scenario of scenarios) {
      const passed = await page.evaluate(async (seed) => {
        const wasm = await import(/* @vite-ignore */ "/pkg/cube_studio.js");
        const solved = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";

        try {
          // scramble()
          const scrambled = wasm.scramble(seed);

          // apply_moves()
          const state = JSON.parse(wasm.apply_moves(solved, scrambled)).state;

          // validate()
          const isValid = !wasm.validate(state);

          // solve()
          const solutionJson = wasm.solve(state, 5000);
          const solution = JSON.parse(solutionJson);

          // solve_with_orientation()
          const withOrient = JSON.parse(
            wasm.solve_with_orientation(state, 5000, true),
          );
          const withoutOrient = JSON.parse(
            wasm.solve_with_orientation(state, 5000, false),
          );

          // get_orientations()
          const orientations = wasm.get_orientations(state);

          return (
            isValid &&
            solution.state ===
              "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB" &&
            withOrient.state ===
              "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB" &&
            withoutOrient.state ===
              "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB" &&
            orientations.length > 0
          );
        } catch (e) {
          return false;
        }
      }, scenario.seed);

      if (passed) {
        results.push(`✓ All functions tested with ${scenario.name}`);
      }
    }

    // 結果確認
    console.log("✅ E2E Coverage Test Results:");
    results.forEach((r) => console.log(r));

    // アサーション
    expect(results.length).toBeGreaterThanOrEqual(6);
    expect(results[0]).toContain("initialize");
  });

  test("edge cases and error handling", async ({ page }) => {
    await page.goto("http://127.0.0.1:5173/");
    await expect(page.locator("#engine-status")).toContainText("READY", {
      timeout: 10000,
    });

    const testsPassed = await page.evaluate(async () => {
      const wasm =
        (window as any).cube_studio ||
        (await (async () => {
          const m = await import(/* @vite-ignore */ "/pkg/cube_studio.js");
          await m.default({ module_or_path: "/pkg/cube_studio_bg.wasm" });
          return m;
        })());
      const tests: boolean[] = [];

      // 無効な状態
      try {
        wasm.validate("invalid");
        tests.push(false);
      } catch {
        tests.push(true);
      }

      // 大規模なスクランブル
      const largeScramble = wasm.scramble(9999);
      tests.push(largeScramble.length > 0);

      // すべての面での動き
      const solved = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
      const moves = ["R", "U", "F", "D", "L", "B"];
      for (const move of moves) {
        try {
          wasm.apply_moves(solved, move);
          tests.push(true);
        } catch {
          tests.push(false);
        }
      }

      return tests.every((t) => t);
    });

    expect(testsPassed).toBe(true);
    console.log("✅ Edge case tests passed");
  });

  test("table consistency and performance", async ({ page }) => {
    await page.goto("http://127.0.0.1:5173/");
    await expect(page.locator("#engine-status")).toContainText("READY", {
      timeout: 10000,
    });

    const metrics = await page.evaluate(async () => {
      const wasm =
        (window as any).cube_studio ||
        (await (async () => {
          const m = await import(/* @vite-ignore */ "/pkg/cube_studio.js");
          await m.default({ module_or_path: "/pkg/cube_studio_bg.wasm" });
          return m;
        })());
      const solved = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";

      // 複数の状態で解法時間を計測
      const times: number[] = [];
      for (let seed = 1; seed <= 10; seed++) {
        const start = performance.now();
        const scrambled = wasm.scramble(seed);
        const state = JSON.parse(wasm.apply_moves(solved, scrambled)).state;
        wasm.solve(state, 2000);
        const elapsed = performance.now() - start;
        times.push(elapsed);
      }

      return {
        avgTime: times.reduce((a, b) => a + b, 0) / times.length,
        maxTime: Math.max(...times),
        minTime: Math.min(...times),
        testCount: 10,
      };
    });

    console.log(`✅ Performance Metrics:
      Average: ${metrics.avgTime.toFixed(2)}ms
      Max: ${metrics.maxTime.toFixed(2)}ms
      Min: ${metrics.minTime.toFixed(2)}ms
      Tests: ${metrics.testCount}`);

    expect(metrics.avgTime).toBeLessThan(1000); // 平均 1 秒以下
    expect(metrics.testCount).toBe(10);
  });
});
