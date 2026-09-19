import { test, expect, type Page } from "@playwright/test";

async function ready(page: Page) {
  await page.goto("/");
  await expect(page.locator("#engine-status")).toContainText("READY");
  await page.locator("#reduced-motion").check();
}

test.describe("R12: 4-tab keyboard navigation and wrap-around", () => {
  test("ArrowRight and ArrowLeft traverse all 4 tabs with proper wrap-around", async ({
    page,
  }) => {
    await ready(page);

    const tabs = page.locator("[data-tab]");
    await expect(tabs).toHaveCount(4);

    const tabNames = ["scramble", "presets", "colors", "moves"];

    // 最初のタブ（scramble）にフォーカス
    const firstTab = tabs.nth(0);
    await firstTab.focus();
    await expect(firstTab).toBeFocused();
    await expect(firstTab).toHaveAttribute("aria-selected", "true");

    // 右矢印で順次移動: 0 -> 1 -> 2 -> 3 -> 0
    await page.keyboard.press("ArrowRight");
    await expect(tabs.nth(1)).toBeFocused();
    await expect(tabs.nth(1)).toHaveAttribute("aria-selected", "true");
    await expect(page.locator("#presets-panel")).toBeVisible();

    await page.keyboard.press("ArrowRight");
    await expect(tabs.nth(2)).toBeFocused();
    await expect(tabs.nth(2)).toHaveAttribute("aria-selected", "true");
    await expect(page.locator("#colors-panel")).toBeVisible();

    // 4番目のタブ（手順を入力 / moves）へ移動できること
    await page.keyboard.press("ArrowRight");
    await expect(tabs.nth(3)).toBeFocused();
    await expect(tabs.nth(3)).toHaveAttribute("aria-selected", "true");
    await expect(page.locator("#moves-panel")).toBeVisible();

    // 4番目からさらに右矢印で端の折り返し（wrap-around）で0番目に戻ること
    await page.keyboard.press("ArrowRight");
    await expect(tabs.nth(0)).toBeFocused();
    await expect(tabs.nth(0)).toHaveAttribute("aria-selected", "true");
    await expect(page.locator("#scramble-panel")).toBeVisible();

    // 0番目から左矢印で端の折り返しで4番目（moves）に移動できること
    await page.keyboard.press("ArrowLeft");
    await expect(tabs.nth(3)).toBeFocused();
    await expect(tabs.nth(3)).toHaveAttribute("aria-selected", "true");
    await expect(page.locator("#moves-panel")).toBeVisible();

    // 左矢印で逆順移動: 3 -> 2 -> 1 -> 0
    await page.keyboard.press("ArrowLeft");
    await expect(tabs.nth(2)).toBeFocused();
    await expect(tabs.nth(2)).toHaveAttribute("aria-selected", "true");

    await page.keyboard.press("ArrowLeft");
    await expect(tabs.nth(1)).toBeFocused();
    await expect(tabs.nth(1)).toHaveAttribute("aria-selected", "true");

    await page.keyboard.press("ArrowLeft");
    await expect(tabs.nth(0)).toBeFocused();
    await expect(tabs.nth(0)).toHaveAttribute("aria-selected", "true");
  });
});
