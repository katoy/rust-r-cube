import { test, expect, type Page } from "@playwright/test";
import {
  getTestImagePath,
  uploadTestImage,
  captureViewOnTestImage,
} from "./test-utils";

async function ready(page: Page) {
  await page.goto("/?type=2x2");
  await expect(page.locator("#engine-status")).toContainText("READY");
  await page.locator("#reduced-motion").check();
}

test.describe("2x2x2 カメラ認識機能", () => {
  test("image-sampler: 2x2 サンプリングと buildState の単体動作", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const result = await page.evaluate(async () => {
      const sampler = await import("/web/image-sampler.ts");

      // 1. buildState の 2x2 テスト
      const state2x2 = sampler.buildState(
        {
          U: "UUUU",
          R: "RRRR",
          F: "FFFF",
          D: "DDDD",
          L: "LLLL",
          B: "BBBB",
        },
        "2x2",
      );

      const partial2x2 = sampler.buildState(
        {
          U: "UUUU",
          F: "FFFF",
        },
        "2x2",
      );

      // 2. sampleFaceFromPixels の 2x2 サンプリング (size = 2)
      // 400x400 のキャンバスを作成し、2x2 の 4 色領域を配置
      // 左上: U (White), 右上: R (Red), 左下: F (Green), 右下: D (Yellow)
      const canvas = document.createElement("canvas");
      canvas.width = 100;
      canvas.height = 100;
      const ctx = canvas.getContext("2d")!;

      // 左上 (u in [0, 0.5), v in [0, 0.5)) -> White
      ctx.fillStyle = "#ffffff";
      ctx.fillRect(0, 0, 50, 50);

      // 右上 (u in [0.5, 1], v in [0, 0.5)) -> Red
      ctx.fillStyle = "#dc2626";
      ctx.fillRect(50, 0, 50, 50);

      // 左下 (u in [0, 0.5), v in [0.5, 1]) -> Green
      ctx.fillStyle = "#16a34a";
      ctx.fillRect(0, 50, 50, 50);

      // 右下 (u in [0.5, 1], v in [0.5, 1]) -> Yellow
      ctx.fillStyle = "#eab308";
      ctx.fillRect(50, 50, 50, 50);

      const imgData = ctx.getImageData(0, 0, 100, 100);
      const points = [
        { x: 0, y: 0 },
        { x: 100, y: 0 },
        { x: 100, y: 100 },
        { x: 0, y: 100 },
      ];

      const sampled = sampler.sampleFaceFromPixels(
        imgData,
        points,
        undefined,
        2,
      );

      return {
        state2x2,
        partial2x2,
        sampled,
      };
    });

    expect(result.state2x2).toBe("UUUURRRRFFFFDDDDLLLLBBBB");
    expect(result.state2x2.length).toBe(24);
    expect(result.partial2x2).toBe("UUUU????FFFF????????????");
    expect(result.sampled).toBe("URFD");
  });

  test("2x2 モードでカメラボタンが表示され、ダイアログで2x2グリッド（各4マス）が展開される", async ({
    page,
  }) => {
    await ready(page);

    // 「色を入力」タブを選択
    await page.getByRole("tab", { name: "色を入力" }).click();

    // カメラボタンが表示されていること
    const camBtn = page.locator("#camera-colors");
    await expect(camBtn).toBeVisible();
    await expect(camBtn).not.toBeDisabled();

    // カメラエディタを開く
    await camBtn.click();
    await expect(page.locator("#camera-editor")).toBeVisible();

    // ガイドヒントが 2x2 向けになっていること（センター除く、の記載がない）
    const hint = page.locator(".camera-result-hint");
    await expect(hint).toHaveText("各セルをクリックして色を修正できます");

    // 展開図プレビュー（#camera-result-faces）の検証
    const faceCards = page.locator("#camera-result-faces .camera-face-card");
    await expect(faceCards).toHaveCount(6);

    // 各カードが .face-grid.grid-2x2 を持ち、4つのステッカーボタンを持つこと
    for (let f = 0; f < 6; f++) {
      const card = faceCards.nth(f);
      const grid = card.locator(".face-grid.grid-2x2");
      await expect(grid).toBeVisible();

      const stickers = grid.locator(".sticker");
      await expect(stickers).toHaveCount(4);

      // 2x2 では全セルがクリック可能（disabled なセンターセルがない）
      for (let s = 0; s < 4; s++) {
        await expect(stickers.nth(s)).toBeEnabled();
        await expect(stickers.nth(s)).not.toHaveAttribute("data-center");
      }
    }

    await page.locator("#camera-close").click();
    await expect(page.locator("#camera-editor")).not.toBeVisible();
  });

  test("2x2 モードで画像をキャプチャし、手動色補正からエディタ反映まで完遂できる", async ({
    page,
  }) => {
    await ready(page);

    // 「色を入力」タブを選択してカメラエディタを開く
    await page.getByRole("tab", { name: "色を入力" }).click();
    await page.locator("#camera-colors").click();
    await expect(page.locator("#camera-editor")).toBeVisible();

    // 画像Aをアップロード
    const viewAPath = getTestImagePath("solved", "A");
    await page.locator("#camera-file-a").setInputFiles(viewAPath);
    await expect(page.locator("#camera-status-a")).toContainText("読込完了");

    // 画像Aをキャプチャ（U, R, F の3面を読み取り）
    await captureViewOnTestImage(page, "A");

    // センター不一致エラーが出ていないこと
    await expect(page.locator("#camera-error")).toHaveText("");
    await expect(page.locator("#camera-progress")).toContainText("3 / 6");

    // 画像Bをアップロード
    const viewBPath = getTestImagePath("solved", "B");
    await page.locator("#camera-file-b").setInputFiles(viewBPath);
    await expect(page.locator("#camera-status-b")).toContainText("読込完了");

    // 画像Bをキャプチャ（D, L, B の3面を読み取り）
    await captureViewOnTestImage(page, "B");
    await expect(page.locator("#camera-error")).toHaveText("");
    await expect(page.locator("#camera-progress")).toContainText("6 / 6");

    // 各面のステッカーが読み取られていること
    const uFaceStickers = page.locator("#camera-face-card-U .sticker");
    await expect(uFaceStickers).toHaveCount(4);

    // 手動色補正: パレットで赤（R）を選び、U面の第1ステッカーをクリックして色変更を確認
    const rChoice = page.locator(
      '.camera-palette button[aria-label="赤を選択"]',
    );
    await rChoice.click();
    await uFaceStickers.nth(0).click();
    await expect(uFaceStickers.nth(0)).toHaveAttribute("data-color", "R");

    // 再度白（U）を選び、U面の第1ステッカーを白に戻して有効な状態（各色4枚）に復元
    const uChoice = page.locator(
      '.camera-palette button[aria-label="白を選択"]',
    );
    await uChoice.click();
    await uFaceStickers.nth(0).click();
    await expect(uFaceStickers.nth(0)).toHaveAttribute("data-color", "U");

    // 反映ボタンをクリック
    await expect(page.locator("#camera-apply")).toBeEnabled();
    await page.locator("#camera-apply").click();
    await expect(page.locator("#camera-editor")).not.toBeVisible();

    // カラーエディタダイアログ（#editor）が開き、2x2 モード（24マス）で反映されていること
    await expect(page.locator("#editor")).toBeVisible();
    await expect(page.locator("#color-count")).toContainText(
      "24 / 24 マス入力済み",
    );

    // エディタを適用
    await page.locator("#editor-apply").click();
    await expect(page.locator("#editor")).not.toBeVisible();

    // キューブ状態が反映されていること（24文字）
    const cubeState =
      (await page.locator("#scene").getAttribute("data-state")) ??
      (await page.evaluate(() => (window as any).cube_store?.getState()));
    expect(cubeState).toHaveLength(24);
    expect(cubeState).toBe("UUUURRRRFFFFDDDDLLLLBBBB");
  });

  test("2x2カメラ: 多色スクランブル画像を取り込み、24文字の配置と手動補正が完全一致すること", async ({
    page,
  }) => {
    await ready(page);

    // 「色を入力」タブを選択してカメラエディタを開く
    await page.getByRole("tab", { name: "色を入力" }).click();
    await page.locator("#camera-colors").click();
    await expect(page.locator("#camera-editor")).toBeVisible();

    // 2x2 多色スクランブル画像Aをアップロードしてキャプチャ
    const viewAPath = getTestImagePath("2x2-gods-number-11", "A");
    await page.locator("#camera-file-a").setInputFiles(viewAPath);
    await expect(page.locator("#camera-status-a")).toContainText("読込完了");
    await captureViewOnTestImage(page, "A");

    // 2x2 多色スクランブル画像Bをアップロードしてキャプチャ
    const viewBPath = getTestImagePath("2x2-gods-number-11", "B");
    await page.locator("#camera-file-b").setInputFiles(viewBPath);
    await expect(page.locator("#camera-status-b")).toContainText("読込完了");
    await captureViewOnTestImage(page, "B");

    // 整合性チェック（validate2x2Faces）が通りエラーがないこと
    await expect(page.locator("#camera-error")).toHaveText("");
    await expect(page.locator("#camera-progress")).toContainText("6 / 6");

    // 各面のステッカー認識結果（BFRUFRRLDLLDBBDDRFLUUUFB）の完全一致検証
    const expected = "BFRUFRRLDLLDBBDDRFLUUUFB";
    const faces = ["U", "R", "F", "D", "L", "B"];
    let detectedState = "";
    for (let f = 0; f < faces.length; f++) {
      const faceName = faces[f];
      const stickers = page.locator(`#camera-face-card-${faceName} .sticker`);
      await expect(stickers).toHaveCount(4);
      for (let s = 0; s < 4; s++) {
        const color = await stickers.nth(s).getAttribute("data-color");
        detectedState += color;
      }
    }
    expect(detectedState).toBe(expected);

    // 手動色補正の検証（M6 指摘: 補正したセルがエディタと盤面に正しく伝播すること）
    // U面の第0ステッカー (初期値 'B') をパレットから 'R' に変更
    const rChoice = page.locator(
      '.camera-palette button[aria-label="赤を選択"]',
    );
    await rChoice.click();
    const uSticker0 = page.locator("#camera-face-card-U .sticker").nth(0);
    await uSticker0.click();
    await expect(uSticker0).toHaveAttribute("data-color", "R");

    // 反映ボタンをクリックしてカラーエディタへ遷移
    await expect(page.locator("#camera-apply")).toBeEnabled();
    await page.locator("#camera-apply").click();
    await expect(page.locator("#camera-editor")).not.toBeVisible();
    await expect(page.locator("#editor")).toBeVisible();

    // エディタ内の U 面第 0 ステッカーが 'R' になっていることを確認（buildState への反映を実証）
    const editorU0 = page.locator('#editor-net [data-index="0"]');
    await expect(editorU0).toHaveAttribute("data-color", "R");

    // エディタ上で青（B）パレットを選択し、U 面第 0 ステッカーを 'B' に戻して合法状態（各色4枚）にする
    const bColorChoice = page.locator(
      '#palette button[aria-label*="青を選択"]',
    );
    await bColorChoice.click();
    await editorU0.click();
    await expect(editorU0).toHaveAttribute("data-color", "B");

    // エディタを適用
    await page.locator("#editor-apply").click();
    await expect(page.locator("#editor")).not.toBeVisible();

    // 最終盤面状態がスクランブル状態（24文字）と完全一致すること
    const cubeState =
      (await page.locator("#scene").getAttribute("data-state")) ??
      (await page.evaluate(() => (window as any).cube_store?.getState()));
    expect(cubeState).toBe(expected);
  });
});
