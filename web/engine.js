/** UI-neutral data and viewport utilities for the browser showcase.
 * The real GPUI component uses the sibling Rust core; these functions mirror its semantics.
 */
export const COLUMNS = [
  { id: 'orderId', label: 'Order ID', width: 134, pinned: true, kind: 'id' },
  { id: 'customer', label: 'Customer', width: 205, pinned: true, kind: 'person' },
  { id: 'company', label: 'Company', width: 180, kind: 'text' },
  { id: 'region', label: 'Region', width: 157, kind: 'region' },
  { id: 'product', label: 'Product', width: 192, kind: 'text' },
  { id: 'category', label: 'Category', width: 147, kind: 'text' },
  { id: 'amount', label: 'Revenue', width: 145, kind: 'money' },
  { id: 'growth', label: 'Growth', width: 126, kind: 'growth' },
  { id: 'status', label: 'Status', width: 140, kind: 'status' },
  { id: 'date', label: 'Order Date', width: 145, kind: 'date' },
  { id: 'channel', label: 'Channel', width: 138, kind: 'text' },
  { id: 'owner', label: 'Sales Owner', width: 166, kind: 'person' },
  { id: 'units', label: 'Units', width: 110, kind: 'number' },
  { id: 'score', label: 'Health Score', width: 162, kind: 'score' },
];

export const REGIONS = ['North America', 'Europe', 'APAC', 'Latin America', 'Middle East'];
export const STATUSES = ['Completed', 'Processing', 'Pending', 'Cancelled'];
const NAMES = ['Olivia Martinez', 'Liam Carter', 'Sophia Chen', 'Noah Williams', 'Emma Thompson', 'James Anderson', 'Ava Patel', 'Lucas Brown', 'Isabella Garcia', 'Mason Wilson', 'Charlotte Lee', 'Ethan Davis', 'Amelia Walker', 'Benjamin Moore', 'Mia Robinson', 'Henry Kim', 'Harper Taylor', 'Daniel Young', 'Ella Nguyen', 'Alexander Scott'];
const COMPANIES = ['Linear', 'Vercel', 'Figma', 'Notion', 'Arc', 'Stripe', 'Ramp', 'Mercury', 'Attio', 'Raycast', 'Retool', 'Webflow', 'Sentry', 'Supabase', 'Resend', 'Framer', 'Airbyte', 'PostHog'];
const PRODUCTS = [['Enterprise Platform','Software'], ['Growth Suite','Software'], ['Data Warehouse','Infrastructure'], ['Cloud Storage','Infrastructure'], ['Analytics Pro','Analytics'], ['Team Workspace','Collaboration'], ['API Gateway','Infrastructure'], ['Workflow Engine','Automation'], ['Security Plus','Security'], ['Insights Studio','Analytics']];
const CHANNELS = ['Direct', 'Partner', 'Online', 'Referral'];
const SEED_OFFSET = 37;

