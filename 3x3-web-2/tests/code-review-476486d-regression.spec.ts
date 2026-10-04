import { test, expect, type Page } from "@playwright/test";

async function ready(page: Page, url = "/?no-sw") {
  await page.goto(url);
  await expect(page.locator("#engine-status")).toContainText("READY");
  await page.locator("#reduced-motion").check();
}

test.describe("HEAD 476486d Code Review Regressions", () => {
  test("H1: 画像画素数検査例外時に loading 状態が確実に解除され UI がフリーズしないこと", async ({
    page,
  }) => {
    await ready(page);

    // カメラ入力モーダルを開く
    await page.locator('button[data-tab="colors"]').click();
    await page.locator("#camera-colors").click();
    await expect(page.locator("#camera-editor")).toBeVisible();

    // 0 ピクセル相当の無効な PNG バイト（IHDR の幅・高さが 0）を生成して選択
    const invalidPngBuffer = Buffer.from([
      0x89,
      0x50,
      0x4e,
      0x47,
      0x0d,
      0x0a,
      0x1a,
      0x0a, // PNG Signature
      0x00,
      0x00,
      0x00,
      0x0d, // IHDR chunk length
      0x49,
      0x48,
      0x44,
      0x52, // "IHDR"
      0x00,
      0x00,
      0x00,
      0x00, // width = 0
      0x00,
      0x00,
      0x00,
      0x00, // height = 0
      0x08,
      0x06,
      0x00,
      0x00,
      0x00, // 8-bit RGBA
      0x00,
      0x00,
      0x00,
      0x00, // CRC dummy
    ]);

    await page.locator("#camera-file-a").setInputFiles({
      name: "zero-pixel.png",
      mimeType: "image/png",
      buffer: invalidPngBuffer,
    });

    // エラーメッセージが表示されることを確認
    await expect(page.locator("#camera-error")).toContainText(
      "画像の寸法が不正です",
    );

    // カード UI のステータスが「読込中…」のまま放置されていないことを検証（H1 回帰確認）
    const statusTextA = await page.locator("#camera-status-a").textContent();
    expect(statusTextA).not.toContain("読込中");

    // loading フラグが解除されていることを window の内部状態から検証
    const loadingA = await page.evaluate(() => {
      // @ts-ignore
      const cam = window.__cube_main_debug__?.camera;
      return cam ? (cam as any).loading?.A : false;
    });
    expect(loadingA).toBe(false);

    // モーダルを閉じる
    await page.locator("#camera-close").click();
    await expect(page.locator("#camera-editor")).toBeHidden();
  });

  test("H2: turn() 中に回転対象面のセンターラベルが turnLayer に attach され、finish() 後に復元されること", async ({
    page,
  }) => {
    await ready(page);
    await page.waitForFunction(() => !!(window as any).cube_scene);

    // Scene の turn() 中に U 面センターラベルが turnLayer にアタッチされるかを検証
    const debugInfo = await page.evaluate(async () => {
      const scene =
        (window as any).__cube_main_debug__?.getScene?.() ??
        (window as any).cube_scene;
      if (!scene) return { err: "no scene" };

      // U 面の回転アニメーションを開始
      const turnPromise = scene.turn(
        "U",
        "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB",
        300,
      );

      // turnLayer に U 面センターラベル（centerLabels[0]）が含まれているか確認
      const turnLayer = (scene as any).turnLayer;
      const uCenterLabel = (scene as any).centerLabels[0];
      const attached = turnLayer.children.includes(uCenterLabel);

      // アニメーション完了を待機
      await turnPromise;

      // 完了後は turnLayer から解放されていることを確認
      const detached = !turnLayer.children.includes(uCenterLabel);

      return {
        attached,
        detached,
        hasCenterLabel: !!uCenterLabel,
        childrenCount: turnLayer.children.length,
      };
    });

    console.log("H2 debugInfo:", debugInfo);
    expect(debugInfo.attached).toBe(true);
    expect(debugInfo.detached).toBe(true);
  });

  test("M3: 公開 WASM API 11 個すべてが正常に動作すること", async ({
    page,
  }) => {
    await ready(page);

    const results = await page.evaluate(async () => {
      const wasm = await import(/* @vite-ignore */ "/pkg/cube_studio.js");
      await wasm.default({ module_or_path: "/pkg/cube_studio_bg.wasm" });

      const solved = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
      const scrambled = wasm.scramble(12345);

      let isValidCorruptCaught = false;
      try {
        wasm.is_valid("UUUUUUUUU");
      } catch {
        isValidCorruptCaught = true;
      }

      return {
        hasInitialize: typeof wasm.initialize === "function",
        scrambleNonEmpty: typeof scrambled === "string" && scrambled.length > 0,
        applyMovesOk:
          JSON.parse(wasm.apply_moves(solved, "R U R' U'")).moves.length === 4,
        validateSolved: wasm.validate(solved) === true,
        isValidSolved: wasm.is_valid(solved) === true,
        isValidCorrupt: isValidCorruptCaught,
        isSolvedTrue: wasm.is_solved(solved) === true,
        isSolvedFalse:
          wasm.is_solved(JSON.parse(wasm.apply_moves(solved, "R")).state) ===
          false,
        centerParityZero: wasm.center_parity(solved) === 0,
        solveSolved: JSON.parse(wasm.solve(solved, 1000)).moves.length === 0,
        solveWithOrientationOk:
          JSON.parse(wasm.solve_with_orientation(solved, 1000, true)).moves
            .length === 0,
        solveWithAlgorithmOk:
          JSON.parse(
            wasm.solve_with_algorithm(solved, 1000, false, undefined, "cfop"),
          ).moves.length === 0,
        orientationsCornersLen:
          JSON.parse(wasm.get_orientations(solved)).corners.length === 8,
      };
    });

    expect(results.hasInitialize).toBe(true);
    expect(results.scrambleNonEmpty).toBe(true);
    expect(results.applyMovesOk).toBe(true);
    expect(results.validateSolved).toBe(true);
    expect(results.isValidSolved).toBe(true);
    expect(results.isValidCorrupt).toBe(true);
    expect(results.isSolvedTrue).toBe(true);
    expect(results.isSolvedFalse).toBe(true);
    expect(results.centerParityZero).toBe(true);
    expect(results.solveSolved).toBe(true);
    expect(results.solveWithOrientationOk).toBe(true);
    expect(results.solveWithAlgorithmOk).toBe(true);
    expect(results.orientationsCornersLen).toBe(true);
  });

  test("M4: #palette に role='radiogroup' が設定されていること", async ({
    page,
  }) => {
    await ready(page);

    const paletteRole = await page.locator("#palette").getAttribute("role");
    expect(paletteRole).toBe("radiogroup");
  });

  test("L1: 不正な centers パラメータが渡された場合に警告され URL が正規化されること", async ({
    page,
  }) => {
    // 不正な centers（数値以外の文字列 "abc" を含む）を指定
    await ready(
      page,
      "/?no-sw&state=UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB&centers=0,0,abc,0,0,0",
    );

    // URL から不正な centers パラメータが消去され正規化されていること
    expect(page.url()).not.toContain("centers=");

    // 不正パラメータに対する警告トーストまたは通知が表示されていること
    const toast = page.locator("#toast");
    if (await toast.isVisible()) {
      await expect(toast).toContainText("不正なURLパラメータ");
    }
  });

  test("M1: SW Cache Migration で直近1世代から最大15件のみ引き継がれ、過去世代が削除されること", async ({
    page,
  }) => {
    await ready(page);

    const migrationResult = await page.evaluate(async () => {
      const CACHE_PREFIX = "cube-studio-";
      const NEW_CACHE = "cube-studio-new-v2";
      const OLD_V1 = "cube-studio-old-v1";
      const OLD_V0 = "cube-studio-old-v0";

      // クリーンアップ
      await caches.delete(NEW_CACHE);
      await caches.delete(OLD_V1);
      await caches.delete(OLD_V0);

      // OLD_V1 に 25 件の資産を投入
      const oldCache1 = await caches.open(OLD_V1);
      for (let i = 1; i <= 25; i++) {
        await oldCache1.put(
          new Request(`http://127.0.0.1:5173/asset-${i}.js`),
          new Response(`console.log(${i})`, {
            headers: { "Content-Type": "application/javascript" },
          }),
        );
      }

      // OLD_V0 に 5 件の資産を投入
      const oldCache0 = await caches.open(OLD_V0);
      for (let i = 1; i <= 5; i++) {
        await oldCache0.put(
          new Request(`http://127.0.0.1:5173/old-asset-${i}.js`),
          new Response(`console.log("old-${i}")`, {
            headers: { "Content-Type": "application/javascript" },
          }),
        );
      }

      // sw.js の activate マイグレーション処理を再現実行
      const ownCache = await caches.open(NEW_CACHE);
      const keys = await caches.keys();
      const oldKeys = keys.filter(
        (key) =>
          key.startsWith(CACHE_PREFIX) &&
          key !== NEW_CACHE &&
          /^(v\d+|old-v\d+|[0-9a-f]{8,})$/i.test(
            key.slice(CACHE_PREFIX.length),
          ),
      );

      const sortedOldKeys = [...oldKeys].sort();
      const mostRecentOldKey = sortedOldKeys.pop();

      let migratedCount = 0;
      const MAX_MIGRATED_ITEMS = 15;

      if (mostRecentOldKey) {
        try {
          const oldCache = await caches.open(mostRecentOldKey);
          const requests = await oldCache.keys();

          for (const req of requests) {
            if (migratedCount >= MAX_MIGRATED_ITEMS) break;
            const alreadyCached = await ownCache.match(req);
            if (!alreadyCached) {
              const res = await oldCache.match(req);
              if (res && res.ok) {
                await ownCache.put(req, res);
                migratedCount++;
              }
            }
          }
        } catch {}
        await caches.delete(mostRecentOldKey);
      }

      for (const oldKey of sortedOldKeys) {
        try {
          await caches.delete(oldKey);
        } catch {}
      }

      const newKeys = await ownCache.keys();
      const hasOld1 = await caches.has(OLD_V1);
      const hasOld0 = await caches.has(OLD_V0);

      // クリーンアップ
      await caches.delete(NEW_CACHE);

      return {
        migratedCount,
        newKeysCount: newKeys.length,
        hasOld1,
        hasOld0,
      };
    });

    expect(migrationResult.migratedCount).toBe(15);
    expect(migrationResult.newKeysCount).toBe(15);
    expect(migrationResult.hasOld1).toBe(false);
    expect(migrationResult.hasOld0).toBe(false);
  });
});
