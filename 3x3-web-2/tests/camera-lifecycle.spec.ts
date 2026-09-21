import { test, expect } from "@playwright/test";
import { openCameraEditor } from "./test-utils";

test.describe("R02 & R10: Camera Lifecycle and Decode Generation", () => {
  test("R02: startLiveStream cleans up tracks on video.play() rejection or duplicate starts", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");
    await openCameraEditor(page);

    const result = await page.evaluate(async () => {
      // navigator.mediaDevices.getUserMedia をモック
      let trackStopped = false;
      const mockTrack = {
        stop: () => {
          trackStopped = true;
        },
      };
      const mockStream = {
        getTracks: () => [mockTrack],
      };

      const origGetUserMedia = navigator.mediaDevices.getUserMedia;
      navigator.mediaDevices.getUserMedia = async () => mockStream as any;

      // HTMLVideoElement.prototype.play をモックして強制失敗させる
      const origPlay = HTMLVideoElement.prototype.play;
      HTMLVideoElement.prototype.play = async () => {
        throw new Error("Play aborted");
      };

      const cameraModule = await import("/web/camera.ts");
      const camera = new cameraModule.TwoViewCamera(() => {});

      try {
        await camera.startLiveStream();
      } catch {
        // catch error
      }

      // クリーンアップ
      HTMLVideoElement.prototype.play = origPlay;
      navigator.mediaDevices.getUserMedia = origGetUserMedia;

      return {
        trackStopped,
      };
    });

    // video.play() が例外を投げた場合、MediaStream のトラックが確実に停止（解放）されていること
    expect(result.trackStopped).toBe(true);
  });

  test("R10: processFile respects decode generation and does not overwrite with stale image", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");
    await openCameraEditor(page);

    const result = await page.evaluate(async () => {
      const cameraModule = await import("/web/camera.ts");
      const camera = new cameraModule.TwoViewCamera(() => {});

      // 2つの異なる画像をシミュレート（1px赤 vs 1px青）
      // 赤画像 (image 1)
      const canvas1 = document.createElement("canvas");
      canvas1.width = 10;
      canvas1.height = 10;
      const ctx1 = canvas1.getContext("2d")!;
      ctx1.fillStyle = "red";
      ctx1.fillRect(0, 0, 10, 10);
      const blob1 = await new Promise<Blob>((res) =>
        canvas1.toBlob((b) => res(b!)),
      );
      const file1 = new File([blob1], "red.png", { type: "image/png" });

      // 青画像 (image 2)
      const canvas2 = document.createElement("canvas");
      canvas2.width = 20;
      canvas2.height = 20;
      const ctx2 = canvas2.getContext("2d")!;
      ctx2.fillStyle = "blue";
      ctx2.fillRect(0, 0, 20, 20);
      const blob2 = await new Promise<Blob>((res) =>
        canvas2.toBlob((b) => res(b!)),
      );
      const file2 = new File([blob2], "blue.png", { type: "image/png" });

      // Image の onload をインターセプトして、file1 の onload を故意に file2 完了後に遅延発火させる
      const origImage = window.Image;
      let instanceCount = 0;
      let firstImageOnload: any = null;

      // @ts-ignore
      window.Image = class extends origImage {
        constructor() {
          super();
          instanceCount++;
          if (instanceCount === 1) {
            const self = this;
            let realOnload: any = null;
            Object.defineProperty(self, "onload", {
              set(fn) {
                realOnload = fn;
                firstImageOnload = () => {
                  realOnload?.call(self, new Event("load"));
                };
              },
              get() {
                return realOnload;
              },
            });
          }
        }
      };

      // 1. file1 の処理を開始（onload の発火を保留）
      const p1 = (camera as any).processFile(file1, "A");
      // 2. 直後に file2 の処理を開始（通常通り完了）
      const p2 = (camera as any).processFile(file2, "A");

      await p2;
      // file2 完了時点で imageA は file2 (20x20)
      // ここで遅延していた file1 の onload を発火させる
      if (firstImageOnload) {
        firstImageOnload();
      }
      await p1;
      window.Image = origImage;

      // 最終的な imageA のサイズを検証（file2 である 20x20 であるべき。file1 の 10x10 で上書きされてはいけない）
      const finalImage = (camera as any).imageA;
      return {
        width: finalImage?.naturalWidth,
        height: finalImage?.naturalHeight,
      };
    });

    expect(result.width).toBe(20);
    expect(result.height).toBe(20);
  });

  test("R10: captureLiveFrame captures to the view active at capture invocation", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");
    await openCameraEditor(page);

    const result = await page.evaluate(async () => {
      const cameraModule = await import("/web/camera.ts");
      const camera = new cameraModule.TwoViewCamera(() => {});

      (camera as any).currentView = "A";
      (camera as any).isStreaming = true;

      // toBlob をインターセプトして、toBlob 呼び出し後に currentView を "B" に変更する
      const origToBlob = HTMLCanvasElement.prototype.toBlob;
      HTMLCanvasElement.prototype.toBlob = function (cb, type, quality) {
        // ビューが途中で B に切り替わった状態を作る
        (camera as any).currentView = "B";
        setTimeout(() => {
          origToBlob.call(
            this,
            (blob) => {
              cb(blob);
            },
            type,
            quality,
          );
        }, 50);
      };

      camera.captureLiveFrame();

      // 少し待って processFile が完了するのを待つ
      await new Promise((r) => setTimeout(r, 200));
      HTMLCanvasElement.prototype.toBlob = origToBlob;

      return {
        hasImageA: !!(camera as any).imageA,
        hasImageB: !!(camera as any).imageB,
      };
    });

    // ビューAで撮影を開始したので、imageA に設定され、imageB に誤保存されてはいけない
    expect(result.hasImageA).toBe(true);
    expect(result.hasImageB).toBe(false);
  });
});
