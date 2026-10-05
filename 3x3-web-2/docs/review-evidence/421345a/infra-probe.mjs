import fs from "node:fs";
import path from "node:path";
import vm from "node:vm";
import assert from "node:assert/strict";
import http from "node:http";
import { fileURLToPath } from "node:url";
import { stripTypeScriptTypes } from "node:module";

const root = process.cwd();
const swSource = fs.readFileSync("public/sw.js", "utf8");
const results = {};
const origin = "http://example.test";
function worker({ cache, fetch }) {
  const handlers = {};
  const context = {
    URL,
    Response,
    self: {
      registration: { scope: origin + "/" },
      location: { href: origin + "/sw.js" },
      addEventListener: (type, handler) => {
        handlers[type] = handler;
      },
      skipWaiting: async () => {},
      clients: { claim: async () => {} },
    },
    caches: { open: async () => cache },
    fetch,
  };
  vm.runInNewContext(swSource, context);
  return handlers;
}
async function dispatch(handler, url, mode = "cors") {
  let response;
  const waits = [];
  handler({
    request: { url, method: "GET", mode },
    respondWith: (p) => {
      response = p;
    },
    waitUntil: (p) => {
      waits.push(p);
    },
  });
  const resolved = await response;
  await Promise.allSettled(waits);
  return resolved;
}
{
  const handlers = worker({
    cache: {
      match: async () => undefined,
      put: async () => {
        throw new DOMException("quota full", "QuotaExceededError");
      },
    },
    fetch: async () =>
      new Response("export const alive = true;", { status: 200 }),
  });
  const response = await dispatch(handlers.fetch, origin + "/assets/new.js");
  assert.equal(response.type, "error");
  results.quotaWriteFailure = {
    expected:
      "deliver successful HTTP 200 network asset despite optional cache failure",
    actual: { type: response.type, status: response.status },
  };
}
{
  const aHtml = '<script type="module" src="/assets/a.js"></script>';
  const bHtml = '<script type="module" src="/assets/b.js"></script>';
  const data = new Map([
    [origin + "/", new Response(aHtml)],
    [origin + "/index.html", new Response(aHtml)],
    [origin + "/assets/a.js", new Response("version A")],
  ]);
  let offline = false;
  const cache = {
    match: async (key) =>
      data.get(typeof key === "string" ? key : key.url)?.clone(),
    put: async (key, value) =>
      data.set(typeof key === "string" ? key : key.url, value),
    addAll: async () => {
      throw new Error("B precache asset temporarily unavailable");
    },
  };
  const handlers = worker({
    cache,
    fetch: async (req) => {
      if (offline || req.url.endsWith("/assets/b.js"))
        throw new TypeError("network unavailable");
      return new Response(bHtml, { headers: { "Content-Type": "text/html" } });
    },
  });
  await dispatch(handlers.fetch, origin + "/", "navigate");
  let installation;
  handlers.install({
    waitUntil: (p) => {
      installation = p;
    },
  });
  await assert.rejects(installation);
  offline = true;
  const html = await (
    await dispatch(handlers.fetch, origin + "/", "navigate")
  ).text();
  const asset = await dispatch(handlers.fetch, origin + "/assets/b.js");
  assert.equal(html, bHtml);
  assert.equal(asset.type, "error");
  results.failedUpdatePoisonsOldNavigation = {
    expected:
      "failed B installation preserves a complete offline A document/assets",
    actual: {
      offlineDocument: html,
      missingAssetType: asset.type,
      indexAlias: await data.get(origin + "/index.html").text(),
    },
  };
}
{
  const source = fs.readFileSync("scripts/launch-offline.js", "utf8");
  const start = source.indexOf("function checkServer(");
  const end = source.indexOf("async function main()");
  const server = http.createServer((req, res) => res.end("UNRELATED SERVER"));
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  try {
    const BASE_URL = `http://127.0.0.1:${server.address().port}/`;
    let spawnCount = 0;
    const sandbox = {
      http,
      BASE_URL,
      PORT: server.address().port,
      rootDir: root,
      isWin: false,
      console: { log: () => {} },
      process: { stderr: { write: () => {} } },
      setTimeout,
      spawn: () => {
        spawnCount++;
        throw new Error("must not spawn in reuse branch");
      },
    };
    vm.runInNewContext(
      source.slice(start, end) + "\nthis.startServer = startServer;",
      sandbox,
    );
    const value = await sandbox.startServer();
    assert.equal(value, null);
    assert.equal(spawnCount, 0);
    results.serverProvenance = {
      expected:
        "prove responding server serves the freshly built project, or reject reuse",
      actual:
        "HTTP 200 unrelated server accepted, no owned preview server started",
    };
  } finally {
    await new Promise((resolve) => server.close(resolve));
  }
}
{
  const source = fs.readFileSync("tests/coverage.spec.ts", "utf8");
  const files = new Map();
  const sandbox = {
    fs: {
      existsSync: (p) => files.has(p),
      mkdirSync: () => {},
      writeFileSync: (p, content) => files.set(p, content),
    },
    path,
    COVERAGE_DIR: "coverage-probe",
    console: { log: () => {} },
    computeMergedLineCoverage: () => {
      throw new Error("no entries expected");
    },
  };
  const reportingSource = source.slice(
    source.indexOf("function generateCoverageReport("),
  );
  vm.runInNewContext(
    stripTypeScriptTypes(reportingSource, { mode: "strip" }),
    sandbox,
  );
  const stats = sandbox.generateCoverageReport([], []);
  const webStats = stats.filter(
    (s) => s.url.includes("/web/") && !s.url.includes("node_modules"),
  );
  let thresholdsChecked = 0;
  for (const s of webStats) {
    thresholdsChecked++;
    const name = s.url.split("/").pop()?.split("?")[0];
    assert.ok(parseFloat(s.percentage) >= (name === "main.ts" ? 65 : 95));
  }
  assert.ok(files.has("coverage-probe/index.html"));
  assert.ok(files.has("coverage-probe/coverage.json"));
  results.emptyCoverageGate = {
    inputEntries: 0,
    thresholdsChecked,
    generatedBothReports: true,
    actual: "all coverage-stage assertions pass with no measured module",
  };
}
{
  const reportPath = path.join(root, "coverage-e2e/coverage.json");
  if (fs.existsSync(reportPath)) {
    const report = JSON.parse(fs.readFileSync(reportPath, "utf8"));
    const observed = new Set(
      report.files.map((f) => f.url.split("/").at(-1).split("?")[0]),
    );
    const web = fs.readdirSync("web").filter((f) => f.endsWith(".ts"));
    results.existingCoverageInventory = {
      note: "pre-existing artifact, not current-run evidence",
      timestamp: report.timestamp,
      missingFromReport: web.filter((f) => !observed.has(f)),
      observedWebCount: report.files.filter((f) => f.url.includes("/web/"))
        .length,
    };
  }
}
const out = path.join(
  path.dirname(fileURLToPath(import.meta.url)),
  "infra-probe-results.json",
);
fs.writeFileSync(
  path.relative(root, out),
  JSON.stringify(results, null, 2) + "\n",
);
console.log(JSON.stringify(results, null, 2));
