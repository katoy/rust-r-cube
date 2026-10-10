import fs from "fs";
import path from "path";
import crypto from "crypto";
import { fileURLToPath } from "url";
import { collectInputFiles, saveBuildManifest } from "./build-manifest.js";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const rootDir = path.resolve(__dirname, "..");
const distDir = path.resolve(__dirname, "../dist");
const swPath = path.join(distDir, "sw.js");

// dist/sw.js が無い場合は CACHE_VERSION 未注入のまま出荷されうるため失敗させる
if (!fs.existsSync(swPath)) {
  console.error(
    "[generate-sw-precache] dist/sw.js が見つかりません。vite build の成果物を確認してください。",
  );
  process.exit(1);
}

// dist/ 配下のドキュメント (*.md、例: public/cubes/README.md) は配信不要なため削除する
const removeMarkdown = (dir) => {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      removeMarkdown(fullPath);
    } else if (entry.name.endsWith(".md")) {
      fs.rmSync(fullPath);
    }
  }
};
removeMarkdown(distDir);

const getFiles = (dir, baseDir = dir) => {
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      files.push(...getFiles(fullPath, baseDir));
    } else {
      const rel = path.relative(baseDir, fullPath).replace(/\\/g, "/");
      // ドキュメント (*.md) は配信対象ではないためプリキャッシュから除外する
      if (
        rel !== "sw.js" &&
        rel !== ".build-manifest.json" &&
        !rel.endsWith(".md")
      ) {
        files.push(`./${rel}`);
      }
    }
  }
  return files;
};

const allFiles = getFiles(distDir);
if (!allFiles.includes("./")) {
  allFiles.unshift("./");
}

// ファイル内容のハッシュを計算（同名資産の更新を検知して SW を更新させる）
const hash = crypto.createHash("sha256");
const sortedFiles = [...allFiles].sort();
for (const file of sortedFiles) {
  if (file === "./") continue;
  const absPath = path.join(distDir, file.replace(/^\.\//, ""));
  if (fs.existsSync(absPath)) {
    hash.update(file);
    hash.update(fs.readFileSync(absPath));
  }
}
const contentHash = hash.digest("hex").slice(0, 10);

const original = fs.readFileSync(swPath, "utf-8");
const precacheStr = JSON.stringify(allFiles, null, 2);
const precachePattern = /const PRECACHE_ASSETS = \[[^\]]*\];/s;
const versionPattern = /const CACHE_VERSION = "[^"]*";/;
// 置換対象が見つからない場合は既定値 (v1) のまま出荷されるため失敗させる
if (!precachePattern.test(original) || !versionPattern.test(original)) {
  console.error(
    "[generate-sw-precache] dist/sw.js に PRECACHE_ASSETS または CACHE_VERSION の定義が見つかりません。",
  );
  process.exit(1);
}
const swContent = original
  .replace(precachePattern, `const PRECACHE_ASSETS = ${precacheStr};`)
  .replace(versionPattern, `const CACHE_VERSION = "${contentHash}";`);
fs.writeFileSync(swPath, swContent, "utf-8");
console.log(
  `[generate-sw-precache] Injected ${allFiles.length} assets and version ${contentHash} into dist/sw.js`,
);

// ビルド入力マニフェストを記録（ファイルの削除・追加・変更の検知用）
const inputs = collectInputFiles(rootDir);
saveBuildManifest(rootDir, distDir, inputs);
