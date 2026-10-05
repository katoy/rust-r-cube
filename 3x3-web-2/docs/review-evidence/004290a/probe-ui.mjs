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
  // Finding 1: 探索中キーボード回転ショートカット発火による意図しない探索中断と局面破壊
  // ==========================================
  const pageF1 = await browser.newPage();
  await pageF1.goto(`${base}/?no-sw`);
  await pageF1.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

  await pageF1.evaluate((state) => {
    window.__cube_main_debug__?.store?.replace(state);
  }, SUPERFLIP_STATE);
  await pageF1.locator("#solver-algorithm").selectOption("korf");

  await pageF1.locator("#solve").click();
  await pageF1.locator("#cancel").waitFor({ state: "visible" });

  const solvingBefore = await pageF1.evaluate(
    () => window.__cube_main_debug__?.store?.getRevision(),
  );
  const uBtnDisabled = await pageF1
    .locator('button[data-move="U"]')
    .isDisabled();

  await pageF1.keyboard.press("KeyU");
  await pageF1.waitForTimeout(400);

  const cancelVisibleAfter = await pageF1.locator("#cancel").isVisible();
  const revisionAfter = await pageF1.evaluate(
    () => window.__cube_main_debug__?.store?.getRevision(),
  );
  const solverNote = await pageF1.locator("#solver-note").textContent();

  output.f1KeyboardMoveDuringSolve = {
    buttonDisabled: uBtnDisabled,
    cancelWasVisibleBefore: true,
    cancelVisibleAfterKeypress: cancelVisibleAfter,
    searchCancelledByKeypress: !cancelVisibleAfter,
    revisionChanged: revisionAfter !== solvingBefore,
    solverNoteAfterKeypress: solverNote,
  };
  await pageF1.close();

  // ==========================================
  // Finding 2: 探索タイムアウト／エラー発生時における solver-note のゾンビ表示残留
  // ==========================================
  const pageF2 = await browser.newPage();
  await pageF2.goto(`${base}/?no-sw`);
  await pageF2.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

  const f2Result = await pageF2.evaluate(async () => {
    const solver = window.__cube_main_debug__?.getSolver();
    if (!solver) return null;

    const originalSolve = solver.solve.bind(solver);
    solver.solve = () =>
      new Promise((_, reject) => {
        setTimeout(
          () =>
            reject(
              new Error(
                "探索時間の上限に達しました。30秒の延長探索を試してください。",
              ),
            ),
          250,
        );
      });

    const solvePromise = window.__cube_main_debug__?.solve(5000);
    await new Promise((r) => setTimeout(r, 260));
    await solvePromise;

    solver.solve = originalSolve;

    const note = document.getElementById("solver-note")?.textContent;
    const extendedHidden = (
      document.getElementById("extended") as HTMLElement
    )?.hidden;
    const cancelHidden = (document.getElementById("cancel") as HTMLElement)
      ?.hidden;
    const message = document.getElementById("message")?.textContent;

    return { note, extendedHidden, cancelHidden, message };
  });

  output.f2ZombieSolverNoteOnTimeout = f2Result;
  await pageF2.close();

  // ==========================================
  // Finding 3: 解法プレビューシーク中の共有リンクの局面すり替え
  // ==========================================
  const pageF3 = await browser.newPage();
  await pageF3.goto(`${base}/?no-sw`);
  await pageF3.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

  await pageF3.locator('button[data-move="R"]').click();
  await pageF3.waitForTimeout(300);

  const scrambleState = await pageF3.evaluate(
    () => window.__cube_main_debug__?.store?.getState(),
  );

  await pageF3.locator("#solve").click();
  await pageF3.locator("#move-list button").first().waitFor();

  await pageF3.locator("#last").click();
  await pageF3.waitForTimeout(300);

  const stepState = await pageF3.evaluate(
    () => window.__cube_main_debug__?.store?.getState(),
  );

  let copiedText = "";
  await pageF3.evaluate(() => {
    navigator.clipboard.writeText = async (text) => {
      window.__lastCopied = text;
    };
  });

  await pageF3.locator("#share-link").click();
  copiedText = await pageF3.evaluate(() => window.__lastCopied);

  output.f3ShareLinkDuringPreview = {
    scrambleState,
    stepState,
    isStepStateSolved: stepState?.startsWith("UUUUUUUUU"),
    copiedUrl: copiedText,
    urlContainsScrambleState: copiedText.includes(`state=${scrambleState}`),
    urlContainsStepState: copiedText.includes(`state=${stepState}`),
  };
  await pageF3.close();

  // ==========================================
  // Finding 4: solve() 冒頭での二重通知（ダブル通知）
  // ==========================================
  const pageF4 = await browser.newPage();
  await pageF4.goto(`${base}/?no-sw`);
  await pageF4.locator("#engine-status").filter({ hasText: "READY" }).waitFor();

  await pageF4.locator('button[data-move="R"]').click();
  await pageF4.locator("#solve").click();
  await pageF4.locator("#move-list button").first().waitFor();
  await pageF4.locator("#last").click();

  const notifications = await pageF4.evaluate(async () => {
    const store = window.__cube_main_debug__?.store;
    if (!store) return [];
    const events = [];
    const unsubscribe = store.subscribe((_s, detail) => {
      events.push(detail.type);
    });

    const initialRestore = store.restoreBaseSnapshot();
    store.setSolution(undefined);
    unsubscribe();
    return { events, initialRestore };
  });

  output.f4DoubleNotificationOnSolve = notifications;
  await pageF4.close();

  // ==========================================
  // Finding 5: 探索中の share-link 有効化
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

  const shareDisabled = await pageF5.locator("#share-link").isDisabled();
  const scrambleDisabled = await pageF5.locator("#scramble").isDisabled();
  const saveDisabled = await pageF5.locator("#save").isDisabled();
  const loadDisabled = await pageF5.locator("#load").isDisabled();

  output.f5ShareLinkEnabledDuringSolve = {
    shareDisabled,
    scrambleDisabled,
    saveDisabled,
    loadDisabled,
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
