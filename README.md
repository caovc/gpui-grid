# GPUI Grid · Rust GPUI + WebAssembly

An independent, AG Grid-inspired data-grid component. **The website now runs the actual Rust/GPUI component compiled to WASM**, rather than a separate JavaScript table.

The GPUI Kit browser backend is currently **showcase-only / experimental**; do not assume full production browser, accessibility or mobile input support.

## Source layout

```text
crates/gpui-grid-core/          Pure Rust typed data model (filter, sort, edit, group, CSV)
crates/gpui-grid/               Native GPUI Kit DataTable delegate + shared GridShowcase view
crates/gpui-grid-web/           wasm-bindgen exports: real GPUI WebPlatform entry
examples/native/                Native desktop launches exactly the shared GridShowcase
scripts/build-wasm.sh           Rust wasm32 compile + wasm-bindgen; downloads OFL fonts
web/                            HTML frame/loader + browser chrome, NO JS grid renderer
.github/workflows/ci.yml        Rust core, native GPUI, real wasm32 CI builds
.github/workflows/pages.yml     Builds real WASM and deploys to GitHub Pages
```

## Build the same Rust view for both platforms

```bash
# Native GPUI, on a machine with macOS/Windows/Linux GPUI prerequisites:
cargo run -p gpui-grid-native

# Browser target:
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.121
./scripts/build-wasm.sh --release
npm --prefix web test
npm --prefix web run build
npm --prefix web run dev
# Visit http://127.0.0.1:4173/
```

`gpui-grid-web` is a real `cdylib` crate. It builds the same `GridShowcase` Rust `Render` tree that the native desktop executable starts. The JS host only loads `gpui_grid_web.js`, initializes `gpui_grid_web_bg.wasm`, and calls Rust's exported `run()`; GPUI WebPlatform paints the actual UI into a canvas via WebGPU/WebGL2. Build **fails** if the WASM artifact is missing, rather than silently serving a mock.

**Features actually wired to this shared GPUI view:** virtual scrolling via GPUI Kit DataTable, column drag/resize, native GPUI sorting/selection, and quick filtering using “Completed only” / “All rows”. `gpui-grid-core` also supports richer filtering, editing, grouping and CSV exports, but some of these features still lack native/WASM UI and should not be claimed to work in the browser. See [feature matrix](docs/FEATURES.md).

### Browser limitations

- GPUI Kit 0.7.3 WASM support is for component showcases; WebGPU/WebGL2 availability varies by browser.
- Browser startup requires a graphics backend and font initialization; WebAssembly may take several seconds to download/start.
- Current wasm32 build uses an HTML outer shell and **a real GPUI canvas in an iframe**. HTML chrome is *not* the grid itself.
- No JavaScript fallback table is provided. Failures are surfaced to the user with an error and console details.
- The first compilation uses significant CI resources; GitHub Actions caches Rust dependencies.

### Font provenance

The build downloads the publicly licensed, subsetted Inter / IBM Plex Sans / JetBrains Mono fonts from [GPUI Kit story-web](https://github.com/longbridge/gpui-kit/tree/main/crates/story-web/fonts) at a pinned revision. Font binaries are not committed to this repo; bundled subset fonts are only included in the generated WebAssembly site. Respect the upstream font licenses when redistributing builds.

## GitHub CI and Pages

- [Repository](https://github.com/caovc/gpui-grid)
- [Actions](https://github.com/caovc/gpui-grid/actions)
- [Pages URL](https://caovc.github.io/gpui-grid/) **only after deployment succeeds**

The previous Actions failures were a missing `StyledExt` import for `font_semibold` in the macOS example and GitHub Pages not yet enabled (REST 404). The native view now uses `font_bold()`; CI checks both the shared GPUI view and the actual `wasm32-unknown-unknown` target.

**One-time Pages setup:** Open [Settings → Pages](https://github.com/caovc/gpui-grid/settings/pages), choose **Source: GitHub Actions**, then rerun the Pages workflow. GitHub App permissions do not permit enabling Pages automatically. Until enabled, Pages CI builds and uploads a testable artifact and emits a warning, without falsely claiming a successful site deployment.

## License

MIT; independent implementation, no AG Grid source code or Enterprise license included. GPUI Grid is not affiliated with AG Grid.
