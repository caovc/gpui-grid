import { test, expect } from '@playwright/test';
import {openCustomerHeaderMenu} from './column-menu.mjs';

const getRustFeature = page => page.frameLocator('#gpui-frame').locator('body')
  .evaluate(async () => (await import('./pkg/gpui_grid_web.js')).preview_feature());

test('feature docs navigate with real Rust WASM example, Rust code and planned truthfulness', async ({ page }, info) => {
  const uncaught = [];
  page.on('pageerror', err => uncaught.push(err.stack || err.message));
  try {
    await page.goto('/?feature=selection', { waitUntil: 'domcontentloaded' });
    await expect(page.locator('#feature-title')).toContainText('单元格选区');
    await expect(page.locator('#runtime')).toContainText('GPUI WASM initialized', {timeout: 100_000});
    await expect.poll(() => getRustFeature(page), {timeout: 15_000}).toBe('selection');

    // The JS shell manages only docs and iframe routing, never Rust row state.
    await page.locator('#tab-code').click();
    await expect(page.locator('#rust-example-code')).toContainText('CellAddress::new');
    await expect(page.locator('#code-panel')).toBeVisible();
    await expect(page.locator('#rust-example-code .rust-keyword').first()).toBeVisible();
    await expect(page.locator('#rust-example-code .rust-type').first()).toBeVisible();
    await expect(page.locator('#rust-example-code')).toHaveAttribute('data-syntax', 'rust');
    await page.locator('#tab-preview').click();
    await expect(page.locator('#gpui-frame')).toBeVisible();

    await page.locator('#feature-search').fill('排序');
    const sorting = page.locator('a.nav-item[data-feature="sorting"]');
    await expect(sorting).toBeVisible();
    await sorting.click();
    await expect(page).toHaveURL(/feature=sorting/);
    await expect(page.locator('#feature-title')).toContainText('排序');
    await expect(page.locator('#runtime')).toContainText('GPUI WASM initialized', {timeout: 100_000});
    await expect.poll(() => getRustFeature(page), {timeout: 15_000}).toBe('sorting');
    await expect(page.frameLocator('#gpui-frame').locator('body > canvas')).toBeVisible();

    await page.locator('#feature-search').fill('');
    await page.locator('a.nav-item[data-feature="grouping"]').click();
    await expect(page).toHaveURL(/feature=grouping/);
    await expect(page.locator('#feature-status')).toContainText('CORE ONLY');
    await expect(page.locator('#planned-example')).toBeVisible();
    await expect(page.locator('#playground-card')).toBeHidden();
    await info.attach('grouping-core-only-docs.png', {body: await page.screenshot(), contentType: 'image/png'});
    expect(uncaught, 'no unhandled JS/GPUI errors').toEqual([]);
  } finally {
    await info.attach('feature-docs-errors.json', {
      body: JSON.stringify(uncaught, null, 2), contentType: 'application/json',
    });
  }
});

test('sorting example uses the native GPUI column header instead of showcase toolbar', async ({ page }, info) => {
  const uncaught = [];
  page.on('pageerror', e => uncaught.push(e.stack || e.message));
  try {
    await page.goto('/?feature=sorting', {waitUntil: 'domcontentloaded'});
    await expect(page.locator('#runtime')).toContainText('GPUI WASM initialized', {timeout: 100_000});
    const canvas = page.frameLocator('#gpui-frame').locator('body > canvas');
    await expect(canvas).toBeVisible();
    const snapshot = () => page.frameLocator('#gpui-frame').locator('body').evaluate(async () => {
      const wasm = await import('./pkg/gpui_grid_web.js');
      return {
        feature: wasm.preview_feature(),
        menu: wasm.preview_open_header_menu(),
        action: wasm.preview_last_header_action(),
      };
    });
    await expect.poll(async () => (await snapshot()).feature).toBe('sorting');
    await openCustomerHeaderMenu(canvas, snapshot);
    // Descending is a real button inside the GPUI column Popover.
    await canvas.click({position: {x: 484, y: 190}});
    await expect.poll(async () => (await snapshot()).action, {
      timeout: 15_000, message: 'GPUI header Descending must update Rust sort state',
    }).toBe('sort:customer:desc');
    await info.attach('real-gpui-header-sorting-doc-demo.png', {
      body: await canvas.screenshot(), contentType: 'image/png',
    });
    expect(uncaught).toEqual([]);
  } finally {
    await info.attach('docs-sorting-runtime-errors.json', {
      body:JSON.stringify(uncaught,null,2),contentType:'application/json',
    });
  }
});


test('documentation and real GPUI viewport fill desktop width/height', async ({page}, info) => {
  await page.setViewportSize({width: 1920, height: 1000});
  await page.goto('/?feature=sorting', {waitUntil: 'domcontentloaded'});
  await expect(page.locator('#feature-title')).toContainText('排序');
  const dimensions = await page.evaluate(() => {
    const main = document.querySelector('.doc-main').getBoundingClientRect();
    const viewport = document.querySelector('.frame-wrap').getBoundingClientRect();
    return {
      mainWidth: main.width, canvasWidth: viewport.width, canvasHeight: viewport.height,
      horizontalOverflow: document.documentElement.scrollWidth - innerWidth,
    };
  });
  expect(dimensions.mainWidth).toBeGreaterThan(1250);
  expect(dimensions.canvasWidth).toBeGreaterThan(1200);
  expect(dimensions.canvasHeight).toBeGreaterThan(690);
  expect(dimensions.horizontalOverflow).toBeLessThanOrEqual(2);
  await info.attach('docs-wide-layout.png', {
    body: await page.screenshot(), contentType: 'image/png',
  });
  await page.locator('#tab-code').click();
  await expect(page.locator('#rust-example-code .rust-comment').first()).toBeVisible();
  await expect(page.locator('#rust-example-code .rust-keyword').first()).toBeVisible();
  await expect(page.locator('#rust-example-code .rust-type').first()).toBeVisible();
  await expect(page.locator('#rust-example-code .rust-string').first()).toBeVisible();
  await info.attach('docs-highlighted-rust.png', {
    body: await page.screenshot(), contentType: 'image/png',
  });
});
