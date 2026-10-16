import {
  COLUMNS, REGIONS, STATUSES, generateRows, filterRows, sortRows, groupRows,
  visibleRange, escapeHtml as esc, formatter, formatValue, rowsToCsv,
  parseDelimited, validateEdit, summaryOf
} from './engine.js';

const $=selector=>document.querySelector(selector);
const gridScroll=$('#grid-scroll');
const gridHeader=$('#grid-header');
const gridBody=$('#grid-body');
const gridRows=$('#grid-rows');
const floatMenu=$('#floating-menu');
const CHECK_WIDTH=46;
const formatterShort=new Intl.NumberFormat('en-US',{notation:'compact',maximumFractionDigits:2});
const state={
  rows:generateRows(12480),columns:COLUMNS.map(c=>({...c,hidden:false})),
  query:'',statusFilter:'All statuses',regionFilter:'All regions',columnFilters:{},
  sort:[],groupField:'',collapsed:new Set(),displayRows:[],flatRows:[],
  selectedRows:new Set(),anchor:null,activeCell:null,activeTab:'all',
  rowHeight:40,density:'comfortable',preset:'commerce',view:'orders',
  menu:null,lastRange:{start:-1,end:-1},isResizing:false,toastTimer:null,
};
const icon=(name)=>`<svg aria-hidden="true"><use href="#i-${name}"/></svg>`;
const initialPageText={orders:['Data Grid','Data Grid <span>Playground</span>','A powerful, pixel-precise grid experience. Built for dense data, delightful to use.'],performance:['Performance Lab','Performance <span>Lab</span>','Explore 100,000 rows through a virtualized viewport without mounting the entire dataset.'],themes:['Theme Studio','Theme <span>Studio</span>','Switch between Quartz-inspired light and dark surfaces while keeping full grid functionality.']};

