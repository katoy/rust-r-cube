import { test, expect, type Page } from "@playwright/test";
import { SOLVED, type ResultData } from "../web/model";
import { holdSolverResults, releaseSolverResults } from "./solver-barrier";

async function ready(page: Page) {
  await page.goto("/?no-sw&alg=R");
  await expect(page.locator("#engine-status")).toContainText("READY");
  await page.locator("#reduced-motion").check();
}

async function snapshot(page: Page) {
  return page.evaluate(() => {
    const store = window.cube_store;
    if (!store) throw new Error("Cube store is unavailable");
    return {
      state: store.getState(),
      centers: store.getCenterTurns(),
      canUndo: store.canUndo(),
      hasSolution: !!store.getSolution(),
      fsm: window.__cube_main_debug__?.appState.kind,
    };
  });
}

test("F01: Redo during preview preserves both preview and base in Undo history", async ({
  page,
}) => {
  await ready(page);
  const base = await snapshot(page);
  await page.locator('[data-move="U"]').click();
  const future = await snapshot(page);
  await page.locator("#undo").click();
  await page.locator("#solve").click();
  await expect(page.locator("#solution-content")).toBeVisible();
  await page.locator("#last").click();
  await page.locator("#redo").click();
  const redone = await snapshot(page);
  expect(redone.state).toBe(future.state);
  expect(redone.centers).toEqual(future.centers);
  await page.locator("#undo").click();
  expect((await snapshot(page)).state).toBe(SOLVED);
  await expect(page.locator("#undo")).toBeEnabled();
  await page.locator("#undo").click();
  const restored = await snapshot(page);
  expect(restored.state).toBe(base.state);
  expect(restored.centers).toEqual(base.centers);
  await page.locator("#redo").click();
  expect((await snapshot(page)).state).toBe(SOLVED);
  await page.locator("#redo").click();
  expect((await snapshot(page)).state).toBe(future.state);
});

for (const module of ["editor", "camera"] as const) {
  test(`F02: delayed ${module} import cannot open after search starts`, async ({
    page,
  }) => {
    await ready(page);
    await holdSolverResults(page);
    let release!: () => void;
    const gate = new Promise<void>((resolve) => {
      release = resolve;
    });
    let requested = false;
    await page.route(
      new RegExp(`/web/${module}\\.ts(?:\\?.*)?$`),
      async (route) => {
        requested = true;
        await gate;
        await route.continue();
      },
    );
    try {
      await page.locator('[data-tab="colors"]').click();
      await page
        .locator(module === "editor" ? "#edit-colors" : "#camera-colors")
        .click();
      await expect.poll(() => requested).toBe(true);
      await page.locator("#solve").click();
      await expect(page.locator("#cancel")).toBeVisible();
      release();
      await expect(page.locator("#message")).toContainText("起動を中断");
      await expect(page.locator("#editor")).toBeHidden();
      await expect(page.locator("#camera-editor")).toBeHidden();
      expect((await snapshot(page)).fsm).toBe("solving");
      await releaseSolverResults(page);
      await expect(page.locator("#solution-content")).toBeVisible();
      await page
        .locator(module === "editor" ? "#edit-colors" : "#camera-colors")
        .click();
      await expect(
        page.locator(module === "editor" ? "#editor" : "#camera-editor"),
      ).toBeVisible();
    } finally {
      release();
      await releaseSolverResults(page);
    }
  });
}

test("F01: Redo equal to the current preview does not hide the preserved base behind a no-op Undo", async ({
  page,
}) => {
  await ready(page);
  const base = await snapshot(page);
  for (let run = 0; run < 2; run++) {
    await page.locator("#solve").click();
    await expect(page.locator("#solution-content")).toBeVisible();
    await page.locator("#last").click();
    if (run === 0) await page.locator("#undo").click();
  }
  // M7: 盤面が future の先頭と同じ場合、実質差分がないため Redo は無効
  await expect(page.locator("#redo")).toBeDisabled();
  await expect(page.locator("#undo")).toBeEnabled();
  await page.locator("#undo").click();
  const restored = await snapshot(page);
  expect(restored.state).toBe(base.state);
  expect(restored.centers).toEqual(base.centers);
});

test("F02: applying an already-open editor cannot bypass the solving guard", async ({
  page,
}) => {
  await ready(page);
  await page.locator('[data-tab="colors"]').click();
  await page.locator("#edit-colors").click();
  await expect(page.locator("#editor")).toBeVisible();
  const before = await snapshot(page);
  await page.evaluate(() =>
    window.__cube_main_debug__?.appState.startSolving(),
  );
  await page.locator("#editor-apply").click();
  await expect(page.locator("#editor-error")).toContainText("探索中");
  await expect(page.locator("#editor")).toBeVisible();
  const after = await snapshot(page);
  expect(after.state).toBe(before.state);
  expect(after.centers).toEqual(before.centers);
  expect(after.fsm).toBe("solving");
});

