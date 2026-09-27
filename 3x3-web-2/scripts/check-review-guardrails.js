#!/usr/bin/env node
/**
 * レビュー漏れ再発防止のための静的ガードレールチェッカー
 *
 * docs/code-review-b755b38-2026-09-27.md で指摘された以下のパターンを静的に走査・警告します：
 * 1. [F1] Rust側で return Err(...) を含む検証ブロックに #[cfg(debug_assertions)] が付与されている
 * 2. [F3] Web側で file.size の事前チェックを行わずに file.text() が実行されている
 * 3. [F4] location.pathname の正規化で index.html が考慮されていない
 * 4. [F5] Service Worker のナビゲーション保存でクエリ付きリクエストがそのまま cache.put されている
 */

import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const args = process.argv.slice(2);
const warnOnly = args.includes("--warn-only");
const dirArg = args.find((a) => !a.startsWith("--"));
const rootDir = dirArg
  ? path.resolve(process.cwd(), dirArg)
  : path.resolve(__dirname, "..");

let hasErrors = false;
let hasWarnings = false;

function reportError(rule, file, line, message) {
  if (warnOnly) {
    reportWarning(rule, file, line, message);
    return;
  }
  console.error(`❌ [ERROR][${rule}] ${file}:${line} - ${message}`);
  hasErrors = true;
}

function reportWarning(rule, file, line, message) {
  console.warn(`⚠️  [WARN][${rule}] ${file}:${line} - ${message}`);
  hasWarnings = true;
}

// 1. Rustコードの条件付きコンパイル検証 [F1]
function checkRustDebugAssertions() {
  const srcDir = path.join(rootDir, "src");
  if (!fs.existsSync(srcDir)) return;

  const files = fs.readdirSync(srcDir).filter((f) => f.endsWith(".rs"));
  for (const file of files) {
    const filePath = path.join(srcDir, file);
    const content = fs.readFileSync(filePath, "utf-8");
    const lines = content.split("\n");

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      if (line.includes("#[cfg(debug_assertions)]")) {
        // 次の数行に return Err があるか走査
        const lookahead = lines.slice(i, i + 25).join("\n");
        if (lookahead.includes("return Err(")) {
          reportError(
            "F1-RELEASE-VERIFICATION",
            `src/${file}`,
            i + 1,
            "#[cfg(debug_assertions)] ブロック内に return Err が存在します。本番(Release)ビルドで検証ロジックが除外されるため、無条件検証にするか意図的ならコメントで明記してください。",
          );
        }
      }
    }
  }
}

// 2. ファイル読み込み順序の検査 [F3]
function checkFileReadOrder() {
  const webDir = path.join(rootDir, "web");
  if (!fs.existsSync(webDir)) return;

  const files = fs.readdirSync(webDir).filter((f) => f.endsWith(".ts"));
  for (const file of files) {
    const filePath = path.join(webDir, file);
    const content = fs.readFileSync(filePath, "utf-8");
    const lines = content.split("\n");

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      if (line.includes(".text()") && !line.includes("// ignore-guardrail")) {
        // 同じ関数の直前で .size チェックがあるか確認
        const lookbehind = lines.slice(Math.max(0, i - 15), i).join("\n");
        if (!lookbehind.includes(".size")) {
          reportError(
            "F3-FILE-SIZE-FIRST",
            `web/${file}`,
            i + 1,
            "file.text() の実行前に file.size の事前検査が見当たりません。巨大ファイル選択時のメモリ圧迫を防ぐため、読み込み前にサイズを検証してください。",
          );
        }
      }
    }
  }
}

// 3. パス正規化の index.html 考慮 [F4]
function checkPathNormalization() {
  const mainPath = path.join(rootDir, "web/main.ts");
  if (!fs.existsSync(mainPath)) return;
  const content = fs.readFileSync(mainPath, "utf-8");
  const lines = content.split("\n");

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (
      line.includes("getScopedStorageKey") ||
      (line.includes("pathname") && line.includes("replace"))
    ) {
      const context = lines.slice(i, i + 15).join("\n");
      if (
        context.includes("pathname") &&
        !(context.includes("index") && context.includes("html"))
      ) {
        reportError(
          "F4-URL-NORMALIZATION",
          "web/main.ts",
          i + 1,
          "URL/pathname 正規化で index.html が考慮されていない可能性があります（'/' と '/index.html' でストレージが分離するリスク）。",
        );
      }
    }
  }
}

// 4. Service Worker のナビゲーションキャッシュ保存 [F5]
function checkServiceWorkerCache() {
  const swPath = path.join(rootDir, "public/sw.js");
  if (!fs.existsSync(swPath)) return;
  const content = fs.readFileSync(swPath, "utf-8");
  const lines = content.split("\n");

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (
      line.includes("cache.put(request,") &&
      !line.includes("// ignore-guardrail")
    ) {
      reportError(
        "F5-SW-QUERY-CACHE",
        "public/sw.js",
        i + 1,
        "cache.put に request オブジェクトが直接渡されています。クエリ付き共有URL(?state=...)を開くたびに別キャッシュが作られるリスクがあります。正規化された URL または ignoreSearch を考慮した保存キーを使用してください。",
      );
    }
  }
}

console.log("🔍 コードレビュー・ガードレール静的検証を実行中...");
checkRustDebugAssertions();
checkFileReadOrder();
checkPathNormalization();
checkServiceWorkerCache();

if (hasErrors) {
  console.error("\n❌ レビュー・ガードレール検査でエラーが検出されました。");
  process.exit(1);
} else if (hasWarnings) {
  console.log(
    "\n⚠️  潜在的なレビュー漏れリスク（警告）が検出されました。修正または意図の確認を推奨します。",
  );
  process.exit(0);
} else {
  console.log("\n✅ レビュー・ガードレール検査をパスしました。");
  process.exit(0);
}
