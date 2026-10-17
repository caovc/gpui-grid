import { FEATURES, FEATURE_GROUPS, LIVE_FEATURES, byFeatureId } from './catalog.js';

// IMPORTANT: This file only renders docs HTML and selects an actual Rust/GPUI
// WASM showcase. It does NOT contain data rows, cell renderers, or JS Grid APIs.
const $ = selector => document.querySelector(selector);
const frame = $('#gpui-frame');
const status = $('#runtime');
const details = $('#runtime-details');
const sidebar = $('#sidebar');
const mobileOverlay = $('#mobile-overlay');
const search = $('#feature-search');
const nav = $('#feature-nav');

let active = byFeatureId(new URLSearchParams(location.search).get('feature'));
let view = 'preview';
const navLinks = new Map();

function el(tag, className = '', value = '') {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (value) node.textContent = value;
  return node;
}
function setText(selector, value) { $(selector).textContent = value; }
function setRuntime(message, state = '', caption = '') {
  status.className = 'runtime' + (state ? ' ' + state : '');
  status.replaceChildren(el('i', 'status-dot'), document.createTextNode(message));
  details.textContent = caption || '等待 GPUI WebPlatform 初始化…';
}
function viewerUrl(feature) {
  const url = new URL('./viewer/', location.href);
  url.searchParams.set('feature', feature.id);
  return url;
}
function makeHref(feature) {
  const u = new URL(location.href);
  u.searchParams.set('feature', feature.id);
  u.hash = '';
  return u.pathname + u.search;
}

function renderNavigation() {
  nav.replaceChildren();
  navLinks.clear();
  for (const group of FEATURE_GROUPS) {
    const section = el('div', 'nav-group');
    section.dataset.group = group.id;
    const heading = el('div', 'nav-heading');
    heading.append(el('span', '', group.label), el('small', '', group.title));
    section.append(heading);
    for (const feature of FEATURES.filter(item => item.group === group.id)) {
      const link = el('a', 'nav-item');
      link.href = makeHref(feature);
      link.dataset.feature = feature.id;
      const icon = el('span', 'nav-item-icon', feature.icon);
      icon.setAttribute('aria-hidden', 'true');
      const label = el('span', 'nav-item-label', feature.short);
      const en = el('small', 'nav-item-en', feature.en);
      label.append(en);
      const trailing = el('span', feature.status === 'live' ? 'nav-live' : 'nav-planned',
        feature.status === 'live' ? '●' : '↗');
      trailing.setAttribute('aria-label', feature.status === 'live' ? '可交互示例' : '开发中');
      link.append(icon, label, trailing);
      link.addEventListener('click', event => {
        if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
        event.preventDefault();
        navigate(feature);
      });
      navLinks.set(feature.id, link);
      section.append(link);
    }
    nav.append(section);
  }
}

function filterNavigation() {
  const query = search.value.trim().toLocaleLowerCase();
  let matches = 0;
  for (const feature of FEATURES) {
    const match = !query ||
      [feature.id, feature.title, feature.short, feature.en, feature.summary, ...(feature.tags || [])]
        .join(' ').toLocaleLowerCase().includes(query);
    navLinks.get(feature.id).hidden = !match;
    if (match) matches++;
  }
  for (const section of nav.querySelectorAll('.nav-group')) {
    section.hidden = ![...section.querySelectorAll('.nav-item')].some(node => !node.hidden);
  }
  let empty = $('#empty-search');
  if (!matches && !empty) {
    empty = el('div', 'empty-search', '没有匹配的功能，请尝试英文或中文关键词。');
    empty.id = 'empty-search';
    nav.append(empty);
  }
  if (empty) empty.hidden = !!matches;
}

