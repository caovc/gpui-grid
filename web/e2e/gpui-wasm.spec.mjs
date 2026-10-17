import {test, expect} from '@playwright/test';
import {PNG} from 'pngjs';
import {openCustomerHeaderMenu} from './column-menu.mjs';

/** The Canvas is drawn by Rust/GPUI, not by DOM elements. A screenshot is
 * required: the mere existence of a canvas does not prove a rendered frame. */
function paintedPixels(image) {
  const png = PNG.sync.read(image);
  let nonWhite = 0;
  const unique = new Set();
  for (let i = 0; i < png.data.length; i += 4 * 12) {
    const [r, g, b, a] = png.data.subarray(i, i + 4);
    if (a > 0 && (r < 232 || g < 232 || b < 232)) nonWhite++;
    unique.add(`${r},${g},${b}`);
  }
  return {nonWhite, uniqueColors: unique.size, width: png.width, height: png.height};
}

test('actual Rust/GPUI WebAssembly paints and handles real canvas click events', async ({page}, info) => {
  const consoleEvents = [];
  const wasmRequests = [];
  const errors = [];
  page.on('console', event => consoleEvents.push(`[${event.type()}] ${event.text()}`));
  page.on('pageerror', error => errors.push(error.stack || error.message));
  page.on('response', response => {
    if (response.url().endsWith('_bg.wasm')) {
      wasmRequests.push({url: response.url(), status: response.status(), contentType: response.headers()['content-type']});
    }
  });
  try {
    await page.goto('/', {waitUntil: 'domcontentloaded'});
    const runtime = page.locator('#runtime');
    await expect(runtime).toContainText('GPUI WASM initialized', {timeout: 100_000});
    const frame = page.frameLocator('#gpui-frame');
    const canvas = frame.locator('body > canvas');
    await expect(canvas).toBeVisible();
    const bounds = await canvas.boundingBox();
    expect(bounds?.width).toBeGreaterThan(850);
    expect(bounds?.height).toBeGreaterThan(380);
    expect(wasmRequests, 'browser must really fetch the compiled WASM binary').toEqual([
      expect.objectContaining({status: 200, contentType: expect.stringContaining('application/wasm')}),
    ]);
    // Avoid falsely reporting success for a blank WebGL/WebGPU surface. Wait
    // until at least one full raster pass after the startup message.
    await page.waitForTimeout(1200);
    const image = await canvas.screenshot();
    await info.attach('gpui-grid-real-wasm-first-frame.png', {body: image, contentType: 'image/png'});
    const painted = paintedPixels(image);
    await info.attach('painted-pixels.json', {body: JSON.stringify(painted, null, 2), contentType: 'application/json'});
    expect(painted.nonWhite, 'blank canvas: no GPUI table pixels were drawn').toBeGreaterThan(2000);
    expect(painted.uniqueColors, 'blank canvas: GPUI view should have a varied color palette').toBeGreaterThan(10);

    const getRows = () => frame.locator('body').evaluate(async () => {
      const wasm = await import('./pkg/gpui_grid_web.js');
      return wasm.preview_visible_rows();
    });
    const initial = await getRows();
    expect(initial).toBe(12_480);
    // The showcase has no toolbar buttons. Exercise the *reusable* GPUI
    // column-header filter Popover using real Canvas pointer + keyboard input.
    const readHeader = () => frame.locator('body').evaluate(async () => {
      const wasm = await import('./pkg/gpui_grid_web.js');
      return {
        menu: wasm.preview_open_header_menu(),
        action: wasm.preview_last_header_action(),
      };
    });
    await openCustomerHeaderMenu(canvas, readHeader);
    await page.keyboard.insertText('Sophia');
    await page.keyboard.press('Enter');
    await expect.poll(async () => (await readHeader()).action, {
      timeout: 15_000, message: 'GPUI InputState filter must update Rust GridModel',
    }).toMatch(/^filter:customer:enter:\d+$/);
    const filterAction = (await readHeader()).action;
    const filtered = Number(filterAction.split(':').at(-1));
    expect(filtered, 'Rust typed column filter should reduce visible records').toBeGreaterThan(0);
    expect(filtered).toBeLessThan(initial);
    await info.attach('gpui-grid-native-column-filter.png', {
      body: await canvas.screenshot(), contentType: 'image/png',
    });
    // A new WASM instance resets to its original dataset; no JS grid mutation.
    await page.reload({waitUntil: 'domcontentloaded'});
    await expect(runtime).toContainText('GPUI WASM initialized', {timeout: 100_000});
    await expect.poll(getRows, {timeout: 15_000}).toBe(initial);
    expect(errors, 'uncaught WASM JavaScript errors').toEqual([]);
  } finally {
    await info.attach('browser-diagnostics.json', {
      body: JSON.stringify({
        consoleEvents: consoleEvents.slice(-200),
        wasmRequests,
        errors,
        userAgent: await page.evaluate(() => navigator.userAgent).catch(() => 'not available'),
      }, null, 2),
      contentType: 'application/json',
    });
  }
});


