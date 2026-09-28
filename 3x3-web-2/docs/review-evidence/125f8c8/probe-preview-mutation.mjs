import { chromium } from "@playwright/test";
import { writeFileSync } from "node:fs";

const browser = await chromium.launch();
try {
  const page = await browser.newPage();
  await page.goto("http://127.0.0.1:5173/?no-sw");
  await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

  const snapshot = () =>
    page.evaluate(() => ({
      state: window.cube_store.getState(),
      canUndo: window.cube_store.canUndo(),
      history: window.cube_store.history.map((item) => item.state),
    }));

  const initial = await snapshot();
  await page.locator('button[data-move="R"]').click();
  const afterR = await snapshot();
  await page.locator("#solve").click();
  await page.locator("#solution-content").waitFor({ state: "visible" });
  await page.locator("#last").click();
  const afterSeek = await snapshot();
  await page.locator('button[data-move="U"]').click();
  const afterU = await snapshot();
  await page.locator("#undo").click();
  const afterFirstUndo = await snapshot();
  const secondUndoDisabled = await page.locator("#undo").isDisabled();
  if (!secondUndoDisabled) await page.locator("#undo").click();
  const afterSecondUndo = await snapshot();

  const result = {
    initial,
    afterR,
    afterSeek,
    afterU,
    afterFirstUndo,
    afterSecondUndo,
    secondUndoDisabled,
    originalRReachable:
      afterFirstUndo.state === afterR.state ||
      afterSecondUndo.state === afterR.state,
  };
  writeFileSync(new URL("./probe-preview-mutation.json", import.meta.url), JSON.stringify(result, null, 2) + "\n");
  console.log(JSON.stringify(result, null, 2));
} finally {
  await browser.close();
}
