// There is no HTML/JavaScript table implementation in this viewer.
// The generated JS is glue for the Rust gpui-grid-web crate, produced by wasm-bindgen.
const notice = document.getElementById('boot');
const report = (type) => window.parent?.postMessage({ source: 'gpui-grid-wasm', type }, window.location.origin);
try {
  const wasm = await import('./pkg/gpui_grid_web.js');
  await wasm.default();
  // A feature route selects a Rust GridShowcase mode. The docs host never
  // renders or mutates table rows in JavaScript.
  const feature = new URLSearchParams(window.location.search).get('feature') || 'overview';
  wasm.run(feature);
  // GPUI's run_embedded initializes graphics asynchronously. Do not advertise success
  // until the GPUI WebPlatform has actually appended its rendering canvas.
  const canvas = await new Promise((resolve, reject) => {
    const existing = document.querySelector('body > canvas');
    if (existing) return resolve(existing);
    const watcher = new MutationObserver(() => {
      const ready = document.querySelector('body > canvas');
      if (ready) { watcher.disconnect(); clearTimeout(timer); resolve(ready); }
    });
    watcher.observe(document.body, { childList: true });
    const timer = setTimeout(() => { watcher.disconnect(); reject(new Error('GPUI WebPlatform did not create a canvas within 30 seconds')); }, 30000);
  });
  await new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));
  notice.remove();
  report('ready');
  canvas.setAttribute('aria-label', 'GPUI Grid Rust WASM data grid');
} catch (error) {
  console.error('GPUI WebAssembly startup error:', error);
  notice.classList.add('error');
  notice.replaceChildren();
  const box = document.createElement('div');
  const heading = document.createElement('h2');
  heading.textContent = 'Unable to start GPUI WebAssembly';
  const trace = document.createElement('pre');
  trace.textContent = String(error?.stack || error);
  box.append(heading, trace);
  notice.append(box);
  report('error');
}
