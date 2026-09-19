#!/usr/bin/env node
import { spawn } from "child_process";
import http from "http";
import path from "path";
import fs from "fs";
import { fileURLToPath } from "url";
import { chromium } from "@playwright/test";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, "..");
const distDir = path.join(rootDir, "dist");

const PORT = 4173;
const BASE_URL = `http://127.0.0.1:${PORT}/`;

// 1. ビルド成果物の確認（存在しない場合は自動ビルド）
async function ensureBuild() {
  const swPath = path.join(distDir, "sw.js");
  const indexPath = path.join(distDir, "index.html");

  if (!fs.existsSync(swPath) || !fs.existsSync(indexPath)) {
    console.log(
      "📦 ビルド成果物が見つかりません。本番ビルドを実行しています...",
    );
    await runCommand("npm", ["run", "build"]);
  }
}

const isWin = process.platform === "win32";

function runCommand(cmd, args) {
  const executable = isWin ? `${cmd}.cmd` : cmd;
  return new Promise((resolve, reject) => {
    const proc = spawn(executable, args, {
      cwd: rootDir,
      stdio: "inherit",
    });
    proc.on("close", (code) => {
      if (code === 0) resolve();
      else
        reject(
          new Error(
            `Command ${cmd} ${args.join(" ")} failed with code ${code}`,
          ),
        );
    });
  });
}

// 2. サーバーの死活監視
function checkServer(url) {
  return new Promise((resolve) => {
    const req = http.get(url, (res) => {
      resolve(res.statusCode >= 200 && res.statusCode < 400);
    });
    req.on("error", () => resolve(false));
    req.setTimeout(1000, () => {
      req.destroy();
      resolve(false);
    });
  });
}

// 3. プレビューサーバーの起動
async function startServer() {
  const isRunning = await checkServer(BASE_URL);
  if (isRunning) {
    console.log(`🌐 プレビューサーバーは既に稼働しています (${BASE_URL})`);
    return null;
  }

  console.log(`🚀 プレビューサーバーを起動しています (ポート ${PORT})...`);
  const npxCmd = isWin ? "npx.cmd" : "npx";
  const serverProc = spawn(
    npxCmd,
    [
      "vite",
      "preview",
      "--port",
      String(PORT),
      "--strictPort",
      "--host",
      "127.0.0.1",
    ],
    { cwd: rootDir, stdio: "pipe" },
  );

  serverProc.stderr.on("data", (d) => {
    const msg = d.toString();
    if (!msg.includes("ExperimentalWarning")) {
      process.stderr.write(msg);
    }
  });

  // サーバーの起動待ち
  const maxRetries = 30;
  for (let i = 0; i < maxRetries; i++) {
    await new Promise((r) => setTimeout(r, 500));
    if (await checkServer(BASE_URL)) {
      console.log(`✅ プレビューサーバーが起動しました (${BASE_URL})`);
      return serverProc;
    }
  }

  serverProc.kill();
  throw new Error("プレビューサーバーの起動がタイムアウトしました。");
}

