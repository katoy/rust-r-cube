// Reproducible, read-only comparison with the sibling project's Kociemba core.
import { readFileSync, writeFileSync, mkdirSync, mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
const source = (name) =>
  readFileSync(
    new URL("../../3x3-web/src/kociemba/" + name, import.meta.url),
    "utf8",
  );
const originals = ["coord.rs", "tables.rs", "search.rs"].map(source);
console.log(
  "Reference source SHA-256:",
  originals.map((s) => createHash("sha256").update(s).digest("hex")),
);
let [coord, tables, search] = originals;
if (
  !coord.includes("impl FaceCube {") ||
  !coord.includes("pub fn from_cube") ||
  !search.includes("pub fn idx_to_move")
)
  throw new Error(
    "Reference source layout changed. Review the adapter before comparing.",
  );
coord =
  "use std::sync::OnceLock;\n" +
  coord
    .slice(
      coord.indexOf("/// コーナーピース"),
      coord.indexOf("impl FaceCube {"),
    )
    .replace(
      /    pub fn from_cube[\s\S]*?    \/\/ --- 座標変換メソッド ---/,
      "",
    );
tables = tables
  .split("#[cfg(test)]")[0]
  .replaceAll("crate::kociemba::coord", "crate::coord");
search =
  search
    .split("pub fn idx_to_move")[0]
    .replace("use crate::cube::Move;", "type Move=usize;") +
  "pub fn idx_to_move(idx:usize)->usize{idx}\n";
const dir = mkdtempSync(join(tmpdir(), "cube-legacy-"));
mkdirSync(join(dir, "src"));
writeFileSync(
  join(dir, "Cargo.toml"),
  '[package]\nname="legacy-baseline"\nversion="0.1.0"\nedition="2021"\n[profile.release]\nopt-level=3\nlto=true\n',
);
for (const [name, content] of Object.entries({
  "coord.rs": coord,
  "tables.rs": tables,
  "search.rs": search,
  "main.rs":
    '#![allow(dead_code,clippy::upper_case_acronyms)]\nmod coord;mod tables;mod search;\nfn main(){\nlet start=std::time::Instant::now();let mut search=search::Search::new();\nprintln!("legacy_initialization_ms={:.3}",start.elapsed().as_secs_f64()*1000.);\nlet mut times=Vec::new();let mut lengths=Vec::new();let mut failed=0;\nfor seed in 1u32..=1000{\nlet mut x=seed;let mut moves=Vec::new();\nwhile moves.len()<25{x^=x<<13;x^=x>>17;x^=x<<5;let m=x as usize%18;\nif moves.last().is_some_and(|last|last/3==m/3){continue;}moves.push(m);}\nlet c=moves.iter().fold(coord::RawCube::default(),|c,m|c.multiply(coord::move_cube_18(*m)));\nlet now=std::time::Instant::now();let result=search.solve(&c,128);\ntimes.push(now.elapsed().as_secs_f64()*1000.);\nif let Some(solution)=result{assert_eq!(solution.iter().fold(c,|c,m|c.multiply(coord::move_cube_18(*m))),coord::RawCube::default());lengths.push(solution.len());}else{failed+=1;}\n}\ntimes.sort_by(f64::total_cmp);lengths.sort();\nprintln!("legacy 1000 states: p50={:.3}ms p95={:.3}ms max={:.3}ms; moves p50={} max={}; failed={}",times[500],times[950],times[999],lengths[lengths.len()/2],lengths.last().unwrap(),failed);\n}\n',
}))
  writeFileSync(join(dir, "src", name), content);
console.log("Isolated reference benchmark:", dir);
const result = spawnSync("cargo", ["run", "--release", "--offline"], {
  cwd: dir,
  stdio: "inherit",
});
process.exitCode = result.status ?? 1;
