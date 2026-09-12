import { test, expect } from "@playwright/test";

/**
 * 包括的 E2E カバレッジテスト
 * すべての WASM 関数と主要なユースケースを網羅
 */
test.describe("Comprehensive E2E Coverage", () => {
  test("完成状態の検証と操作", async ({ page }) => {
    await page.goto("http://127.0.0.1:5173/");
    await expect(page.locator("#engine-status")).toContainText("READY", {
      timeout: 10000,
    });

    // 完成状態の検証
    const result = await page.evaluate(() => {
      const wasm = (window as any).cube_studio;
      if (!wasm) throw new Error("WASM not loaded");

      const solved = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";

      // validate() - 完成状態
      try {
        const isValid = wasm.validate(solved);
        console.log("✓ validate(solved):", isValid);
        return { validated: isValid };
      } catch (e) {
        return { error: String(e) };
      }
    });

    expect(result.validated).toBe(true);
  });

  test("スクランブルと状態操作", async ({ page }) => {
    await page.goto("http://127.0.0.1:5173/");
    await expect(page.locator("#engine-status")).toContainText("READY", {
      timeout: 10000,
    });

    const results = await page.evaluate(() => {
      const wasm = (window as any).cube_studio;
      if (!wasm) throw new Error("WASM not loaded");

      const solved = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
      const tests: Record<string, any> = {};

      try {
        // scramble() - 異なるシード値でテスト
        for (const seed of [1, 100, 1000]) {
          const scramble = wasm.scramble(seed);
          tests[`scramble(${seed})`] = {
            type: typeof scramble,
            length: scramble.length,
            hasValidMoves: scramble
              .split(" ")
              .every((m: string) => /^[URFDLB][2']?$/.test(m)),
          };
        }

        // apply_moves() - スクランブル適用
        const scramble1 = wasm.scramble(1);
        const applied = JSON.parse(wasm.apply_moves(solved, scramble1));
        tests["apply_moves(solved, scramble)"] = {
          hasState: !!applied.state,
          hasMovesArray: Array.isArray(applied.moves),
          hasStatesArray: Array.isArray(applied.states),
          statesLength: applied.states.length,
        };

        // 複数の操作を連鎖実行
        const result1 = JSON.parse(wasm.apply_moves(solved, "R"));
        const result2 = JSON.parse(wasm.apply_moves(result1.state, "U"));
        const result3 = JSON.parse(wasm.apply_moves(result2.state, "F"));
        tests["chained_operations"] = {
          step1Valid: result1.state !== solved,
          step2Valid: result2.state !== result1.state,
          step3Valid: result3.state !== result2.state,
        };

        return tests;
      } catch (e) {
        return { error: String(e) };
      }
    });

    // すべてのテストが成功したことを確認
    expect(results.error).toBeUndefined();
    expect(results["scramble(1)"].hasValidMoves).toBe(true);
    expect(results["scramble(100)"].hasValidMoves).toBe(true);
    expect(results["scramble(1000)"].hasValidMoves).toBe(true);
    expect(results["apply_moves(solved, scramble)"]).toMatchObject({
      hasState: true,
      hasMovesArray: true,
      hasStatesArray: true,
    });
    expect(results["chained_operations"]).toMatchObject({
      step1Valid: true,
      step2Valid: true,
      step3Valid: true,
    });
  });

  test("解法と向き情報の取得", async ({ page }) => {
    await page.goto("http://127.0.0.1:5173/");
    await expect(page.locator("#engine-status")).toContainText("READY", {
      timeout: 10000,
    });

    const results = await page.evaluate(async () => {
      const wasm = (window as any).cube_studio;
      if (!wasm) throw new Error("WASM not loaded");

      const solved = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
      const tests: Record<string, any> = {};

      try {
        // 簡単なスクランブル
        const simpleScramble = "R U F";
        const simpleState = JSON.parse(
          wasm.apply_moves(solved, simpleScramble),
        ).state;

        // solve() - 通常の解法
        const solutionNormal = JSON.parse(wasm.solve(simpleState, 5000));
        tests["solve()"] = {
          isSolved: solutionNormal.state === solved,
          hasMoves: Array.isArray(solutionNormal.moves),
          hasNodes: typeof solutionNormal.nodes === "number",
          hasElapsed: typeof solutionNormal.elapsed_ms === "number",
        };

        // solve_with_orientation() - 向きを含める場合
        const withOrient = JSON.parse(
          wasm.solve_with_orientation(simpleState, 5000, true),
        );
        tests["solve_with_orientation(true)"] = {
          isSolved: withOrient.state === solved,
          movesLength: withOrient.moves.length,
        };

        // solve_with_orientation() - 向きを含めない場合
        const withoutOrient = JSON.parse(
          wasm.solve_with_orientation(simpleState, 5000, false),
        );
        tests["solve_with_orientation(false)"] = {
          isSolved: withoutOrient.state === solved,
          movesLength: withoutOrient.moves.length,
        };

        // get_orientations() - 向き情報取得
        const orientations = JSON.parse(wasm.get_orientations(simpleState));
        tests["get_orientations()"] = {
          hasCorners: Array.isArray(orientations.corners),
          hasEdges: Array.isArray(orientations.edges),
          cornersCount: orientations.corners.length,
          edgesCount: orientations.edges.length,
        };

        // 複数のシード値で解法テスト
        const solveResults = [];
        for (const seed of [1, 10, 50]) {
          const scramble = wasm.scramble(seed);
          const state = JSON.parse(wasm.apply_moves(solved, scramble)).state;
          const solution = JSON.parse(wasm.solve(state, 3000));
          solveResults.push({
            seed,
            isSolved: solution.state === solved,
            movesLength: solution.moves.length,
          });
        }
        tests["multiple_seeds_solve"] = solveResults;

        return tests;
      } catch (e) {
        return { error: String(e) };
      }
    });

    expect(results.error).toBeUndefined();
    expect(results["solve()"]).toMatchObject({
      isSolved: true,
      hasMoves: true,
      hasNodes: true,
      hasElapsed: true,
    });
    expect(results["solve_with_orientation(true)"].isSolved).toBe(true);
    expect(results["solve_with_orientation(false)"].isSolved).toBe(true);
    expect(results["get_orientations()"]).toMatchObject({
      hasCorners: true,
      hasEdges: true,
      cornersCount: 8,
      edgesCount: 12,
    });
    expect(results["multiple_seeds_solve"].length).toBe(3);
    expect(results["multiple_seeds_solve"].every((r: any) => r.isSolved)).toBe(
      true,
    );
  });

  test("エラーハンドリングと境界値", async ({ page }) => {
    await page.goto("http://127.0.0.1:5173/");
    await expect(page.locator("#engine-status")).toContainText("READY", {
      timeout: 10000,
    });

    const results = await page.evaluate(() => {
      const wasm = (window as any).cube_studio;
      if (!wasm) throw new Error("WASM not loaded");

      const tests: Record<string, any> = {};

      // 無効な状態の検証
      try {
        wasm.validate("invalid");
        tests["invalid_state"] = { threw: false };
      } catch (e) {
        tests["invalid_state"] = { threw: true, message: String(e) };
      }

      // 短すぎる状態
      try {
        wasm.validate("SHORT");
        tests["short_state"] = { threw: false };
      } catch (e) {
        tests["short_state"] = { threw: true };
      }

      // 無効な手順の適用
      try {
        const solved = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
        wasm.apply_moves(solved, "X Y Z");
        tests["invalid_moves"] = { threw: false };
      } catch (e) {
        tests["invalid_moves"] = { threw: true };
      }

      // 大規模なスクランブル
      const largeScramble = wasm.scramble(999999);
      tests["large_seed"] = {
        hasContent: largeScramble.length > 0,
        isString: typeof largeScramble === "string",
      };

      // すべての面の回転をテスト
      const solved = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
      const faces = ["U", "R", "F", "D", "L", "B"];
      const faceTests = [];
      for (const face of faces) {
        try {
          for (const modifier of ["", "'", "2"]) {
            const move = face + modifier;
            const result = JSON.parse(wasm.apply_moves(solved, move));
            faceTests.push({
              move,
              success: !!result.state,
            });
          }
        } catch (e) {
          faceTests.push({
            face,
            error: String(e),
          });
        }
      }
      tests["all_face_rotations"] = faceTests;

      return tests;
    });

    expect(results.error).toBeUndefined();
    expect(results["invalid_state"].threw).toBe(true);
    expect(results["short_state"].threw).toBe(true);
    expect(results["invalid_moves"].threw).toBe(true);
    expect(results["large_seed"]).toMatchObject({
      hasContent: true,
      isString: true,
    });
    expect(results["all_face_rotations"].length).toBe(18); // 6面 × 3修飾子
    expect(results["all_face_rotations"].every((t: any) => t.success)).toBe(
      true,
    );
  });

  test("パフォーマンスと一貫性の検証", async ({ page }) => {
    await page.goto("http://127.0.0.1:5173/");
    await expect(page.locator("#engine-status")).toContainText("READY", {
      timeout: 10000,
    });

    const results = await page.evaluate(() => {
      const wasm = (window as any).cube_studio;
      if (!wasm) throw new Error("WASM not loaded");

      const solved = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
      const tests: Record<string, any> = {};

      // 同じシード値で同じスクランブルが生成されるか
      const scramble1 = wasm.scramble(42);
      const scramble2 = wasm.scramble(42);
      tests["deterministic_scramble"] = {
        match: scramble1 === scramble2,
        value: scramble1,
      };

      // 逆操作で元の状態に戻るか
      const scramble = "R U F";
      const scrambled = JSON.parse(wasm.apply_moves(solved, scramble)).state;
      const reversed = JSON.parse(
        wasm.apply_moves(scrambled, "F' U' R'"),
      ).state;
      tests["reversible_operations"] = {
        returnToSolved: reversed === solved,
      };

      // 複数回の同じ操作
      const state1 = JSON.parse(wasm.apply_moves(solved, "R")).state;
      const state2 = JSON.parse(wasm.apply_moves(state1, "R")).state;
      const state3 = JSON.parse(wasm.apply_moves(state2, "R")).state;
      const state4 = JSON.parse(wasm.apply_moves(state3, "R")).state;
      tests["four_rotations"] = {
        returnToSolved: state4 === solved, // R^4 = identity
      };

      // 回転操作のコンポーズ
      const viaCompose = JSON.parse(wasm.apply_moves(solved, "R U R U")).state;
      const viaSeparate1 = JSON.parse(wasm.apply_moves(solved, "R U")).state;
      const viaSeparate2 = JSON.parse(
        wasm.apply_moves(viaSeparate1, "R U"),
      ).state;
      tests["operation_compose"] = {
        consistent: viaCompose === viaSeparate2,
      };

      return tests;
    });

    expect(results.error).toBeUndefined();
    expect(results["deterministic_scramble"].match).toBe(true);
    expect(results["reversible_operations"].returnToSolved).toBe(true);
    expect(results["four_rotations"].returnToSolved).toBe(true);
    expect(results["operation_compose"].consistent).toBe(true);
  });
});
