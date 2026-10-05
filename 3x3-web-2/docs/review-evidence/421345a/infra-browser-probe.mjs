import fs from "node:fs";
import path from "node:path";
import http from "node:http";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

const require = createRequire(path.join(process.cwd(), "package.json"));
const { chromium } = require("@playwright/test");
const artifacts = path.dirname(fileURLToPath(import.meta.url));
const profile = path.relative(
  process.cwd(),
  path.join(artifacts, "infra-browser-profile"),
);
const sw = fs.readFileSync("public/sw.js", "utf8");
let version = "a";
let assetBAvailable = false;
let forceMissingPrecache = false;
const requests = [];
function script(v) {
  return sw
    .replace(
      'const CACHE_VERSION = "v1";',
      `const CACHE_VERSION = "${v.repeat(10)}";`,
    )
    .replace(
      /const PRECACHE_ASSETS = \[[^\]]*\];/s,
      `const PRECACHE_ASSETS = ["./", "./index.html", "./assets/${v}.js"${v === "a" ? ', "./assets/lazy-a.js"' : ""}${forceMissingPrecache ? ', "./assets/missing.js"' : ""}];`,
    );
}
const server = http.createServer((req, res) => {
  requests.push(req.url);
  const pathname = new URL(req.url, "http://localhost").pathname;
  res.setHeader("Cache-Control", "no-store");
  if (pathname === "/sw.js") {
    res.setHeader("Content-Type", "text/javascript");
    res.end(script(version));
  } else if (pathname === "/" || pathname === "/index.html") {
    res.setHeader("Content-Type", "text/html");
    res.end(
      `<title>fixture ${version}</title><script type="module" src="/assets/${version}.js"></script>`,
    );
  } else if (pathname === "/assets/a.js") {
    res.setHeader("Content-Type", "text/javascript");
    res.end(
      'window.bootVersion="a";window.loadLater=()=>import("/assets/lazy-a.js");',
    );
  } else if (pathname === "/assets/lazy-a.js" && version === "a") {
    res.setHeader("Content-Type", "text/javascript");
    res.end("export const oldCameraAvailable=true;");
  } else if (pathname === "/assets/b.js" && assetBAvailable) {
    res.setHeader("Content-Type", "text/javascript");
    res.end('window.bootVersion="b";');
  } else {
    res.writeHead(404).end("missing deployment asset");
  }
});
let context;
const errors = [];
try {
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const origin = `http://127.0.0.1:${server.address().port}`;
  context = await chromium.launchPersistentContext(profile, {
    headless: true,
    serviceWorkers: "allow",
    args: ["--disk-cache-dir=" + path.resolve(profile, "cache")],
  });
  const page = context.pages()[0];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto(origin);
  await page.evaluate(async () => {
    await navigator.serviceWorker.register("./sw.js", {
      updateViaCache: "none",
    });
    await navigator.serviceWorker.ready;
  });
  await page.waitForFunction(
    () => navigator.serviceWorker.controller?.state === "activated",
  );
  assert.equal(await page.evaluate(() => window.bootVersion), "a");
  version = "b";
  await page.reload({ waitUntil: "networkidle" });
  await page.evaluate(async () =>
    (await navigator.serviceWorker.getRegistration()).update(),
  );
  await page.waitForFunction(
    async () => !(await navigator.serviceWorker.getRegistration()).installing,
  );
  const beforeOffline = await page.evaluate(async () => ({
    active: navigator.serviceWorker.controller.scriptURL,
    keys: await caches.keys(),
    bootVersion: window.bootVersion ?? null,
    cachedHTML: await (
      await (
        await caches.open("cube-studio-root-aaaaaaaaaa")
      ).match(location.origin + "/")
    ).text(),
  }));
  await context.setOffline(true);
  await page.reload({ waitUntil: "networkidle" });
  const offline = await page.evaluate(() => ({
    title: document.title,
    bootVersion: window.bootVersion ?? null,
  }));
  assert.equal(offline.title, "fixture b");
  assert.equal(offline.bootVersion, null);
  const result = {
    beforeOffline,
    offline,
    requests: [...requests],
    errors,
    conclusion:
      "Failed B precache leaves active A SW with B navigation HTML; offline A boot is lost.",
  };
  await context.close();
  context = null;
  fs.rmSync(profile, { recursive: true, force: true });
  version = "a";
  context = await chromium.launchPersistentContext(profile, {
    headless: true,
    serviceWorkers: "allow",
  });
  const oldPage = context.pages()[0];
  await oldPage.goto(origin);
  await oldPage.evaluate(async () => {
    await navigator.serviceWorker.register("./sw.js", {
      updateViaCache: "none",
    });
    await navigator.serviceWorker.ready;
  });
  await oldPage.waitForFunction(
    () => navigator.serviceWorker.controller?.state === "activated",
  );
  version = "b";
  assetBAvailable = true;
  await oldPage.evaluate(async () => {
    const changed = new Promise((resolve) =>
      navigator.serviceWorker.addEventListener("controllerchange", resolve, {
        once: true,
      }),
    );
    await (await navigator.serviceWorker.getRegistration()).update();
    await changed;
  });
  await oldPage.waitForFunction(
    () => navigator.serviceWorker.controller?.state === "activated",
  );
  await context.setOffline(true);
  const lazy = await oldPage.evaluate(async () => {
    let error;
    try {
      await window.loadLater();
    } catch (caught) {
      error = caught.message;
    }
    return {
      documentVersion: window.bootVersion,
      remainingCaches: await caches.keys(),
      lazyLoadError: error,
    };
  });
  assert.equal(lazy.documentVersion, "a");
  assert.deepEqual(lazy.remainingCaches, ["cube-studio-root-bbbbbbbbbb"]);
  assert.match(
    lazy.lazyLoadError,
    /Failed to fetch dynamically imported module/,
  );
  result.successfulUpdateBreaksOldLazyImports = lazy;
  await context.close();
  context = null;
  fs.rmSync(profile, { recursive: true, force: true });
  version = "a";
  forceMissingPrecache = true;
  context = await chromium.launchPersistentContext(profile, {
    headless: true,
    serviceWorkers: "allow",
  });
  const failedInstallPage = context.pages()[0];
  await failedInstallPage.goto(origin);
  await failedInstallPage.evaluate(async () => {
    const registration = await navigator.serviceWorker.register("./sw.js");
    const worker = registration.installing;
    if (worker)
      await new Promise((resolve) => {
        worker.addEventListener("statechange", () => {
          if (worker.state === "redundant" || worker.state === "activated")
            resolve();
        });
      });
  });
  const readiness = await failedInstallPage.evaluate(async () => ({
    bootVersion: window.bootVersion,
    activeWorker:
      (await navigator.serviceWorker.getRegistration())?.active?.state ?? null,
    readyResolved: await Promise.race([
      navigator.serviceWorker.ready.then(() => true),
      new Promise((resolve) => setTimeout(() => resolve(false), 500)),
    ]),
  }));
  assert.equal(readiness.bootVersion, "a");
  assert.equal(readiness.readyResolved, false);
  result.firstInstallFailureLeavesLauncherReadyAwaitPending = readiness;
  fs.writeFileSync(
    path.relative(
      process.cwd(),
      path.join(artifacts, "infra-browser-results.json"),
    ),
    JSON.stringify(result, null, 2) + "\n",
  );
  console.log(JSON.stringify(result, null, 2));
} finally {
  if (context) await context.close();
  server.closeAllConnections();
  await new Promise((resolve) => server.close(resolve));
  fs.rmSync(profile, { recursive: true, force: true });
}
