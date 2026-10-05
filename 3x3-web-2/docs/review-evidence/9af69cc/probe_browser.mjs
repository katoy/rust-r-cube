// Run from the repository root with an existing Vite dev server on 5173.
// REVIEW_BASE_URL may select another dev server. Uses disposable contexts.
import { chromium } from '@playwright/test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../../../', import.meta.url));
const evidence = fileURLToPath(new URL('./', import.meta.url));
const manifest = JSON.parse(readFileSync(`${root}test-images/manifest.json`, 'utf8'));
const base = process.env.REVIEW_BASE_URL || 'http://127.0.0.1:5173';
const browser = await chromium.launch({
  headless: true,
  args: ['--use-fake-device-for-media-stream', '--use-fake-ui-for-media-stream'],
});
const report = (name, result) => console.log(JSON.stringify({ name, ...result }));
async function setupCamera() {
  const context = await browser.newContext({ viewport: { width: 1440, height: 1080 }, permissions: ['camera'] });
  const page = await context.newPage();
  await page.goto(`${base}/?no-sw`);
  await page.waitForFunction(() => document.querySelector('#engine-status')?.textContent.includes('READY'));
  await page.getByRole('tab', { name: '色を入力', exact: true }).click();
  await page.locator('#camera-colors').click();
  await page.locator('#camera-editor').waitFor({ state: 'visible' });
  return { context, page };
}
async function upload(page, name = 'solved', view = 'A') {
  await page.locator(`#camera-file-${view.toLowerCase()}`).setInputFiles(`${root}test-images/${manifest.images[name][`view${view}`]}`);
  await page.waitForFunction((view) => !!window.__lastCamera[`image${view}`], view);
  await page.waitForFunction(() => window.__lastCamera.points.length === 6);
}
try {
  {
    const { context, page } = await setupCamera();
    await page.locator('#camera-live-stream').click();
    await page.waitForFunction(() => window.__lastCamera.isStreaming);
    await upload(page);
    await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
    const before = await page.evaluate(() => {
      const cam = window.__lastCamera;
      const canvas = document.querySelector('#camera-canvas');
      const reference = document.createElement('canvas');
      reference.width = canvas.width; reference.height = canvas.height;
      reference.getContext('2d').drawImage(cam.imageA, 0, 0, canvas.width, canvas.height);
      return {
        isStreaming: cam.isStreaming,
        trackState: document.querySelector('#camera-video').srcObject?.getVideoTracks()[0]?.readyState,
        sourceSize: [cam.imageA.naturalWidth, cam.imageA.naturalHeight],
        shownPixel: Array.from(canvas.getContext('2d').getImageData(300, 120, 1, 1).data),
        uploadedPixel: Array.from(reference.getContext('2d').getImageData(300, 120, 1, 1).data),
        readButtonEnabled: !document.querySelector('#camera-capture').disabled,
      };
    });
    await page.locator('#camera-canvas').screenshot({ path: `${evidence}live-after-file.png` });
    await page.locator('#camera-capture').click();
    const after = await page.evaluate(() => ({ isStreaming: window.__lastCamera.isStreaming, faces: window.__lastCamera.faces }));
    assert(before.isStreaming && before.readButtonEnabled);
    assert.notDeepEqual(before.shownPixel, before.uploadedPixel);
    assert.deepEqual(after.faces, { U: 'UUUUUUUUU', R: 'RRRRRRRRR', F: 'FFFFFFFFF' });
    report('file_selected_while_live', { before, after });
    await context.close();
  }
  {
    const { context, page } = await setupCamera();
    await upload(page);
    const initial = await page.evaluate(() => window.__lastCamera.points);
    const repeated = [];
    for (let i = 0; i < 5; i++) {
      await page.locator('#camera-detect').click();
      repeated.push(await page.evaluate(() => window.__lastCamera.points));
    }
    const cleanReference = await page.evaluate(async () => {
      const { detectCubeOutline } = await import('/web/camera-geometry.ts');
      const visible = document.querySelector('#camera-canvas');
      const canvas = document.createElement('canvas');
      canvas.width = visible.width; canvas.height = visible.height;
      canvas.getContext('2d').drawImage(window.__lastCamera.imageA, 0, 0, canvas.width, canvas.height);
      return detectCubeOutline(canvas);
    });
    await page.locator('#camera-clear-points').click();
    await page.locator('#camera-detect').click();
    const afterClear = await page.evaluate(() => window.__lastCamera.points);
    assert.deepEqual(initial, cleanReference);
    assert.deepEqual(afterClear, cleanReference);
    report('outline_reads_its_own_overlay', { initial, repeated, cleanReference, afterClear });
    await context.close();
  }
  {
    const { context, page } = await setupCamera();
    await upload(page);
    const points = await page.evaluate(() => window.__lastCamera.points);
    const box = await page.locator('#camera-canvas').boundingBox();
    const size = await page.locator('#camera-canvas').evaluate(c => ({ width: c.width, height: c.height }));
    const p = points[0];
    await page.mouse.move(box.x + p.x * box.width / size.width, box.y + p.y * box.height / size.height);
    await page.mouse.down();
    await page.mouse.move(box.x + p.x * box.width / size.width, box.y + (p.y - 25) * box.height / size.height, { steps: 5 });
    await page.mouse.up();
    const edited = await page.evaluate(() => window.__lastCamera.points);
    await page.locator('#camera-view-a').click();
    const afterSameTab = await page.evaluate(() => window.__lastCamera.points);
    assert.notDeepEqual(edited, afterSameTab);
    report('active_camera_tab_discards_adjustment', { initial: points, edited, afterSameTab });
    await context.close();
  }
} finally {
  await browser.close();
}
