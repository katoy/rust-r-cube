import { test, expect, type Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import { SOLVED_3X3 } from "../web/model";
import { SOLVED_2X2 } from "../web/model-2x2";
import { holdSolverResults, releaseSolverResults } from "./solver-barrier";

async function ready(page: Page, url = "/?no-sw&alg=R_U_R'_U'") {
  await page.goto(url);
  await expect(page.locator("#engine-status")).toContainText("READY");
  await page.locator("#reduced-motion").check();
}

async function getSnapshot(page: Page) {
  return page.evaluate(() => {
    const store = window.cube_store;
    if (!store) throw new Error("Cube store is unavailable");
    return {
      state: store.getState(),
      step: store.getStep(),
      solutionMoves: store.getSolution()?.moves.length ?? 0,
      algorithm: store.getSolution()?.algorithm,
      fsm: window.__cube_main_debug__?.appState.kind,
    };
  });
}

test.describe("全ソルバー一括比較モード (Algorithm Compare)", () => {
  test("3x3x3 全4解法を比較表示し、最短バッジとアクセシビリティ基準を満たす", async ({
    page,
  }) => {
    await ready(page);

    await page.locator("#compare-solvers").click();
    await expect(page.locator("#solver-compare-panel")).toBeVisible();
    await expect(page.locator("#solution-content")).toBeVisible({
      timeout: 20000,
    });

    const rows = page.locator("#solver-compare-list .solver-compare-row");
    await expect(rows).toHaveCount(4);

    for (const algo of ["kociemba", "cfop", "thistlethwaite", "korf"]) {
      const row = page.locator(
        `#solver-compare-list .solver-compare-row[data-solver="${algo}"]`,
      );
      await expect(row).toBeVisible();
      await expect(row).toBeEnabled();
      await expect(row.locator(".compare-moves")).toContainText("手");
    }

    await expect(
      page.locator("#solver-compare-list .compare-badge-shortest").first(),
    ).toBeVisible();

    const axeResults = await new AxeBuilder({ page }).analyze();
    expect(axeResults.violations).toEqual([]);
  });

  test("比較結果からのワンクリック切替再生で基準盤面と Undo 履歴が整合する", async ({
    page,
  }) => {
    await ready(page, "/?no-sw&alg=R");
    const preMove = (await getSnapshot(page)).state;

    await page.locator('[data-move="U"]').click();
    const scrambledBase = (await getSnapshot(page)).state;
    expect(scrambledBase).not.toBe(preMove);

    await page.locator("#compare-solvers").click();
    await expect(page.locator("#solution-content")).toBeVisible({
      timeout: 20000,
    });

    // 初期選択（kociemba）で最後までシーク
    await page.locator("#last").click();
    expect((await getSnapshot(page)).state).toBe(SOLVED_3X3);

    // CFOP 行をクリックして切替：基準盤面（0手目）へ復元される
    const cfopRow = page.locator(
      '#solver-compare-list .solver-compare-row[data-solver="cfop"]',
    );
    const cfopMovesText = await cfopRow.locator(".compare-moves").textContent();
    const cfopExpectedMoves = parseInt(cfopMovesText || "0", 10);

    await cfopRow.click();
    await expect(cfopRow).toHaveAttribute("aria-pressed", "true");
    await expect(page.locator("#solver-algorithm")).toHaveValue("cfop");

    const afterSwitch = await getSnapshot(page);
    expect(afterSwitch.step).toBe(0);
    expect(afterSwitch.state).toBe(scrambledBase);
    expect(afterSwitch.algorithm).toBe("cfop");
    expect(afterSwitch.solutionMoves).toBe(cfopExpectedMoves);

    // 切替後の解で最後まで進めた後、Undo 2回で元の盤面へ戻せること
    await page.locator("#last").click();
    expect((await getSnapshot(page)).state).toBe(SOLVED_3X3);

    await page.locator("#undo").click();
    expect((await getSnapshot(page)).state).toBe(scrambledBase);

    await page.locator("#undo").click();
    expect((await getSnapshot(page)).state).toBe(preMove);
  });

  test("2x2x2 全3解法を比較表示して切り替えられる", async ({ page }) => {
    await ready(page, "/?no-sw");
    await page.locator("#cube-type-2x2").click();
    await page.locator('[data-move="R"]').click();
    await page.locator('[data-move="U"]').click();
    const scrambled2x2 = (await getSnapshot(page)).state;
    expect(scrambled2x2).not.toBe(SOLVED_2X2);

    await page.locator("#compare-solvers").click();
    await expect(page.locator("#solution-content")).toBeVisible({
      timeout: 15000,
    });

    const rows = page.locator("#solver-compare-list .solver-compare-row");
    await expect(rows).toHaveCount(3);
    for (const algo of ["optimal", "ortega", "lbl"]) {
      const row = page.locator(
        `#solver-compare-list .solver-compare-row[data-solver="${algo}"]`,
      );
      await expect(row).toBeEnabled();
      await expect(row.locator(".compare-moves")).toContainText("手");
    }

    await page
      .locator('#solver-compare-list .solver-compare-row[data-solver="ortega"]')
      .click();
    await expect(page.locator("#solver-algorithm")).toHaveValue("ortega");
    expect((await getSnapshot(page)).algorithm).toBe("ortega");

    await page.locator("#last").click();
    expect((await getSnapshot(page)).state).toBe(SOLVED_2X2);
  });

  test("探索中のキャンセルおよび閉じる／盤面変更で比較パネルがクリアされる", async ({
    page,
  }) => {
    await ready(page);

    // 1. 比較中のキャンセル
    await holdSolverResults(page);
    try {
      await page.locator("#compare-solvers").click();
      await expect(page.locator("#cancel")).toBeVisible();
      await expect(page.locator("#solver-compare-panel")).toBeVisible();
      await expect(
        page.locator(
          '#solver-compare-list .solver-compare-row[data-solver="kociemba"] .compare-status-pending',
        ),
      ).toHaveText("計算中…");

      await page.locator("#cancel").click();
      await expect(page.locator("#solver-compare-panel")).toBeHidden();
      expect((await getSnapshot(page)).fsm).toBe("idle");
    } finally {
      await releaseSolverResults(page);
    }

    // 2. 比較完了後の閉じる操作でパネルが非表示になること
    await page.locator("#compare-solvers").click();
    await expect(page.locator("#solution-content")).toBeVisible({
      timeout: 20000,
    });
    await page.locator("#solution-close").click();
    await expect(page.locator("#solver-compare-panel")).toBeHidden();
    await expect(page.locator("#solution-empty")).toBeVisible();

    // 3. 手動回転で盤面変更した際も比較パネルがクリアされること
    await page.locator("#compare-solvers").click();
    await expect(page.locator("#solution-content")).toBeVisible({
      timeout: 20000,
    });
    await page.locator('[data-move="F"]').click();
    await expect(page.locator("#solver-compare-panel")).toBeHidden();
  });
});
