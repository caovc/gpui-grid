# GPUI Grid — Architecture and UI interaction contracts

## 1. Goal

Create an independent AG Grid-inspired, open-source Rust data grid for native desktop apps, with a **real GPUI WebAssembly** feature documentation site. Styling and behavior aim at a familiar professional grid; completeness is incremental and must be measured. GPUI Kit browser backend is experimental/showcase-only.

## 2. Layers and ownership

```text
                           GPUI Grid application
                             (native / WASM)
                                     |
          ---------------------------------------------------------
          |                                                       |
   GPUI native renderer                                      WebPlatform
   crates/gpui-grid                                         gpui-grid-web
          |                                                       |
          +---------------- shared GridShowcase -------------------+
          |
   GridDelegate + GPUI Kit virtualized DataTable
          |  render_th/header interaction, render_td/editor/selection
          |  TableState + stable presentation columns
          v
   gpui-grid-core (pure Rust, no GPUI dependency)
   GridModel + GridState + GridApi + typed events
   Rows(Vec<Value>), schema Columns, Filter/Sort/Edit,
   CellRange/SelectionRect, CSV/group APIs
          |
   Deterministic sample_orders() or application-provided data
```

Only the outer docs website is HTML/JS. It describes capabilities, switches demo `?feature=` routes and launches WASM. It never draws grid rows.

### Core model (source: `crates/gpui-grid-core/src/`)

`GridModel` owns `Vec<Row>` and an index of stable `Row.id: u64`, `Column.key: String`, and `GridState`. Row cells follow **storage schema order**; displayed columns have independent state (`visible/width/pin/order`). Typed `Value`: `Text`, `Number`, `Boolean`, `Empty`.

Mutations: `set_sorts`, `set_filters`, `set_quick_filter`, `apply_edits`, `select_cell`, `select_range`, `move_column`, `move_visible_column`, `set_column_pin`, `reset_column_state`, `set_column_visible`, `set_column_width`. Validations emit typed `GridEvent`s; failed atomic edits must not partially write rows.

Selection: `CellRange { anchor: CellAddress, focus: CellAddress }`; projection to `SelectionRect` is computed against current visible rows and columns. If an endpoint is filtered/hidden, preserve identity but avoid wrong-cell highlights.

### Native UI (source: `crates/gpui-grid/src/`)

`GridDelegate` implements GPUI Kit `TableDelegate`. GPUI Kit 0.7.3 provides virtualization, base header sorting, row/cell selection, column resize/move and pin-left. `render_td` paints typed values, badge styling, selection highlight and the real in-cell `InputState`; `render_th` can render custom header cells. `GridShowcase` is the shared Rust view launched by native and `gpui-grid-web`.

The existing showcase has example-only toolbar buttons (Completed/All, revenue filtering, etc.). **Those buttons do not constitute an in-grid filter UI.** Next reusable features must put interactive controls into the header and/or context menus.

```text
Header cell (GridDelegate::render_th)
  ├─ typed column name / sort indicator
  └─ GPUI native menu trigger
        └─ GPUI Popover with native InputState / operator buttons
              ├─ sort asc/desc/clear (Core Sort + refresh)
              ├─ filter contains/equals or numeric >/</= (Core Filter + refresh)
              ├─ active-filter visual indicator
              ├─ clear filter / hide column
              └─ move left/right, pin-left/unpin, show all, reset columns
Data cell (render_td)
  ├─ formatted typed value
  ├─ range selection paint (stable IDs)
  └─ active InputState editor (validated Edit transaction)
```

**Event safety:** `render_th` and `render_td` are invoked while GPUI TableState is rendering. Do not synchronously reenter that same entity from a TableEvent callback. Capture `CellAddress`/`column_key`, defer TableState updates when necessary, then `refresh(cx)`. A header button must not bubble a click into parent header sorting/dragging. Confirm GPUI event APIs in pinned dependency; never assume a JS-style DOM event model.

### Feature support states (update `docs/FEATURES.md`)

1. **Core** — typed operations tested without GUI.
2. **Native GPUI component** — user interacts with an actual reusable grid control. A showcase-only top-toolbar action does not count as native in-grid support.
3. **WASM GPUI** — exactly the same UI compiled to `wasm32-unknown-unknown` and browser behavior tested on actual Canvas.
4. **Docs** — real example and truthful status; never pretend a missing feature works.

## 3. Build and release

`cargo test -p gpui-grid-core --all-targets`; native macOS `cargo check -p gpui-grid -p gpui-grid-native`; `./scripts/build-wasm.sh --release`; `npm --prefix web test`, `npm --prefix web run build`, `npm --prefix web run test:e2e`. GitHub Actions checks all four. Pages workflow packages and deploys the real WASM binary; successful deployment is distinct from GPU/keyboard E2E.

Caveats: `gpui-kit = 0.7.3`, `wasm_thread` workaround in WASM script; upstream GPUI WebPlatform remains experimental. Do not claim full AG Grid Enterprise parity or million-row performance without measurements.
