import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { chromium } from "@playwright/test";
import {
  collectInputFiles,
  saveBuildManifest,
  checkInputsFreshness,
} from "../../../scripts/build-manifest.js";

const root = fs.mkdtempSync(path.join(os.tmpdir(), "cube-review-inputs-"));
fs.mkdirSync(path.join(root, "scripts"));
fs.mkdirSync(path.join(root, "dist"));
fs.writeFileSync(path.join(root, "tsconfig.json"), '{"strict":true}');
fs.writeFileSync(path.join(root, "scripts", "build-manifest.js"), "old");
const initialInputs = collectInputFiles(root);
saveBuildManifest(root, path.join(root, "dist"), initialInputs);
fs.writeFileSync(path.join(root, "tsconfig.json"), '{"strict":false}');
fs.writeFileSync(path.join(root, "scripts", "build-manifest.js"), "new");
const changedInputs = collectInputFiles(root);
const freshness = checkInputsFreshness(
  root,
  path.join(root, "dist"),
  changedInputs,
);

fs.mkdirSync(path.join(root, "web"));
fs.mkdirSync(path.join(root, "public"));
fs.copyFileSync("web/main.ts", path.join(root, "web", "main.ts"));
fs.copyFileSync("public/sw.js", path.join(root, "public", "sw.js"));
fs.writeFileSync(
  path.join(root, "web", "storage-key.ts"),
  "export function getScopedStorageKey(baseKey: string, customPath?: string): string { return `${baseKey}:${customPath ?? window.location.pathname}`; }\n",
);
const guardrail = spawnSync(
  process.execPath,
  ["scripts/check-review-guardrails.js", root],
  { encoding: "utf8" },
);

let browser;
try {
  browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  await page.goto("http://127.0.0.1:5173/?no-sw");
  await page.locator("#engine-status").getByText("READY").waitFor();
  await page.getByRole("tab", { name: "色を入力" }).click();
  await page.locator("#camera-colors").click();
  await page.locator("#camera-editor").waitFor({ state: "visible" });

  const inactiveImage = await page.evaluate(async () => {
    const camera = window.__lastCamera;
    const canvas = document.createElement("canvas");
    canvas.width = 20;
    canvas.height = 20;
    const blob = await new Promise((resolve) => canvas.toBlob(resolve));
    const file = new File([blob], "a.png", { type: "image/png" });
    const OriginalImage = window.Image;
    let release = () => {};
    let notifyLoaded = () => {};
    const loaded = new Promise((resolve) => (notifyLoaded = resolve));
    window.Image = class extends OriginalImage {
      constructor() {
        super();
        this.addEventListener("load", notifyLoaded, { once: true });
        Object.defineProperty(this, "onload", {
          set(callback) {
            release = callback;
          },
          get() {
            return null;
          },
        });
      }
    };
    try {
      const pending = camera.processFile(file, "A");
      camera.switchView("B");
      await loaded;
      release();
      await pending;
      const statusBeforeReturn =
        document.querySelector("#camera-status-a")?.textContent;
      const cardClassBeforeReturn =
        document.querySelector("#camera-drop-a")?.className;
      const storedImage = !!camera.imageA;
      camera.switchView("A");
      return {
        storedImage,
        statusBeforeReturn,
        cardClassBeforeReturn,
        statusAfterReturn:
          document.querySelector("#camera-status-a")?.textContent,
      };
    } finally {
      window.Image = OriginalImage;
    }
  });

  console.log(
    JSON.stringify(
      {
        head: "9fb1ebcb304a096db7bc84a895778eacd9c914d9",
        manifest: {
          tsconfigTracked: initialInputs.has("tsconfig.json"),
          manifestScriptTracked: initialInputs.has("scripts/build-manifest.js"),
          freshAfterChangingBoth: freshness,
        },
        guardrailWithBrokenPathNormalizer: {
          exitCode: guardrail.status,
          output: (guardrail.stdout + guardrail.stderr).trim(),
        },
        inactiveImage,
      },
      null,
      2,
    ),
  );
} finally {
  await browser?.close();
  fs.rmSync(root, { recursive: true, force: true });
}
