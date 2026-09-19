import { test, expect } from "@playwright/test";

test.describe("R08: App boots and works even when localStorage throws SecurityError", () => {
  test("app boots and sound toggle functions when localStorage is restricted", async ({
    page,
  }) => {
    // localStorage へのアクセスで SecurityError をスローするよう設定
    await page.addInitScript(() => {
      Object.defineProperty(Storage.prototype, "getItem", {
        value: () => {
          throw new DOMException("Access denied", "SecurityError");
        },
        configurable: true,
      });
      Object.defineProperty(Storage.prototype, "setItem", {
        value: () => {
          throw new DOMException("Access denied", "SecurityError");
        },
        configurable: true,
      });
    });

    await page.goto("/");

    // アプリが正常に起動し、READY になることを検証
    await expect(page.locator("#engine-status")).toContainText("READY", {
      timeout: 10000,
    });
    await expect(page.locator("#scene")).toBeVisible();

    // 音声トグルボタンをクリックしても例外でクラッシュしないこと
    const soundBtn = page.locator("#sound-toggle");
    await expect(soundBtn).toBeVisible();
    await soundBtn.click();
    await expect(soundBtn).toBeVisible();
  });
});
