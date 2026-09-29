import assert from "node:assert/strict";
import { writeFile } from "node:fs/promises";
import { chromium } from "@playwright/test";

const base = process.env.REVIEW_BASE_URL || "http://127.0.0.1:5173";
const solved = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
const superflip = "UBULURUFURURFRBRDRFUFLFRFDFDFDLDRDBDLULBLFLDLBUBRBLBDB";
const browser = await chromium.launch();
const results = {};

async function fresh(query = "") {
  const context = await browser.newContext();
  const page = await context.newPage();
  page.setDefaultTimeout(15000);
  await page.goto(`${base}/?no-sw${query}`);
  await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();
  await page.locator("#reduced-motion").check();
  return { context, page };
}

async function snapshot(page) {
  return page.evaluate(() => ({
    state: window.cube_store.getState(),
    centers: window.cube_store.getCenterTurns(),
    canUndo: window.cube_store.canUndo(),
    canRedo: window.cube_store.canRedo(),
    hasSolution: !!window.cube_store.getSolution(),
    fsm: window.__cube_main_debug__?.appState.kind,
  }));
}

try {
  {
    console.log("Checking validation API contract");
    const { context, page } = await fresh();
    results.validateContract = await page.evaluate((state) => {
      const returned = window.cube_studio.validate(state);
      return { returned, negatedByCompleteCoverageTest: !returned };
    }, solved);
    assert.equal(results.validateContract.returned, true);
    assert.equal(results.validateContract.negatedByCompleteCoverageTest, false);
    await context.close();
  }
  {
    console.log("Checking Redo during solution preview");
    const { context, page } = await fresh();
    await page.locator('[data-move="R"]').click();
    const original = await snapshot(page);
    await page.locator('[data-move="U"]').click();
    const future = await snapshot(page);
    await page.locator("#undo").click();
    await page.locator("#solve").click();
    await page.locator("#solution-content").waitFor();
    await page.locator("#last").click();
    const preview = await snapshot(page);
    await page.locator("#redo").click();
    const afterRedo = await snapshot(page);
    await page.locator("#undo").click();
    const afterUndo = await snapshot(page);
    assert.equal(afterRedo.state, future.state);
    assert.equal(afterUndo.state, solved);
    assert.equal(afterUndo.canUndo, false);
    assert.notEqual(afterUndo.state, original.state);
    results.redoDuringPreview = {
      original,
      future,
      preview,
      afterRedo,
      afterUndo,
    };
    await context.close();
  }
  {
    console.log("Checking delayed editor import during search");
    const { context, page } = await fresh(`&state=${superflip}`);
    let release;
    const held = new Promise((resolve) => {
      release = resolve;
    });
    let arrived;
    const requested = new Promise((resolve) => {
      arrived = resolve;
    });
    await page.route(/\/web\/editor\.ts(?:\?.*)?$/, async (route) => {
      arrived();
      await held;
      await route.continue();
    });
    await page.locator('[data-tab="colors"]').click();
    await page.locator("#edit-colors").click();
    await Promise.race([
      requested,
      new Promise((_, reject) =>
        setTimeout(
          () => reject(new Error("Editor request not observed")),
          15000,
        ),
      ),
    ]);
    await page.locator("#solver-algorithm").selectOption("korf");
    await page.locator("#solve").click();
    await page.locator("#cancel").waitFor();
    const whileSolving = await snapshot(page);
    release();
    await page.locator("#editor").waitFor();
    const modalOpenedWhileSolving = await snapshot(page);
    await page.locator("#editor-apply").click();
    await page.locator("#editor").waitFor({ state: "hidden" });
    const afterApply = await snapshot(page);
    assert.equal(whileSolving.fsm, "solving");
    assert.equal(modalOpenedWhileSolving.fsm, "solving");
    assert.equal(afterApply.fsm, "idle");
    assert.equal(await page.locator("#cancel").isVisible(), false);
    results.delayedEditorDuringSolve = {
      whileSolving,
      modalOpenedWhileSolving,
      afterApply,
    };
    await context.close();
  }
  {
    const { context, page } = await fresh();
    await page.locator('[data-move="R"]').click();
    const before = await snapshot(page);
    await page.locator("#file").setInputFiles({
      name: "invalid-centers.json",
      mimeType: "application/json",
      buffer: Buffer.from(
        JSON.stringify({
          version: 1,
          state: solved,
          centerTurns: "2,0,0,0,0,0",
        }),
      ),
    });
    await page.waitForFunction(
      (state) => window.cube_store.getState() === state,
      solved,
    );
    const after = await snapshot(page);
    const message = await page.locator("#message").textContent();
    assert.notEqual(after.state, before.state);
    assert.deepEqual(after.centers, [0, 0, 0, 0, 0, 0]);
    assert.equal(message, "");
    results.invalidCenterTypeAccepted = { before, after, message };
    await context.close();
  }
  {
    const { context, page } = await fresh();
    await page.locator('[data-move="R"]').click();
    await page.locator("#solve").click();
    await page.locator("#solution-content").waitFor();
    const before = await snapshot(page);
    await page.locator('[data-tab="moves"]').click();
    await page.locator("#algorithm").fill("not-a-move");
    await page.locator("#apply-algorithm").click();
    const after = await snapshot(page);
    assert.equal(before.fsm, "previewing");
    assert.equal(after.fsm, "idle");
    assert.equal(after.hasSolution, true);
    results.failedMutationChangesFsm = {
      before,
      after,
      message: await page.locator("#message").textContent(),
    };
    await context.close();
  }
  await writeFile(
    new URL("./probe-ui.json", import.meta.url),
    JSON.stringify(results, null, 2) + "\n",
  );
  console.log(JSON.stringify(results, null, 2));
} finally {
  await browser.close();
}
