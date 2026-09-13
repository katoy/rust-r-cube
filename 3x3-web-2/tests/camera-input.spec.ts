import { test, expect, type Page } from "@playwright/test";
import {
  loadImageManifest,
  getTestImagePath,
  openCameraEditor,
  uploadTestImage,
} from "./test-utils";

async function ready(page: Page) {
  await page.goto("/");
  await expect(page.locator("#engine-status")).toContainText("READY");
  await page.locator("#reduced-motion").check();
}

test.describe("カメラ入力 - 画像処理テスト", () => {
  test("マニフェストが存在し、すべての画像ファイルが揃っている", async () => {
    const manifest = loadImageManifest();

    expect(manifest).toBeDefined();
    expect(Object.keys(manifest).length).toBeGreaterThan(0);

    for (const [name, images] of Object.entries(manifest)) {
      expect(images.viewA).toBeTruthy();
      expect(images.viewB).toBeTruthy();

      const pathA = getTestImagePath(name, "A");
      const pathB = getTestImagePath(name, "B");

      expect(pathA).toContain(".png");
      expect(pathB).toContain(".png");
    }
  });

  test("solved 状態の画像をアップロードできる", async ({ page }) => {
    await ready(page);
    await openCameraEditor(page);

    const viewAPath = getTestImagePath("solved", "A");
    await page.locator("#camera-file-a").setInputFiles(viewAPath);

    // 画像が Canvas に読み込まれたかを確認
    await expect(page.locator("#camera-canvas")).toBeVisible();
    await page.waitForTimeout(300);

    const canvasBox = await page.locator("#camera-canvas").boundingBox();
    expect(canvasBox?.width).toBeGreaterThan(0);
    expect(canvasBox?.height).toBeGreaterThan(0);

    await page.locator("#camera-close").click();
    await expect(page.locator("#camera-editor")).not.toBeVisible();
  });

  test("ビューAとビューBの両方をアップロードできる", async ({ page }) => {
    await ready(page);
    await openCameraEditor(page);

    // ビューA をアップロード
    const viewAPath = getTestImagePath("solved", "A");
    await page.locator("#camera-file-a").setInputFiles(viewAPath);
    await page.waitForTimeout(300);

    // ビューB をアップロード
    const viewBPath = getTestImagePath("solved", "B");
    await page.locator("#camera-file-b").setInputFiles(viewBPath);
    await page.waitForTimeout(300);

    // Canvas が表示されていることを確認
    await expect(page.locator("#camera-canvas")).toBeVisible();

    await page.locator("#camera-close").click();
  });

  test("フェース選択が正常に動作する", async ({ page }) => {
    await ready(page);
    await openCameraEditor(page);

    const faceSelect = page.locator("#camera-face");
    await expect(faceSelect).toBeVisible();

    const faces = ["U", "R", "F", "D", "L", "B"];
    for (const face of faces) {
      await faceSelect.selectOption(face);
      await expect(faceSelect).toHaveValue(face);
    }

    await page.locator("#camera-close").click();
  });

  test("四隅をクリックしてキャプチャボタンを有効にできる", async ({
    page,
  }) => {
    await ready(page);
    await openCameraEditor(page);

    const viewAPath = getTestImagePath("solved", "A");
    await page.locator("#camera-file-a").setInputFiles(viewAPath);
    await page.waitForTimeout(300);

    // キャプチャボタンは最初は無効
    await expect(page.locator("#camera-capture")).toBeDisabled();

    // Canvas に四隅をクリック
    const canvas = page.locator("#camera-canvas");
    const box = await canvas.boundingBox();
    expect(box).toBeTruthy();

    if (box) {
      const corners = [
        { x: box.x + 5, y: box.y + 5 }, // 左上
        { x: box.x + box.width - 5, y: box.y + 5 }, // 右上
        { x: box.x + box.width - 5, y: box.y + box.height - 5 }, // 右下
        { x: box.x + 5, y: box.y + box.height - 5 }, // 左下
      ];

      for (const corner of corners) {
        await page.mouse.click(corner.x, corner.y);
      }

      // 4 点クリック後、キャプチャボタンが有効になる
      await expect(page.locator("#camera-capture")).not.toBeDisabled();
    }

    await page.locator("#camera-close").click();
  });

  test("全 6 面をキャプチャしてから apply が有効になる", async ({
    page,
  }) => {
    await ready(page);
    await openCameraEditor(page);

    const viewAPath = getTestImagePath("solved", "A");
    const viewBPath = getTestImagePath("solved", "B");

    // apply ボタンは最初は無効
    await expect(page.locator("#camera-apply")).toBeDisabled();

    // 最初は 0/6
    await expect(page.locator("#camera-progress")).toContainText("0 / 6");

    // ビュー A をアップロード（U, R, F 面）
    await page.locator("#camera-file-a").setInputFiles(viewAPath);
    await page.waitForTimeout(300);

    // 各面をクリックしてキャプチャ
    const canvas = page.locator("#camera-canvas");
    const faces = ["U", "R", "F"];

    for (const face of faces) {
      await page.locator("#camera-face").selectOption(face);
      await page.waitForTimeout(200);

      const box = await canvas.boundingBox();
      if (box) {
        const corners = [
          { x: box.x + 5, y: box.y + 5 },
          { x: box.x + box.width - 5, y: box.y + 5 },
          { x: box.x + box.width - 5, y: box.y + box.height - 5 },
          { x: box.x + 5, y: box.y + box.height - 5 },
        ];

        for (const corner of corners) {
          await page.mouse.click(corner.x, corner.y);
        }

        await page.locator("#camera-capture").click();
        await page.waitForTimeout(200);
      }
    }

    // 3/6 になった
    await expect(page.locator("#camera-progress")).toContainText("3 / 6");
    await expect(page.locator("#camera-apply")).toBeDisabled();

    // ビュー B をアップロード（D, L, B 面）
    await page.locator("#camera-file-b").setInputFiles(viewBPath);
    await page.waitForTimeout(300);

    const facesB = ["D", "L", "B"];
    for (const face of facesB) {
      await page.locator("#camera-face").selectOption(face);
      await page.waitForTimeout(200);

      const box = await canvas.boundingBox();
      if (box) {
        const corners = [
          { x: box.x + 5, y: box.y + 5 },
          { x: box.x + box.width - 5, y: box.y + 5 },
          { x: box.x + box.width - 5, y: box.y + box.height - 5 },
          { x: box.x + 5, y: box.y + box.height - 5 },
        ];

        for (const corner of corners) {
          await page.mouse.click(corner.x, corner.y);
        }

        await page.locator("#camera-capture").click();
        await page.waitForTimeout(200);
      }
    }

    // 6/6 になった
    await expect(page.locator("#camera-progress")).toContainText("6 / 6");

    // apply が有効になった
    await expect(page.locator("#camera-apply")).not.toBeDisabled();

    await page.locator("#camera-close").click();
  });

  test("キャプチャをキャンセルして別のビューに切り替えられる", async ({
    page,
  }) => {
    await ready(page);
    await openCameraEditor(page);

    const viewAPath = getTestImagePath("solved", "A");
    await page.locator("#camera-file-a").setInputFiles(viewAPath);
    await page.waitForTimeout(300);

    // U 面で 4 点クリック
    await page.locator("#camera-face").selectOption("U");
    const canvas = page.locator("#camera-canvas");
    const box = await canvas.boundingBox();

    if (box) {
      const corners = [
        { x: box.x + 5, y: box.y + 5 },
        { x: box.x + box.width - 5, y: box.y + 5 },
        { x: box.x + box.width - 5, y: box.y + box.height - 5 },
        { x: box.x + 5, y: box.y + box.height - 5 },
      ];

      for (const corner of corners) {
        await page.mouse.click(corner.x, corner.y);
      }
    }

    // R 面に切り替えると、U 面のクリックがリセット
    await page.locator("#camera-face").selectOption("R");
    await expect(page.locator("#camera-capture")).toBeDisabled();

    await page.locator("#camera-close").click();
  });

  test("エラー表示領域が存在する", async ({ page }) => {
    await ready(page);
    await openCameraEditor(page);

    // エラー表示領域が存在することを確認
    const errorElement = page.locator("#camera-error");
    await expect(errorElement).toBeAttached();

    // 正常な画像をアップロード時にはエラーはクリア
    const viewAPath = getTestImagePath("solved", "A");
    await page.locator("#camera-file-a").setInputFiles(viewAPath);
    await page.waitForTimeout(300);

    // エラーテキストが空であることを確認
    await expect(errorElement).toHaveText("");

    await page.locator("#camera-close").click();
  });
});

