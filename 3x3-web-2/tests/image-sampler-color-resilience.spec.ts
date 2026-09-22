import { test, expect } from "@playwright/test";

test.describe("Image Sampler Color Classification Resilience", () => {
  test("O02: accurately classifies colors under shadows and low lighting conditions", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const result = await page.evaluate(async () => {
      const { classifyColor } = await import("/web/image-sampler.ts");

      // 1. 影のかかった濃紺ステッカー (低明度・高色相純度)
      const shadowedBlue = classifyColor(25, 45, 95);

      // 2. 影のかかった深紅ステッカー
      const shadowedRed = classifyColor(95, 25, 25);

      // 3. 影のかかった深緑ステッカー
      const shadowedGreen = classifyColor(25, 80, 30);

      // 4. 薄暗い白色ステッカー
      const dimWhite = classifyColor(130, 130, 125);

      // 5. プラスチック目地ノイズ (完全な黒)
      const seamBlack = classifyColor(20, 20, 20);

      // 6. 鮮明な各色
      const brightYellow = classifyColor(240, 220, 20);
      const brightOrange = classifyColor(245, 130, 20);

      return {
        shadowedBlue,
        shadowedRed,
        shadowedGreen,
        dimWhite,
        seamBlack,
        brightYellow,
        brightOrange,
      };
    });

    expect(result.shadowedBlue).toBe("B");
    expect(result.shadowedRed).toBe("R");
    expect(result.shadowedGreen).toBe("F");
    expect(result.dimWhite).toBe("U");
    expect(result.seamBlack).toBe("?");
    expect(result.brightYellow).toBe("D");
    expect(result.brightOrange).toBe("L");
  });
});
