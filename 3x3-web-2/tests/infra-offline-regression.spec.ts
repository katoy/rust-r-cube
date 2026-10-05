import { test, expect } from "@playwright/test";
import fs from "node:fs";
import path from "node:path";
import http from "node:http";
import vm from "node:vm";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import {
  verifyServerBuild,
  waitForServiceWorker,
} from "../scripts/launch-offline.js";
import {
  collectInputFiles,
  saveBuildManifest,
} from "../scripts/build-manifest.js";

const root = fileURLToPath(new URL("..", import.meta.url));
const workerSource = fs.readFileSync(path.join(root, "public/sw.js"), "utf8");

async function listen(server: http.Server) {
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address() as { port: number };
  return { origin: `http://127.0.0.1:${address.port}`, port: address.port };
}

async function close(server: http.Server) {
  server.closeAllConnections();
  await new Promise<void>((resolve) => server.close(() => resolve()));
}

for (const scope of ["/", "/nested/cube/"]) {
  test(`F06: failed real SW update preserves complete offline A at ${scope}`, async ({
    page,
    context,
  }) => {
    let version = "a";
    let missingAvailable = false;
    const server = http.createServer((request, response) => {
      const file = new URL(request.url!, "http://localhost").pathname.slice(
        scope.length,
      );
      response.setHeader("Cache-Control", "no-store");
      if (file === "sw.js") {
        response.setHeader("Content-Type", "text/javascript");
        response.end(
          workerSource
            .replace(
              'const CACHE_VERSION = "v1";',
              `const CACHE_VERSION = "${version.repeat(10)}";`,
            )
            .replace(
              /const PRECACHE_ASSETS = \[[^\]]*\];/s,
              `const PRECACHE_ASSETS = ["./", "./index.html", "./assets/app.js"${version === "b" ? ', "./missing.js"' : ""}];`,
            ),
        );
      } else if (!file || file === "index.html") {
        response.setHeader("Content-Type", "text/html");
        response.end(
          `<title>${version}</title><script src="./assets/app.js"></script>`,
        );
      } else if (file === "assets/app.js") {
        response.setHeader("Content-Type", "text/javascript");
        response.end(`window.bootVersion="${version}";`);
      } else if (file === "missing.js" && missingAvailable) {
        response.setHeader("Content-Type", "text/javascript");
        response.end("// restored deployment asset");
      } else response.writeHead(404).end();
    });
    const { origin } = await listen(server);
    try {
      await page.goto(origin + scope);
      await page.evaluate(() =>
        navigator.serviceWorker.register("./sw.js", { updateViaCache: "none" }),
      );
      await waitForServiceWorker(page, 5000);
      version = "b";
      await page.evaluate(async () => {
        const registration = (await navigator.serviceWorker.getRegistration())!;
        const failed = new Promise<void>((resolve) => {
          registration.addEventListener(
            "updatefound",
            () => {
              const worker = registration.installing!;
              worker.addEventListener("statechange", () => {
                if (worker.state === "redundant") resolve();
              });
            },
            { once: true },
          );
        });
        await registration.update();
        await failed;
      });
      const online = await page.goto(origin + scope + "index.html?state=b");
      expect(await online!.text()).toContain("<title>b</title>");
      // A fixed-name asset from deployment B must not overwrite A's precache.
      await page.evaluate(() => fetch("./assets/app.js?refresh=b"));
      expect(
        await page.evaluate(async () => {
          const cache = await caches.open(
            "cube-studio-" +
              (location.pathname.startsWith("/nested/")
                ? "nested-cube"
                : "root") +
              "-aaaaaaaaaa",
          );
          return (await cache.match(
            new URL("./assets/app.js", location.href),
          ))!.text();
        }),
      ).toContain('bootVersion="a"');
      await context.setOffline(true);
      for (const alias of [
        "",
        "index.html",
        "?state=one",
        "index.html?state=two",
      ]) {
        const offline = await page.goto(origin + scope + alias);
        expect(offline!.status()).toBe(200);
        await expect(page).toHaveTitle("a");
        expect(await page.evaluate(() => (window as any).bootVersion)).toBe(
          "a",
        );
      }
      missingAvailable = true;
      await context.setOffline(false);
      await page.evaluate(async () => {
        const changed = new Promise<void>((resolve) =>
          navigator.serviceWorker.addEventListener(
            "controllerchange",
            () => resolve(),
            { once: true },
          ),
        );
        await (await navigator.serviceWorker.getRegistration())!.update();
        await changed;
      });
      await waitForServiceWorker(page, 5000);
      await context.setOffline(true);
      await page.goto(origin + scope + "index.html?state=recovered");
      await expect(page).toHaveTitle("b");
      expect(await page.evaluate(() => (window as any).bootVersion)).toBe("b");
    } finally {
      await close(server);
    }
  });
}

