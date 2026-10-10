import { test, expect, type Page } from "@playwright/test";
import { getErrorIndices } from "../web/model";
import { CORNERS_2X2, SOLVED_2X2 } from "../web/model-2x2";

// レビュー e0a282e §3.3（Web 状態管理）の指摘を、実際の UI 操作順で検証する

const SOLVED_3X3 = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
const DEFAULT_SCRAMBLE_TEXT = "ランダムな回転で、新しい状態をつくります。";

async function ready(page: Page, url = "/?no-sw") {
  await page.goto(url);
  await expect(page.locator("#engine-status")).toContainText("READY");
  await page.locator("#reduced-motion").check();
}

const storeState = (page: Page) =>
  page.evaluate(() => (window as any).cube_store.getState() as string);

const centerTurns = (page: Page) =>
  page.evaluate(() => (window as any).cube_store.getCenterTurns() as number[]);

const savedState = (page: Page) =>
  page.evaluate(() => {
    const raw = localStorage.getItem("cube-studio-v1");
    return raw ? (JSON.parse(raw).state as string) : undefined;
  });

async function applyAlgorithm(page: Page, algorithm: string) {
  await page.locator('[data-tab="moves"]').click();
  await page.locator("#algorithm").fill(algorithm);
  await page.locator("#apply-algorithm").click();
}

// 結果を返さず探索中のまま止まる Worker に差し替える
async function routeHangingWorker(page: Page) {
  await page.route("**/web/solver.worker.ts*", (route) =>
    route.fulfill({
      contentType: "application/javascript",
      body: `postMessage({kind:'ready',elapsed:1}); onmessage=()=>{};`,
    }),
  );
}

test.describe("キューブ種別切替と履歴（M1 / L1）", () => {
  test("3x3 の盤面 → 2x2 → 3x3 と切り替えても Undo で元の盤面に戻せる", async ({
    page,
  }) => {
    await ready(page);
    await page.locator("#scramble").click();
    const scrambled = await storeState(page);
    expect(scrambled).not.toBe(SOLVED_3X3);
    await expect(page.locator("#scramble-text")).not.toHaveText(
      DEFAULT_SCRAMBLE_TEXT,
    );

    await page.locator("#cube-type-2x2").click();
    await expect.poll(() => storeState(page)).toBe(SOLVED_2X2);
    // L1: 3x3 のスクランブル手順が 2x2 の横に残らない
    await expect(page.locator("#scramble-text")).toHaveText(
      DEFAULT_SCRAMBLE_TEXT,
    );
    await expect(page.locator("#undo")).toBeEnabled();

    await page.locator("#cube-type-3x3").click();
    await expect.poll(() => storeState(page)).toBe(SOLVED_3X3);

    await page.locator("#undo").click();
    await expect.poll(() => storeState(page)).toBe(SOLVED_2X2);
    await expect(page.locator("#cube-type-2x2")).toHaveAttribute(
      "aria-checked",
      "true",
    );

    await page.locator("#undo").click();
    await expect.poll(() => storeState(page)).toBe(scrambled);
    await expect(page.locator("#cube-type-3x3")).toHaveAttribute(
      "aria-checked",
      "true",
    );
    await expect(page.locator("#solver-algorithm")).toHaveValue("kociemba");

    // Redo で切替後の状態へ進められる
    await page.locator("#redo").click();
    await expect.poll(() => storeState(page)).toBe(SOLVED_2X2);
  });

  test("探索中は種別ボタンが無効になり、盤面も種別も変わらない", async ({
    page,
  }) => {
    await routeHangingWorker(page);
    await ready(page);
    await page.locator("#scramble").click();
    const scrambled = await storeState(page);

    await page.locator("#solve").click();
    await expect(page.locator("#cancel")).toBeVisible();
    await expect(page.locator("#cube-type-2x2")).toBeDisabled();
    await expect(page.locator("#cube-type-3x3")).toBeDisabled();

    await page.locator("#cube-type-2x2").click({ force: true });
    expect(await storeState(page)).toBe(scrambled);
    await expect(page.locator("#cancel")).toBeVisible();

    await page.locator("#cancel").click();
    await expect(page.locator("#cube-type-2x2")).toBeEnabled();
    await page.locator("#cube-type-2x2").click();
    await expect.poll(() => storeState(page)).toBe(SOLVED_2X2);
  });
});

test.describe("URL の solver と種別の整合（M2 / L2 / L3）", () => {
  test("前回 2x2 の状態で ?solver=cfop を開くと lbl に戻してメッセージを出す", async ({
    page,
  }) => {
    // 前回の作業が 2x2 だった状態を、ページ読込前に用意する
    await page.addInitScript((state) => {
      localStorage.setItem(
        "cube-studio-v1",
        JSON.stringify({
          version: 1,
          state,
          centerTurns: [0, 0, 0, 0, 0, 0],
          cubeType: "2x2",
          solverAlgorithm: "ortega",
        }),
      );
    }, SOLVED_2X2);
    await ready(page, "/?no-sw&solver=cfop&alg=R_U_R'_U'");

    const select = page.locator("#solver-algorithm");
    await expect(select).toHaveValue("lbl");
    expect(
      await select.evaluate((el: HTMLSelectElement) => el.selectedIndex),
    ).toBeGreaterThanOrEqual(0);
    await expect(page.locator("#message")).toContainText("cfop");
    await expect(page.locator("#message")).toContainText("2x2");
    // L2: エンジン準備完了後も 2x2 用の注記が表示される
    await expect(page.locator("#solver-note")).not.toContainText("通常5秒以内");
    await expect(page.locator("#solver-note")).toContainText("完全1層");
  });

  test("3x3 で ?solver=optimal を開くと kociemba に戻す", async ({ page }) => {
    await ready(page, "/?no-sw&solver=optimal");
    await expect(page.locator("#solver-algorithm")).toHaveValue("kociemba");
    await expect(page.locator("#message")).toContainText("optimal");
  });

  test("URL パラメータで適用した盤面は起動直後に保存される（L3）", async ({
    page,
  }) => {
    await ready(page, "/?no-sw&alg=R_U");
    const current = await storeState(page);
    expect(current).not.toBe(SOLVED_3X3);
    await expect.poll(() => savedState(page)).toBe(current);
  });
});

