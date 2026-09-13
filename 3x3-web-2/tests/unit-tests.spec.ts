import { test, expect } from "@playwright/test";

test.describe("Web Modules Unit Tests", () => {
  test("model.ts - inverse and instruction", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const results = await page.evaluate(async () => {
      const model = await import("/web/model.ts");

      // inverse のテスト
      const invNormal = model.inverse("R");
      const invPrime = model.inverse("R'");
      const invDouble = model.inverse("R2");
      const invU = model.inverse("U");
      const invUPrime = model.inverse("U'");
      const invU2 = model.inverse("U2");

      // instruction のテスト
      const instR = model.instruction("R");
      const instRPrime = model.instruction("R'");
      const instR2 = model.instruction("R2");
      const instF = model.instruction("F");

      return {
        invNormal,
        invPrime,
        invDouble,
        invU,
        invUPrime,
        invU2,
        instR,
        instRPrime,
        instR2,
        instF,
        solvedLen: model.SOLVED.length,
        facesLen: model.FACES.length,
      };
    });

    expect(results.invNormal).toBe("R'");
    expect(results.invPrime).toBe("R");
    expect(results.invDouble).toBe("R2");
    expect(results.invU).toBe("U'");
    expect(results.invUPrime).toBe("U");
    expect(results.invU2).toBe("U2");

    expect(results.instR).toContain("右面");
    expect(results.instR).toContain("時計回りに90°");
    expect(results.instRPrime).toContain("反時計回りに90°");
    expect(results.instR2).toContain("180°");
    expect(results.instF).toContain("前面");

    expect(results.solvedLen).toBe(54);
    expect(results.facesLen).toBe(6);
  });

  test("model.ts - getCellArrowInfo calculation", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const arrowInfo = await page.evaluate(async () => {
      const model = await import("/web/model.ts");
      const solved = model.SOLVED;

      // 通常の完成状態
      const info0 = model.getCellArrowInfo(solved, [0, 0, 0, 0, 0, 0]);

      // センターを回転させた状態
      const pi2 = Math.PI / 2;
      const infoRotated = model.getCellArrowInfo(solved, [
        pi2,
        Math.PI,
        0,
        0,
        0,
        0,
      ]);

      // スクランブル状態
      const scrambled =
        "DRBUULUBRDBLDRLFLFBDFUFLDRRRBUBDFUDLBFRDLRLUBUFLUBRDFF";
      const infoScrambled = model.getCellArrowInfo(
        scrambled,
        [0, 0, 0, 0, 0, 0],
      );

      return {
        length0: info0.angles.length,
        colorsLength: info0.colors.length,
        kindsLength: info0.kinds.length,
        centerKind: info0.kinds[4], // U面センター
        cornerKind: info0.kinds[0], // U面コーナー
        edgeKind: info0.kinds[1], // U面エッジ
        rotatedAngle0: infoRotated.angles[4],
        scrambledLength: infoScrambled.angles.length,
      };
    });

    expect(arrowInfo.length0).toBe(54);
    expect(arrowInfo.colorsLength).toBe(54);
    expect(arrowInfo.kindsLength).toBe(54);
    expect(arrowInfo.centerKind).toBe("center");
    expect(arrowInfo.cornerKind).toBe("corner");
    expect(arrowInfo.edgeKind).toBe("edge");
    expect(arrowInfo.rotatedAngle0).toBeCloseTo(Math.PI / 2, 4);
    expect(arrowInfo.scrambledLength).toBe(54);
  });

  test("image-sampler.ts - buildState and sampleFace", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const samplerResults = await page.evaluate(async () => {
      const sampler = await import("/web/image-sampler.ts");

      // 1. buildState: 完全な面マップ
      const fullFaces = {
        U: "UUUUUUUUU",
        R: "RRRRRRRRR",
        F: "FFFFFFFFF",
        D: "DDDDDDDDD",
        L: "LLLLLLLLL",
        B: "BBBBBBBBB",
      };
      const fullState = sampler.buildState(fullFaces);

      // 2. buildState: 一部欠損（未指定面が '?????????' になる）
      const partialState = sampler.buildState({
        U: "UUUUUUUUU",
      });

      // 3. sampleFace: 4隅以外が渡された時のエラー
      let error4Points = "";
      try {
        sampler.sampleFace({} as any, [{ x: 0, y: 0 }]);
      } catch (e: any) {
        error4Points = e.message;
      }

      // 4. sampleFace: Canvas要素によるサンプリングテスト
      const canvas = document.createElement("canvas");
      canvas.width = 100;
      canvas.height = 100;
      const ctx = canvas.getContext("2d")!;
      // 白（U）で塗りつぶす
      ctx.fillStyle = "#eeeade";
      ctx.fillRect(0, 0, 100, 100);

      // HTMLImageElement を作成
      const img = new Image();
      img.src = canvas.toDataURL();
      await new Promise((resolve) => {
        img.onload = resolve;
      });

      const sampled = sampler.sampleFace(img, [
        { x: 0, y: 0 },
        { x: 100, y: 0 },
        { x: 100, y: 100 },
        { x: 0, y: 100 },
      ]);

      return {
        fullState,
        partialState,
        error4Points,
        sampled,
      };
    });

    expect(samplerResults.fullState).toBe(
      "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB",
    );
    expect(samplerResults.partialState).toBe("UUUUUUUUU" + "?".repeat(45));
    expect(samplerResults.error4Points).toBe("面の四隅を4点指定してください。");
    expect(samplerResults.sampled).toBe("UUUUUUUUU");
  });

  test("centers.ts - centerTurns, rotateCenters, automaticCenters, centersFromInput", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const centerResults = await page.evaluate(async () => {
      const centers = await import("/web/centers.ts");
      const pi2 = Math.PI / 2;

      // 1. centerTurns
      const turns = centers.centerTurns([0, pi2, Math.PI, 3 * pi2, -pi2]);

      // 2. rotateCenters
      const rotated = centers.rotateCenters(
        [0, 0, 0, 0, 0, 0],
        ["U", "R'", "F2", "D"],
      );
      const rotatedTurns = centers.centerTurns(rotated);

      // 3. automaticCenters
      const solved = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
      const autoSolved = centers.automaticCenters(solved);

      // 4. centersFromInput: undefined のとき automaticCenters に委譲
      const fromUndefined = centers.centersFromInput(solved, undefined);

      // 5. centersFromInput: 正常値 [0, 0, 0, 0, 0, 0]
      const fromValid = centers.centersFromInput(solved, [0, 0, 0, 0, 0, 0]);

      // 6. centersFromInput: エラーケース（配列の長さ不正、値不正、パリティ不整合）
      let errInvalidLength = "";
      try {
        centers.centersFromInput(solved, [0, 1]);
      } catch (e: any) {
        errInvalidLength = e.message;
      }

      let errNonArray = "";
      try {
        centers.centersFromInput(solved, "invalid");
      } catch (e: any) {
        errNonArray = e.message;
      }

      let errParity = "";
      try {
        // 完成状態はパリティ0なので合計1は不整合
        centers.centersFromInput(solved, [1, 0, 0, 0, 0, 0]);
      } catch (e: any) {
        errParity = e.message;
      }

      return {
        turns,
        rotatedTurns,
        autoSolvedLen: autoSolved.length,
        fromUndefinedLen: fromUndefined.length,
        fromValidLen: fromValid.length,
        errInvalidLength,
        errNonArray,
        errParity,
      };
    });

    expect(centerResults.turns).toEqual([0, 1, 2, 3, 3]);
    expect(centerResults.rotatedTurns[0]).toBe(1); // U = +1
    expect(centerResults.rotatedTurns[1]).toBe(3); // R' = +3
    expect(centerResults.rotatedTurns[2]).toBe(2); // F2 = +2
    expect(centerResults.rotatedTurns[3]).toBe(1); // D = +1

    expect(centerResults.autoSolvedLen).toBe(6);
    expect(centerResults.fromUndefinedLen).toBe(6);
    expect(centerResults.fromValidLen).toBe(6);

    expect(centerResults.errInvalidLength).toContain("0°・90°・180°・270°");
    expect(centerResults.errNonArray).toContain("0°・90°・180°・270°");
    expect(centerResults.errParity).toContain("整合しません");
  });

  test("camera.ts - computeCenter and detectCubeOutline unit functions", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const cameraResults = await page.evaluate(async () => {
      const camera = await import("/web/camera.ts");

      // 1. computeCenter の正常系（アイソメトリック外周6点）
      const hexPoints = [
        { x: 320, y: 80 }, // P1: てっぺん
        { x: 459, y: 160 }, // P2: 右上
        { x: 459, y: 320 }, // P3: 右下
        { x: 320, y: 400 }, // P4: 底
        { x: 181, y: 320 }, // P5: 左下
        { x: 181, y: 160 }, // P6: 左上
      ];
      const center = camera.computeCenter(hexPoints);

      // 2. computeCenter の退化・平行エッジケース（denom < 1e-6）
      const parallelPoints = [
        { x: 100, y: 100 },
        { x: 200, y: 100 },
        { x: 200, y: 200 },
        { x: 100, y: 200 },
        { x: 100, y: 100 },
        { x: 200, y: 100 },
      ];
      const fallbackCenter = camera.computeCenter(parallelPoints);

      // 3. detectCubeOutline: Canvas からの検出
      const canvas = document.createElement("canvas");
      canvas.width = 640;
      canvas.height = 480;
      const ctx = canvas.getContext("2d")!;
      ctx.fillStyle = "#ffffff";
      ctx.fillRect(0, 0, 640, 480);

      // 中央にキューブ風の六角形を描画
      ctx.fillStyle = "#000000";
      ctx.beginPath();
      ctx.moveTo(320, 80);
      ctx.lineTo(459, 160);
      ctx.lineTo(459, 320);
      ctx.lineTo(320, 400);
      ctx.lineTo(181, 320);
      ctx.lineTo(181, 160);
      ctx.closePath();
      ctx.fill();

      const img = new Image();
      img.src = canvas.toDataURL();
      await new Promise((resolve) => {
        img.onload = resolve;
      });

      const detected = camera.detectCubeOutline(canvas, img);

      return {
        centerX: Math.round(center.x),
        centerY: Math.round(center.y),
        fallbackCenterX: Math.round(fallbackCenter.x),
        detectedLength: detected.length,
        detectedP1X: detected[0].x,
      };
    });

    expect(cameraResults.centerX).toBe(320);
    expect(cameraResults.centerY).toBe(240);
    expect(cameraResults.fallbackCenterX).toBeGreaterThan(0);
    expect(cameraResults.detectedLength).toBe(6);
    expect(cameraResults.detectedP1X).toBe(320);
  });

  test("view.ts - icon and net rendering", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const viewResults = await page.evaluate(async () => {
      const view = await import("/web/view.ts");

      // 1. icon
      const cubeIcon = view.icon("cube");
      const shuffleIcon = view.icon("shuffle");
      const unknownIcon = view.icon("non_existent_icon"); // fallback to cube

      // 2. net
      const host = document.createElement("div");
      document.body.appendChild(host);

      let clickedIndex = -1;
      view.net(
        host,
        "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB",
        true,
        (idx) => {
          clickedIndex = idx;
        },
        0, // active face
        [0, 1, 2, 3, 0, 1], // centers
      );

      const stickerButtons = host.querySelectorAll("button.sticker");
      const totalButtons = stickerButtons.length;

      // センター以外のステッカーをクリック
      const firstEditableButton = Array.from(stickerButtons).find(
        (b) => !(b as HTMLButtonElement).disabled,
      ) as HTMLButtonElement;

      if (firstEditableButton) {
        firstEditableButton.click();
      }

      const clicked = clickedIndex;

      // 非エディタブルモードでnet再描画
      view.net(
        host,
        "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB",
        false,
      );
      const spanStickers = host.querySelectorAll("span.sticker").length;

      document.body.removeChild(host);

      return {
        hasCubeIcon: cubeIcon.includes("<svg"),
        hasShuffleIcon: shuffleIcon.includes("<svg"),
        hasFallbackIcon: unknownIcon.includes("<svg"),
        totalButtons,
        clicked,
        spanStickers,
      };
    });

    expect(viewResults.hasCubeIcon).toBe(true);
    expect(viewResults.hasShuffleIcon).toBe(true);
    expect(viewResults.hasFallbackIcon).toBe(true);
    expect(viewResults.totalButtons).toBe(54); // 54 buttons total (6 disabled centers)
    expect(viewResults.clicked).toBeGreaterThanOrEqual(0);
    expect(viewResults.spanStickers).toBe(54);
  });
});
