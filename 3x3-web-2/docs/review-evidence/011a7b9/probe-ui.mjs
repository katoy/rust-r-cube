import { chromium } from "@playwright/test";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../../..",
);
const base = "http://127.0.0.1:5173";
const browser = await chromium.launch({ headless: true });
const output = {};

try {
  const page = await browser.newPage();
  await page.goto(`${base}/?no-sw`);
  await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();
  await page.getByRole("tab", { name: "色を入力" }).click();
  await page.locator("#camera-colors").click();
  await page.locator("#camera-editor").waitFor({ state: "visible" });

  await page
    .locator("#camera-file-a")
    .setInputFiles(path.join(root, "test-images/solved-view-a.png"));
  await page.waitForFunction(() => window.__lastCamera?.points.length === 6);
  await page.locator("#camera-capture").click();
  await page
    .locator("#camera-file-b")
    .setInputFiles(path.join(root, "test-images/solved-view-b.png"));
  await page.waitForFunction(
    () =>
      window.__lastCamera?.currentView === "B" &&
      window.__lastCamera?.points.length === 6,
  );
  await page.locator("#camera-capture").click();

  output.cameraBefore = await page.evaluate(() => ({
    faces: { ...window.__lastCamera.faces },
    progress: document.querySelector("#camera-progress")?.textContent,
    applyEnabled: !document.querySelector("#camera-apply")?.disabled,
    imageA: window.__lastCamera.sourceUrlA,
  }));

  await page
    .locator("#camera-file-a")
    .setInputFiles(path.join(root, "test-images/scrambled-1-view-a.png"));
  await page.waitForFunction(
    (oldUrl) => window.__lastCamera?.sourceUrlA !== oldUrl,
    output.cameraBefore.imageA,
  );
  output.cameraAfter = await page.evaluate(
    (oldUrl) => ({
      faces: { ...window.__lastCamera.faces },
      progress: document.querySelector("#camera-progress")?.textContent,
      applyEnabled: !document.querySelector("#camera-apply")?.disabled,
      imageAChanged: window.__lastCamera.sourceUrlA !== oldUrl,
      statusA: document.querySelector("#camera-status-a")?.textContent,
    }),
    output.cameraBefore.imageA,
  );
  await page.close();

  const presetPage = await browser.newPage();
  await presetPage.goto(`${base}/?no-sw`);
  await presetPage
    .locator("#engine-status")
    .filter({ hasText: "READY" })
    .waitFor();
  await presetPage.locator("#tab-presets").click();
  let releaseOld;
  const oldHeld = new Promise((resolve) => {
    releaseOld = resolve;
  });
  let oldRequested;
  const requested = new Promise((resolve) => {
    oldRequested = resolve;
  });
  await presetPage.route("**/cubes/solved.json", async (route) => {
    oldRequested();
    await oldHeld;
    await route.fulfill({
      status: 503,
      contentType: "application/json",
      body: "{}",
    });
  });
  await presetPage
    .locator("#preset-buttons button")
    .filter({ hasText: "完成状態" })
    .click();
  await requested;
  await presetPage
    .locator("#preset-buttons button")
    .filter({ hasText: "簡単（3手）" })
    .click();
  await presetPage
    .locator("#preset-status")
    .filter({ hasText: "簡単（3手） を読み込みました" })
    .waitFor();
  output.presetBeforeOldFailure = await presetPage
    .locator("#preset-status")
    .textContent();
  output.presetStateBeforeOldFailure = await presetPage
    .locator("#scene")
    .getAttribute("data-state");
  releaseOld();
  await presetPage
    .locator("#preset-status")
    .filter({ hasText: "読み込み失敗" })
    .waitFor();
  output.presetAfterOldFailure = await presetPage
    .locator("#preset-status")
    .textContent();
  output.presetStateAfterOldFailure = await presetPage
    .locator("#scene")
    .getAttribute("data-state");
  await presetPage.close();

  const timelinePage = await browser.newPage();
  await timelinePage.goto(`${base}/?no-sw&alg=R+U`);
  await timelinePage
    .locator("#engine-status")
    .filter({ hasText: "READY" })
    .waitFor();
  await timelinePage.locator("#solve").click();
  await timelinePage.locator("#solution-content").waitFor({ state: "visible" });
  await timelinePage.locator("#next").click();
  await timelinePage.waitForFunction(() => window.cube_store?.getStep() === 1);
  output.timeline = await timelinePage.evaluate(() => ({
    step: window.cube_store.getStep(),
    moves: window.cube_store.getSolution().moves,
    ariaValueText: document
      .querySelector("#timeline")
      ?.getAttribute("aria-valuetext"),
    nextSymbol: document.querySelector("#next-symbol")?.textContent,
  }));
  await timelinePage.close();
} finally {
  await browser.close();
}

const result = JSON.stringify(output, null, 2);
fs.writeFileSync(new URL("./probe-ui.json", import.meta.url), `${result}\n`);
console.log(result);
