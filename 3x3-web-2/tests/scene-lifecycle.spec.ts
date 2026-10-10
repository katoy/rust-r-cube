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

  test("L18: setCubeType は再生中アニメーションを旧サイズのまま完了させてから種別を切り替える", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const calls = await page.evaluate(() => {
      const scene = (window as any).cube_scene;
      const solved3 = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
      const solved2 = "UUUURRRRFFFFDDDDLLLLBBBB";
      const log: { length: number; cubeType: string }[] = [];
      const originalShow = scene.show.bind(scene);
      scene.show = (state: string, next?: string) => {
        log.push({ length: state.length, cubeType: scene.getCubeType() });
        return originalShow(state, next);
      };
      // 長いアニメーションを開始し、完了前に 2x2 へ切り替える
      void scene.turn("R", solved3, 60_000);
      scene.setCubeType("2x2", solved2);
      return log;
    });

    expect(calls).toEqual([
      { length: 54, cubeType: "3x3" },
      { length: 24, cubeType: "2x2" },
    ]);
  });
});
