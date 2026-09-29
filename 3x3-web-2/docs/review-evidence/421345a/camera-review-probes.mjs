import { createRequire } from "node:module";
import { writeFileSync } from "node:fs";
const require = createRequire(`${process.cwd()}/package.json`);
const { chromium } = require("@playwright/test");
const browser = await chromium.launch({
  args: [
    "--use-fake-device-for-media-stream",
    "--use-fake-ui-for-media-stream",
  ],
});
const context = await browser.newContext({
  permissions: ["camera"],
  viewport: { width: 1440, height: 1080 },
});
const results = {};
async function fresh() {
  const page = await context.newPage();
  page.setDefaultTimeout(15000);
  await page.goto("http://127.0.0.1:5197/?no-sw");
  await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();
  await page.waitForTimeout(500);
  await page.locator('button[data-tab="colors"]').click();
  await page.locator("#camera-colors").click();
  await page.locator("#camera-editor").waitFor({ state: "visible" });
  await page.waitForFunction(() => !!window.__lastCamera);
  return page;
}
async function upload(page, view) {
  await page
    .locator(`#camera-file-${view.toLowerCase()}`)
    .setInputFiles(`test-images/solved-view-${view.toLowerCase()}.png`);
  await page.waitForFunction((view) => {
    const c = window.__lastCamera;
    return c?.currentView === view && c.points.length === 6;
  }, view);
}
try {
  {
    const page = await fresh();
    await upload(page, "A");
    const before = await page.evaluate(() => ({
      faces: window.__lastCamera.faces,
      enabled: !document.querySelector("#camera-capture").disabled,
    }));
    await page.locator("#camera-live-stream").click();
    await page.waitForFunction(() => window.__lastCamera.isStreaming);
    await page.waitForTimeout(100);
    const live = await page.evaluate(() => ({
      streaming: window.__lastCamera.isStreaming,
      points: window.__lastCamera.points.length,
      enabled: !document.querySelector("#camera-capture").disabled,
      livePixel: [
        ...document
          .querySelector("#camera-canvas")
          .getContext("2d")
          .getImageData(5, 5, 1, 1).data,
      ],
    }));
    await page.locator("#camera-capture").click();
    results.liveSamplesOldPhoto = {
      before,
      live,
      after: await page.evaluate(() => ({
        faces: window.__lastCamera.faces,
        streaming: window.__lastCamera.isStreaming,
      })),
    };
    await page.close();
  }
  {
    const page = await fresh();
    await upload(page, "A");
    await page.locator("#camera-capture").click();
    await upload(page, "B");
    await page.locator("#camera-capture").click();
    await page.locator("#camera-view-a").click();
    await page.evaluate(() => {
      const OriginalImage = window.Image;
      window.__OriginalImage = OriginalImage;
      window.Image = class extends OriginalImage {
        constructor() {
          super();
          this.addEventListener("load", () => (window.__rawLoaded = true), {
            once: true,
          });
          Object.defineProperty(this, "onload", {
            set(callback) {
              window.__releaseImage = () => callback.call(this);
            },
            get() {
              return null;
            },
          });
        }
      };
    });
    await page
      .locator("#camera-file-a")
      .setInputFiles("test-images/scrambled-1-view-a.png");
    await page.waitForFunction(() => window.__rawLoaded);
    const pending = await page.evaluate(() => ({
      progress: document.querySelector("#camera-progress").textContent,
      captureEnabled: !document.querySelector("#camera-capture").disabled,
      applyEnabled: !document.querySelector("#camera-apply").disabled,
      points: window.__lastCamera.points.length,
      oldImageStillActive: !!window.__lastCamera.imageA,
    }));
    await page.locator("#camera-capture").click();
    const recaptured = await page.evaluate(() => ({
      progress: document.querySelector("#camera-progress").textContent,
      applyEnabled: !document.querySelector("#camera-apply").disabled,
      faces: window.__lastCamera.faces,
    }));
    await page.locator("#camera-apply").click();
    const applied = await page.evaluate(() => ({
      editorOpen: document.querySelector("#editor").open,
      editorColors: [...document.querySelectorAll("#editor-net .sticker")]
        .map((e) => e.dataset.color)
        .join(""),
    }));
    results.pendingDecodeCanApplyOldPhoto = { pending, recaptured, applied };
    await page.close();
  }
  {
    const page = await fresh();
    await page.locator("#camera-live-stream").click();
    await page.waitForFunction(() => window.__lastCamera.isStreaming);
    await page.evaluate(() => {
      const original = HTMLCanvasElement.prototype.toBlob;
      window.__toBlobOriginal = original;
      HTMLCanvasElement.prototype.toBlob = function (callback, type, quality) {
        HTMLCanvasElement.prototype.toBlob = original;
        original.call(
          this,
          (blob) => {
            window.__releasePhoto = () => callback(blob);
          },
          type,
          quality,
        );
      };
    });
    await page.locator("#camera-take-photo").click();
    await page.waitForFunction(() => !!window.__releasePhoto);
    await page.locator("#camera-view-b").click();
    const before = await page.evaluate(() => ({
      view: window.__lastCamera.currentView,
      seq: window.__lastCamera.latestViewActivationSeq,
    }));
    await page.evaluate(() => window.__releasePhoto());
    await page.waitForFunction(() => !!window.__lastCamera.imageA);
    results.delayedCaptureReactivatesA = {
      before,
      after: await page.evaluate(() => ({
        view: window.__lastCamera.currentView,
        imageA: !!window.__lastCamera.imageA,
        imageB: !!window.__lastCamera.imageB,
        seq: window.__lastCamera.latestViewActivationSeq,
      })),
    };
    await page.close();
  }
  {
    const page = await fresh();
    await upload(page, "A");
    await page.locator("#camera-file-a").setInputFiles({
      name: "broken.png",
      mimeType: "image/png",
      buffer: Buffer.from("bad image"),
    });
    await page.waitForFunction(() => window.__lastCamera.imageA === undefined);
    results.decodeErrorInvisible = await page.evaluate(() => ({
      imageA: !!window.__lastCamera.imageA,
      error: document.querySelector("#camera-error").textContent,
      status: document.querySelector("#camera-status-a").textContent,
    }));
    await page.close();
  }
  {
    const page = await fresh();
    await upload(page, "A");
    results.resizeThinImage = await page.evaluate(async () => {
      const c = window.__lastCamera;
      const canvas = document.createElement("canvas");
      canvas.width = 4000;
      canvas.height = 1;
      canvas.getContext("2d").fillRect(0, 0, 4000, 1);
      const blob = await new Promise((resolve) => canvas.toBlob(resolve));
      const old = c.sourceUrlA;
      await c.processFile(
        new File([blob], "wide.png", { type: "image/png" }),
        "A",
      );
      return {
        selectedDimensions: [4000, 1],
        fileBytes: blob.size,
        oldImageRetained: c.sourceUrlA === old,
        dimensions: [c.imageA.naturalWidth, c.imageA.naturalHeight],
        error: document.querySelector("#camera-error").textContent,
        captureEnabled: !document.querySelector("#camera-capture").disabled,
      };
    });
    await page.close();
  }
} finally {
  await browser.close();
  writeFileSync(
    new URL("./camera-review-probes.json", import.meta.url),
    JSON.stringify(results, null, 2),
  );
  console.log(JSON.stringify(results, null, 2));
}
