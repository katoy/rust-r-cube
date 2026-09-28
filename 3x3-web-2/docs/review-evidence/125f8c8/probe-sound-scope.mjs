import { chromium } from "@playwright/test";
import { writeFileSync } from "node:fs";

const browser = await chromium.launch();
try {
  const page = await browser.newPage();
  const base = "http://127.0.0.1:5173";
  await page.goto(`${base}/?no-sw`);
  await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();
  await page.locator("#sound-toggle").click();
  const root = await page.evaluate(() => ({
    enabled: document.querySelector("#sound-toggle").getAttribute("aria-pressed"),
    soundKeys: Object.keys(localStorage).filter((key) => key.includes("sound")),
  }));

  await page.goto(`${base}/index.html?no-sw`);
  await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();
  const index = await page.evaluate(() => ({
    enabled: document.querySelector("#sound-toggle").getAttribute("aria-pressed"),
    soundKeys: Object.keys(localStorage).filter((key) => key.includes("sound")),
  }));
  const result = { root, index };
  writeFileSync(new URL("./probe-sound-scope.json", import.meta.url), JSON.stringify(result, null, 2) + "\n");
  console.log(JSON.stringify(result, null, 2));
} finally {
  await browser.close();
}
