import { test, expect } from "@playwright/test";
import { openCameraEditor } from "./test-utils";

test.describe("R09 & R13: Camera Face Assignment and Palette Focus", () => {
  test("R09: unrecognized or mismatched center colors in view A do not overwrite view B faces", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");
    await openCameraEditor(page);

    const result = await page.evaluate(async () => {
      const cameraModule = await import("/web/camera.ts");
      const camera = new cameraModule.TwoViewCamera(() => {});

      // view A のダミー画像
      const canvasA = document.createElement("canvas");
      canvasA.width = 100;
      canvasA.height = 100;
      const ctxA = canvasA.getContext("2d")!;
      // 黄色（Dの色）で塗りつぶす（Uの位置なのにセンターがDになるケース）
      ctxA.fillStyle = "#ffd500";
      ctxA.fillRect(0, 0, 100, 100);
      const imgA = new Image();
      imgA.src = canvasA.toDataURL();
      await new Promise((r) => {
        imgA.onload = r;
      });

      // view A をセット
      (camera as any).currentView = "A";
      (camera as any).imageA = imgA;
      // 6点と中心を設定
      (camera as any).points = [
        { x: 50, y: 10 },
        { x: 90, y: 30 },
        { x: 90, y: 70 },
        { x: 50, y: 90 },
        { x: 10, y: 70 },
        { x: 10, y: 30 },
      ];
      (camera as any).centerPoint = { x: 50, y: 50 };

      // capture を実行
      (camera as any).capture();

      // view A 実行後の faces を記録
      const facesAfterA = { ...(camera as any).faces };

      // 次に view B のダミー画像（正常な白などで塗りつぶし）
      const canvasB = document.createElement("canvas");
      canvasB.width = 100;
      canvasB.height = 100;
      const ctxB = canvasB.getContext("2d")!;
      ctxB.fillStyle = "#ffffff";
      ctxB.fillRect(0, 0, 100, 100);
      const imgB = new Image();
      imgB.src = canvasB.toDataURL();
      await new Promise((r) => {
        imgB.onload = r;
      });

      (camera as any).currentView = "B";
      (camera as any).imageB = imgB;
      (camera as any).points = [
        { x: 50, y: 10 },
        { x: 90, y: 30 },
        { x: 90, y: 70 },
        { x: 50, y: 90 },
        { x: 10, y: 70 },
        { x: 10, y: 30 },
      ];
      (camera as any).centerPoint = { x: 50, y: 50 };

      // capture を実行
      (camera as any).capture();

      const facesAfterB = { ...(camera as any).faces };

      return {
        facesAfterA: Object.keys(facesAfterA).sort(),
        facesAfterB: Object.keys(facesAfterB).sort(),
        hasU: "U" in facesAfterB,
        hasD: "D" in facesAfterB,
        totalFaces: Object.keys(facesAfterB).length,
      };
    });

    // 画像A実行後、画像Aの担当面である U, R, F が割り当てられていること（D を奪っていないこと）
    expect(result.facesAfterA).toEqual(["F", "R", "U"]);
    // 画像B実行後、画像Aと画像Bの全6面 (B, D, F, L, R, U) が揃っていること
    expect(result.totalFaces).toBe(6);
    expect(result.hasU).toBe(true);
    expect(result.hasD).toBe(true);
  });

  test("R13: clicking or selecting palette buttons does not drop focus to body", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");
    await openCameraEditor(page);

    const paletteButtons = page.locator("#camera-palette .color-choice");
    const count = await paletteButtons.count();
    expect(count).toBeGreaterThan(0);

    // 2番目のボタン（赤）をクリック
    const redButton = paletteButtons.nth(1);
    await redButton.focus();
    await redButton.click();

    // クリック後にフォーカスが body に脱落していないこと
    const activeTagName = await page.evaluate(
      () => document.activeElement?.tagName,
    );
    expect(activeTagName).not.toBe("BODY");

    // フォーカスがパレットボタン（あるいは選択されたボタン）にあること
    const isInsidePalette = await page.evaluate(() => {
      const palette = document.getElementById("camera-palette");
      return (
        palette?.contains(document.activeElement) ||
        document.activeElement?.classList.contains("color-choice")
      );
    });
    expect(isInsidePalette).toBe(true);
  });
});
