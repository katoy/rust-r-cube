import { test, expect } from "@playwright/test";

test.describe("3D Camera View Presets UI", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");
  });

  test("view preset buttons exist and change active view", async ({ page }) => {
    const isoBtn = page.locator("#view-preset-iso");
    const frontBtn = page.locator("#view-preset-front");
    const topBtn = page.locator("#view-preset-top");
    const rightBtn = page.locator("#view-preset-right");

    await expect(isoBtn).toBeVisible();
    await expect(frontBtn).toBeVisible();
    await expect(topBtn).toBeVisible();
    await expect(rightBtn).toBeVisible();

    // デフォルトで iso がアクティブ
    await expect(isoBtn).toHaveClass(/active/);

    // front をクリック
    await frontBtn.click();
    await expect(frontBtn).toHaveClass(/active/);
    await expect(isoBtn).not.toHaveClass(/active/);

    // top をクリック
    await topBtn.click();
    await expect(topBtn).toHaveClass(/active/);
    await expect(frontBtn).not.toHaveClass(/active/);

    // right をクリック
    await rightBtn.click();
    await expect(rightBtn).toHaveClass(/active/);
    await expect(topBtn).not.toHaveClass(/active/);

    // 「視点を戻す」をクリックすると iso が再アクティブ
    await page.locator("#view-reset").click();
    await expect(isoBtn).toHaveClass(/active/);
  });
});
