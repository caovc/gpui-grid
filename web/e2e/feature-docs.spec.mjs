import { test, expect } from '@playwright/test';

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

test('sorting example toolbar is executed in Rust GPUI rather than a JavaScript grid', async ({ page }, info) => {
  const uncaught = [];
  page.on('pageerror', e => uncaught.push(e.stack || e.message));
  try {
    await page.goto('/?feature=sorting', {waitUntil: 'domcontentloaded'});
    await expect(page.locator('#runtime')).toContainText('GPUI WASM initialized', {timeout: 100_000});
    const canvas = page.frameLocator('#gpui-frame').locator('body > canvas');
    await expect(canvas).toBeVisible();
    const snapshot = () => page.frameLocator('#gpui-frame').locator('body').evaluate(async () => {
      const wasm = await import('./pkg/gpui_grid_web.js');
      return {feature:wasm.preview_feature(), sort:wasm.preview_sort_columns(), rows:wasm.preview_visible_rows()};
    });
    await expect.poll(async () => (await snapshot()).feature).toBe('sorting');
    expect((await snapshot()).sort).toBe(0);
    // The visible GPUI toolbar sits directly below the Rust header. Click
    // its "Highest first" button through the real GPU Canvas.
    await canvas.click({position: {x: 285, y: 104}});
    await expect.poll(async () => (await snapshot()).sort, {
      timeout: 15_000, message: 'Rust GPUI sort state must change after Canvas click',
    }).toBe(1);
    await info.attach('real-gpui-sorting-doc-demo.png', {body: await canvas.screenshot(),contentType:'image/png'});
    expect(uncaught).toEqual([]);
  } finally {
    await info.attach('docs-sorting-runtime-errors.json', {
      body:JSON.stringify(uncaught,null,2),contentType:'application/json',
    });
  }
});
