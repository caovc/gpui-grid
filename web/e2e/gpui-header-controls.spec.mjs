import {test, expect} from '@playwright/test';

// Click GPUI Kit's *actual* column header Popover; type into the native
// InputState and press Enter. JavaScript reads Rust diagnostics only.
test('GPUI header ⋯ menu filters Customer by typing in its native InputState', async ({page}, info) => {
  const errors = [];
  page.on('pageerror', error => errors.push(error.stack || error.message));
  const read = () => page.frameLocator('#gpui-frame').locator('body').evaluate(async () => {
    const wasm = await import('./pkg/gpui_grid_web.js');
    return {
      menu: wasm.preview_open_header_menu(),
      action: wasm.preview_last_header_action(),
      rows: wasm.preview_visible_rows(),
    };
  });

  try {
    await page.goto('/?feature=overview', {waitUntil: 'domcontentloaded'});
    await expect(page.locator('#runtime')).toContainText('GPUI WASM initialized', {timeout: 100_000});
    const canvas = page.frameLocator('#gpui-frame').locator('body > canvas');
    await expect(canvas).toBeVisible();
    expect((await read()).rows).toBe(12_480);
    const before = await canvas.screenshot();

    // Customer header occupies the second pinned column (x ~178–373).
    // Its last ~30px is the dedicated native GPUI Popover trigger.
    // A short coordinate tolerance list handles small font/layout differences
    // without triggering JS-side grid events or modifying the Rust model.
    for (const [x, y] of [[352, 125], [360, 125], [342, 125], [354, 120], [352, 130]]) {
      await canvas.click({position: {x, y}});
      const current = await read();
      if (current.menu === 'customer') break;
    }
    await expect.poll(async () => (await read()).menu, {
      timeout: 10_000,
      message: 'real GPUI Popover must open from the Customer header',
    }).toBe('customer');
    await info.attach('gpui-header-menu-open.png', {body: await canvas.screenshot(), contentType: 'image/png'});

    // The Popover uses track_focus() for its GPUI InputState; browser text and
    // Enter must travel through GPUI WebPlatform into Rust, not DOM filtering.
    await page.keyboard.insertText('Sophia');
    await page.keyboard.press('Enter');
    await expect.poll(async () => (await read()).action, {
      timeout: 15_000,
      message: 'filter input Enter should emit a Rust GPUI Filter action',
    }).toMatch(/^filter:customer:enter:\d+$/);
    const snapshot = await read();
    const filteredRows = Number(snapshot.action.split(':').at(-1));
    expect(filteredRows).toBeGreaterThan(0);
    expect(filteredRows).toBeLessThan(12_480);
    await info.attach('gpui-header-native-filtered.png', {body: await canvas.screenshot(), contentType: 'image/png'});
    expect(errors, 'no browser/WASM exceptions').toEqual([]);
  } finally {
    await info.attach('header-menu-browser-errors.json', {
      body: JSON.stringify(errors, null, 2), contentType: 'application/json',
    });
  }
});
