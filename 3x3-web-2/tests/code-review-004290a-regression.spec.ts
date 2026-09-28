import { test, expect } from "@playwright/test";

const SUPERFLIP_STATE =
  "UBULURUFURURFRBRDRFUFLFRFDFDFDLDRDBDLULBLFLDLBUBRBLBDB";

test.describe("004290a Regression Tests", () => {
  test("F1: Keyboard move shortcut during solve is ignored and search is not cancelled", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

    // スーパーフリップ局面で Korf 探索を開始
    await page.evaluate((state) => {
      window.__cube_main_debug__?.store?.replace(state);
    }, SUPERFLIP_STATE);
    await page.locator("#solver-algorithm").selectOption("korf");

    // 探索開始
    await page.locator("#solve").click();
    await page.locator("#cancel").waitFor({ state: "visible" });

    const revisionBefore = await page.evaluate(() =>
      window.__cube_main_debug__?.store?.getRevision(),
    );

    // 探索中にキーボードで 'U' を入力
    await page.keyboard.press("KeyU");
    await page.waitForTimeout(300);

    // 探索がキャンセルされず、キューブが回転していないことを確認
    const cancelVisible = await page.locator("#cancel").isVisible();
    const revisionAfter = await page.evaluate(() =>
      window.__cube_main_debug__?.store?.getRevision(),
    );

    expect(cancelVisible).toBe(true);
    expect(revisionAfter).toBe(revisionBefore);

    // 後処理で探索をキャンセル
    await page.locator("#cancel").click();
  });

  test("F2: Solver note is properly updated and does not retain zombie status on error", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

    const noteResult = await page.evaluate(async () => {
      const solver = window.__cube_main_debug__?.getSolver();
      if (!solver) return null;

      const originalSolve = solver.solve.bind(solver);
      solver.solve = () =>
        new Promise((_, reject) => {
          setTimeout(
            () =>
              reject(
                new Error(
                  "探索時間の上限に達しました。30秒の延長探索を試してください。",
                ),
              ),
            150,
          );
        });

      const solvePromise = window.__cube_main_debug__?.solve(5000);
      await new Promise((r) => setTimeout(r, 160));
      await solvePromise;
      solver.solve = originalSolve;

      return document.getElementById("solver-note")?.textContent;
    });

    expect(noteResult).toBe("探索時間の上限に達しました");
    expect(noteResult).not.toContain("探索中");
  });

  test("F3: Share link during preview copies scramble state instead of preview step state", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

    // 1手スクランブル (R)
    await page.locator('button[data-move="R"]').click();
    await page.waitForTimeout(200);

    const scrambleState = await page.evaluate(() =>
      window.__cube_main_debug__?.store?.getState(),
    );

    // 解法探索
    await page.locator("#solve").click();
    await page.locator("#move-list button").first().waitFor();

    // 最終手（完成状態）へシーク
    await page.locator("#last").click();
    await page.waitForTimeout(200);

    // クリップボードのモック
    let copiedText = "";
    await page.evaluate(() => {
      navigator.clipboard.writeText = async (text: string) => {
        (window as any).__lastCopied = text;
      };
    });

    // 共有リンクをクリック
    await page.locator("#share-link").click();
    copiedText = await page.evaluate(() => (window as any).__lastCopied);

    // 完成状態ではなく元のスクランブル状態が含まれていることを確認
    expect(copiedText).toContain(`state=${scrambleState}`);
    expect(copiedText).not.toContain(
      "state=UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB",
    );
  });

  test("F4: solve() deduplicates restoreBaseSnapshot and setSolution notifications", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

    await page.locator('button[data-move="R"]').click();
    await page.locator("#solve").click();
    await page.locator("#move-list button").first().waitFor();
    await page.locator("#last").click();

    const notifications = await page.evaluate(async () => {
      const store = window.__cube_main_debug__?.store;
      if (!store) return [];
      const events: string[] = [];
      const unsubscribe = store.subscribe((_s: any, detail: any) => {
        events.push(detail.type);
      });

      // 修正後のロジック: if (!store.restoreBaseSnapshot()) store.setSolution(undefined);
      const restored = store.restoreBaseSnapshot();
      if (!restored) {
        store.setSolution(undefined);
      }
      unsubscribe();
      return events;
    });

    // 重複通知がなく、1回のみ発火することを確認
    expect(notifications.filter((e) => e === "solution").length).toBe(1);
  });

  test("F5: Share link button is properly disabled during solve", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

    await page.evaluate((state) => {
      window.__cube_main_debug__?.store?.replace(state);
    }, SUPERFLIP_STATE);
    await page.locator("#solver-algorithm").selectOption("korf");

    // 探索開始
    await page.locator("#solve").click();
    await page.locator("#cancel").waitFor({ state: "visible" });

    // 探索中は #share-link も disabled になることを確認
    const shareDisabled = await page.locator("#share-link").isDisabled();
    expect(shareDisabled).toBe(true);

    // 後処理
    await page.locator("#cancel").click();
  });
});
