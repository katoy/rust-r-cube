import { chromium } from "@playwright/test";
import { writeFileSync } from "node:fs";

const browser = await chromium.launch();
try {
  const page = await browser.newPage();
  await page.goto("http://127.0.0.1:5173/?no-sw");
  await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();
  await page.getByRole("tab", { name: "色を入力" }).click();
  await page.locator("#camera-colors").click();
  await page.locator("#camera-editor").waitFor({ state: "visible" });

  const result = await page.evaluate(async () => {
    const camera = window.__lastCamera;
    const canvas = document.createElement("canvas");
    canvas.width = 20;
    canvas.height = 20;
    const context = canvas.getContext("2d");
    context.fillStyle = "red";
    context.fillRect(0, 0, 20, 20);
    const blob = await new Promise((resolve) => canvas.toBlob(resolve));
    const file = new File([blob], "view-a.png", { type: "image/png" });

    const OriginalImage = window.Image;
    let releaseOnload;
    let imageLoaded;
    const loaded = new Promise((resolve) => (imageLoaded = resolve));
    window.Image = class extends OriginalImage {
      constructor() {
        super();
        this.addEventListener("load", imageLoaded, { once: true });
        Object.defineProperty(this, "onload", {
          set(callback) {
            releaseOnload = callback;
          },
          get() {
            return null;
          },
        });
      }
    };
    try {
      const pendingLoad = camera.processFile(file, "A");
      camera.switchView("B");
      const afterUserSwitch = camera.currentView;
      await loaded;
      releaseOnload();
      await pendingLoad;
      return {
        afterUserSwitch,
        afterDelayedALoad: camera.currentView,
        imageALoaded: !!camera.imageA,
        imageBLoaded: !!camera.imageB,
      };
    } finally {
      window.Image = OriginalImage;
    }
  });
  writeFileSync(new URL("./probe-camera-view-race.json", import.meta.url), JSON.stringify(result, null, 2) + "\n");
  console.log(JSON.stringify(result, null, 2));
} finally {
  await browser.close();
}
