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


test('Shift+arrows and Ctrl/Cmd+A extend real GPUI selection without materializing 120k DOM cells', async ({ page }, info) => {
  const errors = [];
  page.on('pageerror', error => errors.push(error.stack || error.message));
  const selectedCount = () => page.frameLocator('#gpui-frame').locator('body').evaluate(async () => {
    const wasm = await import('./pkg/gpui_grid_web.js');
    return wasm.preview_selected_cell_count();
  });

  try {
    await page.goto('/', { waitUntil: 'domcontentloaded' });
    await expect(page.locator('#runtime')).toContainText('GPUI WASM initialized', { timeout: 100_000 });
    const canvas = page.frameLocator('#gpui-frame').locator('body > canvas');
    await expect(canvas).toBeVisible();
    await canvas.click({ position: { x: 240, y: 158 } });
    await expect.poll(selectedCount, { timeout: 15_000 }).toBe(1);

    // GPUI DataTable's selected cell is the starting position. The custom
    // shift-arrow bindings operate inside its key context, not in JavaScript.
    await page.keyboard.press('Shift+ArrowDown');
    await expect.poll(selectedCount, {
      timeout: 15_000, message: 'Rust must extend one row without changing the anchor',
    }).toBe(2);

    await page.keyboard.press('Shift+ArrowRight');
    await expect.poll(selectedCount, {
      timeout: 15_000, message: 'Rust must extend to the 2x2 keyboard rectangle',
    }).toBe(4);
    await info.attach('gpui-keyboard-rectangle.png', { body: await canvas.screenshot(), contentType: 'image/png' });

    // The browser's primary modifier maps to Ctrl on Linux and Cmd on macOS.
    // 12,480 data rows × 10 displayed columns is a single Rust CellRange.
    await page.keyboard.press('ControlOrMeta+A');
    await expect.poll(selectedCount, {
      timeout: 15_000, message: 'Select All must cover only currently visible GPUI cells',
    }).toBe(124_800);

    // A plain pointer click collapses the large range to one stable cell.
    await canvas.click({ position: { x: 240, y: 158 } });
    await expect.poll(selectedCount, { timeout: 15_000 }).toBe(1);
    expect(errors, 'uncaught genuine GPUI/WASM exceptions').toEqual([]);
  } finally {
    await info.attach('gpui-keyboard-selection-errors.json', {
      body: JSON.stringify({ errors }, null, 2),
      contentType: 'application/json',
    });
  }
});
