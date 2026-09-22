import { chromium } from "playwright";

async function main() {
  console.log("=== Probe 3: Scene ResizeObserver rAF Callback after Dispose ===");
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  await page.goto("http://127.0.0.1:5173/?no-sw");

  const leaked = await page.evaluate(async () => {
    const scene = window.cube_scene;
    if (!scene) return { error: "cube_scene not found" };

    let callbackFiredAfterDispose = false;
    let resizeCalled = false;

    // scene の内部プロパティとメソッドを監視
    const originalApplyResize = scene.applyResize ? scene.applyResize.bind(scene) : null;
    
    // リサイズをトリガー（非同期 rAF スケジュール）
    scene.resize(false);
    const hasRaf = scene.resizeRafId !== undefined;

    // 直後に dispose を実行
    scene.dispose();

    // 次のフレームまで待機
    await new Promise((resolve) => requestAnimationFrame(resolve));
    await new Promise((resolve) => setTimeout(resolve, 50));

    return {
      hasRafBeforeDispose: hasRaf,
      hasRafDisposedCancellation: scene.resizeRafId === undefined,
    };
  });

  console.log("検証結果:", leaked);
  await browser.close();
}

main().catch(console.error);