function visibleColumns(){return state.columns.filter(c=>!c.hidden).sort((a,b)=>Number(!!b.pinned)-Number(!!a.pinned));}
function getLayout(){
  const columns=visibleColumns();
  const width=CHECK_WIDTH+columns.reduce((n,c)=>n+c.width,0);
  const offsets=new Map(); let pinnedOffset=CHECK_WIDTH;
  const lastPinned=columns.filter(c=>c.pinned).at(-1)?.id;
  for(const col of columns){if(col.pinned){offsets.set(col.id,pinnedOffset);pinnedOffset+=col.width;}}
  return {columns,width,offsets,lastPinned};
}
function cellClass(col,layout){return `${col.pinned?' pinned':''}${col.id===layout.lastPinned?' pinned-last':''}`;}
function pinnedStyle(col,layout){return col.pinned?`left:${layout.offsets.get(col.id)}px;`:'';}
function totalHeaderHtml(layout){
  const rows=state.flatRows;
  let checked=0;
  // For large datasets, a full pass just on header updates is avoidable.
  if(state.selectedRows.size)for(const row of rows)if(state.selectedRows.has(row.orderId))checked++;
  const all=rows.length>0&&checked===rows.length;
  const partial=checked>0&&!all;
  return `<div class="header-cell checkbox-cell pinned" style="width:${CHECK_WIDTH}px;left:0;" role="columnheader"><input id="header-check" type="checkbox" aria-label="Select all filtered rows" ${all?'checked':''} data-partial="${partial}" /></div>`;
}
function renderHeader(){
  const layout=getLayout();
  gridHeader.style.width=`${layout.width}px`;
  gridHeader.innerHTML=totalHeaderHtml(layout)+layout.columns.map((col,index)=>{
    const sorted=state.sort.findIndex(spec=>spec.id===col.id);
    const sort=sorted>=0?`<span class="header-sort">${state.sort[sorted].dir==='asc'?'↑':'↓'}${state.sort.length>1?`<small>${sorted+1}</small>`:''}</span>`:'';
    return `<div role="columnheader" class="header-cell${cellClass(col,layout)}${sorted>=0?' sorting':''}" data-colid="${col.id}" data-colindex="${index}" draggable="true" style="width:${col.width}px;${pinnedStyle(col,layout)}" title="Click to sort; Shift-click for multi-sort; drag to reorder">
      <span class="header-text">${esc(col.label)}</span>${sort}<button class="header-more" data-colmenu="${col.id}" aria-label="${esc(col.label)} column menu" title="Column menu">${icon('more')}</button><span class="resize-handle" data-resize="${col.id}" title="Drag to resize"></span></div>`;
  }).join('');
  gridBody.style.width=`${layout.width}px`;
  const headCheck=$('#header-check');
  if(headCheck)headCheck.indeterminate=headCheck.dataset.partial==='true';
}
function personAvatar(name){
  const initials=String(name).split(' ').slice(0,2).map(w=>w[0]).join('').toUpperCase();
  const palette=['purple','blue','orange','green','pink'];
  const color=palette[String(name).charCodeAt(0)%palette.length];
  return `<span class="person-avatar avatar-${color}">${esc(initials)}</span>`;
}
function displayCell(row,col){
  const value=row[col.id];
  switch(col.kind){
    case 'id':return `<span class="id-text">${esc(value)}</span>`;
    case 'person':return `${personAvatar(value)}<span class="cell-text">${esc(value)}</span>`;
    case 'region':{
      const regionClass={'Europe':'europe','APAC':'apac','Latin America':'latin','Middle East':'middle'}[value]||'north';
      return `<span class="region-cell"><span class="region-dot region-${regionClass}"></span><span class="cell-text">${esc(value)}</span></span>`;
    }
    case 'status':return `<span class="status-badge status-${String(value).toLowerCase()}">${esc(value)}</span>`;
    case 'money':return `<span class="cell-text">${esc(formatter.money(value))}</span>`;
    case 'number':return `<span class="cell-text">${esc(formatter.number(value))}</span>`;
    case 'growth':return `<span class="${value>=0?'growth-positive':'growth-negative'}">${value>=0?'↗':'↘'} ${esc(formatter.growth(value))}</span>`;
    case 'score':return `<span class="score-box"><span class="score-track"><span class="score-fill ${value>=85?'score-high':value<=55?'score-low':''}" style="display:block;width:${Math.max(0,Math.min(100,value))}%"></span></span><span>${esc(value)}</span></span>`;
    case 'date':return `<span class="cell-text">${esc(formatter.date(value))}</span>`;
    default:return `<span class="cell-text">${esc(value)}</span>`;
  }
}
function isWithinRange(index,colIndex){
  if(!state.anchor||!state.activeCell)return false;
  const a=state.anchor,b=state.activeCell;
  return index>=Math.min(a.index,b.index)&&index<=Math.max(a.index,b.index)&&colIndex>=Math.min(a.colIndex,b.colIndex)&&colIndex<=Math.max(a.colIndex,b.colIndex);
}
function renderOneRow(item,index,layout){
  const rowHeight=state.rowHeight,top=index*rowHeight;
  if(item.type==='group')return `<div class="grid-row group-row" data-group="${esc(item.key)}" role="row" style="top:${top}px;height:${rowHeight}px;width:${layout.width}px">
    <span class="group-caret">${state.collapsed.has(item.key)?'+':'−'}</span><span>${esc(item.key)}</span><span class="group-count">${formatter.number(item.count)} records</span><span class="group-indicator">${esc(item.field)}</span><span class="group-revenue">Σ ${formatter.money(item.total)}</span></div>`;
  const row=item.row;const rowSelected=state.selectedRows.has(row.orderId);
  const cells=layout.columns.map((col,colIndex)=>{
    const selected=state.activeCell?.index===index&&state.activeCell.colIndex===colIndex;
    const ranged=isWithinRange(index,colIndex);
    const numerical=['money','number','growth','score'].includes(col.kind);
    return `<div role="gridcell" class="cell${cellClass(col,layout)}${selected?' selected-cell':''}${ranged&&!selected?' range-cell':''}${numerical?' num-cell':''}${col.kind==='person'?' person-cell':''}${col.id==='orderId'?' readonly':''}" style="width:${col.width}px;${pinnedStyle(col,layout)}" data-rowindex="${index}" data-colindex="${colIndex}" data-colid="${col.id}" data-rowid="${row.orderId}" title="${esc(formatValue(row[col.id],col.kind))}">${displayCell(row,col)}</div>`;
  }).join('');
  return `<div role="row" class="grid-row${index%2?' striped':''}${rowSelected?' selected-row':''}" data-rowid="${row.orderId}" style="top:${top}px;width:${layout.width}px;height:${rowHeight}px"><div class="cell checkbox-cell pinned" role="gridcell" style="width:${CHECK_WIDTH}px;left:0;"><input type="checkbox" data-rowcheck="${row.orderId}" aria-label="Select ${esc(row.orderId)}" ${rowSelected?'checked':''}></div>${cells}</div>`;
}
function renderVisible(force=false){
  if(state.editing&&!force)return;
  const bodyTop=gridHeader.offsetHeight||42;
  const scrollTop=Math.max(0,gridScroll.scrollTop-bodyTop);
  const viewport=gridScroll.clientHeight||414;
  const range=visibleRange(scrollTop,viewport,state.rowHeight,state.displayRows.length,5);
  if(!force&&range.start===state.lastRange.start&&range.end===state.lastRange.end)return;
  state.lastRange=range;
  const layout=getLayout();
  const parts=[];
  for(let i=range.start;i<range.end;i++)parts.push(renderOneRow(state.displayRows[i],i,layout));
  gridRows.innerHTML=parts.join('');
  $('#rendered-count').textContent=`${range.end-range.start} rendered`;
}
function resetSelections(){state.activeCell=null;state.anchor=null;state.selectedRows.clear();}
function rowsForTab(rows){
  if(state.activeTab==='completed')return rows.filter(r=>r.status==='Completed');
  if(state.activeTab==='processing')return rows.filter(r=>r.status==='Processing'||r.status==='Pending');
  if(state.activeTab==='flagged')return rows.filter(r=>r.status==='Cancelled'||r.score<55);
  return rows;
}
function rebuild(resetScroll=true){
  const records=filterRows(state.rows,{query:state.query,status:state.statusFilter,region:state.regionFilter,columnFilters:state.columnFilters});
  state.flatRows=sortRows(rowsForTab(records),state.sort);
  state.displayRows=groupRows(state.flatRows,state.groupField,state.collapsed);
  if(resetScroll){gridScroll.scrollTop=0;resetSelections();}
  const stats=summaryOf(state.flatRows);
  $('#metric-rows').textContent=formatter.number(stats.total);
  $('#metric-revenue').textContent=formatterShort.format(stats.revenue).replace(/^/, '$');
  $('#metric-completed').textContent=formatter.number(stats.completed);
  $('#metric-completion').textContent=`${stats.total?Math.round(stats.completed/stats.total*100):0}% completion rate`;
  $('#metric-score').textContent=stats.averageScore.toFixed(1);
  $('.metric-tiny-bar:not(.orange-bar) i').style.width=`${stats.total?stats.completed/stats.total*100:0}%`;
  $('.metric-tiny-bar.orange-bar i').style.width=`${stats.averageScore}%`;
  $('#tab-count').textContent=formatterShort.format(state.rows.length);
  $('#footer-rows').textContent=`${formatter.number(state.flatRows.length)} of ${formatter.number(state.rows.length)} rows`;
  $('#footer-selected').textContent=`${formatter.number(state.selectedRows.size)} selected`;
  $('#footer-active').textContent='';
  $('#grid-scroll').setAttribute('aria-rowcount',String(state.displayRows.length));
  $('#empty-state').hidden=state.displayRows.length>0;
  gridBody.style.height=`${Math.max(0,state.displayRows.length*state.rowHeight)}px`;
  state.lastRange={start:-1,end:-1};
  renderHeader();renderVisible(true);
}
function updateRowsWithoutRefilter(){
  state.lastRange={start:-1,end:-1};renderHeader();renderVisible(true);
  $('#footer-selected').textContent=`${formatter.number(state.selectedRows.size)} selected`;
  $('#footer-active').textContent=state.activeCell?`${state.activeCell.colId} · row ${state.activeCell.index+1}`:'';
}
function toast(message,error=false){
  const el=$('#toast');el.textContent=message;el.classList.toggle('error',error);el.classList.add('visible');
  clearTimeout(state.toastTimer);state.toastTimer=setTimeout(()=>el.classList.remove('visible'),2700);
}
function setSort(id,shiftKey=false,direction){
  const specs=shiftKey?[...state.sort]:state.sort.filter(s=>s.id===id);
  const existing=specs.find(s=>s.id===id);
  const dir=direction||(existing?existing.dir==='asc'?'desc':existing.dir==='desc'?null:'asc':'asc');
  const remaining=specs.filter(s=>s.id!==id);
  if(dir)remaining.push({id,dir});
  state.sort=remaining;
  closeMenu();rebuild();
}
function resetFilters(){
  state.query='';state.statusFilter='All statuses';state.regionFilter='All regions';state.columnFilters={};
  $('#search-input').value='';$('#status-filter').value='All statuses';$('#region-filter').value='All regions';$('#company-filter').value='';
  $('#filter-counter').hidden=true;rebuild();
}
function updateFilterIndicator(){
  const active=(state.statusFilter!=='All statuses')+(state.regionFilter!=='All regions')+Object.values(state.columnFilters).filter(Boolean).length;
  $('#filter-counter').hidden=!active;$('#filter-counter').textContent=String(active);
}
function refreshSelectionVisual(){
  for(const cell of gridRows.querySelectorAll('.cell[data-rowindex]')){
    const i=Number(cell.dataset.rowindex),c=Number(cell.dataset.colindex);
    const selected=state.activeCell?.index===i && state.activeCell?.colIndex===c;
    cell.classList.toggle('selected-cell',selected);
    cell.classList.toggle('range-cell',!selected&&isWithinRange(i,c));
  }
  for(const row of gridRows.querySelectorAll('.grid-row[data-rowid]')){
    const selected=state.selectedRows.has(row.dataset.rowid);
    row.classList.toggle('selected-row',selected);
    const input=row.querySelector('input[data-rowcheck]');if(input)input.checked=selected;
  }
  $('#footer-selected').textContent=`${formatter.number(state.selectedRows.size)} selected`;
  $('#footer-active').textContent=state.activeCell?`${state.activeCell.colId} · row ${state.activeCell.index+1}`:'';
}
function chooseCell(index,colIndex,shiftKey=false){
  const item=state.displayRows[index];const col=getLayout().columns[colIndex];
  if(!item||item.type!=='data'||!col)return;
  if(!shiftKey||!state.anchor)state.anchor={index,colIndex};
  state.activeCell={index,colIndex,colId:col.id,rowId:item.row.orderId};
  refreshSelectionVisual();
}
function ensureInView(index){
  const desired=(index*state.rowHeight)+42;
  if(desired<gridScroll.scrollTop+42)gridScroll.scrollTop=Math.max(0,desired-100);
  else if(desired+state.rowHeight>gridScroll.scrollTop+gridScroll.clientHeight)gridScroll.scrollTop=desired+state.rowHeight-gridScroll.clientHeight+35;
}
function editCell(index,colIndex,initialInput){
  const item=state.displayRows[index],col=getLayout().columns[colIndex];
  if(!item||item.type!=='data'||!col||col.id==='orderId')return;
  if(state.editing)return;
  chooseCell(index,colIndex);
  const cell=gridRows.querySelector(`.cell[data-rowindex="${index}"][data-colindex="${colIndex}"]`);
  if(!cell)return;
  const original=item.row[col.id];
  const isSelect=['status','region'].includes(col.id);
  const editor=document.createElement(isSelect?'select':'input');
  editor.className='cell-editor';editor.setAttribute('aria-label',`Edit ${col.label}`);
  if(isSelect){(col.id==='status'?STATUSES:REGIONS).forEach(x=>{const option=document.createElement('option');option.value=x;option.textContent=x;editor.append(option);});editor.value=original;}
  else {editor.value=initialInput??String(original??'');editor.type=col.id==='date'?'date':'text';editor.autocomplete='off';}
  cell.replaceChildren(editor);state.editing={row:item.row,col,editor,original,index,colIndex,committed:false};
  editor.focus();if(!isSelect&&editor.type==='text')editor.select();
  const finish=(save)=>{
    if(!state.editing||state.editing.editor!==editor||state.editing.committed)return;
    state.editing.committed=true;
    if(save){const result=validateEdit(col,editor.value);if(result.ok){item.row[col.id]=result.value;toast(`Updated ${col.label} in ${item.row.orderId}`);}else{toast(result.error,true);}}
    state.editing=null;rebuild(false);gridScroll.focus({preventScroll:true});
  };
  editor.addEventListener('keydown',e=>{
    if(e.key==='Enter'){e.preventDefault();e.stopPropagation();finish(true);}
    if(e.key==='Escape'){e.preventDefault();e.stopPropagation();finish(false);}
  });
  editor.addEventListener('blur',()=>finish(true),{once:true});
  if(isSelect)editor.addEventListener('change',()=>finish(true),{once:true});
}
function showMenu(kind,colId,x,y,rowId){
  closeMenu();state.menu={kind,colId,rowId};
  const col=state.columns.find(c=>c.id===colId);
  const actions=kind==='header'?[
    ['asc','↑','Sort ascending'],['desc','↓','Sort descending'],['clear-sort','×','Clear sorting'],['separator'],
    ['pin','⌖',col.pinned?'Unpin column':'Pin column left'],['hide','◧','Hide column'],['filter','⌕','Filter column']
  ]:[['copy','⧉','Copy cell value'],['filter-value','⌕','Filter by this value'],['separator'],['export','↓','Export visible rows']];
  floatMenu.innerHTML=`<div class="menu-label">${esc(kind==='header'?col.label:'CELL ACTIONS')}</div>`+actions.map(a=>a[0]==='separator'?'<div class="menu-divider"></div>':`<button class="menu-item" data-menu-action="${a[0]}"><span style="width:15px;text-align:center;font-size:14px;line-height:1;">${a[1]}</span>${esc(a[2])}</button>`).join('');
  floatMenu.hidden=false;
  floatMenu.style.left=`${Math.max(8,Math.min(x,window.innerWidth-220))}px`;
  floatMenu.style.top=`${Math.max(8,Math.min(y,window.innerHeight-276))}px`;
}
function closeMenu(){floatMenu.hidden=true;state.menu=null;}
async function copyText(text){
  try{await navigator.clipboard.writeText(text);toast('Copied to clipboard');}
  catch{
    const ta=document.createElement('textarea');ta.value=text;ta.style.position='fixed';ta.style.opacity='0';document.body.append(ta);ta.select();
    const ok=document.execCommand('copy');ta.remove();toast(ok?'Copied to clipboard':'Clipboard access not available',!ok);
  }
}
function clipboardSelection(){
  const layout=getLayout();
  if(state.selectedRows.size){
    const rows=state.flatRows.filter(r=>state.selectedRows.has(r.orderId));
    return rows.map(r=>layout.columns.map(c=>String(r[c.id]??'')).join('\t')).join('\n');
  }
  if(!state.activeCell)return '';
  const a=state.anchor||state.activeCell,b=state.activeCell;
  const output=[];
  for(let i=Math.min(a.index,b.index);i<=Math.max(a.index,b.index);i++){
    const item=state.displayRows[i];if(item?.type!=='data')continue;
    const row=[];for(let c=Math.min(a.colIndex,b.colIndex);c<=Math.max(a.colIndex,b.colIndex);c++){row.push(String(item.row[layout.columns[c]?.id]??''));}
    output.push(row.join('\t'));
  }
  return output.join('\n');
}
function downloadCsv(){
  if(!state.flatRows.length){toast('There are no rows to export',true);return;}
  const cols=visibleColumns();const csv='\ufeff'+rowsToCsv(state.flatRows,cols);
  const blob=new Blob([csv],{type:'text/csv;charset=utf-8'});const url=URL.createObjectURL(blob);
  const a=document.createElement('a');a.href=url;a.download='gpui-grid-orders.csv';document.body.append(a);a.click();a.remove();setTimeout(()=>URL.revokeObjectURL(url),1000);
  toast(`Exported ${formatter.number(state.flatRows.length)} rows`);
}
function pasteData(text){
  if(!state.activeCell||!text.trim())return;
  const delimiter=text.includes('\t')?'\t':',';
  const matrix=parseDelimited(text,delimiter);
  if(!matrix.length)return;
  const cols=getLayout().columns;let changes=0;
  for(let i=0;i<matrix.length;i++){
    let offset=state.activeCell.index+i;
    let rowItem=state.displayRows[offset];
    if(!rowItem||rowItem.type!=='data')continue;
    for(let j=0;j<matrix[i].length;j++){
      const col=cols[state.activeCell.colIndex+j];if(!col)continue;
      const valid=validateEdit(col,matrix[i][j]);if(valid.ok){rowItem.row[col.id]=valid.value;changes++;}
    }
  }
  if(changes){rebuild(false);toast(`Pasted ${changes} cell${changes===1?'':'s'}`);}else toast('No editable values in selection',true);
}
function drawColumnPanel(){
  const opts=$('#column-options');opts.innerHTML=state.columns.map(c=>`<div class="column-option"><input type="checkbox" id="column-${c.id}" data-toggle-column="${c.id}" ${c.hidden?'':'checked'} ${c.id==='orderId'?'disabled':''}><label for="column-${c.id}">${esc(c.label)}</label><button data-pin-column="${c.id}" class="${c.pinned?'pinned-action':''}" aria-label="${c.pinned?'Unpin':'Pin'} ${esc(c.label)}" title="${c.pinned?'Unpin':'Pin'} column">${icon('pin')}</button></div>`).join('');
}
function alterColumn(id,action){
  const col=state.columns.find(c=>c.id===id);if(!col)return;
  if(action==='hide'){if(state.columns.filter(c=>!c.hidden).length<=1){toast('Keep at least one column visible',true);return;}col.hidden=true;}
  if(action==='toggle'){if(!col.hidden&&state.columns.filter(c=>!c.hidden).length<=1){toast('Keep at least one column visible',true);return;}col.hidden=!col.hidden;}
  if(action==='pin')col.pinned=!col.pinned;
  closeMenu();resetSelections();renderHeader();state.lastRange={start:-1,end:-1};renderVisible(true);drawColumnPanel();
}
function setDataSize(count){
  state.rows=generateRows(count);state.sort=[];state.collapsed.clear();state.selectedRows.clear();$('#dataset-select').value=String(count);
  rebuild();toast(`Loaded ${formatter.number(count)} deterministic demo rows`);
}
function setTab(tab){
  state.activeTab=tab;
  document.querySelectorAll('[data-tab]').forEach(btn=>{const on=btn.dataset.tab===tab;btn.classList.toggle('selected',on);btn.setAttribute('aria-selected',String(on));});
  rebuild();
}
function updatePageTitle(view){
  const content=initialPageText[view];if(!content)return;
  $('#breadcrumb').textContent=content[0];$('#page-title').innerHTML=content[1];$('#page-description').textContent=content[2];
  document.querySelectorAll('[data-view].nav-link').forEach(btn=>btn.classList.toggle('active',btn.dataset.view===view));
}
function setPreset(preset){
  state.preset=preset;
  document.querySelectorAll('[data-preset]').forEach(btn=>btn.classList.toggle('active-soft',btn.dataset.preset===preset));
  if(preset==='enterprise'){state.columnFilters={};state.statusFilter='All statuses';state.regionFilter='All regions';state.groupField='category';$('#group-select').value='category';toast('Enterprise CRM: grouped by product category');}
  if(preset==='global'){state.columnFilters={};state.statusFilter='All statuses';state.regionFilter='All regions';state.groupField='region';$('#group-select').value='region';toast('Global Operations: grouped by region');}
  if(preset==='commerce'){state.groupField='';$('#group-select').value='';toast('Sales Orders dataset restored');}
  updateFilterIndicator();$('#status-filter').value=state.statusFilter;$('#region-filter').value=state.regionFilter;$('#company-filter').value='';state.collapsed.clear();rebuild();
}
function toggleTheme(){
  const dark=document.body.classList.toggle('theme-dark');document.body.classList.toggle('theme-light',!dark);
  $('#theme-toggle').innerHTML=icon(dark?'sun':'moon');
  try{localStorage.setItem('gpui-grid-theme',dark?'dark':'light');}catch{}
  toast(`Quartz ${dark?'dark':'light'} theme active`);
}
function showDocs(){const dlg=$('#docs-dialog');if(!dlg.open)dlg.showModal();}
function hideDocs(){const dlg=$('#docs-dialog');if(dlg.open)dlg.close();}
function resetDemo(){
  state.columns=COLUMNS.map(c=>({...c,hidden:false}));state.sort=[];state.groupField='';state.collapsed.clear();state.selectedRows.clear();state.rowHeight=40;
  state.activeTab='all';state.activeCell=null;state.anchor=null;state.statusFilter='All statuses';state.regionFilter='All regions';state.columnFilters={};state.query='';
  $('#search-input').value='';$('#group-select').value='';$('#density-select').value='comfortable';$('#filter-panel').hidden=true;
  $('#status-filter').value='All statuses';$('#region-filter').value='All regions';$('#company-filter').value='';$('#column-panel').hidden=true;
  $('#filter-counter').hidden=true;document.querySelectorAll('[data-tab]').forEach(b=>{b.classList.toggle('selected',b.dataset.tab==='all');b.setAttribute('aria-selected',String(b.dataset.tab==='all'));});
  setDataSize(12480);state.view='orders';updatePageTitle('orders');
}
let searchTimer=null;
$('#search-input').addEventListener('input',event=>{clearTimeout(searchTimer);searchTimer=setTimeout(()=>{state.query=event.target.value;rebuild();},100);});
$('#search-input').addEventListener('keydown',e=>{if(e.key==='Escape'&&state.query){state.query='';e.target.value='';rebuild();}});
$('#status-filter').addEventListener('change',e=>{state.statusFilter=e.target.value;updateFilterIndicator();rebuild();});
$('#region-filter').addEventListener('change',e=>{state.regionFilter=e.target.value;updateFilterIndicator();rebuild();});
$('#company-filter').addEventListener('input',e=>{clearTimeout(searchTimer);searchTimer=setTimeout(()=>{state.columnFilters.company=e.target.value.trim();updateFilterIndicator();rebuild();},140);});
$('#group-select').addEventListener('change',e=>{state.groupField=e.target.value;state.collapsed.clear();rebuild();});
$('#density-select').addEventListener('change',e=>{state.density=e.target.value;state.rowHeight=e.target.value==='compact'?30:40;rebuild();});
$('#dataset-select').addEventListener('change',e=>setDataSize(Number(e.target.value)));
$('#theme-toggle').addEventListener('click',toggleTheme);
$('#reset-btn').addEventListener('click',resetDemo);
$('#export-btn').addEventListener('click',downloadCsv);
$('#advanced-filters').addEventListener('click',()=>{$('#filter-panel').hidden=!$('#filter-panel').hidden;});
$('#clear-filters').addEventListener('click',resetFilters);
$('#empty-clear').addEventListener('click',()=>{resetFilters();setTab('all');});
$('#show-columns').addEventListener('click',()=>{$('#column-panel').hidden=!$('#column-panel').hidden;drawColumnPanel();});
$('#close-columns').addEventListener('click',()=>{$('#column-panel').hidden=true;});
$('#reset-columns').addEventListener('click',()=>{state.columns=COLUMNS.map(c=>({...c,hidden:false}));renderHeader();state.lastRange={start:-1,end:-1};renderVisible(true);drawColumnPanel();toast('Column layout restored');});
$('#import-btn').addEventListener('click',()=>$('#csv-file').click());
$('#csv-file').addEventListener('change',async event=>{
  const file=event.target.files?.[0];if(!file)return;
  try{
    const data=parseDelimited(await file.text());if(data.length<2)throw new Error('A CSV with headers and rows is required');
    const headers=data.shift().map(s=>s.trim().toLowerCase());
    const columns=COLUMNS.map(c=>({column:c,index:headers.findIndex(h=>h===c.id.toLowerCase()||h===c.label.toLowerCase())}));
    if(columns.filter(c=>c.index>=0).length<3)throw new Error('At least three recognized column headers are required');
    const generated=generateRows(Math.min(data.length,20000));
    const imported=data.slice(0,20000).map((fields,i)=>{
      const row={...generated[i]};
      for(const {column,index} of columns){if(index<0||fields[index]==null||fields[index]==='')continue;const result=validateEdit(column,fields[index]);if(result.ok)row[column.id]=result.value;else if(column.id==='orderId')row.orderId=fields[index];}
      return row;
    });
    // Normalize duplicates so selection and editing have a stable identity.
    const seen=new Set();imported.forEach((r,i)=>{if(!r.orderId||seen.has(r.orderId))r.orderId=`IMPORTED-${i+1}`;seen.add(r.orderId);});
    state.rows=imported;state.sort=[];$('#dataset-select').value='';rebuild();toast(`Imported ${formatter.number(imported.length)} CSV rows`);
  }catch(err){toast(`CSV import: ${err.message}`,true);}
  event.target.value='';
});
$('#close-docs').addEventListener('click',hideDocs);
$('#got-it').addEventListener('click',hideDocs);
$('#docs-dialog').addEventListener('click',e=>{if(e.target.id==='docs-dialog')hideDocs();});

