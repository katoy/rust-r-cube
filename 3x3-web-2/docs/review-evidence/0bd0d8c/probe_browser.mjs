// Builds on an existing pkg/; starts a private Vite server on port 5183.
// REVIEW_BASE_URL can instead point to an existing dev server.
// Each probe uses its own disposable browser context and fake camera.
import { chromium } from '@playwright/test';
import { spawn } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
const root = fileURLToPath(new URL('../../../', import.meta.url));
const manifest = JSON.parse(readFileSync(new URL('../../../test-images/manifest.json',import.meta.url),'utf8'));
const base = process.env.REVIEW_BASE_URL || 'http://127.0.0.1:5183';
const server = process.env.REVIEW_BASE_URL ? undefined : spawn(process.execPath, ['node_modules/vite/bin/vite.js', '--host', '127.0.0.1', '--port', '5183', '--strictPort'], {cwd:root,stdio:'ignore'});
for(let i=0;i<100;i++) {
  if(await fetch(base).then(r=>r.ok).catch(()=>false)) break;
  if(server?.exitCode !== null && server?.exitCode !== undefined) throw new Error('Vite server exited');
  if(i===99) { server?.kill(); throw new Error('Vite startup timeout'); }
  await new Promise(resolve=>setTimeout(resolve,100));
}
const browser = await chromium.launch({headless:true,args:['--use-fake-device-for-media-stream','--use-fake-ui-for-media-stream']});
const report = (name, result) => console.log(JSON.stringify({name, ...result}));
async function setup() {
  const context = await browser.newContext({viewport:{width:1440,height:1080},permissions:['camera']});
  const page = await context.newPage();
  await page.goto(`${base}/?no-sw`);
  await page.waitForFunction(() => document.querySelector('#engine-status')?.textContent.includes('READY'));
  return {context,page};
}
try {
  {
    const {context,page} = await setup();
    await page.locator('#reduced-motion').check();
    await page.getByRole('tab',{name:'手順を入力'}).click();
    await page.locator('#algorithm').fill('R U F');
    await page.locator('#apply-algorithm').click();
    const scrambled = await page.evaluate(() => window.cube_store.getSnapshot());
    await page.locator('#solve').click();
    await page.waitForFunction(() => !!window.cube_store.getSolution());
    await page.locator('#last').click();
    await page.locator('#solution-close').click();
    const finished = await page.evaluate(() => ({shown:window.cube_store.getSnapshot(), saved:JSON.parse(localStorage.getItem('cube-studio-v1'))}));
    await page.reload();
    await page.waitForFunction(() => document.querySelector('#engine-status')?.textContent.includes('READY'));
    report('playback_persistence', {scrambled,finished,restored:await page.evaluate(() => window.cube_store.getSnapshot())});
    await context.close();
  }
  {
    const {context,page} = await setup();
    const before = await page.evaluate(async () => {
      const registration = await navigator.serviceWorker.register('/sw.js',{scope:'/other-app/'});
      const worker = registration.installing || registration.waiting || registration.active;
      if (worker.state !== 'activated') await new Promise((resolve,reject) => {
        const timer = setTimeout(() => reject(new Error('SW activation timed out')),10000);
        worker.addEventListener('statechange', () => {if(worker.state === 'activated'){clearTimeout(timer);resolve();}});
      });
      return (await navigator.serviceWorker.getRegistrations()).map(r=>r.scope);
    });
    await page.reload();
    await page.waitForFunction(async () => (await navigator.serviceWorker.getRegistrations()).length === 0);
    report('unrelated_service_worker_unregistered',{before,after:await page.evaluate(async () => (await navigator.serviceWorker.getRegistrations()).map(r=>r.scope))});
    await context.close();
  }
  {
    const {context,page} = await setup();
    await page.locator('#reduced-motion').uncheck();
    // Give undo a meaningful preceding state, then start solving an animated move.
    await page.locator('[data-move="U"]').click();
    await page.waitForFunction(() => !window.cube_scene.active);
    await page.evaluate(() => {
      document.querySelector('[data-move="R"]').click();
      document.querySelector('#undo').click();
    });
    const during = await page.evaluate(() => ({state:window.cube_store.getState(), animationActive:!!window.cube_scene.active}));
    await page.waitForFunction(() => !window.cube_scene.active);
    const after = await page.evaluate(() => ({state:window.cube_store.getState(), animationActive:!!window.cube_scene.active}));
    report('undo_during_animation',{during,after});
    await context.close();
  }
  {
    const {context,page} = await setup();
    await page.getByRole('tab',{name:'色を入力',exact:true}).click();
    await page.locator('#camera-colors').click();
    await page.locator('#camera-editor').waitFor({state:'visible'});
    await page.locator('#camera-file-a').setInputFiles(`${root}test-images/${manifest.images.solved.viewA}`);
    await page.waitForFunction(()=>window.__lastCamera.points.length===6);
    await page.locator('#camera-capture').click();
    const cell = page.locator('#camera-face-card-F button[data-index="2"]');
    await cell.focus();
    const before = await page.evaluate(() => ({tag:document.activeElement.tagName,face:document.activeElement.dataset.face,index:document.activeElement.dataset.index}));
    await page.keyboard.press('Space');
    const after = await page.evaluate(() => ({tag:document.activeElement.tagName,id:document.activeElement.id,face:document.activeElement.dataset.face,index:document.activeElement.dataset.index}));
    await page.keyboard.press('Tab');
    const nextTab = await page.evaluate(() => ({tag:document.activeElement.tagName,id:document.activeElement.id,face:document.activeElement.dataset.face,index:document.activeElement.dataset.index,label:document.activeElement.getAttribute('aria-label')}));
    report('camera_sticker_keyboard_focus',{before,after,nextTab,painted:await cell.getAttribute('data-color')});
    const palette = page.locator('#camera-palette [role="radio"]');
    await palette.first().focus();
    await page.keyboard.press('ArrowRight');
    report('camera_palette_arrow_navigation',{activeIndex:await page.evaluate(()=>Array.from(document.querySelectorAll('#camera-palette [role="radio"]')).indexOf(document.activeElement)),selected:await page.locator('#camera-palette [aria-checked="true"]').getAttribute('aria-label')});
    await context.close();
  }
  {
    const {context,page} = await setup();
    await page.getByRole('tab',{name:'色を入力',exact:true}).click();
    await page.locator('#camera-colors').click();
    await page.locator('#camera-live-stream').click();
    await page.waitForFunction(()=>window.__lastCamera.isStreaming);
    // Delay only the real toBlob callback to deterministically exercise ordering.
    await page.evaluate(()=> {
      const original = HTMLCanvasElement.prototype.toBlob;
      HTMLCanvasElement.prototype.toBlob = function(callback,...args) {
        return original.call(this,blob=>{ window.releaseCapturedFrame=()=>callback(blob); },...args);
      };
    });
    await page.locator('#camera-take-photo').click();
    await page.waitForFunction(()=>!!window.releaseCapturedFrame);
    await page.locator('#camera-file-a').setInputFiles(`${root}test-images/${manifest.images.solved.viewA}`);
    await page.waitForFunction(()=>window.__lastCamera.imageA?.naturalWidth===640);
    const afterNewFile = await page.locator('#camera-status-a').textContent();
    await page.evaluate(()=>window.releaseCapturedFrame());
    await page.waitForFunction(()=>window.__lastCamera.imageA?.naturalWidth!==640);
    report('old_capture_overwrites_new_file',{afterNewFile,afterOldCapture:await page.locator('#camera-status-a').textContent()});
    await context.close();
  }
} finally { await browser.close(); server?.kill(); }
