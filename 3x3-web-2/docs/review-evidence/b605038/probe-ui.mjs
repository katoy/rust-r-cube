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

try {
  // ==========================================
  // Finding 1: 解法プレビューシーク後の再探索で、スクランブル局面ではなくシーク後局面が探索対象になる
  // ==========================================
  const pageF1 = await browser.newPage();
  const scrambleAlg = "B L2 D F2 R2 B2 U L2 D R2 F2 U'";
  await pageF1.goto(`${base}/?no-sw&alg=${encodeURIComponent(scrambleAlg)}`);
  await pageF1.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

  const originalState = await pageF1.evaluate(() =>
    window.__cube_main_debug__?.store?.getState(),
  );

  // 1回目の解法探索 (Kociemba)
  await pageF1.locator("#solve").click();
  await pageF1.locator("#move-list button").first().waitFor();

  // タイムラインで 5手目へシーク
  await pageF1.evaluate(() => window.__cube_main_debug__?.seek(5, false));
  const stateAtStep5 = await pageF1.evaluate(() =>
    window.__cube_main_debug__?.store?.getState(),
  );

  // ソルバーアルゴリズムを CFOP に変更して再探索
  await pageF1.locator("#solver-algorithm").selectOption("cfop");
  await pageF1.locator("#solve").click();
  await pageF1.locator("#solve").waitFor({ state: "visible" });

  const solution2 = await pageF1.evaluate(() =>
    window.__cube_main_debug__?.store?.getSolution(),
  );

  output.f1PreviewSeekSolveRace = {
    originalState,
    stateAtStep5,
    solution2StartState: solution2?.states[0],
    solvedWrongState: solution2?.states[0] === stateAtStep5,
    algorithm: solution2?.algorithm,
    movesCount: solution2?.moves?.length,
  };
  await pageF1.close();

  // ==========================================
  // Finding 2: 探索中に Undo をクリックすると探索がキャンセルされず、UIが拘束されたままバックグラウンドで古い探索が継続する
  // ==========================================
  const pageF2 = await browser.newPage();
  await pageF2.goto(`${base}/?no-sw`);
  await pageF2.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

  await pageF2.getByRole("tab", { name: "プリセット" }).click();
  await pageF2.getByRole("button", { name: "スーパーフリップ" }).click();
  await pageF2.locator("#preset-status").filter({ hasText: "読み込みました" }).waitFor();

  await pageF2.locator('button[data-move="R"]').click();
  await pageF2.waitForTimeout(300);

  await pageF2.locator("#solver-algorithm").selectOption("korf");
  await pageF2.locator("#solve").click();
  await pageF2.locator("#cancel").waitFor({ state: "visible" });

  const undoDisabledDuringSolve = await pageF2.locator("#undo").isDisabled();
  await pageF2.locator("#undo").click();

  output.f2UndoDuringSolve = await pageF2.evaluate(() => {
    const cancelBtn = document.querySelector("#cancel");
    const solveBtn = document.querySelector("#solve");
    const solverNote = document.querySelector("#solver-note")?.textContent;
    const solverClient = window.__cube_main_debug__?.getSolver();
    return {
      undoDisabledDuringSolve: false,
      cancelVisible: !cancelBtn?.hasAttribute("hidden"),
      solveHidden: solveBtn?.hasAttribute("hidden"),
      solverNote,
      solverClientPending: (solverClient as any)?.pending !== undefined,
    };
  });
  output.f2UndoDuringSolve.undoDisabledDuringSolve = undoDisabledDuringSolve;
  await pageF2.locator("#cancel").click();
  await pageF2.close();

  // ==========================================
  // Finding 3: 探索中に「色を入力」を開くと探索がキャンセルされず、ダイアログ背後で解法がセットされる
  // ==========================================
  const pageF3 = await browser.newPage();
  await pageF3.goto(`${base}/?no-sw&alg=R+U+R'+U'`);
  await pageF3.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

  await pageF3.locator("#solve").click();
  await pageF3.getByRole("tab", { name: "色を入力" }).click();
  await pageF3.locator("#edit-colors").click();
  await pageF3.locator("#editor").waitFor({ state: "visible" });

  await pageF3.waitForTimeout(1000);

  output.f3EditorDuringSolve = await pageF3.evaluate(() => {
    const editorOpen = (document.querySelector("#editor") as HTMLDialogElement)?.open;
    const solution = window.__cube_main_debug__?.store?.getSolution();
    const hasSolutionClass = document.body.classList.contains("has-solution");
    return {
      editorOpen,
      hasSolution: solution !== undefined,
      solutionMoves: solution?.moves,
      hasSolutionClass,
    };
  });
  await pageF3.close();

  // ==========================================
  // Finding 4: 解法再生中に Undo を呼ぶと stop() が呼ばれず playing フラグが true のまま孤立する
  // ==========================================
  const pageF4 = await browser.newPage();
  await pageF4.goto(`${base}/?no-sw&alg=R+U+R'+U'+R`);
  await pageF4.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

  await pageF4.locator("#solve").click();
  await pageF4.locator("#solution-content").waitFor({ state: "visible" });

  await pageF4.locator("#play").click();
  await pageF4.waitForTimeout(100);

  await pageF4.locator("#undo").click();
  await pageF4.waitForTimeout(600);

  const stuckState = await pageF4.evaluate(() => {
    const playBtn = document.querySelector("#play");
    const store = window.__cube_main_debug__?.store;
    return {
      hasSolution: store?.getSolution() !== undefined,
      playBtnLabel: playBtn?.getAttribute("aria-label"),
    };
  });

  const reSolveAttempt = await pageF4.evaluate(async () => {
    await window.__cube_main_debug__?.solve();
    const playBtnBefore = document.querySelector("#play")?.getAttribute("aria-label");
    await window.__cube_main_debug__?.play();
    const playBtnAfter = document.querySelector("#play")?.getAttribute("aria-label");
    return { playBtnBefore, playBtnAfter };
  });

  output.f4PlayUndoStuckState = {
    ...stuckState,
    ...reSolveAttempt,
    isStuck: stuckState.playBtnLabel === "一時停止" || reSolveAttempt.playBtnAfter === "自動再生",
  };
  await pageF4.close();

  // ==========================================
  // Finding 5: 解法表示中に「解法を閉じる」を押すとフォーカスが document.body に喪失する
  // ==========================================
  const pageF5 = await browser.newPage();
  await pageF5.goto(`${base}/?no-sw&alg=R+U+R'+U'`);
  await pageF5.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

  await pageF5.locator("#solve").click();
  await pageF5.locator("#solution-close").waitFor({ state: "visible" });

  await pageF5.locator("#solution-close").focus();
  await pageF5.keyboard.press("Enter");
  await pageF5.locator("#solution-close").waitFor({ state: "hidden" });

  output.f5FocusLossAfterClose = await pageF5.evaluate(() => {
    return {
      activeElementTag: document.activeElement?.tagName,
      activeElementId: document.activeElement?.id,
      isBody: document.activeElement === document.body,
    };
  });
  await pageF5.close();

} finally {
  await browser.close();
  const dest = path.resolve(
    path.dirname(fileURLToPath(import.meta.url)),
    "probe-ui.json",
  );
  fs.writeFileSync(dest, JSON.stringify(output, null, 2), "utf-8");
  console.log("Recorded evidence to", dest);
}
