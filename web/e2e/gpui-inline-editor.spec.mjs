import { test, expect } from '@playwright/test';

// This test clicks and types into the actual Rust GPUI WebPlatform canvas.
// JS only reads wasm-bindgen diagnostics; it never changes row data.
test('inline GPUI CellEditor handles F2, Escape, and Tab without a DOM grid', async ({ page }, info) => {
  const errors = [];
  const consoleEvents = [];
  page.on('pageerror', error => errors.push(error.stack || error.message));
  page.on('console', message => {
    if (message.type() === 'error') consoleEvents.push(message.text());
  });
  const probe = async () => page.frameLocator('#gpui-frame').locator('body').evaluate(async () => {
    const wasm = await import('./pkg/gpui_grid_web.js');
    return {
      editorRow: wasm.preview_active_editor_row_id(),
      saved: wasm.preview_last_saved_edit(),
    };
  });

  try {
    await page.goto('/', { waitUntil: 'domcontentloaded' });
    await expect(page.locator('#runtime')).toContainText('GPUI WASM initialized', { timeout: 100_000 });
    const canvas = page.frameLocator('#gpui-frame').locator('body > canvas');
    await expect(canvas).toBeVisible();

    // Customer is editable, while the first Order ID column is readonly.
    // A regular click sets GPUI DataTable selection; F2 must start the Rust InputState.
    await canvas.click({ position: { x: 240, y: 158 } });
    await page.keyboard.press('F2');
    await expect.poll(async () => (await probe()).editorRow, { timeout: 15_000 }).toBe(1);
    await info.attach('gpui-inline-editor-f2.png', {
      body: await canvas.screenshot(), contentType: 'image/png',
    });

    await page.keyboard.press('Escape');
    await expect.poll(async () => (await probe()).editorRow, { timeout: 15_000 }).toBe(0);
    expect((await probe()).saved, 'Escape must never commit the draft').toBe('');

    // Double-click is the second real GPUI entry point to the same in-cell editor.
    await canvas.dblclick({ position: { x: 240, y: 158 }, delay: 75 });
    await expect.poll(async () => (await probe()).editorRow, { timeout: 15_000 }).toBe(1);
    const marker = '-GPUI-Tab-Saved';
    await page.keyboard.insertText(marker);
    await page.keyboard.press('Tab');

    await expect.poll(async () => (await probe()).saved, { timeout: 15_000 }).toContain(marker);
    expect((await probe()).saved, 'saved address must retain stable Rust row/column ID')
      .toContain('1:customer:');

    // Tab must reopen an editor for the next editable cell; Escape closes it.
    await expect.poll(async () => (await probe()).editorRow, { timeout: 15_000 }).toBe(1);
    await info.attach('gpui-inline-editor-tab-advanced.png', {
      body: await canvas.screenshot(), contentType: 'image/png',
    });
    await page.keyboard.press('Escape');
    await expect.poll(async () => (await probe()).editorRow, { timeout: 15_000 }).toBe(0);
    expect(errors, 'uncaught browser/WASM errors').toEqual([]);
  } finally {
    await info.attach('inline-editor-browser-diagnostics.json', {
      body: JSON.stringify({ errors, consoleEvents: consoleEvents.slice(-100) }, null, 2),
      contentType: 'application/json',
    });
  }
});
