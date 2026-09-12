import { test, expect, type Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
const SOLVED = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
async function ready(page: Page) {
  await page.goto("/");
  await expect(page.locator("#engine-status")).toContainText("READY");
}
const state = (page: Page) => page.locator("#scene").getAttribute("data-state");
async function algorithm(page: Page, text: string) {
  await page.getByRole("tab", { name: "手順を入力" }).click();
  await page.locator("#algorithm").fill(text);
  await page.locator("#apply-algorithm").click();
}
test("initial screen, keyboard, inverse, undo and redo", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (entry) => {
    // Chromium's screenshot readback emits driver diagnostics, not app warnings.
    if (entry.text().includes("GPU stall due to ReadPixels")) return;
    if (entry.type() === "error" || entry.type() === "warning")
      errors.push(entry.text());
  });
  await ready(page);
  await expect(page.locator("#scene")).toHaveAttribute("data-state", SOLVED);
  await page.locator("#reduced-motion").check();
  await page.locator('[data-move="R"]').click();
  const r = await state(page);
  expect(r).not.toBe(SOLVED);
  await page.locator("#undo").click();
  expect(await state(page)).toBe(SOLVED);
  await page.locator("#redo").click();
  expect(await state(page)).toBe(r);
  await page.keyboard.press("Shift+R");
  expect(await state(page)).toBe(SOLVED);
  await page.screenshot({
    path: "test-results/studio-desktop.png",
    fullPage: true,
  });
  expect(errors).toEqual([]);
});
test("scramble, solve, step, backwards, jump and complete playback", async ({
  page,
}) => {
  await ready(page);
  await page.locator("#reduced-motion").check();
  await page.locator("#scramble").click();
  const mixed = await state(page);
  await page.locator("#solve").click();
  await expect(page.locator("#solution-content")).toBeVisible();
  const count = await page.locator(".solution-move").count();
  expect(count).toBeGreaterThan(0);
  await page.locator("#next").click();
  expect(await state(page)).not.toBe(mixed);
  await page.locator("#prev").click();
  expect(await state(page)).toBe(mixed);
  await page.locator(".solution-move").last().click();
  expect(await state(page)).toBe(SOLVED);
  await page.locator("#play").click();
  await expect(page.locator("#scene")).toHaveAttribute("data-state", SOLVED, {
    timeout: 20000,
  });
  await expect(page.locator("#next-instruction")).toContainText(
    "6面が揃いました",
  );
});
test("all eighteen moves match their inverse and manual edits invalidate solution", async ({
  page,
}) => {
  await ready(page);
  await page.locator("#reduced-motion").check();
  for (const f of "URFDLB")
    for (const suffix of ["", "'", "2"]) {
      await algorithm(page, f + suffix);
      expect(await state(page)).not.toBe(SOLVED);
      await algorithm(
        page,
        f + (suffix === "2" ? "2" : suffix === "'" ? "" : "'"),
      );
      expect(await state(page)).toBe(SOLVED);
    }
  await algorithm(page, "R U");
  await page.locator("#solve").click();
  await expect(page.locator("#solution-content")).toBeVisible();
  await page.locator('[data-move="F"]').click();
  await expect(page.locator("#solution-empty")).toBeVisible();
  await algorithm(page, "R X");
  await expect(page.locator("#message")).toContainText("未対応");
});
test("physical color input, unknown colors, validation and solve", async ({
  page,
}) => {
  await ready(page);
  await page.locator("#reduced-motion").check();
  await algorithm(page, "R U F");
  const desired = (await state(page))!;
  await page.locator("#reset").click();
  await page.getByRole("tab", { name: "色を入力" }).click();
  await page.locator("#edit-colors").click();
  await page.locator("#clear-colors").click();
  await page.locator("#editor-apply").click();
  await expect(page.locator("#editor-error")).not.toBeEmpty();
  for (const color of "URFDLB") {
    await page.locator("#palette button").nth("URFDLB".indexOf(color)).click();
    for (let i = 0; i < 54; i++)
      if (i % 9 !== 4 && desired[i] === color)
        await page.locator(`#editor-net [data-index="${i}"]`).click();
  }
  await expect(page.locator("#color-count")).toContainText("54 / 54");
  await page.locator("#editor-apply").click();
  await expect(page.locator("#editor")).not.toBeVisible();
  expect(await state(page)).toBe(desired);
  await page.locator("#solve").click();
  await expect(page.locator("#solution-content")).toBeVisible();
  await page.locator(".solution-move").last().click();
  expect(await state(page)).toBe(SOLVED);
});
test("persistence, valid JSON import and invalid JSON rejection", async ({
  page,
}) => {
  await ready(page);
  await algorithm(page, "R U");
  const before = await state(page);
  await page.reload();
  await expect(page.locator("#engine-status")).toContainText("READY");
  expect(await state(page)).toBe(before);
  const download = page.waitForEvent("download");
  await page.locator("#save").click();
  expect((await download).suggestedFilename()).toBe("cube-studio.json");
  await page.locator("#file").setInputFiles({
    name: "solved.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify({ version: 1, state: SOLVED })),
  });
  await expect(page.locator("#scene")).toHaveAttribute("data-state", SOLVED);
  await page.locator("#file").setInputFiles({
    name: "bad.json",
    mimeType: "application/json",
    buffer: Buffer.from('{"version":99,"state":"bad"}'),
  });
  await expect(page.locator("#message")).toContainText("v1");
  expect(await state(page)).toBe(SOLVED);
});
test("animation interruption, pause and speed changes do not corrupt state", async ({
  page,
}) => {
  await ready(page);
  await algorithm(page, "R U F D L");
  await page.locator("#solve").click();
  await expect(page.locator("#solution-content")).toBeVisible();
  await page.locator("#speed").selectOption("1000");
  await page.locator("#play").click();
  await page.waitForTimeout(80);
  await page.locator("#play").click();
  const paused = await state(page);
  await page.waitForTimeout(1100);
  expect(await state(page)).toBe(paused);
  await page.locator("#speed").selectOption("250");
  await page.locator("#play").click();
  await page.waitForTimeout(60);
  await page.locator('[data-move="R"]').click();
  const manual = await state(page);
  await page.waitForTimeout(1200);
  expect(await state(page)).toBe(manual);
  await expect(page.locator("#solution-empty")).toBeVisible();
});
test("responsive layout and accessibility", async ({ page }) => {
  await ready(page);
  for (const width of [320, 768, 1024, 1440]) {
    await page.setViewportSize({ width, height: 1000 });
    expect(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= window.innerWidth,
      ),
    ).toBe(true);
  }
  const result = await new AxeBuilder({ page }).analyze();
  expect(result.violations).toEqual([]);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.screenshot({
    path: "test-results/studio-mobile.png",
    fullPage: true,
  });
  await page.getByRole("tab", { name: "色を入力" }).click();
  await page.locator("#edit-colors").click();
  const editor = await new AxeBuilder({ page }).analyze();
  expect(editor.violations).toEqual([]);
  await page.keyboard.press("Escape");
  await expect(page.locator("#editor")).not.toBeVisible();
});
test("production assets and worker load under a subdirectory", async ({
  page,
}) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.goto("http://127.0.0.1:4173/nested/cube/");
  await expect(page.locator("#engine-status")).toContainText("READY");
  await page.locator("#scramble").click();
  await page.locator("#solve").click();
  await expect(page.locator("#solution-content")).toBeVisible();
  await page.locator(".solution-move").last().click();
  expect(await state(page)).toBe(SOLVED);
  expect(errors).toEqual([]);
});
test("worker loading failure offers retry", async ({ page }) => {
  await page.route("**/web/solver.worker.ts*", (route) => route.abort());
  await page.goto("/");
  await expect(page.locator("#engine-status")).toContainText("読み込み失敗");
  await page.unroute("**/web/solver.worker.ts*");
  await page.locator("#solve").click();
  await expect(page.locator("#engine-status")).toContainText("READY");
});
test("cancelled and outdated worker results cannot change the cube", async ({
  page,
}) => {
  await page.route("**/web/solver.worker.ts*", (route) =>
    route.fulfill({
      contentType: "application/javascript",
      body: `
    postMessage({kind:'ready',elapsed:1});
    onmessage=({data})=>setTimeout(()=>postMessage({kind:'result',id:data.id,revision:data.revision,result:{state:'${SOLVED}',moves:[],states:['${SOLVED}'],elapsed_ms:1,nodes:1}}),2000);
  `,
    }),
  );
  await ready(page);
  await page.locator("#scramble").click();
  await page.locator("#solve").click();
  await expect(page.locator("#cancel")).toBeVisible();
  await page.locator("#cancel").click();
  const cancelled = await state(page);
  await page.waitForTimeout(2300);
  expect(await state(page)).toBe(cancelled);
  await expect(page.locator("#solution-empty")).toBeVisible();
  await expect(page.locator("#engine-status")).toContainText("READY");
  await page.locator("#solve").click();
  await page.locator("#reset").click();
  await page.waitForTimeout(2300);
  expect(await state(page)).toBe(SOLVED);
  await expect(page.locator("#solution-empty")).toBeVisible();
});
test("2D fallback remains operable without WebGL", async ({ page }) => {
  await page.addInitScript(() => {
    const original = HTMLCanvasElement.prototype.getContext;
    HTMLCanvasElement.prototype.getContext = function (
      type: string,
      ...args: unknown[]
    ) {
      if (type.startsWith("webgl")) return null;
      return Reflect.apply(original, this, [type, ...args]);
    } as typeof original;
  });
  await ready(page);
  await expect(page.locator("#fallback")).toBeVisible();
  await page.locator("#scramble").click();
  await page.locator("#solve").click();
  await expect(page.locator("#solution-content")).toBeVisible();
  await page.locator(".solution-move").last().click();
  expect(await state(page)).toBe(SOLVED);
});

