import { test, expect, type Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

async function openEditor(page: Page) {
  await page.getByRole("tab", { name: "色を入力" }).click();
  await page.locator("#edit-colors").click();
}
async function ready(page: Page) {
  await page.goto("/");
  await expect(page.locator("#engine-status")).toContainText("READY");
  await page.locator("#reduced-motion").check();
}

test("manual centers survive apply, undo, redo and reload, and solve to zero", async ({
  page,
}) => {
  await ready(page);
  await openEditor(page);
  await page.getByLabel("U センターの向き").selectOption("2");
  await page.locator("#editor-apply").click();
  await expect(page.locator("#editor")).not.toBeVisible();
  await expect(page.locator("#cube-status")).toContainText("センター");
  await page.locator("#undo").click();
  await openEditor(page);
  await expect(page.getByLabel("U センターの向き")).toHaveValue("0");
  await page.locator("#editor-close").click();
  await page.locator("#redo").click();
  await page.reload();
  await expect(page.locator("#engine-status")).toContainText("READY");
  await openEditor(page);
  await expect(page.getByLabel("U センターの向き")).toHaveValue("2");
  await page.locator("#editor-close").click();
  await page.locator("#solve").click();
  await expect(page.locator("#solution-content")).toBeVisible();
  await page.locator(".solution-move").last().click();
  await openEditor(page);
  for (const face of "URFDLB") {
    await expect(page.getByLabel(`${face} センターの向き`)).toHaveValue("0");
  }
});

test("incompatible manual centers are rejected and automatic centers are solvable", async ({
  page,
}) => {
  await ready(page);
  await page.locator('[data-move="R"]').click();
  await openEditor(page);
  await page.getByLabel("R センターの向き").selectOption("0");
  await page.locator("#editor-apply").click();
  await expect(page.locator("#editor-error")).toContainText("センター");
  await page.locator("#auto-centers").click();
  await expect(page.getByLabel("U センターの向き")).toHaveValue("1");
  await page.locator("#editor-apply").click();
  await expect(page.locator("#editor")).not.toBeVisible();
  await page.locator("#solve").click();
  await expect(page.locator("#solution-content")).toBeVisible();
  await page.locator(".solution-move").last().click();
  await openEditor(page);
  for (const face of "URFDLB") {
    await expect(page.getByLabel(`${face} センターの向き`)).toHaveValue("0");
  }
});

test("all four angles survive JSON export/import and cancel leaves centers unchanged", async ({
  page,
}) => {
  await ready(page);
  await openEditor(page);
  const turns = [1, 2, 3, 0, 2, 0];
  for (const [i, face] of [..."URFDLB"].entries()) {
    await page
      .getByLabel(`${face} センターの向き`)
      .selectOption(String(turns[i]));
  }
  await page.locator("#editor-apply").click();
  const downloadPromise = page.waitForEvent("download");
  await page.locator("#save").click();
  const download = await downloadPromise;
  const path = await download.path();
  expect(path).not.toBeNull();
  await page.locator("#reset").click();
  await page.locator("#file").setInputFiles(path!);
  await openEditor(page);
  for (const [i, face] of [..."URFDLB"].entries()) {
    await expect(page.getByLabel(`${face} センターの向き`)).toHaveValue(
      String(turns[i]),
    );
  }
  await page.getByLabel("U センターの向き").selectOption("0");
  await page.locator("#editor-close").click();
  await openEditor(page);
  await expect(page.getByLabel("U センターの向き")).toHaveValue("1");
});

test("legacy color-only import gets compatible centers and malformed center input is rejected", async ({
  page,
}) => {
  await ready(page);
  await page.locator('[data-move="R"]').click();
  const state = await page.locator("#scene").getAttribute("data-state");
  await page.locator("#reset").click();
  const load = async (centerTurns?: unknown) =>
    page.locator("#file").setInputFiles({
      name: "cube.json",
      mimeType: "application/json",
      buffer: Buffer.from(JSON.stringify({ version: 1, state, centerTurns })),
    });
  await load();
  await openEditor(page);
  await expect(page.getByLabel("U センターの向き")).toHaveValue("1");
  await expect(page.getByLabel("R センターの向き")).toHaveValue("0");
  await page.locator("#editor-close").click();
  await load([0, 0, 0, 0, 0, 0]);
  await expect(page.locator("#message")).toContainText("整合しません");
  await load([4, 0, 0, 0, 0, 0]);
  await expect(page.locator("#message")).toContainText("0°・90°");
});

test("center input works without WebGL", async ({ page }) => {
  await page.addInitScript(() => {
    const getContext = HTMLCanvasElement.prototype.getContext;
    HTMLCanvasElement.prototype.getContext = function (
      type: string,
      ...args: unknown[]
    ) {
      if (type.includes("webgl")) return null;
      return getContext.call(this, type as "2d", ...(args as [])) as any;
    } as typeof getContext;
  });
  await ready(page);
  await expect(page.locator("#fallback")).toBeVisible();
  await openEditor(page);
  await page.getByLabel("U センターの向き").selectOption("2");
  await page.locator("#editor-apply").click();
  await expect(page.locator('#fallback-net [data-index="4"]')).toHaveText("↓");
  await page.locator("#solve").click();
  await expect(page.locator("#solution-content")).toBeVisible();
  await page.locator(".solution-move").last().click();
  await expect(page.locator('#fallback-net [data-index="4"]')).toHaveText("↑");
});

test("center controls are keyboard accessible and fit narrow screens", async ({
  page,
}) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await ready(page);
  await openEditor(page);
  for (const width of [320, 768, 1024, 1440]) {
    await page.setViewportSize({ width, height: 1080 });
    await page.getByLabel("U センターの向き").focus();
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("Enter");
    await expect(page.getByLabel("U センターの向き")).toBeFocused();
    const fits = await page
      .locator("#editor")
      .evaluate((dialog) => dialog.scrollWidth <= dialog.clientWidth);
    expect(fits).toBe(true);
    if (width === 320 || width === 1440) {
      await page.screenshot({ path: `test-results/center-input-${width}.png` });
    }
  }
  const accessibility = await new AxeBuilder({ page })
    .include("#editor")
    .analyze();
  expect(accessibility.violations).toEqual([]);
  expect(errors).toEqual([]);
});

test("clearing colors preserves manually specified center directions", async ({
  page,
}) => {
  await ready(page);
  await openEditor(page);
  await page.getByLabel("U センターの向き").selectOption("2");
  await page.locator("#clear-colors").click();
  await expect(page.getByLabel("U センターの向き")).toHaveValue("2");
  await expect(page.locator("#center-mode")).toContainText("手動入力");
  await page.locator("#auto-centers").click();
  await expect(page.locator("#editor-error")).not.toBeEmpty();
  await expect(page.getByLabel("U センターの向き")).toHaveValue("2");
});

test("two-view image input opens with camera and file capture controls", async ({
  page,
}) => {
  await ready(page);
  await page.getByRole("tab", { name: "色を入力" }).click();
  await page.locator("#camera-colors").click();
  await expect(page.locator("#camera-editor")).toBeVisible();
  await expect(page.locator("#camera-file-a")).toHaveAttribute(
    "capture",
    "environment",
  );
  await expect(page.locator("#camera-file-b")).toHaveAttribute(
    "capture",
    "environment",
  );
  await expect(page.locator("#camera-capture")).toBeDisabled();
  await page.locator("#camera-close").click();
  await expect(page.locator("#camera-editor")).not.toBeVisible();
});