/** Fast deterministic integer hash; same row stays stable after resizing the demo. */
function hash(x) { x = (x ^ (x >>> 16)) * 0x45d9f3b; x |= 0; x = (x ^ (x >>> 16)) * 0x45d9f3b; x |= 0; return (x ^ (x >>> 16)) >>> 0; }
export function generateRows(count = 12480) {
  const rows = new Array(count);
  for (let i = 0; i < count; i++) {
    const v = hash(i + SEED_OFFSET);
    const customer = NAMES[v % NAMES.length];
    const product = PRODUCTS[(v >>> 7) % PRODUCTS.length];
    const month = ((v >>> 5) % 9) + 1;
    const day = ((v >>> 11) % 28) + 1;
    const status = STATUSES[(v >>> 17) % STATUSES.length];
    rows[i] = {
      orderId: `ORD-${String(10420 + i).padStart(6, '0')}`,
      customer, company: COMPANIES[(v >>> 3) % COMPANIES.length],
      region: REGIONS[(v >>> 9) % REGIONS.length],
      product: product[0], category: product[1],
      amount: Math.round((840 + (v % 182000) / 11) * 100) / 100,
      growth: Math.round(((v >>> 8) % 600 - 170) / 10) / 10,
      status, date: `2026-${String(month).padStart(2,'0')}-${String(day).padStart(2,'0')}`,
      channel: CHANNELS[(v >>> 2) % CHANNELS.length],
      owner: NAMES[(v >>> 18) % NAMES.length], units: 1 + ((v >>> 4) % 300),
      score: 25 + ((v >>> 6) % 76),
    };
  }
  // Hand-curated visible first rows give the default view a natural, editorial data mix.
  const featured = [
    ['Sophia Chen','Figma','North America','Enterprise Platform','Software',24850,12.8,'Completed','2026-09-18','Direct','Olivia Martinez',24,96],
    ['James Wilson','Vercel','Europe','Cloud Storage','Infrastructure',18420,8.2,'Processing','2026-09-17','Partner','Daniel Young',18,83],
    ['Olivia Martinez','Linear','APAC','Analytics Pro','Analytics',12350,-2.4,'Completed','2026-09-16','Online','Sophia Chen',36,91],
    ['Ethan Davis','Stripe','North America','Growth Suite','Software',32400,24.5,'Pending','2026-09-15','Direct','James Anderson',12,72],
    ['Ava Patel','Notion','Europe','Team Workspace','Collaboration',8900,6.8,'Completed','2026-09-15','Referral','Emma Thompson',43,88],
    ['Noah Williams','Ramp','Latin America','API Gateway','Infrastructure',15780,-4.1,'Cancelled','2026-09-14','Partner','Lucas Brown',14,58],
    ['Emma Thompson','Mercury','North America','Insights Studio','Analytics',21750,18.6,'Completed','2026-09-14','Online','Harper Taylor',27,95],
    ['Lucas Brown','Raycast','APAC','Security Plus','Security',11260,4.3,'Processing','2026-09-13','Direct','Mia Robinson',8,79],
    ['Mia Robinson','Retool','Europe','Workflow Engine','Automation',19540,11.2,'Completed','2026-09-12','Referral','Henry Kim',31,92],
    ['Liam Carter','Supabase','Middle East','Data Warehouse','Infrastructure',14220,-1.8,'Pending','2026-09-11','Partner','Charlotte Lee',19,68],
  ];
  featured.slice(0, count).forEach((r, i) => { const [customer,company,region,product,category,amount,growth,status,date,channel,owner,units,score] = r; rows[i] = {orderId:`ORD-${String(10420+i).padStart(6,'0')}`,customer,company,region,product,category,amount,growth,status,date,channel,owner,units,score}; });
  return rows;
}

export const formatter = {
  money: v => new Intl.NumberFormat('en-US',{style:'currency',currency:'USD',maximumFractionDigits:2}).format(v),
  number: v => Number(v).toLocaleString('en-US'),
  growth: v => `${v > 0 ? '+' : ''}${Number(v).toFixed(1)}%`,
  date: v => new Date(`${v}T12:00:00`).toLocaleDateString('en-US',{month:'short',day:'numeric',year:'numeric'}),
};
export const formatValue = (value, kind='text') => value == null ? '' : (formatter[kind] ? formatter[kind](value) : String(value));

