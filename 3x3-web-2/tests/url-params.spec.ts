import { test, expect } from "@playwright/test";

test.describe("R06: URL ?alg= parameter idempotency", () => {
  test("?alg= parameter produces identical state across page reloads and does not accumulate onto saved state", async ({
    page,
  }) => {
    // 1. 初回訪問: ?alg=R
    await page.goto("/?alg=R");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const firstState = await page.locator("#scene").getAttribute("data-state");
    expect(firstState).not.toBeNull();

    // 2. ページをリロード
    await page.reload();
    await expect(page.locator("#engine-status")).toContainText("READY");

    const secondState = await page.locator("#scene").getAttribute("data-state");

    // リロード後も同じ局面であるべき（累積して R2 になってはいけない）
    expect(secondState).toBe(firstState);
  });
});
