# GPUI Grid — Agent Development Instructions

> **This file is the first document any coding agent must read before editing this repository.** Applies recursively to every directory unless a nested `AGENTS.md` adds specific rules. These are project instructions, not evidence of implemented features.

## Mission and non-negotiable constraints

Build an independent, open-source, **AG Grid-inspired native Rust data-grid** using **GPUI + GPUI Kit**. Match common enterprise grid interaction patterns where feasible, but do **not copy AG Grid source code, licensed Enterprise modules, branding, or proprietary assets**.

- **Single UI implementation:** `gpui-grid` is the source of native widgets and layout. The **same Rust `GridShowcase` / `GridDelegate`** is rendered in desktop GPUI and the browser through GPUI Kit's experimental **WebPlatform** compiled to `wasm32-unknown-unknown`. Do **not** replace the grid with HTML, Canvas2D JavaScript, React, AG Grid, or a JS-only mock.
- **Documentation site is presentation only:** `web/` may implement feature routes, content search, Rust source snippets, page tabs, and WASM loader; **all interactive data-grid pixels and state mutations must come from Rust GPUI**. `web/catalog.js` is documentation metadata, never a row-data store.
- **True interaction requires an in-grid control:** An API method or a button in `GridShowcase` is **not yet** an AG Grid-like UI feature. Users should interact directly with real GPUI **column headers, column menus/popovers, filter controls, editors, range-selection handles, row/group headers, and context menus** where appropriate. Keep the showcase toolbar for test affordances only; ship reusable behavior in `GridDelegate` or associated GPUI components.
- **Stable identity:** Treat `row_id: u64` + `column_key: String` as persistent cell identity. Never persist viewport row/column indices across sort, filter, hide/reorder, or virtualization; indices are ephemeral display coordinates. For ranges, persist two `CellAddress` endpoints, not millions of selected cells.
- **Core/UI separation:** `gpui-grid-core` stays platform-independent. Typed validation, atomic edits, sort/filter/group/query, events and selection belong there; GPUI widgets, focus, menu interactions and painting belong in `gpui-grid`.
- **Test honestly:** `cargo check` or a successful Pages deploy does **not** prove Canvas drawing or browser input. Playwright runs the **real GPUI WASM** and must verify actual event-driven Rust state. Do not remove failing assertions, silently disable E2E, introduce a JS fallback, or declare a planned feature done.
- **Upstream compatibility:** Current manifests pin `gpui-kit = "=0.7.3"`. Verify API signatures against the resolved dependency; upstream `main` might differ. WASM compilation currently scopes `RUSTC_BOOTSTRAP=1` to its build command due to an upstream `wasm_thread` issue. Never enable unstable flags for the whole workspace without documenting the cause.
- **No user-specific / internal data in repo:** Use deterministic synthetic `sample_orders` data only; no production EDC/clinical/organizational details, credentials or access tokens.

## Repository map

| Path | Purpose |
|---|---|
| `crates/gpui-grid-core/` | Typed `GridModel`, `GridState`, `GridApi`, `GridEvent`, `Filter`, `Sort`, `Edit`, `CellRange` |
| `crates/gpui-grid/` | GPUI Kit `TableDelegate`, virtualized `DataTable`, renderers, header/editor/menu components, shared `GridShowcase` |
| `crates/gpui-grid-web/` | wasm-bindgen exports and GPUI Kit `WebPlatform` bootstrap |
| `examples/native/` | Native executable using shared Rust view |
| `web/` | Feature docs/navigation and iframe loader; only embeds actual WASM Canvas |
| `web/e2e/` | Chromium Playwright real-WASM GPU/keyboard/pointer integration tests |
| `scripts/build-wasm.sh` | Real GPUI wasm32 build + wasm-bindgen generation |
| `.github/workflows/` | Rust Core tests, macOS check, real WASM build, Chromium E2E, Pages publish |
| `docs/FEATURES.md` | **Truthful** core/native/WASM support matrix; never overclaim |
| `docs/ROADMAP.md` | Sequenced roadmap with Issue links |
| `docs/ARCHITECTURE.md` | Modules, data/event flow, native/WASM split, interaction design |
| `docs/AGENT_WORKFLOW.md` | Implementation, CI/debug, acceptance and PR checklist |
| `CONTRIBUTING.md` | Human contribution and local development basics |

