import { test } from 'node:test';
import { strict as assert } from 'node:assert';
import { readFile } from 'node:fs/promises';
import { FEATURES, FEATURE_GROUPS, LIVE_FEATURES, byFeatureId } from './catalog.js';

test('docs feature catalog has unique routable IDs and truthfully separates live/core/planned', () => {
  const ids = FEATURES.map(f => f.id);
  assert.equal(new Set(ids).size, ids.length);
  assert.ok(LIVE_FEATURES.length >= 7);
  for (const f of FEATURES) {
    assert.ok(FEATURE_GROUPS.some(g => g.id === f.group), f.id + ' group exists');
    assert.ok(['live','core','planned'].includes(f.status), f.id + ' status');
    assert.ok(f.title && f.summary && f.code && f.api.length && f.steps.length, f.id + ' docs');
    if (f.status === 'live') {
      assert.ok(f.hints?.length >= 3, f.id + ' has real actions to demonstrate');
    }
  }
  assert.equal(byFeatureId('not-a-feature').id, 'overview');
  assert.equal(byFeatureId('selection').status, 'live');
  assert.equal(byFeatureId('grouping').status, 'core');
  assert.equal(byFeatureId('clipboard').status, 'planned');
});

test('docs shell navigates to per-feature real WASM and does not implement a JS grid', async () => {
  const shell = await readFile(new URL('./shell.js', import.meta.url), 'utf8');
  const page = await readFile(new URL('./index.html', import.meta.url), 'utf8');
  const viewer = await readFile(new URL('./viewer/loader.js', import.meta.url), 'utf8');
  const rust = await readFile(new URL('../crates/gpui-grid/src/showcase.rs', import.meta.url), 'utf8');
  const bootstrap = await readFile(new URL('../crates/gpui-grid-web/src/lib.rs', import.meta.url), 'utf8');
  const build = await readFile(new URL('./build.mjs', import.meta.url), 'utf8');
  assert.match(page, /id="feature-nav"/);
  assert.match(page, /id="feature-search"/);
  assert.match(page, /id="gpui-frame"/);
  assert.match(page, /id="rust-example-code"/);
  assert.match(shell, /url\.searchParams\.set\('feature', feature\.id\)/);
  assert.match(shell, /frame\.src = next\.href/);
  assert.match(shell, /feature\.status === 'live'/);
  assert.match(shell, /frame\.src = 'about:blank'/);
  assert.match(viewer, /wasm\.run\(feature\)/);
  assert.match(rust, /pub enum GridFeature/);
  assert.match(rust, /pub fn new_with_feature/);
  const grid = await readFile(new URL('../crates/gpui-grid/src/lib.rs', import.meta.url), 'utf8');
  const controls = await readFile(new URL('../crates/gpui-grid/src/header_controls.rs', import.meta.url), 'utf8');
  assert.match(grid, /fn render_th\(/, 'column header must render inside TableDelegate');
  assert.match(controls, /Popover::new\(pop_id\)/, 'native GPUI header Popover');
  assert.match(controls, /Input::new\(&spec\.input\)/, 'native GPUI InputState');
  assert.match(controls, /Clear this filter/);
  assert.match(controls, /Show all columns/);
  assert.doesNotMatch(shell, /sortRows|filterRows|renderCell/);
  assert.match(bootstrap, /pub fn run\(feature: Option<String>\)/);
  assert.match(build, /'catalog\.js'/);
  // JS is documentation only; never render a pseudo-grid row/cell.
  assert.doesNotMatch(shell, /document\.createElement\(['"](?:tr|td|table)['"]\)/);
  assert.doesNotMatch(shell, /render(?:Rows?|Cells?|Grid)|sortRows|new GridModel/);
});