for(const btn of document.querySelectorAll('[data-tab]'))btn.addEventListener('click',()=>setTab(btn.dataset.tab));
for(const btn of document.querySelectorAll('[data-view]'))btn.addEventListener('click',()=>{
  const view=btn.dataset.view;
  if(view==='docs'){showDocs();return;}
  state.view=view;updatePageTitle(view);
  if(view==='performance'){$('#group-select').value='';state.groupField='';setDataSize(100000);}
  if(view==='themes'){toggleTheme();}
});
for(const btn of document.querySelectorAll('[data-preset]'))btn.addEventListener('click',()=>setPreset(btn.dataset.preset));

$('#column-options').addEventListener('change',e=>{const input=e.target.closest('[data-toggle-column]');if(input)alterColumn(input.dataset.toggleColumn,'toggle');});
$('#column-options').addEventListener('click',e=>{const button=e.target.closest('[data-pin-column]');if(button)alterColumn(button.dataset.pinColumn,'pin');});

gridHeader.addEventListener('click',event=>{
  if(event.target.closest('[data-resize]'))return;
  if(event.target.closest('#header-check')){
    const rows=state.flatRows;const all=rows.length>0&&rows.every(r=>state.selectedRows.has(r.orderId));
    for(const row of rows){if(all)state.selectedRows.delete(row.orderId);else state.selectedRows.add(row.orderId);}
    updateRowsWithoutRefilter();return;
  }
  const menu=event.target.closest('[data-colmenu]');if(menu){event.stopPropagation();const rect=menu.getBoundingClientRect();showMenu('header',menu.dataset.colmenu,rect.left-130,rect.bottom+8);return;}
  const header=event.target.closest('[data-colid]');if(header&& !state.isResizing)setSort(header.dataset.colid,event.shiftKey);
});
gridHeader.addEventListener('dragstart',e=>{const h=e.target.closest('.header-cell[data-colid]');if(!h||e.target.closest('[data-resize]'))return;e.dataTransfer.effectAllowed='move';e.dataTransfer.setData('text/plain',h.dataset.colid);});
gridHeader.addEventListener('dragover',e=>{if(e.target.closest('.header-cell[data-colid]'))e.preventDefault();});
gridHeader.addEventListener('drop',e=>{
  const target=e.target.closest('.header-cell[data-colid]');if(!target)return;e.preventDefault();
  const from=e.dataTransfer.getData('text/plain');const to=target.dataset.colid;
  if(from===to)return;
  const old=state.columns.findIndex(c=>c.id===from),next=state.columns.findIndex(c=>c.id===to);
  if(old<0||next<0)return;
  const item=state.columns.splice(old,1)[0];item.pinned=state.columns.find(c=>c.id===to)?.pinned??false;
  state.columns.splice(next,0,item);resetSelections();renderHeader();renderVisible(true);drawColumnPanel();toast('Column order updated');
});
gridHeader.addEventListener('pointerdown',e=>{
  const handle=e.target.closest('[data-resize]');if(!handle)return;
  e.preventDefault();e.stopPropagation();const col=state.columns.find(c=>c.id===handle.dataset.resize);if(!col)return;
  state.isResizing=true;const startX=e.clientX,startWidth=col.width;
  document.body.style.cursor='col-resize';document.body.style.userSelect='none';
  function move(evt){col.width=Math.max(86,Math.min(420,startWidth+evt.clientX-startX));state.lastRange={start:-1,end:-1};renderHeader();renderVisible(true);}
  function stop(){document.removeEventListener('pointermove',move);document.removeEventListener('pointerup',stop);document.body.style.cursor='';document.body.style.userSelect='';setTimeout(()=>{state.isResizing=false;},0);}
  document.addEventListener('pointermove',move);document.addEventListener('pointerup',stop,{once:true});
});