for (const cached of [false, true]) {
  test(`F07: Cache.put rejection preserves ${cached ? "cached and network" : "HTTP 200"} responses`, async () => {
    let handler: (event: any) => void;
    const warnings: unknown[][] = [];
    const sandbox = {
      URL,
      Response,
      console: { warn: (...args: unknown[]) => warnings.push(args) },
      self: {
        registration: { scope: "http://example.test/" },
        addEventListener: (type: string, fn: any) => {
          if (type === "fetch") handler = fn;
        },
      },
      caches: {
        open: async () => ({
          match: async () => (cached ? new Response("cached") : undefined),
          put: async () => {
            throw new Error("QuotaExceededError");
          },
        }),
      },
      fetch: async () => new Response("network", { status: 200 }),
    };
    vm.runInNewContext(workerSource, sandbox);
    let reply: Promise<Response>;
    const waits: Promise<unknown>[] = [];
    handler!({
      request: {
        method: "GET",
        mode: "cors",
        url: "http://example.test/runtime.js",
      },
      respondWith: (promise: Promise<Response>) => {
        reply = promise;
      },
      waitUntil: (promise: Promise<unknown>) => waits.push(promise),
    });
    const response = await reply!;
    expect(response.status).toBe(200);
    expect(await response.text()).toBe(cached ? "cached" : "network");
    await Promise.all(waits);
    expect(waits).toHaveLength(1);
    expect(warnings).toHaveLength(1);
    expect(String(warnings[0][1])).toContain("QuotaExceededError");
    sandbox.fetch = async () => {
      throw new TypeError("network unavailable");
    };
    handler!({
      request: {
        method: "GET",
        mode: "cors",
        url: "http://example.test/runtime.js",
      },
      respondWith: (promise: Promise<Response>) => {
        reply = promise;
      },
      waitUntil: (promise: Promise<unknown>) => waits.push(promise),
    });
    const fallback = await reply!;
    expect(fallback.status).toBe(cached ? 200 : 0);
    if (cached) expect(await fallback.text()).toBe("cached");
    await Promise.all(waits);
  });
}

test("F07: injected Cache.put failure in a real SW still serves HTTP 200", async ({
  page,
}) => {
  const server = http.createServer((request, response) => {
    if (request.url === "/sw.js") {
      response.setHeader("Content-Type", "text/javascript");
      response.end(
        'Cache.prototype.put = async () => { throw new DOMException("quota full", "QuotaExceededError"); };\n' +
          workerSource.replace(
            /const PRECACHE_ASSETS = \[[^\]]*\];/s,
            'const PRECACHE_ASSETS = ["./", "./index.html"];',
          ),
      );
    } else if (request.url === "/runtime.js") {
      response.setHeader("Content-Type", "text/javascript");
      response.end("export const alive = true;");
    } else {
      response.setHeader("Content-Type", "text/html");
      response.end("<title>Quota fixture</title>");
    }
  });
  const { origin } = await listen(server);
  try {
    await page.goto(origin);
    await page.evaluate(() => navigator.serviceWorker.register("./sw.js"));
    await waitForServiceWorker(page, 5000);
    expect(
      await page.evaluate(async () => {
        const response = await fetch("./runtime.js");
        return { status: response.status, body: await response.text() };
      }),
    ).toEqual({ status: 200, body: "export const alive = true;" });
  } finally {
    await close(server);
  }
});

