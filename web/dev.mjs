import {createServer} from 'node:http';
import {readFile} from 'node:fs/promises';
import {dirname,join,normalize} from 'node:path';
import {fileURLToPath} from 'node:url';
const dir=dirname(fileURLToPath(import.meta.url));
const mime={'.html':'text/html; charset=utf-8','.js':'text/javascript; charset=utf-8','.css':'text/css; charset=utf-8'};
const port=Number(process.env.PORT||4173);
createServer(async(req,res)=>{
  const path=decodeURIComponent((req.url||'/').split('?')[0]);
  const target=normalize(join(dir,path==='/'?'index.html':path));
  if(!target.startsWith(dir)){res.writeHead(403).end();return;}
  try{const bytes=await readFile(target);const ext=target.slice(target.lastIndexOf('.'));res.writeHead(200,{'Content-Type':mime[ext]||'application/octet-stream'});res.end(bytes);}catch{res.writeHead(404).end('Not found');}
}).listen(port,'127.0.0.1',()=>console.log(`Preview: http://127.0.0.1:${port}`));
