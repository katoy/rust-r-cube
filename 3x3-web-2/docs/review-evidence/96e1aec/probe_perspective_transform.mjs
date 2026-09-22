// 実証プローブ 2: 画像サンプリングにおける透視射影変換 (Perspective Transform) の特異点検証
import { chromium } from "playwright";

async function main() {
  console.log("=== Probe 2: Perspective Transform Degeneracy and Robustness ===");
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  await page.goto("http://127.0.0.1:5173/?no-sw");
  await page.waitForSelector("#engine-status");

  const result = await page.evaluate(async () => {
    const sampler = await import("/web/image-sampler.ts");

    // 1. 正常な正方形
    const normalQuad = [
      { x: 0, y: 0 },
      { x: 100, y: 0 },
      { x: 100, y: 100 },
      { x: 0, y: 100 },
    ];
    const transform = sampler.getPerspectiveTransform(normalQuad);
    const centerPoint = transform(0.5, 0.5);

    // 2. 退化四角形 (4点が一直線上に並ぶ - 外積チェックで検出)
    const collinearQuad = [
      { x: 0, y: 0 },
      { x: 50, y: 0 },
      { x: 100, y: 0 },
      { x: 150, y: 0 },
    ];
    let collinearThrew = false;
    try {
      sampler.getPerspectiveTransform(collinearQuad);
    } catch (e) {
      collinearThrew = true;
    }

    // 3. 三角形（2点が同一座標）
    const triangleQuad = [
      { x: 0, y: 0 },
      { x: 100, y: 0 },
      { x: 50, y: 100 },
      { x: 50, y: 100 },
    ];
    let triangleThrew = false;
    try {
      sampler.getPerspectiveTransform(triangleQuad);
    } catch (e) {
      triangleThrew = true;
    }

    return {
      normalTransformValid:
        typeof transform === "function" &&
        Math.abs(centerPoint.x - 50) < 1e-3 &&
        Math.abs(centerPoint.y - 50) < 1e-3,
      collinearSafelyCaught: collinearThrew,
      triangleSafelyCaught: triangleThrew,
    };
  });

  console.log("検証結果:", result);
  if (result.normalTransformValid && result.collinearSafelyCaught && result.triangleSafelyCaught) {
    console.log("=> 【実証成功】正常系では正確な射影写像関数を生成し、退化四角形や同一点列に対しては数学的例外を安全に送出している！");
  }
  await browser.close();
}

main().catch(console.error);
