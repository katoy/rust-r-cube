import { test, expect } from "@playwright/test";
import path from "node:path";

const getTestImagePath = (name: string, view: "A" | "B") =>
  path.resolve(
    process.cwd(),
    `test-images/${name}-view-${view.toLowerCase()}.png`,
  );

test.describe("63bce97 レビュー指摘点 (F1〜F4) 回帰テスト", () => {
  // F1: 解法再探索中のシーク・再生操作による新探索結果破棄の防止
  test("F1: 解法表示中に再探索を開始した際、古い解法が失効し、再生・シーク操作が無効化されて探索完了後に新解法が確実に反映される", async ({
    page,
  }) => {
    // 12手スクランブル局面
    await page.goto("/?no-sw&alg=B+L2+D+F2+R2+B2+U+L2+D+R2+F2+U'");
    await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

    // 1回目の解法探索 (Kociemba)
    await page.locator("#solve").click();
    await page.locator("#move-list button").first().waitFor();
    await expect(page.locator("#move-count")).toContainText("手");

    // ソルバーアルゴリズムを CFOP に切り替え
    await page.locator("#solver-algorithm").selectOption("cfop");

    // 2回目の探索を開始
    await page.locator("#solve").click();

    // 探索中はキャンセルボタンが表示され、古い解法コンテンツが非表示になること
    await expect(page.locator("#cancel")).toBeVisible();
    await expect(page.locator("#solution-content")).toBeHidden();

    // 探索中にキーボードの左右キー（シーク）やスペースキー（再生）を押しても無効であること
    await page.keyboard.press("ArrowRight");
    await page.keyboard.press("Space");

    // 探索完了を待つ
    await page.locator("#solve").waitFor({ state: "visible" });
    await expect(page.locator("#cancel")).toBeHidden();

    // 探索完了後、新しい CFOP 解法が正しく反映されていること
    const solution = await page.evaluate(() =>
      (window as any).__cube_main_debug__?.store?.getSolution(),
    );
    expect(solution?.algorithm).toBe("cfop");
    expect(solution?.moves.length).toBeGreaterThan(0);
    await expect(page.locator("#solution-content")).toBeVisible();
    await expect(page.locator("#solver-note")).toContainText(
      "探索 · 完成を検証",
    );
  });

  // F2: カメラエディタ終了後、DOM カードステータス・クラス・ファイル入力値の一貫した初期化
  test("F2: カメラエディタで画像を読み込んだ後、ダイアログを閉じると DOM カードステータスが未選択に戻り、input.value がクリアされて同名ファイルを再選択可能になる", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();
    await page.getByRole("tab", { name: "色を入力" }).click();
    await page.locator("#camera-colors").click();
    await page.locator("#camera-editor").waitFor({ state: "visible" });

    // 画像Aを読み込む
    const imagePath = getTestImagePath("solved", "A");
    await page.locator("#camera-file-a").setInputFiles(imagePath);
    await page.waitForFunction(
      () => (window as any).__lastCamera?.imageA !== undefined,
    );

    // 読み込み完了状態を確認
    await expect(page.locator("#camera-status-a")).toContainText("読込完了");
    await expect(page.locator("#camera-drop-a")).toHaveClass(/has-file/);

    // ダイアログを閉じる（閉じるボタン）
    await page.locator("#camera-close").click();
    await page.locator("#camera-editor").waitFor({ state: "hidden" });

    // ダイアログクローズ直後の状態を検査：DOM が未選択に戻り、クラスが除去され、input.value が空であること
    const closedState = await page.evaluate(() => {
      const statusText =
        document.querySelector("#camera-status-a")?.textContent;
      const hasFileClass = document
        .querySelector("#camera-drop-a")
        ?.classList.contains("has-file");
      const inputValue = (
        document.querySelector("#camera-file-a") as HTMLInputElement
      )?.value;
      const camera = (window as any).__lastCamera;
      return {
        statusText,
        hasFileClass,
        inputValue,
        hasImageA: camera?.imageA !== undefined,
        facesCount: Object.keys(camera?.faces || {}).length,
      };
    });

    expect(closedState.statusText).toContain("未選択");
    expect(closedState.hasFileClass).toBe(false);
    expect(closedState.inputValue).toBe("");
    expect(closedState.hasImageA).toBe(false);
    expect(closedState.facesCount).toBe(0);

    // 再度カメラエディタを開き、同じ画像ファイルを再選択した際に正しく読み込めること
    await page.locator("#camera-colors").click();
    await page.locator("#camera-editor").waitFor({ state: "visible" });
    await page.locator("#camera-file-a").setInputFiles(imagePath);
    await page.waitForFunction(
      () => (window as any).__lastCamera?.imageA !== undefined,
    );
    await expect(page.locator("#camera-status-a")).toContainText("読込完了");
    await expect(page.locator("#camera-drop-a")).toHaveClass(/has-file/);

    // Escape キーで閉じた場合も同様に初期化されること
    await page.keyboard.press("Escape");
    await page.locator("#camera-editor").waitFor({ state: "hidden" });
    const escapeClosedState = await page.evaluate(() => ({
      statusText: document.querySelector("#camera-status-a")?.textContent,
      hasFileClass: document
        .querySelector("#camera-drop-a")
        ?.classList.contains("has-file"),
      hasImageA: (window as any).__lastCamera?.imageA !== undefined,
    }));
    expect(escapeClosedState.statusText).toContain("未選択");
    expect(escapeClosedState.hasFileClass).toBe(false);
    expect(escapeClosedState.hasImageA).toBe(false);
  });

  // F3: 0手解法（完成状態）のタイムライン aria-valuetext が「完成 (0手)」と正確に設定される
  test("F3: 完成状態で解法探索を行った際、タイムラインの aria-valuetext が『完成 (0手)』と設定される", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

    // 完成状態（SOLVED）のまま解法探索を実行
    await page.locator("#solve").click();
    await page
      .locator("#message")
      .filter({ hasText: "すでに6面が揃っています" })
      .waitFor();

    // タイムラインのアクセシビリティ属性を検証
    const timelineProps = await page.evaluate(() => {
      const timeline = document.querySelector("#timeline");
      return {
        valuetext: timeline?.getAttribute("aria-valuetext"),
        valuenow: timeline?.getAttribute("aria-valuenow"),
        max: timeline?.getAttribute("max"),
        disabled: (timeline as HTMLInputElement)?.disabled,
        stepCount: document.querySelector("#step-count")?.textContent,
      };
    });

    expect(timelineProps.valuetext).toBe("完成 (0手)");
    expect(timelineProps.valuenow).toBe("0");
    expect(timelineProps.max).toBe("0");
    expect(timelineProps.stepCount).toBe("0 / 0");

    // 2手解法（R U）で開始・途中・完成の各 aria-valuetext も正確に動作することを確認
    await page.goto("/?no-sw&alg=R+U");
    await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();
    await page.locator("#solve").click();
    await page.locator("#move-list button").first().waitFor();

    // step 0 (開始状態)
    const step0Valuetext = await page
      .locator("#timeline")
      .getAttribute("aria-valuetext");
    expect(step0Valuetext).toContain("開始状態。次は1手目");

    // step 1 (1手完了)
    await page.locator("#next").click();
    const step1Valuetext = await page
      .locator("#timeline")
      .getAttribute("aria-valuetext");
    expect(step1Valuetext).toContain("1手完了。次は2手目");

    // step 2 (完成 2手)
    await page.locator("#next").click();
    const step2Valuetext = await page
      .locator("#timeline")
      .getAttribute("aria-valuetext");
    expect(step2Valuetext).toBe("完成 (2手)");
  });

  // F4: 探索中にプリセットを選択した際、即座に探索が中断されステータス表示が競合しない
  test("F4: 探索中にプリセットを選択した際、即座に探索が中止され、プリセット完了後に旧探索のノード数が上書きされない", async ({
    page,
  }) => {
    // 探索に時間のかかる複雑な局面 (12手スクランブル)
    await page.goto("/?no-sw&alg=B+L2+D+F2+R2+B2+U+L2+D+R2+F2+U'");
    await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

    // プリセットタブを事前に開いておく（探索開始後即座にクリックできるようにする）
    await page.locator("#tab-presets").click();
    await page.locator("#preset-buttons button").first().waitFor();

    // Korf 最短探索を開始（IDA* により確実に数秒以上探索が走る）
    await page.locator("#solver-algorithm").selectOption("korf");
    await page.locator("#solve").click();

    // 探索中であることを確認
    await expect(page.locator("#cancel")).toBeVisible();

    // 探索がアクティブな状態のまま即座にプリセット「簡単（3手）」をクリック
    await page
      .locator("#preset-buttons button")
      .filter({ hasText: "簡単（3手）" })
      .click();

    // プリセット読み込み完了を待機
    await page
      .locator("#preset-status")
      .filter({ hasText: "簡単（3手） を読み込みました" })
      .waitFor();

    // 探索が即座に中断され、solve ボタンが再表示されていること
    await expect(page.locator("#solve")).toBeVisible();
    await expect(page.locator("#cancel")).toBeHidden();

    // 旧探索のノード数通知で solver-note が上書きされていないこと
    const noteText = await page.locator("#solver-note").textContent();
    expect(noteText).not.toContain("ノードを探索 · 完成を検証");
  });
});
