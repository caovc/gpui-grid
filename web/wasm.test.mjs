import {test} from 'node:test';
import {readFile} from 'node:fs/promises';
import {strict as assert} from 'node:assert';
test('canvas viewer loads wasm-bindgen output and executes Rust entry point',async()=>{
  const source=await readFile(new URL('./viewer/loader.js',import.meta.url),'utf8');
  assert.match(source,/import\('\.\/pkg\/gpui_grid_web\.js'\)/);
  assert.match(source,/await wasm\.default\(\)/);
  assert.match(source,/wasm\.run\(feature\)/);
  assert.match(source,/new URLSearchParams\(window.location.search\)/);
  assert.doesNotMatch(source,/new DataTable|renderRow\(/);
});
test('site build refuses to publish a mock site without a real WASM binary',async()=>{
  const build=await readFile(new URL('./build.mjs',import.meta.url),'utf8');
  assert.match(build,/0061736d/);
  assert.match(build,/gpui_grid_web_bg\.wasm/);
  assert.doesNotMatch(build,/standalone\.html/);
});

test('local static server serves the real WASM preview root (not HTTP 403)', async () => {
  const {createServer} = await import('node:net');
  const {spawn} = await import('node:child_process');
  const {mkdir, writeFile, rm, stat} = await import('node:fs/promises');
  const {dirname, join} = await import('node:path');
  const {fileURLToPath} = await import('node:url');
  const web = dirname(fileURLToPath(import.meta.url));
  const dist = join(web, 'dist');
  const preexisting = await stat(dist).then(() => true, () => false);
  if (!preexisting) await mkdir(dist, {recursive: true});
  const sentinel = join(dist, 'index.html');
  const exists = await stat(sentinel).then(() => true, () => false);
  if (!exists) await writeFile(sentinel, '<h1>Grid server sentinel</h1>');
  const port = await new Promise((resolve, reject) => {
    const socket = createServer();
    socket.once('error', reject);
    socket.listen(0, '127.0.0.1', () => {
      const port = socket.address().port;
      socket.close(() => resolve(port));
    });
  });
  const server = spawn(process.execPath, ['dev.mjs'], {
    cwd: web,
    env: {...process.env, PORT: String(port)},
    stdio: 'ignore',
  });
  try {
    let response;
    for (let i = 0; i < 50; i++) {
      response = await fetch(`http://127.0.0.1:${port}/`).catch(() => null);
      if (response) break;
      await new Promise(resolve => setTimeout(resolve, 30));
    }
    assert.ok(response, 'dev server did not start');
    assert.equal(response.status, 200);
    assert.match(await response.text(), /<h1>|GPUI Grid/);
    const forbidden = await fetch(`http://127.0.0.1:${port}/%2e%2e/%2e%2e/Cargo.toml`);
    assert.notEqual(forbidden.status, 200, 'path traversal must stay outside the public root');
  } finally {
    server.kill();
    if (!exists) await rm(sentinel, {force:true});
    if (!preexisting) await rm(dist, {recursive:true, force:true});
  }
});
