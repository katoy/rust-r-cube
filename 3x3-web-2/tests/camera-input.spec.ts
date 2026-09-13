import { test, expect, type Page } from "@playwright/test";
import {
  loadImageManifest,
  loadFullManifest,
  getTestImagePath,
  getFaceCorners,
  getOutlineCorners,
  getExpectedState,
  getEditorState,
  openCameraEditor,
  uploadTestImage,
  captureViewOnTestImage,
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
    expect(fullManifest.views.A.outline).toHaveLength(6);
    expect(fullManifest.views.B.outline).toHaveLength(6);

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

  test("フェース・ビュー選択セレクトが正常に動作する", async ({ page }) => {
    await ready(page);
    await openCameraEditor(page);

    const faceSelect = page.locator("#camera-face");
    await expect(faceSelect).toBeVisible();

    await faceSelect.selectOption("U");
    await expect(faceSelect).toHaveValue("U");
    await expect(page.locator("#camera-view-a")).toHaveAttribute(
      "aria-selected",
      "true",
    );

    await faceSelect.selectOption("D");
    await expect(faceSelect).toHaveValue("D");
    await expect(page.locator("#camera-view-b")).toHaveAttribute(
      "aria-selected",
      "true",
    );

    await page.locator("#camera-close").click();
  });

  test("画像をアップロードすると外周6角が自動検出されキャプチャボタンが有効になる", async ({
    page,
  }) => {
    await ready(page);
    await openCameraEditor(page);

    const viewAPath = getTestImagePath("solved", "A");
    await page.locator("#camera-file-a").setInputFiles(viewAPath);
    await page.waitForTimeout(300);

    // アップロード直後に自動検出されるため、キャプチャボタンが有効
    await expect(page.locator("#camera-capture")).not.toBeDisabled();
    await expect(page.locator("#camera-help")).toContainText("自動検出");

    // 画像Aの3面をそのままキャプチャ
    await page.locator("#camera-capture").click();
    await page.waitForTimeout(100);

    // 読み取り完了後、入力済み進捗が 3/6 面になる
    await expect(page.locator("#camera-progress")).toContainText("3 / 6");

    await page.locator("#camera-close").click();
  });

  test("クリアボタンで角をリセットし手動クリックで再指定できる", async ({
    page,
  }) => {
    await ready(page);
    await openCameraEditor(page);

    const viewAPath = getTestImagePath("solved", "A");
    await page.locator("#camera-file-a").setInputFiles(viewAPath);
    await page.waitForTimeout(300);

    // クリアボタンを押してリセット
    await page.locator("#camera-clear-points").click();
    await expect(page.locator("#camera-capture")).toBeDisabled();
    await expect(page.locator("#camera-help")).toContainText("0/6点");

    // 外周6点を手動クリックして再指定
    const outline = getOutlineCorners("A");
    const canvas = page.locator("#camera-canvas");
    const box = await canvas.boundingBox();
    if (!box) throw new Error("Canvas が見つかりません");

    const manifest = loadFullManifest();
    const scaleX = box.width / manifest.imageWidth;
    const scaleY = box.height / manifest.imageHeight;

    for (const pt of outline) {
      await page.mouse.click(box.x + pt.x * scaleX, box.y + pt.y * scaleY);
      await page.waitForTimeout(30);
    }

    // 6点指定後にキャプチャボタンが有効化
    await expect(page.locator("#camera-capture")).not.toBeDisabled();

    await page.locator("#camera-close").click();
  });

  test("自動検出された頂点をドラッグして微調整できる", async ({ page }) => {
    await ready(page);
    await openCameraEditor(page);

    const viewAPath = getTestImagePath("solved", "A");
    await page.locator("#camera-file-a").setInputFiles(viewAPath);
    await page.waitForTimeout(300);

    const canvas = page.locator("#camera-canvas");
    const box = await canvas.boundingBox();
    if (!box) throw new Error("Canvas が見つかりません");

    // P1 (てっぺん: x=320, y=80) 付近をつかんで少し上へドラッグ
    const startX = box.x + box.width * (320 / 640);
    const startY = box.y + box.height * (80 / 480);

    await page.mouse.move(startX, startY);
    await page.mouse.down();
    await page.mouse.move(startX, startY - 15);
    await page.mouse.up();

    // キャプチャボタンは依然として有効
    await expect(page.locator("#camera-capture")).not.toBeDisabled();

    // 「角を自動検出」ボタンを押すと初期位置に再検出される
    await page.locator("#camera-detect").click();
    await page.waitForTimeout(100);
    await expect(page.locator("#camera-capture")).not.toBeDisabled();

    await page.locator("#camera-close").click();
  });

  test("外周6点指定で画像A・画像Bをキャプチャしてから apply が有効になる", async ({
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

    await captureViewOnTestImage(page, "A");

    await expect(page.locator("#camera-progress")).toContainText("3 / 6");
    await expect(page.locator("#camera-apply")).toBeDisabled();

    // ビュー B をアップロード（D, L, B 面）
    await page.locator("#camera-file-b").setInputFiles(viewBPath);
    await page.waitForTimeout(300);

    await captureViewOnTestImage(page, "B");

    await expect(page.locator("#camera-progress")).toContainText("6 / 6");
    await expect(page.locator("#camera-apply")).not.toBeDisabled();

    await page.locator("#camera-close").click();
  });

  test("ビューを切り替えると自動検出が再実行される", async ({ page }) => {
    await ready(page);
    await openCameraEditor(page);

    const viewAPath = getTestImagePath("solved", "A");
    await page.locator("#camera-file-a").setInputFiles(viewAPath);
    await page.waitForTimeout(300);

    // クリアして未指定状態にする
    await page.locator("#camera-clear-points").click();
    await expect(page.locator("#camera-capture")).toBeDisabled();

    // ビューAを再表示すると再検出されてキャプチャ可能になる
    await page.locator("#camera-detect").click();
    await expect(page.locator("#camera-capture")).not.toBeDisabled();

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

    // ビューA のアップロードと 3面キャプチャ
    await uploadTestImage(page, "solved", "A");
    await captureViewOnTestImage(page, "A");

    // ビューB のアップロードと 3面キャプチャ
    await uploadTestImage(page, "solved", "B");
    await captureViewOnTestImage(page, "B");

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
    await captureViewOnTestImage(page, "A");

    await uploadTestImage(page, "scrambled-1", "B");
    await captureViewOnTestImage(page, "B");

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
    await captureViewOnTestImage(page, "A");

    await uploadTestImage(page, "superflip", "B");
    await captureViewOnTestImage(page, "B");

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
    await captureViewOnTestImage(page, "A");

    await uploadTestImage(page, "mixed-colors", "B");
    await captureViewOnTestImage(page, "B");

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
    await captureViewOnTestImage(page, "A");

    await uploadTestImage(page, "partial", "B");
    await captureViewOnTestImage(page, "B");

    await page.locator("#camera-apply").click();

    await expect(page.locator("#camera-editor")).not.toBeVisible();
    await expect(page.locator("#editor")).toBeVisible();

    const editorState = await getEditorState(page);
    expect(editorState).toBe(expectedState);
    expect(editorState).toContain("?");
  });
});
