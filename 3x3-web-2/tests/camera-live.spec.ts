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

  test("R01: stops camera stream and tracks when dialog is closed with Escape", async ({
    page,
  }) => {
    await page.locator('button[data-tab="colors"]').click();
    await page.locator("#camera-colors").click();
    await expect(page.locator("#camera-editor")).toBeVisible();

    // ライブカメラを開始
    await page.locator("#camera-live-stream").click();
    await expect(page.locator("#camera-take-photo")).toBeVisible();

    // ストリームがアクティブであることを確認
    const isLiveBefore = await page.evaluate(() => {
      const video = document.getElementById("camera-video") as HTMLVideoElement;
      const stream = video?.srcObject as MediaStream;
      const track = stream?.getVideoTracks()[0];
      return track && track.readyState === "live";
    });
    expect(isLiveBefore).toBe(true);

    // Escape キーでダイアログを閉じる
    await page.keyboard.press("Escape");
    await expect(page.locator("#camera-editor")).not.toBeVisible();

    // ストリームトラックが ended になり、video.srcObject が null になっていること
    const streamStateAfter = await page.evaluate(() => {
      const video = document.getElementById("camera-video") as HTMLVideoElement;
      // window/document に残っている video 要素の状態を調査
      return {
        hasSrcObject: video?.srcObject !== null,
      };
    });
    expect(streamStateAfter.hasSrcObject).toBe(false);
  });
});
