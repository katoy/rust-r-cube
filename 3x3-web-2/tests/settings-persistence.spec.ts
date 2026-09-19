import { test, expect } from "@playwright/test";

test.describe("R09: Settings persistence across multiple reloads", () => {
  test("playback speed and reduced-motion settings survive reloads and are not overwritten during restore", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    // 解法を表示させるためにスクランブル＆解くを実行
    await page.locator("#scramble").click();
    await page.locator("#solve").click();
    await expect(page.locator("#solution-content")).toBeVisible({
      timeout: 15000,
    });

    // 設定を変更: speed = 250, reducedMotion = true
    await page.locator("#reduced-motion").check();
    await page.locator("#speed").selectOption("250");

    // 変更後の localStorage の設定を確認
    const savedBefore = await page.evaluate(() => {
      const raw = localStorage.getItem("cube-studio-v1");
      return raw ? JSON.parse(raw) : null;
    });
    expect(savedBefore.reducedMotion).toBe(true);
    expect(savedBefore.speed).toBe("250");

    // 1回目のリロード
    await page.reload();
    await expect(page.locator("#engine-status")).toContainText("READY");

    // 復元直後の localStorage の設定を検証
    // バグがある場合、store.replace の通知リスナーが初期値で localStorage を上書きしてしまう
    const savedAfterFirstReload = await page.evaluate(() => {
      const raw = localStorage.getItem("cube-studio-v1");
      return raw ? JSON.parse(raw) : null;
    });
    expect(savedAfterFirstReload.reducedMotion).toBe(true);
    expect(savedAfterFirstReload.speed).toBe("250");

    // 2回目のリロード（操作なしで連続リロード）
    await page.reload();
    await expect(page.locator("#engine-status")).toContainText("READY");

    // 2回目のリロード後も設定が維持されていること
    await expect(page.locator("#reduced-motion")).toBeChecked();
  });
});
