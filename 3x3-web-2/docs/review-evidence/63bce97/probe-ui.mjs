import { chromium } from "@playwright/test";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../../..",
);
const base = "http://localhost:5173";
const browser = await chromium.launch({ headless: true });
const output = {};

try {
  // ==========================================
  // Probe F1: 0手解法（完成状態）時のタイムライン aria-valuetext
  // ==========================================
  const pageF1 = await browser.newPage();
  await pageF1.goto(`${base}/?no-sw`);
  await pageF1.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

  // 完成状態（SOLVED）のまま解法探索を実行
  await pageF1.locator("#solve").click();
  // 解法完了メッセージまたは 0 / 0 手表示を待つ
  await pageF1.locator("#message").filter({ hasText: "すでに6面が揃っています" }).waitFor();

  output.f1Timeline = await pageF1.evaluate(() => {
    const timeline = document.querySelector("#timeline");
    const stepCount = document.querySelector("#step-count")?.textContent;
    const nextInstruction = document.querySelector("#next-instruction")?.textContent;
    const valuetext = timeline?.getAttribute("aria-valuetext");
    const valuenow = timeline?.getAttribute("aria-valuenow");
    const max = timeline?.getAttribute("max");
    return {
      valuetext,
      valuenow,
      max,
      stepCount,
      nextInstruction,
      expectedText: "完成 (0手)",
      isMisleading: valuetext === "開始状態",
    };
  });
  await pageF1.close();

  // ==========================================
  // Probe F2: 解法再探索中のタイムラインシークによる結果破棄レース
  // ==========================================
  const pageF2 = await browser.newPage();
  // 探索に時間がかかる局面 (12手スクランブル)
  await pageF2.goto(`${base}/?no-sw&alg=B+L2+D+F2+R2+B2+U+L2+D+R2+F2+U'`);
  await pageF2.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

  // 1回目の解法探索
  await pageF2.locator("#solve").click();
  await pageF2.locator("#move-list button").first().waitFor();

  const initialSolution = await pageF2.evaluate(() => ({
    moveCount: document.querySelector("#move-count")?.textContent,
    stepCount: document.querySelector("#step-count")?.textContent,
    hasSolution: document.body.classList.contains("has-solution"),
    playDisabled: document.querySelector("#play")?.disabled,
    timelineDisabled: document.querySelector("#timeline")?.disabled,
  }));

  // ここでソルバーアルゴリズムを変更するか、重い探索（CFOPなど）で再探索を開始
  await pageF2.locator("#solver-algorithm").selectOption("cfop");

  // 再探索をクリックする直前の revision
  const beforeSolveRevision = await pageF2.evaluate(() => window.__cube_main_debug__?.store?.getRevision());

  // solve を実行（バックグラウンドで走らせる）
  await pageF2.locator("#solve").click();

  // 探索中の UI 状態を検査（古い解法が表示されたまま、かつシークボタンが押せるか）
  output.f2DuringSolve = await pageF2.evaluate(() => {
    const playBtn = document.querySelector("#play");
    const timeline = document.querySelector("#timeline");
    const nextBtn = document.querySelector("#next");
    const moveButtons = document.querySelectorAll("#move-list button");
    return {
      solving: document.querySelector("#solve").hidden,
      cancelVisible: !document.querySelector("#cancel").hidden,
      solutionContentVisible: !document.querySelector("#solution-content").hidden,
      playDisabled: playBtn ? playBtn.disabled : null,
      timelineDisabled: timeline ? timeline.disabled : null,
      nextDisabled: nextBtn ? nextBtn.disabled : null,
      moveButtonsCount: moveButtons.length,
    };
  });

  // 探索中にタイムラインで 1 手シークする（これにより store.revision が進む）
  await pageF2.evaluate(() => window.__cube_main_debug__?.seek(1, false));

  const revisionAfterSeek = await pageF2.evaluate(() => window.__cube_main_debug__?.store?.getRevision());

  // 探索完了（cancel ボタンが非表示になる）まで待つ
  await pageF2.locator("#solve").waitFor({ state: "visible" });

  output.f2AfterSolve = await pageF2.evaluate(() => {
    const note = document.querySelector("#solver-note")?.textContent;
    const solution = window.__cube_main_debug__?.store?.getSolution();
    return {
      note,
      solutionAlgorithm: solution?.algorithm,
      solutionMoves: solution?.moves,
      // 探索ノートが「ノードを探索 · 完成を検証」でなく「探索中」のまま、または解法が更新されていないか
      discarded: !note?.includes("ノードを探索 · 完成を検証"),
    };
  });
  output.f2Data = {
    initialSolution,
    beforeSolveRevision,
    revisionAfterSeek,
    revisionChangedDuringSolve: revisionAfterSeek !== beforeSolveRevision,
  };
  await pageF2.close();

  // ==========================================
  // Probe F3: カメラエディタ閉じた後の DOM ステータスと内部状態の乖離
  // ==========================================
  const pageF3 = await browser.newPage();
  await pageF3.goto(`${base}/?no-sw`);
  await pageF3.locator("#engine-status").filter({ hasText: "READY" }).waitFor();
  await pageF3.getByRole("tab", { name: "色を入力" }).click();
  await pageF3.locator("#camera-colors").click();
  await pageF3.locator("#camera-editor").waitFor({ state: "visible" });

  // 画像Aを読み込む
  await pageF3
    .locator("#camera-file-a")
    .setInputFiles(path.join(root, "test-images/solved-view-a.png"));
  await pageF3.waitForFunction(() => window.__lastCamera?.imageA !== undefined);

  output.f3BeforeClose = await pageF3.evaluate(() => ({
    hasImageA: window.__lastCamera?.imageA !== undefined,
    statusTextA: document.querySelector("#camera-status-a")?.textContent,
    cardHasClass: document.querySelector("#camera-drop-a")?.classList.contains("has-file"),
    inputAValue: document.querySelector("#camera-file-a")?.value,
  }));

  // ダイアログを閉じる（Escキーまたは閉じるボタン）
  await pageF3.locator("#camera-close").click();
  await pageF3.locator("#camera-editor").waitFor({ state: "hidden" });

  output.f3AfterClose = await pageF3.evaluate(() => ({
    hasImageA: window.__lastCamera?.imageA !== undefined,
    statusTextA: document.querySelector("#camera-status-a")?.textContent,
    cardHasClass: document.querySelector("#camera-drop-a")?.classList.contains("has-file"),
    inputAValue: document.querySelector("#camera-file-a")?.value,
    facesLength: Object.keys(window.__lastCamera?.faces || {}).length,
    inconsistent:
      window.__lastCamera?.imageA === undefined &&
      document.querySelector("#camera-status-a")?.textContent?.includes("読込完了"),
  }));
  await pageF3.close();

  // ==========================================
  // Probe F4: 探索中プリセット押下時の二重非同期レース
  // ==========================================
  const pageF4 = await browser.newPage();
  await pageF4.goto(`${base}/?no-sw&alg=R+U+F+D+L+B`);
  await pageF4.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

  // 重い探索（CFOP）を開始
  await pageF4.locator("#solver-algorithm").selectOption("cfop");
  await pageF4.locator("#solve").click();

  // 探索が走っていることを確認
  await pageF4.waitForFunction(() => !document.querySelector("#cancel").hidden);

  // 探索中にプリセットボタン（簡単 3手）をクリック
  await pageF4.locator("#tab-presets").click();
  await pageF4
    .locator("#preset-buttons button")
    .filter({ hasText: "簡単（3手）" })
    .click();

  output.f4DuringPreset = await pageF4.evaluate(() => ({
    solvingStillActive: !document.querySelector("#cancel").hidden,
    presetStatus: document.querySelector("#preset-status")?.textContent,
  }));

  // プリセット完了まで待機
  await pageF4.locator("#preset-status").filter({ hasText: "簡単（3手） を読み込みました" }).waitFor();

  output.f4AfterPreset = await pageF4.evaluate(() => ({
    solving: !document.querySelector("#solve").hidden,
    state: window.cube_studio?.store?.getState(),
    presetStatus: document.querySelector("#preset-status")?.textContent,
    solverNote: document.querySelector("#solver-note")?.textContent,
  }));
  await pageF4.close();

} catch (err) {
  output.error = String(err?.stack || err);
} finally {
  await browser.close();
  const evidencePath = path.join(root, "docs/review-evidence/63bce97/probe-ui.json");
  fs.writeFileSync(evidencePath, JSON.stringify(output, null, 2), "utf-8");
  console.log(`Saved probe evidence to ${evidencePath}`);
}
