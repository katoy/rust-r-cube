import { test, expect } from "@playwright/test";

test.describe("Cube State and Algorithm URL Sharing", () => {
  test("loads cube state from url query parameter ?alg=", async ({ page }) => {
    // R U R' U' を URL パラメータとして渡して開く
    await page.goto("/?alg=R+U+R'+U'");
    await expect(page.locator("#engine-status")).toContainText("READY");

    // キューブが完成状態ではなく、スクランブル/適用状態になっていること
    await expect(page.locator("#cube-status")).not.toContainText("完成状態");

    // 解法探索ボタンが有効であること
    await expect(page.locator("#solve")).toBeEnabled();
  });

  test("share button copies valid URL to clipboard and shows feedback", async ({
    page,
    context,
  }) => {
    // クリップボード権限を付与
    await context.grantPermissions(["clipboard-read", "clipboard-write"]);

    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const shareBtn = page.locator("#share-link");
    await expect(shareBtn).toBeVisible();

    await shareBtn.click();

    // メッセージエリアにフィードバックが表示されること
    await expect(page.locator("#message")).toContainText("共有リンク");

    // クリップボードの内容を検証
    const clipboardText = await page.evaluate(() =>
      navigator.clipboard.readText(),
    );
    expect(clipboardText).toContain("?state=");
  });
});
