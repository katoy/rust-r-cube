import { readFile, writeFile } from "node:fs/promises";

const inventory = JSON.parse(
  await readFile(new URL("./inventory.json", import.meta.url), "utf8"),
);
const coverage = JSON.parse(
  await readFile("coverage-e2e/coverage.json", "utf8"),
);
const expected = inventory.files
  .filter((file) => file.category === "web")
  .map((file) => file.path);
const observed = coverage.files.map((file) => new URL(file.url).pathname);
const result = {
  timestamp: coverage.timestamp,
  method: coverage.method,
  expected,
  missing: expected.filter(
    (path) => !observed.some((url) => url.endsWith(`/${path}`)),
  ),
  note: "Module capture inventory only; not a Rust/WASM semantic coverage measurement.",
};
await writeFile(
  new URL("./coverage-inventory.json", import.meta.url),
  JSON.stringify(result, null, 2) + "\n",
);
console.log(JSON.stringify(result, null, 2));