test("F08: controller readiness has an actual browser-side deadline", async ({
  page,
}) => {
  const server = http.createServer((_, response) =>
    response.end("<title>No registration</title>"),
  );
  const { origin } = await listen(server);
  try {
    await page.goto(origin);
    const started = Date.now();
    await expect(waitForServiceWorker(page, 300)).rejects.toThrow(
      "readiness timed out",
    );
    expect(Date.now() - started).toBeLessThan(3000);
  } finally {
    await close(server);
  }
});

for (const failingInstall of [false, true]) {
  test(`F08: ${failingInstall ? "redundant installation is diagnosed" : "ready without controller is bounded"}`, async ({
    page,
  }) => {
    const server = http.createServer((request, response) => {
      if (request.url === "/sw.js") {
        response.setHeader("Content-Type", "text/javascript");
        response.end(
          failingInstall
            ? 'self.addEventListener("install", event => event.waitUntil(new Promise((_, reject) => setTimeout(() => reject(new Error("precache failed")), 300))));'
            : 'self.addEventListener("install", event => event.waitUntil(self.skipWaiting()));',
        );
      } else {
        response.setHeader("Content-Type", "text/html");
        response.end("<title>Readiness fixture</title>");
      }
    });
    const { origin } = await listen(server);
    try {
      await page.goto(origin);
      await page.evaluate(() => navigator.serviceWorker.register("./sw.js"));
      const started = Date.now();
      await expect(waitForServiceWorker(page, 1000)).rejects.toThrow(
        failingInstall
          ? "installation failed (redundant)"
          : "readiness timed out",
      );
      expect(Date.now() - started).toBeLessThan(3000);
    } finally {
      await close(server);
    }
  });
}

async function launcherFixture() {
  const directory = fs.mkdtempSync(path.join(root, ".offline-regression-"));
  fs.mkdirSync(path.join(directory, "scripts"));
  fs.mkdirSync(path.join(directory, "dist"));
  fs.symlinkSync(
    path.join(root, "node_modules"),
    path.join(directory, "node_modules"),
    "dir",
  );
  for (const script of ["launch-offline.js", "build-manifest.js"])
    fs.copyFileSync(
      path.join(root, "scripts", script),
      path.join(directory, "scripts", script),
    );
  fs.writeFileSync(
    path.join(directory, "package.json"),
    JSON.stringify({
      type: "module",
      scripts: { build: "node -e \"console.log('FIXTURE_FRESH_BUILD')\"" },
    }),
  );
  fs.writeFileSync(
    path.join(directory, "dist/index.html"),
    '<!doctype html><div id="engine-status">READY</div><script>navigator.serviceWorker.register("./sw.js");</script>',
  );
  fs.writeFileSync(
    path.join(directory, "dist/sw.js"),
    workerSource.replace(
      /const PRECACHE_ASSETS = \[[^\]]*\];/s,
      'const PRECACHE_ASSETS = ["./", "./index.html", "./absent.png"];',
    ),
  );
  saveBuildManifest(
    directory,
    path.join(directory, "dist"),
    collectInputFiles(directory),
  );
  return directory;
}

function launch(directory: string, port: number, fresh = false) {
  return new Promise<{ code: number | null; output: string }>(
    (resolve, reject) => {
      const proc = spawn(
        process.execPath,
        [
          path.join(directory, "scripts/launch-offline.js"),
          "--headless",
          `--port=${port}`,
          ...(fresh ? ["--fresh"] : []),
        ],
        { cwd: directory },
      );
      let output = "";
      proc.stdout.on("data", (data) => {
        output += data;
      });
      proc.stderr.on("data", (data) => {
        output += data;
      });
      const timer = setTimeout(() => {
        proc.kill("SIGTERM");
        reject(new Error("Launcher exceeded test deadline:\n" + output));
      }, 30000);
      proc.on("error", reject);
      proc.on("close", (code) => {
        clearTimeout(timer);
        resolve({ code, output });
      });
    },
  );
}

