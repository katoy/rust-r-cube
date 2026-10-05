import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
const require = createRequire(
  new URL("../../../package.json", import.meta.url),
);
const { chromium } = require("@playwright/test");
const browser = await chromium.launch({ headless: true });
try {
  const page = await browser.newPage({ serviceWorkers: "block" });
  await page.goto("http://127.0.0.1:5173/");
  await page.waitForFunction(
    () =>
      window.cube_studio &&
      document.querySelector("#engine-status").textContent.includes("READY"),
  );
  console.log(
    "cancel",
    JSON.stringify(
      await page.evaluate(async () => {
        const { SolverClient } = await import("/web/solver-client.ts");
        const events = [];
        const client = new SolverClient((s, m) => events.push({ s, m }));
        while (!client.ready) await new Promise((r) => setTimeout(r, 10));
        const solved = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
        const state = JSON.parse(
          window.cube_studio.apply_moves(
            solved,
            "U R2 F B R B2 R U2 L B2 R U' D' R2 F R' L B2 U2 F2",
          ),
        ).state;
        const first = client
          .solve(state, 1, 30000, false, undefined, "korf")
          .catch((e) => e.message);
        await new Promise((r) => setTimeout(r, 200));
        client.cancel();
        const before = performance.now();
        const second = await client
          .solve(solved, 2, 5000, false, undefined, "kociemba")
          .then(
            (r) => ({ ok: true, moves: r.moves.length }),
            (e) => ({ ok: false, error: e.message }),
          );
        return {
          first: await first,
          second,
          elapsed_ms: performance.now() - before,
          ready: client.ready,
          events,
        };
      }),
    ),
  );
  await page.locator('[data-tab="colors"]').click();
  await page.locator("#camera-colors").click();
  await page.locator("#camera-editor").waitFor({ state: "visible" });
  console.log(
    "camera-stale-play",
    JSON.stringify(
      await page.evaluate(async () => {
        const cam = window.__lastCamera;
        const video = document.querySelector("#camera-video");
        let resolveOld;
        let plays = 0;
        let stops = [0, 0];
        const streams = [new MediaStream(), new MediaStream()];
        streams.forEach(
          (s, i) => (s.getTracks = () => [{ stop: () => stops[i]++ }]),
        );
        let gets = 0;
        navigator.mediaDevices.getUserMedia = async () => streams[gets++];
        video.play = () =>
          ++plays === 1
            ? new Promise((r) => (resolveOld = r))
            : Promise.resolve();
        const first = cam.startLiveStream();
        await new Promise((r) => setTimeout(r, 10));
        await cam.startLiveStream();
        const before = {
          streaming: cam.isStreaming,
          hasNew: video.srcObject === streams[1],
          stops: [...stops],
        };
        resolveOld();
        await first;
        return {
          before,
          after: {
            streaming: cam.isStreaming,
            hasNew: video.srcObject === streams[1],
            stops,
          },
        };
      }),
    ),
  );
} finally {
  await browser.close();
}