test.describe("2x2 のセンター向き（M3）", () => {
  test("盤面が完成に戻る手順を適用してもセンター向きが残らず、Undo も無効", async ({
    page,
  }) => {
    await ready(page, "/?no-sw&type=2x2");
    await applyAlgorithm(page, "U R L U2 R' L' U R L U2 R' L'");
    await expect(page.locator("#cube-status")).toHaveText("完成状態");
    expect(await storeState(page)).toBe(SOLVED_2X2);
    expect(await centerTurns(page)).toEqual([0, 0, 0, 0, 0, 0]);
    await expect(page.locator("#undo")).toBeDisabled();
  });

  test("2x2 で L を含む手順を適用・シークしてもセンター回転は 0 のまま", async ({
    page,
  }) => {
    await ready(page, "/?no-sw&type=2x2");
    await applyAlgorithm(page, "L D B");
    expect(await centerTurns(page)).toEqual([0, 0, 0, 0, 0, 0]);

    await page.locator("#solve").click();
    await expect(page.locator("#solution-content")).toBeVisible();
    await page.locator(".solution-move").last().click();
    await expect.poll(() => storeState(page)).toBe(SOLVED_2X2);
    expect(await centerTurns(page)).toEqual([0, 0, 0, 0, 0, 0]);
  });
});

test.describe("プレビューと永続化（M4 / L4）", () => {
  test("シーク途中の盤面ではなく、確定した盤面が保存・復元される", async ({
    page,
  }) => {
    await ready(page, "/?no-sw&alg=R_U_F_L_D");
    const base = await storeState(page);
    await expect.poll(() => savedState(page)).toBe(base);

    await page.locator("#solve").click();
    await expect(page.locator("#solution-content")).toBeVisible();
    await page.locator('.solution-move[data-step="2"]').click();
    await expect(page.locator("#step-count")).toHaveText(/^3 \//);
    expect(await storeState(page)).not.toBe(base);
    expect(await savedState(page)).toBe(base);

    await page.reload();
    await expect(page.locator("#engine-status")).toContainText("READY");
    expect(await storeState(page)).toBe(base);
  });

  test("再探索しても Redo 履歴が消えない", async ({ page }) => {
    await ready(page);
    await applyAlgorithm(page, "R");
    await applyAlgorithm(page, "U");
    await page.locator("#undo").click();
    await expect(page.locator("#redo")).toBeEnabled();

    await page.locator("#solve").click();
    await expect(page.locator("#solution-content")).toBeVisible();
    await expect(page.locator("#redo")).toBeEnabled();

    // 再探索（base への復元を伴う）
    await page.locator("#solve").click();
    await expect(page.locator("#solution-content")).toBeVisible();
    await expect(page.locator("#solve")).toBeVisible();
    await expect(page.locator("#redo")).toBeEnabled();
  });
});

test.describe("2x2 配色エラーの強調表示（I5）", () => {
  test("getErrorIndices が 2x2 のスロット番号を解釈する", () => {
    const msg = "スロット 3 のコーナー配色が物理的に不正です。";
    expect(getErrorIndices(msg, "2x2")).toEqual(CORNERS_2X2[2]);
    expect(getErrorIndices("スロット 9 のコーナー配色", "2x2")).toEqual([]);
    expect(getErrorIndices(msg)).toEqual([]);
  });

  test("2x2 エディタで不正なコーナーを適用すると該当マスが強調される", async ({
    page,
  }) => {
    await ready(page, "/?no-sw&type=2x2");
    await page.locator('[data-tab="colors"]').click();
    await page.locator("#edit-colors").click();
    await expect(page.locator("#editor")).toBeVisible();

    // UFR コーナー（スロット 1）の R 面ステッカー(4) と F 面ステッカー(8) を入れ替える
    const palette = page.locator("#palette .color-choice");
    await palette.nth(2).click(); // F
    await page.locator('#editor-net [data-index="4"]').click();
    await palette.nth(1).click(); // R
    await page.locator('#editor-net [data-index="8"]').click();
    await page.locator("#editor-apply").click();

    await expect(page.locator("#editor-error")).toContainText("スロット");
    await expect(page.locator("#editor-error")).toContainText("スロット 1");
    const errorIndices = await page
      .locator("#editor-net .sticker.is-error")
      .evaluateAll((cells) => cells.map((c) => Number(c.dataset.index)));
    expect(errorIndices.sort((a, b) => a - b)).toEqual(
      [...CORNERS_2X2[0]].sort((a, b) => a - b),
    );
  });
});