function renderList(target, entries, type) {
  const root = $(target);
  root.replaceChildren();
  entries.forEach((value, index) => {
    if (type === 'hint') {
      const card = el('div', 'hint-item');
      card.append(el('span', 'hint-index', String(index + 1).padStart(2, '0')),
        el('span', '', value));
      root.append(card);
    } else if (type === 'step') {
      const row = el('div', 'step-item');
      row.append(el('span', 'step-index', String(index + 1)),
        el('p', '', value));
      root.append(row);
    } else if (type === 'api') {
      const tr = el('tr');
      const key = el('td');
      key.append(el('code', '', value[0]));
      tr.append(key, el('td', '', value[1]));
      root.append(tr);
    } else {
      root.append(el('li', '', value));
    }
  });
}

function setTab(next) {
  view = next;
  $('#preview-panel').hidden = next !== 'preview';
  $('#code-panel').hidden = next !== 'code';
  for (const tab of document.querySelectorAll('.playground-tab')) {
    const selected = tab.dataset.view === next;
    tab.classList.toggle('active', selected);
    tab.setAttribute('aria-selected', String(selected));
  }
}

function renderPage(feature) {
  active = feature;
  document.title = feature.title + ' · GPUI Grid Docs';
  setText('#current-group', FEATURE_GROUPS.find(g => g.id === feature.group)?.title || 'Docs');
  setText('#breadcrumb-current', feature.en);
  setText('#feature-category', FEATURE_GROUPS.find(g => g.id === feature.group)?.label || '');
  setText('#feature-index', String(FEATURES.indexOf(feature) + 1).padStart(2, '0'));
  setText('#feature-title', feature.title);
  setText('#feature-summary', feature.summary);
  setText('#demo-context', feature.demoContext);
  setText('#feature-description', feature.description);
  $('#feature-status').textContent = feature.status === 'live' ? '● LIVE RUST WASM'
    : feature.status === 'core' ? '◐ CORE ONLY' : '○ PLANNED';
  $('#feature-status').className = 'feature-status ' + feature.status;
  const tagRoot = $('#feature-tags');
  tagRoot.replaceChildren(...feature.tags.map(tag => el('span', 'feature-tag', tag)));
  renderList('#demo-hints', feature.hints || [], 'hint');
  renderList('#feature-steps', feature.steps, 'step');
  renderList('#api-rows', feature.api, 'api');
  renderList('#feature-notes', feature.notes, 'note');
  $('#rust-example-code').textContent = feature.code.trim();
  $('#planned-description').textContent = feature.demoContext;

  const live = feature.status === 'live';
  $('#playground-card').hidden = !live;
  $('#planned-example').hidden = live;
  $('#demo-hints').hidden = !live;
  if (live) {
    const next = viewerUrl(feature);
    $('#open-fullscreen').href = next.href;
    if (frame.src !== next.href) {
      setRuntime('Initializing WASM…', '', '正在加载 ' + feature.en + ' 对应的 Rust GPUI 视图…');
      // The iframe has its own GPUI WebPlatform instance. Recreate it when the
      // demo changes, rather than calling any JS imitation of grid operations.
      frame.src = next.href;
    }
  } else if (frame.getAttribute('src') !== 'about:blank') {
    // Planned pages intentionally do NOT show the overview grid as a fake demo.
    frame.src = 'about:blank';
    setRuntime('No live demo', '', '该功能的 GPUI 界面正在开发中');
  }

  for (const [key, link] of navLinks) {
    const selected = key === feature.id;
    link.classList.toggle('active', selected);
    if (selected) link.setAttribute('aria-current', 'page');
    else link.removeAttribute('aria-current');
  }
  const ix = FEATURES.indexOf(feature);
  for (const [selector, neighbor, kind] of [
    ['#prev-page', FEATURES[Math.max(0, ix - 1)], 'prev'],
    ['#next-page', FEATURES[Math.min(FEATURES.length - 1, ix + 1)], 'next'],
  ]) {
    const a = $(selector);
    a.href = makeHref(neighbor);
    a.querySelector('strong').textContent = neighbor.short;
    a.classList.toggle('disabled', neighbor.id === feature.id);
    a.setAttribute('aria-label', kind === 'prev' ? '上一篇：' + neighbor.title : '下一篇：' + neighbor.title);
  }
  setTab('preview');
}

