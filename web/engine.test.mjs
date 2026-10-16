import test from 'node:test';
import assert from 'node:assert/strict';
import {COLUMNS, generateRows, filterRows, sortRows,groupRows,visibleRange,rowsToCsv,parseDelimited,validateEdit,summaryOf} from './engine.js';
test('deterministic datasets and unique stable ids',()=>{
 const a=generateRows(120);const b=generateRows(120);
 assert.deepEqual(a,b);assert.equal(new Set(a.map(r=>r.orderId)).size,120);
 assert.equal(COLUMNS.length,14);
});
test('sort is stable and does not mutate original',()=>{
 const data=[{n:'b',id:0},{n:'a',id:1},{n:'a',id:2}];
 assert.deepEqual(sortRows(data,[{id:'n',dir:'asc'}]).map(r=>r.id),[1,2,0]);
 assert.equal(data[0].n,'b');
 assert.deepEqual(sortRows(data,[{id:'n',dir:'desc'}]).map(r=>r.id),[0,1,2]);
});
test('filter combines query, status and column filter',()=>{
 const rows=generateRows(50);const matching=filterRows(rows,{query:'sophia',status:'Completed',columnFilters:{company:'Figma'}});
 assert.equal(matching.length,1);assert.equal(matching[0].customer,'Sophia Chen');
});
test('group aggregation and collapse',()=>{
 const rows=[{region:'APAC',amount:10},{region:'Europe',amount:20},{region:'APAC',amount:5}];
 const open=groupRows(rows,'region');assert.equal(open.length,5);assert.equal(open[0].total,15);
 const closed=groupRows(rows,'region',new Set(['APAC']));assert.equal(closed.length,3);
});
test('viewport calculation keeps DOM bounded',()=>{
 const p=visibleRange(40000,650,40,100000);
 assert.ok(p.end-p.start<45);assert.ok(p.start<=1000&&p.end>=1017);
 assert.deepEqual(visibleRange(0,100,40,0),{start:0,end:0});
});
test('CSV quotes commas, prevents formula injection, parses multiline',()=>{
 const csv=rowsToCsv([{value:'hello, world',comment:'=cmd()'}],[{id:'value',label:'Value'},{id:'comment',label:'Comment'}]);
 assert.match(csv,/"hello, world"/);assert.match(csv,/'=cmd\(\)/);
 assert.deepEqual(parseDelimited('a,b\r\n"x,y","line1\nline2"'),[['a','b'],['x,y','line1\nline2']]);
});
test('editing validates numeric and text boundaries',()=>{
 assert.equal(validateEdit({id:'amount'},'12,500').value,12500);
 assert.equal(validateEdit({id:'score'},'120').ok,false);
 assert.equal(validateEdit({id:'orderId'},'x').ok,false);
 assert.equal(validateEdit({id:'status'},'Completed').ok,true);
});
test('summary returns sensible totals',()=>{
 assert.deepEqual(summaryOf([{amount:10,status:'Completed',score:80},{amount:20,status:'Pending',score:90}]),{total:2,revenue:30,completed:1,averageScore:85});
});
