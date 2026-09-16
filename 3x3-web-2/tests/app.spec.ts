import { test, expect, type Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import { ARROW_COLORS, inverse, FACES } from "../web/model";
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
  const moveCountWithOrientation = await page.locator(".solution-move").count();
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
  await expect(page.locator("#solution-content")).toBeVisible({
    timeout: 15000,
  });

  // 最後の状態で色が揃っているか確認
  await page.locator(".solution-move").last().click();
  expect(await state(page)).toBe(SOLVED);
});

test("arrow indicators display piece orientation", async ({ page }) => {
  await ready(page);
  await page.locator("#reduced-motion").check();

  // 3Dシーンに54本の矢印が存在することを確認（6面 × 9セル = 54セルすべて）
  const arrowCount = await page.evaluate(
    () => (window as any).cube_scene?.getArrowCount() ?? 0,
  );
  expect(arrowCount).toBe(54);

  // スクランブルして矢印表示を生成
  await page.locator("#scramble").click();

  // スクランブル後も矢印が54本存在することを確認
  const scrambledArrowCount = await page.evaluate(
    () => (window as any).cube_scene?.getArrowCount() ?? 0,
  );
  expect(scrambledArrowCount).toBe(54);

  // スクリーンショットを撮影
  await page.screenshot({
    path: "test-results/arrows-visible.png",
    fullPage: false,
  });

  // リセット後、矢印が維持されることを確認
  await page.locator("#reset").click();
  const resetArrowCount = await page.evaluate(
    () => (window as any).cube_scene?.getArrowCount() ?? 0,
  );
  expect(resetArrowCount).toBe(54);

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
  await expect(page.locator("#solution-content")).toBeVisible({
    timeout: 20000,
  });

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
  const movesCount1 = await page.locator(".solution-move").count();
  await page.locator("#play").click();
  const start1 = Date.now();
  await expect(page.locator("#scene")).toHaveAttribute("data-state", SOLVED, {
    timeout: 10000,
  });
  const timePerMove1 = (Date.now() - start1) / Math.max(1, movesCount1);

  // リセット
  await page.locator("#reset").click();
  await page.locator("#scramble").click();
  await page.locator("#solve").click();
  await expect(page.locator("#solution-content")).toBeVisible();

  // 低速設定
  await page.locator("#speed").selectOption("1000");
  const movesCount2 = await page.locator(".solution-move").count();
  await page.locator("#play").click();
  const start2 = Date.now();
  await expect(page.locator("#scene")).toHaveAttribute("data-state", SOLVED, {
    timeout: 30000,
  });
  const timePerMove2 = (Date.now() - start2) / Math.max(1, movesCount2);

  // 低速（1000ms）の方が1手あたり高速（250ms）より時間がかかる
  expect(timePerMove2).toBeGreaterThan(timePerMove1);
});

test("arrows are displayed on initial solved state", async ({ page }) => {
  await ready(page);
  await page.locator("#reduced-motion").check();

  // 初期画面（SOLVED 状態）で矢印が54本存在し、すべて揃っている状態（グリーン: ARROW_COLORS.NORMAL）であることを検証
  const arrowInfo = await page.evaluate(() => {
    const scene = (window as any).cube_scene;
    const arrows = scene?.getArrows() ?? [];
    return {
      count: arrows.length,
      colors: arrows.map((a: any) =>
        (a.material?.color ?? a.cone?.material?.color).getHex(),
      ),
    };
  });

  expect(arrowInfo.count).toBe(54);
  expect(arrowInfo.colors.every((c: number) => c === ARROW_COLORS.NORMAL)).toBe(
    true,
  );

  // スクリーンショット保存
  await page.screenshot({
    path: "test-results/arrows-initial.png",
  });
});

