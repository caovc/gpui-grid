import {expect} from '@playwright/test';

/**
 * Click the real GPUI column-header Popover in its Canvas.
 * Wait for Rust's read-only open state after EACH click. Clicking multiple
 * candidate points without waiting can open and immediately dismiss the
 * popover or inadvertently toggle GPUI Kit's header sort.
 *
 * No JavaScript GridModel manipulation, no DOM mock table.
 */
export async function openCustomerHeaderMenu(canvas, snapshot) {
  if ((await snapshot()).menu === 'customer') return;
  const points = [[354, 125], [359, 125], [348, 125], [354, 120], [354, 130]];
  for (const [x, y] of points) {
    await canvas.click({position: {x, y}});
    try {
      await expect.poll(async () => (await snapshot()).menu, {
        timeout: 1400,
        intervals: [100, 200, 350],
      }).toBe('customer');
      return;
    } catch {
      // Try another point only AFTER an actual wait for Rust Popover state.
    }
  }
  await expect.poll(async () => (await snapshot()).menu, {
    timeout: 10_000,
    message: 'Real Rust GPUI Customer header menu must open from Canvas click',
  }).toBe('customer');
}
