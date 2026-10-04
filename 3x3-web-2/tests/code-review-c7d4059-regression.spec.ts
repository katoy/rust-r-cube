import { test, expect, type Page } from "@playwright/test";
import { normalizeOutlinePoints } from "../web/camera-geometry";

async function ready(page: Page, url = "/?no-sw") {
  await page.goto(url);
  await expect(page.locator("#engine-status")).toContainText("READY");
  await page.locator("#reduced-motion").check();
}

async function snapshot(page: Page) {
  return page.evaluate(() => {
    const store = window.cube_store;
    if (!store) throw new Error("Cube store is unavailable");
    return {
      state: store.getState(),
      centers: store.getCenterTurns(),
      canUndo: store.canUndo(),
      canRedo: store.canRedo(),
      hasSolution: !!store.getSolution(),
      fsm: window.__cube_main_debug__?.appState.kind,
    };
  });
}

test("M5: URL query params (?alg=R) are cleared from URL bar and state persists across reloads", async ({
  page,
}) => {
  // 1. 共有 URL ?alg=R でアクセス
  await ready(page, "/?no-sw&alg=R");

  // 2. URL からクエリパラメータが消去されていることを検証 (M5)
  const currentUrl = page.url();
  expect(currentUrl).not.toContain("alg=R");

  // 現在のキューブ状態を確認（Rが適用されている）
  const snap1 = await snapshot(page);

  // 3. 次の操作（例: U）を実行
  await page.locator('[data-move="U"]').click();
  const snap2 = await snapshot(page);
  expect(snap2.state).not.toBe(snap1.state);

  // 4. リロードを実行
  await page.reload();
  await expect(page.locator("#engine-status")).toContainText("READY");

  // 5. リロード後も U を適用した状態が復元されており、古い ?alg=R の初期状態に戻っていないことを検証
  const snapAfterReload = await snapshot(page);
  expect(snapAfterReload.state).toBe(snap2.state);
});

test("M7: canRedo() checks actual snapshot difference against future head", async ({
  page,
}) => {
  await ready(page);

  // 1. 操作 R' を実行 (Shift + click)
  await page.locator('button[data-move="R"]').click({ modifiers: ["Shift"] });
  // 2. 操作 R を実行
  await page.locator('button[data-move="R"]').click();

  // 3. Undo を実行（R' の状態に戻る）
  await page.locator("#undo").click();
  expect((await snapshot(page)).canRedo).toBe(true);

  // 4. 解法を探す（解法は [R] になる）
  await page.locator("#solve").click();
  await expect(page.locator("#solution-content")).toBeVisible();

  // 5. 1手目へシークすると、盤面が future の先頭と同じになる
  await page.locator("#next").click();

  // このとき盤面と future の先頭が同一であるため、実質的に redo できる差分はない (M7)
  const snap = await snapshot(page);
  expect(snap.canRedo).toBe(false);
});

test("H1 / M8: normalizeOutlinePoints sorts points clockwise regardless of CCW input", () => {
  // 重心 (100, 100) 周りの正六角形頂点
  // 角度順: P0:最上部(100, 40), P1:(150, 70), P2:(150, 130), P3:(100, 160), P4:(50, 130), P5:(50, 70)
  const p0 = { x: 100, y: 40 };
  const p1 = { x: 150, y: 70 };
  const p2 = { x: 150, y: 130 };
  const p3 = { x: 100, y: 160 };
  const p4 = { x: 50, y: 130 };
  const p5 = { x: 50, y: 70 };

  // 時計回り
  const clockwise = [p0, p1, p2, p3, p4, p5];
  const normClockwise = normalizeOutlinePoints(clockwise);
  expect(normClockwise).toEqual([p0, p1, p2, p3, p4, p5]);

  // 反時計回り (P0, P5, P4, P3, P2, P1)
  const counterClockwise = [p0, p5, p4, p3, p2, p1];
  const normCounter = normalizeOutlinePoints(counterClockwise);
  expect(normCounter).toEqual([p0, p1, p2, p3, p4, p5]);

  // 任意の開始点からの回転 (P2 から始まる時計回り)
  const shifted = [p2, p3, p4, p5, p0, p1];
  const normShifted = normalizeOutlinePoints(shifted);
  expect(normShifted).toEqual([p0, p1, p2, p3, p4, p5]);
});
