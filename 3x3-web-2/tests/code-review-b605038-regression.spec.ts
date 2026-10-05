import { test, expect } from "@playwright/test";
import { holdSolverResults, releaseSolverResults } from "./solver-barrier";

test.describe("b605038 レビュー指摘点 (F1〜F5) 回帰テスト", () => {
  // F1: 解法プレビューシーク後（または再生完了後）の再探索時の状態復元
  test("F1: 解法プレビューでシーク後に再探索を開始した際、元のスクランブル局面に復元されてから探索が実行される", async ({
    page,
  }) => {
    const scrambleAlg = "B L2 D F2 R2 B2 U L2 D R2 F2 U'";
    await page.goto(`/?no-sw&alg=${encodeURIComponent(scrambleAlg)}`);
    await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

    const originalState = await page.evaluate(() =>
      window.__cube_main_debug__?.store?.getState(),
    );

    // 1回目の解法探索 (Kociemba)
    await page.locator("#solve").click();
    await page.locator("#move-list button").first().waitFor();

    // タイムラインで 5手目へシーク
    await page.evaluate(() => window.__cube_main_debug__?.seek(5, false));
    const stateAtStep5 = await page.evaluate(() =>
      window.__cube_main_debug__?.store?.getState(),
    );
    expect(stateAtStep5).not.toBe(originalState);

    // ソルバーアルゴリズムを CFOP に変更
    await page.locator("#solver-algorithm").selectOption("cfop");

    // 2回目の探索を開始
    await page.locator("#solve").click();
    await page.locator("#solve").waitFor({ state: "visible" });

    const solution2 = await page.evaluate(() =>
      window.__cube_main_debug__?.store?.getSolution(),
    );
    expect(solution2).toBeDefined();
    expect(solution2!.algorithm).toBe("cfop");

    // 検証: 新しい解法の step 0（開始局面）が元のスクランブル局面と一致すること
    const solution2StartState = solution2!.states[0];
    expect(solution2StartState).toBe(originalState);
    expect(solution2StartState).not.toBe(stateAtStep5);

    // さらに、完成局面まで進めてから解法を閉じた際、完成局面が保持され、Undo で元のスクランブル局面に戻れること
    await page.evaluate(
      (len) => window.__cube_main_debug__?.seek(len, false),
      solution2!.moves.length,
    );
    await page.locator("#solution-close").click();
    await page.locator("#solution-content").waitFor({ state: "hidden" });

    // 閉じた後は完成局面が維持されていること（既存仕様 R02 準拠）
    const stateAfterClose = await page.evaluate(() =>
      window.__cube_main_debug__?.store?.getState(),
    );
    expect(stateAfterClose).toBe(
      "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB",
    );

    // Undo を押すとプレビュー前のスクランブル局面に戻ること
    await page.locator("#undo").click();
    const stateAfterUndo = await page.evaluate(() =>
      window.__cube_main_debug__?.store?.getState(),
    );
    expect(stateAfterUndo).toBe(originalState);
  });

  // F2: 探索中の Undo / Redo 無効化および Worker 即時キャンセル
  test("F2: 探索中に Undo / Redo ボタンが無効化され、探索中の不用意な状態変更とゴースト探索が防止される", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

    // プリセットからスーパーフリップを選択
    await page.getByRole("tab", { name: "プリセット" }).click();
    await page.getByRole("button", { name: "スーパーフリップ" }).click();
    await page
      .locator("#preset-status")
      .filter({ hasText: "読み込みました" })
      .waitFor();

    // 1手手動回転して履歴を作る
    await page.locator('button[data-move="R"]').click();
    await page.waitForTimeout(300);

    // ソルバーアルゴリズムを Korf (IDA*) にして探索開始（時間のかかる探索）
    await page.locator("#solver-algorithm").selectOption("korf");
    await page.locator("#solve").click();

    await expect(page.locator("#cancel")).toBeVisible();

    // 探索中は #undo ボタンが disabled になっていること
    const isUndoDisabled = await page.locator("#undo").isDisabled();
    expect(isUndoDisabled).toBe(true);

    // 万が一コード経由で undo が発火しても即座に cancelSearch が走り探索が停止すること
    await page.evaluate(() =>
      (document.querySelector("#undo") as HTMLButtonElement)?.click(),
    );

    // キャンセルボタンが隠れ、探索が即時中断されたこと
    await expect(page.locator("#cancel")).toBeHidden();
    await expect(page.locator("#solve")).toBeVisible();
  });

  // F3: 探索中のエディタ・カメラボタン無効化および非同期レース防止
  test("F3: 探索中の色・カメラ入力は直接ハンドラを呼んでも拒否される", async ({
    page,
  }) => {
    await page.goto("/?no-sw&alg=R");
    await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

    await holdSolverResults(page);
    await page.locator("#solver-algorithm").selectOption("korf");
    await page.locator("#solve").click();
    await expect(page.locator("#cancel")).toBeVisible();

    // 「色を入力」タブへ移動
    await page.getByRole("tab", { name: "色を入力" }).click();

    // 探索中は edit-colors と camera-colors が disabled になっていること
    expect(await page.locator("#edit-colors").isDisabled()).toBe(true);
    expect(await page.locator("#camera-colors").isDisabled()).toBe(true);

    for (const id of ["edit-colors", "camera-colors"]) {
      await page
        .locator(`#${id}`)
        .evaluate((button: HTMLButtonElement) =>
          button.onclick?.call(button, new MouseEvent("click")),
        );
      await expect(page.locator("#message")).toContainText("探索中");
      await expect(page.locator("#editor")).toBeHidden();
      await expect(page.locator("#camera-editor")).toBeHidden();
      await expect(page.locator("#cancel")).toBeVisible();
    }
    await releaseSolverResults(page);
    await expect(page.locator("#solution-content")).toBeVisible();
  });

  // F4: 解法自動再生中に Undo を呼んだ際、playing フラグが確実に false にリセットされる
  test("F4: 解法再生中に Undo を呼んでも stop() が走り、playing フラグが孤立せず次回再生が正常に開始される", async ({
    page,
  }) => {
    await page.goto("/?no-sw&alg=R+U+R'+U'+R");
    await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

    await page.locator("#solve").click();
    await page.locator("#solution-content").waitFor({ state: "visible" });

    // 自動再生を開始
    await page.locator("#play").click();
    await page.waitForTimeout(100);

    // 再生中に #undo をクリック
    await page.locator("#undo").click();
    await page.waitForTimeout(400);

    // 再度 solve して解法を取得
    await page.locator("#solve").click();
    await page.locator("#solution-content").waitFor({ state: "visible" });

    // play ボタンのラベルが「自動再生」になっており、クリックで即座に再生が開始されること
    await expect(page.locator("#play")).toHaveAttribute(
      "aria-label",
      "自動再生",
    );
    await page.locator("#play").click();
    await expect(page.locator("#play")).toHaveAttribute(
      "aria-label",
      "一時停止",
    );

    // 停止
    await page.locator("#play").click();
  });

  // F5: 解法クローズボタン押下時のキーボードフォーカス保持
  test("F5: 解法表示中に「解法を閉じる」を押すとフォーカスが #solve ボタンに安全に移動する", async ({
    page,
  }) => {
    await page.goto("/?no-sw&alg=R+U+R'+U'");
    await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

    await page.locator("#solve").click();
    await page.locator("#solution-close").waitFor({ state: "visible" });

    // #solution-close にフォーカスを当てて Enter
    await page.locator("#solution-close").focus();
    await page.keyboard.press("Enter");

    await page.locator("#solution-close").waitFor({ state: "hidden" });

    // クリック後の activeElement が #solve であること
    const activeElementId = await page.evaluate(
      () => document.activeElement?.id,
    );
    expect(activeElementId).toBe("solve");
  });
});
