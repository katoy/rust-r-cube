import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test.describe("Comprehensive Accessibility (a11y) Audits", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");
  });

  test("initial main page meets WCAG accessibility standards", async ({
    page,
  }) => {
    const results = await new AxeBuilder({ page }).analyze();
    expect(results.violations).toEqual([]);
  });

  test("solution content page meets accessibility standards", async ({
    page,
  }) => {
    // プリセットを解いて解法を表示
    await page.locator("#tab-presets").click();
    await page
      .locator("#preset-buttons button", { hasText: "簡単（5手）" })
      .click();
    await page.locator("#solve").click();
    await expect(page.locator("#solution-content")).toBeVisible();

    const results = await new AxeBuilder({ page }).analyze();
    expect(results.violations).toEqual([]);
  });

  test("help dialog meets accessibility standards", async ({ page }) => {
    await page.locator("#help").click();
    await expect(page.locator("#help-dialog")).toBeVisible();

    const results = await new AxeBuilder({ page })
      .include("#help-dialog")
      .analyze();
    expect(results.violations).toEqual([]);
  });

  test("camera editor dialog meets accessibility standards", async ({
    page,
  }) => {
    await page.locator('button[data-tab="colors"]').click();
    await page.locator("#camera-colors").click();
    await expect(page.locator("#camera-editor")).toBeVisible();

    const results = await new AxeBuilder({ page })
      .include("#camera-editor")
      .analyze();
    expect(results.violations).toEqual([]);
  });
});
