import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";

function category(path) {
  if (path.startsWith("src/") || path === "build.rs") return "rust";
  if (path.startsWith("web/")) return "web";
  if (path.startsWith("scripts/") || path === "start.sh") return "tooling";
  if (path.startsWith("tests/")) return "tests";
  if (path.startsWith("cubes/") || path.startsWith("public/"))
    return "public-and-fixtures";
  if (path.startsWith("docs/review-evidence/")) return "historical-evidence";
  if (path.endsWith(".md")) return "documentation";
  if (path.includes("coverage")) return "historical-generated-coverage";
  return "configuration-and-metadata";
}

const paths = execFileSync("git", ["ls-files", "."], { encoding: "utf8" })
  .trim()
  .split("\n");
const files = await Promise.all(
  paths.map(async (path) => {
    const contents = await readFile(path);
    return {
      path,
      category: category(path),
      bytes: contents.length,
      sha256: createHash("sha256").update(contents).digest("hex"),
    };
  }),
);
const counts = {};
for (const file of files)
  counts[file.category] = (counts[file.category] || 0) + 1;
const output = {
  commit: execFileSync("git", ["rev-parse", "HEAD"], {
    encoding: "utf8",
  }).trim(),
  scope: "3x3-web-2; related parent CI is listed in the report separately",
  note: "Inventory and input hashes, not a claim that every historical document or generated artifact was read in full.",
  counts,
  files,
};
await writeFile(
  new URL("./inventory.json", import.meta.url),
  JSON.stringify(output, null, 2) + "\n",
);
console.log(JSON.stringify(counts, null, 2));
