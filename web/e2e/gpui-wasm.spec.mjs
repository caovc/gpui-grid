import {test, expect} from '@playwright/test';
import {PNG} from 'pngjs';

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
    // GPUI's two native buttons are right-aligned inside a 24px-padded
    // header. Use Canvas pointer events, not a JS call to change the model.
    // This exercises exactly the input path used by the desktop component.
    const {width} = bounds;
    await canvas.click({position: {x: width - 184, y: 35}});
    await expect.poll(getRows, {timeout: 15_000, message: 'GPUI Completed button click must change Rust view'}).toBeLessThan(initial);
    const filtered = await getRows();
    expect(filtered).toBeGreaterThan(0);
    const filteredImage = await canvas.screenshot();
    await info.attach('gpui-grid-filtered.png', {body: filteredImage, contentType: 'image/png'});
    await canvas.click({position: {x: width - 63, y: 35}});
    await expect.poll(getRows, {timeout: 15_000, message: 'GPUI All rows button must restore Rust view'}).toBe(initial);
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
