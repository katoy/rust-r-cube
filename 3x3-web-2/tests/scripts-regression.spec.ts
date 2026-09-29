import { test, expect } from "@playwright/test";
import { exec, spawn } from "child_process";
import path from "path";
import { fileURLToPath } from "url";
import fs from "fs";
import { createServer } from "node:http";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const projectRoot = path.resolve(__dirname, "..");

test.describe("R14 & R16: Script Exit Codes and Symlink Resolution", () => {
  test("R14: generate-test-images.js exits with code 1 on failure", async () => {
    const scriptPath = path.join(
      projectRoot,
      "scripts/generate-test-images.js",
    );

    // 意図的に書き込み不可の場所に manifest を指定して実行させる（あるいは無効引数/環境）
    const result = await new Promise<{ code: number | null; stderr: string }>(
      (resolve) => {
        // 出力先ディレクトリを親が存在しない架空のパスにする環境変数をシミュレート、
        // または node -e でスクリプトを呼び出し内部エラーを起こす
        const proc = spawn(
          "node",
          [
            "-e",
            `
          // fs.writeFileSync をモックして強制エラーを起こす
          const fs = require("fs");
          const origWrite = fs.writeFileSync;
          fs.writeFileSync = () => { throw new Error("Disk Full"); };
          require(${JSON.stringify(scriptPath)});
        `,
          ],
          { cwd: projectRoot },
        );

        let stderr = "";
        proc.stderr.on("data", (d) => {
          stderr += d.toString();
        });
        proc.on("close", (code) => {
          resolve({ code, stderr });
        });
      },
    );

    // 失敗時に正常終了 (0) ではなく exit code 1 で終了すること
    expect(result.code).toBe(1);
  });

  test("R14: launch-offline.js exits with code 1 on failure", async () => {
    const scriptPath = path.join(projectRoot, "scripts/launch-offline.js");

    const result = await new Promise<{ code: number | null; stderr: string }>(
      (resolve) => {
        const proc = spawn(
          "node",
          [scriptPath, "--headless", "--port=invalid"],
          { cwd: projectRoot },
        );

        let stderr = "";
        proc.stderr.on("data", (d) => {
          stderr += d.toString();
        });
        proc.on("close", (code) => {
          resolve({ code, stderr });
        });
      },
    );

    // 失敗時に exit code 1 で終了すること
    expect(result.code).toBe(1);
    expect(result.stderr).toContain("Invalid --port");
  });

  test("R16: scripts/start.sh resolves symlink and sets working directory to project root", async () => {
    const symlinkPath = path.join(projectRoot, "scripts/start.sh");
    expect(fs.existsSync(symlinkPath)).toBe(true);

    const result = await new Promise<{
      code: number | null;
      stdout: string;
      stderr: string;
    }>((resolve) => {
      // --help を実行してヘルプメッセージが正しく表示されること
      exec(
        `bash "${symlinkPath}" --help`,
        { cwd: projectRoot },
        (err, stdout, stderr) => {
          resolve({
            code: err ? (err.code ?? 1) : 0,
            stdout,
            stderr,
          });
        },
      );
    });

    expect(result.code).toBe(0);
    expect(result.stdout).toContain("使い方:");
    expect(result.stdout).toContain("--offline");

    // bash -x で実行し、cd 先のディレクトリが projectRoot であることを検証
    const traceResult = await new Promise<string>((resolve) => {
      exec(
        `bash -x "${symlinkPath}" --help`,
        { cwd: path.join(projectRoot, "scripts") },
        (err, stdout, stderr) => {
          resolve(stderr);
        },
      );
    });

    // cd コマンドの引数に projectRoot が渡されていること
    expect(traceResult).toContain(`cd ${projectRoot}`);
  });

  test("F3: check-review-guardrails.js exits with code 1 on violation fixture", async () => {
    const tmpDir = path.join(projectRoot, "tmp-guardrails-test");
    const tmpSrcDir = path.join(tmpDir, "src");
    fs.mkdirSync(tmpSrcDir, { recursive: true });

    try {
      // 違反コードを含む一時Rustファイルを配置
      fs.writeFileSync(
        path.join(tmpSrcDir, "bad.rs"),
        `
#[cfg(debug_assertions)]
fn bad_fn() -> Result<(), ()> {
  return Err(());
}
`,
      );

      const scriptPath = path.join(
        projectRoot,
        "scripts/check-review-guardrails.js",
      );

      // 1. 通常実行（デフォルト）で exit code 1 で失敗すること
      const failResult = await new Promise<{
        code: number | null;
        stdout: string;
        stderr: string;
      }>((resolve) => {
        exec(
          `node "${scriptPath}" "${tmpDir}"`,
          { cwd: projectRoot },
          (err, stdout, stderr) => {
            resolve({
              code: err ? (err.code ?? 1) : 0,
              stdout,
              stderr,
            });
          },
        );
      });

      expect(failResult.code).toBe(1);
      expect(failResult.stderr).toContain("F1-RELEASE-VERIFICATION");

      // 2. --warn-only 指定時は exit code 0 で終了すること
      const warnResult = await new Promise<{
        code: number | null;
        stdout: string;
        stderr: string;
      }>((resolve) => {
        exec(
          `node "${scriptPath}" --warn-only "${tmpDir}"`,
          { cwd: projectRoot },
          (err, stdout, stderr) => {
            resolve({
              code: err ? (err.code ?? 1) : 0,
              stdout,
              stderr,
            });
          },
        );
      });

      expect(warnResult.code).toBe(0);
      expect(warnResult.stderr).toContain("WARN");
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  test("F5: launch-offline.js terminates cleanly with numeric exit code on SIGINT", async () => {
    const scriptPath = path.join(projectRoot, "scripts/launch-offline.js");
    // The suite's port 4173 preview has a nested base, not the launcher's root.
    const reservation = createServer();
    await new Promise<void>((resolve, reject) => {
      reservation.once("error", reject);
      reservation.listen(0, "127.0.0.1", resolve);
    });
    const port = (reservation.address() as { port: number }).port;
    await new Promise<void>((resolve, reject) =>
      reservation.close((error) => (error ? reject(error) : resolve())),
    );

    const result = await new Promise<{
      code: number | null;
      stdout: string;
      stderr: string;
      signalSent: boolean;
      watchdogExpired: boolean;
    }>((resolve, reject) => {
      const proc = spawn(
        process.execPath,
        [scriptPath, "--headless", `--port=${port}`],
        { cwd: projectRoot },
      );

      let stdout = "";
      let stderr = "";
      let signalSent = false;
      let watchdogExpired = false;
      const watchdog = setTimeout(() => {
        watchdogExpired = true;
        proc.kill("SIGTERM");
      }, 15000);

      proc.stdout.on("data", (d) => {
        stdout += d.toString();
        if (
          !signalSent &&
          stdout.includes("プレビューサーバーが起動しました")
        ) {
          signalSent = true;
          proc.kill("SIGINT");
        }
      });

      proc.stderr.on("data", (d) => {
        stderr += d.toString();
      });

      proc.on("error", (error) => {
        clearTimeout(watchdog);
        reject(error);
      });
      proc.on("close", (code) => {
        clearTimeout(watchdog);
        resolve({ code, stdout, stderr, signalSent, watchdogExpired });
      });
    });

    expect(result.watchdogExpired, result.stdout + result.stderr).toBe(false);
    expect(result.signalSent, result.stdout + result.stderr).toBe(true);
    expect(result.stderr).not.toContain("ERR_INVALID_ARG_TYPE");
    expect(result.stderr).not.toContain("TypeError");
    expect(result.code, result.stdout + result.stderr).toBe(130);
    expect(result.stdout).toContain("終了処理を実行中");
    await expect
      .poll(async () => {
        try {
          await fetch(`http://127.0.0.1:${port}/`, {
            signal: AbortSignal.timeout(1000),
          });
          return true;
        } catch {
          return false;
        }
      })
      .toBe(false);
  });
});
