import { test, expect } from '@playwright/test';

// Exercises a genuine Rust/GPUI DataTable via Canvas pointer events.
// The wasm-bindgen bridge exposes read-only model-derived diagnostics.
test('Shift-click selects a stable Rust rectangular range in GPUI WebAssembly', async ({ page }, info) => {
  const pageErrors = [];
  page.on('pageerror', err => pageErrors.push(err.stack || err.message));
  const selectedCells = async () => page.frameLocator('#gpui-frame').locator('body').evaluate(async () => {
    const wasm = await import('./pkg/gpui_grid_web.js');
    return wasm.preview_selected_cell_count();
  });
  const rows = async () => page.frameLocator('#gpui-frame').locator('body').evaluate(async () => {
    const wasm = await import('./pkg/gpui_grid_web.js');
    return wasm.preview_visible_rows();
  });

  try {
    await page.goto('/', { waitUntil: 'domcontentloaded' });
    await expect(page.locator('#runtime')).toContainText('GPUI WASM initialized', { timeout: 100_000 });
    const canvas = page.frameLocator('#gpui-frame').locator('body > canvas');
    await expect(canvas).toBeVisible();
    expect(await rows()).toBe(12_480);
    expect(await selectedCells()).toBe(0);

    // Customer, first row. Read-only Order ID is not the selection anchor.
    await canvas.click({ position: { x: 240, y: 158 } });
    await expect.poll(selectedCells, { timeout: 15_000 }).toBe(1);
    const before = await canvas.screenshot();
    await info.attach('gpui-single-cell.png', { body: before, contentType: 'image/png' });

    // Company, second row. The GPUI click event must reach Rust with Shift.
    // It is 2 visible rows x 2 presentation columns, irrespective of storage indices.
    await canvas.click({ position: { x: 430, y: 190 }, modifiers: ['Shift'] });
    await expect.poll(selectedCells, {
      timeout: 15_000,
      message: 'Rust GridModel must extend anchor/focus to a 2x2 selection',
    }).toBe(4);
    const after = await canvas.screenshot();
    await info.attach('gpui-range-selection.png', { body: after, contentType: 'image/png' });
    expect(after.equals(before), 'range highlight should change real GPUI Canvas pixels').toBe(false);

    await canvas.click({ position: { x: 240, y: 158 } });
    await expect.poll(selectedCells, { timeout: 15_000 }).toBe(1);
    expect(pageErrors, 'no uncaught browser/WASM exceptions').toEqual([]);
  } finally {
    await info.attach('gpui-selection-errors.json', {
      body: JSON.stringify({ pageErrors }, null, 2),
      contentType: 'application/json',
    });
  }
});
