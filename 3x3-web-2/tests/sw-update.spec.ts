import { test as base, expect, type Page } from "@playwright/test";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { createServer } from "node:http";
import path from "node:path";
import os from "node:os";
import { fileURLToPath } from "node:url";

const projectDir = fileURLToPath(new URL("..", import.meta.url));
const presetFile = "cubes/easy-5-moves.json";
const firstPreset = { name: "version-1" };
const secondPreset = { name: "version-2" };
const deploymentScopes = [
  "/nested/cube/",
  "/nested/cube-alt/",
  "/nested-cube/",
  "/root/",
  "/",
];

type Build = {
  distDir: string;
  writePreset: (value: typeof firstPreset) => void;
  generate: () => { source: string; version: string };
};

const test = base.extend<{
  build: Build;
  deployment: { origin: string; holdPresetResponses: () => () => void };
}>({
  build: async ({}, use) => {
    const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "sw-regression-"));
    const distDir = path.join(tempDir, "dist");
    const scriptsDir = path.join(tempDir, "scripts");
    try {
      fs.mkdirSync(path.join(distDir, "cubes"), { recursive: true });
      fs.mkdirSync(scriptsDir);
      fs.writeFileSync(
        path.join(distDir, "index.html"),
        "<!doctype html><title>Service Worker regression test</title>",
      );
      fs.copyFileSync(
        path.join(projectDir, "public/sw.js"),
        path.join(distDir, "sw.js"),
      );
      // 実スクリプトを相対パス構造ごと隔離し、作業中の dist は変更しない。
      const scriptPath = path.join(scriptsDir, "generate-sw-precache.mjs");
      fs.copyFileSync(
        path.join(projectDir, "scripts/generate-sw-precache.js"),
        scriptPath,
      );
      const writePreset: Build["writePreset"] = (value) =>
        fs.writeFileSync(path.join(distDir, presetFile), JSON.stringify(value));
      writePreset(firstPreset);
      await use({
        distDir,
        writePreset,
        generate: () => {
          execFileSync(process.execPath, [scriptPath], { timeout: 10000 });
          const source = fs.readFileSync(path.join(distDir, "sw.js"), "utf-8");
          const version = source.match(/const CACHE_VERSION = "([^"]+)";/)?.[1];
          expect(version).toBeTruthy();
          return { source, version: version! };
        },
      });
    } finally {
      fs.rmSync(tempDir, { recursive: true, force: true });
    }
  },
  deployment: async ({ build }, use) => {
    let presetGate: Promise<void> = Promise.resolve();
    let releasePreset = () => {};
    // 更新版SWの配信にも実HTTPを使う（Service Worker更新要求はrouteできない）。
    const server = createServer(async (request, response) => {
      const pathname = new URL(request.url!, "http://localhost").pathname;
      const scope = deploymentScopes.find((scope) =>
        pathname.startsWith(scope),
      );
      const file = pathname.slice(scope?.length ?? 1) || "index.html";
      const contentTypes: Record<string, string> = {
        "index.html": "text/html",
        "sw.js": "text/javascript",
        [presetFile]: "application/json",
      };
      if (!Object.hasOwn(contentTypes, file)) {
        response.writeHead(404).end();
        return;
      }
      const body = fs.readFileSync(path.join(build.distDir, file));
      if (file === presetFile) await presetGate;
      response.writeHead(200, {
        "Content-Type": contentTypes[file],
        "Cache-Control": "no-store",
      });
      response.end(body);
    });
    try {
      await new Promise<void>((resolve, reject) => {
        server.once("error", reject);
        server.listen(0, "127.0.0.1", resolve);
      });
      const address = server.address();
      if (!address || typeof address === "string") {
        throw new Error("Test server did not bind a TCP port");
      }
      await use({
        origin: `http://127.0.0.1:${address.port}`,
        holdPresetResponses: () => {
          presetGate = new Promise<void>((resolve) => {
            releasePreset = resolve;
          });
          return releasePreset;
        },
      });
    } finally {
      releasePreset();
      server.closeAllConnections();
      await new Promise<void>((resolve) => server.close(() => resolve()));
    }
  },
});

async function registerWorker(page: Page) {
  await page.evaluate(async () => {
    await navigator.serviceWorker.register("./sw.js", {
      scope: "./",
      updateViaCache: "none",
    });
    await navigator.serviceWorker.ready;
  });
  await page.waitForFunction(
    () => navigator.serviceWorker.controller?.state === "activated",
  );
}

async function cachedPreset(page: Page, cacheName: string) {
  return page.evaluate(
    async ({ cacheName, presetFile }) => {
      const cache = await caches.open(cacheName);
      const response = await cache.match(new URL(presetFile, location.href));
      return response ? response.json() : null;
    },
    { cacheName, presetFile },
  );
}

