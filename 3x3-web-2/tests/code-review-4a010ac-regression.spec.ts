import { test, expect, type Page } from "@playwright/test";
import {
  parseUrlParams,
  buildShareUrl,
  VALID_SOLVERS,
} from "../web/url-params";
import { getPhaseLabel, PHASE_LABEL_MAP } from "../web/triggers";
import { SOLVED_2X2 } from "../web/model-2x2";

async function ready(page: Page, url = "/?no-sw") {
  await page.goto(url);
  await expect(page.locator("#engine-status")).toContainText("READY");
  await page.locator("#reduced-motion").check();
}

test.describe("HEAD 4a010ac Review Regressions", () => {
  test("H1: VALID_SOLVERS に lbl と ortega が含まれ、URL 解析および共有リンクに反映されること", () => {
    expect(VALID_SOLVERS).toContain("lbl");
    expect(VALID_SOLVERS).toContain("ortega");

    // parseUrlParams の検証
    const parsed = parseUrlParams("?type=2x2&solver=ortega");
    expect(parsed.cubeType).toBe("2x2");
    expect(parsed.solver).toBe("ortega");

    const parsedLbl = parseUrlParams("?type=2x2&solver=lbl");
    expect(parsedLbl.solver).toBe("lbl");

    // buildShareUrl の検証 (2x2 で非デフォルトの ortega を指定)
    const shareUrlOrtega = buildShareUrl(
      "http://localhost:5173/",
      SOLVED_2X2,
      undefined,
      "ortega",
      "2x2",
    );
    expect(shareUrlOrtega).toContain("type=2x2");
    expect(shareUrlOrtega).toContain("solver=ortega");

    // 2x2 でデフォルトの lbl を指定した場合は solver パラメータが省略されること
    const shareUrlLbl = buildShareUrl(
      "http://localhost:5173/",
      SOLVED_2X2,
      undefined,
      "lbl",
      "2x2",
    );
    expect(shareUrlLbl).toContain("type=2x2");
    expect(shareUrlLbl).not.toContain("solver=");
  });

  test("H1: 2x2 で ortega を選択した localStorage 状態がリロード時に正しく復元されること", async ({
    page,
  }) => {
    await page.addInitScript(() => {
      localStorage.setItem(
        "cube-studio-v1",
        JSON.stringify({
          version: 1,
          state: "UUUURRRRFFFFDDDDLLLLBBBB",
          centerTurns: [0, 0, 0, 0, 0, 0],
          cubeType: "2x2",
          reducedMotion: false,
          speed: "500",
          solverAlgorithm: "ortega",
        }),
      );
    });

    await ready(page);

    // キューブタイプが 2x2 であること
    const cubeType2x2Btn = page.locator("#cube-type-2x2");
    await expect(cubeType2x2Btn).toHaveAttribute("aria-checked", "true");

    // セレクトボックスの復元値が ortega であること
    const algoSelect = page.locator("#solver-algorithm");
    await expect(algoSelect).toHaveValue("ortega");
  });

  test("H2: 2x2 キューブで手動回転して完成させた場合に playSuccess が呼ばれること", async ({
    page,
  }) => {
    await ready(page);

    // 2x2 に切り替え
    await page.locator("#cube-type-2x2").click();

    // sound.playSuccess の呼び出しを監視
    await page.evaluate(() => {
      (window as any).__playSuccessCalled = false;
      const soundModule = (window as any).sound;
      if (soundModule) {
        const orig = soundModule.playSuccess.bind(soundModule);
        soundModule.playSuccess = () => {
          (window as any).__playSuccessCalled = true;
          orig();
        };
      }
    });

    // R を回して R' を回して完成状態に戻す
    await page.locator("button[data-move='R']").click();
    await page.waitForTimeout(100);
    await page.locator("#prime").click();
    await page.locator("button[data-move='R']").click();

    // 完成音が呼ばれたことを検証
    await page.waitForFunction(
      () => (window as any).__playSuccessCalled === true,
      {
        timeout: 3000,
      },
    );
  });

  test("M1: 3x3 と 2x2 の切り替えを連続して行っても Three.js リソースが安全に破棄・再構築されること", async ({
    page,
  }) => {
    await ready(page);

    for (let i = 0; i < 3; i++) {
      await page.locator("#cube-type-2x2").click();
      await page.waitForTimeout(50);
      await page.locator("#cube-type-3x3").click();
      await page.waitForTimeout(50);
    }

    const state = await page.evaluate(() =>
      (window as any).cube_store?.getState(),
    );
    expect(state?.length).toBe(54);
  });

  test("M2: 2x2 の JSON 盤面ファイルをインポートした際に正常に読み込めること", async ({
    page,
  }) => {
    await ready(page);

    const jsonContent = JSON.stringify({
      version: 1,
      state: SOLVED_2X2,
      cubeType: "2x2",
    });

    await page.evaluate((content) => {
      const blob = new Blob([content], { type: "application/json" });
      const file = new File([blob], "cube-2x2.json", {
        type: "application/json",
      });
      const dt = new DataTransfer();
      dt.items.add(file);
      const input = document.getElementById("file") as HTMLInputElement;
      input.files = dt.files;
      input.dispatchEvent(new Event("change", { bubbles: true }));
    }, jsonContent);

    await page.waitForTimeout(200);
    const currentState = await page.evaluate(() =>
      (window as any).cube_store?.getState(),
    );
    expect(currentState).toBe(SOLVED_2X2);
  });

  test("M3: triggers.ts の getPhaseLabel が動的フェーズ名（公式名付き）を正しく認識すること", () => {
    const rawPhase = "ステップ 2: 上面色揃え (OLL: Sune (スーネ))";
    const label = getPhaseLabel(rawPhase);
    expect(label).toContain("ステップ 2: 上面色揃え");

    const ortegaPhase = "ステップ 3: 両層同時配置 (PBL: Adj-Adj (隣接-隣接))";
    const ortegaLabel = getPhaseLabel(ortegaPhase);
    expect(ortegaLabel).toContain("ステップ 3: 両層同時配置");
  });

  test("L1: 2x2 展開図エディタのガイド文言に『センター』が含まれないこと", async ({
    page,
  }) => {
    await ready(page);

    // 2x2 に切り替えて色入力タブを開いてエディタを開く
    await page.locator("#cube-type-2x2").click();
    await page.locator('[data-tab="colors"]').click();
    await page.locator("#edit-colors").click();
    await expect(page.locator("#editor")).toBeVisible();

    const guideText = await page.locator("#guide-orientation").textContent();
    expect(guideText).not.toContain("センター");
    expect(guideText).toContain("正面に");
  });

  test("L2: 2x2 選択時の solver-note が適切な説明文言を表示すること", async ({
    page,
  }) => {
    await ready(page);

    await page.locator("#cube-type-2x2").click();
    const noteText = await page.locator("#solver-note").textContent();
    expect(noteText).not.toContain("通常5秒以内");
    expect(noteText).toContain("完全1層");
  });

  test("L3: ヘルプダイアログに 2x2 の 11手 HTM および LBL/Ortega の記載があること", async ({
    page,
  }) => {
    await ready(page);

    const helpDialog = page.locator("#help-dialog");
    const helpContent = await helpDialog.textContent();
    expect(helpContent).toContain("11手");
    expect(helpContent).toContain("LBL法");
    expect(helpContent).toContain("Ortega法");
  });

  test("L4: 2x2 完成状態プリセット読込時に scramble-text がクリアされること", async ({
    page,
  }) => {
    await ready(page);

    await page.locator("#cube-type-2x2").click();
    await page.locator('[data-tab="presets"]').click();

    // まずスクランブルプリセットを読み込む
    const checkerPreset = page.locator("#preset-buttons button", {
      hasText: "チェッカー風",
    });
    await checkerPreset.click();

    const scrambleText = page.locator("#scramble-text");
    await expect(scrambleText).not.toBeEmpty();

    // 完成状態プリセットを読み込む
    const solvedPreset = page.locator("#preset-buttons button", {
      hasText: "完成状態",
    });
    await solvedPreset.click();

    await expect(scrambleText).toBeEmpty();
  });

  test("L5: WASM get_orientations が 2x2 の 24文字盤面に対してコーナー向きを返すこと", async ({
    page,
  }) => {
    await ready(page);

    const result = await page.evaluate((solved2x2) => {
      const studio = (window as any).cube_studio;
      if (!studio || !studio.get_orientations) return null;
      return JSON.parse(studio.get_orientations(solved2x2));
    }, SOLVED_2X2);

    expect(result).not.toBeNull();
    expect(result.corners).toHaveLength(8);
    expect(result.edges).toHaveLength(0);
  });
});
