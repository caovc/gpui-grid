import {mkdir, copyFile, readFile, stat, rm} from 'node:fs/promises';
import {dirname, join} from 'node:path';
import {fileURLToPath} from 'node:url';
const root = dirname(fileURLToPath(import.meta.url));
const dist = join(root,'dist');
const pkg = join(root,'pkg');
const js = join(pkg,'gpui_grid_web.js');
const wasm = join(pkg,'gpui_grid_web_bg.wasm');
try {
  const magic = (await readFile(wasm)).subarray(0,4);
  if (magic.toString('hex') !== '0061736d') throw Error('Invalid WebAssembly binary magic');
  if ((await stat(wasm)).size < 1024 || !(await readFile(js,'utf8')).includes('export')) throw Error('WASM bindings appear incomplete');
} catch (error) {
  console.error('No valid GPUI WASM bundle found. Run ./scripts/build-wasm.sh --release first.');
  throw error;
}
await rm(dist,{force:true,recursive:true});
await mkdir(join(dist,'viewer','pkg'),{recursive:true});
for (const file of ['index.html','styles.css','shell.js']) await copyFile(join(root,file),join(dist,file));
for (const file of ['index.html','loader.js']) await copyFile(join(root,'viewer',file),join(dist,'viewer',file));
await copyFile(js,join(dist,'viewer','pkg','gpui_grid_web.js'));
await copyFile(wasm,join(dist,'viewer','pkg','gpui_grid_web_bg.wasm'));
console.log(`Built actual Rust/GPUI WebAssembly site: ${dist}`);