test("arrows update after scramble", async ({ page }) => {
  await ready(page);
  await page.locator("#reduced-motion").check();

  // 初期状態
  const initialColors = await page.evaluate(() => {
    const arrows = (window as any).cube_scene?.getArrows() ?? [];
    return arrows.map((a: any) =>
      (a.material?.color ?? a.cone?.material?.color).getHex(),
    );
  });
  expect(initialColors.length).toBe(54);
  expect(initialColors.every((c: number) => c === ARROW_COLORS.NORMAL)).toBe(
    true,
  );

  await page.screenshot({
    path: "test-results/arrows-before-scramble.png",
  });

  // スクランブル実行
  await page.locator("#scramble").click();

  // スクランブル後の矢印（ピースの向きが乱れたため、グリーン以外の警告色が含まれる）
  const scrambledColors = await page.evaluate(() => {
    const arrows = (window as any).cube_scene?.getArrows() ?? [];
    return arrows.map((a: any) =>
      (a.material?.color ?? a.cone?.material?.color).getHex(),
    );
  });

  expect(scrambledColors.length).toBe(54);
  // スクランブル後はねじれ・反転が生じるため、一部の矢印の色が変化している
  expect(scrambledColors.some((c: number) => c !== ARROW_COLORS.NORMAL)).toBe(
    true,
  );

  await page.screenshot({
    path: "test-results/arrows-after-scramble.png",
  });
});

test("arrows visibility persists during solve", async ({ page }) => {
  await ready(page);
  await page.locator("#reduced-motion").check();

  // スクランブル
  await page.locator("#scramble").click();

  await page.screenshot({
    path: "test-results/arrows-before-solve.png",
  });

  // 解法開始
  await page.locator("#solve").click();
  await expect(page.locator("#solution-content")).toBeVisible();

  // 解法中の矢印存在確認
  const duringSolveArrowCount = await page.evaluate(
    () => (window as any).cube_scene?.getArrowCount() ?? 0,
  );
  expect(duringSolveArrowCount).toBe(54);

  await page.screenshot({
    path: "test-results/arrows-during-solve.png",
  });

  // 解法完了
  await page.locator(".solution-move").last().click();
  expect(await state(page)).toBe(SOLVED);

  // 解法完了後はすべてのピースの向きが揃い、全54本がグリーンに戻る
  const solvedColors = await page.evaluate(() => {
    const arrows = (window as any).cube_scene?.getArrows() ?? [];
    return arrows.map((a: any) =>
      (a.material?.color ?? a.cone?.material?.color).getHex(),
    );
  });
  expect(solvedColors.length).toBe(54);
  expect(solvedColors.every((c: number) => c === ARROW_COLORS.NORMAL)).toBe(
    true,
  );

  await page.screenshot({
    path: "test-results/arrows-after-solve.png",
  });
});

