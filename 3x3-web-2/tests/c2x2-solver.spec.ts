import { test, expect, type Page } from "@playwright/test";

const SOLVED_3X3 = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
const SOLVED_2X2 = "UUUURRRRFFFFDDDDLLLLBBBB";

async function ready(page: Page) {
  await page.goto("/");
  await expect(page.locator("#engine-status")).toContainText("READY");
}

const getState = (page: Page) =>
  page.locator("#scene").getAttribute("data-state");

test.describe("2x2 Rubik's Cube Solver & UI", () => {
  test("switch between 3x3 and 2x2 cube types seamlessly", async ({ page }) => {
    await ready(page);

    // 初期状態は 3x3
    await expect(page.locator("#cube-type-3x3")).toHaveClass(/active/);
    await expect(page.locator("#scene")).toHaveAttribute(
      "data-state",
      SOLVED_3X3,
    );
    await expect(page.locator("#scramble-moves-count")).toHaveText("25手");

    // 2x2 へ切り替え
    await page.locator("#cube-type-2x2").click();
    await expect(page.locator("#cube-type-2x2")).toHaveClass(/active/);
    await expect(page.locator("#cube-type-3x3")).not.toHaveClass(/active/);
    await expect(page.locator("#scene")).toHaveAttribute(
      "data-state",
      SOLVED_2X2,
    );
    await expect(page.locator("#cube-status")).toHaveText("完成状態");
    await expect(page.locator("#scramble-moves-count")).toHaveText("11手");
    await expect(page.locator("#orientation-label-text")).toContainText(
      "標準の向きに揃える",
    );

    // アルゴリズムセレクタが LBL法 / Ortega法 / Optimal 最短探索 になっており選択可能であること
    const algoSelect = page.locator("#solver-algorithm");
    await expect(algoSelect).toHaveValue("lbl");
    await expect(algoSelect).not.toBeDisabled();
    await expect(algoSelect.locator("option")).toHaveCount(3);

    // 3x3 へ再度切り替え
    await page.locator("#cube-type-3x3").click();
    await expect(page.locator("#cube-type-3x3")).toHaveClass(/active/);
    await expect(page.locator("#scene")).toHaveAttribute(
      "data-state",
      SOLVED_3X3,
    );
    await expect(page.locator("#scramble-moves-count")).toHaveText("25手");
  });

  test("2x2 manual moves, undo and redo", async ({ page }) => {
    await ready(page);
    await page.locator("#cube-type-2x2").click();
    await page.locator("#reduced-motion").check();

    // R を回す
    await page.locator('[data-move="R"]').click();
    const stateAfterR = await getState(page);
    expect(stateAfterR).not.toBe(SOLVED_2X2);
    expect(stateAfterR?.length).toBe(24);

    // undo
    await page.locator("#undo").click();
    expect(await getState(page)).toBe(SOLVED_2X2);

    // redo
    await page.locator("#redo").click();
    expect(await getState(page)).toBe(stateAfterR);

    // reset
    await page.locator("#reset").click();
    expect(await getState(page)).toBe(SOLVED_2X2);
  });

  test("2x2 scramble and solve with optimal solution", async ({ page }) => {
    await ready(page);
    await page.locator("#cube-type-2x2").click();
    await page.locator("#reduced-motion").check();
    await page.locator("#solver-algorithm").selectOption("optimal");

    // スクランブル実行
    await page.locator("#scramble").click();
    const scrambled = await getState(page);
    expect(scrambled).not.toBe(SOLVED_2X2);
    expect(scrambled?.length).toBe(24);

    // 解法探索
    await page.locator("#solve").click();
    await expect(page.locator("#solution-content")).toBeVisible();

    // 解法手数が神の数字 11 手以内であることを確認
    const moveCountText = await page.locator("#move-count").textContent();
    const moveCount = parseInt(moveCountText || "99", 10);
    expect(moveCount).toBeLessThanOrEqual(11);
    expect(moveCount).toBeGreaterThan(0);

    // 最後の手順へジャンプ
    await page.locator("#last").click();
    expect(await getState(page)).toBe(SOLVED_2X2);
    await expect(page.locator("#next-instruction")).toContainText("揃いました");

    // 最初の手順へ戻る
    await page.locator("#first").click();
    expect(await getState(page)).toBe(scrambled);
  });

  test("2x2 LBL solver provides 3 distinct learning phases", async ({
    page,
  }) => {
    await ready(page);
    await page.locator("#cube-type-2x2").click();
    await page.locator("#reduced-motion").check();

    // LBL法を選択 (デフォルト)
    const algoSelect = page.locator("#solver-algorithm");
    await expect(algoSelect).toHaveValue("lbl");

    // プリセットから「最難関 (11手)」を選択
    await page.getByRole("tab", { name: "プリセット" }).click();
    await page.getByRole("button", { name: /最難関 \(11手\)/ }).click();

    // 解法を探索
    await page.locator("#solve").click();
    await expect(page.locator("#solution-content")).toBeVisible();

    // 手順リストにフェーズバッジ（完全1層、上面色揃え、上面位置揃え）が含まれること
    const badges = page.locator("#move-list .phase-badge");
    const badgeCount = await badges.count();
    expect(badgeCount).toBeGreaterThanOrEqual(2);

    const badgeTexts = await badges.allTextContents();
    const hasLayer1 = badgeTexts.some((t) => t.includes("完全1層"));
    const hasOll = badgeTexts.some((t) => t.includes("OLL"));
    expect(hasLayer1).toBe(true);
    expect(hasOll).toBe(true);

    // 次の手ガイダンスに学習フェーズが含まれていること
    const instructionText = await page
      .locator("#next-instruction")
      .textContent();
    expect(instructionText).toMatch(/【ステップ/);

    // 最後の手順へジャンプして完成状態を確認
    await page.locator("#last").click();
    expect(await getState(page)).toBe(SOLVED_2X2);
  });

  test("2x2 Ortega solver provides 3 speed-solving phases", async ({
    page,
  }) => {
    await ready(page);
    await page.locator("#cube-type-2x2").click();
    await page.locator("#reduced-motion").check();

    // Ortega法を選択
    const algoSelect = page.locator("#solver-algorithm");
    await algoSelect.selectOption("ortega");
    await expect(algoSelect).toHaveValue("ortega");

    // スクランブル実行
    await page.locator("#scramble").click();

    // 解法を探索
    await page.locator("#solve").click();
    await expect(page.locator("#solution-content")).toBeVisible();

    // 手順リストに Ortega のフェーズバッジ（最初の1面、OLL、PBL）が含まれること
    const badges = page.locator("#move-list .phase-badge");
    const badgeCount = await badges.count();
    expect(badgeCount).toBeGreaterThanOrEqual(2);

    const badgeTexts = await badges.allTextContents();
    const hasFirstFace = badgeTexts.some((t) => t.includes("最初の1面"));
    const hasPblOrOll = badgeTexts.some(
      (t) => t.includes("OLL") || t.includes("PBL"),
    );
    expect(hasFirstFace).toBe(true);
    expect(hasPblOrOll).toBe(true);

    // 最後の手順へジャンプして完成状態を確認
    await page.locator("#last").click();
    expect(await getState(page)).toBe(SOLVED_2X2);
  });

  test("2x2 presets loading", async ({ page }) => {
    await ready(page);
    await page.locator("#cube-type-2x2").click();

    // プリセットタブを開く
    await page.getByRole("tab", { name: "プリセット" }).click();

    // 2x2 専用のプリセットボタンが存在することを確認
    await expect(
      page.getByRole("button", { name: /チェッカー風/ }),
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: /基本手順 \(6手\)/ }),
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: /最難関 \(11手\)/ }),
    ).toBeVisible();

    // チェッカー風をクリック
    await page.getByRole("button", { name: /チェッカー風/ }).click();
    const stateChecker = await getState(page);
    expect(stateChecker).not.toBe(SOLVED_2X2);
    expect(stateChecker?.length).toBe(24);

    // 最難関をクリック
    await page.getByRole("button", { name: /最難関 \(11手\)/ }).click();
    const stateHard = await getState(page);
    expect(stateHard).not.toBe(stateChecker);
    expect(stateHard?.length).toBe(24);

    // 最難関を Optimal で解く
    await page.locator("#solver-algorithm").selectOption("optimal");
    await page.locator("#solve").click();
    await expect(page.locator("#solution-content")).toBeVisible();
    const moveCountText = await page.locator("#move-count").textContent();
    const moveCount = parseInt(moveCountText || "99", 10);
    expect(moveCount).toBeLessThanOrEqual(11);
    expect(moveCount).toBeGreaterThan(0);
  });

  test("2x2 color editor dialog", async ({ page }) => {
    await ready(page);
    await page.locator("#cube-type-2x2").click();

    // 色入力タブを開いて色入力ダイアログを起動
    await page.getByRole("tab", { name: "色を入力" }).click();
    await page.locator("#edit-colors").click();

    await expect(page.locator("#editor")).toBeVisible();

    // 2x2 のときはセンター入力セクションが非表示
    await expect(page.locator("#center-input-section")).toBeHidden();

    // 24 マス入力済み表示
    await expect(page.locator("#color-count")).toContainText(
      "24 / 24 マス入力済み",
    );

    // クリアボタンを押すと 0 / 24 になる
    await page.locator("#clear-colors").click();
    await expect(page.locator("#color-count")).toContainText(
      "0 / 24 マス入力済み",
    );

    // ダイアログを閉じる
    await page.locator("#editor-close").click();
    await expect(page.locator("#editor")).toBeHidden();
  });

  test("2x2 manual moves with normal animation maintain scene graph invariants", async ({
    page,
  }) => {
    await ready(page);
    await page.locator("#cube-type-2x2").click();
    // Note: #reduced-motion is NOT checked (normal animation speed)

    // 2x2 初期状態の不変条件
    const initialCheck = await page.evaluate(() => {
      const sc = (window as any).cube_scene;
      return {
        arrowGroupVisible: sc?.arrowGroup?.visible,
        allArrowsHidden: sc?.arrowMeshes?.every((m: any) => !m.visible),
        allOutlinesHidden: sc?.outlineMeshes?.every((m: any) => !m.visible),
        centerLabelsCount: sc?.centerLabels?.length ?? 0,
      };
    });
    expect(initialCheck.arrowGroupVisible).toBe(false);
    expect(initialCheck.allArrowsHidden).toBe(true);
    expect(initialCheck.allOutlinesHidden).toBe(true);
    expect(initialCheck.centerLabelsCount).toBe(0);

    // 通常アニメーション中のシーングラフを 16ms ごとにサンプリング監視
    const samplePromise = page.evaluate(() => {
      return new Promise<{
        violations: string[];
        sampledFrames: number;
      }>((resolve) => {
        const sc = (window as any).cube_scene;
        const violations: string[] = [];
        let sampledFrames = 0;

        const check = () => {
          if (!sc) return;
          sampledFrames++;
          if (sc.arrowGroup?.visible) {
            violations.push("arrowGroup was visible");
          }
          if (sc.arrowMeshes?.some((m: any) => m.visible)) {
            violations.push("some arrowMesh was visible");
          }
          if (sc.outlineMeshes?.some((m: any) => m.visible)) {
            violations.push("some outlineMesh was visible");
          }

          if (sc.active) {
            const layerChildren: any[] = sc.turnLayer?.children ?? [];
            // 2x2 の回転層は 4 つのコーナーブロック（各ブロックはボディ1+ステッカー3 = 計4メッシュ、合計16メッシュ）
            if (layerChildren.length !== 16) {
              violations.push(
                `turnLayer has unexpected children count: ${layerChildren.length} (expected 16)`,
              );
            }
            // すべての子要素が sc.pieces (ボディ・ステッカー) のみであること
            if (layerChildren.some((c) => !sc.pieces?.includes(c))) {
              violations.push("turnLayer contained objects not in sc.pieces");
            }
            // 矢印やラベルが混入していないこと
            if (
              layerChildren.some(
                (c) =>
                  sc.arrowMeshes?.includes(c) ||
                  sc.outlineMeshes?.includes(c) ||
                  sc.centerLabels?.includes(c),
              )
            ) {
              violations.push("turnLayer contained arrow or label mesh");
            }
          }
        };

        const timer = setInterval(check, 16);
        setTimeout(() => {
          clearInterval(timer);
          resolve({ violations, sampledFrames });
        }, 500);
      });
    });

    // R 回転を手動実行
    await page.locator('[data-move="R"]').click();
    const result = await samplePromise;

    expect(result.sampledFrames).toBeGreaterThan(5);
    expect(result.violations).toEqual([]);

    // 回転終了を待つ
    await page.waitForFunction(() => {
      const sc = (window as any).cube_scene;
      return !sc?.active;
    });

    // 回転終了後、turnLayer が空であること
    const postCheck = await page.evaluate(() => {
      const sc = (window as any).cube_scene;
      return {
        turnLayerEmpty: sc?.turnLayer?.children?.length === 0,
        arrowGroupVisible: sc?.arrowGroup?.visible,
      };
    });
    expect(postCheck.turnLayerEmpty).toBe(true);
    expect(postCheck.arrowGroupVisible).toBe(false);
  });

  test("2x2 solver solution playback does not leak arrows or labels during animation", async ({
    page,
  }) => {
    await ready(page);
    await page.locator("#cube-type-2x2").click();
    // Note: #reduced-motion is NOT checked (normal animation)

    // プリセットから「基本手順 (6手)」を選択
    await page.getByRole("tab", { name: "プリセット" }).click();
    await page.getByRole("button", { name: /基本手順 \(6手\)/ }).click();

    // 解法を探索
    await page.locator("#solve").click();
    await expect(page.locator("#solution-content")).toBeVisible();

    // 再生中のサンプリング監視をセットアップ
    const playbackSamplingPromise = page.evaluate(() => {
      return new Promise<{
        violations: string[];
        turnsDetected: number;
      }>((resolve) => {
        const sc = (window as any).cube_scene;
        const violations: string[] = [];
        let turnsDetected = 0;
        let wasActive = false;

        const check = () => {
          if (!sc) return;
          if (sc.active) {
            if (!wasActive) {
              turnsDetected++;
              wasActive = true;
            }
            const layerChildren: any[] = sc.turnLayer?.children ?? [];
            if (layerChildren.length !== 16) {
              violations.push(
                `turnLayer has unexpected children count: ${layerChildren.length} during playback`,
              );
            }
            if (layerChildren.some((c) => !sc.pieces?.includes(c))) {
              violations.push(
                "turnLayer contained objects not in sc.pieces during playback",
              );
            }
            if (
              layerChildren.some(
                (c) =>
                  sc.arrowMeshes?.includes(c) ||
                  sc.outlineMeshes?.includes(c) ||
                  sc.centerLabels?.includes(c),
              )
            ) {
              violations.push(
                "turnLayer contained arrow or label during playback",
              );
            }
          } else {
            wasActive = false;
          }

          if (sc.arrowGroup?.visible) {
            violations.push("arrowGroup was visible during playback");
          }
        };

        const timer = setInterval(check, 16);
        setTimeout(() => {
          clearInterval(timer);
          resolve({ violations, turnsDetected });
        }, 1200);
      });
    });

    // 再生ボタンを押す
    await page.locator("#play").click();
    const playbackResult = await playbackSamplingPromise;

    // 一時停止
    await page.locator("#play").click();

    expect(playbackResult.turnsDetected).toBeGreaterThan(0);
    expect(playbackResult.violations).toEqual([]);

    // 1手進める（Next）操作時もチェック
    const stepPromise = page.evaluate(async () => {
      const sc = (window as any).cube_scene;
      let leakFound = false;
      const check = () => {
        if (
          sc?.arrowGroup?.visible ||
          sc?.turnLayer?.children?.some(
            (c: any) =>
              sc.arrowMeshes?.includes(c) || sc.outlineMeshes?.includes(c),
          )
        ) {
          leakFound = true;
        }
      };
      const timer = setInterval(check, 10);
      await new Promise((r) => setTimeout(r, 350));
      clearInterval(timer);
      return leakFound;
    });

    await page.locator("#next").click();
    const stepLeak = await stepPromise;
    expect(stepLeak).toBe(false);
  });
});
