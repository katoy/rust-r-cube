import fs from "node:fs";
import http from "node:http";
import path from "node:path";
import vm from "node:vm";
import { spawn, execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const source = fs.readFileSync("scripts/launch-offline.js", "utf8");
const reserver = http.createServer();
await new Promise((resolve) => reserver.listen(0, "127.0.0.1", resolve));
const PORT = reserver.address().port;
await new Promise((resolve) => reserver.close(resolve));
const BASE_URL = `http://127.0.0.1:${PORT}/`;
const sandbox = {
  http,
  PORT,
  BASE_URL,
  isWin: false,
  rootDir: process.cwd(),
  spawn,
  console,
  process,
  setTimeout,
};
vm.runInNewContext(
  source.slice(
    source.indexOf("function checkServer("),
    source.indexOf("async function main()"),
  ) + "\nthis.startServer = startServer; this.checkServer = checkServer;",
  sandbox,
);
let serverProc;
let descendants = [];
try {
  serverProc = await sandbox.startServer();
  const rows = execFileSync("ps", ["-axo", "pid=,ppid="], { encoding: "utf8" })
    .trim()
    .split("\n")
    .map((line) => line.trim().split(/\s+/).map(Number));
  const visit = (pid) => {
    for (const [child, parent] of rows)
      if (parent === pid) {
        descendants.push(child);
        visit(child);
      }
  };
  visit(serverProc.pid);
  serverProc.kill();
  await new Promise((resolve) => setTimeout(resolve, 500));
  const stillResponds = await sandbox.checkServer(BASE_URL);
  const result = {
    parentPid: serverProc.pid,
    descendants,
    afterOnlyServerProcKillStillResponds: stillResponds,
    expected: "owned preview server and descendants stop at launcher cleanup",
  };
  const out = path.join(
    path.dirname(fileURLToPath(import.meta.url)),
    "infra-cleanup-results.json",
  );
  fs.writeFileSync(
    path.relative(process.cwd(), out),
    JSON.stringify(result, null, 2) + "\n",
  );
  console.log(JSON.stringify(result, null, 2));
} finally {
  for (const pid of descendants.reverse()) {
    try {
      process.kill(pid, "SIGTERM");
    } catch {}
  }
  if (serverProc) serverProc.kill();
  await new Promise((resolve) => setTimeout(resolve, 300));
  console.log(
    "After exact-PID cleanup server responds:",
    await sandbox.checkServer(BASE_URL),
  );
}
