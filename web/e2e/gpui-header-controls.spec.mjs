import {test, expect} from '@playwright/test';
import {openCustomerHeaderMenu} from './column-menu.mjs';

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
    await openCustomerHeaderMenu(canvas, read);
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

test('native header Popover exposes real Rust sort and column-hide controls', async ({page}, info) => {
  const browserErrors = [];
  page.on('pageerror', err => browserErrors.push(err.stack || err.message));
  const diag = () => page.frameLocator('#gpui-frame').locator('body').evaluate(async () => {
    const wasm = await import('./pkg/gpui_grid_web.js');
    return {
      menu: wasm.preview_open_header_menu(),
      action: wasm.preview_last_header_action(),
    };
  });
  const canvas = page.frameLocator('#gpui-frame').locator('body > canvas');
  // Open a real GPUI header menu, sort from it, and inspect Rust's action.
  await page.goto('/?feature=overview', {waitUntil: 'domcontentloaded'});
  await expect(page.locator('#runtime')).toContainText('GPUI WASM initialized', {timeout: 100_000});
  await expect(canvas).toBeVisible();
  await openCustomerHeaderMenu(canvas, diag);
  // The compact menu puts the Descending button in its first actions row.
  await canvas.click({position: {x: 484, y: 190}});
  await expect.poll(async () => (await diag()).action, {
    timeout: 15_000, message: 'header Descending should mutate the Rust GridModel sort',
  }).toBe('sort:customer:desc');
  await info.attach('header-menu-descending-sort.png', {body: await canvas.screenshot(),contentType:'image/png'});

  // Start from a fresh model. Hide Customer from its column-header popover,
  // verifying the result is 9 *Rust* presentation columns, not DOM elements.
  await page.reload({waitUntil: 'domcontentloaded'});
  await expect(page.locator('#runtime')).toContainText('GPUI WASM initialized', {timeout: 100_000});
  await expect(canvas).toBeVisible();
  await openCustomerHeaderMenu(canvas, diag);
  await canvas.click({position: {x: 405, y: 241}});
  await expect.poll(async () => (await diag()).action, {
    timeout: 15_000, message: 'Hide column from native GPUI menu must update Rust column state',
  }).toBe('column-hide:customer:9');
  await info.attach('header-menu-hide-column.png', {body: await canvas.screenshot(),contentType:'image/png'});
  expect(browserErrors).toEqual([]);
});

test('native GPUI menu moves a pinned column and unpins/reset presentation using Rust state', async ({page}, info) => {
  const inspect = () => page.frameLocator('#gpui-frame').locator('body').evaluate(async () => {
    const wasm = await import('./pkg/gpui_grid_web.js');
    return {menu:wasm.preview_open_header_menu(), action:wasm.preview_last_header_action()};
  });
  await page.goto('/?feature=columns', {waitUntil:'domcontentloaded'});
  await expect(page.locator('#runtime')).toContainText('GPUI WASM initialized', {timeout:100_000});
  const canvas = page.frameLocator('#gpui-frame').locator('body > canvas');
  await expect(canvas).toBeVisible();
  await openCustomerHeaderMenu(canvas, inspect);

  // New command row is directly below existing Hide/Show buttons.
  // This click must enter Rust GPUI's move handler; no JS grid operation.
  await canvas.click({position:{x:411,y:270}});
  await expect.poll(async () => (await inspect()).action, {
    timeout:15_000, message:'Move Left should update Rust pinned column order',
  }).toMatch(/^column-move:customer:left:customer,order_id,/);
  await info.attach('header-native-move-left.png', {body:await canvas.screenshot(),contentType:'image/png'});

  // A fresh model positions Customer at its original header coordinates.
  await page.reload({waitUntil:'domcontentloaded'});
  await expect(page.locator('#runtime')).toContainText('GPUI WASM initialized', {timeout:100_000});
  await openCustomerHeaderMenu(canvas, inspect);
  await canvas.click({position:{x:568,y:270}});
  await expect.poll(async () => (await inspect()).action, {
    timeout:15_000, message:'Unpin should mutate Core ColumnState',
  }).toMatch(/^column-pin:customer:none:order_id,customer,/);
  await info.attach('header-native-unpin.png', {body:await canvas.screenshot(),contentType:'image/png'});

  // Reset is next to Hide/Show and returns the exact default column order.
  await openCustomerHeaderMenu(canvas, inspect);
  await canvas.click({position:{x:585,y:241}});
  await expect.poll(async () => (await inspect()).action, {
    timeout:15_000, message:'Reset should restore schema column state in Rust',
  }).toMatch(/^columns-reset:order_id,customer,/);
  await info.attach('header-native-columns-reset.png', {body:await canvas.screenshot(),contentType:'image/png'});
});
