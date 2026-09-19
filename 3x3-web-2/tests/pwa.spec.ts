import { test, expect } from "@playwright/test";

test.describe("PWA and Offline Support", () => {
  test("manifest.webmanifest and PWA meta tags are present and valid", async ({
    page,
  }) => {
    await page.goto("/");

    // 1. マニフェストリンクの検証
    const manifestLink = page.locator('link[rel="manifest"]');
    await expect(manifestLink).toHaveCount(1);
    const href = await manifestLink.getAttribute("href");
    expect(href).toBeTruthy();

    // 2. マニフェスト内容の取得と構造検証
    const manifestResponse = await page.request.get(href!);
    expect(manifestResponse.ok()).toBe(true);
    const manifest = await manifestResponse.json();

    expect(manifest.name).toBe("CUBE STUDIO");
    expect(manifest.short_name).toBe("Cube Studio");
    expect(manifest.display).toBe("standalone");
    expect(manifest.start_url).toBeTruthy();
    expect(manifest.background_color).toBe("#141716");
    expect(manifest.theme_color).toBe("#141716");
    expect(Array.isArray(manifest.icons)).toBe(true);
    expect(manifest.icons.length).toBeGreaterThan(0);

    // 3. Apple Touch Icon & メタタグの検証
    const appleTouchIcon = page.locator('link[rel="apple-touch-icon"]');
    await expect(appleTouchIcon).toHaveCount(1);
    const themeColor = page.locator('meta[name="theme-color"]');
    await expect(themeColor).toHaveAttribute("content", "#141716");
  });

  test("service worker registers successfully", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const isRegistered = await page.evaluate(async () => {
      if (!("serviceWorker" in navigator)) return false;
      const registration = await navigator.serviceWorker.getRegistration();
      return !!registration;
    });

    expect(isRegistered).toBe(true);
  });

  test("app loads and solves cube while completely offline", async ({
    page,
    context,
  }) => {
    // 1. 初回オンラインアクセスで Service Worker の準備を待機
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    // Service Worker がコントローラーになるのを確認
    await page.evaluate(async () => {
      await navigator.serviceWorker.ready;
      if (!navigator.serviceWorker.controller) {
        await new Promise((resolve) => {
          navigator.serviceWorker.addEventListener(
            "controllerchange",
            resolve,
            {
              once: true,
            },
          );
        });
      }
    });

    // 2. SW コントロール下で一度ロードし、全アセット（WASM、Worker 等）をキャッシュへ格納
    await page.reload();
    await expect(page.locator("#engine-status")).toContainText("READY");

    // 3. ネットワークを切断（完全オフライン状態）
    await context.setOffline(true);

    try {
      // 4. オフライン状態でリロード
      await page.reload();

      // 5. オフラインでもエンジンが READY になり、UI が正常起動
      await expect(page.locator("#engine-status")).toContainText("READY", {
        timeout: 15000,
      });
      await expect(page.locator("#scene")).toHaveAttribute(
        "data-state",
        "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB",
      );

      // 6. オフライン状態でスクランブルと解法探索が動作することを検証
      await page.locator("#reduced-motion").check();
      await page.locator('[data-move="R"]').click();
      await page.locator("#solve").click();
      await expect(page.locator("#solution-content")).toBeVisible({
        timeout: 10000,
      });

      const moveCount = await page.locator(".solution-move").count();
      expect(moveCount).toBeGreaterThan(0);
    } finally {
      await context.setOffline(false);
    }
  });

  test("R02: does not delete caches belonging to other apps on the same origin", async ({
    page,
    context,
  }) => {
    // 既存の SW とキャッシュをクリーンアップ
    await page.goto("/");
    await page.evaluate(async () => {
      const regs = await navigator.serviceWorker.getRegistrations();
      for (const r of regs) await r.unregister();
      const keys = await caches.keys();
      for (const k of keys) await caches.delete(k);
    });

    // 他アプリのキャッシュを作成
    await page.evaluate(async () => {
      const otherCache = await caches.open("another-app-v1");
      await otherCache.put("/dummy", new Response("dummy content"));
    });

    // ページを再読み込みして Service Worker を登録・有効化させる
    await page.reload();
    await expect(page.locator("#engine-status")).toContainText("READY");

    await page.evaluate(async () => {
      await navigator.serviceWorker.ready;
      // activate が完了するのを少し待つ
      await new Promise((resolve) => setTimeout(resolve, 500));
    });

    // 他アプリのキャッシュが残っていることを検証
    const hasOtherAppCache = await page.evaluate(async () => {
      return await caches.has("another-app-v1");
    });
    expect(hasOtherAppCache).toBe(true);
  });
});
