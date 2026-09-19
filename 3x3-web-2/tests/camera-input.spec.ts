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
    await expect(page.locator("#camera-drop-a .file-card-title")).toHaveText(
      "画像A（白面・赤面・緑面）",
    );
    await expect(page.locator("#camera-drop-b .file-card-title")).toHaveText(
      "画像B（黄面・橙面・青面）",
    );
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
    await expect(page.locator("#camera-help")).toContainText(
      "白面・赤面・緑面",
    );
    await expect(page.locator("#camera-drop-a .file-card-title")).toHaveText(
      "画像A（白面・赤面・緑面）",
    );
    await expect(page.locator("#camera-face option[value='U']")).toHaveText(
      "画像A（白面・赤面・緑面）",
    );

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

    // 中心点 (x=320, y=240 付近) をつかんでドラッグ微調整できる
    const centerX = box.x + box.width * (320 / 640);
    const centerY = box.y + box.height * (240 / 480);
    await page.mouse.move(centerX, centerY);
    await page.mouse.down();
    await page.mouse.move(centerX + 10, centerY + 10);
    await page.mouse.up();
    await expect(page.locator("#camera-capture")).not.toBeDisabled();

    await page.locator("#camera-close").click();
  });

  test("5点（5角形）など途中の頂点もドラッグして移動や右クリック削除ができる", async ({
    page,
  }) => {
    await ready(page);
    await openCameraEditor(page);

    const viewAPath = getTestImagePath("solved", "A");
    await page.locator("#camera-file-a").setInputFiles(viewAPath);
    await page.waitForTimeout(300);

    // クリアして手動入力モードへ
    await page.locator("#camera-clear-points").click();
    await expect(page.locator("#camera-help")).toContainText("0/6点");

    const canvas = page.locator("#camera-canvas");
    const box = await canvas.boundingBox();
    if (!box) throw new Error("Canvas が見つかりません");

    const outline = getOutlineCorners("A");
    const manifest = loadFullManifest();
    const scaleX = box.width / manifest.imageWidth;
    const scaleY = box.height / manifest.imageHeight;

    // 5点だけクリックして配置（5角形状態）
    for (let i = 0; i < 5; i++) {
      await page.mouse.click(
        box.x + outline[i].x * scaleX,
        box.y + outline[i].y * scaleY,
      );
      await page.waitForTimeout(30);
    }
    await expect(page.locator("#camera-help")).toContainText("5/6点");

    // 5番目の頂点をドラッグして移動
    const pt5X = box.x + outline[4].x * scaleX;
    const pt5Y = box.y + outline[4].y * scaleY;

    await page.mouse.move(pt5X, pt5Y);
    await page.mouse.down();
    await page.mouse.move(pt5X + 20, pt5Y - 20);
    await page.mouse.up();

    // 5点のまま（追加されていないこと）
    await expect(page.locator("#camera-help")).toContainText("5/6点");

    // 1番目の頂点もドラッグして移動
    const pt1X = box.x + outline[0].x * scaleX;
    const pt1Y = box.y + outline[0].y * scaleY;
    await page.mouse.move(pt1X, pt1Y);
    await page.mouse.down();
    await page.mouse.move(pt1X, pt1Y - 10);
    await page.mouse.up();

    await expect(page.locator("#camera-help")).toContainText("5/6点");

    // 右クリックで頂点削除のテスト（5点から4点になる）
    await page.mouse.click(pt1X, pt1Y - 10, { button: "right" });
    await expect(page.locator("#camera-help")).toContainText("4/6点");

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

  test("読み取り結果が表示され、通常セルの色を補正でき、センターセルの色は編集できない", async ({
    page,
  }) => {
    await ready(page);
    await openCameraEditor(page);

    const viewAPath = getTestImagePath("solved", "A");
    await page.locator("#camera-file-a").setInputFiles(viewAPath);
    await page.waitForTimeout(300);

    // 画像Aをキャプチャ（3面の各センターが認識され該当面U, F, Rとして設定される）
    await page.locator("#camera-capture").click();
    await page.waitForTimeout(100);

    // 読み取り結果セクションが表示されている
    const resultSection = page.locator("#camera-result-section");
    await expect(resultSection).toBeVisible();

    // ヒント文がセンター除くになっている
    await expect(page.locator(".camera-result-hint")).toContainText(
      "センター除く",
    );

    // パレットが表示されている
    const palette = page.locator("#camera-palette");
    await expect(palette).toBeVisible();

    // U面（上面）のセンターセル（インデックス4）の初期色は "U"（白）でdisabled（編集不可）
    const centerCellU = page.locator(
      '#camera-face-card-U .sticker[data-index="4"]',
    );
    await expect(centerCellU).toHaveAttribute("data-color", "U");
    await expect(centerCellU).toHaveAttribute("data-center", "true");
    await expect(centerCellU).toBeDisabled();

    // パレットで赤（R）を選択
    await palette.locator('button[aria-label*="赤"]').click();

    // U面のセンターセル（インデックス4）をクリックしても色は変わらない（編集不可）
    await centerCellU.click({ force: true });
    await expect(centerCellU).toHaveAttribute("data-color", "U");

    // U面のエッジセル（通常セル、インデックス1）をクリックして赤に変更できる
    const edgeCellU = page.locator(
      '#camera-face-card-U .sticker[data-index="1"]',
    );
    await edgeCellU.click();
    await expect(edgeCellU).toHaveAttribute("data-color", "R");

    await page.locator("#camera-close").click();
  });

  test("実機写真（IMG_3243.png）から白・緑・赤の3面を自動検出・正確に読み取れる", async ({
    page,
  }) => {
    const realPhotoA = "/Users/katoy/Downloads/IMG_3243.png";
    const fs = await import("fs");
    if (!fs.existsSync(realPhotoA)) {
      test.skip();
      return;
    }

    await ready(page);
    await openCameraEditor(page);

    // 画像Aに実機写真をセット
    await page.locator("#camera-file-a").setInputFiles(realPhotoA);
    await page.waitForTimeout(500);

    // 外周6点が自動検出され、キャプチャボタンが有効
    const captureButton = page.locator("#camera-capture");
    await expect(captureButton).toBeEnabled();

    // キャプチャ実行
    await captureButton.click();

    // 画像Bがまだ未ロードなので、画像Aのタブが選択されたままプレビュー結果が表示される
    await expect(page.locator("#camera-view-a")).toHaveAttribute(
      "aria-selected",
      "true",
    );

    // 読み取り完了面数が「3 / 6 面」となる
    await expect(page.locator("#camera-progress")).toContainText("3 / 6 面");

    // 各面のセンターピースが正しくU（白）、F（緑）、R（赤）として割り当てられている
    const centerCellU = page.locator(
      '#camera-face-card-U .sticker[data-index="4"]',
    );
    const centerCellF = page.locator(
      '#camera-face-card-F .sticker[data-index="4"]',
    );
    const centerCellR = page.locator(
      '#camera-face-card-R .sticker[data-index="4"]',
    );

    await expect(centerCellU).toHaveAttribute("data-color", "U");
    await expect(centerCellF).toHaveAttribute("data-color", "F");
    await expect(centerCellR).toHaveAttribute("data-color", "R");

    await page.locator("#camera-close").click();
  });

  test("実機写真2枚（IMG_3243.png と IMG_3244.png）から全6面を読み取り、色入力へ反映できる", async ({
    page,
  }) => {
    const realPhotoA = "/Users/katoy/Downloads/IMG_3243.png";
    const realPhotoB = "/Users/katoy/Downloads/IMG_3244.png";
    const fs = await import("fs");
    if (!fs.existsSync(realPhotoA) || !fs.existsSync(realPhotoB)) {
      test.skip();
      return;
    }

    await ready(page);
    await openCameraEditor(page);

    // 1. 画像Aをアップロードしてキャプチャ
    await page.locator("#camera-file-a").setInputFiles(realPhotoA);
    await page.waitForTimeout(500);
    await page.locator("#camera-capture").click();
    await expect(page.locator("#camera-progress")).toContainText("3 / 6 面");

    // 2. 画像Bをアップロード
    await page.locator("#camera-file-b").setInputFiles(realPhotoB);
    await page.waitForTimeout(500);

    // 画像Bの表示に切り替わっていることを確認
    await expect(page.locator("#camera-view-b")).toHaveAttribute(
      "aria-selected",
      "true",
    );

    // キャプチャボタンが有効なのでキャプチャ
    await page.locator("#camera-capture").click();

    // 6 / 6 面となり、反映ボタンが有効になる
    await expect(page.locator("#camera-progress")).toContainText("6 / 6 面");
    const applyButton = page.locator("#camera-apply");
    await expect(applyButton).toBeEnabled();

    // 色入力へ反映をクリック
    await applyButton.click();

    // カメラダイアログが閉じ、メイン画面の展開図に色が反映されている
    await expect(page.locator("#camera-editor")).not.toBeVisible();
    await expect(page.locator("#app")).toBeVisible();
  });

  test("枠を回転ボタン（および 'r' キー）で六角形枠を60度ずつ回転できる", async ({
    page,
  }) => {
    await ready(page);
    await openCameraEditor(page);

    const rotateBtn = page.locator("#camera-rotate-points");
    // 画像未選択時は非活性
    await expect(rotateBtn).toBeDisabled();

    // 画像Aをアップロード
    const viewAPath = getTestImagePath("solved", "A");
    await page.locator("#camera-file-a").setInputFiles(viewAPath);
    await page.waitForTimeout(300);

    // 6点自動検出されたら回転ボタンが活性化
    await expect(rotateBtn).toBeEnabled();

    // 1回目のクリックで頂点が60度回転
    await rotateBtn.click();
    await page.waitForTimeout(100);

    // キーボード 'r' でも回転できる
    await page.keyboard.press("r");
    await page.waitForTimeout(100);

    // クリアすると非活性になる
    await page.locator("#camera-clear-points").click();
    await expect(rotateBtn).toBeDisabled();

    await page.locator("#camera-close").click();
  });

  test("実機写真Bで枠を回転させて逆Y字型キューブに適合させて認識できる", async ({
    page,
  }) => {
    const realPhotoB = "/Users/katoy/Downloads/IMG_3244.png";
    const fs = await import("fs");
    if (!fs.existsSync(realPhotoB)) {
      test.skip();
      return;
    }

    await ready(page);
    await openCameraEditor(page);

    // 画像Bタブに切り替えて画像Bをアップロード
    await page.locator("#camera-view-b").click();
    await page.locator("#camera-file-b").setInputFiles(realPhotoB);
    await page.waitForTimeout(500);

    // 枠を回転ボタンを押して逆Y字型（写真の稜線の向き）に合わせる
    const rotateBtn = page.locator("#camera-rotate-points");
    await expect(rotateBtn).toBeEnabled();
    await rotateBtn.click();
    await page.waitForTimeout(300);

    // キャプチャ実行
    await page.locator("#camera-capture").click();

    // 下面が黄色（D）、他の面も青（B）・橙（L）として認識される
    const centerCellD = page.locator(
      '#camera-face-card-D .sticker[data-index="4"]',
    );
    await expect(centerCellD).toHaveAttribute("data-color", "D");

    await page.locator("#camera-close").click();
  });
});
