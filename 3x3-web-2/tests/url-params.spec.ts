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

  test("R15: invalid ?alg= parameter does not reset restored saved state to solved", async ({
    page,
  }) => {
    // 1. 初回訪問で R 操作を実行して状態を保存
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");
    await page.locator('[data-move="R"]').click();
    const rState = await page.locator("#scene").getAttribute("data-state");
    expect(rState).not.toBeNull();
    expect(rState).not.toBe(
      "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB",
    );

    // 2. 不正な alg パラメータでアクセス
    await page.goto("/?alg=INVALID_MOVE");
    await expect(page.locator("#engine-status")).toContainText("READY");

    // 不正な手順で完成状態にリセットされず、保存されていた R の局面が維持されていること
    const stateAfterInvalid = await page
      .locator("#scene")
      .getAttribute("data-state");
    expect(stateAfterInvalid).toBe(rState);

    // 3. 有効な手順を指定した場合は、完成状態からその手順が適用されること
    await page.goto("/?alg=U");
    await expect(page.locator("#engine-status")).toContainText("READY");
    const stateAfterU = await page.locator("#scene").getAttribute("data-state");
    expect(stateAfterU).not.toBe(rState);
    expect(stateAfterU).not.toBe(
      "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB",
    );
  });

  test("R16: ?alg= parameter with underscores (e.g. R_U_R'_U') applies moves to cube state", async ({
    page,
  }) => {
    // 共有リンクやショートハンド等で使われるアンダースコア区切り ?alg=R_U_R'_U'
    await page.goto("/?alg=R_U_R'_U'");
    await expect(page.locator("#engine-status")).toContainText("READY");

    // キューブが完成状態ではなく、アルゴリズム適用状態になっていること
    await expect(page.locator("#cube-status")).not.toContainText("完成状態");

    // 適用された局面が、空白区切り ?alg=R+U+R'+U' と完全に一致すること
    const stateUnderScore = await page
      .locator("#scene")
      .getAttribute("data-state");

    await page.goto("/?alg=R+U+R'+U'");
    await expect(page.locator("#engine-status")).toContainText("READY");
    const stateSpace = await page.locator("#scene").getAttribute("data-state");

    expect(stateUnderScore).toBe(stateSpace);
  });
});