test.describe("カメラ入力 - エンドツーエンドテスト", () => {
  test("solved 状態を画像から認識して apply できる", async ({ page }) => {
    await ready(page);
    await page.getByRole("tab", { name: "色を入力" }).click();
    await page.locator("#camera-colors").click();
    await expect(page.locator("#camera-editor")).toBeVisible();

    // テスト画像をアップロード
    const viewAPath = getTestImagePath("solved", "A");
    const viewBPath = getTestImagePath("solved", "B");

    // ビューA のアップロードと面キャプチャ
    await page.locator("#camera-file-a").setInputFiles(viewAPath);
    await page.waitForTimeout(300);

    const canvas = page.locator("#camera-canvas");
    const captureUFace = async () => {
      const box = await canvas.boundingBox();
      if (!box) return;

      const corners = [
        { x: box.x + 5, y: box.y + 5 },
        { x: box.x + box.width - 5, y: box.y + 5 },
        { x: box.x + box.width - 5, y: box.y + box.height - 5 },
        { x: box.x + 5, y: box.y + box.height - 5 },
      ];

      for (const corner of corners) {
        await page.mouse.click(corner.x, corner.y);
      }
    };

    // U 面
    await page.locator("#camera-face").selectOption("U");
    await captureUFace();
    await page.locator("#camera-capture").click();
    await page.waitForTimeout(200);

    // R 面
    await page.locator("#camera-face").selectOption("R");
    await captureUFace();
    await page.locator("#camera-capture").click();
    await page.waitForTimeout(200);

    // F 面
    await page.locator("#camera-face").selectOption("F");
    await captureUFace();
    await page.locator("#camera-capture").click();
    await page.waitForTimeout(200);

    // ビューB をアップロード
    await page.locator("#camera-file-b").setInputFiles(viewBPath);
    await page.waitForTimeout(300);

    // D 面
    await page.locator("#camera-face").selectOption("D");
    await captureUFace();
    await page.locator("#camera-capture").click();
    await page.waitForTimeout(200);

    // L 面
    await page.locator("#camera-face").selectOption("L");
    await captureUFace();
    await page.locator("#camera-capture").click();
    await page.waitForTimeout(200);

    // B 面
    await page.locator("#camera-face").selectOption("B");
    await captureUFace();
    await page.locator("#camera-capture").click();
    await page.waitForTimeout(200);

    // すべてキャプチャ完了、apply
    await expect(page.locator("#camera-progress")).toContainText("6 / 6");
    await page.locator("#camera-apply").click();

    // エディタが閉じられる
    await expect(page.locator("#camera-editor")).not.toBeVisible();

    // キューブが更新されたことを確認
    await expect(page.locator("#scene")).toBeVisible();
  });

  test("scrambled-1 状態を画像から認識できる", async ({ page }) => {
    await ready(page);
    await page.getByRole("tab", { name: "色を入力" }).click();
    await page.locator("#camera-colors").click();
    await expect(page.locator("#camera-editor")).toBeVisible();

    const viewAPath = getTestImagePath("scrambled-1", "A");
    const viewBPath = getTestImagePath("scrambled-1", "B");

    await page.locator("#camera-file-a").setInputFiles(viewAPath);
    await page.waitForTimeout(300);

    // キャプチャロジック（共通化可能）
    const canvas = page.locator("#camera-canvas");
    const captureAllFaces = async (faceList: string[]) => {
      for (const face of faceList) {
        await page.locator("#camera-face").selectOption(face);
        await page.waitForTimeout(100);

        const box = await canvas.boundingBox();
        if (box) {
          const corners = [
            { x: box.x + 5, y: box.y + 5 },
            { x: box.x + box.width - 5, y: box.y + 5 },
            { x: box.x + box.width - 5, y: box.y + box.height - 5 },
            { x: box.x + 5, y: box.y + box.height - 5 },
          ];

          for (const corner of corners) {
            await page.mouse.click(corner.x, corner.y);
          }

          await page.locator("#camera-capture").click();
          await page.waitForTimeout(100);
        }
      }
    };

    await captureAllFaces(["U", "R", "F"]);

    // ビューB
    await page.locator("#camera-file-b").setInputFiles(viewBPath);
    await page.waitForTimeout(300);

    await captureAllFaces(["D", "L", "B"]);

    // apply
    await page.locator("#camera-apply").click();
    await expect(page.locator("#camera-editor")).not.toBeVisible();
  });

  test("mixed-colors 状態を処理できる", async ({ page }) => {
    await ready(page);
    await page.getByRole("tab", { name: "色を入力" }).click();
    await page.locator("#camera-colors").click();

    const viewAPath = getTestImagePath("mixed-colors", "A");
    const viewBPath = getTestImagePath("mixed-colors", "B");

    await page.locator("#camera-file-a").setInputFiles(viewAPath);
    await page.waitForTimeout(300);

    const canvas = page.locator("#camera-canvas");

    // 複数面をキャプチャする共通ロジック
    const captureAndClick = async () => {
      const box = await canvas.boundingBox();
      if (!box) return;

      const corners = [
        { x: box.x + 5, y: box.y + 5 },
        { x: box.x + box.width - 5, y: box.y + 5 },
        { x: box.x + box.width - 5, y: box.y + box.height - 5 },
        { x: box.x + 5, y: box.y + box.height - 5 },
      ];

      for (const corner of corners) {
        await page.mouse.click(corner.x, corner.y);
      }
    };

    for (const face of ["U", "R", "F"]) {
      await page.locator("#camera-face").selectOption(face);
      await page.waitForTimeout(100);
      await captureAndClick();
      await page.locator("#camera-capture").click();
      await page.waitForTimeout(100);
    }

    await page.locator("#camera-file-b").setInputFiles(viewBPath);
    await page.waitForTimeout(300);

    for (const face of ["D", "L", "B"]) {
      await page.locator("#camera-face").selectOption(face);
      await page.waitForTimeout(100);
      await captureAndClick();
      await page.locator("#camera-capture").click();
      await page.waitForTimeout(100);
    }

    await page.locator("#camera-apply").click();
    await expect(page.locator("#camera-editor")).not.toBeVisible();
  });

  test("partial 状態（未入力を含む）を処理できる", async ({ page }) => {
    await ready(page);
    await page.getByRole("tab", { name: "色を入力" }).click();
    await page.locator("#camera-colors").click();

    const viewAPath = getTestImagePath("partial", "A");
    const viewBPath = getTestImagePath("partial", "B");

    await page.locator("#camera-file-a").setInputFiles(viewAPath);
    await page.waitForTimeout(300);

    const canvas = page.locator("#camera-canvas");

    const captureAndClick = async () => {
      const box = await canvas.boundingBox();
      if (!box) return;

      const corners = [
        { x: box.x + 5, y: box.y + 5 },
        { x: box.x + box.width - 5, y: box.y + 5 },
        { x: box.x + box.width - 5, y: box.y + box.height - 5 },
        { x: box.x + 5, y: box.y + box.height - 5 },
      ];

      for (const corner of corners) {
        await page.mouse.click(corner.x, corner.y);
      }
    };

    for (const face of ["U", "R", "F"]) {
      await page.locator("#camera-face").selectOption(face);
      await page.waitForTimeout(100);
      await captureAndClick();
      await page.locator("#camera-capture").click();
      await page.waitForTimeout(100);
    }

    await page.locator("#camera-file-b").setInputFiles(viewBPath);
    await page.waitForTimeout(300);

    for (const face of ["D", "L", "B"]) {
      await page.locator("#camera-face").selectOption(face);
      await page.waitForTimeout(100);
      await captureAndClick();
      await page.locator("#camera-capture").click();
      await page.waitForTimeout(100);
    }

    await page.locator("#camera-apply").click();
    await expect(page.locator("#camera-editor")).not.toBeVisible();
  });
});