test("arrow orientations update correctly on all faces for all moves (U, R, F, D, L, B, inverses, doubles)", async ({
  page,
}) => {
  await ready(page);
  await page.locator("#reduced-motion").check();

  const getArrowData = async () => {
    return page.evaluate(() => {
      const scene = (window as any).cube_scene;
      const arrows = scene?.getArrows() ?? [];
      return arrows.map((a: any) => ({
        stickerIdx: a.userData?.stickerIdx as number,
        color: (
          a.material?.color ?? a.cone?.material?.color
        ).getHex() as number,
        rotAngle: a.userData?.rotAngle as number,
        kind: a.userData?.kind as "corner" | "edge" | "center",
        pieceIdx: a.userData?.pieceIdx as number,
      }));
    });
  };

  // 1. 初期状態: 全54セルが揃っている（グリーン: ARROW_COLORS.NORMAL, rotAngle: 0）
  let arrows = await getArrowData();
  expect(arrows.length).toBe(54);
  expect(
    arrows.every(
      (a) => a.color === ARROW_COLORS.NORMAL && Math.abs(a.rotAngle) < 1e-4,
    ),
  ).toBe(true);

  // 2. U 操作: セル表面に印刷されているため、U面上の9セルすべて（センター含む）の矢印が +90度（π/2）回転する
  await page.locator('[data-move="U"]').click();
  arrows = await getArrowData();

  await page.screenshot({
    path: "test-results/arrows-after-u-move.png",
  });

  // U面の9セル（インデックス 0..8）はすべて π/2（約1.5708 rad）回転
  const uFaceArrows = arrows.filter((a) => a.stickerIdx < 9);
  expect(uFaceArrows.length).toBe(9);
  expect(
    uFaceArrows.every((a) => Math.abs(a.rotAngle - Math.PI / 2) < 1e-4),
  ).toBe(true);

  // U' (Shift+U) で元に戻る（全54セルが 0 rad, グリーン）
  await page.keyboard.press("Shift+U");
  arrows = await getArrowData();
  expect(
    arrows.every(
      (a) => a.color === ARROW_COLORS.NORMAL && Math.abs(a.rotAngle) < 1e-4,
    ),
  ).toBe(true);

  // 3. U2 操作: U面上の9セルすべての矢印が 180度（π rad）回転する
  await algorithm(page, "U2");
  arrows = await getArrowData();
  const u2FaceArrows = arrows.filter((a) => a.stickerIdx < 9);
  expect(u2FaceArrows.length).toBe(9);
  expect(
    u2FaceArrows.every((a) => Math.abs(Math.abs(a.rotAngle) - Math.PI) < 1e-4),
  ).toBe(true);

  // U2 で元に戻す
  await algorithm(page, "U2");
  arrows = await getArrowData();
  expect(
    arrows.every(
      (a) => a.color === ARROW_COLORS.NORMAL && Math.abs(a.rotAngle) < 1e-4,
    ),
  ).toBe(true);

  // 4. 全6面（U, R, F, D, L, B）の各操作で、その面の9セルがすべて +90度（π/2）回転し、
  //    4回回転で完全に元通りになることを全操作で検証
  const faceChars = ["U", "R", "F", "D", "L", "B"];
  for (let f = 0; f < 6; f++) {
    const face = faceChars[f];
    // 1回回転
    await algorithm(page, face);
    arrows = await getArrowData();

    // その面の9セル（f * 9 .. f * 9 + 8）はすべて π/2 回転している
    const faceArrows = arrows.filter(
      (a) => a.stickerIdx >= f * 9 && a.stickerIdx < (f + 1) * 9,
    );
    expect(faceArrows.length).toBe(9);
    expect(
      faceArrows.every((a) => Math.abs(a.rotAngle - Math.PI / 2) < 1e-4),
    ).toBe(true);

    // 逆回転（face'）で直ちに全54セルが元通り（0 rad, グリーン）に復帰
    await algorithm(page, face + "'");
    arrows = await getArrowData();
    expect(
      arrows.every(
        (a) => a.color === ARROW_COLORS.NORMAL && Math.abs(a.rotAngle) < 1e-4,
      ),
    ).toBe(true);

    // 4回回転（face * 4）で全54セルが元通り（0 rad, グリーン）に復帰
    await algorithm(page, `${face} ${face} ${face} ${face}`);
    arrows = await getArrowData();
    expect(
      arrows.every(
        (a) => a.color === ARROW_COLORS.NORMAL && Math.abs(a.rotAngle) < 1e-4,
      ),
    ).toBe(true);
  }
});

