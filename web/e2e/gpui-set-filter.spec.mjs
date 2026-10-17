import {test, expect} from '@playwright/test';
import {openCustomerHeaderMenu} from './column-menu.mjs';

// All input is pointer/keyboard data sent into *real Rust GPUI Canvas*.
// wasm-bindgen exports below are read-only; there is no JavaScript GridModel.
test('Set Filter renders native GPUI checkboxes and changes the Rust data view', async ({page}, info) => {
  const errors = [];
  page.on('pageerror', e => errors.push(e.stack || e.message));
  const inspect = () => page.frameLocator('#gpui-frame').locator('body').evaluate(async () => {
    const wasm = await import('./pkg/gpui_grid_web.js');
    return {
      menu: wasm.preview_open_header_menu(),
      last: wasm.preview_last_header_action(),
      rows: wasm.preview_visible_rows(),
    };
  });
  await page.goto('/?feature=filtering', {waitUntil: 'domcontentloaded'});
  await expect(page.locator('#runtime')).toContainText('GPUI WASM initialized', {timeout: 100_000});
  const canvas = page.frameLocator('#gpui-frame').locator('body > canvas');
  await expect(canvas).toBeVisible();
  expect((await inspect()).rows).toBe(12_480);
  await openCustomerHeaderMenu(canvas, inspect);
  await info.attach('quartz-set-filter-open.png', {body: await canvas.screenshot(),contentType:'image/png'});

  // A 690px GPUI Canvas starts lower on the documentation page. The popup
  // and checkboxes are correctly painted inside the frame, but the browser
  // viewport must scroll so its lower Canvas pixels can receive pointer hits.
  // Scroll only the outer docs page; leave native GPUI state untouched.
  await page.evaluate(() => {
    const iframe = document.querySelector('#gpui-frame');
    window.scrollTo({
      top: iframe.getBoundingClientRect().top + window.scrollY - 105,
      behavior: 'instant',
    });
  });
  await expect.poll(async () => (await inspect()).menu).toBe('customer');
  const points = [[425, 460], [425, 485], [425, 505], [425, 530], [425, 555]];
  for (const [x, y] of points) {
    await canvas.click({position:{x,y}, timeout: 10_000});
    const {last} = await inspect();
    if (/^set(?:-all)?:customer:/.test(last)) break;
  }
  await expect.poll(async () => (await inspect()).last, {
    timeout: 15_000,
    message:'actual GPUI checkbox should mutate typed Rust OneOf selection',
  }).toMatch(/^set(?:-all)?:customer:/);

  const action = (await inspect()).last;
  const newCount = Number(action.split(':').at(-1));
  expect(newCount, 'checkbox should change the Rust filtered row count').toBeLessThan(12_480);
  await info.attach('quartz-set-filter-selected.png', {body: await canvas.screenshot(),contentType:'image/png'});
  expect(errors).toEqual([]);
});