async function updateWorker(page: Page) {
  await page.evaluate(async () => {
    const registration = await navigator.serviceWorker.getRegistration();
    const controllerChanged = new Promise<void>((resolve) => {
      navigator.serviceWorker.addEventListener(
        "controllerchange",
        () => resolve(),
        { once: true },
      );
    });
    await registration!.update();
    await controllerChanged;
  });
  await page.waitForFunction(
    () => navigator.serviceWorker.controller?.state === "activated",
  );
}

test("R14: actual precache generator changes SW only when asset content changes", async ({
  build,
}) => {
  const first = build.generate();
  expect(first.source).toContain(`"./${presetFile}"`);
  expect(build.generate()).toEqual(first);

  // 同じ長さ・同じファイル名の内容変更だけで更新を検知する。
  build.writePreset(secondPreset);
  const second = build.generate();
  expect(second.version).not.toBe(first.version);
  expect(second.source).not.toBe(first.source);
  expect(build.generate()).toEqual(second);

  build.writePreset(firstPreset);
  expect(build.generate()).toEqual(first);
});

for (const { scope, prefix, otherDeployment } of [
  {
    scope: "/",
    prefix: "cube-studio-root-",
    otherDeployment: "cube-studio-nested-cube-v1",
  },
  {
    scope: "/nested/cube/",
    prefix: "cube-studio-nested-cube-",
    otherDeployment: "cube-studio-root-v1",
  },
]) {
  test(`R02/R14: ${scope} preserves foreign caches through activation and a real SW update`, async ({
    page,
    context,
    build,
    deployment,
  }) => {
    const first = build.generate();
    const preserved = ["another-app-v1", "cube-studio-v2", otherDeployment];
    await page.goto(deployment.origin + scope);
    await page.evaluate(
      async (names) => {
        for (const name of names) {
          const cache = await caches.open(name);
          await cache.put("/cache-owner", new Response(name));
        }
      },
      [...preserved, `${prefix}old-v0`],
    );

    const expectCaches = async (version: string) => {
      await expect
        .poll(() => page.evaluate(async () => (await caches.keys()).sort()))
        .toEqual([...preserved, prefix + version].sort());
      // 名前が残るだけでなく、他配置の保存データも維持される。
      expect(
        await page.evaluate(async (names) => {
          return Promise.all(
            names.map(async (name) => {
              const cache = await caches.open(name);
              return (await cache.match("/cache-owner"))?.text();
            }),
          );
        }, preserved),
      ).toEqual(preserved);
    };

    await registerWorker(page);
    await expectCaches(first.version);
    expect(await cachedPreset(page, prefix + first.version)).toEqual(
      firstPreset,
    );

    build.writePreset(secondPreset);
    const second = build.generate();
    expect(second.version).not.toBe(first.version);
    await updateWorker(page);
    await expectCaches(second.version);
    expect(await cachedPreset(page, prefix + second.version)).toEqual(
      secondPreset,
    );

    await context.setOffline(true);
    expect(
      await page.evaluate(
        async (file) => (await fetch(file)).json(),
        presetFile,
      ),
    ).toEqual(secondPreset);
  });
}

test("R14: cached preset returns before revalidation finishes and the updated value works offline", async ({
  page,
  context,
  build,
  deployment,
}) => {
  const { version } = build.generate();
  await page.goto(`${deployment.origin}/nested/cube/`);
  await registerWorker(page);

  // キャッシュがない場合のネットワーク取得・保存も実ブラウザで確認する。
  const cacheName = `cube-studio-nested-cube-${version}`;
  await page.evaluate(
    async ({ cacheName, presetFile }) => {
      const cache = await caches.open(cacheName);
      await cache.delete(new URL(presetFile, location.href));
    },
    { cacheName, presetFile },
  );
  expect(await cachedPreset(page, cacheName)).toBeNull();
  expect(
    await page.evaluate(async (file) => (await fetch(file)).json(), presetFile),
  ).toEqual(firstPreset);
  expect(await cachedPreset(page, cacheName)).toEqual(firstPreset);

  // SWは更新せず、固定URLの応答だけ変更してSWR単体の動作を確認する。
  build.writePreset(secondPreset);
  const release = deployment.holdPresetResponses();
  try {
    expect(
      await page.evaluate(
        async (file) => (await fetch(file)).json(),
        presetFile,
      ),
    ).toEqual(firstPreset);
  } finally {
    release();
  }

  await expect.poll(() => cachedPreset(page, cacheName)).toEqual(secondPreset);
  await context.setOffline(true);
  expect(
    await page.evaluate(async (file) => (await fetch(file)).json(), presetFile),
  ).toEqual(secondPreset);
});

