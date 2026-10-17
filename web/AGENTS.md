# Feature documentation / genuine WASM preview instructions

Read root `AGENTS.md` first.

This is **not a JavaScript Data Grid**. `web/shell.js` and `web/catalog.js` implement feature navigation, searchable documentation, headings, API descriptions, code samples, dark/light docs chrome and iframe routes. They must never sort, filter, edit, paginate or render grid rows.

- Feature-specific live demos launch a real `gpui_grid_web_bg.wasm` generated from `crates/gpui-grid-web`, which runs the same native Rust GPUI `GridShowcase`.
- `web/viewer/loader.js` is allowed to load WASM and pass the `feature` route to `run(feature)`. No JS mock fallback.
- The docs catalog's `status` must be `live`, `core`, or `planned`; do not silently render the overview grid as a live example of an unimplemented feature. Core-only/Planned explicitly show no live interactive grid.
- Add a feature page only after its Rust GPUI widget exists. Provide what-to-click guidance, actual Rust API code, known platform limits, and links to Issues. Avoid fabricated example API identifiers.
- `web/build.mjs` refuses to publish if genuine WASM magic/bindings are missing. Keep that gate. Update bundled asset copy list when adding docs assets.
- Preserve existing legacy Playwright Canvas E2E. For new in-grid controls, test on the **iframe canvas** and check model-derived, **read-only** WASM diagnostics. Do not simulate operations via JavaScript.
- Keep router/deep links working on GitHub Pages at `/gpui-grid/?feature=...`. Capture WebGL2/WebGPU failures as diagnostics rather than masking them.
- Never bundle proprietary AG Grid assets, fonts, theme source code, or Enterprise functionality.