const collator = new Intl.Collator('en', { numeric:true, sensitivity:'base' });
export function compareValue(a,b) {
  if (a == null || a === '') return b == null || b === '' ? 0 : -1;
  if (b == null || b === '') return 1;
  if (typeof a === 'number' && typeof b === 'number') return a-b;
  return collator.compare(String(a),String(b));
}
/** Stable sorting without mutating input; multiple specs have priority left-to-right. */
export function sortRows(rows, specs) {
  if (!specs?.length) return rows;
  return rows.map((row,index)=>({row,index})).sort((a,b)=>{
    for(const spec of specs){const c=compareValue(a.row[spec.id],b.row[spec.id]);if(c) return spec.dir==='desc'?-c:c;}
    return a.index-b.index;
  }).map(x=>x.row);
}
export function filterRows(rows, {query='', status='All statuses', region='All regions', columnFilters={}}={}) {
  const q = query.trim().toLowerCase();
  const checks = Object.entries(columnFilters).filter(([,v])=>String(v).trim());
  if (!q && status==='All statuses' && region==='All regions' && !checks.length) return rows;
  return rows.filter(row=>{
    if(status!=='All statuses' && row.status!==status) return false;
    if(region!=='All regions' && row.region!==region) return false;
    if(checks.some(([k,v])=>!String(row[k]??'').toLowerCase().includes(String(v).toLowerCase())))return false;
    if(q && ![row.orderId,row.customer,row.company,row.region,row.product,row.category,row.status,row.owner,row.channel].some(v=>String(v??'').toLowerCase().includes(q)))return false;
    return true;
  });
}
/** Data rows and group rows with aggregates. */
export function groupRows(rows, field, collapsed=new Set()) {
  if(!field) return rows.map(row=>({type:'data',row}));
  const buckets = new Map();
  for(const row of rows){const key=String(row[field]??'Unknown');if(!buckets.has(key))buckets.set(key,[]);buckets.get(key).push(row);}
  const result=[];
  for(const key of [...buckets.keys()].sort((a,b)=>collator.compare(a,b))){
    const members=buckets.get(key);
    result.push({type:'group',field,key,count:members.length,total:members.reduce((acc,r)=>acc+r.amount,0)});
    if(!collapsed.has(key)) for(const row of members)result.push({type:'data',row,group:key});
  }
  return result;
}
export function visibleRange(scrollTop, viewportHeight, rowHeight, total, overscan=6) {
  const start=Math.max(0,Math.floor(Math.max(0,scrollTop)/rowHeight)-overscan);
  const end=Math.min(total,Math.ceil((Math.max(0,scrollTop)+viewportHeight)/rowHeight)+overscan);
  return {start,end};
}
export function escapeHtml(value) { return String(value??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c])); }
export function escapeCsv(value) {
  let t=String(value??'');
  // Avoid spreadsheet formula injection in exported text.
  if (/^[\s]*[=+@\-\t\r]/.test(t) && typeof value !== 'number') t="'"+t;
  return /[",\n\r]/.test(t) ? '"'+t.replaceAll('"','""')+'"' : t;
}
export function rowsToCsv(rows, columns){return [columns.map(c=>escapeCsv(c.label)).join(','), ...rows.map(r=>columns.map(c=>escapeCsv(r[c.id])).join(','))].join('\r\n');}
export function parseDelimited(text, delimiter=','){
  // RFC 4180-ish, including quoted multiline values and CRLF.
  const out=[];let row=[], field='', quoted=false;
  for(let i=0;i<text.length;i++){
    const c=text[i];
    if(quoted){if(c==='"' && text[i+1]==='"'){field+='"';i++;}else if(c==='"'){quoted=false;}else{field+=c;}continue;}
    if(c==='"' && !field){quoted=true;continue;}
    if(c===delimiter){row.push(field);field='';continue;}
    if(c==='\n' || c==='\r'){if(c==='\r'&&text[i+1]==='\n')i++;row.push(field);if(row.some(x=>x!==''))out.push(row);row=[];field='';continue;}
    field+=c;
  }
  row.push(field);if(row.some(x=>x!==''))out.push(row);
  return out;
}
export function validateEdit(column,raw){
  if(!column || column.id==='orderId')return {ok:false,error:'This column is read-only'};
  if(['amount','growth','units','score'].includes(column.id)){
    const numeric=Number(String(raw).replace(/[,$%]/g,'').trim());
    if(!Number.isFinite(numeric) || String(raw).trim()==='')return {ok:false,error:'Enter a valid number'};
    if(column.id==='score'&&(numeric<0||numeric>100))return {ok:false,error:'Score must be between 0 and 100'};
    if((column.id==='amount'||column.id==='units')&&numeric<0)return {ok:false,error:'Value cannot be negative'};
    return {ok:true,value:column.id==='units'?Math.round(numeric):numeric};
  }
  if(column.id==='status' && !STATUSES.includes(raw)) return {ok:false,error:'Choose a valid status'};
  if(column.id==='region' && !REGIONS.includes(raw)) return {ok:false,error:'Choose a valid region'};
  if(column.id==='date' && !/^\d{4}-\d{2}-\d{2}$/.test(raw))return {ok:false,error:'Use YYYY-MM-DD'};
  if(!String(raw).trim())return {ok:false,error:'Value cannot be empty'};
  return {ok:true,value:String(raw).trim()};
}
export function summaryOf(rows){return {total:rows.length,revenue:rows.reduce((v,r)=>v+Number(r.amount||0),0),completed:rows.filter(r=>r.status==='Completed').length,averageScore:rows.length?rows.reduce((v,r)=>v+Number(r.score||0),0)/rows.length:0};}
