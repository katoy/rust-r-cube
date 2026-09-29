#!/usr/bin/env node
import { spawn } from "child_process";
import http from "http";
import path from "path";
import fs from "fs";
import crypto from "crypto";
import { fileURLToPath } from "url";
import { chromium } from "@playwright/test";

import { collectInputFiles, checkInputsFreshness } from "./build-manifest.js";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, "..");
const distDir = path.join(rootDir, "dist");

const portArg = process.argv.find((arg) => arg.startsWith("--port="));
const PORT = portArg ? Number(portArg.slice("--port=".length)) : 4173;
const BASE_URL = `http://127.0.0.1:${PORT}/`;

// 1. ビルド成果物の確認（存在しない場合、またはソースが更新・追加・削除されている場合は自動ビルド）
async function ensureBuild(forceBuild = false) {
  const swPath = path.join(distDir, "sw.js");
  const indexPath = path.join(distDir, "index.html");

  if (!fs.existsSync(swPath) || !fs.existsSync(indexPath)) {
    console.log(
      "📦 ビルド成果物が見つかりません。本番ビルドを実行しています...",
    );
    await runCommand("npm", ["run", "build"]);
    return;
  }

  if (forceBuild) {
    console.log("🔨 強制再ビルドを実行しています (--build / --fresh)...");
    await runCommand("npm", ["run", "build"]);
    return;
  }

  const currentInputs = collectInputFiles(rootDir);
  const freshness = checkInputsFreshness(rootDir, distDir, currentInputs);

  if (!freshness.fresh) {
    console.log(
      `🔄 ソースコードまたは設定の変更を検知しました (${freshness.reason})。最新の成果物をビルドしています...`,
    );
    await runCommand("npm", ["run", "build"]);
  } else {
    const distMtime = Math.min(
      fs.statSync(swPath).mtimeMs,
      fs.statSync(indexPath).mtimeMs,
    );
    console.log(
      `⚡ 既存の最新ビルド成果物を使用します (${new Date(distMtime).toLocaleTimeString()})`,
    );
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
    proc.on("error", reject);
  });
}

// 2. サーバーの死活監視
function checkServer(url) {
  return new Promise((resolve) => {
    const req = http.get(url, (res) => {
      res.resume();
      resolve(res.statusCode >= 200 && res.statusCode < 400);
    });
    req.on("error", () => resolve(false));
    req.setTimeout(1000, () => {
      req.destroy();
      resolve(false);
    });
  });
}

export async function verifyServerBuild(baseUrl, directory = distDir) {
  const files = fs.readdirSync(directory, { recursive: true });
  for (const file of files) {
    const filename = path.join(directory, file);
    if (!fs.statSync(filename).isFile() || file === ".build-manifest.json")
      continue;
    const expected = fs.readFileSync(filename);
    const expectedHash = crypto
      .createHash("sha256")
      .update(expected)
      .digest("hex");
    const url = new URL(
      file.split(path.sep).map(encodeURIComponent).join("/"),
      baseUrl,
    );
    const actualHash = await new Promise((resolve, reject) => {
      const req = http.get(url, (res) => {
        if (res.statusCode !== 200) {
          res.resume();
          reject(
            new Error(
              `Preview build mismatch: ${url} (HTTP ${res.statusCode})`,
            ),
          );
          return;
        }
        const hash = crypto.createHash("sha256");
        let size = 0;
        res.on("data", (chunk) => {
          size += chunk.length;
          if (size > expected.length) {
            req.destroy(new Error(`Preview build mismatch: ${url} (size)`));
            return;
          }
          hash.update(chunk);
        });
        res.on("end", () => resolve(hash.digest("hex")));
        res.on("error", reject);
      });
      req.on("error", reject);
      // An absolute deadline also bounds servers that trickle bytes forever.
      const timer = setTimeout(() => {
        req.destroy(new Error(`Preview verification timed out: ${url}`));
      }, 5000);
      req.on("close", () => clearTimeout(timer));
    });
    if (actualHash !== expectedHash)
      throw new Error(`Preview build mismatch: ${url} (content)`);
  }
}

