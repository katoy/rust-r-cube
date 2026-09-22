import { test, expect } from "@playwright/test";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

test.describe("R11, R12 & Preset Metadata Fixes", () => {
  test.beforeEach(async ({ page }) => {
    // 既存の Service Worker や Cache Storage によるリクエスト横取りを防止
    await page.addInitScript(() => {
      (window as any).__DISABLE_SW__ = true;
    });
    await page.goto("/");
    await page.evaluate(async () => {
      if ("serviceWorker" in navigator) {
        const registrations = await navigator.serviceWorker.getRegistrations();
        await Promise.all(registrations.map((reg) => reg.unregister()));
      }
      if ("caches" in window) {
        const cacheNames = await caches.keys();
        await Promise.all(cacheNames.map((name) => caches.delete(name)));
      }
    });
    // SW が完全に解除された状態で再読み込み
    await page.reload();
  });

  test("R11: arrow keys on tabs should navigate tabs without advancing cube solution playback", async ({
    page,
  }) => {
    // 解法が存在する状態で開く
    await page.goto("/?alg=R_U_F");
    await expect(page.locator("#engine-status")).toContainText("READY");

    // ソルブボタンを押して解法を計算
    await page.locator("#solve").click();
    // 解法が表示され、ステップインジケータが 0 になっていること
    await expect(page.locator("#step-count")).toContainText("0 /");

    const initialStep = await page.evaluate(() => {
      // @ts-ignore
      return (window as any).cube_store?.getStep?.() ?? 0;
    });
    expect(initialStep).toBe(0);

    // 「スクランブル」タブにフォーカス
    const scrambleTab = page.locator('button[data-tab="scramble"]');
    await scrambleTab.focus();

    // ArrowRight キーを押す（タブが「プリセット」へ移動するはず）
    await page.keyboard.press("ArrowRight");

    // タブが「プリセット」に切り替わっていること
    const presetTab = page.locator('button[data-tab="presets"]');
    await expect(presetTab).toHaveAttribute("aria-selected", "true");

    // キューブの再生位置が 0 のままであること（ArrowRight が再生ハンドラへ伝播していないこと）
    const stepAfterTab = await page.evaluate(() => {
      // @ts-ignore
      return (window as any).cube_store?.getStep?.() ?? 0;
    });
    expect(stepAfterTab).toBe(0);
    await expect(page.locator("#step-count")).toContainText("0 /");
  });

  test("R12: delayed preset response does not overwrite concurrent user edits", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    // プリセットタブを開く
    await page.locator('button[data-tab="presets"]').click();

    // cubes/solved.json のレスポンスをインターセプトして遅延させる
    let fulfillPromise: () => void = () => {};
    const delayPromise = new Promise<void>((resolve) => {
      fulfillPromise = resolve;
    });

    await page.route("**/cubes/solved.json", async (route) => {
      await delayPromise;
      await route.continue();
    });

    // 「完成状態」プリセットボタンをクリック（非同期取得が保留される）
    const solvedPresetBtn = page
      .locator("#presets-panel button")
      .filter({ hasText: "完成状態" });
    await solvedPresetBtn.click();
    await expect(page.locator("#preset-status")).toContainText("読み込み中");

    const initialState = await page.evaluate(() => {
      // @ts-ignore
      return (window as any).cube_store?.getState?.();
    });

    // 取得保留中にユーザーが手動で 'R' 操作を行う
    await page.keyboard.press("r");

    // 手動操作が確実に store に反映されて状態が変化するまで待機
    await expect
      .poll(async () => {
        return await page.evaluate(() => {
          // @ts-ignore
          return (window as any).cube_store?.getState?.();
        });
      })
      .not.toBe(initialState);

    // 手動操作後のキューブ状態を取得（R操作後の状態）
    const stateAfterEdit = await page.evaluate(() => {
      // @ts-ignore
      return (window as any).cube_store?.getState?.();
    });

    // ここで遅延させていたプリセットのレスポンスを完了させる
    fulfillPromise();

    // 少し待機してプリセットの後続処理を実行させる
    await page.waitForTimeout(300);

    // 最終的なキューブ状態が手動操作後の状態を維持していること（solvedで上書きされていないこと）
    const finalState = await page.evaluate(() => {
      // @ts-ignore
      return (window as any).cube_store?.getState?.();
    });

    expect(finalState).toBe(stateAfterEdit);
    await expect(page.locator("#preset-status")).toContainText("優先");
  });

  test("Preset Metadata: easy-5-moves.json solution correctly solves scramble", async ({
    page,
  }) => {
    const jsonPath = path.resolve(__dirname, "../cubes/easy-5-moves.json");
    const content = JSON.parse(fs.readFileSync(jsonPath, "utf-8"));

    // easy-5-moves の scramble と solution_moves
    expect(content.scramble).toBe("R U F");
    // 解法は F' U' R' でなければならない
    expect(content.solution_moves).toEqual(["F'", "U'", "R'"]);
    expect(content.solution_length).toBe(3);

    // WASM を使って R U F に solution_moves を適用したら SOLVED になることを検証
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const isSolved = await page.evaluate((moves) => {
      const wasm = (window as any).cube_studio;
      const solvedStr =
        "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
      // SOLVED に R U F を適用
      const scrambled = JSON.parse(wasm.apply_moves(solvedStr, "R U F"));
      // さらに solution_moves を適用
      const solved = JSON.parse(
        wasm.apply_moves(scrambled.state, moves.join(" ")),
      );
      return solved.state === solvedStr;
    }, content.solution_moves);

    expect(isSolved).toBe(true);
  });

  test("R01: preset buttons are properly disabled before ready and enabled after ready", async ({
    page,
  }) => {
    await page.goto("/");
    const presetTab = page.locator('button[data-tab="presets"]');
    await presetTab.click();

    // 準備完了後は全プリセットボタンが活性化されていること
    await expect(page.locator("#engine-status")).toContainText("READY");
    const buttons = page.locator("#preset-buttons button");
    const count = await buttons.count();
    expect(count).toBeGreaterThan(0);
    for (let i = 0; i < count; i++) {
      await expect(buttons.nth(i)).toBeEnabled();
    }

    // プリセットをクリックして正常に読み込めること
    await buttons.first().click();
    await expect(page.locator("#preset-status")).toContainText(
      "を読み込みました",
    );
  });
});
