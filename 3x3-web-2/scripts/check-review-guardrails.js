#!/usr/bin/env node
/**
 * レビュー漏れ再発防止のための静的ガードレールチェッカー
 *
 * docs/code-review-b755b38-2026-09-27.md で指摘された以下のパターンを静的に走査・警告します：
 * 1. [F1] Rust側で検証・エラー返却に #[cfg(debug_assertions)] / debug_assert! が使われている
 * 2. [F3] Web側で file.size の事前チェックを行わずにファイル全文を読み込んでいる
 * 3. [F4] location.pathname の正規化で index.html が考慮されていない
 * 4. [F5] Service Worker の cache.put で許可リスト（正規化キー）以外が保存キーになっている
 * 5. [F6] Three.js のジオメトリ・マテリアル・テクスチャを生成するのに dispose() が無い
 * 6. [F7] スクリプトが成果物の存在チェックだけでビルドを省略している
 *
 * すり抜けパターンは tests/fixtures/guardrails/ に置き、tests/scripts-regression.spec.ts で検出を回帰テストする。
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

function getFilesRecursively(dir, ext) {
  if (!fs.existsSync(dir)) return [];
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  let files = [];
  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      files = files.concat(getFilesRecursively(fullPath, ext));
    } else if (entry.name.endsWith(ext)) {
      files.push(fullPath);
    }
  }
  return files;
}

// 文字列中の位置 index を 1 始まりの行番号に変換する
function lineOf(content, index) {
  return content.slice(0, index).split("\n").length;
}

// openIndex の開き括弧に対応する閉じ括弧の位置を返す（見つからなければ -1）
function findMatching(content, openIndex) {
  const pairs = { "(": ")", "{": "}", "[": "]" };
  const stack = [];
  for (let i = openIndex; i < content.length; i++) {
    const ch = content[i];
    if (pairs[ch]) {
      stack.push(pairs[ch]);
    } else if (ch === ")" || ch === "}" || ch === "]") {
      if (stack.pop() !== ch) return -1;
      if (stack.length === 0) return i;
    }
  }
  return -1;
}

// from 以降の 1 文（深さ 0 の ; まで）または 1 ブロック（深さ 0 に戻る } まで）を返す
function extractStatement(content, from) {
  let depth = 0;
  for (let i = from; i < content.length; i++) {
    const ch = content[i];
    if (ch === "(" || ch === "{" || ch === "[") {
      depth++;
    } else if (ch === ")" || ch === "}" || ch === "]") {
      depth--;
      if (depth === 0 && ch === "}") return content.slice(from, i + 1);
      if (depth < 0) return content.slice(from, i);
    } else if (ch === ";" && depth === 0) {
      return content.slice(from, i + 1);
    }
  }
  return content.slice(from);
}

// 行番号を保ったまま // 行コメントを空白に置き換える（URL 等の "://" は対象外）
function stripLineComments(content) {
  return content.replace(/(^|[^:])\/\/[^\n]*/g, (m, prefix) =>
    prefix.padEnd(m.length, " "),
  );
}

// #[cfg(test)] が付いたテストモジュールを行番号を保ったまま空白化する
function maskRustTestModules(content) {
  let masked = content;
  const re = /#\[cfg\(test\)\]/g;
  let m;
  while ((m = re.exec(masked)) !== null) {
    const open = masked.indexOf("{", m.index);
    if (open === -1) break;
    const close = findMatching(masked, open);
    const end = close === -1 ? masked.length : close + 1;
    masked =
      masked.slice(0, m.index) +
      masked.slice(m.index, end).replace(/[^\n]/g, " ") +
      masked.slice(end);
  }
  return masked;
}

function isRustTestFile(relPath) {
  const base = path.basename(relPath);
  return (
    base === "tests.rs" ||
    base.endsWith("_tests.rs") ||
    relPath.split(path.sep).includes("tests")
  );
}

