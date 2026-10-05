import { test, expect } from "@playwright/test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import {
  collectInputFiles,
  saveBuildManifest,
  checkInputsFreshness,
} from "../scripts/build-manifest.js";
import { getScopedStorageKey } from "../web/storage-key.js";

test.describe("9fb1ebc レビュー指摘点 (F1〜F4) 回帰テスト", () => {
  // F1: Service Worker プリキャッシュ生成スクリプトが隔離環境でも build-manifest.js と共に実行できる
  test("F1: sw-update build フィクスチャ相当の隔離環境で generate-sw-precache.js が正常動作する", () => {
    const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "sw-f1-regression-"));
    const distDir = path.join(tempDir, "dist");
    const scriptsDir = path.join(tempDir, "scripts");
    try {
      fs.mkdirSync(distDir, { recursive: true });
      fs.mkdirSync(scriptsDir, { recursive: true });
      fs.writeFileSync(
        path.join(tempDir, "package.json"),
        JSON.stringify({ type: "module" }),
      );
      fs.writeFileSync(
        path.join(distDir, "index.html"),
        "<!doctype html><title>Test</title>",
      );
      fs.copyFileSync("public/sw.js", path.join(distDir, "sw.js"));
      fs.copyFileSync(
        "scripts/build-manifest.js",
        path.join(scriptsDir, "build-manifest.js"),
      );
      const scriptPath = path.join(scriptsDir, "generate-sw-precache.js");
      fs.copyFileSync("scripts/generate-sw-precache.js", scriptPath);

      const result = spawnSync(process.execPath, [scriptPath], {
        cwd: tempDir,
        encoding: "utf-8",
      });

      expect(result.status).toBe(0);
      const swContent = fs.readFileSync(path.join(distDir, "sw.js"), "utf-8");
      expect(swContent).toMatch(/const CACHE_VERSION = "[^"]+";/);
      expect(fs.existsSync(path.join(distDir, ".build-manifest.json"))).toBe(
        true,
      );
    } finally {
      fs.rmSync(tempDir, { recursive: true, force: true });
    }
  });

  // F2: 入力マニフェストに tsconfig.json と scripts/build-manifest.js が含まれ、更新時に検知される
  test("F2: collectInputFiles が tsconfig.json と scripts/build-manifest.js を記録し変更を検知する", () => {
    const tempDir = fs.mkdtempSync(
      path.join(os.tmpdir(), "manifest-f2-regression-"),
    );
    const distDir = path.join(tempDir, "dist");
    const scriptsDir = path.join(tempDir, "scripts");
    try {
      fs.mkdirSync(distDir, { recursive: true });
      fs.mkdirSync(scriptsDir, { recursive: true });

      fs.writeFileSync(
        path.join(tempDir, "tsconfig.json"),
        JSON.stringify({ compilerOptions: { strict: true } }),
      );
      fs.writeFileSync(
        path.join(scriptsDir, "build-manifest.js"),
        "// initial manifest generator",
      );

      const initialInputs = collectInputFiles(tempDir);
      expect(initialInputs.has("tsconfig.json")).toBe(true);
      expect(initialInputs.has("scripts/build-manifest.js")).toBe(true);

      saveBuildManifest(tempDir, distDir, initialInputs);

      // tsconfig.json の変更
      fs.writeFileSync(
        path.join(tempDir, "tsconfig.json"),
        JSON.stringify({ compilerOptions: { strict: false } }),
      );
      const afterTsConfigChange = collectInputFiles(tempDir);
      const freshnessTs = checkInputsFreshness(
        tempDir,
        distDir,
        afterTsConfigChange,
      );
      expect(freshnessTs.fresh).toBe(false);
      expect(freshnessTs.reason).toContain("tsconfig.json");

      // scripts/build-manifest.js の変更
      saveBuildManifest(tempDir, distDir, afterTsConfigChange);
      fs.writeFileSync(
        path.join(scriptsDir, "build-manifest.js"),
        "// updated manifest generator",
      );
      const afterScriptChange = collectInputFiles(tempDir);
      const freshnessScript = checkInputsFreshness(
        tempDir,
        distDir,
        afterScriptChange,
      );
      expect(freshnessScript.fresh).toBe(false);
      expect(freshnessScript.reason).toContain("scripts/build-manifest.js");
    } finally {
      fs.rmSync(tempDir, { recursive: true, force: true });
    }
  });

  // F3: 非表示側へ画像を読み込んだ際、ビューBの表示や選択を維持したまま、カードAが即座に「読込完了」になる
  test("F3: 非表示側の画像読み込み完了時にカードとステータスが即時更新され、ビューと選択点は維持される", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await expect(page.locator("#engine-status")).toContainText("READY");
    await page.getByRole("tab", { name: "色を入力" }).click();
    await page.locator("#camera-colors").click();
    await page.locator("#camera-editor").waitFor({ state: "visible" });

    const result = await page.evaluate(async () => {
      const camera = (window as any).__lastCamera;
      const canvas = document.createElement("canvas");
      canvas.width = 20;
      canvas.height = 20;
      const ctx = canvas.getContext("2d")!;
      ctx.fillStyle = "blue";
      ctx.fillRect(0, 0, 20, 20);
      const blob = await new Promise<Blob | null>((res) => canvas.toBlob(res));
      const file = new File([blob!], "a.png", { type: "image/png" });

      const OriginalImage = window.Image;
      let releaseOnload: () => void = () => {};
      let notifyLoaded: () => void = () => {};
      const loaded = new Promise<void>((res) => (notifyLoaded = res));

      (window as any).Image = class extends OriginalImage {
        constructor() {
          super();
          this.addEventListener("load", notifyLoaded, { once: true });
          Object.defineProperty(this, "onload", {
            set(callback) {
              releaseOnload = callback;
            },
            get() {
              return null;
            },
          });
        }
      };

      try {
        // 画像Aの読み込みを開始
        const pending = camera.processFile(file, "A");
        // 利用者がビューBへ切り替える
        camera.switchView("B");
        const viewBeforeLoaded = camera.currentView;

        // 画像Aのロード完了を保留解除
        await loaded;
        releaseOnload();
        await pending;

        // 画像Aが非表示側で完了した直後の状態
        const currentViewWhileB = camera.currentView;
        const statusBeforeReturn =
          document.querySelector("#camera-status-a")?.textContent;
        const cardHasFileBeforeReturn = document
          .querySelector("#camera-drop-a")
          ?.classList.contains("has-file");
        const storedImage = !!camera.imageA;

        // ビューAに切り替えた後の状態
        camera.switchView("A");
        const statusAfterReturn =
          document.querySelector("#camera-status-a")?.textContent;
        const cardHasFileAfterReturn = document
          .querySelector("#camera-drop-a")
          ?.classList.contains("has-file");

        return {
          viewBeforeLoaded,
          currentViewWhileB,
          storedImage,
          statusBeforeReturn,
          cardHasFileBeforeReturn,
          statusAfterReturn,
          cardHasFileAfterReturn,
        };
      } finally {
        window.Image = OriginalImage;
      }
    });

    expect(result.viewBeforeLoaded).toBe("B");
    expect(result.currentViewWhileB).toBe("B");
    expect(result.storedImage).toBe(true);
    // 非表示側での完了時にも「未選択」ではなく「読込完了 (20×20)」になり、has-file が付与される
    expect(result.statusBeforeReturn).toBe("読込完了 (20×20)");
    expect(result.cardHasFileBeforeReturn).toBe(true);
    expect(result.statusAfterReturn).toBe("読込完了 (20×20)");
    expect(result.cardHasFileAfterReturn).toBe(true);
  });

  // F4: 静的ガードレールが web/storage-key.ts の正規化漏れを検知し、現行コードと別名URLテストをパスする
  test("F4: 静的ガードレールが web/storage-key.ts を走査し、index.html の未考慮を検出する", () => {
    const tempDir = fs.mkdtempSync(
      path.join(os.tmpdir(), "guardrail-f4-regression-"),
    );
    const webDir = path.join(tempDir, "web");
    const publicDir = path.join(tempDir, "public");
    try {
      fs.mkdirSync(webDir, { recursive: true });
      fs.mkdirSync(publicDir, { recursive: true });
      fs.copyFileSync("web/main.ts", path.join(webDir, "main.ts"));
      fs.copyFileSync("public/sw.js", path.join(publicDir, "sw.js"));

      // 壊れた storage-key.ts（pathname を直接キーに連結して index.html を考慮しない）
      fs.writeFileSync(
        path.join(webDir, "storage-key.ts"),
        "export function getScopedStorageKey(baseKey: string, customPath?: string): string { return `${baseKey}:${customPath ?? window.location.pathname}`; }\n",
      );

      const brokenResult = spawnSync(
        process.execPath,
        ["scripts/check-review-guardrails.js", tempDir],
        { encoding: "utf-8" },
      );

      expect(brokenResult.status).toBe(1);
      const output = brokenResult.stderr + brokenResult.stdout;
      expect(output).toContain("F4-URL-NORMALIZATION");
      expect(output).toContain("web/storage-key.ts");

      // 正しい storage-key.ts に戻すとパスする
      fs.copyFileSync(
        "web/storage-key.ts",
        path.join(webDir, "storage-key.ts"),
      );
      const validResult = spawnSync(
        process.execPath,
        ["scripts/check-review-guardrails.js", tempDir],
        { encoding: "utf-8" },
      );
      expect(validResult.status).toBe(0);
    } finally {
      fs.rmSync(tempDir, { recursive: true, force: true });
    }
  });

  test("F4-behavior: getScopedStorageKey が別名URL（ルートと index.html）で一貫したキーを返す", () => {
    const keyRoot = getScopedStorageKey("cube_setting", "/");
    const keyIndex = getScopedStorageKey("cube_setting", "/index.html");
    expect(keyRoot).toBe("cube_setting");
    expect(keyIndex).toBe("cube_setting");

    const keyNested = getScopedStorageKey("cube_setting", "/nested/cube/");
    const keyNestedIndex = getScopedStorageKey(
      "cube_setting",
      "/nested/cube/index.html",
    );
    expect(keyNested).toBe("cube_setting:/nested/cube");
    expect(keyNestedIndex).toBe("cube_setting:/nested/cube");
  });
});
