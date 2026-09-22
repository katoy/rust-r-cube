import { chromium } from "playwright";

async function main() {
  console.log("=== Probe 4: Service Worker Auto-Registration in Dev Environment ===");
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext();
  const page = await context.newPage();

  // クリーンな状態で開発サーバーの通常URL（クエリなし）を開く
  await page.goto("http://127.0.0.1:5173/");
  await page.waitForLoadState("networkidle");

  const swStatus = await page.evaluate(async () => {
    if (!("serviceWorker" in navigator)) return { supported: false };
    const reg = await navigator.serviceWorker.getRegistration();
    return {
      supported: true,
      hasRegistration: !!reg,
      scope: reg ? reg.scope : null,
      active: reg && reg.active ? reg.active.state : null,
    };
  });

  console.log("Service Worker 登録状態:", swStatus);
  if (swStatus.hasRegistration) {
    console.log("=> 【実証成功】開発サーバー (127.0.0.1:5173) において、クエリ未指定の場合に無条件で Service Worker が登録され、開発環境が汚染される！");
  }
  await browser.close();
}

main().catch(console.error);