test("F08: failed real installation exits nonzero and stops only its owned server", async () => {
  const directory = await launcherFixture();
  const missingAsset = http.createServer((_, response) => {
    response.setHeader("Access-Control-Allow-Origin", "*");
    response.writeHead(404).end();
  });
  const missing = await listen(missingAsset);
  const swPath = path.join(directory, "dist/sw.js");
  fs.writeFileSync(
    swPath,
    fs
      .readFileSync(swPath, "utf8")
      .replace("./absent.png", missing.origin + "/absent.png"),
  );
  const reservation = http.createServer();
  const { port, origin } = await listen(reservation);
  await close(reservation);
  try {
    const started = Date.now();
    const result = await launch(directory, port);
    expect(result.code, result.output).toBe(1);
    expect(result.output).toMatch(
      /Service Worker (readiness timed out|installation failed)/,
    );
    expect(Date.now() - started).toBeLessThan(25000);
    await expect
      .poll(async () => {
        try {
          await fetch(origin);
          return true;
        } catch {
          return false;
        }
      })
      .toBe(false);
  } finally {
    await close(missingAsset);
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("F17: --fresh rejects unrelated HTTP 200 server without killing it", async () => {
  const directory = await launcherFixture();
  const server = http.createServer((_, response) =>
    response.end("UNRELATED SERVER"),
  );
  const { port, origin } = await listen(server);
  try {
    const result = await launch(directory, port, true);
    expect(result.code).toBe(1);
    expect(result.output).toContain("FIXTURE_FRESH_BUILD");
    expect(result.output).toContain("Preview build mismatch");
    expect(await (await fetch(origin)).text()).toBe("UNRELATED SERVER");
    expect(result.output).not.toContain("オフラインモードで正常に起動");
  } finally {
    await close(server);
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("F08/F17: matching owned preview verifies offline boot and is stopped on success", async () => {
  const directory = await launcherFixture();
  const swPath = path.join(directory, "dist/sw.js");
  fs.writeFileSync(
    swPath,
    fs.readFileSync(swPath, "utf8").replace(', "./absent.png"', ""),
  );
  const reservation = http.createServer();
  const { port, origin } = await listen(reservation);
  await close(reservation);
  try {
    const result = await launch(directory, port, true);
    expect(result.code, result.output).toBe(0);
    expect(result.output).toContain("FIXTURE_FRESH_BUILD");
    expect(result.output).toContain("オフラインモードで正常に起動");
    await expect
      .poll(async () => {
        try {
          await fetch(origin);
          return true;
        } catch {
          return false;
        }
      })
      .toBe(false);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("F17: provenance checks every asset, not just index and SW", async () => {
  const directory = await launcherFixture();
  const dist = path.join(directory, "dist");
  fs.mkdirSync(path.join(dist, "assets"));
  fs.writeFileSync(path.join(dist, "assets/lazy.js"), "CURRENT ASSET");
  let stale = false;
  const server = http.createServer((request, response) => {
    const file = decodeURIComponent(
      new URL(request.url!, "http://localhost").pathname.slice(1),
    );
    response.end(
      stale && file === "assets/lazy.js"
        ? "STALE ASSET"
        : fs.readFileSync(path.join(dist, file)),
    );
  });
  const { origin } = await listen(server);
  try {
    await verifyServerBuild(origin + "/", dist);
    stale = true;
    await expect(verifyServerBuild(origin + "/", dist)).rejects.toThrow(
      "Preview build mismatch",
    );
  } finally {
    await close(server);
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
