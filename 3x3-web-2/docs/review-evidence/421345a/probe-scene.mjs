import assert from "node:assert/strict";
import { writeFile } from "node:fs/promises";
import { chromium } from "@playwright/test";

const browser = await chromium.launch();
try {
  const page = await browser.newPage();
  await page.goto("http://127.0.0.1:5173/?no-sw");
  await page.locator("#engine-status").filter({ hasText: "READY" }).waitFor();
  await page.waitForFunction(() => {
    const scene = window.cube_scene;
    return scene && scene.renderer.info.memory.textures >= 8;
  });
  const result = await page.evaluate(() => {
    const scene = window.cube_scene;
    let shadow;
    scene.scene.traverse((object) => {
      if (object.isLight && object.shadow?.map) shadow = object.shadow;
    });
    if (!shadow) throw new Error("Allocated shadow map not found");
    let renderTargetDisposals = 0;
    shadow.map.addEventListener("dispose", () => {
      renderTargetDisposals++;
    });
    const before = { ...scene.renderer.info.memory };
    scene.dispose();
    return {
      before,
      after: { ...scene.renderer.info.memory },
      renderTargetDisposals,
      shadowMapSize: [shadow.map.width, shadow.map.height],
      shadowColorTextureCount: shadow.map.textures.length,
      shadowHasDepthTexture: !!shadow.map.depthTexture,
      retainedByWindow: window.cube_scene === scene,
    };
  });
  assert.equal(result.renderTargetDisposals, 0);
  assert.ok(result.after.textures >= 2);
  await writeFile(
    new URL("./probe-scene.json", import.meta.url),
    JSON.stringify(result, null, 2) + "\n",
  );
  console.log(JSON.stringify(result, null, 2));
} finally {
  await browser.close();
}
