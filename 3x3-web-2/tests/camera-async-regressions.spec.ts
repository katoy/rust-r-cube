import { test, expect, type Page } from "@playwright/test";
import { getTestImagePath, openCameraEditor } from "./test-utils";

async function upload(page: Page, view: "A" | "B" = "A") {
  await page
    .locator(`#camera-file-${view.toLowerCase()}`)
    .setInputFiles(getTestImagePath("solved", view));
  await expect(
    page.locator(`#camera-status-${view.toLowerCase()}`),
  ).toContainText("読込完了");
  await expect(page.locator("#camera-capture")).toBeEnabled();
}

async function holdNextDecode(page: Page) {
  await page.evaluate(() => {
    const OriginalImage = window.Image;
    window.Image = class extends OriginalImage {
      constructor() {
        super();
        window.Image = OriginalImage;
        Object.defineProperty(this, "onload", {
          set(callback) {
            this.addEventListener(
              "load",
              () => {
                let released = false;
                const release = () => {
                  if (released) return;
                  released = true;
                  clearTimeout(timer);
                  callback.call(this, new Event("load"));
                };
                const timer = setTimeout(release, 8000);
                (window as any).releaseDecode = release;
              },
              { once: true },
            );
          },
        });
      }
    };
  });
}

async function largePng(page: Page, width: number, height: number) {
  const data = await page.evaluate(
    ({ width, height }) => {
      const canvas = document.createElement("canvas");
      canvas.width = width;
      canvas.height = height;
      canvas.getContext("2d")!.fillRect(0, 0, width, height);
      return canvas.toDataURL("image/png").split(",")[1];
    },
    { width, height },
  );
  return {
    name: "large.png",
    mimeType: "image/png",
    buffer: Buffer.from(data, "base64"),
  };
}

test.beforeEach(async ({ page }) => {
  await page.goto("/?no-sw");
  await expect(page.locator("#engine-status")).toContainText("READY");
  await openCameraEditor(page);
});

test("F04: replacement decode invalidates old guides and prevents old capture/apply", async ({
  page,
}) => {
  await upload(page);
  await page.locator("#camera-capture").click();
  await upload(page, "B");
  await page.locator("#camera-capture").click();
  await expect(page.locator("#camera-apply")).toBeEnabled();
  await page.locator("#camera-view-a").click();
  await holdNextDecode(page);
  await page
    .locator("#camera-file-a")
    .setInputFiles(getTestImagePath("scrambled-1", "A"));
  await page.waitForFunction(() => !!(window as any).releaseDecode);
  await expect(page.locator("#camera-progress")).toHaveText("3 / 6 面");
  await expect(page.locator("#camera-capture")).toBeDisabled();
  await expect(page.locator("#camera-apply")).toBeDisabled();
  expect(
    await page.evaluate(() => (window as any).__lastCamera.points.length),
  ).toBe(0);
  // Defense in depth: a stale queued handler must not bypass the UI guard.
  await page.evaluate(() => {
    (window as any).__lastCamera.capture();
    (document.querySelector("#camera-apply") as HTMLButtonElement).onclick!(
      new MouseEvent("click"),
    );
  });
  await expect(page.locator("#camera-progress")).toHaveText("3 / 6 面");
  await expect(page.locator("#camera-editor")).toBeVisible();
  await page.evaluate(() => (window as any).releaseDecode());
  await expect(page.locator("#camera-capture")).toBeEnabled();
  await page.locator("#camera-capture").click();
  await expect(page.locator("#camera-apply")).toBeEnabled();
});

test("F05: static-to-live blocks old-photo sampling and editing until stopped", async ({
  page,
}) => {
  await upload(page);
  const guides = await page.evaluate(() => (window as any).__lastCamera.points);
  await page.locator("#camera-live-stream").click();
  await expect(page.locator("#camera-take-photo")).toBeVisible();
  await expect(page.locator("#camera-capture")).toBeDisabled();
  await expect(page.locator("#camera-detect")).toBeDisabled();
  await expect(page.locator("#camera-rotate-points")).toBeDisabled();
  await page.keyboard.press("r");
  await page.evaluate(() => (window as any).__lastCamera.capture());
  await expect(page.locator("#camera-progress")).toHaveText("0 / 6 面");
  expect(
    await page.evaluate(() => (window as any).__lastCamera.points),
  ).toEqual(guides);
  await page.locator("#camera-stop-stream").click();
  await expect(page.locator("#camera-capture")).toBeEnabled();
  await page.locator("#camera-capture").click();
  expect(await page.evaluate(() => (window as any).__lastCamera.faces)).toEqual(
    {
      U: "UUUUUUUUU",
      R: "RRRRRRRRR",
      F: "FFFFFFFFF",
    },
  );
});