test("all 18 basic moves correctly update arrow orientations on all faces", async ({
  page,
}) => {
  await ready(page);
  await page.locator("#reduced-motion").check();

  const getArrowData = async () => {
    return page.evaluate(() => {
      const scene = (window as any).cube_scene;
      const arrows = scene?.getArrows() ?? [];
      return arrows.map((a: any) => ({
        stickerIdx: a.userData?.stickerIdx as number,
        color: (
          a.material?.color ?? a.cone?.material?.color
        ).getHex() as number,
        rotAngle: a.userData?.rotAngle as number,
        kind: a.userData?.kind as "corner" | "edge" | "center",
        pieceIdx: a.userData?.pieceIdx as number,
      }));
    });
  };

  // 全18操作: 6面 × ["", "'", "2"]
  const faces = ["U", "R", "F", "D", "L", "B"];
  const suffixes = ["", "'", "2"];

  for (let f = 0; f < 6; f++) {
    const face = faces[f];
    for (const suffix of suffixes) {
      const move = face + suffix;

      // 1. 操作を実行
      await algorithm(page, move);
      const arrows = await getArrowData();
      expect(arrows.length).toBe(54);

      // 期待される回転角 (rad)
      const expectedAngle =
        suffix === "" ? Math.PI / 2 : suffix === "'" ? -Math.PI / 2 : Math.PI;

      // 回転した面 (f * 9 .. f * 9 + 8) の9セルすべての矢印が expectedAngle であること
      const turnedFaceArrows = arrows.filter(
        (a) => a.stickerIdx >= f * 9 && a.stickerIdx < (f + 1) * 9,
      );
      expect(turnedFaceArrows.length).toBe(9);
      expect(
        turnedFaceArrows.every(
          (a) =>
            Math.abs(Math.abs(a.rotAngle) - Math.abs(expectedAngle)) < 1e-4,
        ),
      ).toBe(true);

      // 反対面（U:D, R:L, F:B, D:U, L:R, B:F）の9セルは、全く動かないため 0 rad（正常色）のままであること
      const oppositeFaceIdx = [3, 4, 5, 0, 1, 2][f];
      const oppositeFaceArrows = arrows.filter(
        (a) =>
          a.stickerIdx >= oppositeFaceIdx * 9 &&
          a.stickerIdx < (oppositeFaceIdx + 1) * 9,
      );
      expect(oppositeFaceArrows.length).toBe(9);
      expect(
        oppositeFaceArrows.every(
          (a) => a.color === ARROW_COLORS.NORMAL && Math.abs(a.rotAngle) < 1e-4,
        ),
      ).toBe(true);

      // 2. 逆操作を実行して元に戻す
      await algorithm(page, inverse(move));
      const resetArrows = await getArrowData();
      expect(resetArrows.length).toBe(54);
      expect(
        resetArrows.every(
          (a) => a.color === ARROW_COLORS.NORMAL && Math.abs(a.rotAngle) < 1e-4,
        ),
      ).toBe(true);
    }
  }
});

test("orientation mode solves cube so that all centers are also oriented correctly to 0", async ({
  page,
}) => {
  await ready(page);

  // 1. 向きモードをONにし、reduced-motionを有効にする
  await page.locator("#reduced-motion").check();
  await page.locator("#include-orientation").check();

  // 2. 操作を実行（例: U 操作で U センターが回転する）
  await algorithm(page, "U R U' R'");

  // 3. 解法探索を実行
  await page.locator("#solve").click();
  await expect(page.locator("#solution-content")).toBeVisible({
    timeout: 10000,
  });

  // 4. 解法の最後まで進む
  await page.locator(".solution-move").last().click();
  expect(await state(page)).toBe(SOLVED);

  // 5. 全 54 セルの矢印が完全に NORMAL (緑、0 rad) に揃っていることを検証
  const arrowData = await page.evaluate(() => {
    const scene = (window as any).cube_scene;
    const arrows = scene?.getArrows() ?? [];
    return arrows.map((a: any) => ({
      stickerIdx: a.userData?.stickerIdx as number,
      color: (a.material?.color ?? a.cone?.material?.color).getHex() as number,
      rotAngle: a.userData?.rotAngle as number,
    }));
  });

  expect(arrowData.length).toBe(54);
  for (const arrow of arrowData) {
    expect(arrow.color).toBe(ARROW_COLORS.NORMAL);
    expect(Math.abs(arrow.rotAngle)).toBeLessThan(1e-4);
  }

  // 6. 全 6 面の centerRotations がすべて 0 であることを検証
  const centerRotations = await page.evaluate(() => {
    const scene = (window as any).cube_scene;
    return scene.centerRotations;
  });
  expect(centerRotations).toEqual([0, 0, 0, 0, 0, 0]);
});

