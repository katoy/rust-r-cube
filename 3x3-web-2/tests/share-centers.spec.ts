import { test, expect } from "@playwright/test";

test.describe("R07: Supercube center orientation preserved in share link", () => {
  test("share link includes center turns and restores supercube center state", async ({
    page,
    context,
  }) => {
    await context.grantPermissions(["clipboard-read", "clipboard-write"]);
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    // 「色を入力」タブからエディタを開き、U センターの向きを 180度 (2) に設定
    await page.getByRole("tab", { name: "色を入力" }).click();
    await page.locator("#edit-colors").click();
    await page.getByLabel("U センターの向き").selectOption("2");
    await page.locator("#editor-apply").click();
    await expect(page.locator("#editor")).not.toBeVisible();

    // 共有リンクボタンをクリック
    await page.locator("#share-link").click();

    // クリップボードから共有URLを取得
    const shareUrl = await page.evaluate(async () => {
      return await navigator.clipboard.readText();
    });
    expect(shareUrl).toContain("state=");

    // 新しいページで共有URLを開く
    const newPage = await context.newPage();
    await newPage.goto(shareUrl);
    await expect(newPage.locator("#engine-status")).toContainText("READY");

    // 復元後、エディタを開いて U センターの向きが "2" であることを検証
    await newPage.getByRole("tab", { name: "色を入力" }).click();
    await newPage.locator("#edit-colors").click();
    await expect(newPage.getByLabel("U センターの向き")).toHaveValue("2");
  });
});
