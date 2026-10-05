import { chromium } from "@playwright/test";

// Run `npm run dev -- --port 5173 --strictPort` before this probe.

const SOLVED = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
const browser = await chromium.launch({ headless: true });

try {
  const page = await browser.newPage();
  await page.goto("http://127.0.0.1:5173/?no-sw");
  await page.waitForFunction(() =>
    document.querySelector("#engine-status")?.textContent?.includes("READY"),
  );
  await page.locator("#reduced-motion").check();
  await page.locator('[data-move="R"]').click();
  const scrambled = await page.locator("#scene").getAttribute("data-state");
  await page.locator("#solve").click();
  await page.waitForFunction(() => Boolean(window.cube_store?.getSolution()));
  await page.locator("#last").click();
  const beforeClose = await page.evaluate(() => ({
    state: window.cube_store.getState(),
    canUndo: window.cube_store.canUndo(),
    undoDisabled: document.querySelector("#undo").disabled,
  }));
  await page.locator("#solution-close").click();
  const afterClose = await page.evaluate(() => ({
    state: window.cube_store.getState(),
    canUndo: window.cube_store.canUndo(),
    undoDisabled: document.querySelector("#undo").disabled,
  }));
  console.log(JSON.stringify({
    scrambledIsSolved: scrambled === SOLVED,
    beforeClose,
    afterClose,
  }, null, 2));
} finally {
  await browser.close();
}
