/**
 * カメラ入力テスト用ユーティリティ
 */

import { Page, expect } from "@playwright/test";
import * as fs from "fs";
import * as path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
export const TEST_IMAGES_DIR = path.join(__dirname, "../test-images");

/**
 * テスト画像マニフェストを読み込む
 */
export function loadImageManifest(): Record<
  string,
  { viewA: string; viewB: string }
> {
  const manifestPath = path.join(TEST_IMAGES_DIR, "manifest.json");
  const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf-8"));
  return manifest.images;
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
  view: "A" | "B"
) {
  const imagePath = getTestImagePath(imageName, view);
  const inputSelector = view === "A" ? "#camera-file-a" : "#camera-file-b";
  await page.locator(inputSelector).setInputFiles(imagePath);

  // 画像が読み込まれるまで待機
  await page.waitForTimeout(500);
}

/**
 * 面の四隅をクリックしてキャプチャ（テスト用簡易実装）
 */
export async function captureFaceOnTestImage(
  page: Page,
  faceLabel: string
) {
  // フェース選択
  await page.locator("#camera-face").selectOption(faceLabel);

  // Canvas の位置を取得
  const canvas = page.locator("#camera-canvas");
  const box = await canvas.boundingBox();
  if (!box) throw new Error("Canvas が見つかりません");

  // 四隅をクリック（左上→右上→右下→左下）
  const corners = [
    { x: box.x + 10, y: box.y + 10 },
    { x: box.x + box.width - 10, y: box.y + 10 },
    { x: box.x + box.width - 10, y: box.y + box.height - 10 },
    { x: box.x + 10, y: box.y + box.height - 10 },
  ];

  for (const corner of corners) {
    await page.mouse.click(corner.x, corner.y);
  }

  // キャプチャボタンをクリック
  await page.locator("#camera-capture").click();
}

/**
 * すべての面をキャプチャしてカメラ入力を完了（テスト用）
 */
export async function completeImageCapture(
  page: Page,
  imageName: string
) {
  await openCameraEditor(page);
  await uploadTestImage(page, imageName, "A");

  const faces = ["U", "R", "F"];
  for (const face of faces) {
    await captureFaceOnTestImage(page, face);
  }

  await uploadTestImage(page, imageName, "B");

  const facesB = ["D", "L", "B"];
  for (const face of facesB) {
    await captureFaceOnTestImage(page, face);
  }

  // すべての面をキャプチャしたら apply
  await page.locator("#camera-apply").click();
  await expect(page.locator("#camera-editor")).not.toBeVisible();
}
