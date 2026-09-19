import { test, expect } from "@playwright/test";

test.describe("Beginner Guidance and Notation UI", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");
  });

  test("face-buttons have informative title tooltips for beginners", async ({
    page,
  }) => {
    const uBtn = page.locator('button[data-move="U"]');
    await expect(uBtn).toBeVisible();
    await expect(uBtn).toHaveAttribute("title", "上面 (Up)");

    const rBtn = page.locator('button[data-move="R"]');
    await expect(rBtn).toHaveAttribute("title", "右面 (Right)");

    const fBtn = page.locator('button[data-move="F"]');
    await expect(fBtn).toHaveAttribute("title", "前面 (Front)");

    const dBtn = page.locator('button[data-move="D"]');
    await expect(dBtn).toHaveAttribute("title", "下面 (Down)");

    const lBtn = page.locator('button[data-move="L"]');
    await expect(lBtn).toHaveAttribute("title", "左面 (Left)");

    const bBtn = page.locator('button[data-move="B"]');
    await expect(bBtn).toHaveAttribute("title", "背面 (Back)");

    const primeBtn = page.locator("#prime");
    await expect(primeBtn).toHaveAttribute("title", "逆回転 (反時計回り 90°)");

    const doubleBtn = page.locator("#double");
    await expect(doubleBtn).toHaveAttribute("title", "180度回転");
  });

  test("help dialog provides notation table for beginners", async ({
    page,
  }) => {
    await page.locator("#help").click();
    await expect(page.locator("#help-dialog")).toBeVisible();

    // 記号解説テーブルまたはリストが存在すること
    const notationTable = page.locator(".help-notation-table");
    await expect(notationTable).toBeVisible();

    // U, R, F, D, L, B の説明が含まれていること
    await expect(notationTable).toContainText("上面 (Up)");
    await expect(notationTable).toContainText("右面 (Right)");
    await expect(notationTable).toContainText("前面 (Front)");

    await page.locator("#help-close").click();
    await expect(page.locator("#help-dialog")).not.toBeVisible();
  });
});