// 1. Rustコードの条件付きコンパイル検証 [F1]
// - #[cfg(debug_assertions)] / cfg!(debug_assertions) が付いた文・ブロックに
//   Err 返却・? 演算子・assert・検証関数呼び出しが含まれていればエラー
// - debug_assert! の条件に関数呼び出し（検証処理）が含まれていればエラー
function checkRustDebugAssertions() {
  const srcDir = path.join(rootDir, "src");
  const files = getFilesRecursively(srcDir, ".rs");
  for (const filePath of files) {
    const relPath = path.relative(rootDir, filePath);
    if (isRustTestFile(relPath)) continue;
    const content = maskRustTestModules(
      stripLineComments(fs.readFileSync(filePath, "utf-8")),
    );

    const cfgRe = /#\[cfg\(([^\]]*)\)\]|\bcfg!\s*\(((?:[^()]|\([^()]*\))*)\)/g;
    let m;
    while ((m = cfgRe.exec(content)) !== null) {
      const cond = m[1] ?? m[2] ?? "";
      if (!/\bdebug_assertions\b/.test(cond) || /\bnot\s*\(/.test(cond)) {
        continue;
      }
      const statement = extractStatement(content, m.index + m[0].length);
      if (/\bErr\s*\(|\?\s*[;),]|assert|verify|validate/.test(statement)) {
        reportError(
          "F1-RELEASE-VERIFICATION",
          relPath,
          lineOf(content, m.index),
          "デバッグ用条件付きコンパイル内に検証・エラー返却ロジックが存在します。本番(Release)ビルドで検証ロジックが除外されるため、無条件の検証にしてください。",
        );
      }
    }

    const assertRe = /\bdebug_assert(?:_eq|_ne)?!\s*\(/g;
    while ((m = assertRe.exec(content)) !== null) {
      const open = m.index + m[0].length - 1;
      const close = findMatching(content, open);
      const argsText = content.slice(
        open + 1,
        close === -1 ? undefined : close,
      );
      if (/(?:\b[A-Za-z_]\w*|\.\s*\w+)\s*\(/.test(argsText)) {
        reportError(
          "F1-DEBUG-ASSERT-VERIFY",
          relPath,
          lineOf(content, m.index),
          "debug_assert! の条件で検証関数を呼び出しています。Release ビルドで検証が除外されるため、assert! または Result による無条件の検証にしてください。",
        );
      }
    }
  }
}

// 2. ファイル読み込み順序の検査 [F3]
// メソッドチェーンの改行や FileReader / Response 経由の全文読み込みも対象にする
function checkFileReadOrder() {
  const webDir = path.join(rootDir, "web");
  const files = getFilesRecursively(webDir, ".ts");
  const readPatterns = [
    /\.\s*(?:text|arrayBuffer|bytes)\s*\(\s*\)/g,
    /\breadAs(?:Text|ArrayBuffer|DataURL|BinaryString)\s*\(/g,
  ];
  for (const filePath of files) {
    const relPath = path.relative(rootDir, filePath);
    const raw = fs.readFileSync(filePath, "utf-8");
    const rawLines = raw.split("\n");
    const content = stripLineComments(raw);
    const lines = content.split("\n");

    for (const pattern of readPatterns) {
      pattern.lastIndex = 0;
      let m;
      while ((m = pattern.exec(content)) !== null) {
        const lineNo = lineOf(content, m.index);
        if (rawLines[lineNo - 1].includes("// ignore-guardrail")) continue;
        // 同じ関数の直前で .size チェックがあるか確認
        const lookbehind = lines
          .slice(Math.max(0, lineNo - 16), lineNo - 1)
          .join("\n");
        if (!lookbehind.includes(".size")) {
          reportError(
            "F3-FILE-SIZE-FIRST",
            relPath,
            lineNo,
            "ファイル全文の読み込み（text() / arrayBuffer() / FileReader.readAs* 等）の前に file.size の事前検査が見当たりません。巨大ファイル選択時のメモリ圧迫を防ぐため、読み込み前にサイズを検証してください。",
          );
        }
      }
    }
  }
}

// 3. パス正規化の index.html 考慮 [F4]
function checkPathNormalization() {
  const targetFiles = ["web/main.ts", "web/storage-key.ts"];
  for (const relPath of targetFiles) {
    const fullPath = path.join(rootDir, relPath);
    if (!fs.existsSync(fullPath)) continue;
    const content = fs.readFileSync(fullPath, "utf-8");
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
            relPath,
            i + 1,
            "URL/pathname 正規化で index.html が考慮されていない可能性があります（'/' と '/index.html' でストレージが分離するリスク）。",
          );
        }
      }
    }
  }
}

// 4. Service Worker のキャッシュ保存キー [F5]
// cache.put の呼び出しを全件列挙し、第 1 引数が許可リストの正規化キー以外ならエラーにする
const SW_PUT_KEY_ALLOWLIST = new Set(["canonicalKey", "boundedCacheKey"]);

// 引数リスト文字列から深さ 0 の最初の引数を取り出す
function firstArgument(argsText) {
  let depth = 0;
  for (let i = 0; i < argsText.length; i++) {
    const ch = argsText[i];
    if ("([{".includes(ch)) depth++;
    else if (")]}".includes(ch)) depth--;
    else if (ch === "," && depth === 0) return argsText.slice(0, i).trim();
  }
  return argsText.trim();
}

