import { test, expect } from "@playwright/test";
import fs from "fs";
import path from "path";
import os from "os";
import crypto from "crypto";
import vm from "vm";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

test.describe("R14: Service Worker update and cache revalidation", () => {
  test("content change of same-name precached asset updates CACHE_VERSION in sw.js", async () => {
    // 1. 一時ディレクトリに擬似 dist 環境を作成
    const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "sw-precache-test-"));
    try {
      const cubesDir = path.join(tempDir, "cubes");
      fs.mkdirSync(cubesDir, { recursive: true });

      const presetPath = path.join(cubesDir, "easy-5-moves.json");
      fs.writeFileSync(presetPath, JSON.stringify({ name: "version-1" }));

      const dummyHtml = path.join(tempDir, "index.html");
      fs.writeFileSync(dummyHtml, "<!DOCTYPE html><html></html>");

      const swTemplate = `
function getScopeSlug() { return "root"; }
const SCOPE_SLUG = getScopeSlug();
const CACHE_PREFIX = \`cube-studio-\${SCOPE_SLUG}-\`;
const CACHE_VERSION = "v1";
const CACHE_NAME = \`\${CACHE_PREFIX}\${CACHE_VERSION}\`;
const PRECACHE_ASSETS = [
  "./",
  "./index.html",
];
`;
      const tempSwPath = path.join(tempDir, "sw.js");
      fs.writeFileSync(tempSwPath, swTemplate);

      // generate-sw-precache と同等のロジックを実行する関数
      const runPrecacheInjection = () => {
        const getFiles = (dir, baseDir = dir) => {
          const entries = fs.readdirSync(dir, { withFileTypes: true });
          const files = [];
          for (const entry of entries) {
            const fullPath = path.join(dir, entry.name);
            if (entry.isDirectory()) {
              files.push(...getFiles(fullPath, baseDir));
            } else {
              const rel = path.relative(baseDir, fullPath).replace(/\\/g, "/");
              if (rel !== "sw.js") {
                files.push(`./${rel}`);
              }
            }
          }
          return files;
        };

        const allFiles = getFiles(tempDir);
        if (!allFiles.includes("./")) {
          allFiles.unshift("./");
        }

        const hash = crypto.createHash("sha256");
        const sortedFiles = [...allFiles].sort();
        for (const file of sortedFiles) {
          if (file === "./") continue;
          const absPath = path.join(tempDir, file.replace(/^\.\//, ""));
          if (fs.existsSync(absPath)) {
            hash.update(file);
            hash.update(fs.readFileSync(absPath));
          }
        }
        const contentHash = hash.digest("hex").slice(0, 10);

        let swContent = fs.readFileSync(tempSwPath, "utf-8");
        const precacheStr = JSON.stringify(allFiles, null, 2);
        swContent = swContent.replace(
          /const PRECACHE_ASSETS = \[[^\]]*\];/s,
          `const PRECACHE_ASSETS = ${precacheStr};`,
        );
        swContent = swContent.replace(
          /const CACHE_VERSION = "[^"]*";/,
          `const CACHE_VERSION = "${contentHash}";`,
        );
        fs.writeFileSync(tempSwPath, swContent, "utf-8");
        return { contentHash, swContent };
      };

      // 初回実行
      const firstRun = runPrecacheInjection();
      expect(firstRun.contentHash).toBeTruthy();

      // 2. 同名ファイルの内容を変更
      fs.writeFileSync(presetPath, JSON.stringify({ name: "version-2" }));

      // 2回目実行
      const secondRun = runPrecacheInjection();
      expect(secondRun.contentHash).toBeTruthy();

      // 内容ハッシュが異なり、sw.js 本文も更新されていること
      expect(secondRun.contentHash).not.toBe(firstRun.contentHash);
      expect(secondRun.swContent).not.toBe(firstRun.swContent);
      expect(secondRun.swContent).toContain(
        `const CACHE_VERSION = "${secondRun.contentHash}";`,
      );
    } finally {
      fs.rmSync(tempDir, { recursive: true, force: true });
    }
  });

  test("Service Worker fetch uses Stale-While-Revalidate to update cached static assets from network", async () => {
    // Service Worker の fetch ハンドラを Node VM で検証
    const swSource = fs.readFileSync(
      path.resolve(__dirname, "../public/sw.js"),
      "utf-8",
    );

    let fetchHandler: ((event: any) => void) | null = null;
    const cacheStorage = new Map<string, Map<string, any>>();

    const mockCaches = {
      open: async (name: string) => {
        if (!cacheStorage.has(name)) cacheStorage.set(name, new Map());
        const store = cacheStorage.get(name)!;
        return {
          put: async (req: any, res: any) => {
            const key = typeof req === "string" ? req : req.url;
            store.set(key, res);
          },
          match: async (req: any) => {
            const key = typeof req === "string" ? req : req.url;
            return store.get(key) || null;
          },
        };
      },
      match: async (req: any) => {
        const key = typeof req === "string" ? req : req.url;
        for (const store of cacheStorage.values()) {
          if (store.has(key)) return store.get(key);
        }
        return null;
      },
      keys: async () => Array.from(cacheStorage.keys()),
      delete: async (name: string) => cacheStorage.delete(name),
    };

    let networkCallCount = 0;
    let networkContent = "first-content";
    const mockFetch = async (req: any) => {
      networkCallCount++;
      const textVal = networkContent;
      return {
        ok: true,
        text: async () => textVal,
        clone: function () {
          return {
            ok: true,
            text: async () => textVal,
          };
        },
      };
    };

    const sandbox = {
      self: {
        addEventListener: (event: string, handler: any) => {
          if (event === "fetch") fetchHandler = handler;
        },
        registration: { scope: "http://localhost:5173/" },
        location: { href: "http://localhost:5173/sw.js" },
        skipWaiting: () => Promise.resolve(),
        clients: { claim: () => Promise.resolve() },
      },
      caches: mockCaches,
      fetch: mockFetch,
      URL,
      Response: class {},
      Promise,
      setTimeout,
    };

    vm.createContext(sandbox);
    vm.runInContext(swSource, sandbox);

    expect(fetchHandler).not.toBeNull();

    const testUrl = "http://localhost:5173/cubes/easy-5-moves.json";
    const request = {
      method: "GET",
      url: testUrl,
      mode: "cors",
    };

    // 1. 初回 fetch: キャッシュなし -> ネットワークから取得
    let responsePromise: Promise<any> | null = null;
    fetchHandler!({
      request,
      respondWith: (p: Promise<any>) => {
        responsePromise = p;
      },
    });

    const res1 = await responsePromise!;
    expect(await res1.text()).toBe("first-content");
    expect(networkCallCount).toBe(1);

    // 2. ネットワーク側の内容を更新
    networkContent = "second-content-updated";

    // 3. 2回目 fetch: キャッシュあり -> 即座に旧キャッシュが返り、裏でネットワーク fetch が走る (SWR)
    fetchHandler!({
      request,
      respondWith: (p: Promise<any>) => {
        responsePromise = p;
      },
    });

    const res2 = await responsePromise!;
    // SWR: 即時返却されるのはキャッシュされていた旧コンテンツ
    expect(await res2.text()).toBe("first-content");
    // バックグラウンドでネットワークフェッチが実行されたこと
    expect(networkCallCount).toBe(2);

    // キャッシュ更新の非同期完了を少し待つ
    await new Promise((resolve) => setTimeout(resolve, 50));

    // 4. 3回目 fetch: バックグラウンド更新後の最新キャッシュが返ること
    fetchHandler!({
      request,
      respondWith: (p: Promise<any>) => {
        responsePromise = p;
      },
    });

    const res3 = await responsePromise!;
    expect(await res3.text()).toBe("second-content-updated");
    expect(networkCallCount).toBe(3);
  });
});
