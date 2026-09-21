import { test, expect } from "@playwright/test";
import { exec, spawn } from "child_process";
import path from "path";
import { fileURLToPath } from "url";
import fs from "fs";

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
          [
            "-e",
            `
          // chromium.launch をモックして強制エラーを起こす
          const pw = require("@playwright/test");
          if (pw.chromium) {
            pw.chromium.launch = async () => { throw new Error("Launch failed"); };
          }
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

    // 失敗時に exit code 1 で終了すること
    expect(result.code).toBe(1);
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
});
