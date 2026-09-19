import fs from "fs";
import path from "path";
import crypto from "crypto";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const distDir = path.resolve(__dirname, "../dist");
const swPath = path.join(distDir, "sw.js");

if (fs.existsSync(swPath)) {
  const getFiles = (dir, baseDir = dir) => {
    const entries = fs.readdirSync(dir, { withFileTypes: true });
    const files = [];
    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        files.push(...getFiles(fullPath, baseDir));
      } else {
        const rel = path.relative(baseDir, fullPath).replace(/\\/g, "/");
        if (rel !== "sw.js") {
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

  let swContent = fs.readFileSync(swPath, "utf-8");
  const precacheStr = JSON.stringify(allFiles, null, 2);
  swContent = swContent.replace(
    /const PRECACHE_ASSETS = \[[^\]]*\];/s,
    `const PRECACHE_ASSETS = ${precacheStr};`,
  );
  swContent = swContent.replace(
    /const CACHE_VERSION = "[^"]*";/,
    `const CACHE_VERSION = "${contentHash}";`,
  );
  fs.writeFileSync(swPath, swContent, "utf-8");
  console.log(
    `[generate-sw-precache] Injected ${allFiles.length} assets and version ${contentHash} into dist/sw.js`,
  );
}
