import { test, expect } from "@playwright/test";

test.describe("R08: Image Sampler Perspective Projection", () => {
  test("sampleFace correctly uses perspective projection instead of bilinear interpolation", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const result = await page.evaluate(async () => {
      const sampler = await import("/web/image-sampler.ts");

      // 四隅: (50,50), (350,50), (250,350), (130,350)
      const points = [
        { x: 50, y: 50 },
        { x: 350, y: 50 },
        { x: 250, y: 350 },
        { x: 130, y: 350 },
      ];

      // 4点射影変換によるテスト画像を Canvas に直接描画
      // [0, 1]x[0, 1] の正方形座標 (u, v) から四角形 (x, y) への射影行列 H
      // x(u, v) = (300*u + 275*v + 50) / (1 + 1.5*v)
      // y(u, v) = (825*v + 50) / (1 + 1.5*v)
      //
      // 逆変換: (x, y) から (u, v)
      // y*(1 + 1.5*v) = 825*v + 50 => v*(825 - 1.5*y) = y - 50 => v = (y - 50) / (825 - 1.5*y)
      // x*(1 + 1.5*v) = 300*u + 275*v + 50 => u = (x*(1 + 1.5*v) - 275*v - 50) / 300
      const canvas = document.createElement("canvas");
      canvas.width = 400;
      canvas.height = 400;
      const ctx = canvas.getContext("2d")!;
      ctx.fillStyle = "#000000";
      ctx.fillRect(0, 0, 400, 400);

      const imgData = ctx.createImageData(400, 400);

      // 色の定義:
      // Row 0 (v in [0, 1/3)): Red (R) rgb(200, 0, 0)
      // Row 1 (v in [1/3, 2/3)): Green (F) rgb(0, 180, 0)
      // Row 2 (v in [2/3, 1])): Blue (B) rgb(0, 0, 200)
      for (let py = 0; py < 400; py++) {
        const denomV = 825 - 1.5 * py;
        if (Math.abs(denomV) < 1e-5) continue;
        const v = (py - 50) / denomV;
        if (v < 0 || v > 1) continue;

        const w = 1 + 1.5 * v;
        for (let px = 0; px < 400; px++) {
          const u = (px * w - 275 * v - 50) / 300;
          if (u < 0 || u > 1) continue;

          let r = 0,
            g = 0,
            b = 0;
          if (v < 1 / 3) {
            r = 200; // Red
          } else if (v < 2 / 3) {
            g = 180; // Green
          } else {
            b = 200; // Blue
          }

          const idx = (py * 400 + px) * 4;
          imgData.data[idx] = r;
          imgData.data[idx + 1] = g;
          imgData.data[idx + 2] = b;
          imgData.data[idx + 3] = 255;
        }
      }
      ctx.putImageData(imgData, 0, 0);

      const img = new Image();
      img.src = canvas.toDataURL();
      await new Promise((resolve) => {
        img.onload = resolve;
      });

      const sampled = sampler.sampleFace(img, points);
      return sampled;
    });

    // 期待値: 3行が RRR / FFF / BBB となること
    // 双線形補間だと RRRRRRBBB になってしまう
    expect(result).toBe("RRRFFFBBB");
  });

  test("sampleFace rejects degenerate or non-convex quadrilaterals", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const errors = await page.evaluate(async () => {
      const sampler = await import("/web/image-sampler.ts");
      const img = new Image();
      img.src =
        "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
      await new Promise((resolve) => {
        img.onload = resolve;
      });

      const errs: string[] = [];
      // 1. 同一直線（共線）
      try {
        sampler.sampleFace(img, [
          { x: 0, y: 0 },
          { x: 50, y: 50 },
          { x: 100, y: 100 },
          { x: 150, y: 150 },
        ]);
      } catch (e: any) {
        errs.push(e.message);
      }

      // 2. 自己交差（頂点のねじれ）
      try {
        sampler.sampleFace(img, [
          { x: 0, y: 0 },
          { x: 100, y: 0 },
          { x: 0, y: 100 },
          { x: 100, y: 100 },
        ]);
      } catch (e: any) {
        errs.push(e.message);
      }

      return errs;
    });

    expect(errors.length).toBe(2);
    expect(errors[0]).toContain("有効な四角形");
    expect(errors[1]).toContain("有効な四角形");
  });
});