gridRows.addEventListener('click',e=>{
  const cb=e.target.closest('[data-rowcheck]');if(cb){const id=cb.dataset.rowcheck;if(state.selectedRows.has(id))state.selectedRows.delete(id);else state.selectedRows.add(id);updateRowsWithoutRefilter();return;}
  const group=e.target.closest('[data-group]');if(group){const key=group.dataset.group;if(state.collapsed.has(key))state.collapsed.delete(key);else state.collapsed.add(key);rebuild(false);return;}
  const cell=e.target.closest('.cell[data-rowindex]');if(!cell)return;
  chooseCell(Number(cell.dataset.rowindex),Number(cell.dataset.colindex),e.shiftKey);
  gridScroll.focus({preventScroll:true});
});
gridRows.addEventListener('dblclick',e=>{
  const cell=e.target.closest('.cell[data-rowindex]');if(!cell)return;
  editCell(Number(cell.dataset.rowindex),Number(cell.dataset.colindex));
});
gridRows.addEventListener('contextmenu',e=>{
  const cell=e.target.closest('.cell[data-rowindex]');if(!cell)return;
  e.preventDefault();chooseCell(Number(cell.dataset.rowindex),Number(cell.dataset.colindex));
  showMenu('cell',cell.dataset.colid,e.clientX,e.clientY,cell.dataset.rowid);
});
gridScroll.addEventListener('scroll',()=>{requestAnimationFrame(()=>renderVisible());closeMenu();},{passive:true});
gridScroll.addEventListener('keydown',e=>{
  if(e.target.matches('input,select,textarea'))return;
  const mod=e.metaKey||e.ctrlKey;
  if(mod &&e.key.toLowerCase()==='c'){e.preventDefault();const t=clipboardSelection();if(t)copyText(t);return;}
  if(mod&&e.key.toLowerCase()==='a'){e.preventDefault();state.flatRows.forEach(r=>state.selectedRows.add(r.orderId));updateRowsWithoutRefilter();return;}
  if(e.key==='Escape'){closeMenu();state.anchor=null;state.activeCell=null;updateRowsWithoutRefilter();return;}
  if(!state.activeCell)return;
  const {index,colIndex}=state.activeCell;
  if(e.key==='Enter'||e.key==='F2'){e.preventDefault();editCell(index,colIndex);return;}
  const moves={ArrowUp:[-1,0],ArrowDown:[1,0],ArrowLeft:[0,-1],ArrowRight:[0,1],Tab:[0,e.shiftKey?-1:1]};
  if(moves[e.key]){
    e.preventDefault();const [dy,dx]=moves[e.key];let nextIndex=index+dy;
    nextIndex=Math.max(0,Math.min(state.displayRows.length-1,nextIndex));
    while(state.displayRows[nextIndex]?.type==='group'&&nextIndex!==index){nextIndex=Math.max(0,Math.min(state.displayRows.length-1,nextIndex+(dy||1)));}
    const col=Math.max(0,Math.min(visibleColumns().length-1,colIndex+dx));
    chooseCell(nextIndex,col,e.shiftKey&&e.key!=='Tab');ensureInView(nextIndex);
  }
});
gridScroll.addEventListener('paste',e=>{if(e.target.matches('input,select'))return;if(!state.activeCell)return;e.preventDefault();pasteData(e.clipboardData.getData('text/plain'));});

