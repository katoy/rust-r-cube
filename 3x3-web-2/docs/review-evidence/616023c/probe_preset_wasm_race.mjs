import { chromium } from "@playwright/test";

async function main() {
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({ serviceWorkers: "block" });
  const page = await context.newPage();

  // WASM ロードを意図的に 2000ms 遅延させる
  let releaseWasm;
  const wasmBlocker = new Promise((resolve) => {
    releaseWasm = resolve;
  });

  await page.route("**/cube_studio_bg*.wasm*", async (route) => {
    console.log("[probe] WASM request intercepted, delaying...");
    await wasmBlocker;
    await route.continue();
  });

  await page.goto("http://127.0.0.1:5173/?no-sw");

  // プリセットタブに切り替え
  await page.locator('button[data-tab="presets"]').click();

  // WASM 未完了時点でボタンが disabled になっているか確認
  const buttons = page.locator("#preset-buttons button");
  const count = await buttons.count();
  console.log(`[probe] Preset buttons count: ${count}`);

  const firstDisabled = await buttons.first().isDisabled();
  console.log(`[probe] Is preset button disabled before WASM load?: ${firstDisabled}`);

  // ランダム（seed=1）プリセットをクリックしてみる
  const seedButton = buttons.filter({ hasText: "ランダム（seed=1）" });
  await seedButton.click();

  // 少し待ってステータスを確認
  await page.waitForTimeout(500);
  const status = await page.locator("#preset-status").textContent();
  console.log(`[probe] Status after clicking before WASM loaded: "${status}"`);

  // WASM 解除
  releaseWasm();
  await page.waitForTimeout(1000);

  await browser.close();
}

main().catch(console.error);
