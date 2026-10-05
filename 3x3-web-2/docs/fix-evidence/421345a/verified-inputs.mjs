import { readFile, readdir, writeFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import assert from "node:assert/strict";

const output = new URL("./verified-inputs.json", import.meta.url);
const hash = async (path) =>
  createHash("sha256")
    .update(await readFile(path))
    .digest("hex");

if (process.argv.includes("--verify")) {
  const snapshot = JSON.parse(await readFile(output, "utf8"));
  assert.equal(
    execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim(),
    snapshot.head,
  );
  for (const file of snapshot.files)
    assert.equal(
      await hash(file.path),
      file.sha256,
      `Changed during validation: ${file.path}`,
    );
  console.log(`Unchanged validated source inputs: ${snapshot.files.length}`);
} else {
  const inventory = JSON.parse(
    await readFile(
      new URL("../../review-evidence/421345a/inventory.json", import.meta.url),
      "utf8",
    ),
  );
  const categories = new Set([
    "rust",
    "web",
    "tooling",
    "configuration-and-metadata",
    "tests",
  ]);
  const paths = new Set(
    inventory.files
      .filter((file) => categories.has(file.category))
      .map((file) => file.path),
  );
  for (const file of await readdir("tests"))
    if (file.endsWith(".ts")) paths.add(`tests/${file}`);
  paths.add("../.github/workflows/3x3-web-2.yml");
  paths.add("../.github/workflows/deploy-pages.yml");
  const files = [];
  for (const path of [...paths].sort())
    files.push({ path, sha256: await hash(path) });
  await writeFile(
    output,
    JSON.stringify(
      {
        head: execFileSync("git", ["rev-parse", "HEAD"], {
          encoding: "utf8",
        }).trim(),
        capturedAt: new Date().toISOString(),
        note: "Modified worktree source inputs, not a clean-HEAD assertion or review of every historical artifact.",
        files,
      },
      null,
      2,
    ) + "\n",
  );
  console.log(`Captured validation source inputs: ${files.length}`);
}
