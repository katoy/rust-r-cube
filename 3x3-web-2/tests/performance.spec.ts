import { test, expect } from "@playwright/test";
test("1000 deterministic states in a real WASM worker", async ({ page }) => {
  test.skip(
    process.env.CUBE_BENCH !== "1",
    "Run npm run benchmark:web for the full performance corpus.",
  );
  test.setTimeout(180000);
  await page.goto("/");
  await expect(page.locator("#engine-status")).toContainText("READY");
  const result = await page.evaluate(async () => {
    const wasm = await import(/* @vite-ignore */ "/pkg/cube_studio.js");
    await wasm.default({ module_or_path: "/pkg/cube_studio_bg.wasm" });
    const worker = new Worker("/web/solver.worker.ts", { type: "module" });
    const initialization = await new Promise<number>((resolve, reject) => {
      worker.onmessage = ({ data }) =>
        data.kind === "ready" ? resolve(data.elapsed) : reject(data.error);
      worker.onerror = reject;
    });
    const times: number[] = [],
      lengths: number[] = [];
    let ticks = 0;
    const heartbeat = setInterval(() => ticks++, 10);
    const solved = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
    try {
      for (let seed = 1; seed <= 1000; seed++) {
        const input = JSON.parse(
          wasm.apply_moves(solved, wasm.scramble(seed)),
        ).state;
        const data = await new Promise<any>((resolve, reject) => {
          worker.onmessage = ({ data }) =>
            data.error ? reject(new Error(data.error)) : resolve(data.result);
          worker.postMessage({
            kind: "solve",
            id: seed,
            revision: seed,
            state: input,
            budget: 5000,
          });
        });
        if (data.state !== solved) throw new Error("Unverified solution");
        times.push(data.elapsed_ms);
        lengths.push(data.moves.length);
      }
    } finally {
      clearInterval(heartbeat);
      worker.terminate();
    }
    times.sort((a, b) => a - b);
    lengths.sort((a, b) => a - b);
    return {
      initialization,
      p50: times[500],
      p95: times[950],
      max: times[999],
      movesP50: lengths[500],
      movesMax: lengths[999],
      mainThreadTicks: ticks,
    };
  });
  console.log("WASM_BENCHMARK", JSON.stringify(result));
  expect(result.p95).toBeLessThan(2000);
  expect(result.mainThreadTicks).toBeGreaterThan(10);
});