async function main() {
  const args = process.argv.slice(2);
  const isHeadless = args.includes("--headless");
  const showHelp = args.includes("--help") || args.includes("-h");

  if (showHelp) {
    console.log(`使い方: node scripts/launch-offline.js [オプション]

オプション:
  --headless    ブラウザを非表示（ヘッドレスモード）で実行してオフライン起動を検証
  -h, --help    このヘルプを表示
`);
    process.exit(0);
  }

  await ensureBuild();

  let serverProc = null;
  let browser = null;

  const cleanup = async () => {
    console.log("\n🧹 終了処理を実行中...");
    if (browser) {
      try {
        await browser.close();
      } catch {}
    }
    if (serverProc) {
      serverProc.kill();
    }
    process.exit(0);
  };

  process.on("SIGINT", cleanup);
  process.on("SIGTERM", cleanup);

  try {
    serverProc = await startServer();

    console.log(
      `🖥️  ブラウザ (Chromium) を起動しています (${isHeadless ? "ヘッドレス" : "GUI表示"})...`,
    );
    browser = await chromium.launch({
      headless: isHeadless,
      args: isHeadless ? [] : ["--window-size=1366,900"],
    });

    const context = await browser.newContext({
      viewport: { width: 1366, height: 900 },
    });
    const page = await context.newPage();

    console.log(
      "📥 初回アクセス中（アセットと Service Worker をキャッシュします）...",
    );
    await page.goto(BASE_URL, { waitUntil: "networkidle" });

    // エンジンが READY になるのを待機
    await page.waitForSelector("#engine-status", { state: "attached" });
    await page.waitForFunction(
      () =>
        document
          .querySelector("#engine-status")
          ?.textContent?.includes("READY"),
      null,
      { timeout: 15000 },
    );

    // Service Worker の登録とコントローラー化の完了待機
    await page.evaluate(async () => {
      if (!("serviceWorker" in navigator)) return;
      await navigator.serviceWorker.ready;
      if (!navigator.serviceWorker.controller) {
        await new Promise((resolve) => {
          navigator.serviceWorker.addEventListener(
            "controllerchange",
            resolve,
            { once: true },
          );
        });
      }
    });

    // キャッシュされたファイル一覧を取得
    const cachedCount = await page.evaluate(async () => {
      const keys = await caches.keys();
      let count = 0;
      for (const k of keys) {
        const c = await caches.open(k);
        const reqs = await c.keys();
        count += reqs.length;
      }
      return count;
    });
    console.log(
      `📦 Service Worker に ${cachedCount} 個のアセットがキャッシュされました。`,
    );

    // ネットワークをオフラインに設定
    console.log("🔌 ネットワーク接続を遮断します (setOffline: true)...");
    await context.setOffline(true);

    // オフライン状態でリロード
    console.log(
      "🔄 オフライン環境でリロードしてキャッシュ起動を検証しています...",
    );
    await page.reload({ waitUntil: "networkidle" });

    // オフラインでの再起動完了を確認
    await page.waitForFunction(
      () =>
        document
          .querySelector("#engine-status")
          ?.textContent?.includes("READY"),
      null,
      { timeout: 10000 },
    );

    // 画面上にオフライン通知トースト/バナーを表示
    await page.evaluate(() => {
      const banner = document.createElement("div");
      banner.id = "offline-demo-banner";
      banner.innerHTML = `
        <div style="
          position: fixed;
          top: 12px;
          left: 50%;
          transform: translateX(-50%);
          z-index: 99999;
          background: rgba(20, 23, 22, 0.92);
          border: 1px solid #10b981;
          color: #f3f4f6;
          padding: 10px 20px;
          border-radius: 9999px;
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
          font-size: 13px;
          font-weight: 500;
          box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.5);
          display: flex;
          align-items: center;
          gap: 8px;
          pointer-events: auto;
        ">
          <span style="display:inline-block; width:8px; height:8px; border-radius:50%; background:#10b981; animation: pulse 1.5s infinite;"></span>
          <span>⚡ <strong>オフラインモードで動作中</strong> (ネットワーク完全遮断済み)</span>
        </div>
      `;
      document.body.appendChild(banner);
    });

    console.log(
      "\n============================================================",
    );
    console.log("🎉 Cube Studio がオフラインモードで正常に起動しました！");
    console.log("📡 ネットワークは遮断されています (Offline mode: ACTIVE)");
    console.log(
      "🕹️  ブラウザ上でキューブの回転・解法探索・再生を自由にお試しいただけます。",
    );
    console.log(
      "\n終了するには、ブラウザを閉じるか、このターミナルで Ctrl+C を押してください。",
    );
    console.log(
      "============================================================\n",
    );

    if (isHeadless) {
      console.log(
        "✅ ヘッドレスモードでのオフライン起動検証が正常に完了しました。",
      );
      await cleanup();
      return;
    }

    // ブラウザが閉じられるまで待機
    await new Promise((resolve) => {
      page.on("close", resolve);
      browser.on("disconnected", resolve);
    });

    console.log("ブラウザが閉じられました。");
    await cleanup();
  } catch (err) {
    console.error("❌ エラーが発生しました:", err);
    await cleanup();
  }
}

main();
