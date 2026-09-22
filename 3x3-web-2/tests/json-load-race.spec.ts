import { expect, test, type Page } from "@playwright/test";

declare global {
  interface Window {
    jsonReadControls: Record<
      string,
      { release: () => void; fail: (message: string) => void }
    >;
    jsonLoadCompletions: Record<string, Promise<unknown>>;
  }
}

async function prepareDelayedReads(page: Page) {
  await page.goto("/?no-sw");
  await expect(page.locator("#engine-status")).toContainText("READY");
  return page.evaluate(() => {
    const input = document.querySelector<HTMLInputElement>("#file")!;
    const handleChange = input.onchange!;
    const readText = File.prototype.text;
    window.jsonReadControls = {};
    window.jsonLoadCompletions = {};
    // Decode the real file, then hold its result at the asynchronous boundary.
    File.prototype.text = async function () {
      const content = await readText.call(this);
      await new Promise<void>((resolve, reject) => {
        window.jsonReadControls[this.name] = {
          release: resolve,
          fail: (message) => reject(new Error(message)),
        };
      });
      return content;
    };
    input.onchange = function (event) {
      const name = input.files![0].name;
      window.jsonLoadCompletions[name] = Promise.resolve(
        handleChange.call(this, event),
      );
    };
    const initial = window.cube_store.getState();
    const stateAfter = (move: string) =>
      JSON.parse(window.cube_studio.apply_moves(initial, move)).state as string;
    return { initial, older: stateAfter("R"), newer: stateAfter("U") };
  });
}

async function selectFile(page: Page, name: string, state: string) {
  await page.locator("#file").setInputFiles({
    name,
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify({ version: 1, state })),
  });
  await page.waitForFunction((name) => !!window.jsonReadControls[name], name);
}

async function completeRead(page: Page, name: string, error?: string) {
  await page.evaluate(
    async ({ name, error }) => {
      const control = window.jsonReadControls[name];
      if (error) control.fail(error);
      else control.release();
      // Wait through the real onchange handler, including catch and finally.
      await window.jsonLoadCompletions[name];
    },
    { name, error },
  );
}

test.describe("JSON file selection races", () => {
  for (const order of [
    ["older.json", "newer.json"],
    ["newer.json", "older.json"],
  ]) {
    test(`latest selection wins when ${order[0]} finishes first`, async ({
      page,
    }) => {
      const states = await prepareDelayedReads(page);
      await selectFile(page, "older.json", states.older);
      await selectFile(page, "newer.json", states.newer);
      const message = await page.locator("#message").textContent();

      for (const name of order) await completeRead(page, name);

      expect(await page.evaluate(() => window.cube_store.getState())).toBe(
        states.newer,
      );
      await expect(page.locator("#message")).toHaveText(message ?? "");
      await expect(page.locator("#file")).toHaveValue("");
    });
  }

  test("an obsolete read error preserves the pending selection and message", async ({
    page,
  }) => {
    const states = await prepareDelayedReads(page);
    await selectFile(page, "older.json", states.older);
    await selectFile(page, "newer.json", states.newer);
    const message = await page.locator("#message").textContent();

    await completeRead(page, "older.json", "obsolete file read failed");

    expect.soft(await page.locator("#message").textContent()).toBe(message);
    expect
      .soft(
        await page
          .locator("#file")
          .evaluate((input: HTMLInputElement) => input.files?.[0]?.name),
      )
      .toBe("newer.json");
    expect(await page.evaluate(() => window.cube_store.getState())).toBe(
      states.initial,
    );
    await completeRead(page, "newer.json");
    expect(await page.evaluate(() => window.cube_store.getState())).toBe(
      states.newer,
    );
  });

  test("manual moves during the latest read still prevent replacement", async ({
    page,
  }) => {
    const states = await prepareDelayedReads(page);
    await selectFile(page, "newer.json", states.newer);
    await page.locator("#reduced-motion").check();
    await page.locator('button[data-move="R"]').click();
    const manualState = await page.evaluate(() => window.cube_store.getState());
    expect(manualState).toBe(states.older);

    await completeRead(page, "newer.json");

    expect(await page.evaluate(() => window.cube_store.getState())).toBe(
      manualState,
    );
    await expect(page.locator("#message")).toContainText(
      "読込中にキューブが変更されました",
    );
    await expect(page.locator("#file")).toHaveValue("");
  });
});