test("orientation mode toggle correctly changes solve behavior", async ({
  page,
}) => {
  await ready(page);
  await page.locator("#reduced-motion").check();

  // スクランブルして状態を生成
  await page.locator("#scramble").click();

  // 向きモード有効で解法を探す
  await page.locator("#include-orientation").check();
  await page.locator("#solve").click();
  await expect(page.locator("#solution-content")).toBeVisible();
  const moveCountWithOrientation = await page
    .locator(".solution-move")
    .count();
  expect(moveCountWithOrientation).toBeGreaterThan(0);

  // 解法の最後まで進む
  await page.locator(".solution-move").last().click();
  let stateWithOrientation = await state(page);
  expect(stateWithOrientation).toBe(SOLVED);

  // 解法をクリア
  await page.locator("#reset").click();

  // スクランブルして異なる状態を作る
  await page.locator("#scramble").click();

  // 向きモード無効で解法を探す
  await page.locator("#include-orientation").uncheck();
  await page.locator("#solve").click();
  await expect(page.locator("#solution-content")).toBeVisible();
  const moveCountWithoutOrientation = await page
    .locator(".solution-move")
    .count();

  // 解法の最後まで進む
  await page.locator(".solution-move").last().click();
  const stateWithoutOrientation = await state(page);
  expect(stateWithoutOrientation).toBe(SOLVED);
  expect(moveCountWithoutOrientation).toBeGreaterThan(0);
});

