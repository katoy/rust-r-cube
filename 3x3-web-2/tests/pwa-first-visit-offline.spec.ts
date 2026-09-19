import { test, expect } from "@playwright/test";

test.describe("R10: Complete offline boot after single visit without prior reload", () => {
  test("app boots offline after single initial visit on production preview", async ({
    browser,
  }) => {
    // 完全にクリーンな新規コンテキストを作成
    const context = await browser.newContext();
    const page = await context.newPage();

    try {
      page.on("console", (msg) => console.log("PAGE:", msg.text()));
      page.on("pageerror", (err) => console.log("PAGE ERROR:", err.message));
      page.on("requestfailed", (req) =>
        console.log("REQ FAILED:", req.url(), req.failure()?.errorText),
      );

      // 1. 初回訪問
      await page.goto("http://127.0.0.1:4173/nested/cube/");
      await expect(page.locator("#engine-status")).toContainText("READY");

      // Service Worker がコントローラーになるのを確認
      await page.evaluate(async () => {
        await navigator.serviceWorker.ready;
        if (!navigator.serviceWorker.controller) {
          await new Promise((resolve) => {
            navigator.serviceWorker.addEventListener(
              "controllerchange",
              resolve,
              { once: true },
            );
          });
        }
      });

      const cachedUrls = await page.evaluate(async () => {
        const keys = await caches.keys();
        const urls: string[] = [];
        for (const k of keys) {
          const c = await caches.open(k);
          const reqs = await c.keys();
          urls.push(...reqs.map((r) => r.url));
        }
        return urls;
      });
      console.log("Cached URLs before offline:", cachedUrls);

      // 2. 事前のオンラインリロードを挟まず、直ちにオフライン化
      await context.setOffline(true);

      // 3. オフラインでリロード
      await page.reload();

      // 4. オフラインでも正常に起動し READY になること
      await expect(page.locator("#engine-status")).toContainText("READY", {
        timeout: 10000,
      });
    } finally {
      await context.close();
    }
  });
});
