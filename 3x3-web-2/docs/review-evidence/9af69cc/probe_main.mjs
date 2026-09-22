// Starts a private Vite server; exercises real JSON decoding and playback.
import { chromium } from '@playwright/test';
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';

const root = fileURLToPath(new URL('../../../', import.meta.url));
const base = 'http://127.0.0.1:5185';
const server = spawn(process.execPath, ['node_modules/vite/bin/vite.js', '--host', '127.0.0.1', '--port', '5185', '--strictPort'], { cwd: root, stdio: 'ignore' });
let browser;
const report = (name, value) => console.log(JSON.stringify({ name, ...value }));
const ready = page => page.waitForFunction(() => document.querySelector('#engine-status')?.textContent.includes('READY'));
const snapshot = page => page.evaluate(() => window.cube_store.getSnapshot());
async function setup() {
  const context = await browser.newContext({ viewport: { width: 1440, height: 1080 } });
  const page = await context.newPage();
  await page.goto(`${base}/?no-sw`);
  await ready(page);
  return { context, page };
}
try {
  for (let i = 0; i < 100; i++) {
    if (await fetch(base).then(r => r.ok).catch(() => false)) break;
    if (server.exitCode !== null || i === 99) throw new Error('Vite startup failed');
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  browser = await chromium.launch({ headless: true });
  {
    const { context, page } = await setup();
    const inputs = await page.evaluate(() => {
      const solved = 'UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB';
      return ['R', 'U'].map(move => JSON.parse(window.cube_studio.apply_moves(solved, move)).state);
    });
    await page.evaluate(() => {
      const original = File.prototype.text;
      window.releaseReads = {};
      File.prototype.text = function () {
        const name = this.name;
        const decoded = original.call(this);
        return new Promise(resolve => { window.releaseReads[name] = () => resolve(decoded); });
      };
    });
    for (const [index, name] of ['older.json', 'newer.json'].entries()) {
      await page.locator('#file').setInputFiles({ name, mimeType: 'application/json', buffer: Buffer.from(JSON.stringify({ version: 1, state: inputs[index] })) });
      await page.waitForFunction(name => !!window.releaseReads[name], name);
    }
    // Both selections have started. Complete the older read first.
    await page.evaluate(() => window.releaseReads['older.json']());
    await page.waitForFunction(state => window.cube_store.getState() === state, inputs[0]);
    await page.evaluate(() => window.releaseReads['newer.json']());
    await page.waitForFunction(() => document.querySelector('#message').textContent.includes('読込中にキューブが変更'));
    const actual = await snapshot(page);
    assert.equal(actual.state, inputs[0]);
    assert.notEqual(actual.state, inputs[1]);
    report('latest_json_selection_rejected', { olderState: inputs[0], newerState: inputs[1], actual, message: await page.locator('#message').textContent() });
    await context.close();
  }
  {
    const { context, page } = await setup();
    await page.getByRole('tab', { name: '手順を入力' }).click();
    await page.locator('#algorithm').fill('R U F');
    await page.locator('#apply-algorithm').click();
    await page.waitForFunction(() => !window.cube_scene?.active);
    await page.locator('#solve').click();
    await page.waitForFunction(() => !!window.cube_store.getSolution());
    await page.locator('#play').click();
    await page.waitForFunction(() => window.cube_store.getStep() === 1);
    await page.locator('#play').click();
    const paused = await snapshot(page);
    const saved = await page.evaluate(() => JSON.parse(localStorage.getItem('cube-studio-v1')));
    assert.equal(saved.state, paused.state);
    assert.deepEqual(saved.centerTurns, paused.centerTurns);
    await page.reload();
    await ready(page);
    assert.deepEqual(await snapshot(page), paused);
    report('midplay_pause_restores', { paused, restored: await snapshot(page) });
    await page.locator('#solve').click();
    await page.waitForFunction(() => !!window.cube_store.getSolution());
    await page.locator('#next').click();
    const leaving = await snapshot(page);
    // Leave during the manual step's animation, then return without URL state.
    await page.goto('about:blank');
    await page.goto(`${base}/?no-sw`);
    await ready(page);
    assert.deepEqual(await snapshot(page), leaving);
    report('leave_during_manual_seek_restores', { leaving, restored: await snapshot(page) });
    await context.close();
  }
} finally {
  await browser?.close();
  server.kill();
}