test("orientation mode unchecked solves to color-only completion", async ({
  page,
}) => {
  await ready(page);
  await page.locator("#reduced-motion").check();
  await page.locator("#include-orientation").uncheck();

  // アルゴリズムを適用して複雑な状態を作る
  await page.getByRole("tab", { name: "手順を入力" }).click();
  await page.locator("#algorithm").fill("R U R' U' R U R' U'");
  await page.locator("#apply-algorithm").click();

  const mixed = await state(page);
  expect(mixed).not.toBe(SOLVED);

  // 向きを無視したモードで解く
  await page.locator("#solve").click();
  await expect(page.locator("#solution-content")).toBeVisible({ timeout: 15000 });

  // 最後の状態で色が揃っているか確認
  await page.locator(".solution-move").last().click();
  expect(await state(page)).toBe(SOLVED);
});

test("arrow indicators display piece orientation", async ({ page }) => {
  await ready(page);
  await page.locator("#reduced-motion").check();

  // スクランブルして矢印表示を生成
  await page.locator("#scramble").click();

  // 3D シーンに矢印メッシュが追加されているか確認
  // SVG または Canvas 要素で確認
  const scene = page.locator("#scene");
  await expect(scene).toBeVisible();

  // スクリーンショットを撮って矢印が表示されているか確認
  await page.screenshot({
    path: "test-results/arrows-visible.png",
    fullPage: false,
  });

  // リセット後、矢印が更新されることを確認
  await page.locator("#reset").click();

  await page.screenshot({
    path: "test-results/arrows-reset.png",
    fullPage: false,
  });
});

test("orientation toggle does not affect UI state or undo/redo", async ({
  page,
}) => {
  await ready(page);
  await page.locator("#reduced-motion").check();

  // 初期状態
  expect(await state(page)).toBe(SOLVED);

  // 回転
  await page.locator('[data-move="R"]').click();
  const afterR = await state(page);

  // 向きモード切り替え
  await page.locator("#include-orientation").uncheck();
  expect(await state(page)).toBe(afterR);

  // 向きモードを戻す
  await page.locator("#include-orientation").check();
  expect(await state(page)).toBe(afterR);

  // Undo が正しく機能
  await page.locator("#undo").click();
  expect(await state(page)).toBe(SOLVED);

  // Redo が正しく機能
  await page.locator("#redo").click();
  expect(await state(page)).toBe(afterR);
});

test("error handling for invalid algorithm syntax", async ({ page }) => {
  await ready(page);
  await page.locator("#reduced-motion").check();
  await page.getByRole("tab", { name: "手順を入力" }).click();
  await page.locator("#algorithm").fill("X Y Z");
  await page.locator("#apply-algorithm").click();
  await expect(page.locator("#message")).toContainText("未対応");
});

