import { test, expect } from "@playwright/test";
import { getTestImagePath } from "./test-utils";

test.describe("0bd0d8c Code Review Regression Tests (R01 - R04)", () => {
  test("R01: SW suppression or ?no-sw must not unregister unrelated SW registrations under other scopes", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await expect(page.locator("#engine-status")).toContainText("READY");

    // 他アプリのスコープ (/other-app/) に SW を登録
    const before = await page.evaluate(async () => {
      const reg = await navigator.serviceWorker.register("/sw.js", {
        scope: "/other-app/",
      });
      const worker = reg.installing || reg.waiting || reg.active;
      if (worker && worker.state !== "activated") {
        await new Promise<void>((resolve, reject) => {
          const timer = setTimeout(
            () => reject(new Error("SW activation timeout")),
            10000,
          );
          worker.addEventListener("statechange", () => {
            if (worker.state === "activated") {
              clearTimeout(timer);
              resolve();
            }
          });
        });
      }
      return (await navigator.serviceWorker.getRegistrations()).map(
        (r) => r.scope,
      );
    });
    expect(before.some((s) => s.includes("/other-app/"))).toBe(true);

    // ページを再読み込み (?no-sw)
    await page.reload();
    await expect(page.locator("#engine-status")).toContainText("READY");
    await page.waitForTimeout(500);

    // 別スコープの SW 登録が維持されていること
    const after = await page.evaluate(async () => {
      return (await navigator.serviceWorker.getRegistrations()).map(
        (r) => r.scope,
      );
    });
    expect(after.some((s) => s.includes("/other-app/"))).toBe(true);
  });

  test("R02: solution seek, finish, and close must persist state to localStorage and restore on reload", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await expect(page.locator("#engine-status")).toContainText("READY");
    await page.locator("#reduced-motion").check();

    // 1. 手順 R U F を適用
    await page.getByRole("tab", { name: "手順を入力" }).click();
    await page.locator("#algorithm").fill("R U F");
    await page.locator("#apply-algorithm").click();

    const scrambled = await page.evaluate(() =>
      window.cube_store.getSnapshot(),
    );
    expect(scrambled.state).not.toBe(
      "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB",
    );

    // 2. 解法を求めて最後の手順へジャンプ
    await page.locator("#solve").click();
    await page.waitForFunction(() => !!window.cube_store.getSolution());
    await page.locator("#last").click();

    // 3. 解法を閉じる
    await page.locator("#solution-close").click();

    // 4. localStorage に保存されている状態を確認
    const saved = await page.evaluate(() => {
      const raw = localStorage.getItem("cube-studio-v1");
      return raw ? JSON.parse(raw) : null;
    });

    expect(saved).not.toBeNull();
    expect(saved.state).toBe(
      "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB",
    );
    expect(saved.centerTurns).toEqual([0, 0, 0, 0, 0, 0]);

    // 5. ページを再読み込みして復元されることを確認
    await page.reload();
    await expect(page.locator("#engine-status")).toContainText("READY");
    const restored = await page.evaluate(() => window.cube_store.getSnapshot());
    expect(restored.state).toBe(
      "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB",
    );
    expect(restored.centerTurns).toEqual([0, 0, 0, 0, 0, 0]);
  });

  test("R03: delayed camera capture frame must not overwrite newly selected file", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await expect(page.locator("#engine-status")).toContainText("READY");

    await page.getByRole("tab", { name: "色を入力", exact: true }).click();
    await page.locator("#camera-colors").click();
    await page.locator("#camera-editor").waitFor({ state: "visible" });

    await page.locator("#camera-live-stream").click();
    await page.waitForFunction(() => (window as any).__lastCamera?.isStreaming);

    // toBlob コールバックを保留できるようにフック
    await page.evaluate(() => {
      const original = HTMLCanvasElement.prototype.toBlob;
      HTMLCanvasElement.prototype.toBlob = function (callback, ...args) {
        return original.call(
          this,
          (blob) => {
            (window as any).releaseCapturedFrame = () => callback(blob);
          },
          ...args,
        );
      };
    });

    // 撮影を実行（toBlob は保留状態）
    await page.locator("#camera-take-photo").click();
    await page.waitForFunction(() => !!(window as any).releaseCapturedFrame);

    // 後から 640x480 の画像ファイルを選択
    const testImagePath = getTestImagePath("solved", "A");
    await page.locator("#camera-file-a").setInputFiles(testImagePath);
    await page.waitForFunction(
      () => (window as any).__lastCamera?.imageA?.naturalWidth === 640,
    );
    const afterNewFile = await page.locator("#camera-status-a").textContent();
    expect(afterNewFile).toContain("640×480");

    // 古い撮影フレームのコールバックを完了させる
    await page.evaluate(() => (window as any).releaseCapturedFrame());
    await page.waitForTimeout(300);

    // 新しいファイル選択結果が保持されていること（古い撮影で上書きされない）
    const currentWidth = await page.evaluate(
      () => (window as any).__lastCamera?.imageA?.naturalWidth,
    );
    expect(currentWidth).toBe(640);
    const afterOldCapture = await page
      .locator("#camera-status-a")
      .textContent();
    expect(afterOldCapture).toContain("640×480");
  });

  test("R04: sticker color correction retains focus and Tab moves to next cell", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await expect(page.locator("#engine-status")).toContainText("READY");

    await page.getByRole("tab", { name: "色を入力", exact: true }).click();
    await page.locator("#camera-colors").click();
    await page.locator("#camera-editor").waitFor({ state: "visible" });

    const testImagePath = getTestImagePath("solved", "A");
    await page.locator("#camera-file-a").setInputFiles(testImagePath);
    await page.waitForFunction(
      () => (window as any).__lastCamera?.points?.length === 6,
    );
    await page.locator("#camera-capture").click();

    // F面の右上セル (index=2) にフォーカス
    const cell = page.locator("#camera-face-card-F button[data-index='2']");
    await cell.focus();

    const before = await page.evaluate(() => ({
      tag: document.activeElement?.tagName,
      face: (document.activeElement as HTMLElement)?.dataset?.face,
      index: (document.activeElement as HTMLElement)?.dataset?.index,
    }));
    expect(before).toEqual({ tag: "BUTTON", face: "F", index: "2" });

    // Spaceキーで色を修正
    await page.keyboard.press("Space");

    // フォーカスが BODY に失われず、同じセルに維持されること
    const afterSpace = await page.evaluate(() => ({
      tag: document.activeElement?.tagName,
      face: (document.activeElement as HTMLElement)?.dataset?.face,
      index: (document.activeElement as HTMLElement)?.dataset?.index,
    }));
    expect(afterSpace).toEqual({ tag: "BUTTON", face: "F", index: "2" });

    // 次の Tab で次の編集可能セル (F[3]) へ進むこと (U[0] に戻らない)
    await page.keyboard.press("Tab");
    const nextTab = await page.evaluate(() => ({
      tag: document.activeElement?.tagName,
      face: (document.activeElement as HTMLElement)?.dataset?.face,
      index: (document.activeElement as HTMLElement)?.dataset?.index,
    }));
    expect(nextTab).toEqual({ tag: "BUTTON", face: "F", index: "3" });
  });
});
