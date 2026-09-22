import { test, expect, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";
import { getTestImagePath, openCameraEditor } from "./test-utils";

async function loadImage(page: Page) {
  await page
    .locator("#camera-file-a")
    .setInputFiles(getTestImagePath("solved", "A"));
  await page.waitForFunction(
    () => (window as any).__lastCamera?.points.length === 6,
  );
}

async function guides(page: Page) {
  return page.evaluate(() => {
    const camera = (window as any).__lastCamera;
    return { points: camera.points, center: camera.centerPoint };
  });
}

async function dragGuide(page: Page, index: number, dx: number, dy: number) {
  const point = await page.evaluate(async (index) => {
    const camera = (window as any).__lastCamera;
    const { computeCenter } = await import("/web/camera-geometry.ts");
    const p = index === 6 ? computeCenter(camera.points) : camera.points[index];
    const canvas = document.querySelector<HTMLCanvasElement>("#camera-canvas")!;
    const rect = canvas.getBoundingClientRect();
    const scale = Math.min(
      rect.width / canvas.width,
      rect.height / canvas.height,
    );
    return {
      x: rect.x + (rect.width - canvas.width * scale) / 2 + p.x * scale,
      y: rect.y + (rect.height - canvas.height * scale) / 2 + p.y * scale,
    };
  }, index);
  await page.mouse.move(point.x, point.y);
  await page.mouse.down();
  await page.mouse.move(point.x + dx, point.y + dy, { steps: 4 });
  await page.mouse.up();
}

test.beforeEach(async ({ page }) => {
  await page.goto("/?no-sw");
  await expect(page.locator("#engine-status")).toContainText("READY");
  await openCameraEditor(page);
});

for (const target of ["file", "card-drop", "canvas-drop"] as const) {
  test(`selecting an image via ${target} stops live video and displays that image`, async ({
    page,
  }) => {
    await page.locator("#camera-live-stream").click();
    await page.waitForFunction(() => (window as any).__lastCamera?.isStreaming);
    await page.evaluate(() => {
      (window as any).previousStream = (
        document.querySelector("#camera-video") as HTMLVideoElement
      ).srcObject;
    });

    if (target === "file") {
      await loadImage(page);
    } else {
      const bytes = [...readFileSync(getTestImagePath("solved", "A"))];
      await page
        .locator(target === "card-drop" ? "#camera-drop-a" : "#camera-canvas")
        .evaluate((element, bytes) => {
          const dataTransfer = new DataTransfer();
          dataTransfer.items.add(
            new File([new Uint8Array(bytes)], "solved.png", {
              type: "image/png",
            }),
          );
          element.dispatchEvent(
            new DragEvent("drop", { bubbles: true, dataTransfer }),
          );
        }, bytes);
      await page.waitForFunction(
        () => (window as any).__lastCamera?.points.length === 6,
      );
    }

    const result = await page.evaluate(async () => {
      await new Promise<void>((resolve) =>
        requestAnimationFrame(() => requestAnimationFrame(() => resolve())),
      );
      const camera = (window as any).__lastCamera;
      const canvas =
        document.querySelector<HTMLCanvasElement>("#camera-canvas")!;
      const raw = document.createElement("canvas");
      raw.width = canvas.width;
      raw.height = canvas.height;
      raw
        .getContext("2d")!
        .drawImage(camera.imageA, 0, 0, raw.width, raw.height);
      // The corner is outside the guides, so it must still show the selected image.
      return {
        streaming: camera.isStreaming,
        tracks: (window as any).previousStream
          .getTracks()
          .map((track: MediaStreamTrack) => track.readyState),
        displayedPixel: [
          ...canvas.getContext("2d")!.getImageData(5, 5, 1, 1).data,
        ],
        imagePixel: [...raw.getContext("2d")!.getImageData(5, 5, 1, 1).data],
      };
    });
    expect(result.streaming).toBe(false);
    expect(result.tracks).toEqual(["ended"]);
    expect(result.displayedPixel).toEqual(result.imagePixel);
    await expect(page.locator("#camera-take-photo")).toBeHidden();
    await expect(page.locator("#camera-live-stream")).toBeVisible();
    await page.locator("#camera-capture").click();
    expect(
      await page.evaluate(() => (window as any).__lastCamera.faces),
    ).toEqual({
      U: "UUUUUUUUU",
      R: "RRRRRRRRR",
      F: "FFFFFFFFF",
    });
  });
}

test("selecting an image cancels a pending camera start", async ({ page }) => {
  await page.evaluate(() => {
    const original = navigator.mediaDevices.getUserMedia.bind(
      navigator.mediaDevices,
    );
    navigator.mediaDevices.getUserMedia = async (constraints) => {
      const stream = await original(constraints);
      (window as any).pendingStream = stream;
      return new Promise<MediaStream>((resolve) => {
        (window as any).releaseStream = () => resolve(stream);
      });
    };
    (window as any).startingCamera = (
      window as any
    ).__lastCamera.startLiveStream();
  });
  await page.waitForFunction(() => !!(window as any).releaseStream);
  await loadImage(page);
  const result = await page.evaluate(async () => {
    (window as any).releaseStream();
    await (window as any).startingCamera;
    return {
      streaming: (window as any).__lastCamera.isStreaming,
      tracks: (window as any).pendingStream
        .getTracks()
        .map((track: MediaStreamTrack) => track.readyState),
    };
  });
  expect(result).toEqual({ streaming: false, tracks: ["ended"] });
  await expect(page.locator("#camera-live-stream")).toBeEnabled();
});

test("outline detection ignores previously drawn and manually moved guides", async ({
  page,
}) => {
  await loadImage(page);
  const initial = await guides(page);
  for (let i = 0; i < 5; i++) {
    await page.locator("#camera-detect").click();
    expect(await guides(page)).toEqual(initial);
  }
  await dragGuide(page, 0, 0, -25);
  expect(await guides(page)).not.toEqual(initial);
  await page.locator("#camera-detect").click();
  expect(await guides(page)).toEqual(initial);
});

test("reselecting the active image preserves manually adjusted vertices and center", async ({
  page,
}) => {
  await loadImage(page);
  const initial = await guides(page);
  await dragGuide(page, 0, 0, -25);
  await dragGuide(page, 6, 15, 15);
  const adjusted = await guides(page);
  expect(adjusted.points).not.toEqual(initial.points);
  expect(adjusted.center).toBeDefined();
  await page.locator("#camera-view-a").click();
  expect(await guides(page)).toEqual(adjusted);
  await page.locator("#camera-clear-points").click();
  expect((await guides(page)).points).toEqual([]);
  await page.locator("#camera-detect").click();
  expect(await guides(page)).toEqual(initial);
});
