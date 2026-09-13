import { test, expect, type Page } from "@playwright/test";
import {
  loadImageManifest,
  loadFullManifest,
  getTestImagePath,
  getFaceCorners,
  getExpectedState,
  getEditorState,
  openCameraEditor,
  uploadTestImage,
  captureFaceOnTestImage,
} from "./test-utils";

async function ready(page: Page) {
  await page.goto("/");
  await expect(page.locator("#engine-status")).toContainText("READY");
  await page.locator("#reduced-motion").check();
}

test.describe("カメラ入力 - 画像処理テスト", () => {
  test("マニフェストが存在し、すべての画像ファイルと立体座標が揃っている", async () => {
    const fullManifest = loadFullManifest();

    expect(fullManifest).toBeDefined();
    expect(fullManifest.views.A.faces).toEqual(["U", "R", "F"]);
    expect(fullManifest.views.B.faces).toEqual(["D", "L", "B"]);

    for (const face of ["U", "R", "F"]) {
      const corners = fullManifest.views.A.corners[face];
      expect(corners).toHaveLength(4);
    }

    for (const face of ["D", "L", "B"]) {
      const corners = fullManifest.views.B.corners[face];
      expect(corners).toHaveLength(4);
    }

    const manifest = loadImageManifest();
    expect(Object.keys(manifest).length).toBeGreaterThan(0);

    for (const [name, images] of Object.entries(manifest)) {
      expect(images.viewA).toBeTruthy();
      expect(images.viewB).toBeTruthy();
      expect(images.expectedState).toHaveLength(54);

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

    const viewAPath = getTestImagePath("solved", "A");
    await page.locator("#camera-file-a").setInputFiles(viewAPath);
    await page.waitForTimeout(300);

    await expect(page.locator("#camera-status-a")).toContainText("読込完了");
    await expect(page.locator("#camera-drop-a")).toHaveClass(/has-file/);

    const viewBPath = getTestImagePath("solved", "B");
    await page.locator("#camera-file-b").setInputFiles(viewBPath);
    await page.waitForTimeout(300);

    await expect(page.locator("#camera-status-b")).toContainText("読込完了");
    await expect(page.locator("#camera-drop-b")).toHaveClass(/has-file/);
    await expect(page.locator("#camera-canvas")).toBeVisible();

    // ビュー切り替えボタンの検証
    await page.locator("#camera-view-a").click();
    await expect(page.locator("#camera-view-a")).toHaveAttribute(
      "aria-selected",
      "true",
    );
    await expect(page.locator("#camera-view-b")).toHaveAttribute(
      "aria-selected",
      "false",
    );

    await page.locator("#camera-view-b").click();
    await expect(page.locator("#camera-view-b")).toHaveAttribute(
      "aria-selected",
      "true",
    );
    await expect(page.locator("#camera-view-a")).toHaveAttribute(
      "aria-selected",
      "false",
    );

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

  test("四隅をクリックしてキャプチャボタンを有効にできる", async ({ page }) => {
    await ready(page);
    await openCameraEditor(page);

    const viewAPath = getTestImagePath("solved", "A");
    await page.locator("#camera-file-a").setInputFiles(viewAPath);
    await page.waitForTimeout(300);

    // キャプチャボタンは最初は無効
    await expect(page.locator("#camera-capture")).toBeDisabled();

    // U面の正確な4隅をクリック
    await captureFaceOnTestImage(page, "U");

    // 読み取り完了後、入力済み進捗が 1/6 面になる
    await expect(page.locator("#camera-progress")).toContainText("1 / 6");

    await page.locator("#camera-close").click();
  });

  test("全 6 面を正確な四隅指定でキャプチャしてから apply が有効になる", async ({
    page,
  }) => {
    await ready(page);
    await openCameraEditor(page);

    const viewAPath = getTestImagePath("solved", "A");
    const viewBPath = getTestImagePath("solved", "B");

    await expect(page.locator("#camera-apply")).toBeDisabled();
    await expect(page.locator("#camera-progress")).toContainText("0 / 6");

    // ビュー A をアップロード（U, R, F 面）
    await page.locator("#camera-file-a").setInputFiles(viewAPath);
    await page.waitForTimeout(300);

    for (const face of ["U", "R", "F"]) {
      await captureFaceOnTestImage(page, face);
    }

    await expect(page.locator("#camera-progress")).toContainText("3 / 6");
    await expect(page.locator("#camera-apply")).toBeDisabled();

    // ビュー B をアップロード（D, L, B 面）
    await page.locator("#camera-file-b").setInputFiles(viewBPath);
    await page.waitForTimeout(300);

    for (const face of ["D", "L", "B"]) {
      await captureFaceOnTestImage(page, face);
    }

    await expect(page.locator("#camera-progress")).toContainText("6 / 6");
    await expect(page.locator("#camera-apply")).not.toBeDisabled();

    await page.locator("#camera-close").click();
  });

  test("キャプチャをキャンセルして別の面に切り替えるとポイントがリセットされる", async ({
    page,
  }) => {
    await ready(page);
    await openCameraEditor(page);

    const viewAPath = getTestImagePath("solved", "A");
    await page.locator("#camera-file-a").setInputFiles(viewAPath);
    await page.waitForTimeout(300);

    // U 面で 1 点だけクリック
    await page.locator("#camera-face").selectOption("U");
    const canvas = page.locator("#camera-canvas");
    const box = await canvas.boundingBox();

    if (box) {
      await page.mouse.click(box.x + 50, box.y + 50);
    }

    // R 面に切り替えるとポイント入力がリセットされる
    await page.locator("#camera-face").selectOption("R");
    await expect(page.locator("#camera-capture")).toBeDisabled();

    await page.locator("#camera-close").click();
  });

  test("エラー表示領域が存在する", async ({ page }) => {
    await ready(page);
    await openCameraEditor(page);

    const errorElement = page.locator("#camera-error");
    await expect(errorElement).toBeAttached();

    const viewAPath = getTestImagePath("solved", "A");
    await page.locator("#camera-file-a").setInputFiles(viewAPath);
    await page.waitForTimeout(300);

    await expect(errorElement).toHaveText("");

    await page.locator("#camera-close").click();
  });
});

test.describe("カメラ入力 - エンドツーエンドテスト（対角2方向立体認識検証）", () => {
  test("solved 状態を画像から認識して色入力および3Dキューブへ正しく反映できる", async ({
    page,
  }) => {
    await ready(page);
    await openCameraEditor(page);

    const expectedState = getExpectedState("solved");

    // ビューA のアップロードと U, R, F 面キャプチャ
    await uploadTestImage(page, "solved", "A");
    for (const face of ["U", "R", "F"]) {
      await captureFaceOnTestImage(page, face);
    }

    // ビューB のアップロードと D, L, B 面キャプチャ
    await uploadTestImage(page, "solved", "B");
    for (const face of ["D", "L", "B"]) {
      await captureFaceOnTestImage(page, face);
    }

    await expect(page.locator("#camera-progress")).toContainText("6 / 6");
    await page.locator("#camera-apply").click();

    // カメラエディタが閉じられ、色入力エディタが開く
    await expect(page.locator("#camera-editor")).not.toBeVisible();
    await expect(page.locator("#editor")).toBeVisible();

    // 色入力エディタ（#editor-net）に反映された全54ステッカーが期待ステートと完全一致するか検証
    const editorState = await getEditorState(page);
    expect(editorState).toBe(expectedState);

    // 色入力エディタの反映ボタンを押してメイン画面のキューブへ適用
    await page.locator("#editor-apply").click();
    await expect(page.locator("#editor")).not.toBeVisible();

    // メインシーンの state が solved であることを検証
    await expect(page.locator("#scene")).toHaveAttribute(
      "data-state",
      expectedState,
    );
  });

  test("scrambled-1 状態を対角画像から正確に認識できる", async ({ page }) => {
    await ready(page);
    await openCameraEditor(page);

    const expectedState = getExpectedState("scrambled-1");

    await uploadTestImage(page, "scrambled-1", "A");
    for (const face of ["U", "R", "F"]) {
      await captureFaceOnTestImage(page, face);
    }

    await uploadTestImage(page, "scrambled-1", "B");
    for (const face of ["D", "L", "B"]) {
      await captureFaceOnTestImage(page, face);
    }

    await expect(page.locator("#camera-progress")).toContainText("6 / 6");
    await page.locator("#camera-apply").click();

    await expect(page.locator("#camera-editor")).not.toBeVisible();
    await expect(page.locator("#editor")).toBeVisible();

    const editorState = await getEditorState(page);
    expect(editorState).toBe(expectedState);

    // キューブに反映
    await page.locator("#editor-apply").click();
    await expect(page.locator("#editor")).not.toBeVisible();

    await expect(page.locator("#scene")).toHaveAttribute(
      "data-state",
      expectedState,
    );
  });

  test("superflip 状態を対角画像から正確に認識できる", async ({ page }) => {
    await ready(page);
    await openCameraEditor(page);

    const expectedState = getExpectedState("superflip");

    await uploadTestImage(page, "superflip", "A");
    for (const face of ["U", "R", "F"]) {
      await captureFaceOnTestImage(page, face);
    }

    await uploadTestImage(page, "superflip", "B");
    for (const face of ["D", "L", "B"]) {
      await captureFaceOnTestImage(page, face);
    }

    await expect(page.locator("#camera-progress")).toContainText("6 / 6");
    await page.locator("#camera-apply").click();

    await expect(page.locator("#camera-editor")).not.toBeVisible();
    await expect(page.locator("#editor")).toBeVisible();

    const editorState = await getEditorState(page);
    expect(editorState).toBe(expectedState);
  });

  test("mixed-colors 状態を対角画像から正確に認識できる", async ({ page }) => {
    await ready(page);
    await openCameraEditor(page);

    const expectedState = getExpectedState("mixed-colors");

    await uploadTestImage(page, "mixed-colors", "A");
    for (const face of ["U", "R", "F"]) {
      await captureFaceOnTestImage(page, face);
    }

    await uploadTestImage(page, "mixed-colors", "B");
    for (const face of ["D", "L", "B"]) {
      await captureFaceOnTestImage(page, face);
    }

    await page.locator("#camera-apply").click();

    await expect(page.locator("#camera-editor")).not.toBeVisible();
    await expect(page.locator("#editor")).toBeVisible();

    const editorState = await getEditorState(page);
    expect(editorState).toBe(expectedState);
  });

  test("partial 状態（未入力を含む）を対角画像から正確に認識できる", async ({
    page,
  }) => {
    await ready(page);
    await openCameraEditor(page);

    const expectedState = getExpectedState("partial");

    await uploadTestImage(page, "partial", "A");
    for (const face of ["U", "R", "F"]) {
      await captureFaceOnTestImage(page, face);
    }

    await uploadTestImage(page, "partial", "B");
    for (const face of ["D", "L", "B"]) {
      await captureFaceOnTestImage(page, face);
    }

    await page.locator("#camera-apply").click();

    await expect(page.locator("#camera-editor")).not.toBeVisible();
    await expect(page.locator("#editor")).toBeVisible();

    const editorState = await getEditorState(page);
    expect(editorState).toBe(expectedState);
    expect(editorState).toContain("?");
  });
});
