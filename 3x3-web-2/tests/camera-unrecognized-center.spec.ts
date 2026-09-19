import { test, expect, type Page } from "@playwright/test";
import {
  openCameraEditor,
  uploadTestImage,
  captureViewOnTestImage,
} from "./test-utils";

async function ready(page: Page) {
  await page.goto("/");
  await expect(page.locator("#engine-status")).toContainText("READY");
  await page.locator("#reduced-motion").check();
}

test.describe("R11: Unrecognized center recovery in camera and editor", () => {
  test("unrecognized centers '?' are normalized to face color when applied to editor", async ({
    page,
  }) => {
    await ready(page);
    await openCameraEditor(page);

    // partial 画像 (センターが '?' になる画像) をアップロードして取り込む
    await uploadTestImage(page, "partial", "A");
    await captureViewOnTestImage(page, "A");

    await uploadTestImage(page, "partial", "B");
    await captureViewOnTestImage(page, "B");

    // カメラプレビュー側でもセンターセル（i=4）が '?' ではなく 'U' になっていること
    const previewUCenter = page.locator(
      '#camera-face-card-U .sticker[data-index="4"]',
    );
    await expect(previewUCenter).toHaveAttribute("data-color", "U");

    // カメラ結果をエディタへ適用
    await page.locator("#camera-apply").click();
    await expect(page.locator("#editor")).toBeVisible();

    // U センター（index 4）のステッカーが '?' ではなく 'U' に補正され、編集不可でも詰まないこと
    const uCenterCell = page.locator('#guide-grid .sticker[data-index="4"]');
    const uColor = await uCenterCell.getAttribute("data-color");
    expect(uColor).toBe("U");

    // partial 画像の周辺セル（例: U面の index 0）の色が保持されていること（? や空白ではなく色が付いていること）
    const uCornerCell = page.locator('#guide-grid .sticker[data-index="0"]');
    const cornerColor = await uCornerCell.getAttribute("data-color");
    expect(cornerColor).not.toBeNull();
    expect(cornerColor).not.toBe("?");
  });
});
