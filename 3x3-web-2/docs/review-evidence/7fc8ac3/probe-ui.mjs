import { chromium } from "@playwright/test";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../../..",
);
const base = "http://127.0.0.1:5173";
const browser = await chromium.launch({ headless: true });
const output = {};

const SUPERFLIP_STATE =
  "UBULURUFURURFRBRDRFUFLFRFDFDFDLDRDBDLULBLFLDLBUBRBLBDB";

try {
  // ==========================================
  // Finding 1: 解法プレビューシーク中に「保存」（#save）を押すと、スクランブルではなくシーク途中局面（完成状態）が保存される
  // ==========================================
  const pageF1 = await browser.newPage();
  await pageF1.goto(`${base}/?no-sw`);
  await pageF1.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

  // 1手スクランブル (R)
  await pageF1.locator('button[data-move="R"]').click();
  await pageF1.waitForTimeout(200);

  const scrambleState = await pageF1.evaluate(
    () => window.__cube_main_debug__?.store?.getState(),
  );

  // 解法探索
  await pageF1.locator("#solve").click();
  await pageF1.locator("#move-list button").first().waitFor();

  // 最終手（完成状態）へシーク
  await pageF1.locator("#last").click();
  await pageF1.waitForTimeout(200);

  const stepState = await pageF1.evaluate(
    () => window.__cube_main_debug__?.store?.getState(),
  );

  // #save ボタン押下時の createObjectURL をインターセプトして保存内容を捕捉
  await pageF1.evaluate(() => {
    const origCreate = URL.createObjectURL;
    URL.createObjectURL = (blob) => {
      blob.text().then((text) => {
        (window).__lastSavedJson = JSON.parse(text);
      });
      return origCreate(blob);
    };
  });
  await pageF1.locator("#save").click();
  await pageF1.waitForTimeout(300);

  const savedJson = await pageF1.evaluate(() => (window).__lastSavedJson);

  output.f1SaveDuringPreview = {
    scrambleState,
    stepState,
    isStepStateSolved: stepState?.startsWith("UUUUUUUUU"),
    savedJsonState: savedJson.state,
    savedContainsScrambleState: savedJson.state === scrambleState,
    savedContainsStepState: savedJson.state === stepState,
  };
  await pageF1.close();

  // ==========================================
  // Finding 2: 解法パネルの「×」（#solution-close）押下時に、一時プレビュー局面がキューブ実盤面に勝手にコミットされる
  // ==========================================
  const pageF2 = await browser.newPage();
  await pageF2.goto(`${base}/?no-sw`);
  await pageF2.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

  await pageF2.locator('button[data-move="R"]').click();
  await pageF2.waitForTimeout(200);

  const scrambleStateF2 = await pageF2.evaluate(
    () => window.__cube_main_debug__?.store?.getState(),
  );

  await pageF2.locator("#solve").click();
  await pageF2.locator("#move-list button").first().waitFor();

  // 完成状態までプレビューを進める
  await pageF2.locator("#last").click();
  await pageF2.waitForTimeout(200);

  // 解法カードの「×」（閉じる）ボタンを押す
  await pageF2.locator("#solution-close").click();
  await pageF2.waitForTimeout(200);

  const stateAfterClose = await pageF2.evaluate(
    () => window.__cube_main_debug__?.store?.getState(),
  );
  const statusTextAfterClose = await pageF2.locator("#cube-status").textContent();
  const solveButtonText = await pageF2.locator("#solve").textContent();

  output.f2SolutionCloseCommitsPreviewState = {
    scrambleState: scrambleStateF2,
    stateAfterClose,
    isRestoredToScramble: stateAfterClose === scrambleStateF2,
    isCommittedToPreviewState: stateAfterClose?.startsWith("UUUUUUUUU"),
    statusTextAfterClose,
    solveButtonText,
  };
  await pageF2.close();

  // ==========================================
  // Finding 3: プリセット通信待機中の solve() 開始による非同期レースコンディション
  // ==========================================
  const pageF3 = await browser.newPage();
  await pageF3.goto(`${base}/?no-sw`);
  await pageF3.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

  const f3RaceResult = await pageF3.evaluate(async () => {
    const store = window.__cube_main_debug__?.store;
    const initialRev = store?.getRevision();

    // プリセット fetch を遅延モックしてシミュレート
    let fetchResolver;
    const slowFetchPromise = new Promise((resolve) => {
      fetchResolver = resolve;
    });

    const originalFetch = window.fetch;
    window.fetch = async (...args) => {
      if (typeof args[0] === "string" && args[0].includes("superflip.json")) {
        await slowFetchPromise;
        return new Response(
          JSON.stringify({
            state: "UBULURUFURURFRBRDRFUFLFRFDFDFDLDRDBDLULBLFLDLBUBRBLBDB",
          }),
          { status: 200, headers: { "Content-Type": "application/json" } },
        );
      }
      return originalFetch(...args);
    };

    // 1. プリセットボタンをクリック
    const presetBtn = document.querySelector("#preset-buttons button:nth-child(2)");
    presetBtn?.click();

    // 2. fetch 待機中にユーザーが solve() を呼ぶ
    const solvePromise = window.__cube_main_debug__?.solve(5000);

    // solve() 開始直後の revision を確認
    const revDuringSolve = store?.getRevision();

    // 3. fetch を完了させる
    fetchResolver();
    await new Promise((r) => setTimeout(r, 100));

    window.fetch = originalFetch;

    return {
      initialRev,
      revDuringSolve,
      revisionChangedOnSolveStart: revDuringSolve !== initialRev,
      presetStatus: document.getElementById("preset-status")?.textContent,
    };
  });

  output.f3PresetRaceDuringSolve = f3RaceResult;
  await pageF3.close();

  // ==========================================
  // Finding 4: 空入力または空白のみでの applyAlgorithm による無駄な解法破棄と効果音副作用
  // ==========================================
  const pageF4 = await browser.newPage();
  await pageF4.goto(`${base}/?no-sw`);
  await pageF4.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

  await pageF4.locator('button[data-move="R"]').click();
  await pageF4.locator("#solve").click();
  await pageF4.locator("#move-list button").first().waitFor();

  const solutionBefore = await pageF4.evaluate(
    () => !!window.__cube_main_debug__?.store?.getSolution(),
  );

  // 手順入力タブを開いてから空白で「手順を適用」を押す
  await pageF4.locator('[data-tab="moves"]').click();
  await pageF4.locator("#algorithm").fill("   ");
  await pageF4.locator("#apply-algorithm").click();
  await pageF4.waitForTimeout(200);

  const solutionAfter = await pageF4.evaluate(
    () => !!window.__cube_main_debug__?.store?.getSolution(),
  );

  output.f4EmptyAlgorithmDestroysSolution = {
    solutionExistedBefore: solutionBefore,
    solutionExistedAfter: solutionAfter,
    solutionWasDestroyed: solutionBefore && !solutionAfter,
  };
  await pageF4.close();

  // ==========================================
  // Finding 5: 探索中（solving = true）の修飾子ボタン有効化とキーボード active-press 付与
  // ==========================================
  const pageF5 = await browser.newPage();
  await pageF5.goto(`${base}/?no-sw`);
  await pageF5.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

  await pageF5.evaluate((state) => {
    window.__cube_main_debug__?.store?.replace(state);
  }, SUPERFLIP_STATE);
  await pageF5.locator("#solver-algorithm").selectOption("korf");
  await pageF5.locator("#solve").click();
  await pageF5.locator("#cancel").waitFor({ state: "visible" });

  const primeDisabled = await pageF5.locator("#prime").isDisabled();
  const doubleDisabled = await pageF5.locator("#double").isDisabled();

  // 探索中にキーボードで 'U' を押し下げた状態で active-press クラスを検査
  await pageF5.keyboard.down("KeyU");
  await pageF5.waitForTimeout(50);

  const uBtnActivePress = await pageF5
    .locator('button[data-move="U"]')
    .evaluate((el) => el.classList.contains("active-press"));

  await pageF5.keyboard.up("KeyU");

  output.f5ModifierAndActivePressDuringSolve = {
    primeDisabled,
    doubleDisabled,
    uBtnActivePressWhileDisabled: uBtnActivePress,
  };
  await pageF5.locator("#cancel").click();
  await pageF5.close();
} finally {
  await browser.close();
  const outPath = path.resolve(
    path.dirname(fileURLToPath(import.meta.url)),
    "probe-ui.json",
  );
  fs.writeFileSync(outPath, JSON.stringify(output, null, 2));
  console.log(`Saved probe results to ${outPath}`);
}
