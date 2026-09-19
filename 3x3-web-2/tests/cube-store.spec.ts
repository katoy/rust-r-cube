import { test, expect } from "@playwright/test";

test.describe("CubeStore Unit Tests", () => {
  test("initial state and properties", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const result = await page.evaluate(async () => {
      const { CubeStore } = await import("/web/cube-store.ts");
      const { SOLVED } = await import("/web/model.ts");
      const store = new CubeStore();

      return {
        state: store.getState(),
        solved: SOLVED,
        centerTurns: store.getCenterTurns(),
        revision: store.getRevision(),
        modifier: store.getModifier(),
        canUndo: store.canUndo(),
        canRedo: store.canRedo(),
        solution: store.getSolution(),
        step: store.getStep(),
      };
    });

    expect(result.state).toBe(result.solved);
    expect(result.centerTurns).toEqual([0, 0, 0, 0, 0, 0]);
    expect(result.revision).toBe(0);
    expect(result.modifier).toBe("");
    expect(result.canUndo).toBe(false);
    expect(result.canRedo).toBe(false);
    expect(result.solution).toBeUndefined();
    expect(result.step).toBe(0);
  });

  test("replace updates state, records history, and increments revision", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const result = await page.evaluate(async () => {
      const { CubeStore } = await import("/web/cube-store.ts");
      const { SOLVED } = await import("/web/model.ts");
      const store = new CubeStore();

      const { apply_moves } = (window as any).cube_studio;
      const rState = JSON.parse(apply_moves(SOLVED, "R")).state;

      store.replace(rState);

      return {
        stateAfterFirst: store.getState(),
        canUndoAfterFirst: store.canUndo(),
        canRedoAfterFirst: store.canRedo(),
        revisionAfterFirst: store.getRevision(),
      };
    });

    expect(result.canUndoAfterFirst).toBe(true);
    expect(result.canRedoAfterFirst).toBe(false);
    expect(result.revisionAfterFirst).toBe(1);
  });

  test("undo and redo restore states and center turns", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const result = await page.evaluate(async () => {
      const { CubeStore } = await import("/web/cube-store.ts");
      const { SOLVED } = await import("/web/model.ts");
      const { apply_moves } = (window as any).cube_studio;
      const rState = JSON.parse(apply_moves(SOLVED, "R")).state;
      const store = new CubeStore();

      store.replace(rState, true, [Math.PI / 2, 0, 0, 0, 0, 0]);
      const undid = store.undo();
      const stateAfterUndo = store.getState();
      const centerTurnsAfterUndo = store.getCenterTurns();
      const canRedoAfterUndo = store.canRedo();

      const redid = store.redo();
      const stateAfterRedo = store.getState();
      const centerTurnsAfterRedo = store.getCenterTurns();

      return {
        undid,
        stateAfterUndo,
        centerTurnsAfterUndo,
        canRedoAfterUndo,
        solved: SOLVED,
        redid,
        stateAfterRedo,
        centerTurnsAfterRedo,
        rState,
      };
    });

    expect(result.undid).toBe(true);
    expect(result.stateAfterUndo).toBe(result.solved);
    expect(result.centerTurnsAfterUndo).toEqual([0, 0, 0, 0, 0, 0]);
    expect(result.canRedoAfterUndo).toBe(true);

    expect(result.redid).toBe(true);
    expect(result.stateAfterRedo).toBe(result.rState);
    expect(result.centerTurnsAfterRedo).toEqual([1, 0, 0, 0, 0, 0]);
  });

  test("modifier toggle and set", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const result = await page.evaluate(async () => {
      const { CubeStore } = await import("/web/cube-store.ts");
      const store = new CubeStore();

      store.toggleModifier("'");
      const mod1 = store.getModifier();
      store.toggleModifier("'");
      const mod2 = store.getModifier();

      store.toggleModifier("2");
      const mod3 = store.getModifier();
      store.toggleModifier("'");
      const mod4 = store.getModifier();

      return { mod1, mod2, mod3, mod4 };
    });

    expect(result.mod1).toBe("'");
    expect(result.mod2).toBe("");
    expect(result.mod3).toBe("2");
    expect(result.mod4).toBe("'");
  });

  test("solution and step tracking", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const result = await page.evaluate(async () => {
      const { CubeStore } = await import("/web/cube-store.ts");
      const { SOLVED } = await import("/web/model.ts");
      const store = new CubeStore();

      const dummySolution = {
        state: SOLVED,
        moves: ["R", "U", "R'", "U'"],
        states: [SOLVED, SOLVED, SOLVED, SOLVED, SOLVED],
        elapsed_ms: 10,
        nodes: 50,
      };

      store.setSolution(dummySolution);
      const sol = store.getSolution();
      store.setStep(2);
      const step = store.getStep();

      return {
        hasSolution: !!sol,
        movesLen: sol?.moves.length,
        step,
      };
    });

    expect(result.hasSolution).toBe(true);
    expect(result.movesLen).toBe(4);
    expect(result.step).toBe(2);
  });

  test("subscribe triggers listener on store changes", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const result = await page.evaluate(async () => {
      const { CubeStore } = await import("/web/cube-store.ts");
      const store = new CubeStore();

      let callCount = 0;
      const unsubscribe = store.subscribe(() => {
        callCount++;
      });

      const { SOLVED } = await import("/web/model.ts");
      const { apply_moves } = (window as any).cube_studio;
      const rState = JSON.parse(apply_moves(SOLVED, "R")).state;
      store.replace(rState);
      store.toggleModifier("'");
      store.undo();
      unsubscribe();
      store.redo();

      return { callCount };
    });

    // replace (1) + toggleModifier (2) + undo (3) = 3 (unsubscribe後はカウントされない)
    expect(result.callCount).toBe(3);
  });

  test("subscribe passes event type for distinct store mutations", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const result = await page.evaluate(async () => {
      const { CubeStore } = await import("/web/cube-store.ts");
      const { SOLVED } = await import("/web/model.ts");
      const { apply_moves } = (window as any).cube_studio;
      const store = new CubeStore();

      const events: string[] = [];
      store.subscribe((_s, event) => {
        events.push(event?.type);
      });

      const rState = JSON.parse(apply_moves(SOLVED, "R")).state;
      store.replace(rState);
      store.toggleModifier("'");
      store.undo();
      store.redo();
      store.setSolution({
        state: SOLVED,
        moves: ["R"],
        states: [SOLVED, rState],
        elapsed_ms: 5,
        nodes: 10,
      });
      store.setStep(1);
      store.updateAfterSeek(SOLVED, [0, 0, 0, 0, 0, 0], 0);
      store.applyAlgorithmResult(rState, [0, 0, 0, 0, 0, 0]);

      return { events };
    });

    expect(result.events).toEqual([
      "replace",
      "modifier",
      "undo",
      "redo",
      "solution",
      "step",
      "seek",
      "algorithm",
    ]);
  });
});