test("F14: delayed JPEG preserves the later selected view and original storage target", async ({
  page,
}) => {
  await page.locator("#camera-live-stream").click();
  await expect(page.locator("#camera-take-photo")).toBeVisible();
  await page.evaluate(() => {
    const original = HTMLCanvasElement.prototype.toBlob;
    HTMLCanvasElement.prototype.toBlob = function (callback, type, quality) {
      HTMLCanvasElement.prototype.toBlob = original;
      original.call(
        this,
        (blob) => {
          let released = false;
          const release = () => {
            if (released) return;
            released = true;
            clearTimeout(timer);
            callback(blob);
          };
          const timer = setTimeout(release, 8000);
          (window as any).releasePhoto = release;
        },
        type,
        quality,
      );
    };
  });
  await page.locator("#camera-take-photo").click();
  await page.waitForFunction(() => !!(window as any).releasePhoto);
  await expect(page.locator("#camera-capture")).toBeDisabled();
  await page.locator("#camera-view-b").click();
  await page.evaluate(() => (window as any).releasePhoto());
  await expect(page.locator("#camera-status-a")).toContainText("読込完了");
  await expect(page.locator("#camera-view-b")).toHaveAttribute(
    "aria-selected",
    "true",
  );
  expect(
    await page.evaluate(() => ({
      a: !!(window as any).__lastCamera.imageA,
      b: !!(window as any).__lastCamera.imageB,
    })),
  ).toEqual({ a: true, b: false });
});

for (const resized of [false, true]) {
  test(`F15: ${resized ? "resized" : "raw"} decode failure remains visible and invalidates old photo`, async ({
    page,
  }) => {
    await upload(page);
    let file = {
      name: "broken.png",
      mimeType: "image/png",
      buffer: Buffer.from("bad image"),
    };
    if (resized) {
      file = await largePng(page, 1800, 10);
      await page.evaluate(() => {
        const original = URL.createObjectURL;
        URL.createObjectURL = (blob) => {
          if (!(blob instanceof File)) {
            URL.createObjectURL = original;
            return original(
              new Blob(["bad resized image"], { type: "image/jpeg" }),
            );
          }
          return original(blob);
        };
      });
    }
    await page.locator("#camera-file-a").setInputFiles(file);
    await expect(page.locator("#camera-error")).toContainText(
      "画像Aを読み込めませんでした",
    );
    await expect(page.locator("#camera-capture")).toBeDisabled();
    await expect(page.locator("#camera-apply")).toBeDisabled();
    await page.locator("#camera-view-b").click();
    await page.locator("#camera-view-a").click();
    await expect(page.locator("#camera-error")).toContainText(
      "画像Aを読み込めませんでした",
    );
    await upload(page);
    await expect(page.locator("#camera-error")).toBeEmpty();
  });
}

test("F16: real 4000x1 replacement resizes to nonzero pixels instead of retaining old photo", async ({
  page,
}) => {
  await upload(page);
  const file = await largePng(page, 4000, 1);
  await page.locator("#camera-file-a").setInputFiles(file);
  await expect(page.locator("#camera-status-a")).toContainText("1600×1");
  expect(
    await page.evaluate(() => {
      const image = (window as any).__lastCamera.imageA;
      return [image.naturalWidth, image.naturalHeight];
    }),
  ).toEqual([1600, 1]);
  await expect(page.locator("#camera-progress")).toHaveText("0 / 6 面");
  await expect(page.locator("#camera-apply")).toBeDisabled();
  await expect(page.locator("#camera-error")).toBeEmpty();
});

test("F16: current resize encoding failure is explicit and cannot revive old photo", async ({
  page,
}) => {
  await upload(page);
  const file = await largePng(page, 1800, 10);
  await page.evaluate(() => {
    const original = HTMLCanvasElement.prototype.toBlob;
    HTMLCanvasElement.prototype.toBlob = function (callback) {
      HTMLCanvasElement.prototype.toBlob = original;
      callback(null);
    };
  });
  await page.locator("#camera-file-a").setInputFiles(file);
  await expect(page.locator("#camera-error")).toContainText(
    "画像Aを読み込めませんでした",
  );
  await expect(page.locator("#camera-capture")).toBeDisabled();
  await expect(page.locator("#camera-apply")).toBeDisabled();
  expect(await page.evaluate(() => !!(window as any).__lastCamera.imageA)).toBe(
    false,
  );
});
