import { test, expect } from "@playwright/test";

const SOLVED =
  "U".repeat(9) +
  "R".repeat(9) +
  "F".repeat(9) +
  "D".repeat(9) +
  "L".repeat(9) +
  "B".repeat(9);

test.describe("Playback Controls Enhancement", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");
    await page.locator("#reduced-motion").check();
  });

  test("jump to first step and last step with buttons (#first and #last)", async ({
    page,
  }) => {
    // スクランブルして解く
    await page.locator("#scramble").click();
    const scrambled = await page.locator("#scene").getAttribute("data-state");
    await page.locator("#solve").click();
    await expect(page.locator("#solution-content")).toBeVisible();

    const movesCount = await page.locator(".solution-move").count();
    expect(movesCount).toBeGreaterThan(0);

    // #first と #last ボタンが存在すること
    const firstBtn = page.locator("#first");
    const lastBtn = page.locator("#last");
    await expect(firstBtn).toBeVisible();
    await expect(lastBtn).toBeVisible();

    // 最初（step=0）の状態では #first および #prev が disabled
    await expect(firstBtn).toBeDisabled();
    await expect(page.locator("#prev")).toBeDisabled();
    await expect(page.locator("#step-count")).toHaveText(`0 / ${movesCount}`);

    // #last をクリックすると最終手へジャンプし、解けた状態になる
    await lastBtn.click();
    await expect(page.locator("#step-count")).toHaveText(
      `${movesCount} / ${movesCount}`,
    );
    await expect(page.locator("#scene")).toHaveAttribute("data-state", SOLVED);
    await expect(lastBtn).toBeDisabled();
    await expect(page.locator("#next")).toBeDisabled();
    await expect(firstBtn).toBeEnabled();
    await expect(page.locator("#prev")).toBeEnabled();

    // #first をクリックすると 0 手目（初期スクランブル状態）へジャンプする
    await firstBtn.click();
    await expect(page.locator("#step-count")).toHaveText(`0 / ${movesCount}`);
    await expect(page.locator("#scene")).toHaveAttribute(
      "data-state",
      scrambled!,
    );
    await expect(firstBtn).toBeDisabled();
    await expect(page.locator("#prev")).toBeDisabled();
    await expect(lastBtn).toBeEnabled();
    await expect(page.locator("#next")).toBeEnabled();
  });

  test("keyboard shortcuts Home and End jump to start and finish", async ({
    page,
  }) => {
    await page.locator("#scramble").click();
    const scrambled = await page.locator("#scene").getAttribute("data-state");
    await page.locator("#solve").click();
    await expect(page.locator("#solution-content")).toBeVisible();

    const movesCount = await page.locator(".solution-move").count();

    // 1手進める
    await page.keyboard.press("ArrowRight");
    await expect(page.locator("#step-count")).toHaveText(`1 / ${movesCount}`);

    // End キーで最終手へジャンプ
    await page.keyboard.press("End");
    await expect(page.locator("#step-count")).toHaveText(
      `${movesCount} / ${movesCount}`,
    );
    await expect(page.locator("#scene")).toHaveAttribute("data-state", SOLVED);

    // Home キーで最初へジャンプ
    await page.keyboard.press("Home");
    await expect(page.locator("#step-count")).toHaveText(`0 / ${movesCount}`);
    await expect(page.locator("#scene")).toHaveAttribute(
      "data-state",
      scrambled!,
    );
  });

  test("current step button in move-list automatically stays in view during seek/playback", async ({
    page,
  }) => {
    await page.locator("#scramble").click();
    await page.locator("#solve").click();
    await expect(page.locator("#solution-content")).toBeVisible();

    const movesCount = await page.locator(".solution-move").count();
    expect(movesCount).toBeGreaterThan(10); // スクランブル後の手数は通常10手以上

    // move-list にスクロールが発生する制限（高さ60px）を与え、表示領域外の要素がある状態にする
    const moveList = page.locator("#move-list");
    await moveList.evaluate((el) => {
      el.style.maxHeight = "60px";
      el.style.overflowY = "auto";
    });

    // 最後の手にジャンプすると、末尾の要素が表示されるよう下方にスクロールする
    await page.locator("#last").click();
    await expect(page.locator("#step-count")).toHaveText(
      `${movesCount} / ${movesCount}`,
    );
    await expect
      .poll(async () => moveList.evaluate((el) => el.scrollTop))
      .toBeGreaterThan(0);

    // 最初の手順（0手目）へ戻すと、スクロールが先頭（scrollTop=0）に戻る
    await page.locator("#first").click();
    await expect(page.locator("#step-count")).toHaveText(`0 / ${movesCount}`);
    await expect
      .poll(async () => moveList.evaluate((el) => el.scrollTop))
      .toBe(0);
  });
});
