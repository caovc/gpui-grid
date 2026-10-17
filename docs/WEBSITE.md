# Feature documentation website

The docs site is a **feature-indexed static shell**, not a JavaScript grid application. Following the organizational pattern of comprehensive grid documentation, each section contains: explanation, real example, Rust source, public APIs, and current limitations.

## Feature pages

| Route | Actual Rust GPUI example |
|---|---|
| `/?feature=overview` | Same base `GridShowcase::new` and 12,480-row DataTable |
| `/?feature=sorting` | Real GPUI column menu: ascending/descending, clear and Add Multi-sort |
| `/?feature=filtering` | Native GPUI header InputState and typed filter operators |
| `/?feature=selection` | GPUI Shift+click, Shift+arrows and Ctrl/Cmd+A without demo buttons |
| `/?feature=editing` | Shared in-cell GPUI InputState and Edit transaction |
| `/?feature=columns` | GPUI drag/resize/pinning and header Hide column / Show all |
| `/?feature=virtualization` | 50,000 local Rust rows with GPUI Kit virtual list |
| `/?feature=grouping` | Core-only notice; no fake GPU/JS implementation |
| `/?feature=clipboard`, `server-side`, `pivot` | Planned notice; no fake live example |

There is no generated HTML table; `web/shell.js` manipulates documentation DOM only. The page embeds a dedicated iframe at `web/viewer/?feature=<name>`. `loader.js` initializes the compiled `gpui_grid_web_bg.wasm` and calls the wasm-bindgen `run(feature)` export. Rust's `GridFeature::from_key` chooses a feature-focused `GridShowcase` and the same `GridDelegate` used by the native executable. This is not client-side RPC into a JavaScript grid.

## Local development

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.121
./scripts/build-wasm.sh --release
npm --prefix web test
npm --prefix web run build
npm --prefix web run dev
# http://127.0.0.1:4173/?feature=selection
```

`web/build.mjs` **refuses to produce a site** without the actual WASM binary. The browser must support a GPUI WebPlatform GPU backend; WebAssembly is still experimental/showcase-only. A passed WASM compile is not proof of visual or input support in arbitrary browsers.

## Tests and deployment

- Node tests validate the feature catalog, route wiring, actual wasm-bindgen loader, and planned-feature truthfulness.
- Playwright tests launch real Rust GPUI WebAssembly in Chromium with software GL and inspect Canvas output plus read-only Rust diagnostics; HTML-only tests cannot replace them.
- GitHub Actions builds the actual Rust wasm32 crate, generates wasm-bindgen bindings, and publishes the static output from `web/dist` to GitHub Pages.

Keep documentation honest: only mark a feature Live once its GPUI interaction is exercised. Core-only features may have Rust API examples but **must not show a fake live Grid**. The site is independent and is not affiliated with AG Grid.
