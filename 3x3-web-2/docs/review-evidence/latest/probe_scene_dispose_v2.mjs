import { chromium } from "playwright";

async function main() {
  console.log("=== Probe 3: Scene ResizeObserver rAF Callback after Dispose ===");
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  await page.goto("http://127.0.0.1:5173/?no-sw");

  const result = await page.evaluate(async () => {
    const scene = window.cube_scene;
    if (!scene) return { error: "cube_scene not found" };

    let applyResizeCalledAfterDispose = false;
    let isDisposed = false;

    // applyResize にフックを仕掛ける
    const originalApplyResize = scene.applyResize.bind(scene);
    scene.applyResize = function() {
      if (isDisposed) {
        applyResizeCalledAfterDispose = true;
      }
      return originalApplyResize();
    };

    // リサイズをスケジュール（rAF）
    scene.resize(false);
    
    // 直後に破棄（dispose）
    scene.dispose();
    isDisposed = true;

    // rAF が実行されるまで待機
    await new Promise((resolve) => requestAnimationFrame(resolve));
    await new Promise((resolve) => setTimeout(resolve, 50));

    return {
      applyResizeCalledAfterDispose,
    };
  });

  console.log("検証結果:", result);
  if (result.applyResizeCalledAfterDispose) {
    console.log("=> 【実証成功】dispose() 後にキューされていた applyResize() が発火し、破棄済みインスタンスに対してリサイズ処理が遅延実行された！");
  }
  await browser.close();
}

main().catch(console.error);
