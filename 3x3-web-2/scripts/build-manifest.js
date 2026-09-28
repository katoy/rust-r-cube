import fs from "fs";
import path from "path";
import crypto from "crypto";

export function collectInputFiles(rootDir) {
  const inputs = new Map();

  const scanDir = (dirPath, relBase = "") => {
    if (!fs.existsSync(dirPath)) return;
    const entries = fs.readdirSync(dirPath, { withFileTypes: true });
    for (const entry of entries) {
      if (
        entry.name === "node_modules" ||
        entry.name === ".git" ||
        entry.name === "target" ||
        entry.name === "dist"
      ) {
        continue;
      }
      const fullPath = path.join(dirPath, entry.name);
      const relPath = path.join(relBase, entry.name).replace(/\\/g, "/");
      if (entry.isDirectory()) {
        scanDir(fullPath, relPath);
      } else {
        const stat = fs.statSync(fullPath);
        const hash = crypto
          .createHash("sha256")
          .update(fs.readFileSync(fullPath))
          .digest("hex")
          .slice(0, 16);
        inputs.set(relPath, { mtime: stat.mtimeMs, size: stat.size, hash });
      }
    }
  };

  scanDir(path.join(rootDir, "src"), "src");
  scanDir(path.join(rootDir, "web"), "web");
  scanDir(path.join(rootDir, "public"), "public");

  const configFiles = [
    "vite.config.ts",
    "Cargo.toml",
    "Cargo.lock",
    "build.rs",
    "package.json",
    "package-lock.json",
    "index.html",
    "scripts/generate-sw-precache.js",
  ];
  for (const rel of configFiles) {
    const full = path.join(rootDir, rel);
    if (fs.existsSync(full)) {
      const stat = fs.statSync(full);
      const hash = crypto
        .createHash("sha256")
        .update(fs.readFileSync(full))
        .digest("hex")
        .slice(0, 16);
      inputs.set(rel.replace(/\\/g, "/"), {
        mtime: stat.mtimeMs,
        size: stat.size,
        hash,
      });
    }
  }

  return inputs;
}

export function saveBuildManifest(rootDir, distDir, inputs) {
  const manifestPath = path.join(distDir, ".build-manifest.json");
  try {
    const data = Object.fromEntries(inputs.entries());
    fs.writeFileSync(manifestPath, JSON.stringify(data, null, 2), "utf-8");
  } catch (err) {
    console.warn("⚠️ ビルドマニフェストの保存に失敗しました:", err.message);
  }
}

export function checkInputsFreshness(rootDir, distDir, currentInputs) {
  const manifestPath = path.join(distDir, ".build-manifest.json");
  if (!fs.existsSync(manifestPath)) {
    return { fresh: false, reason: "ビルドマニフェストが存在しません" };
  }

  let savedManifest;
  try {
    savedManifest = JSON.parse(fs.readFileSync(manifestPath, "utf-8"));
  } catch {
    return { fresh: false, reason: "ビルドマニフェストが破損しています" };
  }

  const savedKeys = Object.keys(savedManifest);
  const currentKeys = Array.from(currentInputs.keys());

  // 1. ファイル削除の検知
  for (const key of savedKeys) {
    if (!currentInputs.has(key)) {
      return { fresh: false, reason: `ファイルが削除されました: ${key}` };
    }
  }

  // 2. ファイル追加の検知
  for (const key of currentKeys) {
    if (!(key in savedManifest)) {
      return {
        fresh: false,
        reason: `新しいファイルが追加されました: ${key}`,
      };
    }
  }

  // 3. ファイル変更の検知
  for (const [key, curr] of currentInputs.entries()) {
    const prev = savedManifest[key];
    if (prev.hash !== curr.hash) {
      return { fresh: false, reason: `ファイルが変更されました: ${key}` };
    }
  }

  return { fresh: true };
}