test('real GPUI double-click opens Rust editor; typed text is committed through InputState', async ({page}, info) => {
  const uncaughtErrors = [];
  const consoleErrors = [];
  page.on('pageerror', error => uncaughtErrors.push(error.stack || error.message));
  // GPUI WebPlatform may report recoverable asset misses or repeated bootstrap
  // diagnostics at console.error level. Capture them without falsely marking a
  // successful Rust edit transaction as a fatal browser exception.
  page.on('console', event => {if(event.type() === 'error') consoleErrors.push(event.text());});
  try {
    await page.goto('/', {waitUntil: 'domcontentloaded'});
    await expect(page.locator('#runtime')).toContainText('GPUI WASM initialized', {timeout: 100_000});
    const frame = page.frameLocator('#gpui-frame');
    const canvas = frame.locator('body > canvas');
    await expect(canvas).toBeVisible();

    // The first data row begins ~145 px below the GPUI canvas origin; 240px
    // lands in its editable Customer cell, not the read-only Order ID column.
    // This is a real double-click dispatched to GPUI's TableEvent pipeline.
    await canvas.dblclick({position: {x: 240, y: 158}, delay: 80});
    const diag = () => frame.locator('body').evaluate(async () => {
      const wasm = await import('./pkg/gpui_grid_web.js');
      return {editing: wasm.preview_active_editor_row_id(), saved: wasm.preview_last_saved_edit()};
    });
    await expect.poll(async () => (await diag()).editing, {
      timeout: 15_000, message: 'Double-click should open the Rust GPUI CellEditor for row 1',
    }).toBe(1);
    await info.attach('real-gpui-cell-editor-open.png', {body: await canvas.screenshot(), contentType: 'image/png'});

    // GPUI input is focused by Rust when the edit session begins. Type via the
    // browser keyboard/IME path; NEVER call a JS model mutation or fake grid.
    const marker = '-playwright-typed';
    await page.keyboard.insertText(marker);
    await page.keyboard.press('Enter');
    await expect.poll(async () => (await diag()).editing, {
      timeout: 15_000, message: 'PressEnter should commit the Rust GPUI edit',
    }).toBe(0);
    expect((await diag()).saved).toContain('1:customer:');
    expect((await diag()).saved).toContain(marker);
    await info.attach('real-gpui-cell-editor-committed.png', {body: await canvas.screenshot(), contentType: 'image/png'});
    expect(uncaughtErrors, 'no uncaught JavaScript/WASM exceptions').toEqual([]);
  } finally {
    await info.attach('editor-browser-errors.json', {body: JSON.stringify({uncaughtErrors, consoleErrors: consoleErrors.slice(-120), totalConsoleErrors: consoleErrors.length}, null, 2), contentType: 'application/json'});
  }
});
