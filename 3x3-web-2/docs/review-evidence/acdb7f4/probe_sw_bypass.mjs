// docs/review-evidence/acdb7f4/probe_sw_bypass.mjs
// Service Worker による page.route バイパスとテスト間汚染の実証
import { chromium } from "playwright";

async function run() {
  console.log("=== Probe: Service Worker Bypassing page.route ===");
  const browser = await chromium.launch();
  const context = await browser.newContext();
  const page = await context.newPage();

  // 1. 初回ロード（Service Worker が登録される）
  console.log("1. ページへ初回アクセスし、Service Worker の登録を待機...");
  await page.goto("http://127.0.0.1:5173/");
  await page.waitForTimeout(2500);

  const isControlled = await page.evaluate(
    () => navigator.serviceWorker.controller !== null,
  );
  console.log(`   navigator.serviceWorker.controller 存在: ${isControlled}`);

  // 2. Playwright の page.route で worker を abort する設定を追加
  console.log("2. page.route(\"**/web/solver.worker.ts*\", route => route.abort()) を設定...");
  let intercepted = false;
  await page.route("**/web/solver.worker.ts*", (route) => {
    intercepted = true;
    console.log("   [page.route 発火]:", route.request().url());
    return route.abort();
  });

  // 3. 再度アクセス（tests/app.spec.ts:261 と同一の操作）
  console.log("3. ページを再読み込みし、エンジンのステータスを確認...");
  await page.goto("http://127.0.0.1:5173/");
  await page.waitForTimeout(2000);

  const status = await page.textContent("#engine-status");
  console.log(`   #engine-status 表示: "${status}"`);
  console.log(`   page.route がインターセプトできたか: ${intercepted}`);

  if (!intercepted && status.includes("READY")) {
    console.log(
      "\n=> 【実証成功】Service Worker がキャッシュから応答したため、Playwright の page.route() が発火せずバイパスされました！\n" +
      "   これにより、tests/app.spec.ts:261 の worker failure テストが一括実行時に '読み込み失敗' とならずタイムアウト失敗します。"
    );
  } else {
    console.log("   再現せず");
  }

  await browser.close();
}

run().catch(console.error);
