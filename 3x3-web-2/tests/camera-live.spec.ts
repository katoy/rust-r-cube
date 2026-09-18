import { test, expect } from "@playwright/test";

test.describe("Camera WebRTC Live Stream UI", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");
  });

  test("camera editor provides live stream camera controls", async ({
    page,
  }) => {
    // 「色を入力」タブからカメラ入力を開く
    await page.locator('button[data-tab="colors"]').click();
    await page.locator("#camera-colors").click();
    await expect(page.locator("#camera-editor")).toBeVisible();

    // ライブカメラ起動ボタンが存在すること
    const liveBtn = page.locator("#camera-live-stream");
    await expect(liveBtn).toBeVisible();
    await expect(liveBtn).toContainText("ライブカメラ");

    // ダイアログを閉じる
    await page.locator("#camera-close").click();
    await expect(page.locator("#camera-editor")).not.toBeVisible();
  });
});
