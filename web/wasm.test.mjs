import {test} from 'node:test';
import {readFile} from 'node:fs/promises';
import {strict as assert} from 'node:assert';
test('canvas viewer loads wasm-bindgen output and executes Rust entry point',async()=>{
  const source=await readFile(new URL('./viewer/loader.js',import.meta.url),'utf8');
  assert.match(source,/import\('\.\/pkg\/gpui_grid_web\.js'\)/);
  assert.match(source,/await wasm\.default\(\)/);
  assert.match(source,/wasm\.run\(\)/);
  assert.doesNotMatch(source,/new DataTable|renderRow\(/);
});
test('site build refuses to publish a mock site without a real WASM binary',async()=>{
  const build=await readFile(new URL('./build.mjs',import.meta.url),'utf8');
  assert.match(build,/0061736d/);
  assert.match(build,/gpui_grid_web_bg\.wasm/);
  assert.doesNotMatch(build,/standalone\.html/);
});