for (const { scope, otherScope } of [
  { scope: "/nested/cube/", otherScope: "/nested/cube-alt/" },
  { scope: "/nested/cube/", otherScope: "/nested-cube/" },
  { scope: "/", otherScope: "/root/" },
]) {
  test(`R02: ${scope} preserves the real deployment at ${otherScope} through activation and update`, async ({
    page,
    build,
    deployment,
  }) => {
    // 手書きのキャッシュ名ではなく、別配置の実SWが保存したデータを使う。
    build.generate();
    await page.goto(deployment.origin + otherScope);
    await registerWorker(page);
    const otherCaches = await page.evaluate(() => caches.keys());
    expect(otherCaches).toHaveLength(1);
    const otherCache = otherCaches[0];
    const otherPresetUrl = new URL(presetFile, deployment.origin + otherScope)
      .href;
    const readOtherPreset = () =>
      page.evaluate(
        async ({ cacheName, url }) => {
          // caches.open()で、削除済みキャッシュを再作成しない。
          const response = await caches.match(url, { cacheName });
          return response ? response.json() : null;
        },
        { cacheName: otherCache, url: otherPresetUrl },
      );
    expect(await readOtherPreset()).toEqual(firstPreset);

    build.writePreset(secondPreset);
    build.generate();
    await page.goto(deployment.origin + scope);
    await registerWorker(page);
    expect(
      await readOtherPreset(),
      `${scope} activation must preserve ${otherScope}'s cached preset`,
    ).toEqual(firstPreset);
    const ownCaches = (await page.evaluate(() => caches.keys())).filter(
      (name) => name !== otherCache,
    );
    expect(ownCaches).toHaveLength(1);

    build.writePreset({ name: "version-3" });
    build.generate();
    await updateWorker(page);
    expect(
      await readOtherPreset(),
      `${scope} update must preserve ${otherScope}'s cached preset`,
    ).toEqual(firstPreset);
    const updatedCaches = await page.evaluate(() => caches.keys());
    expect(updatedCaches).toHaveLength(2);
    expect(updatedCaches).toContain(otherCache);
    expect(updatedCaches).not.toContain(ownCaches[0]);
  });
}

for (const offline of [false, true]) {
  test(`R14: updated preset overrides a legacy cache while ${offline ? "offline" : "online"}`, async ({
    page,
    context,
    build,
    deployment,
  }) => {
    build.generate();
    await page.goto(`${deployment.origin}/nested/cube/`);
    // 旧バージョン利用者が持つ、同じURLの古い応答を先に保存する。
    await page.evaluate(async (file) => {
      const legacy = await caches.open("cube-studio-v1");
      await legacy.put(
        new URL(file, location.href),
        new Response(JSON.stringify({ name: "legacy" }), {
          headers: { "Content-Type": "application/json" },
        }),
      );
    }, presetFile);
    await registerWorker(page);

    build.writePreset(secondPreset);
    const second = build.generate();
    await updateWorker(page);
    const currentCaches = (await page.evaluate(() => caches.keys())).filter(
      (name) => name.endsWith(second.version),
    );
    expect(currentCaches).toHaveLength(1);
    // 更新自体は成功していることを確認してから、利用者に返る内容を検証する。
    expect(await cachedPreset(page, currentCaches[0])).toEqual(secondPreset);

    await context.setOffline(offline);
    expect(
      await page.evaluate(
        async (file) => (await fetch(file)).json(),
        presetFile,
      ),
      "A completed SW update must serve the new preset even when a legacy cache exists",
    ).toEqual(secondPreset);
  });
}

test("offline navigation with query parameters (?state=...&alg=...) returns cached index.html from own cache", async ({
  page,
  context,
  build,
  deployment,
}) => {
  build.generate();
  const scope = "/nested/cube/";
  await page.goto(deployment.origin + scope);
  await registerWorker(page);

  await context.setOffline(true);
  // クエリパラメータ付きでナビゲーションリクエストを行う
  const response = await page.goto(
    `${deployment.origin}${scope}?state=custom&alg=R_U_R'`,
  );
  expect(response).not.toBeNull();
  expect(response!.status()).toBe(200);
  const text = await response!.text();
  expect(text).toContain("Service Worker regression test");
});

test("preserves legacy un-scoped cache (e.g. cube-studio-v1) as foreign deployment on activation without touching other apps", async ({
  page,
  build,
  deployment,
}) => {
  build.generate();
  const scope = "/nested/cube/";
  await page.goto(deployment.origin + scope);

  // 別配置や他アプリのキャッシュを準備
  await page.evaluate(async () => {
    const legacy = await caches.open("cube-studio-v1");
    await legacy.put("/legacy-file", new Response("legacy"));
    const otherApp = await caches.open("another-app-v1");
    await otherApp.put("/other-file", new Response("other"));
  });

  await registerWorker(page);

  const remainingKeys = await page.evaluate(() => caches.keys());
  // 他アプリのキャッシュは保護される
  expect(remainingKeys).toContain("another-app-v1");
  // 別配置とみなされる un-scoped cube-studio キャッシュも勝手に削除されない（R02原則）
  expect(remainingKeys).toContain("cube-studio-v1");
  // 自身のキャッシュが作成されていること
  expect(remainingKeys.some((k) => k.startsWith("cube-studio-nested-cube-"))).toBe(true);
});
