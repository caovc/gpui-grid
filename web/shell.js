// This shell is only page chrome. Every data grid pixel and interaction is drawn by GPUI WASM.
const frame = document.getElementById('gpui-frame');
const status = document.getElementById('runtime');
const details = document.getElementById('runtime-details');
document.getElementById('reload').addEventListener('click', () => {
  status.className = 'runtime'; status.lastChild.textContent = 'Initializing WASM…';
  details.textContent = 'Restarting Rust / GPUI runtime…';
  frame.contentWindow.location.reload();
});
window.addEventListener('message', event => {
  if (event.source !== frame.contentWindow || event.origin !== window.location.origin) return;
  if (event.data?.source !== 'gpui-grid-wasm') return;
  if (event.data.type === 'ready') {
    status.className = 'runtime ready';
    status.lastChild.textContent = 'GPUI WASM initialized';
    details.textContent = 'Rust → wasm-bindgen → GPUI WebPlatform';
  } else if (event.data.type === 'error') {
    status.className = 'runtime error';
    status.lastChild.textContent = 'WASM startup failed';
    details.textContent = 'See canvas error details and browser console.';
  }
});
