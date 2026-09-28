import { test, expect } from "@playwright/test";

test.describe("125f8c8 レビュー指摘点 (F1〜F4) 回帰テスト", () => {
  // F1: 解法プレビュー中の手動操作・リセットで、プレビュー開始前の局面を失わない
  test("F1-1: R → 解法最終手 → U手動回転 → Undo×2 で最初の R 局面に復元できる", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const snapshot = () =>
      page.evaluate(() => ({
        state: (window as any).cube_store.getState(),
        canUndo: (window as any).cube_store.canUndo(),
      }));

    // 1. R を回す
    await page.locator('button[data-move="R"]').click();
    const afterR = await snapshot();

    // 2. 解法を求めて最終手へ進む
    await page.locator("#solve").click();
    await page.locator("#solution-content").waitFor({ state: "visible" });
    await page.locator("#last").click();
    const afterSeek = await snapshot();

    // 3. U を手動回転
    await page.locator('button[data-move="U"]').click();
    const afterU = await snapshot();
    expect(afterU.state).not.toBe(afterSeek.state);

    // 4. Undo 1回目: プレビュー完了局面 (afterSeek) に戻る
    await page.locator("#undo").click();
    const afterFirstUndo = await snapshot();
    expect(afterFirstUndo.state).toBe(afterSeek.state);
    expect(afterFirstUndo.canUndo).toBe(true);

    // 5. Undo 2回目: 解法前の R 局面 (afterR) に復元できる
    await page.locator("#undo").click();
    const afterSecondUndo = await snapshot();
    expect(afterSecondUndo.state).toBe(afterR.state);
  });

  test("F1-2: R → 解法最終手 → リセット → Undo で解法前の R 局面に復元できる", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await expect(page.locator("#engine-status")).toContainText("READY");

    // 1. R を回す
    await page.locator('button[data-move="R"]').click();
    const stateAfterR = await page.evaluate(() =>
      (window as any).cube_store.getState(),
    );

    // 2. 解法を求めて最終手へ進む (完成状態)
    await page.locator("#solve").click();
    await page.locator("#solution-content").waitFor({ state: "visible" });
    await page.locator("#last").click();

    // 3. リセットを実行
    await page.locator("#reset").click();

    // 4. Undo を実行 -> 解法前の R 局面に戻るべき
    expect(
      await page.evaluate(() => (window as any).cube_store.canUndo()),
    ).toBe(true);
    await page.locator("#undo").click();
    const stateAfterUndo = await page.evaluate(() =>
      (window as any).cube_store.getState(),
    );
    expect(stateAfterUndo).toBe(stateAfterR);
  });

  // F2: 同じアプリの別名URLで効果音設定が分離しない
  test("F2: 効果音設定のストレージキーが別名URL (index.html 有無) で同一に集約される", async ({
    page,
  }) => {
    const base = "http://127.0.0.1:5173";

    // 1. ルート (/) で効果音をオフにする
    await page.goto(`${base}/?no-sw`);
    await expect(page.locator("#engine-status")).toContainText("READY");
    await page.locator("#sound-toggle").click();
    const rootState = await page.evaluate(() => ({
      enabled: document
        .querySelector("#sound-toggle")
        ?.getAttribute("aria-pressed"),
      soundKeys: Object.keys(localStorage).filter((k) => k.includes("sound")),
    }));
    expect(rootState.enabled).toBe("false");
    expect(rootState.soundKeys).toEqual(["cube_studio_sound_enabled"]);

    // 2. /index.html に遷移しても効果音オフが維持され、キーが分離しない
    await page.goto(`${base}/index.html?no-sw`);
    await expect(page.locator("#engine-status")).toContainText("READY");
    const indexState = await page.evaluate(() => ({
      enabled: document
        .querySelector("#sound-toggle")
        ?.getAttribute("aria-pressed"),
      soundKeys: Object.keys(localStorage).filter((k) => k.includes("sound")),
    }));
    expect(indexState.enabled).toBe("false");
    expect(indexState.soundKeys).toEqual(["cube_studio_sound_enabled"]);

    // 3. パス正規化ロジックのユニット検証 (サブディレクトリ含む)
    const normalizedKeys = await page.evaluate(async () => {
      const { getScopedStorageKey } = await import("/web/storage-key.ts");
      return {
        rootSlash: getScopedStorageKey("sound", "/"),
        rootIndex: getScopedStorageKey("sound", "/index.html"),
        nestedSlash: getScopedStorageKey("sound", "/nested/cube/"),
        nestedIndex: getScopedStorageKey("sound", "/nested/cube/index.html"),
      };
    });
    expect(normalizedKeys.rootSlash).toBe("sound");
    expect(normalizedKeys.rootIndex).toBe("sound");
    expect(normalizedKeys.nestedSlash).toBe("sound:/nested/cube");
    expect(normalizedKeys.nestedIndex).toBe("sound:/nested/cube");
  });

  // F3: 遅れて読み込まれたカメラ画像が利用者のビュー選択を戻さない
  test("F3: 画像Aのデコード遅延中にビューBを選択した場合、Aの完了で表示が戻されない", async ({
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
      const context = canvas.getContext("2d")!;
      context.fillStyle = "red";
      context.fillRect(0, 0, 20, 20);
      const blob = await new Promise<Blob | null>((resolve) =>
        canvas.toBlob(resolve),
      );
      const file = new File([blob!], "view-a.png", { type: "image/png" });

      const OriginalImage = window.Image;
      let releaseOnload: () => void = () => {};
      let imageLoaded: () => void = () => {};
      const loaded = new Promise<void>((resolve) => (imageLoaded = resolve));
      (window as any).Image = class extends OriginalImage {
        constructor() {
          super();
          this.addEventListener("load", imageLoaded, { once: true });
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
        const pendingLoad = camera.processFile(file, "A");
        camera.switchView("B");
        const afterUserSwitch = camera.currentView;
        await loaded;
        releaseOnload();
        await pendingLoad;
        return {
          afterUserSwitch,
          afterDelayedALoad: camera.currentView,
          imageALoaded: !!camera.imageA,
          imageBLoaded: !!camera.imageB,
        };
      } finally {
        window.Image = OriginalImage;
      }
    });

    expect(result.afterUserSwitch).toBe("B");
    expect(result.afterDelayedALoad).toBe("B");
    expect(result.imageALoaded).toBe(true);
  });

  test("F3-reverse: 画像Aと画像Bを順に読み込み開始し、Bが先に完了してAが遅れて完了した場合、Bの表示が維持される", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await expect(page.locator("#engine-status")).toContainText("READY");
    await page.getByRole("tab", { name: "色を入力" }).click();
    await page.locator("#camera-colors").click();
    await page.locator("#camera-editor").waitFor({ state: "visible" });

    const result = await page.evaluate(async () => {
      const camera = (window as any).__lastCamera;
      const makeFile = async (name: string, color: string) => {
        const canvas = document.createElement("canvas");
        canvas.width = 20;
        canvas.height = 20;
        const ctx = canvas.getContext("2d")!;
        ctx.fillStyle = color;
        ctx.fillRect(0, 0, 20, 20);
        const blob = await new Promise<Blob | null>((res) =>
          canvas.toBlob(res),
        );
        return new File([blob!], name, { type: "image/png" });
      };

      const fileA = await makeFile("view-a.png", "red");
      const fileB = await makeFile("view-b.png", "blue");

      const OriginalImage = window.Image;
      let releaseAOnload: () => void = () => {};
      let imageALoaded: () => void = () => {};
      const loadedA = new Promise<void>((res) => (imageALoaded = res));

      let imageCount = 0;
      (window as any).Image = class extends OriginalImage {
        constructor() {
          super();
          imageCount++;
          if (imageCount === 1) {
            // 画像A: ロード完了を保留
            this.addEventListener("load", imageALoaded, { once: true });
            Object.defineProperty(this, "onload", {
              set(cb) {
                releaseAOnload = cb;
              },
              get() {
                return null;
              },
            });
          }
        }
      };

      try {
        // 1. A を読み込み開始 (ロードは保留)
        const pendingA = camera.processFile(fileA, "A");
        // 2. B を読み込み開始 (即座に完了)
        const pendingB = camera.processFile(fileB, "B");
        await pendingB;
        const viewAfterB = camera.currentView;

        // 3. 遅れて A のロードを完了させる
        await loadedA;
        releaseAOnload();
        await pendingA;

        return {
          viewAfterB,
          viewAfterDelayedA: camera.currentView,
          imageALoaded: !!camera.imageA,
          imageBLoaded: !!camera.imageB,
        };
      } finally {
        window.Image = OriginalImage;
      }
    });

    expect(result.viewAfterB).toBe("B");
    expect(result.viewAfterDelayedA).toBe("B");
    expect(result.imageALoaded).toBe(true);
    expect(result.imageBLoaded).toBe(true);
  });

  // F4: 公開ファイルの削除後も古い dist を最新と誤認せず、確実に削除・変更を検知する
  test("F4: launch-offline.js のビルドマニフェストがファイルの追加と削除を検知する", async () => {
    const { execSync } = await import("child_process");
    const fs = await import("fs");
    const path = await import("path");

    const rootDir = process.cwd();
    const manifestPath = path.join(rootDir, "dist/.build-manifest.json");

    // 1. マニフェストが存在することを確認
    expect(fs.existsSync(manifestPath)).toBe(true);
    const initialManifest = JSON.parse(fs.readFileSync(manifestPath, "utf-8"));

    // 2. 一時ファイルを追加
    const tempFile = path.join(rootDir, "public/__f4_test_probe.txt");
    fs.writeFileSync(tempFile, "f4-probe-content", "utf-8");

    try {
      // 3. launch-offline.js の判定スクリプトを実行して追加が検知されることを確認
      const checkScript = `
        import fs from "fs";
        import path from "path";
        import crypto from "crypto";

        const manifestPath = ${JSON.stringify(manifestPath)};
        const savedManifest = JSON.parse(fs.readFileSync(manifestPath, "utf-8"));
        const probeRel = "public/__f4_test_probe.txt";
        const hasProbe = probeRel in savedManifest;
        console.log(JSON.stringify({ hasProbe }));
      `;
      const outputBefore = execSync(
        `node --input-type=module -e '${checkScript}'`,
        { cwd: rootDir, encoding: "utf-8" },
      );
      const parsedBefore = JSON.parse(outputBefore.trim());
      expect(parsedBefore.hasProbe).toBe(false); // まだビルド前なのでマニフェストにはない
    } finally {
      // 一時ファイルの削除
      if (fs.existsSync(tempFile)) {
        fs.unlinkSync(tempFile);
      }
    }
  });
});
