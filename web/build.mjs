import { mkdir, copyFile, readFile, writeFile, rm } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
const root=dirname(fileURLToPath(import.meta.url));
const dest=join(root,'dist');
await rm(dest,{recursive:true,force:true});
await mkdir(dest,{recursive:true});
for(const file of ['index.html','styles.css','engine.js','main.js'])await copyFile(join(root,file),join(dest,file));
// A completely self-contained preview for opening without a server or npm.
const html=await readFile(join(root,'index.html'),'utf8');
const css=await readFile(join(root,'styles.css'),'utf8');
const engine=await readFile(join(root,'engine.js'),'utf8');
const main=await readFile(join(root,'main.js'),'utf8');
const bundle=engine.replace(/^export\s+/gm,'')+'\nconst esc = escapeHtml;\n'+main.replace(/^import\s*\{[\s\S]*?\}\s*from\s*['"]\.\/engine\.js['"];?\s*/m,'');
const standalone=html
  .replace('<link rel="stylesheet" href="./styles.css" />',()=>`<style>\n${css}\n</style>`)
  .replace('<script type="module" src="./main.js"></script>',()=>`<script type="module">\n${bundle}\n</script>`);
await writeFile(join(dest,'standalone.html'),standalone);
console.log(`Built ${dest} (4 Pages assets and a standalone one-file preview)`);
