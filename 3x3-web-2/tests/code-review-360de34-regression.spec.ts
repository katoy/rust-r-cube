import { test, expect } from "@playwright/test";

const SOLVED = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
const SUPERFLIP_STATE =
  "UBULURUFURURFRBRDRFUFLFRFDFDFDLDRDBDLULBLFLDLBUBRBLBDB";

test.describe("360de34 レビュー指摘点 (F1〜F5) 回帰テスト", () => {
  // F1: 解法シーク後に解法カードを閉じた際の局面確定と Undo 復元性（R02 永続化）
  test("F1: 解法シーク後に解法カードを閉じた際、完成局面が維持され Undo で元のスクランブルに戻せる", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

    await page.locator('button[data-move="R"]').click();
    const scrambleState = await page.evaluate(() =>
      window.__cube_main_debug__?.store?.getState(),
    );
    expect(scrambleState).not.toBe(SOLVED);

    await page.locator("#solve").click();
    await page.locator("#move-list button").first().waitFor();

    await page.locator("#last").click();
    const stateAtLast = await page.evaluate(() =>
      window.__cube_main_debug__?.store?.getState(),
    );
    expect(stateAtLast).toBe(SOLVED);

    // 解法カードを閉じる
    await page.locator("#solution-close").click();

    const stateAfterClose = await page.evaluate(() =>
      window.__cube_main_debug__?.store?.getState(),
    );
    expect(stateAfterClose).toBe(SOLVED);

    // Undo で元のスクランブル局面に戻れること
    await page.locator("#undo").click();
    const stateAfterUndo = await page.evaluate(() =>
      window.__cube_main_debug__?.store?.getState(),
    );
    expect(stateAfterUndo).toBe(scrambleState);
  });

  // F2: 不正局面での探索失敗時に「30秒で再探索」ボタンとタイムアウト文言が誤表示されない
  test("F2: 不正局面での探索失敗時に『30秒で再探索』ボタンが表示されず、ステータスが適切に表示される", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

    // センター向きと配色が不整合な局面を設定（パリティエラー）
    await page.evaluate(() => {
      const store = window.__cube_main_debug__?.store;
      store?.replace(
        "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB",
        true,
        [1, 0, 0, 0, 0, 0],
      );
    });

    await page.locator("#solve").click();
    await page.waitForTimeout(300);

    const errorMessage = await page.locator("#message").textContent();
    const solverNote = await page.locator("#solver-note").textContent();
    const extendedVisible = await page.locator("#extended").isVisible();

    expect(errorMessage).toContain("センターの向きと配色が整合しません");
    expect(solverNote).not.toBe("探索時間の上限に達しました");
    expect(solverNote).toBe("探索できませんでした");
    expect(extendedVisible).toBe(false);
  });

  // F3: 解法プレビューシーク中に「配色を手動で編集」を開いた際、現在のプレビュー局面がエディタに読み込まれ、適用時は Undo 履歴にスクランブルが保持される
  test("F3: 解法プレビューシーク中に『配色を手動で編集』を開いた際、現在のプレビュー局面がエディタに読み込まれ、エディタ適用時にスクランブルが Undo で復元可能", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

    await page.locator('button[data-move="R"]').click();
    const scrambleState = await page.evaluate(() =>
      window.__cube_main_debug__?.store?.getState(),
    );

    await page.locator("#solve").click();
    await page.locator("#move-list button").first().waitFor();

    // 完成状態へシーク
    await page.locator("#last").click();

    // 色入力タブを開いて「6面の色を入力」をクリック
    await page.locator('[data-tab="colors"]').click();
    await page.locator("#edit-colors").click();
    await page.locator("#editor").waitFor({ state: "visible" });

    // エディタ内のドラフト局面が現在のシーク局面（完成状態）と一致することを検証（center-input との仕様整合）
    const solvedState =
      "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
    const editorNetFirstColor = await page
      .locator('#editor-net .sticker[data-index="0"]')
      .getAttribute("data-color");
    expect(editorNetFirstColor).toBe(solvedState[0]);

    // エディタで適用（編集コミット）
    await page.locator("#editor-apply").click();
    await expect(page.locator("#editor")).not.toBeVisible();

    // Undo を押すと、解法プレビュー前の元のスクランブル局面へ復元できることを検証（commitBaseSnapshotIfPreviewing の保証）
    await page.locator("#undo").click();
    const restoredState = await page.evaluate(() =>
      window.__cube_main_debug__?.store?.getState(),
    );
    expect(restoredState).toBe(scrambleState);
  });

  // F4: 探索中にファイル読み込みイベントが発生しても探索が保護される
  test("F4: 探索中にファイル読み込みイベントが発生しても、探索が中断されず盤面が上書きされない", async ({
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

    // 探索中にファイル入力を発火
    await page.evaluate(async () => {
      const fileInput = document.getElementById("file") as HTMLInputElement;
      const testFileContent = JSON.stringify({
        version: 1,
        state: "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB",
        centerTurns: [0, 0, 0, 0, 0, 0],
      });
      const file = new File([testFileContent], "test.json", {
        type: "application/json",
      });
      const dataTransfer = new DataTransfer();
      dataTransfer.items.add(file);
      fileInput.files = dataTransfer.files;
      fileInput.dispatchEvent(new Event("change"));
    });

    await page.waitForTimeout(200);

    // 探索が継続中であること（キャンセルボタンが表示されたまま、局面がSuperflipのまま）
    const cancelVisible = await page.locator("#cancel").isVisible();
    const currentState = await page.evaluate(() =>
      window.__cube_main_debug__?.store?.getState(),
    );

    expect(cancelVisible).toBe(true);
    expect(currentState).toBe(SUPERFLIP_STATE);

    // 後処理
    await page.locator("#cancel").click();
  });

  // F5: solution-close 押下時に notify('solution') が1回のみ発火する
  test("F5: solution-close 押下時に notify('solution') の二重発火が発生しない", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

    await page.locator('button[data-move="R"]').click();
    await page.locator("#solve").click();
    await page.locator("#move-list button").first().waitFor();

    const notifyCount = await page.evaluate(async () => {
      const store = window.__cube_main_debug__?.store;
      let count = 0;
      const unsubscribe = store?.subscribe((_s: any, detail: any) => {
        if (detail.type === "solution") {
          count++;
        }
      });

      const closeBtn = document.getElementById("solution-close");
      closeBtn?.click();

      unsubscribe?.();
      return count;
    });

    expect(notifyCount).toBe(1);
  });
});
