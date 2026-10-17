import {test} from 'node:test';
import {strict as assert} from 'node:assert';
import {tokenizeRust} from './rust-highlight.js';

test('Rust highlighting recognizes keywords, types, functions, macros, strings, comments and numbers', () => {
  const sample = [
    'use gpui_grid::{GridDelegate, Sort};',
    '/// Rust source, not JS grid',
    'fn build() {',
    '    let mut grid = GridDelegate::new(sample_orders(12_480));',
    '    println!("not // a comment \\"quoted\\"");',
    '    /* nested /* comments */ work */',
    '    #[derive(Debug)]',
    '    let s = r#"raw string"#;',
    '}',
  ].join('\n');
  const parts = tokenizeRust(sample);
  assert.equal(parts.map(part => part.text).join(''), sample, 'tokenization must be lossless');
  for (const kind of ['keyword','type','function','macro','comment','string','number','attribute']) {
    assert.ok(parts.some(part => part.kind === kind), 'expected '+kind);
  }
  assert.ok(parts.some(part => part.text.includes('12_480') && part.kind === 'number'));
  assert.ok(parts.some(part => part.text.includes('/* nested /* comments */ work */') && part.kind === 'comment'));
});

test('highlighter treats arbitrary input as text, not executable markup', () => {
  const input = 'let html = "<script>alert(1)</script>"; // <iframe src=x>';
  const tokens = tokenizeRust(input);
  assert.equal(tokens.map(t => t.text).join(''), input);
  assert.equal(tokens.find(t => t.text.startsWith('"<script>')).kind, 'string');
  assert.equal(tokens.find(t => t.text.startsWith('// <iframe')).kind, 'comment');
});

test('function calls and Rust primitives are highlighted without stealing comments', () => {
  const tokens = tokenizeRust("let answer: f64 = f64::from(42); // from(42) comment");
  assert.ok(tokens.some(t => t.text === 'f64' && t.kind === 'type'));
  assert.ok(tokens.some(t => t.text === 'from' && t.kind === 'function'));
  assert.ok(tokens.some(t => t.text.includes('from(42) comment') && t.kind === 'comment'));
});
