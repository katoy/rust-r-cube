// docs/review-evidence/acdb7f4/probe_seek_persist.mjs
// 解法アニメーションシーク時の不要な同期 localStorage.setItem 呼び出し回数（I/Oスラッシング）実証
import { chromium } from "playwright";

async function run() {
  console.log("=== Probe: Seek Animation localStorage Thrashing ===");
  const browser = await chromium.launch();
  const context = await browser.newContext();
  const page = await context.newPage();

  await page.goto("http://127.0.0.1:5173/");
  await page.waitForSelector("#engine-status.ready");

  // localStorage.setItem の呼び出し回数をインターセプトして計測
  await page.evaluate(() => {
    window.__setItemCalls = 0;
    window.__setItemPayloads = [];
    const origSetItem = localStorage.setItem.bind(localStorage);
    localStorage.setItem = (key, value) => {
      window.__setItemCalls++;
      window.__setItemPayloads.push({ key, value });
      return origSetItem(key, value);
    };
  });

  // 簡単なプリセットを解く
  console.log("1. プリセット '簡単（5手）' を解く...");
  await page.locator("#tab-presets").click();
  await page.locator("#preset-buttons button", { hasText: "簡単（5手）" }).click();
  await page.locator("#solve").click();
  await page.waitForSelector("#solution-content:not([hidden])");

  const callsAfterSolve = await page.evaluate(() => window.__setItemCalls);
  console.log(`   解法完了直後の setItem 呼出回数: ${callsAfterSolve}`);

  // 自動再生を実行
  console.log("2. 自動再生（Play）を実行...");
  await page.locator("#play").click();
  // 再生完了を待つ（3手）
  await page.waitForTimeout(1500);

  const callsAfterPlay = await page.evaluate(() => window.__setItemCalls);
  const seekCalls = callsAfterPlay - callsAfterSolve;
  console.log(`   再生完了後の setItem 総呼出回数: ${callsAfterPlay}`);
  console.log(`   => 再生中に発生した setItem 回数: ${seekCalls} 回`);

  const payloads = await page.evaluate(() => window.__setItemPayloads);
  console.log("\n   書き込まれたペイロード例（最後の3件）:");
  payloads.slice(-3).forEach((p, idx) => {
    const data = JSON.parse(p.value);
    console.log(`     [Call ${payloads.length - 2 + idx}] state: ${data.state.slice(0, 18)}... (ステップ途中の局面を永続化)`);
  });

  if (seekCalls > 0) {
    console.log(
      "\n=> 【課題実証】解法アニメーションの各ステップごとに localStorage.setItem が同期実行されています。\n" +
      "   再生中の途中状態がローカルストレージに過剰に書き込まれ、長手数の解法（CFOPの100手超など）では数百回の同期I/Oが発生します。\n" +
      "   また再生中にリロードされた場合、解法の途中の壊れた局面から復元されてしまいます。"
    );
  }

  await browser.close();
}

run().catch(console.error);