// 3. プレビューサーバーの起動
async function startServer(onSpawn = () => {}) {
  const isRunning = await checkServer(BASE_URL);
  if (isRunning) {
    await verifyServerBuild(BASE_URL);
    console.log(`🌐 プレビューサーバーは既に稼働しています (${BASE_URL})`);
    return null;
  }

  console.log(`🚀 プレビューサーバーを起動しています (ポート ${PORT})...`);
  const serverProc = spawn(
    process.execPath,
    [
      path.join(rootDir, "node_modules/vite/bin/vite.js"),
      "preview",
      "--port",
      String(PORT),
      "--strictPort",
      "--host",
      "127.0.0.1",
    ],
    { cwd: rootDir, stdio: "pipe" },
  );
  onSpawn(serverProc);

  serverProc.stdout.resume();
  let startError;
  serverProc.on("error", (error) => {
    startError = error;
  });
  serverProc.stderr.on("data", (d) => {
    const msg = d.toString();
    if (!msg.includes("ExperimentalWarning")) {
      process.stderr.write(msg);
    }
  });

  // サーバーの起動待ち
  try {
    const maxRetries = 30;
    for (let i = 0; i < maxRetries; i++) {
      await new Promise((r) => setTimeout(r, 500));
      if (startError) throw startError;
      if (serverProc.exitCode !== null)
        throw new Error(
          `Preview server exited with code ${serverProc.exitCode}`,
        );
      if (await checkServer(BASE_URL)) {
        await verifyServerBuild(BASE_URL);
        console.log(`✅ プレビューサーバーが起動しました (${BASE_URL})`);
        return serverProc;
      }
    }
    throw new Error("プレビューサーバーの起動がタイムアウトしました。");
  } catch (error) {
    serverProc.kill();
    throw error;
  }
}

export async function waitForServiceWorker(page, timeoutMs = 15000) {
  await page.evaluate((timeout) => {
    if (!("serviceWorker" in navigator))
      throw new Error("Service Worker is unavailable");
    return new Promise((resolve, reject) => {
      const serviceWorker = navigator.serviceWorker;
      let ready = false;
      let finished = false;
      let lastState = "registration missing";
      const watched = new Set();
      const finish = (error) => {
        if (finished) return;
        finished = true;
        clearTimeout(deadline);
        clearInterval(poll);
        serviceWorker.removeEventListener("controllerchange", check);
        for (const worker of watched)
          worker.removeEventListener("statechange", check);
        if (error) reject(error);
        else resolve();
      };
      const check = async () => {
        if (finished) return;
        if ([...watched].some((candidate) => candidate.state === "redundant")) {
          finish(new Error("Service Worker installation failed (redundant)"));
          return;
        }
        if (ready && serviceWorker.controller?.state === "activated") {
          finish();
          return;
        }
        try {
          const registration = await serviceWorker.getRegistration();
          if (finished) return;
          const worker =
            registration?.installing ||
            registration?.waiting ||
            registration?.active;
          if (!worker) return;
          lastState = worker.state;
          if (!watched.has(worker)) {
            watched.add(worker);
            worker.addEventListener("statechange", check);
          }
          if ([...watched].some((candidate) => candidate.state === "redundant"))
            finish(new Error("Service Worker installation failed (redundant)"));
        } catch (error) {
          finish(error);
        }
      };
      const deadline = setTimeout(
        () =>
          finish(
            new Error(
              `Service Worker readiness timed out after ${timeout}ms (${lastState})`,
            ),
          ),
        timeout,
      );
      const poll = setInterval(check, 100);
      serviceWorker.addEventListener("controllerchange", check);
      serviceWorker.ready.then(() => {
        ready = true;
        check();
      }, finish);
      check();
    });
  }, timeoutMs);
}

async function main() {
  const args = process.argv.slice(2);
  const isHeadless = args.includes("--headless");
  const forceBuild = args.includes("--build") || args.includes("--fresh");
  const showHelp = args.includes("--help") || args.includes("-h");

  if (showHelp) {
    console.log(`使い方: node scripts/launch-offline.js [オプション]

オプション:
  --headless    ブラウザを非表示（ヘッドレスモード）で実行してオフライン起動を検証
  --fresh, --build 強制的にビルドを実行して最新状態を検証
  --port=番号  プレビューサーバーのポート（既定: 4173）
  -h, --help    このヘルプを表示
`);
    process.exit(0);
  }

  if (!Number.isInteger(PORT) || PORT < 1 || PORT > 65535)
    throw new Error("Invalid --port: expected an integer between 1 and 65535");
  await ensureBuild(forceBuild);

  let serverProc = null;
  let browser = null;

  const cleanup = async (exitCode = 0) => {
    console.log("\n🧹 終了処理を実行中...");
    if (browser) {
      try {
        await browser.close();
      } catch {}
    }
    if (serverProc) {
      serverProc.kill();
    }
    const numericCode =
      typeof exitCode === "number"
        ? exitCode
        : exitCode === "SIGINT"
          ? 130
          : exitCode === "SIGTERM"
            ? 143
            : 0;
    process.exit(numericCode);
  };

  process.on("SIGINT", () => cleanup(130));
  process.on("SIGTERM", () => cleanup(143));

  try {
    serverProc = await startServer((proc) => {
      serverProc = proc;
    });

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
    await waitForServiceWorker(page);

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
    await cleanup(1);
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === __filename) {
  main().catch((error) => {
    console.error("❌ エラーが発生しました:", error);
    process.exitCode = 1;
  });
}