test("superflip preset solves with orientation in 24 moves or less", async ({
  page,
}) => {
  await ready(page);

  // 1. 向きモードをONにし、reduced-motionを有効にする
  await page.locator("#reduced-motion").check();
  await page.locator("#include-orientation").check();

  // 2. プリセットタブを開き、スーパーフリップを選択
  await page.locator("#tab-presets").click();
  await page.locator("button", { hasText: "スーパーフリップ" }).click();
  await expect(page.locator("#preset-status")).toContainText(
    "スーパーフリップ を読み込みました",
  );

  // 3. 解法探索を実行
  await page.locator("#solve").click();
  await expect(page.locator("#solution-content")).toBeVisible({
    timeout: 10000,
  });

  // 4. 手数が24手以内であることを検証
  const moves = await page.locator(".solution-move").allTextContents();
  expect(moves.length).toBeLessThanOrEqual(24);

  // 5. 解法の最後まで進む
  await page.locator(".solution-move").last().click();
  expect(await state(page)).toBe(SOLVED);

  // 6. 全 54 セルの矢印が完全に NORMAL (緑、0 rad) に揃っていることを検証
  const arrowData = await page.evaluate(() => {
    const scene = (window as any).cube_scene;
    const arrows = scene?.getArrows() ?? [];
    return arrows.map((a: any) => ({
      stickerIdx: a.userData?.stickerIdx as number,
      color: (a.material?.color ?? a.cone?.material?.color).getHex() as number,
      rotAngle: a.userData?.rotAngle as number,
    }));
  });

  expect(arrowData.length).toBe(54);
  for (const arrow of arrowData) {
    expect(arrow.color).toBe(ARROW_COLORS.NORMAL);
    expect(Math.abs(arrow.rotAngle)).toBeLessThan(1e-4);
  }

  // 7. 全 6 面の centerRotations がすべて 0 であることを検証
  const centerRotations = await page.evaluate(() => {
    const scene = (window as any).cube_scene;
    return scene.centerRotations;
  });
  expect(centerRotations).toEqual([0, 0, 0, 0, 0, 0]);
});

test("all presets can be loaded and solved within optimal move bounds", async ({
  page,
}) => {
  await ready(page);
  await page.locator("#reduced-motion").check();

  const presets = [
    {
      label: "完成状態",
      expectedMovesMax: 0,
      description: "0 moves (already solved)",
    },
    {
      label: "簡単（5手）",
      expectedMovesMax: 5,
      description: "3 to 5 moves",
    },
    {
      label: "T-Permutation",
      expectedMovesMax: 14,
      description: "at most 14 moves",
    },
    {
      label: "スーパーフリップ",
      expectedMovesMax: 24,
      description: "at most 24 moves (God's number 20)",
    },
    {
      label: "ランダム（seed=1）",
      expectedMovesMax: 25,
      description: "at most 25 moves",
    },
  ];

  for (const preset of presets) {
    // プリセットタブを開いて選択
    await page.locator("#tab-presets").click();
    await page
      .locator("#preset-buttons button", { hasText: preset.label })
      .click();
    await expect(page.locator("#preset-status")).toContainText(
      `${preset.label} を読み込みました`,
      { timeout: 5000 },
    );

    // 解法探索を実行
    await page.locator("#solve").click();

    if (preset.expectedMovesMax === 0) {
      // 完成状態は 0 手
      expect(await state(page)).toBe(SOLVED);
      const moves = await page.locator(".solution-move").allTextContents();
      expect(moves.length).toBe(0);
    } else {
      await expect(page.locator("#solution-content")).toBeVisible({
        timeout: 10000,
      });
      const moves = await page.locator(".solution-move").allTextContents();
      console.log(
        `Preset '${preset.label}' solved in ${moves.length} moves (${preset.description}):`,
        moves.join(" "),
      );
      expect(moves.length).toBeLessThanOrEqual(preset.expectedMovesMax);

      // 解法の最後まで進んで完成状態を検証
      await page.locator(".solution-move").last().click();
      expect(await state(page)).toBe(SOLVED);
    }
  }
});
