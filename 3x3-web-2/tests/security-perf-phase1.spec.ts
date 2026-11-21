import { test, expect } from "@playwright/test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import vm from "node:vm";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));
const swSource = fs.readFileSync(path.join(root, "public/sw.js"), "utf-8");

test.describe("Phase 1 Security & Performance Regressions", () => {
  test("S1: production build enforces CSP and referrer policy with zero violations", async ({
    page,
  }) => {
    const pageErrors: string[] = [];
    const cspViolations: string[] = [];
    page.on("pageerror", (err) => pageErrors.push(err.message));

    await page.addInitScript(() => {
      (window as any).__cspViolations = [];
      document.addEventListener("securitypolicyviolation", (e) => {
        (window as any).__cspViolations.push(
          `${e.violatedDirective}: ${e.blockedURI || e.sourceFile || "inline"}`,
        );
      });
    });

    await page.goto("http://127.0.0.1:4173/nested/cube/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const metaInfo = await page.evaluate(() => ({
      csp: document
        .querySelector('meta[http-equiv="Content-Security-Policy"]')
        ?.getAttribute("content"),
      referrer: document
        .querySelector('meta[name="referrer"]')
        ?.getAttribute("content"),
    }));
    expect(metaInfo.csp).toContain("default-src 'self'");
    expect(metaInfo.csp).toContain("script-src 'self' 'wasm-unsafe-eval'");
    expect(metaInfo.csp).toContain("style-src 'self'");
    expect(metaInfo.csp).toContain("object-src 'none'");
    expect(metaInfo.referrer).toBe("no-referrer");

    // プリセット読込・スクランブル・解法探索・色編集・カメラ画面で CSP 違反が出ないことを検証
    await page.locator('button[data-tab="presets"]').click();
    await page.locator("#preset-buttons button").first().click();
    await expect(page.locator("#preset-status")).toContainText(
      "を読み込みました",
    );

    await page.locator('button[data-tab="scramble"]').click();
    await page.locator("#scramble").click();
    await page.locator("#solve").click();
    await expect(page.locator("#solution-content")).toBeVisible();

    await page.getByRole("tab", { name: "色を入力" }).click();
    await page.locator("#edit-colors").click();
    await expect(page.locator("#editor")).toBeVisible();
    await page.locator("#editor-close").click();

    await page.locator("#camera-colors").click();
    await expect(page.locator("#camera-editor")).toBeVisible();
    await page.locator("#camera-close").click();

    const recordedViolations = await page.evaluate(
      () => (window as any).__cspViolations as string[],
    );
    cspViolations.push(...recordedViolations);
    expect(cspViolations).toEqual([]);
    expect(pageErrors).toEqual([]);
  });

  test("S2: production build does not expose debug globals even when webdriver or __TEST_ENV__ is set", async ({
    page,
  }) => {
    await page.addInitScript(() => {
      (window as any).__TEST_ENV__ = true;
    });
    await page.goto("http://127.0.0.1:4173/nested/cube/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    // カメラ画面を開いてからも __lastCamera が公開されないこと
    await page.getByRole("tab", { name: "色を入力" }).click();
    await page.locator("#camera-colors").click();
    await expect(page.locator("#camera-editor")).toBeVisible();
    await page.locator("#camera-close").click();

    const exposed = await page.evaluate(() => ({
      cube_store: typeof (window as any).cube_store,
      cube_scene: typeof (window as any).cube_scene,
      cube_studio: typeof (window as any).cube_studio,
      sound: typeof (window as any).sound,
      __cube_main_debug__: typeof (window as any).__cube_main_debug__,
      __lastCamera: typeof (window as any).__lastCamera,
    }));

    expect(exposed).toEqual({
      cube_store: "undefined",
      cube_scene: "undefined",
      cube_studio: "undefined",
      sound: "undefined",
      __cube_main_debug__: "undefined",
      __lastCamera: "undefined",
    });
  });

  test("S3: hostile 0xFF-filled JPEG is rejected immediately without freezing the main thread", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await expect(page.locator("#engine-status")).toContainText("READY");
    await page.getByRole("tab", { name: "色を入力" }).click();
    await page.locator("#camera-colors").click();
    await expect(page.locator("#camera-editor")).toBeVisible();

    const tmpFile = path.join(os.tmpdir(), "hostile-ff-dos.jpg");
    const buf = Buffer.alloc(1024 * 1024, 0xff);
    buf[0] = 0xff;
    buf[1] = 0xd8;
    fs.writeFileSync(tmpFile, buf);

    try {
      const started = Date.now();
      await page.locator("#camera-file-a").setInputFiles(tmpFile);
      await expect(page.locator("#camera-error")).not.toBeEmpty({
        timeout: 2000,
      });
      const elapsed = Date.now() - started;
      expect(elapsed).toBeLessThan(1500);
    } finally {
      fs.rmSync(tmpFile, { force: true });
    }
  });

  test("S5: Service Worker ignores out-of-scope URLs and rejects 206/opaque runtime caching", async () => {
    let fetchHandler: (event: any) => void;
    const putCalls: string[] = [];
    let mockResponse = new Response("ok", { status: 200 });

    const sandbox = {
      URL,
      Response,
      console: { warn: () => {} },
      self: {
        location: {
          href: "http://example.test/nested/cube/sw.js",
          origin: "http://example.test",
        },
        registration: { scope: "http://example.test/nested/cube/" },
        addEventListener: (type: string, fn: any) => {
          if (type === "fetch") fetchHandler = fn;
        },
      },
      caches: {
        open: async () => ({
          keys: async () => [],
          match: async () => undefined,
          put: async (key: string) => {
            putCalls.push(key);
          },
          delete: async () => true,
        }),
      },
      fetch: async () => mockResponse,
    };
    vm.runInNewContext(swSource, sandbox);

    const dispatch = async (url: string) => {
      let replied: Promise<Response> | undefined;
      const waits: Promise<unknown>[] = [];
      fetchHandler!({
        request: { method: "GET", mode: "cors", url },
        respondWith: (p: Promise<Response>) => {
          replied = p;
        },
        waitUntil: (p: Promise<unknown>) => waits.push(p),
      });
      if (replied) await replied;
      await Promise.all(waits);
      return Boolean(replied);
    };

    // 1. スコープ外の同一オリジン URL は SW が横取りしないこと
    expect(await dispatch("http://example.test/other-app/secret.json")).toBe(
      false,
    );
    expect(putCalls).toEqual([]);

    // 2. 206 Partial Content はキャッシュに保存しないこと
    mockResponse = new Response("part", { status: 206 });
    expect(await dispatch("http://example.test/nested/cube/video.bin")).toBe(
      true,
    );
    expect(putCalls).toEqual([]);

    // 3. opaque レスポンスはキャッシュに保存しないこと
    mockResponse = {
      ok: false,
      status: 0,
      type: "opaque",
      clone() {
        return this;
      },
    } as unknown as Response;
    expect(await dispatch("http://example.test/nested/cube/opaque.bin")).toBe(
      true,
    );
    expect(putCalls).toEqual([]);

    // 4. スコープ内の通常 HTTP 200 はキャッシュ保存されること
    mockResponse = new Response("ok", { status: 200 });
    expect(await dispatch("http://example.test/nested/cube/preset.json")).toBe(
      true,
    );
    expect(putCalls).toEqual(["http://example.test/nested/cube/preset.json"]);
  });

  test("P6: skips hidden #fallback-net DOM rebuilds when WebGL is active and renders on fallback()", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const before = await page.evaluate(() => ({
      hidden: document.getElementById("fallback")?.hidden,
      cells: document.getElementById("fallback-net")?.childElementCount ?? -1,
    }));

    // WebGL が有効な場合、非表示の #fallback-net は構築されない
    if (before.hidden) {
      expect(before.cells).toBe(0);

      await page.locator('[data-move="R"]').click();
      const afterMove = await page.evaluate(
        () => document.getElementById("fallback-net")?.childElementCount ?? -1,
      );
      expect(afterMove).toBe(0);

      // fallback() 呼び出し時に即座に展開図が構築されること
      await page.evaluate(() =>
        (window as any).__cube_main_debug__?.fallback(),
      );
      await expect(page.locator("#fallback")).toBeVisible();
      const afterFallback = await page.evaluate(() => ({
        faces: document.getElementById("fallback-net")?.childElementCount ?? 0,
        stickers: document.querySelectorAll("#fallback-net .sticker").length,
      }));
      expect(afterFallback.faces).toBe(6);
      expect(afterFallback.stickers).toBe(54);
    }
  });
});
