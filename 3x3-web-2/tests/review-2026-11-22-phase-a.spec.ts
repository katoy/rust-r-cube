import { test, expect } from "@playwright/test";
import fs from "node:fs";
import path from "node:path";
import vm from "node:vm";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));
const swSource = fs.readFileSync(path.join(root, "public/sw.js"), "utf-8");

test.describe("2026-11-22 Code Review Phase A Regressions", () => {
  test("CORR-01: countPieceOrientations counts distinct pieces on single U/R/F turns", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const results = await page.evaluate(async () => {
      const model = await import("/web/model.ts");
      const centers = await import("/web/centers.ts");
      const studio = (window as any).cube_studio;

      const solvedInfo = model.getCellArrowInfo(
        model.SOLVED_3X3,
        [0, 0, 0, 0, 0, 0],
      );
      const incompleteInfo = model.getCellArrowInfo(
        "?".repeat(54),
        [0, 0, 0, 0, 0, 0],
      );
      // 既知スクランブル状態での方向ベクトル・角チェック
      const scrambled = JSON.parse(
        studio.apply_moves(model.SOLVED_3X3, "R U F'"),
      );
      const rotScrambled = centers.rotateCenters(
        [0, 0, 0, 0, 0, 0],
        ["R", "U", "F'"],
      );
      const infoScrambled = model.getCellArrowInfo(
        scrambled.state,
        rotScrambled,
      );

      const cases = ["U", "R", "F"].map((mv) => {
        const applied = JSON.parse(studio.apply_moves(model.SOLVED_3X3, mv));
        const rot = centers.rotateCenters([0, 0, 0, 0, 0, 0], [mv]);
        const info = model.getCellArrowInfo(applied.state, rot);
        return { mv, counts: model.countPieceOrientations(info) };
      });
      return { cases, solvedInfo, incompleteInfo, infoScrambled };
    });

    expect(results.solvedInfo.angles.every((a: number) => a === 0)).toBe(true);
    expect(results.incompleteInfo.angles).toHaveLength(54);
    expect(results.infoScrambled.angles[4]).toBeCloseTo(Math.PI / 2, 5); // Uセンター +90°
    expect(results.infoScrambled.angles[22]).toBeCloseTo(-Math.PI / 2, 5); // F'センター -90°

    for (const c of results.cases) {
      expect(c.counts, `single move ${c.mv}`).toMatchObject({
        cornerTwist: 4,
        edgeFlip: 4,
        centerRotate: 1,
      });
      if (c.mv === "U") {
        expect(c.counts.normal).toBe(45);
      } else {
        expect(c.counts.normal).toBeGreaterThan(0);
        expect(c.counts.normal).toBeLessThan(45);
      }
    }
  });

  test("CORR-04 & CORR-07: url-params rejects empty/decimal centers and oversized alg", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const parsed = await page.evaluate(async () => {
      const urlParams = await import("/web/url-params.ts");
      return {
        emptyCommas: urlParams.parseUrlParams("?centers=,,,,,"),
        decimal: urlParams.parseUrlParams("?centers=1.5,0,0,0,0,0"),
        negative: urlParams.parseUrlParams("?centers=-1,0,0,0,0,0"),
        valid: urlParams.parseUrlParams("?centers=0,1,2,3,0,1"),
        oversizedAlg: urlParams.parseUrlParams(`?alg=${"U".repeat(4097)}`),
        maxAlg: urlParams.parseUrlParams(`?alg=${"U".repeat(4096)}`),
      };
    });

    expect(parsed.emptyCommas.hasInvalidCenters).toBe(true);
    expect(parsed.emptyCommas.centers).toBeUndefined();
    expect(parsed.decimal.hasInvalidCenters).toBe(true);
    expect(parsed.decimal.centers).toBeUndefined();
    expect(parsed.negative.hasInvalidCenters).toBe(true);
    expect(parsed.valid.centers).toEqual([0, 1, 2, 3, 0, 1]);
    expect(parsed.oversizedAlg.alg).toBeUndefined();
    expect(parsed.maxAlg.alg?.length).toBe(4096);
  });

  test("CORR-07: file-io checks content length and uses Object.hasOwn", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const fileResults = await page.evaluate(async () => {
      const fileIo = await import("/web/file-io.ts");
      const model = await import("/web/model.ts");

      let oversizedErr = "";
      try {
        fileIo.validateAndParseCubeJson("a".repeat(65537), 100);
      } catch (e) {
        oversizedErr = String(e);
      }

      let protoErr = "";
      try {
        fileIo.validateAndParseCubeJson('{"__proto__":{"version":1}}', 30);
      } catch (e) {
        protoErr = String(e);
      }

      const valid = fileIo.validateAndParseCubeJson(
        JSON.stringify({
          version: 1,
          state: model.SOLVED_3X3,
          centerTurns: [0, 0, 0, 0, 0, 0],
        }),
        120,
      );

      return { oversizedErr, protoErr, valid };
    });

    expect(fileResults.oversizedErr).toContain("64KB");
    expect(fileResults.protoErr).toContain("Cube Studio v1");
    expect(fileResults.valid.state).toHaveLength(54);
  });

  test("CORR-07: camera readImageDimensions verifies PNG IHDR chunk tag", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const dimResult = await page.evaluate(async () => {
      const camera = await import("/web/camera.ts");
      // PNG シグネチャだがチャンクタイプが "IHDR" ではない ("FAKE")
      const fakePng = new Uint8Array(32);
      fakePng.set([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a], 0);
      fakePng.set([0x00, 0x00, 0x00, 0x0d, 0x46, 0x41, 0x4b, 0x45], 8);
      new DataView(fakePng.buffer).setUint32(16, 100);
      new DataView(fakePng.buffer).setUint32(20, 100);

      // 正規 PNG IHDR
      const validPng = new Uint8Array(fakePng);
      validPng.set([0x49, 0x48, 0x44, 0x52], 12);

      return {
        fake: await camera.readImageDimensions(new Blob([fakePng])),
        valid: await camera.readImageDimensions(new Blob([validPng])),
      };
    });

    expect(dimResult.fake).toBeNull();
    expect(dimResult.valid).toEqual({ width: 100, height: 100 });
  });

  test("CORR-05: CubeStore does not push redundant history on 2*PI center turn equivalence", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await expect(page.locator("#engine-status")).toContainText("READY");

    const storeRes = await page.evaluate(async () => {
      const { CubeStore } = await import("/web/cube-store.ts");
      const store = new CubeStore();
      // 同一状態かつ +2*PI 回転 (ターン数としては 0 と等価)
      store.applyAlgorithmResult(store.getState(), [
        Math.PI * 2,
        0,
        0,
        0,
        0,
        0,
      ]);
      const undoAfterFullTurn = store.canUndo();
      // 90° 回転では履歴が積まれること
      store.applyAlgorithmResult(store.getState(), [
        Math.PI / 2,
        0,
        0,
        0,
        0,
        0,
      ]);
      const undoAfterQuarterTurn = store.canUndo();
      return { undoAfterFullTurn, undoAfterQuarterTurn };
    });

    expect(storeRes.undoAfterFullTurn).toBe(false);
    expect(storeRes.undoAfterQuarterTurn).toBe(true);
  });

  test("PERF-04: 3x3 face move does not rebuild preset buttons DOM", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await expect(page.locator("#engine-status")).toContainText("READY");
    await page.locator("#reduced-motion").check();

    await page.evaluate(() => {
      (window as any).__firstPresetBtn = document.querySelector(
        "#preset-buttons button",
      );
    });

    await page.locator('[data-move="U"]').click();
    await expect(page.locator("#cube-status")).not.toHaveText("完成状態");

    const sameNode = await page.evaluate(() => {
      return (
        document.querySelector("#preset-buttons button") ===
        (window as any).__firstPresetBtn
      );
    });
    expect(sameNode).toBe(true);
  });

  test("CORR-02: compareAllSolvers recovers Worker via restart after solver failure", async ({
    page,
  }) => {
    await page.goto("/?no-sw");
    await expect(page.locator("#engine-status")).toContainText("READY");
    await page.locator("#reduced-motion").check();
    await page.locator("#scramble").click();

    await page.evaluate(async () => {
      const dbg = (window as any).__cube_main_debug__;
      const solver = dbg.getSolver();
      const origSolve = solver.solve.bind(solver);
      let tripped = false;
      solver.solve = (...args: any[]) => {
        if (!tripped) {
          tripped = true;
          solver.ready = false;
          return Promise.reject(new Error("simulated worker timeout"));
        }
        return origSolve(...args);
      };
      await dbg.compareAllSolvers(5000);
    });

    const rows = page.locator("#solver-compare-list .solver-compare-row");
    await expect(rows).toHaveCount(4);
    await expect(rows.nth(0)).toContainText("失敗");
    await expect(rows.nth(1)).toContainText("手");
    await expect(rows.nth(2)).toContainText("手");
  });

  test("SIMP-04 & PERF-09: sw.js deletes old caches without migration and skips no-store", async () => {
    expect(swSource).not.toContain("MAX_MIGRATED_ITEMS");
    expect(swSource).toContain("no-store");

    const listeners: Record<string, (e: any) => void> = {};
    const deleted: string[] = [];
    const puts: string[] = [];
    const mockCache = {
      addAll: async () => {},
      match: async () => undefined,
      keys: async () => [],
      put: async (k: string) => {
        puts.push(k);
      },
      delete: async () => true,
    };

    const sandbox: any = {
      URL,
      Set,
      Promise,
      console,
      Response: { error: () => ({ status: 0 }) },
      fetch: async () => ({
        status: 200,
        type: "basic",
        headers: {
          get: (h: string) => (h === "Cache-Control" ? "no-store" : null),
        },
        clone() {
          return this;
        },
      }),
      caches: {
        open: async () => mockCache,
        keys: async () => ["cube-studio-root-v0", "cube-studio-root-v1"],
        delete: async (k: string) => {
          deleted.push(k);
          return true;
        },
      },
      self: {
        location: {
          href: "https://example.com/",
          origin: "https://example.com",
        },
        registration: { scope: "https://example.com/" },
        skipWaiting: async () => {},
        clients: { claim: async () => {} },
        addEventListener: (type: string, fn: (e: any) => void) => {
          listeners[type] = fn;
        },
      },
    };
    vm.runInNewContext(swSource, sandbox);

    const waitPromises: Promise<unknown>[] = [];
    listeners.activate({
      waitUntil: (p: Promise<unknown>) => waitPromises.push(p),
    });
    await Promise.all(waitPromises.splice(0));
    expect(deleted).toEqual(["cube-studio-root-v0"]);

    let respondedWith: Promise<unknown> | undefined;
    listeners.fetch({
      request: {
        method: "GET",
        mode: "cors",
        url: "https://example.com/assets/test.js",
      },
      respondWith: (p: Promise<unknown>) => {
        respondedWith = p;
      },
      waitUntil: (p: Promise<unknown>) => waitPromises.push(p),
    });
    await respondedWith;
    await Promise.all(waitPromises);
    expect(puts).toEqual([]);
  });
});
