import { test, expect } from "@playwright/test";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

test.describe("R03 & R15: Service Worker Cache Scope and waitUntil", () => {
  test("R03: activate on parent scope /nested/ does not delete /nested/cube/ cache", async ({
    page,
  }) => {
    const swPath = path.resolve(__dirname, "../public/sw.js");
    const swCode = fs.readFileSync(swPath, "utf-8");

    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const result = await page.evaluate(async (code) => {
      // Service Worker 環境をモック
      const cacheStore = new Map<string, Map<string, any>>();
      const mockCaches = {
        open: async (name: string) => {
          if (!cacheStore.has(name)) cacheStore.set(name, new Map());
          const c = cacheStore.get(name)!;
          return {
            put: async (req: any, res: any) => c.set(req.url || req, res),
            match: async (req: any) => c.get(req.url || req),
            addAll: async () => {},
          };
        },
        keys: async () => Array.from(cacheStore.keys()),
        delete: async (name: string) => cacheStore.delete(name),
      };

      // 1. 子スコープ /nested/cube/ のキャッシュをあらかじめ作成
      // 子スコープの SW を評価してそのキャッシュ名を算出
      const childSelf = {
        registration: { scope: "http://example.com/nested/cube/" },
        location: { href: "http://example.com/nested/cube/sw.js" },
        addEventListener: () => {},
        skipWaiting: async () => {},
        clients: { claim: async () => {} },
      };
      const getChildCache = new Function(
        "self",
        "caches",
        `
        ${code}
        return CACHE_NAME;
      `,
      );
      const childCacheName = getChildCache(childSelf, mockCaches);
      await mockCaches.open(childCacheName);

      // 2. 親スコープ /nested/ の SW を評価
      let activateHandler: ((event: any) => void) | null = null;
      const parentSelf: any = {
        registration: { scope: "http://example.com/nested/" },
        location: { href: "http://example.com/nested/sw.js" },
        clients: { claim: async () => {} },
        skipWaiting: async () => {},
        addEventListener: (event: string, fn: any) => {
          if (event === "activate") activateHandler = fn;
        },
      };

      const runParent = new Function(
        "self",
        "caches",
        `
        ${code}
      `,
      );
      runParent(parentSelf, mockCaches);

      // 親スコープの activate イベントを実行
      let waitUntilPromise: Promise<any> | null = null;
      activateHandler!({
        waitUntil: (p: Promise<any>) => {
          waitUntilPromise = p;
        },
      });

      if (waitUntilPromise) {
        await waitUntilPromise;
      }

      const keysAfterActivate = await mockCaches.keys();
      return {
        childCacheName,
        keysAfterActivate,
        hasChildCache: keysAfterActivate.includes(childCacheName),
      };
    }, swCode);

    // 親スコープ /nested/ の activate 後も、子スコープ /nested/cube/ のキャッシュが削除されずに残っていること
    expect(result.hasChildCache).toBe(true);
  });

  test("R03: distinct scopes /a/b/c_/d/ and /a/b/c/_d/ do not generate identical cache prefixes", async ({
    page,
  }) => {
    const swPath = path.resolve(__dirname, "../public/sw.js");
    const swCode = fs.readFileSync(swPath, "utf-8");

    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const prefixes = await page.evaluate((code) => {
      const getPrefix = (scopeUrl: string) => {
        const mockSelf = {
          registration: { scope: scopeUrl },
          location: { href: `${scopeUrl}sw.js` },
          addEventListener: () => {},
          skipWaiting: async () => {},
          clients: { claim: async () => {} },
        };
        const fn = new Function(
          "self",
          "caches",
          `
          ${code}
          return CACHE_PREFIX;
        `,
        );
        return fn(mockSelf, { open: async () => ({}) });
      };

      const p1 = getPrefix("http://example.com/a/b/c_/d/");
      const p2 = getPrefix("http://example.com/a/b/c/_d/");
      return { p1, p2 };
    }, swCode);

    expect(prefixes.p1).not.toBe(prefixes.p2);
  });

  test("R15: stale-while-revalidate and navigation pass cache update promises to event.waitUntil", async ({
    page,
  }) => {
    const swPath = path.resolve(__dirname, "../public/sw.js");
    const swCode = fs.readFileSync(swPath, "utf-8");

    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const result = await page.evaluate(async (code) => {
      const waitUntils: Promise<any>[] = [];
      let fetchHandler: ((event: any) => void) | null = null;

      const mockCache = {
        match: async (req: any) => new Response("cached-body"),
        put: async () => {},
      };
      const mockCaches = {
        open: async () => mockCache,
      };

      const mockSelf: any = {
        registration: { scope: "http://example.com/" },
        location: { href: "http://example.com/sw.js" },
        addEventListener: (event: string, fn: any) => {
          if (event === "fetch") fetchHandler = fn;
        },
      };

      const run = new Function("self", "caches", code);
      run(mockSelf, mockCaches);

      // 1. 静的アセットのキャッシュヒット時のリクエスト
      let respondedWithPromise: Promise<any> | null = null;
      const staticEvent = {
        request: new Request("http://example.com/app.js"),
        respondWith: (p: Promise<any>) => {
          respondedWithPromise = p;
        },
        waitUntil: (p: Promise<any>) => {
          waitUntils.push(p);
        },
      };

      fetchHandler!(staticEvent);
      if (respondedWithPromise) {
        await respondedWithPromise;
      }

      return {
        waitUntilCount: waitUntils.length,
      };
    }, swCode);

    // キャッシュヒット時でも、バックグラウンド更新 fetch/cache.put が waitUntil に渡されていること
    expect(result.waitUntilCount).toBeGreaterThan(0);
  });
});
