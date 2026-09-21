import { readFileSync, readdirSync } from "node:fs";
import { createServer } from "node:http";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
const require = createRequire(
  new URL("../../../package.json", import.meta.url),
);
const { chromium } = require("@playwright/test");
const dist = fileURLToPath(new URL("../../../dist", import.meta.url));
const files = new Map();
function walk(dir, pre = "") {
  for (const ent of readdirSync(dir, { withFileTypes: true })) {
    const rel = pre + ent.name;
    if (ent.isDirectory()) walk(dir + "/" + ent.name, rel + "/");
    else files.set("/" + rel, readFileSync(dir + "/" + ent.name));
  }
}
walk(dist);
const camera = [...files.keys()].find((k) => /\/camera-[^/]+\.js$/.test(k));
const replacement = "/assets/camera-review-v2.js";
let version = 1;
const server = createServer((req, res) => {
  let key = new URL(req.url, "http://localhost").pathname;
  if (key === "/") key = "/index.html";
  if (version === 2 && key === camera) {
    res.writeHead(404);
    res.end();
    return;
  }
  let body = files.get(version === 2 && key === replacement ? camera : key);
  if (!body) {
    res.writeHead(404);
    res.end();
    return;
  }
  if (version === 2 && key.endsWith(".js")) {
    body = Buffer.from(
      body
        .toString()
        .replaceAll(
          camera.slice("/assets/".length),
          replacement.slice("/assets/".length),
        )
        .replace(
          /const CACHE_VERSION = "[^"]*";/,
          'const CACHE_VERSION = "decafbad42";',
        ),
    );
  }
  const type = key.endsWith(".js")
    ? "text/javascript"
    : key.endsWith(".wasm")
      ? "application/wasm"
      : key.endsWith(".css")
        ? "text/css"
        : key.endsWith(".json") || key.endsWith(".webmanifest")
          ? "application/json"
          : key.endsWith(".html")
            ? "text/html"
            : "application/octet-stream";
  res.writeHead(200, { "Content-Type": type, "Cache-Control": "no-store" });
  res.end(body);
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const url = "http://127.0.0.1:" + server.address().port;
const browser = await chromium.launch({ headless: true });
try {
  const context = await browser.newContext();
  const page = await context.newPage();
  const errors = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.goto(url);
  await page.waitForFunction(() =>
    document.querySelector("#engine-status")?.textContent.includes("READY"),
  );
  await page.evaluate(async () => {
    await navigator.serviceWorker.ready;
    if (!navigator.serviceWorker.controller)
      await new Promise((r) =>
        navigator.serviceWorker.addEventListener("controllerchange", r, {
          once: true,
        }),
      );
    window.reviewMarker = "old-document";
  });
  const oldCaches = await page.evaluate(() => caches.keys());
  version = 2;
  await page.evaluate(async () => {
    const reg = await navigator.serviceWorker.getRegistration();
    const changed = new Promise((r) =>
      navigator.serviceWorker.addEventListener("controllerchange", r, {
        once: true,
      }),
    );
    await reg.update();
    await changed;
  });
  await page.waitForFunction(async () =>
    (await caches.keys()).some((k) => k.endsWith("decafbad42")),
  );
  await context.setOffline(true);
  await page.locator('[data-tab="colors"]').click();
  await page.locator("#camera-colors").click();
  await page.waitForTimeout(1000);
  console.log(
    JSON.stringify({
      oldCaches,
      newCaches: await page.evaluate(() => caches.keys()),
      sameDocument: await page.evaluate(
        () => window.reviewMarker === "old-document",
      ),
      cameraOpened: await page
        .locator("#camera-editor")
        .evaluate((d) => d.open),
      errors,
    }),
  );
} finally {
  await browser.close();
  await new Promise((r) => server.close(r));
}
