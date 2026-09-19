import { test, expect } from "@playwright/test";

test.describe("Solution Structure and Phase Guide UI", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");
  });

  test("solution display includes phase information and move tags", async ({
    page,
  }) => {
    // プリセットタブから「簡単（5手）」を読み込む
    await page.locator("#tab-presets").click();
    await page
      .locator("#preset-buttons button", { hasText: "簡単（5手）" })
      .click();
    await expect(page.locator("#preset-status")).toContainText(
      "簡単（5手） を読み込みました",
    );

    // 解法を探索
    await page.locator("#solve").click();
    await expect(page.locator("#solution-content")).toBeVisible({
      timeout: 15000,
    });

    // フェーズバッジ（Phase 1 / Phase 2）またはステップ解説要素が表示されること
    const phaseTags = page.locator(".phase-badge, .move-phase-tag");
    await expect(phaseTags.first()).toBeVisible();

    // 次の手順の解説エリアにフェーズまたはトリガー案内が含まれること
    const nextInstruction = page.locator("#next-instruction");
    await expect(nextInstruction).toBeVisible();
    const instructionText = await nextInstruction.textContent();
    expect(instructionText && instructionText.length > 0).toBe(true);
  });
});
