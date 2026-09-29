import { test, expect } from "@playwright/test";

const SUPERFLIP_STATE =
  "UBULURUFURURFRBRDRFUFLFRFDFDFDLDRDBDLULBLFLDLBUBRBLBDB";

test.describe("7fc8ac3 レビュー指摘点 (F1〜F5) 回帰テスト", () => {
  // F1: 解法プレビューシーク中の「保存」（#save）によるスクランブル局面の確実な保存
  test("F1: 解法プレビューシーク中に『保存』を押した際、プレビュー途中局面ではなく元のスクランブル局面がファイル保存される", async ({
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

    // #save ボタン押下時の createObjectURL をインターセプトして保存内容を捕捉
    await page.evaluate(() => {
      const origCreate = URL.createObjectURL;
      URL.createObjectURL = (blob) => {
        blob.text().then((text) => {
          (window as any).__lastSavedJson = JSON.parse(text);
        });
        return origCreate(blob);
      };
    });
    await page.locator("#save").click();
    await page.waitForTimeout(300);

    const savedJson = await page.evaluate(
      () => (window as any).__lastSavedJson,
    );

    // 保存された JSON の局面が、完成局面ではなく元のスクランブル局面であることを検証
    expect(savedJson.state).toBe(scrambleState);
    expect(savedJson.state).not.toContain(
      "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB",
    );
  });

  // F2: 解法パネル「閉じる」押下時の局面確定と Undo による復元性（R02 準拠）
  test("F2: 解法プレビューシーク後に解法カードの『×』を押した際、完成局面が維持され Undo で元のスクランブル局面に復元できる", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

    await page.locator('button[data-move="R"]').click();
    await page.waitForTimeout(200);

    const scrambleState = await page.evaluate(() =>
      window.__cube_main_debug__?.store?.getState(),
    );

    await page.locator("#solve").click();
    await page.locator("#move-list button").first().waitFor();

    // 完成状態までプレビューを進める
    await page.locator("#last").click();
    await page.waitForTimeout(200);

    // 解法カードの「×」（閉じる）ボタンを押す
    await page.locator("#solution-close").click();
    await page.waitForTimeout(200);

    const stateAfterClose = await page.evaluate(() =>
      window.__cube_main_debug__?.store?.getState(),
    );
    const statusTextAfterClose = await page
      .locator("#cube-status")
      .textContent();
    const solveButtonText = await page.locator("#solve").textContent();

    // 閉じた後は完成状態が維持されていること（R02 仕様）
    expect(stateAfterClose).toBe(
      "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB",
    );
    expect(statusTextAfterClose).toBe("完成状態");
    expect(solveButtonText).toContain("完成状態を確認");

    // Undo を押すと元のスクランブル状態に復元されること
    await page.locator("#undo").click();
    const stateAfterUndo = await page.evaluate(() =>
      window.__cube_main_debug__?.store?.getState(),
    );
    expect(stateAfterUndo).toBe(scrambleState);
  });

  // F3: プリセット非同期通信待機中の solve() 開始によるレースコンディション保護
  test("F3: プリセット通信待機中に探索が開始された場合、遅延到着したプリセットデータで探索が中断・上書きされない", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

    const result = await page.evaluate(async () => {
      const store = window.__cube_main_debug__?.store;
      const initialRev = store?.getRevision();

      let fetchResolver: () => void = () => {};
      const slowFetchPromise = new Promise<void>((resolve) => {
        fetchResolver = resolve;
      });

      const originalFetch = window.fetch;
      window.fetch = async (...args) => {
        if (typeof args[0] === "string" && args[0].includes("superflip.json")) {
          await slowFetchPromise;
          return new Response(
            JSON.stringify({
              state: "UBULURUFURURFRBRDRFUFLFRFDFDFDLDRDBDLULBLFLDLBUBRBLBDB",
            }),
            { status: 200, headers: { "Content-Type": "application/json" } },
          );
        }
        return originalFetch(...args);
      };

      // プリセットボタンをクリック
      const presetBtn = document.querySelector(
        "#preset-buttons button:nth-child(2)",
      ) as HTMLButtonElement;
      presetBtn?.click();

      // fetch 待機中にユーザーが solve() を開始
      const solvePromise = window.__cube_main_debug__?.solve(5000);

      // fetch を完了させる
      fetchResolver();
      await new Promise((r) => setTimeout(r, 100));

      window.fetch = originalFetch;

      const presetStatus =
        document.getElementById("preset-status")?.textContent;

      return {
        initialRev,
        presetStatus,
      };
    });

    // 探索を優先し、プリセットで上書きされなかったメッセージを確認
    expect(result.presetStatus).toContain(
      "読み込み中にキューブが操作されたため、現在の操作を優先しました",
    );

    // 後処理
    if (await page.locator("#cancel").isVisible()) {
      await page.locator("#cancel").click();
    }
  });

  // F4: 空アルゴリズムまたは空白のみ適用時の解法破棄と不要回転音の防止
  test("F4: 空入力または空白のみで『手順を適用』をクリックした際、表示中の解法が破棄されない", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

    await page.locator('button[data-move="R"]').click();
    await page.locator("#solve").click();
    await page.locator("#move-list button").first().waitFor();

    const solutionBefore = await page.evaluate(
      () => !!window.__cube_main_debug__?.store?.getSolution(),
    );
    expect(solutionBefore).toBe(true);

    // 手順入力タブを開いて空白スペースのみを入力して適用
    await page.locator('[data-tab="moves"]').click();
    await page.locator("#algorithm").fill("   ");
    await page.locator("#apply-algorithm").click();
    await page.waitForTimeout(200);

    const solutionAfter = await page.evaluate(
      () => !!window.__cube_main_debug__?.store?.getSolution(),
    );

    // 空入力によって解法が破棄されず、維持されていることを検証
    expect(solutionAfter).toBe(true);
  });

  // F5: 探索中の修飾子ボタン無効化とキーボード active-press 点灯防止
  test("F5: 探索中に #prime, #double ボタンが無効化され、キーボード押下時にも回転ボタンに active-press が付与されない", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

    await page.evaluate((state) => {
      window.__cube_main_debug__?.store?.replace(state);
    }, SUPERFLIP_STATE);
    await page.locator("#solver-algorithm").selectOption("korf");
    await page.locator("#solve").click();
    await page.locator("#cancel").waitFor({ state: "visible" });

    // 修飾子ボタンが無効化されていること
    const primeDisabled = await page.locator("#prime").isDisabled();
    const doubleDisabled = await page.locator("#double").isDisabled();
    expect(primeDisabled).toBe(true);
    expect(doubleDisabled).toBe(true);

    // 探索中にキーボードで 'U' を押し下げても active-press が点灯しないこと
    await page.keyboard.down("KeyU");
    await page.waitForTimeout(50);

    const uBtnActivePress = await page
      .locator('button[data-move="U"]')
      .evaluate((el) => el.classList.contains("active-press"));
    await page.keyboard.up("KeyU");

    expect(uBtnActivePress).toBe(false);

    // 後処理
    await page.locator("#cancel").click();
  });
});