test("F03/F12: malformed center types reject the actual file without leaving preview", async ({
  page,
}) => {
  await ready(page);
  await page.locator("#solve").click();
  await expect(page.locator("#solution-content")).toBeVisible();
  const before = await snapshot(page);
  for (const centerTurns of [null, "2,0,0,0,0,0", { U: 2 }, 2, true]) {
    await page.locator("#file").setInputFiles({
      name: "bad-centers.json",
      mimeType: "application/json",
      buffer: Buffer.from(
        JSON.stringify({ version: 1, state: SOLVED, centerTurns }),
      ),
    });
    await expect(page.locator("#file")).toHaveValue("");
    await expect(page.locator("#message")).toContainText("センター");
    expect(await snapshot(page)).toEqual(before);
  }
  await page.locator("#file").setInputFiles({
    name: "legacy.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify({ version: 1, state: SOLVED })),
  });
  await expect(page.locator("#file")).toHaveValue("");
  expect((await snapshot(page)).state).toBe(SOLVED);
  expect((await snapshot(page)).fsm).toBe("idle");
});

test("F12: empty and invalid algorithms keep preview; valid input commits it", async ({
  page,
}) => {
  await ready(page);
  await page.locator("#solve").click();
  await expect(page.locator("#solution-content")).toBeVisible();
  const before = await snapshot(page);
  await page.locator("#tab-moves").click();
  for (const algorithm of [" ", "not-a-move"]) {
    await page.locator("#algorithm").fill(algorithm);
    await page.locator("#apply-algorithm").click();
    expect(await snapshot(page)).toEqual(before);
    await expect(page.locator("#solution-content")).toBeVisible();
  }
  await page.locator("#algorithm").fill("U");
  await page.locator("#apply-algorithm").click();
  expect((await snapshot(page)).fsm).toBe("idle");
  expect((await snapshot(page)).hasSolution).toBe(false);
});

test("F13: disposing a rendered scene releases the actual shadow target", async ({
  page,
}) => {
  await ready(page);
  await page.waitForFunction(
    () => (window.cube_scene?.["renderer"].info.memory.textures ?? 0) >= 8,
  );
  const result = await page.evaluate(async () => {
    const moduleUrl = "/node_modules/.vite/deps/three.js";
    const three: typeof import("three") = await import(
      /* @vite-ignore */ moduleUrl
    );
    const scene = window.cube_scene;
    if (!scene) throw new Error("Cube scene is unavailable");
    let target: import("three").WebGLRenderTarget | null = null;
    scene["scene"].traverse((object) => {
      if (object instanceof three.DirectionalLight && object.castShadow)
        target = object.shadow.map;
    });
    if (!target) throw new Error("Allocated shadow target is unavailable");
    const shadowTarget: import("three").WebGLRenderTarget = target;
    let disposals = 0;
    shadowTarget.addEventListener("dispose", () => disposals++);
    const before = scene["renderer"].info.memory.textures;
    scene.dispose();
    return {
      disposals,
      before,
      after: scene["renderer"].info.memory.textures,
      children: scene["scene"].children.length,
    };
  });
  expect(result.disposals).toBe(1);
  expect(result.after).toBeLessThan(result.before);
  expect(result.children).toBe(0);
});

test("F09: a valid 72-move fallback replays color and center phases and restores its base", async ({
  page,
}) => {
  await ready(page);
  await page.locator("#tab-presets").click();
  await page
    .locator("#preset-buttons button")
    .filter({ hasText: "スーパーフリップ" })
    .click();
  await expect(page.locator("#preset-status")).toContainText("読み込みました");
  const base = await snapshot(page);
  await page.evaluate(() => {
    const client = window.__cube_main_debug__?.getSolver();
    const wasm = window.cube_studio;
    const store = window.cube_store;
    if (!client || !wasm || !store)
      throw new Error("Cube runtime is unavailable");
    const color = "R L F U D' R2 F2 R F B' U B2 R2 D2 B2 U B2 U' F2 B2 U B2";
    const centers =
      "B L F' L' F' D F2 L' F' L' F L F' D' B' F L F U' F' U F U F2 L' F U F D F2 U' D' F U D F2 U' D' L U D L2 U' D' L U D L2 U' D'";
    const result: ResultData = JSON.parse(
      wasm.apply_moves(store.getState(), `${color} ${centers}`),
    );
    result.algorithm = "kociemba";
    result.phases = [
      { name: "Phase 1", start: 0, end: 10 },
      { name: "Phase 2", start: 10, end: 22 },
      { name: "センター向き解決", start: 22, end: 72 },
    ];
    client.solve = async () => result;
  });
  await page.locator("#solve").click();
  await expect(page.locator(".solution-move")).toHaveCount(72);
  await expect(
    page.locator(".phase-badge").filter({ hasText: "センター" }),
  ).toBeVisible();
  await page.locator(".solution-move").nth(21).click();
  expect((await snapshot(page)).state).toBe(SOLVED);
  expect((await snapshot(page)).centers).toEqual([3, 0, 2, 0, 3, 0]);
  await page.locator("#last").click();
  expect((await snapshot(page)).centers).toEqual([0, 0, 0, 0, 0, 0]);
  await page.locator("#undo").click();
  const restored = await snapshot(page);
  expect(restored.state).toBe(base.state);
  expect(restored.centers).toEqual(base.centers);
});
