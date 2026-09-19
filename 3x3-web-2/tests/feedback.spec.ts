import { test, expect } from "@playwright/test";

test.describe("Interaction Feedback and Sound UI", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");
  });

  test("sound toggle button exists and can be toggled", async ({ page }) => {
    const soundToggle = page.locator("#sound-toggle");
    await expect(soundToggle).toBeVisible();

    // デフォルトでONまたはOFFで、クリックでトグルされること
    const initialAria = await soundToggle.getAttribute("aria-pressed");
    await soundToggle.click();
    const toggledAria = await soundToggle.getAttribute("aria-pressed");
    expect(toggledAria).not.toBe(initialAria);

    // リロード後も localStorage に状態が保持されること
    await page.reload();
    await expect(page.locator("#engine-status")).toContainText("READY");
    const reloadedAria = await page
      .locator("#sound-toggle")
      .getAttribute("aria-pressed");
    expect(reloadedAria).toBe(toggledAria);
  });

  test("keyboard key press highlights corresponding face button", async ({
    page,
  }) => {
    const uBtn = page.locator('button[data-move="U"]');
    await expect(uBtn).toBeVisible();

    // Uキーを押下
    await page.keyboard.down("KeyU");
    await expect(uBtn).toHaveClass(/active-press/);

    // Uキーを離す
    await page.keyboard.up("KeyU");
    await expect(uBtn).not.toHaveClass(/active-press/);
  });
});
