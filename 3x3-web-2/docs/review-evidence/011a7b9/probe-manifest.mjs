import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { checkInputsFreshness } from "../../../scripts/build-manifest.js";

const root = fs.mkdtempSync(path.join(os.tmpdir(), "cube-review-manifest-"));
const dist = path.join(root, "dist");
fs.mkdirSync(dist);
const inputs = new Map([["src/main.rs", { hash: "abc", size: 1, mtime: 1 }]]);
const results = {};
try {
  for (const [name, content] of Object.entries({
    malformed_json: "{",
    valid_json_null: "null",
    valid_json_null_entry: '{"src/main.rs":null}',
  })) {
    fs.writeFileSync(path.join(dist, ".build-manifest.json"), content);
    try {
      results[name] = checkInputsFreshness(root, dist, inputs);
    } catch (error) {
      results[name] = { threw: String(error) };
    }
  }
} finally {
  fs.rmSync(root, { recursive: true, force: true });
}
const json = JSON.stringify(results, null, 2);
fs.writeFileSync(
  new URL("./probe-manifest.json", import.meta.url),
  `${json}\n`,
);
console.log(json);
