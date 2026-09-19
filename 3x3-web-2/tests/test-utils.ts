/**
 * カメラ入力テスト用ユーティリティ
 */

import { Page, expect } from "@playwright/test";
import * as fs from "fs";
import * as path from "path";
import { fileURLToPath } from "url";

import { execSync } from "child_process";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
export const TEST_IMAGES_DIR = path.join(__dirname, "../test-images");

export interface Point {
  x: number;
  y: number;
}

export interface ImageManifestData {
  generated: string;
  imageWidth: number;
  imageHeight: number;
  totalStates: number;
  views: {
    A: {
      faces: string[];
      corners: Record<string, Point[]>;
      outline?: Point[];
    };
    B: {
      faces: string[];
      corners: Record<string, Point[]>;
      outline?: Point[];
    };
  };
  images: Record<
    string,
    {
      viewA: string;
      viewB: string;
      expectedState?: string;
    }
  >;
}

/**
 * テスト画像マニフェスト全体を読み込む
 */
export function loadFullManifest(): ImageManifestData {
  const manifestPath = path.join(TEST_IMAGES_DIR, "manifest.json");
  if (!fs.existsSync(manifestPath)) {
    const scriptPath = path.resolve(
      __dirname,
      "../scripts/generate-test-images.js",
    );
    execSync(`node "${scriptPath}"`, { stdio: "pipe" });
  }
  return JSON.parse(fs.readFileSync(manifestPath, "utf-8"));
}

/**
 * テスト画像マニフェスト（画像ファイル一覧）を読み込む
 */
export function loadImageManifest(): Record<
  string,
  { viewA: string; viewB: string; expectedState?: string }
> {
  return loadFullManifest().images;
}

/**
 * テスト画像のパスを取得
 */
export function getTestImagePath(imageName: string, view: "A" | "B"): string {
  const manifest = loadImageManifest();
  if (!manifest[imageName]) {
    throw new Error(`テスト画像 "${imageName}" が見つかりません`);
  }
  const filename =
    view === "A" ? manifest[imageName].viewA : manifest[imageName].viewB;
  return path.join(TEST_IMAGES_DIR, filename);
}

/**
 * 面ラベル（U, R, F, D, L, B）に対応する立体の4隅座標を取得
 */
export function getFaceCorners(faceLabel: string): Point[] {
  const manifest = loadFullManifest();
  if (manifest.views.A.corners[faceLabel]) {
    return manifest.views.A.corners[faceLabel];
  }
  if (manifest.views.B.corners[faceLabel]) {
    return manifest.views.B.corners[faceLabel];
  }
  throw new Error(`面 "${faceLabel}" の座標定義が見つかりません`);
}

/**
 * ビュー（A, B）の外周6点（てっぺんから時計回り）を取得
 */
export function getOutlineCorners(view: "A" | "B"): Point[] {
  const manifest = loadFullManifest();
  if (manifest.views[view]?.outline) {
    return manifest.views[view].outline!;
  }
  // フォールバック（計算値）
  return [
    { x: 320, y: 80 },
    { x: 459, y: 160 },
    { x: 459, y: 320 },
    { x: 320, y: 400 },
    { x: 181, y: 320 },
    { x: 181, y: 160 },
  ];
}

/**
 * 期待する54文字のキューブ状態文字列を取得
 */
export function getExpectedState(imageName: string): string {
  const manifest = loadImageManifest();
  if (!manifest[imageName]?.expectedState) {
    throw new Error(`テスト画像 "${imageName}" の期待ステートが見つかりません`);
  }
  return manifest[imageName].expectedState!;
}

/**
 * カメラ入力エディタを開く
 */
export async function openCameraEditor(page: Page) {
  await page.getByRole("tab", { name: "色を入力" }).click();
  await page.locator("#camera-colors").click();
  await expect(page.locator("#camera-editor")).toBeVisible();
}

/**
 * テスト画像をカメラエディタにアップロード
 */
export async function uploadTestImage(
  page: Page,
  imageName: string,
  view: "A" | "B",
) {
  const imagePath = getTestImagePath(imageName, view);
  const inputSelector = view === "A" ? "#camera-file-a" : "#camera-file-b";
  await page.locator(inputSelector).setInputFiles(imagePath);

  // 画像が読み込まれるまで待機
  await page.waitForTimeout(300);
}

/**
 * ビュー（AまたはB）の外周6点をキャプチャ（自動検出された6点または指定座標）
 */
export async function captureViewOnTestImage(
  page: Page,
  view: "A" | "B",
  manualPoints?: Point[],
) {
  // ビューの切り替え
  const tabSelector = view === "A" ? "#camera-view-a" : "#camera-view-b";
  await page.locator(tabSelector).click();
  await page.waitForTimeout(150);

  if (manualPoints && manualPoints.length > 0) {
    // 手動指定テストの場合: クリアボタンを押して指定座標をクリック
    await page.locator("#camera-clear-points").click();
    await page.waitForTimeout(50);

    const canvas = page.locator("#camera-canvas");
    const box = await canvas.boundingBox();
    if (!box) throw new Error("Canvas が見つかりません");

    const manifest = loadFullManifest();
    const scaleX = box.width / manifest.imageWidth;
    const scaleY = box.height / manifest.imageHeight;

    for (const pt of manualPoints) {
      const clickX = box.x + pt.x * scaleX;
      const clickY = box.y + pt.y * scaleY;
      await page.mouse.click(clickX, clickY);
      await page.waitForTimeout(30);
    }
  }

  // キャプチャボタンをクリック
  await expect(page.locator("#camera-capture")).not.toBeDisabled();
  await page.locator("#camera-capture").click();
  await page.waitForTimeout(100);
}

/**
 * 面単位のテスト用ラッパー（外周6点キャプチャを呼び出し）
 */
export async function captureFaceOnTestImage(page: Page, faceLabel: string) {
  const view = ["U", "R", "F"].includes(faceLabel) ? "A" : "B";
  const progressText = await page.locator("#camera-progress").textContent();
  const count = parseInt(progressText?.split("/")[0] || "0", 10);
  // まだ該当ビューが読み取られていなければ読み取る
  if ((view === "A" && count < 3) || (view === "B" && count < 6)) {
    await captureViewOnTestImage(page, view);
  }
}

/**
 * 色入力エディタ（#editor-net）に反映された全54ステッカーの色文字列を取得
 */
export async function getEditorState(page: Page): Promise<string> {
  return await page.evaluate(() => {
    const stickers = Array.from(
      document.querySelectorAll("#editor-net .sticker"),
    );
    return stickers.map((el) => el.getAttribute("data-color") || "?").join("");
  });
}

/**
 * すべての面をキャプチャしてカメラ入力とキューブへの反映を完了
 */
export async function completeImageCapture(page: Page, imageName: string) {
  await openCameraEditor(page);
  await uploadTestImage(page, imageName, "A");

  for (const face of ["U", "R", "F"]) {
    await captureFaceOnTestImage(page, face);
  }

  await uploadTestImage(page, imageName, "B");

  for (const face of ["D", "L", "B"]) {
    await captureFaceOnTestImage(page, face);
  }

  // すべての面をキャプチャしたら apply
  await expect(page.locator("#camera-progress")).toContainText("6 / 6");
  await page.locator("#camera-apply").click();
  await expect(page.locator("#camera-editor")).not.toBeVisible();

  // 色入力ダイアログが開くのでキューブに反映
  await expect(page.locator("#editor")).toBeVisible();
  await page.locator("#editor-apply").click();
  await expect(page.locator("#editor")).not.toBeVisible();
}