function checkServiceWorkerCache() {
  const swPath = path.join(rootDir, "public/sw.js");
  if (!fs.existsSync(swPath)) return;
  const content = stripLineComments(fs.readFileSync(swPath, "utf-8"));

  // 許可リストのキーは <url>.origin + <url>.pathname（クエリ無し）で定義されていること
  const defRe = /\bcanonicalKey\s*=(?!=)\s*([^;]*);/g;
  let m;
  while ((m = defRe.exec(content)) !== null) {
    if (!/^(\w+)\.origin\s*\+\s*\1\.pathname$/.test(m[1].trim())) {
      reportError(
        "F5-SW-QUERY-CACHE",
        "public/sw.js",
        lineOf(content, m.index),
        "canonicalKey は <url>.origin + <url>.pathname（クエリ無し）で定義してください。",
      );
    }
  }

  // boundedCacheKey を使用する場合は、MAX_QUERY_ENTRIES による上限管理があることを確認
  if (content.includes("boundedCacheKey")) {
    if (
      !content.includes("MAX_QUERY_ENTRIES") ||
      !content.includes(".delete(")
    ) {
      reportError(
        "F5-SW-QUERY-CACHE",
        "public/sw.js",
        1,
        "boundedCacheKey を使用する場合は MAX_QUERY_ENTRIES 定義と古いキーの delete による上限管理を実装してください。",
      );
    }
  }

  const putRe = /\.put\s*\(/g;
  while ((m = putRe.exec(content)) !== null) {
    const open = m.index + m[0].length - 1;
    const close = findMatching(content, open);
    const key = firstArgument(
      content.slice(open + 1, close === -1 ? undefined : close),
    );
    if (!SW_PUT_KEY_ALLOWLIST.has(key)) {
      reportError(
        "F5-SW-QUERY-CACHE",
        "public/sw.js",
        lineOf(content, m.index),
        `cache.put の保存キー「${key}」が許可リスト（${[...SW_PUT_KEY_ALLOWLIST].join(", ")}）にありません。request やクエリ付き URL をそのまま保存するとキャッシュが無制限に蓄積します。`,
      );
    }
  }
}

// 5. Three.js リソースの破棄 [F6]（原則 ⑤）
// ジオメトリ・マテリアル・テクスチャ・レンダラーを生成するファイルに dispose() が無ければエラー
function checkThreeDispose() {
  const webDir = path.join(rootDir, "web");
  const files = getFilesRecursively(webDir, ".ts");
  const createRe =
    /new\s+THREE\.(?:\w*Geometry|\w*Material|\w*Texture|WebGLRenderer)\s*\(/;
  for (const filePath of files) {
    const relPath = path.relative(rootDir, filePath);
    const content = stripLineComments(fs.readFileSync(filePath, "utf-8"));
    const m = createRe.exec(content);
    if (m && !/\.dispose\s*\(/.test(content)) {
      reportError(
        "F6-THREE-DISPOSE",
        relPath,
        lineOf(content, m.index),
        "Three.js のジオメトリ・マテリアル・テクスチャを生成していますが dispose() が見当たりません。キャッシュ共有するか明示的に破棄してください。",
      );
    }
  }
}

// 6. 成果物の鮮度保証 [F7]（原則 ⑥）
// 成果物の存在チェックでビルドを省略するスクリプトに、ソースとの鮮度比較が無ければエラー
function checkArtifactFreshness() {
  const candidates = [
    ...fs
      .readdirSync(rootDir)
      .filter((name) => name.endsWith(".sh"))
      .map((name) => path.join(rootDir, name)),
    ...getFilesRecursively(path.join(rootDir, "scripts"), ".js"),
    ...getFilesRecursively(path.join(rootDir, "scripts"), ".mjs"),
    ...getFilesRecursively(path.join(rootDir, "scripts"), ".sh"),
  ].filter((file) => !fs.lstatSync(file).isSymbolicLink());
  const existenceRe = /existsSync\s*\(|\[\s*!?\s*-[efsd]\s/;
  const buildRe =
    /npm run (?:build|wasm)|wasm-pack|\[\s*"run",\s*"(?:build|wasm)"/;
  const freshnessRe = /checkInputsFreshness|\s-nt\s|\s-newer\s/;
  for (const filePath of candidates) {
    const relPath = path.relative(rootDir, filePath);
    const content = fs.readFileSync(filePath, "utf-8");
    if (freshnessRe.test(content)) continue;
    const lines = content.split("\n");
    for (let i = 0; i < lines.length; i++) {
      if (!existenceRe.test(lines[i])) continue;
      const context = lines.slice(i, i + 7).join("\n");
      if (buildRe.test(context)) {
        reportError(
          "F7-ARTIFACT-FRESHNESS",
          relPath,
          i + 1,
          "成果物の存在チェックだけでビルドを省略しています。ソースの更新日時やビルドマニフェストと比較して鮮度を保証してください。",
        );
      }
    }
  }
}

console.log("🔍 コードレビュー・ガードレール静的検証を実行中...");
checkRustDebugAssertions();
checkFileReadOrder();
checkPathNormalization();
checkServiceWorkerCache();
checkThreeDispose();
checkArtifactFreshness();

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
