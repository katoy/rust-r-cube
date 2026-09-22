import { test, expect } from "@playwright/test";

test.describe("CubeScene Lifecycle and Cleanup", () => {
  test("R05: cancels pending resize rAF when dispose() is called", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const result = await page.evaluate(async () => {
      const scene = (window as any).cube_scene;
      if (!scene) return { error: "cube_scene not found" };

      let applyResizeCalledAfterDispose = false;
      let isDisposed = false;

      const originalApplyResize = scene.applyResize.bind(scene);
      scene.applyResize = function () {
        if (isDisposed) {
          applyResizeCalledAfterDispose = true;
        }
        return originalApplyResize();
      };

      // デバウンスリサイズをスケジュール (rAF)
      scene.resize(false);

      // 直ちに破棄
      scene.dispose();
      isDisposed = true;

      // rAF が発火しうるタイミングまで待機
      await new Promise((resolve) => requestAnimationFrame(resolve));
      await new Promise((resolve) => setTimeout(resolve, 50));

      return {
        applyResizeCalledAfterDispose,
      };
    });

    expect(result.error).toBeUndefined();
    expect(result.applyResizeCalledAfterDispose).toBe(false);
  });
});