function closeMobileMenu() {
  document.body.classList.remove('sidebar-open');
  mobileOverlay.hidden = true;
  $('#mobile-menu').setAttribute('aria-expanded', 'false');
}
function navigate(feature, historyMethod = 'pushState') {
  if (historyMethod && (feature.id !== active.id || location.search !== '?feature=' + feature.id)) {
    history[historyMethod]({feature: feature.id}, '', makeHref(feature));
  }
  renderPage(feature);
  closeMobileMenu();
  document.querySelector('.doc-main').scrollIntoView({behavior: 'auto', block: 'start'});
}

renderNavigation();
renderPage(active);
search.addEventListener('input', filterNavigation);
window.addEventListener('popstate', () => navigate(byFeatureId(new URLSearchParams(location.search).get('feature')), null));
$('#reload').addEventListener('click', () => {
  if (active.status !== 'live') return;
  setRuntime('Initializing WASM…', '', '重新加载 Rust / GPUI runtime…');
  if (frame.contentWindow) frame.contentWindow.location.reload();
});
window.addEventListener('message', event => {
  if (event.source !== frame.contentWindow || event.origin !== location.origin) return;
  if (event.data?.source !== 'gpui-grid-wasm') return;
  if (event.data.type === 'ready') {
    setRuntime('GPUI WASM initialized', 'ready', 'Rust → wasm-bindgen → GPUI WebPlatform · ' + active.en);
  } else if (event.data.type === 'error') {
    setRuntime('GPUI WASM startup failed', 'error', '检查浏览器 WebGPU/WebGL2、Canvas 与 Console 错误。');
  }
});
for (const tab of document.querySelectorAll('.playground-tab')) {
  tab.addEventListener('click', () => setTab(tab.dataset.view));
}
$('#copy-code').addEventListener('click', async () => {
  try {
    await navigator.clipboard.writeText($('#rust-example-code').textContent);
    const label = $('#copy-code');
    label.textContent = '✓ 已复制';
    setTimeout(() => { label.textContent = '⧉ 复制代码'; }, 1600);
  } catch {
    $('#copy-code').textContent = '请手动选择文本复制';
  }
});

$('#mobile-menu').addEventListener('click', () => {
  const opening = !document.body.classList.contains('sidebar-open');
  document.body.classList.toggle('sidebar-open', opening);
  mobileOverlay.hidden = !opening;
  $('#mobile-menu').setAttribute('aria-expanded', String(opening));
});
mobileOverlay.addEventListener('click', closeMobileMenu);
document.addEventListener('keydown', event => {
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k'
      || event.key === '/' && !['INPUT', 'TEXTAREA'].includes(document.activeElement.tagName)) {
    event.preventDefault();
    search.focus();
    if (!document.body.classList.contains('sidebar-open') && innerWidth < 880)
      $('#mobile-menu').click();
  }
  if (event.key === 'Escape') {
    closeMobileMenu();
    if (document.activeElement === search) search.blur();
  }
});

function applyTheme(dark) {
  document.documentElement.classList.toggle('theme-dark', dark);
  $('#theme-toggle').textContent = dark ? '☀' : '◐';
  $('#theme-toggle').setAttribute('aria-pressed', String(dark));
}
let stored = null;
try { stored = localStorage.getItem('gpui-grid-docs-theme'); } catch {}
applyTheme(stored === 'dark');
$('#theme-toggle').addEventListener('click', () => {
  const dark = !document.documentElement.classList.contains('theme-dark');
  applyTheme(dark);
  try { localStorage.setItem('gpui-grid-docs-theme', dark ? 'dark' : 'light'); } catch {}
});
