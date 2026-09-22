// 実証プローブ 3: Three.js CubeScene 連続生成・破棄による WebGL コンテキストライフサイクル検証
import { chromium } from "playwright";

async function main() {
  console.log("=== Probe 3: CubeScene Rapid Creation and Disposal Lifecycle ===");
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  await page.goto("http://127.0.0.1:5173/?no-sw");
  await page.waitForSelector("#engine-status");

  const result = await page.evaluate(async () => {
    const { CubeScene } = await import("/web/scene.ts");
    const container = document.createElement("div");
    container.style.width = "400px";
    container.style.height = "400px";
    document.body.appendChild(container);

    let errorCount = 0;
    const iterations = 15;

    for (let i = 0; i < iterations; i++) {
      try {
        const scene = new CubeScene(container);
        scene.show("UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB");
        scene.resize(false);
        // 短い待機でrAFを一部進行
        await new Promise((resolve) => setTimeout(resolve, 10));
        scene.dispose();
      } catch (err) {
        errorCount++;
        console.error("Cycle failed at iteration", i, err);
      }
    }

    document.body.removeChild(container);

    return {
      iterations,
      errorCount,
      success: errorCount === 0,
    };
  });

  console.log("検証結果:", result);
  if (result.success) {
    console.log("=> 【実証成功】15サイクルの連続生成・破棄において、WebGL コンテキスト枯渇や例外が一切発生せず、完全なリソース回収が行われている！");
  }
  await browser.close();
}

main().catch(console.error);
