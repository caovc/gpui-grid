/**
 * Small, offline Rust tokenizer for documentation snippets. No CDN, no eval,
 * and no HTML injection: the DOM renderer uses textContent only.
 * This is syntax highlighting for the documentation, NOT a JavaScript grid.
 */
const KEYWORDS = new Set(
  'as async await break const continue crate dyn else enum extern false fn for if impl in let loop match mod move mut pub ref return self Self static struct super trait true type union unsafe use where while macro_rules'.split(' ')
);
const PRIMITIVES = new Set(
  'bool char str usize isize u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64'.split(' ')
);
const SYMBOLIC = new Set('Some None Ok Err Result Option String Vec Box Arc Rc Cell RefCell'.split(' '));

function readString(source, start, quote) {
  let pos = start + 1;
  while (pos < source.length) {
    if (source[pos] === '\\') { pos += 2; continue; }
    if (source[pos] === quote) return pos + 1;
    pos++;
  }
  return source.length;
}

function readBlockComment(source, start) {
  let depth = 1;
  let pos = start + 2;
  while (pos < source.length && depth) {
    if (source.startsWith('/*', pos)) { depth++; pos += 2; }
    else if (source.startsWith('*/', pos)) { depth--; pos += 2; }
    else pos++;
  }
  return pos;
}

export function tokenizeRust(source) {
  source = String(source);
  const tokens = [];
  let cursor = 0;
  let plain = '';
  function push(kind, value) {
    if (!value) return;
    if (!kind) { plain += value; return; }
    if (plain) { tokens.push({ kind: null, text: plain }); plain = ''; }
    tokens.push({ kind, text: value });
  }

  while (cursor < source.length) {
    const rest = source.slice(cursor);
    if (rest.startsWith('//')) {
      const end = source.indexOf('\n', cursor);
      const stop = end < 0 ? source.length : end;
      push('comment', source.slice(cursor, stop)); cursor = stop; continue;
    }
    if (rest.startsWith('/*')) {
      const stop = readBlockComment(source, cursor);
      push('comment', source.slice(cursor, stop)); cursor = stop; continue;
    }
    if (rest.startsWith('#[')) {
      let depth = 1, pos = cursor + 2;
      for (; pos < source.length && depth; pos++) {
        if (source[pos] === '[') depth++;
        if (source[pos] === ']') depth--;
      }
      push('attribute', source.slice(cursor, pos)); cursor = pos; continue;
    }
    const raw = rest.match(/^r(#{0,8})"/);
    if (raw) {
      const close = '"' + raw[1];
      const end = source.indexOf(close, cursor + raw[0].length);
      const stop = end < 0 ? source.length : end + close.length;
      push('string', source.slice(cursor, stop)); cursor = stop; continue;
    }
    if (rest[0] === '"') {
      const stop = readString(source, cursor, '"');
      push('string', source.slice(cursor, stop)); cursor = stop; continue;
    }
    if (rest[0] === "'") {
      const ch = rest.match(/^'(?:\\.|[^'\\])'/);
      if (ch) { push('string', ch[0]); cursor += ch[0].length; continue; }
      const lifetime = rest.match(/^'[a-zA-Z_]\w*/);
      if (lifetime) { push('lifetime', lifetime[0]); cursor += lifetime[0].length; continue; }
    }
    if (/^[0-9]/.test(rest)) {
      const number = rest.match(/^[0-9][0-9_]*(?:\.[0-9_]+)?(?:[a-zA-Z][a-zA-Z0-9_]*)?/);
      if (number) { push('number', number[0]); cursor += number[0].length; continue; }
    }
    const identifier = rest.match(/^[a-zA-Z_]\w*!?/);
    if (identifier) {
      const word = identifier[0];
      const next = source.slice(cursor + word.length);
      const kind = word.endsWith('!') ? 'macro'
        : KEYWORDS.has(word) ? 'keyword'
        : PRIMITIVES.has(word) || SYMBOLIC.has(word) || /^[A-Z]/.test(word) ? 'type'
        : /^\s*\(/.test(next) ? 'function'
        : null;
      push(kind, word); cursor += word.length; continue;
    }
    push(null, rest[0]); cursor++;
  }
  if (plain) tokens.push({kind: null, text: plain});
  return tokens;
}

export function renderRustHighlight(element, source) {
  const doc = element.ownerDocument;
  const fragment = doc.createDocumentFragment();
  for (const token of tokenizeRust(source)) {
    if (!token.kind) fragment.append(doc.createTextNode(token.text));
    else {
      const span = doc.createElement('span');
      span.className = 'rust-token rust-' + token.kind;
      span.textContent = token.text;
      fragment.append(span);
    }
  }
  element.replaceChildren(fragment);
  element.classList.add('language-rust');
  element.dataset.syntax = 'rust';
}
