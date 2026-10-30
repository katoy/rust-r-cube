import { test, expect, type Page } from "@playwright/test";

const SOLVED = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
const GODS_NUMBER = 20;

async function loadSuperflipWithoutOrientation(page: Page) {
  await page.goto("/");
  await expect(page.locator("#engine-status")).toContainText("READY");
  await page.locator("#reduced-motion").check();
  await page.locator("#include-orientation").uncheck();
  await page.locator("#solver-algorithm").selectOption("kociemba");
  await page.locator("#tab-presets").click();
  await page.locator("button", { hasText: "スーパーフリップ" }).click();
  await expect(page.locator("#preset-status")).toContainText(
    "スーパーフリップ を読み込みました",
  );
}

test.describe("センター向きを考慮しない Kociemba の 20 手保証", () => {
  // スーパーフリップは 20 手の解に届くまで既定予算（5 秒）を超えることがあるため長めに待つ
  test.setTimeout(180_000);

  test("予算を超えても強制終了せず、スーパーフリップを 20 手で解く", async ({
    page,
  }) => {
    await loadSuperflipWithoutOrientation(page);

    await page.locator("#solve").click();
    await expect(page.locator("#solution-content")).toBeVisible({
      timeout: 150_000,
    });
    await expect(page.locator("#message")).toHaveText("");

    const moves = await page.locator(".solution-move").allTextContents();
    expect(moves.length).toBe(GODS_NUMBER);

    await page.locator(".solution-move").last().click();
    expect(await page.locator("#scene").getAttribute("data-state")).toBe(
      SOLVED,
    );
  });

  test("延長中の探索も取り消しボタンで中断できる", async ({ page }) => {
    await loadSuperflipWithoutOrientation(page);

    await page.locator("#solve").click();
    await expect(page.locator("#cancel")).toBeVisible();
    await page.locator("#cancel").click();

    await expect(page.locator("#cancel")).toBeHidden();
    await expect(page.locator("#solve")).toBeEnabled({ timeout: 30_000 });
    await expect(page.locator("#solution-content")).toBeHidden();
  });
});
