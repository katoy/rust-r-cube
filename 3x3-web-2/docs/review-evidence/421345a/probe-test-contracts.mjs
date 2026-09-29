import assert from "node:assert/strict";
import { writeFile } from "node:fs/promises";
import { chromium } from "@playwright/test";

const browser = await chromium.launch();
try {
  const page = await browser.newPage();
  page.setDefaultTimeout(15000);
  await page.goto(
    `${process.env.REVIEW_BASE_URL || "http://127.0.0.1:5198"}/?no-sw&alg=B+L2+D+F2+R2+B2+U+L2+D+R2+F2+U'`,
  );
  await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();
  await page.locator("#solve").click();
  await page.locator("#move-list button").first().waitFor();
  await page.locator("#tab-presets").click();
  await page.locator("#preset-buttons button").first().waitFor();

  await page.evaluate(() => {
    const debug = window.__cube_main_debug__;
    const result = debug.store.getSolution();
    debug.getSolver().solve = () =>
      new Promise((resolve) => {
        window.__reviewFinishSolve = () => resolve(result);
      });
  });
  await page.locator("#solve").click();
  await page.locator("#cancel").waitFor();
  const pending = await page.evaluate(() => ({
    fsm: window.__cube_main_debug__.appState.kind,
    solutionVisible: !document.querySelector("#solution-content").hidden,
    presetDisabled: document.querySelector("#preset-buttons button").disabled,
  }));
  assert.equal(pending.fsm, "solving");
  assert.equal(pending.solutionVisible, false);
  assert.equal(pending.presetDisabled, true);

  const modalAfterDirectHandler = await page.evaluate(() => {
    document.querySelector("#edit-colors").onclick(new MouseEvent("click"));
    return {
      fsm: window.__cube_main_debug__.appState.kind,
      editorOpen: document.querySelector("#editor").open,
    };
  });
  assert.equal(modalAfterDirectHandler.fsm, "solving");
  assert.equal(modalAfterDirectHandler.editorOpen, false);

  let clickCompleted = false;
  const click = page
    .locator("#preset-buttons button")
    .filter({ hasText: "簡単（3手）" })
    .click()
    .then(() => {
      clickCompleted = true;
    });
  await new Promise((resolve) => setTimeout(resolve, 800));
  assert.equal(clickCompleted, false);
  const waitedWhileSolving = { clickCompleted, ...pending };
  await page.evaluate(() => window.__reviewFinishSolve());
  await click;
  await page
    .locator("#preset-status")
    .filter({ hasText: "簡単（3手） を読み込みました" })
    .waitFor();
  const afterCompletion = await page.evaluate(() => ({
    fsm: window.__cube_main_debug__.appState.kind,
    presetStatus: document.querySelector("#preset-status").textContent,
  }));
  const result = {
    method:
      "Real UI with a controlled solver-client completion barrier; not a solver benchmark.",
    pending,
    modalAfterDirectHandler,
    waitedWhileSolving,
    afterCompletion,
    conclusion:
      "Disabled preset clicks wait for search completion, while direct editor handlers are refused during solving.",
  };
  await writeFile(
    new URL("./probe-test-contracts.json", import.meta.url),
    JSON.stringify(result, null, 2) + "\n",
  );
  console.log(JSON.stringify(result, null, 2));
} finally {
  await browser.close();
}