floatMenu.addEventListener('click',e=>{
  const el=e.target.closest('[data-menu-action]');if(!el||!state.menu)return;
  const action=el.dataset.menuAction;const {colId,rowId,kind}=state.menu;
  if(action==='asc'||action==='desc')setSort(colId,false,action);
  if(action==='clear-sort'){state.sort=state.sort.filter(s=>s.id!==colId);closeMenu();rebuild();}
  if(action==='pin'||action==='hide')alterColumn(colId,action);
  if(action==='filter'){
    $('#filter-panel').hidden=false;
    if(colId==='company')$('#company-filter').focus();else $('#search-input').focus();
    closeMenu();
  }
  if(action==='copy'||action==='filter-value'){
    const row=state.flatRows.find(r=>r.orderId===rowId);const value=row?.[colId];
    if(action==='copy')copyText(String(value??''));
    else {$('#search-input').value=String(value??'');state.query=String(value??'');rebuild();}
    closeMenu();
  }
  if(action==='export'){closeMenu();downloadCsv();}
});
document.addEventListener('pointerdown',e=>{if(!floatMenu.hidden&&!e.target.closest('#floating-menu')&&!e.target.closest('[data-colmenu]'))closeMenu();});
window.addEventListener('keydown',e=>{
  const mod=e.metaKey||e.ctrlKey;
  if(mod&&e.key.toLowerCase()==='k'){e.preventDefault();$('#search-input').focus();$('#search-input').select();}
  if(e.key==='Escape'&&!$('#docs-dialog').open)closeMenu();
});

try{if(localStorage.getItem('gpui-grid-theme')==='dark'){document.body.classList.add('theme-dark');document.body.classList.remove('theme-light');$('#theme-toggle').innerHTML=icon('sun');}}catch{}
rebuild();