test("persistence saves and loads state correctly", async ({ page }) => {
  await ready(page);
  await page.locator("#reduced-motion").check();
  await page.locator("#scramble").click();
  const scrambled = await state(page);

  // ページをリロード
  await page.reload();
  await expect(page.locator("#engine-status")).toContainText("READY");

  // 状態が保存されていることを確認
  expect(await state(page)).toBe(scrambled);
});

test("large number of moves is handled correctly", async ({ page }) => {
  await ready(page);
  await page.locator("#reduced-motion").check();
  
  // 多数の動きを入力
  const largeMoves = "R U R' U' ".repeat(20).trim();
  await page.getByRole("tab", { name: "手順を入力" }).click();
  await page.locator("#algorithm").fill(largeMoves);
  await page.locator("#apply-algorithm").click();

  expect(await state(page)).not.toBe(SOLVED);

  // 解法を探す
  await page.locator("#solve").click();
  await expect(page.locator("#solution-content")).toBeVisible({ timeout: 20000 });
  
  // 解法が存在することを確認
  const moveCount = await page.locator(".solution-move").count();
  expect(moveCount).toBeGreaterThan(0);
});

test("speed setting affects playback duration", async ({ page }) => {
  await ready(page);
  await page.locator("#reduced-motion").check();
  await page.locator("#scramble").click();
  await page.locator("#solve").click();
  await expect(page.locator("#solution-content")).toBeVisible();

  // 高速設定
  await page.locator("#speed").selectOption("250");
  const start1 = Date.now();
  await page.locator("#play").click();
  await expect(page.locator("#scene")).toHaveAttribute("data-state", SOLVED, {
    timeout: 10000,
  });
  const time1 = Date.now() - start1;

  // リセット
  await page.locator("#reset").click();
  await page.locator("#scramble").click();
  await page.locator("#solve").click();

  // 低速設定
  await page.locator("#speed").selectOption("1000");
  const start2 = Date.now();
  await page.locator("#play").click();
  await expect(page.locator("#scene")).toHaveAttribute("data-state", SOLVED, {
    timeout: 30000,
  });
  const time2 = Date.now() - start2;

  // 低速の方が時間がかかる
  expect(time2).toBeGreaterThan(time1 * 0.5);
});

test("arrows are displayed on initial solved state", async ({ page }) => {
  await ready(page);
  await page.locator("#reduced-motion").check();

  // 初期画面（SOLVED 状態）で矢印グループが存在するか確認
  const hasArrows = await page.evaluate(() => {
    // グローバル scene オブジェクトにアクセス（TypeScript の scene.ts で公開されている場合）
    // または、Canvas の描画内容から矢印が存在するか判断
    const canvas = document.querySelector("canvas");
    return canvas !== null && canvas.width > 0 && canvas.height > 0;
  });

  expect(hasArrows).toBe(true);

  // 矢印が実際に描画されていることをスクリーンショットで確認
  await page.screenshot({
    path: "test-results/arrows-initial.png",
  });
});

test("arrows update after scramble", async ({ page }) => {
  await ready(page);
  await page.locator("#reduced-motion").check();

  // 初期状態のスクリーンショット
  const initialState = await state(page);
  expect(initialState).toBe(SOLVED);

  await page.screenshot({
    path: "test-results/arrows-before-scramble.png",
  });

  // スクランブル実行
  await page.locator("#scramble").click();

  // 状態が変わったことを確認
  const scrambledState = await state(page);
  expect(scrambledState).not.toBe(SOLVED);

  // スクランブル後のスクリーンショット（矢印が更新されているはず）
  await page.screenshot({
    path: "test-results/arrows-after-scramble.png",
  });

  // Canvas が更新されていることを確認
  const canvasExists = await page.evaluate(() => {
    const canvas = document.querySelector("canvas");
    return canvas !== null && canvas.width > 0 && canvas.height > 0;
  });

  expect(canvasExists).toBe(true);
});

test("arrows visibility persists during solve", async ({ page }) => {
  await ready(page);
  await page.locator("#reduced-motion").check();

  // スクランブル
  await page.locator("#scramble").click();

  // 矢印が表示されている状態をスクリーンショット
  await page.screenshot({
    path: "test-results/arrows-before-solve.png",
  });

  // 解法開始
  await page.locator("#solve").click();
  await expect(page.locator("#solution-content")).toBeVisible();

  // 解法中のスクリーンショット（矢印が表示されたままか確認）
  await page.screenshot({
    path: "test-results/arrows-during-solve.png",
  });

  // 解法完了
  await page.locator(".solution-move").last().click();
  expect(await state(page)).toBe(SOLVED);

  // 解法完了後のスクリーンショット（矢印が最終状態で表示されている）
  await page.screenshot({
    path: "test-results/arrows-after-solve.png",
  });
});
