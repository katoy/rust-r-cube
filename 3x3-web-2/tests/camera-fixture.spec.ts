import { test, expect } from "@playwright/test";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";
import { loadFullManifest } from "./test-utils";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const TEST_IMAGES_DIR = path.resolve(__dirname, "../test-images");
const MANIFEST_PATH = path.join(TEST_IMAGES_DIR, "manifest.json");

test.describe("R03: Camera fixture availability in clean environment", () => {
  test("loadFullManifest automatically ensures manifest and test images exist", () => {
    // manifest.json が一時的に存在しない状態をシミュレート
    const backupPath = `${MANIFEST_PATH}.bak`;
    let backedUp = false;
    if (fs.existsSync(MANIFEST_PATH)) {
      fs.renameSync(MANIFEST_PATH, backupPath);
      backedUp = true;
    }

    try {
      // 現状の実装では manifest.json が無ければ ENOENT で throw されるはず
      const manifest = loadFullManifest();
      expect(manifest).toBeDefined();
      expect(manifest.images).toBeDefined();
    } finally {
      if (backedUp && fs.existsSync(backupPath)) {
        if (fs.existsSync(MANIFEST_PATH)) {
          fs.unlinkSync(MANIFEST_PATH);
        }
        fs.renameSync(backupPath, MANIFEST_PATH);
      }
    }
  });
});