Read additional `crates/gpui-grid/AGENTS.md` or `web/AGENTS.md` for those subtrees.

## Workflow for every functional change

1. **Discover**: inspect `main`, `docs/FEATURES.md`, related GitHub Issue(s), relevant Rust modules and upstream GPUI Kit API. Read recent Actions results rather than assuming the last run is green.
2. **Scope**: describe separately: Core feature; **reusable GPUI control**; feature-specific Rust showcase/demo; doc page; native+WASM tests. If a header filter is requested, adding a toolbar-only shortcut is insufficient.
3. **Branch**: `feat/<issue>-<slug>`; make reviewable, atomic commits. Avoid large unrelated formatting churn.
4. **Implement**: type-safe Core contract → GPUI in-grid interaction + proper focus/keyboard handling → genuine WASM feature demo → browser E2E → docs.
5. **Tests**: see commands below. For controls, verify pointer, keyboard, cancellation, error, focus and model truth under sort/filter/hide. Screenshots in browser tests are evidence; read-only WASM diagnostics never mutate Rust rows.
6. **Publish**: open PR; only merge after Core, macOS, WASM and Chromium checks succeed; inspect GitHub Actions logs if not. On merge, verify Pages actually deployed the new SHA. Add precise completion/remaining notes to the Issue.
7. **Truth table**: update `docs/FEATURES.md` for **Core-only**, **GPUI UI**, and **Browser WASM** distinctly. Never mark an incomplete control as 1:1 AG Grid parity.

## Commands and quality gates

```bash
cargo test -p gpui-grid-core --all-targets
cargo check -p gpui-grid-core
# macOS GPUI runner is the canonical native compile gate:
cargo check -p gpui-grid -p gpui-grid-native

rustup target add wasm32-unknown-unknown
# wasm-bindgen CLI is pinned by CI; follow script/README for fonts and toolchain:
./scripts/build-wasm.sh --release
npm --prefix web test
npm --prefix web run build
npm --prefix web install --no-audit --no-fund
./web/node_modules/.bin/playwright install --with-deps chromium
npm --prefix web run test:e2e
```

Do not assume `cargo run` works on headless Linux without a compatible GPU/windowing environment. GitHub Actions `macos-latest` checks native; Ubuntu checks wasm32 and executes Chromium software GPU with captured console/trace/screenshots.

**E2E acceptance example:** click an actual GPUI header filter icon on the Canvas → interact with a **GPUI input/popover** → apply filter → assert `preview_visible_rows()` changed from Rust; check screen pixels changed. A JS call to mutate `GridModel` is invalid test evidence.

## Review boundaries

- No async `App` mutation from rendering callbacks without respecting GPUI's borrow/event rules. The `TableDelegate::render_th` / `render_td` runs inside TableState render. For TableEvent handler reentrancy, capture stable values and use `cx.defer` when needed.
- Keep complex query logic outside `render_td`, `render_th`, and scroll callbacks. Prefer cached viewport state and measure workloads before claiming 100k/1M smoothness.
- Document accessibility, IME, clipboard and WebGPU/WebGL2 limits explicitly; GPUI Kit WASM backend is experimental/showcase-only.
- Do not expose extra mutable browser debugging hooks. WASM exports may provide **read-only counters** for test assertions, never JS-side table mutation.
- Prefer `gpui_kit::component::* ` and GPUI-native interaction components; reproduce common AG Grid **behavior**, not copyrighted artwork.

For detailed patterns, consult [Architecture](docs/ARCHITECTURE.md), [AG Grid parity matrix](docs/AG_GRID_PARITY.md) and [Agent Workflow](docs/AGENT_WORKFLOW.md).
