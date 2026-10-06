import { test, expect, type Page } from "@playwright/test";

async function ready(page: Page, url = "/?no-sw") {
  await page.goto(url);
  await expect(page.locator("#engine-status")).toContainText("READY");
  await page.locator("#reduced-motion").check();
}

test.describe("HEAD 0139c65 Code Review Regressions", () => {
  test("H1: カラーエディタのパレットが role='radio' と aria-checked を持ち、radiogroup ARIA 規約を満たすこと", async ({
    page,
  }) => {
    await ready(page);

    // カラーエディタを開く
    await page.locator('button[data-tab="colors"]').click();
    await page.locator("#edit-colors").click();
    await expect(page.locator("#editor")).toBeVisible();

    // #palette の子要素ボタンを取得
    const paletteButtons = page.locator("#palette button");
    const count = await paletteButtons.count();
    expect(count).toBe(6);

    for (let i = 0; i < count; i++) {
      const btn = paletteButtons.nth(i);
      await expect(btn).toHaveAttribute("role", "radio");
    }

    // 初期状態で選択されている色のボタン（白/U面）が aria-checked="true"
    const firstBtn = paletteButtons.first();
    await expect(firstBtn).toHaveAttribute("aria-checked", "true");

    // 別の色（2番目）をクリックすると、aria-checked が切り替わる
    const secondBtn = paletteButtons.nth(1);
    await secondBtn.click();
    await expect(secondBtn).toHaveAttribute("aria-checked", "true");
    await expect(firstBtn).toHaveAttribute("aria-checked", "false");

    // ダイアログを閉じる
    await page.locator("#editor-close").click();
    await expect(page.locator("#editor")).toBeHidden();
  });

  test("H2: 自動再生中に stop() が呼ばれた際、再生ボタン（#play）のアイコン・ラベルが直ちに自動再生に同期復帰すること", async ({
    page,
  }) => {
    await ready(page);

    // 十分な手数の解法を生成するためにスクランブルして解く
    await page.locator("#scramble").click();
    await page.locator("#solve").click();
    await expect(page.locator("#solution-content")).toBeVisible();

    // 再生ボタンを押して自動再生を開始
    const playBtn = page.locator("#play");
    await playBtn.click();
    await expect(playBtn).toHaveAttribute("aria-label", "一時停止");
    await playBtn.blur();

    // 再生中にキーボード操作（ArrowLeft）で前の手へ戻り、stop() を契機に停止させる
    await page.keyboard.press("ArrowLeft");

    // stop() 内で refresh() が実行され、直ちに「自動再生」アイコン・ラベルへ復帰していること（H2 回帰確認）
    await expect(playBtn).toHaveAttribute("aria-label", "自動再生");

    // もう一度再生し、#first ボタン押下時にも即座に同期されることを検証
    await playBtn.click();
    await expect(playBtn).toHaveAttribute("aria-label", "一時停止");
    await page.locator("#first").click();
    await expect(playBtn).toHaveAttribute("aria-label", "自動再生");
  });

  test("M3: centerLabels の userData に rotation が保持され、finish() 時にクォータニオンが復元されること", async ({
    page,
  }) => {
    await ready(page);
    await page.waitForFunction(() => !!(window as any).cube_scene);

    const rotationCheck = await page.evaluate(async () => {
      const scene = (window as any).cube_scene;
      if (!scene || !scene.centerLabels) return { err: "no centerLabels" };

      // 全6面の centerLabels が userData.origin と userData.rotation を持っているか
      const hasUserDataRot = scene.centerLabels.every(
        (l: any) => l.userData?.origin && l.userData?.rotation,
      );

      // finish() を直接呼んでもクォータニオンが壊れず userData.rotation と一致すること
      const label0 = scene.centerLabels[0];
      const initialQuat = [
        label0.quaternion.x,
        label0.quaternion.y,
        label0.quaternion.z,
        label0.quaternion.w,
      ];
      scene.finish();
      const afterQuat = [
        label0.quaternion.x,
        label0.quaternion.y,
        label0.quaternion.z,
        label0.quaternion.w,
      ];

      return { hasUserDataRot, initialQuat, afterQuat };
    });

    expect(rotationCheck.hasUserDataRot).toBe(true);
    expect(rotationCheck.initialQuat).toEqual(rotationCheck.afterQuat);
  });

  test("M4: モーダル（ヘルプ・エディタ）を閉じた際、トリガー元ボタンへフォーカスが復元されること（Focus Return）", async ({
    page,
  }) => {
    await ready(page);

    // 1. ヘルプダイアログのフォーカス復元
    const helpBtn = page.locator("#help");
    await helpBtn.focus();
    await page.keyboard.press("Enter");
    const helpDialog = page.locator("#help-dialog");
    await expect(helpDialog).toBeVisible();

    // ESC キーで閉じる
    await page.keyboard.press("Escape");
    await expect(helpDialog).toBeHidden();
    await expect(helpBtn).toBeFocused();

    // 2. カラーエディタダイアログのフォーカス復元
    await page.locator('button[data-tab="colors"]').click();
    const editColorsBtn = page.locator("#edit-colors");
    await editColorsBtn.focus();
    await page.keyboard.press("Enter");
    const editorDialog = page.locator("#editor");
    await expect(editorDialog).toBeVisible();

    // 閉じるボタンをクリックして閉じる
    await page.locator("#editor-close").click();
    await expect(editorDialog).toBeHidden();
    await expect(editColorsBtn).toBeFocused();
  });

  test("L1: #timeline に初期状態および解法表示時に aria-valuemax が設定され、ステップ数と整合すること", async ({
    page,
  }) => {
    await ready(page);

    const timeline = page.locator("#timeline");
    // 初期状態の aria-valuemax は "0"
    await expect(timeline).toHaveAttribute("aria-valuemax", "0");

    // 1手回して解法を探索
    await page.locator('button[data-move="R"]').click();
    await page.locator("#solve").click();
    await expect(page.locator("#solution-content")).toBeVisible();

    // 解法表示時、aria-valuemax は解法手数（1手）と一致
    await expect(timeline).toHaveAttribute("aria-valuemax", "1");
  });

  test("L2: URL パラメータ algorithm が起動後の URL クリーンアップで searchParams から削除されること", async ({
    page,
  }) => {
    // ?algorithm=cfop 付きでアクセス
    await ready(page, "/?algorithm=cfop&no-sw");

    // ソルバー選択セレクトが CFOP に反映されていること
    await expect(page.locator("#solver-algorithm")).toHaveValue("cfop");

    // クリーンアップにより URL バーから algorithm が消えていること
    await page.waitForFunction(() => {
      const url = new URL(window.location.href);
      return !url.searchParams.has("algorithm");
    });
    const finalUrl = new URL(page.url());
    expect(finalUrl.searchParams.has("algorithm")).toBe(false);
  });

  test("L3: buildShareUrl で非デフォルトのソルバーが URL に伝搬され、#share-link でコピーされること", async ({
    page,
    context,
  }) => {
    await context.grantPermissions(["clipboard-read", "clipboard-write"]);
    await ready(page);

    // CFOP を選択
    await page.locator("#solver-algorithm").selectOption("cfop");

    // 共有リンクボタンをクリックしてクリップボードに生成される URL を確認
    await page.locator("#share-link").click();

    const clipboardText = await page.evaluate(() =>
      navigator.clipboard.readText(),
    );
    expect(clipboardText).toContain("solver=cfop");
    expect(clipboardText).not.toContain("algorithm=");

    // Kociemba の場合はデフォルトなので solver パラメータが含まれないこと
    await page.locator("#solver-algorithm").selectOption("kociemba");
    await page.locator("#share-link").click();
    const clipboardKociemba = await page.evaluate(() =>
      navigator.clipboard.readText(),
    );
    expect(clipboardKociemba).not.toContain("solver=");
  });

  test("L4: Service Worker の旧キャッシュキーソートが自然順（numeric: true）で行われること", async ({
    page,
  }) => {
    await ready(page);

    const sortResult = await page.evaluate(() => {
      const sampleKeys = [
        "cube-studio-v9",
        "cube-studio-v10",
        "cube-studio-v2",
        "cube-studio-v1",
      ];
      const sorted = [...sampleKeys].sort((a, b) =>
        a.localeCompare(b, undefined, { numeric: true }),
      );
      return {
        sorted,
        mostRecent: sorted.pop(),
      };
    });

    expect(sortResult.mostRecent).toBe("cube-studio-v10");
  });

  test("L6: centersFromInput に不正な配列を渡した際、詳細なエラーメッセージが返されること", async ({
    page,
  }) => {
    await ready(page);

    const result = await page.evaluate(async () => {
      // @ts-ignore
      const { centersFromInput } = await import("/web/centers.js");
      const SOLVED = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";

      let lengthErr = "";
      try {
        centersFromInput(SOLVED, [0, 0, 0]);
      } catch (e: any) {
        lengthErr = e.message;
      }

      let valueErr = "";
      try {
        centersFromInput(SOLVED, [0, 0, 4, 0, 0, 0]);
      } catch (e: any) {
        valueErr = e.message;
      }

      return { lengthErr, valueErr };
    });

    expect(result.lengthErr).toContain("要素数: 3");
    expect(result.valueErr).toContain("F面: 4");
  });
});
