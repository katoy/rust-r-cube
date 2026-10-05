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

const SOLVED = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
const SUPERFLIP_STATE =
  "UBULURUFURURFRBRDRFUFLFRFDFDFDLDRDBDLULBLFLDLBUBRBLBDB";

try {
  // ==========================================
  // Finding 1: 解法シーク後に解法カードを閉じると完成状態が破棄されスクランブルに戻ってしまう
  // ==========================================
  const pageF1 = await browser.newPage();
  await pageF1.goto(`${base}/?no-sw`);
  await pageF1.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

  await pageF1.locator('button[data-move="R"]').click();
  const scrambleState = await pageF1.evaluate(
    () => window.__cube_main_debug__?.store?.getState(),
  );

  await pageF1.locator("#solve").click();
  await pageF1.locator("#move-list button").first().waitFor();

  await pageF1.locator("#last").click();
  const stateAtLast = await pageF1.evaluate(
    () => window.__cube_main_debug__?.store?.getState(),
  );

  await pageF1.locator("#solution-close").click();

  const stateAfterClose = await pageF1.evaluate(
    () => window.__cube_main_debug__?.store?.getState(),
  );
  const savedInStorage = await pageF1.evaluate(() => {
    const raw = localStorage.getItem("cube-studio-v1");
    return raw ? JSON.parse(raw) : null;
  });

  output.f1CloseDestroysSolvedState = {
    scrambleState,
    stateAtLast,
    stateAfterClose,
    isRestoredToScramble: stateAfterClose === scrambleState,
    isCommittedSolvedState: stateAfterClose === SOLVED,
    savedStateInStorage: savedInStorage?.state,
    savedIsSolved: savedInStorage?.state === SOLVED,
    savedIsScramble: savedInStorage?.state === scrambleState,
  };
  await pageF1.close();

  // ==========================================
  // Finding 2: 不正局面での探索失敗時に「30秒で再探索」ボタンと誤ったステータス文言が表示される
  // ==========================================
  const pageF2 = await browser.newPage();
  await pageF2.goto(`${base}/?no-sw`);
  await pageF2.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

  await pageF2.evaluate(() => {
    const store = window.__cube_main_debug__?.store;
    store?.replace(
      "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB",
      true,
      [1, 0, 0, 0, 0, 0],
    );
  });

  await pageF2.locator("#solve").click();
  await pageF2.waitForTimeout(300);

  const errorMessage = await pageF2.locator("#message").textContent();
  const solverNote = await pageF2.locator("#solver-note").textContent();
  const extendedVisible = await pageF2.locator("#extended").isVisible();

  output.f2InvalidStateShowsExtendedAndTimeout = {
    errorMessage,
    solverNote,
    extendedVisible,
    isTimeoutNoteShown: solverNote === "探索時間の上限に達しました",
    isExtendedButtonShown: extendedVisible,
  };
  await pageF2.close();

  // ==========================================
  // Finding 3: 解法プレビューシーク中に「配色を手動で編集」を開くと、プレビュー途中局面がエディタに渡される
  // ==========================================
  const pageF3 = await browser.newPage();
  await pageF3.goto(`${base}/?no-sw`);
  await pageF3.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

  await pageF3.locator('button[data-move="R"]').click();
  const scrambleStateF3 = await pageF3.evaluate(
    () => window.__cube_main_debug__?.store?.getState(),
  );

  await pageF3.locator("#solve").click();
  await pageF3.locator("#move-list button").first().waitFor();

  await pageF3.locator("#last").click();

  await pageF3.locator('[data-tab="colors"]').click();
  await pageF3.locator("#edit-colors").click();
  await pageF3.locator("#editor").waitFor({ state: "visible" });

  const editorDraft = await pageF3.evaluate(() => {
    return (window).__cube_main_debug__?.store?.getState();
  });

  output.f3EditColorsDuringPreview = {
    scrambleState: scrambleStateF3,
    editorInitialState: editorDraft,
    isEditorDraftSolved: editorDraft === SOLVED,
    isEditorDraftScramble: editorDraft === scrambleStateF3,
  };
  await pageF3.locator("#editor-close").click();
  await pageF3.close();

  // ==========================================
  // Finding 4: 探索中におけるファイル読み込みレース
  // ==========================================
  const pageF4 = await browser.newPage();
  await pageF4.goto(`${base}/?no-sw`);
  await pageF4.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

  await pageF4.evaluate((state) => {
    window.__cube_main_debug__?.store?.replace(state);
  }, SUPERFLIP_STATE);
  await pageF4.locator("#solver-algorithm").selectOption("korf");
  await pageF4.locator("#solve").click();
  await pageF4.locator("#cancel").waitFor({ state: "visible" });

  const raceResult = await pageF4.evaluate(async () => {
    const store = window.__cube_main_debug__?.store;
    const initialRev = store?.getRevision();

    const fileInput = document.getElementById("file");
    const testFileContent = JSON.stringify({
      version: 1,
      state: "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB",
      centerTurns: [0, 0, 0, 0, 0, 0],
    });
    const file = new File([testFileContent], "test.json", {
      type: "application/json",
    });

    const dataTransfer = new DataTransfer();
    dataTransfer.items.add(file);
    fileInput.files = dataTransfer.files;

    fileInput.dispatchEvent(new Event("change"));

    await new Promise((r) => setTimeout(r, 200));

    const stateAfterLoad = store?.getState();
    const cancelVisible =
      document.getElementById("cancel")?.offsetParent !== null;

    return {
      initialRev,
      stateAfterLoad,
      isSearchCancelled: !cancelVisible,
      stateWasReplacedWithFile:
        stateAfterLoad ===
        "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB",
    };
  });

  output.f4FileLoadRaceDuringSolve = raceResult;
  await pageF4.close();

  // ==========================================
  // Finding 5: solution-close 押下時の CubeStore 二重通知
  // ==========================================
  const pageF5 = await browser.newPage();
  await pageF5.goto(`${base}/?no-sw`);
  await pageF5.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

  await pageF5.locator('button[data-move="R"]').click();
  await pageF5.locator("#solve").click();
  await pageF5.locator("#move-list button").first().waitFor();

  const notifyCounts = await pageF5.evaluate(async () => {
    const store = window.__cube_main_debug__?.store;
    let solutionEventCount = 0;
    const unsubscribe = store?.subscribe((_s, detail) => {
      if (detail.type === "solution") {
        solutionEventCount++;
      }
    });

    const closeBtn = document.getElementById("solution-close");
    closeBtn?.click();

    unsubscribe?.();

    return {
      solutionEventCount,
    };
  });

  output.f5SolutionCloseDuplicateNotify = {
    solutionEventCount: notifyCounts.solutionEventCount,
    hasDuplicateNotification: notifyCounts.solutionEventCount > 1,
  };
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
