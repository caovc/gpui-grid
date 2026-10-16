import {createServer} from 'node:http';
import {readFile} from 'node:fs/promises';
import {resolve,join,sep,extname} from 'node:path';
import {fileURLToPath} from 'node:url';
const dir=fileURLToPath(new URL('./dist/',import.meta.url));
const mime={'.html':'text/html; charset=utf-8','.js':'text/javascript; charset=utf-8','.css':'text/css; charset=utf-8','.wasm':'application/wasm'};
const port=Number(process.env.PORT||4173);
createServer(async(req,res)=>{
  let name;
  try{name=decodeURIComponent(new URL(req.url||'/',`http://localhost:${port}`).pathname);}catch{res.writeHead(400).end();return;}
  const target=resolve(dir,'.'+name,(name.endsWith('/')?'index.html':''));
  if(target!==dir&&!target.startsWith(dir+sep)){res.writeHead(403).end();return;}
  try{const bytes=await readFile(target);res.writeHead(200,{'Content-Type':mime[extname(target)]||'application/octet-stream','Cache-Control':'no-store'});res.end(bytes);}catch{res.writeHead(404).end('Not found');}
}).listen(port,'127.0.0.1',()=>console.log(`GPUI WASM preview: http://127.0.0.1:${port}`));
