import { test, expect } from "@playwright/test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { checkInputsFreshness } from "../scripts/build-manifest.js";

const getTestImagePath = (name: string, view: "A" | "B") =>
  path.resolve(
    process.cwd(),
    `test-images/${name}-view-${view.toLowerCase()}.png`,
  );

test.describe("011a7b9 レビュー指摘点 (F1〜F4) 回帰テスト", () => {
  // F1: 写真を差し替えた場合に旧写真の読取結果を適用できず、再キャプチャ後にのみ適用可能になる
  test("F1: 6面読取後に画像Aを差し替えると旧結果が失効し、再キャプチャ前は適用不可（3/6面）、再キャプチャ後に適用可能（6/6面）になる", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();
    await page.getByRole("tab", { name: "色を入力" }).click();
    await page.locator("#camera-colors").click();
    await page.locator("#camera-editor").waitFor({ state: "visible" });

    // 1. 画像A・画像Bを初回読取
    await page
      .locator("#camera-file-a")
      .setInputFiles(getTestImagePath("solved", "A"));
    await page.waitForFunction(
      () => (window as any).__lastCamera?.points.length === 6,
    );
    await page.locator("#camera-capture").click();

    await page
      .locator("#camera-file-b")
      .setInputFiles(getTestImagePath("solved", "B"));
    await page.waitForFunction(
      () =>
        (window as any).__lastCamera?.currentView === "B" &&
        (window as any).__lastCamera?.points.length === 6,
    );
    await page.locator("#camera-capture").click();

    // 6面揃って適用可能
    await expect(page.locator("#camera-progress")).toContainText("6 / 6 面");
    await expect(page.locator("#camera-apply")).toBeEnabled();

    // 2. 画像Aを別局面の画像に差し替え
    const oldUrlA = await page.evaluate(
      () => (window as any).__lastCamera.sourceUrlA,
    );
    await page
      .locator("#camera-file-a")
      .setInputFiles(getTestImagePath("scrambled-1", "A"));

    await page.waitForFunction(
      (oldUrl) => (window as any).__lastCamera?.sourceUrlA !== oldUrl,
      oldUrlA,
    );

    // 差し替え直後（再キャプチャ前）は進捗が 3 / 6 面に減少し、適用ボタンが無効化されること
    await expect(page.locator("#camera-progress")).toContainText("3 / 6 面");
    await expect(page.locator("#camera-apply")).toBeDisabled();

    const facesAfterChange = await page.evaluate(
      () => (window as any).__lastCamera.faces,
    );
    expect(facesAfterChange.U).toBeUndefined();
    expect(facesAfterChange.R).toBeUndefined();
    expect(facesAfterChange.F).toBeUndefined();
    expect(facesAfterChange.D).toBeDefined();
    expect(facesAfterChange.L).toBeDefined();
    expect(facesAfterChange.B).toBeDefined();

    // 3. 画像Aを新画像から再キャプチャ
    await page.waitForFunction(
      () => (window as any).__lastCamera?.points.length === 6,
    );
    await page.locator("#camera-capture").click();

    // 再キャプチャ後は新画像から6面になり、適用ボタンが再度有効化されること
    await expect(page.locator("#camera-progress")).toContainText("6 / 6 面");
    await expect(page.locator("#camera-apply")).toBeEnabled();

    const facesAfterRecapture = await page.evaluate(
      () => (window as any).__lastCamera.faces,
    );
    expect(facesAfterRecapture.U).toBeDefined();
    expect(facesAfterRecapture.R).toBeDefined();
    expect(facesAfterRecapture.F).toBeDefined();
  });

  // F2: 古いプリセット要求の失敗（HTTPエラー等）が新しい成功表示を上書きしない
  test("F2: 保留中の旧プリセット要求の失敗が、後発の成功プリセット表示を上書きしない", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();
    await page.locator("#tab-presets").click();

    let releaseOld: () => void = () => {};
    const oldHeld = new Promise<void>((resolve) => {
      releaseOld = resolve;
    });
    let oldRequested: () => void = () => {};
    const requested = new Promise<void>((resolve) => {
      oldRequested = resolve;
    });

    await page.route("**/cubes/solved.json", async (route) => {
      oldRequested();
      await oldHeld;
      await route.fulfill({
        status: 503,
        contentType: "application/json",
        body: "{}",
      });
    });

    // 1. 完成状態の取得を開始（遅延・保留）
    await page
      .locator("#preset-buttons button")
      .filter({ hasText: "完成状態" })
      .click();
    await requested;

    // 2. 続いて簡単（3手）をクリック（即座に成功）
    await page
      .locator("#preset-buttons button")
      .filter({ hasText: "簡単（3手）" })
      .click();
    await page
      .locator("#preset-status")
      .filter({ hasText: "簡単（3手） を読み込みました" })
      .waitFor();

    const statusBeforeOldRelease = await page
      .locator("#preset-status")
      .textContent();
    expect(statusBeforeOldRelease).toContain("簡単（3手） を読み込みました");

    // 3. 保留していた旧リクエストへ HTTP 503 を返却
    releaseOld();
    await page.waitForTimeout(300);

    // 旧リクエストのエラーによって新リクエストの成功表示が上書きされないこと
    const statusAfterOldRelease = await page
      .locator("#preset-status")
      .textContent();
    expect(statusAfterOldRelease).toContain("簡単（3手） を読み込みました");
    expect(statusAfterOldRelease).not.toContain("読み込み失敗");
  });

  test("F2: 保留中の旧プリセット要求が成功しても、後発の失敗プリセット表示を上書きしない（逆順完了保護）", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();
    await page.locator("#tab-presets").click();

    let releaseOld: () => void = () => {};
    const oldHeld = new Promise<void>((resolve) => {
      releaseOld = resolve;
    });
    let oldRequested: () => void = () => {};
    const requested = new Promise<void>((resolve) => {
      oldRequested = resolve;
    });

    await page.route("**/cubes/solved.json", async (route) => {
      oldRequested();
      await oldHeld;
      await route.continue();
    });

    // 2つ目の要求は 500 エラーを返す
    await page.route("**/cubes/easy-5-moves.json", async (route) => {
      await route.fulfill({
        status: 500,
        contentType: "application/json",
        body: "{}",
      });
    });

    // 1. 完成状態の取得を開始（遅延・保留）
    await page
      .locator("#preset-buttons button")
      .filter({ hasText: "完成状態" })
      .click();
    await requested;

    // 2. 続いて簡単（3手）をクリック（即座に 500 で失敗）
    await page
      .locator("#preset-buttons button")
      .filter({ hasText: "簡単（3手）" })
      .click();
    await page
      .locator("#preset-status")
      .filter({ hasText: "読み込み失敗" })
      .waitFor();

    const statusAfterFail = await page.locator("#preset-status").textContent();
    expect(statusAfterFail).toContain("読み込み失敗");

    // 3. 保留していた旧リクエスト（完成状態: 成功）を解放
    releaseOld();
    await page.waitForTimeout(300);

    // 後から届いた古い成功リクエストによって、最新の失敗状態が上書きされないこと
    const statusAfterOldRelease = await page
      .locator("#preset-status")
      .textContent();
    expect(statusAfterOldRelease).toContain("読み込み失敗");
    expect(statusAfterOldRelease).not.toContain("完成状態 を読み込みました");
  });

  // F1-error: 画像読込失敗時にも古い読取結果が残らず適用不可となること
  test("F1: 画像選択で読み込み失敗（非画像等）となった場合にも旧結果は適用不可のまま維持される", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();
    await page.getByRole("tab", { name: "色を入力" }).click();
    await page.locator("#camera-colors").click();
    await page.locator("#camera-editor").waitFor({ state: "visible" });

    // 画像A・Bを正常読取
    await page
      .locator("#camera-file-a")
      .setInputFiles(getTestImagePath("solved", "A"));
    await page.waitForFunction(
      () => (window as any).__lastCamera?.points.length === 6,
    );
    await page.locator("#camera-capture").click();

    await page
      .locator("#camera-file-b")
      .setInputFiles(getTestImagePath("solved", "B"));
    await page.waitForFunction(
      () =>
        (window as any).__lastCamera?.currentView === "B" &&
        (window as any).__lastCamera?.points.length === 6,
    );
    await page.locator("#camera-capture").click();

    await expect(page.locator("#camera-progress")).toContainText("6 / 6 面");
    await expect(page.locator("#camera-apply")).toBeEnabled();

    // 壊れた画像/非画像ファイルを選択
    const tempTextFile = path.join(os.tmpdir(), "dummy-invalid.png");
    fs.writeFileSync(tempTextFile, "not an image content");
    try {
      await page.locator("#camera-file-a").setInputFiles(tempTextFile);
      await page.waitForTimeout(300);

      // 読込失敗時も画像Aの面は失効し、3/6面で適用不可であること
      await expect(page.locator("#camera-progress")).toContainText("3 / 6 面");
      await expect(page.locator("#camera-apply")).toBeDisabled();
    } finally {
      fs.rmSync(tempTextFile, { force: true });
    }
  });

  // F3: タイムラインの aria-valuetext が完了手数と次の手を明確に分離して読む
  test("F3: タイムラインの aria-valuetext が開始・中間・完了位置で正確に読み上げられる", async ({
    page,
  }) => {
    await page.goto("/?no-sw&alg=R+U");
    await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();
    await page.locator("#solve").click();
    await page.locator("#solution-content").waitFor({ state: "visible" });

    const timeline = page.locator("#timeline");

    // 開始位置（step = 0）: 次の1手目を示す
    expect(await timeline.getAttribute("aria-valuenow")).toBe("0");
    const val0 = await timeline.getAttribute("aria-valuetext");
    expect(val0).toBe("開始状態。次は1手目 U' (同時最適化: 色＆センター向き)");

    // 1手進める（step = 1）: 1手完了し、次は2手目 R'
    await page.locator("#next").click();
    await page.waitForFunction(
      () => (window as any).cube_store?.getStep() === 1,
    );

    expect(await timeline.getAttribute("aria-valuenow")).toBe("1");
    const val1 = await timeline.getAttribute("aria-valuetext");
    expect(val1).toBe("1手完了。次は2手目 R' (同時最適化: 色＆センター向き)");

    // 2手進める（step = 2, 完成状態）: 完成（2手）
    await page.locator("#next").click();
    await page.waitForFunction(
      () => (window as any).cube_store?.getStep() === 2,
    );

    expect(await timeline.getAttribute("aria-valuenow")).toBe("2");
    const val2 = await timeline.getAttribute("aria-valuetext");
    expect(val2).toBe("完成 (2手)");
  });

  // F4: 構文上正しい破損マニフェスト（null, 配列, nullエントリ等）で TypeError にならず fresh: false を返す
  test("F4: checkInputsFreshness が null や配列、欠損・破損エントリを含むマニフェストで例外を出さず再ビルドを促す", () => {
    const tempDir = fs.mkdtempSync(
      path.join(os.tmpdir(), "manifest-f4-regression-"),
    );
    const distDir = path.join(tempDir, "dist");
    fs.mkdirSync(distDir, { recursive: true });
    const currentInputs = new Map([
      ["src/main.rs", { hash: "abc", size: 100, mtime: 1000 }],
    ]);

    try {
      const cases: Record<string, string> = {
        malformed_json: "{",
        valid_json_null: "null",
        valid_json_array: "[]",
        valid_json_number: "42",
        valid_json_string: '"invalid"',
        valid_json_null_entry: '{"src/main.rs": null}',
        valid_json_empty_entry: '{"src/main.rs": {}}',
        valid_json_non_string_hash: '{"src/main.rs": {"hash": 123}}',
      };

      for (const [name, content] of Object.entries(cases)) {
        fs.writeFileSync(path.join(distDir, ".build-manifest.json"), content);
        let result: { fresh: boolean; reason?: string } | undefined;
        expect(() => {
          result = checkInputsFreshness(tempDir, distDir, currentInputs);
        }).not.toThrow();
        expect(result?.fresh, `Case ${name} should return fresh: false`).toBe(
          false,
        );
        expect(result?.reason).toBeDefined();
      }
    } finally {
      fs.rmSync(tempDir, { recursive: true, force: true });
    }
  });
});
