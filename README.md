# GPUI Grid · Rust GPUI + WebAssembly

An independent, AG Grid-inspired data-grid component. **The website now runs the actual Rust/GPUI component compiled to WASM**, rather than a separate JavaScript table.

The GPUI Kit browser backend is currently **showcase-only / experimental**; do not assume full production browser, accessibility or mobile input support.

📚 [按功能浏览在线文档 / Feature-based live docs](https://caovc.github.io/gpui-grid/?feature=overview) · 🧪 [Real GPUI WASM browser E2E tests](docs/BROWSER_E2E.md) · 📋 [Development roadmap / 后续开发路线图](docs/ROADMAP.md) · [Tracking epic #1](https://github.com/caovc/gpui-grid/issues/1) · [Current issues](https://github.com/caovc/gpui-grid/issues)

## Source layout

```text
crates/gpui-grid-core/          Pure Rust typed data model (filter, sort, edit, group, CSV)
crates/gpui-grid/               Native GPUI Kit DataTable delegate + shared GridShowcase view
crates/gpui-grid-web/           wasm-bindgen exports: real GPUI WebPlatform entry
examples/native/                Native desktop launches exactly the shared GridShowcase
scripts/build-wasm.sh           Rust wasm32 compile + wasm-bindgen; downloads OFL fonts
web/                            AG Grid-inspired feature documentation + Rust WASM gallery host; NO JS grid renderer
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

The upstream `wasm_thread` 0.3.3 dependency currently requires an unstable Rust intrinsic. Like the GPUI Kit upstream WASM checks, the script scopes `RUSTC_BOOTSTRAP=1` to the **WASM cargo rustc command only**. This is a temporary compiler compatibility workaround, not a promise of stable Rust support.

`gpui-grid-web` is a real `cdylib` crate. It builds the same `GridShowcase` Rust `Render` tree that the native desktop executable starts. The JS host only loads `gpui_grid_web.js`, initializes `gpui_grid_web_bg.wasm`, and calls Rust's exported `run()`; GPUI WebPlatform paints the actual UI into a canvas via WebGPU/WebGL2. Build **fails** if the WASM artifact is missing, rather than silently serving a mock.

**Toolbar-free component preview:** GridShowcase no longer renders feature-specific action buttons above the grid. Use each native GPUI header ⋯ menu, cell editing and keyboard selection instead. The docs layout fills available screen width/height and locally highlights Rust source without external CDNs. Global quick filter remains a Core-only API.

**In-grid interaction milestone:** Native GPUI header `⋯` menus now use `TableDelegate::render_th` + GPUI `Popover`/`InputState` for sorting, typed filtering and column hide/show. This is a **reusable component interaction**, not a showcase-only button. The interactive filter can preserve filters applied on other columns; invalid numeric input is rejected before changing Core state. Details: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), [docs/FEATURES.md](docs/FEATURES.md).

**Live documentation examples:** The site has feature-specific genuine Rust GPUI demos for Overview, Sorting, Filtering, Cell Selection, Cell Editing, Columns, and Virtualization (50k local rows). The page URL accepts `?feature=sorting`, `?feature=filtering`, `?feature=selection`, etc. The `web/catalog.js` module contains *documentation only*; it never stores data rows or simulates a grid. The viewer passes the selected feature to the `gpui-grid-web` Rust WebAssembly entry point, which creates the same `GridShowcase` used by native GPUI desktop apps. Advanced Grouping, Clipboard, Async RowModel, and Pivot pages are explicitly marked Core-only or Planned. See [feature matrix](docs/FEATURES.md) and [website architecture](docs/WEBSITE.md).

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
