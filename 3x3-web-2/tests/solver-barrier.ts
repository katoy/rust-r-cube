import type { Page } from "@playwright/test";
import type { AppStateMachine } from "../web/app-state";
import type { CubeStore } from "../web/cube-store";
import type { SolverClient } from "../web/solver-client";

declare global {
  interface Window {
    __cube_main_debug__?: {
      appState: AppStateMachine;
      store: CubeStore;
      getSolver(): SolverClient | undefined;
    };
    __solver_test_barrier__?: {
      release(): void;
      resultReady: boolean;
    };
  }
}

export async function holdSolverResults(page: Page) {
  await page.evaluate(() => {
    const client = window.__cube_main_debug__?.getSolver();
    if (!client) throw new Error("Solver debug hook is unavailable");
    const original = client.solve.bind(client);
    let release!: () => void;
    const gate = new Promise<void>((resolve) => {
      release = resolve;
    });
    const barrier = { release, resultReady: false };
    window.__solver_test_barrier__ = barrier;
    client.solve = async (...args: Parameters<typeof original>) => {
      const result = await original(...args);
      barrier.resultReady = true;
      await gate;
      return result;
    };
  });
}

export async function releaseSolverResults(page: Page) {
  await page.evaluate(() => window.__solver_test_barrier__?.release());
}
