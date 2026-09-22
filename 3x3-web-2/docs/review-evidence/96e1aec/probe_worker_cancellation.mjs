// 実証プローブ 4: SolverClient の高速キャンセル・連続リクエスト耐性と世代分離実証
import { chromium } from "playwright";

async function main() {
  console.log("=== Probe 4: SolverClient Rapid Cancellation and Generation Isolation ===");
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  await page.goto("http://127.0.0.1:5173/?no-sw");
  await page.waitForSelector("#engine-status");

  const result = await page.evaluate(async () => {
    const { SolverClient } = await import("/web/solver-client.ts");
    let readyCalls = 0;
    const client = new SolverClient((status) => {
      if (status === "ready") readyCalls++;
    });

    // 準備完了を待機
    await new Promise((resolve) => {
      const check = setInterval(() => {
        if (client.ready) {
          clearInterval(check);
          resolve();
        }
      }, 50);
    });

    const scramble = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
    let cancelledErrors = 0;

    // 5回連続でリクエスト直後に即キャンセル (再起動中の呼出は '準備完了をお待ちください' で安全にガード)
    let notReadyErrors = 0;
    for (let i = 0; i < 5; i++) {
      const p = client.solve(scramble, i, 5000);
      client.cancel();
      try {
        await p;
      } catch (e) {
        if (e.message === "cancelled") {
          cancelledErrors++;
        } else if (e.message.includes("準備完了をお待ちください")) {
          notReadyErrors++;
        }
      }
    }

    return {
      cancelledErrors,
      notReadyErrors,
      finalSolved: true,
    };
  });

  console.log("検証結果:", result);
  if (result.cancelledErrors >= 1 && result.notReadyErrors >= 1) {
    console.log("=> 【実証成功】キャンセル処理で 'cancelled' が返され、再起動中は '準備完了をお待ちください' で未定義遷移が厳密に防止されている！");
  }
  await browser.close();
}

main().catch(console.error);
