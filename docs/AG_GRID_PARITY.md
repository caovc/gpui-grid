# AG Grid-like UI / Function Parity Roadmap

**Visual target: AG Grid Quartz (light first).** The implementation uses *independent* Rust/GPUI code and independently chosen visual tokens. AG Grid is a separate product; Enterprise components and source are not bundled or copied.

The goal is a highly familiar, production-usable component, **not a claim that all AG Grid Enterprise APIs are currently supported**.

## Acceptance hierarchy

A feature is not complete when its Core API exists. Every row below requires **real GPUI UI input** on native desktop and the *same UI* rendered through genuine GPUI WebAssembly, followed by Canvas E2E. Only the documentation chrome is HTML/CSS.

| Feature or UI | Expected interaction | Current status |
| --- | --- | --- |
| Quartz surface/density | restrained borders, neutral header, readable typography, blue selected range, consistent editor/hover tokens | Partial: `quartz.rs` light color tokens, continue row-height and dark-mode work |
| Header and column menu | menu on each column, filter indicator, sort priority, hover and focus states | Initial GPUI header Popover |
| Sort | ascending, descending, clear, multi-sort indicator, Shift-click multi-sort | Native Popover commands; Shift-click TODO |
| Text/Number/Boolean filtering | operator selection, typed input validation, clear one column | Native GPUI InputState + operators |
| **Set Filter** | mini-search, (Select All), individual checkboxes, count, AND across columns, OR within column | In progress #7; low-cardinality native prototype |
| Set Filter advanced | virtualized options, data-set refresh, Apply/Reset configuration, blanks, key/value formatting, self-filter-aware counts | Planned #7 |
| Column Tool Panel | choose/show/hide/reorder/pin/reset, floating side panel | Only column menu hide/show available; #9 |
| Cell editing | inline editor F2/double click, Enter, Esc, Tab, typed validator/IME | Partial; #4 |
| Selection | selected rect, Shift+click/arrow, Ctrl/Cmd+A | Partial; dragging/autoscroll TODO #5 |
| Clipboard | TSV copy/paste, batch atomic edit, undo/redo | Planned #6 |
| Grouping and aggregation | group row headers, expand/collapse, count/sum/avg, pinned totals | Core only; #10 |
| Remote row model | async batches, loading/error/cache and server-side filter/sort | Planned #12 |
| Docs | left feature tree, text + Rust-highlighted sample, **actual** native GPUI WASM live example | Shipped; continue visual detail parity |

## Quartz-oriented component measurements (targets)

- Header row: approximately **40–48px**; data row: **36–42px**; fonts **12–14px** depending on density preset.
- Header: near-white #F8FAFB, neutral text #25313D and subtle grid border #E2E7EB.
- Interactive emphasis: restrained medium blue (#237FDA), selection tint (#E0EFFF), bounded focus ring.
- Column menu: compact white GPUI Popover, subtle 1px dividers, native InputState/Checkbox controls, values count, bounded scrolling.
- States: hover, active, disabled, pending validation, selected cell/row/range, filtered column and sorted priority must be visually distinct.
- Every in-grid widget supports mouse, keyboard, logical focus, and remains stable under sort/resize/pin/virtual scroll.
- Dark Quartz is a future separate palette, not automatically enabled by the HTML docs theme switch.

## Set Filter contract

The starting model is a vector of **typed Value** choices. A single `OneOf(Vec<Value>)` condition represents OR over selected values, while different column filter conditions combine by AND. Empty `OneOf([])` means show **zero** rows. Removing the filter altogether means show **all** rows, matching spreadsheet expectations.

The GPUI column-header menu shows the Set Filter only for low-cardinality columns (12 or fewer distinct values). High-cardinality data continues to offer text or numeric condition filters. A mini-search changes visible checkboxes, not data results, until selection is changed; Enter selects mini-search matches. Checkbox actions immediately commit Core state and repaint the GPUI DataTable.

This is an initial implementation and not a full replacement for the proprietary AG Grid Enterprise Set Filter. Reference UX docs: https://www.ag-grid.com/javascript-data-grid/filter-set/ and https://www.ag-grid.com/javascript-data-grid/filter-set-mini-filter/.

## Agent review checklist

1. Open relevant AG Grid public documentation and visually compare the real GPUI Canvas, not just the docs shell.
2. Write a reusable native grid widget; avoid showcase-level feature toolbars.
3. Verify typed state matches rendered visual options across sort/filter/edit and after clearing.
4. Add Canvas E2E with screenshots + immutable wasm-bindgen probes; *never* mutate Rust grid from browser JS.
5. Update `docs/FEATURES.md`, relevant feature catalog descriptions, and this parity matrix with precise status.
