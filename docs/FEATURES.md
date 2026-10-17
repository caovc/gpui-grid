# GPUI Grid · Feature truth table

One view is used for **desktop native GPUI and the browser's GPUI WebPlatform**. The Web viewer no longer uses a simulated JavaScript data grid.

| Feature | Rust core | GPUI native view | Browser WASM (same view) |
|---|---|---|---|
| Typed row and column model | ✅ | ✅ | ✅ |
| GridState / GridOptions / GridApi + typed change events | ✅ | foundation API | foundation API |
| Stable focus row ID and presentation column state | ✅ | model integrated, UI TODO | model integrated, UI TODO |
| 12,480 sample rows with virtualized DataTable | ✅ | ✅ | ✅ (browser GPU dependent) |
| Column resize, move, pinned first column | n/a | ✅ GPUI Kit | ✅ GPUI Kit |
| Sort by column | ✅ | ✅ | ✅ |
| Select cell and row | n/a | ✅ | ✅ |
| Stable-ID rectangular selection (Shift+click, viewport highlight) | ✅ | ✅ initial implementation | ✅ same GPUI Canvas (browser GPU dependent) |
| Quick filter (global text search) | ✅ | Core API only; no demo toolbar | Core API only; no demo toolbar |
| Complex typed filters | ✅ | ✅ GPUI header Contains/Equals and numeric comparisons | ✅ same GPUI header menu |
| Multi-column sort | ✅ | ✅ Add Multi-sort in native column menu | ✅ same GPUI column menu |
| Typed edit transactions | ✅ | ✅ GPUI in-cell InputState (phase 2) | ✅ shared GPUI in-cell InputState (GPU dependent) |
| CSV export | ✅ | API only | API only |
| Grouping / aggregation | ✅ | UI not wired | UI not wired |
| Excel XLSX, Pivot, server-side row model | ❌ | ❌ | ❌ |
| Browser WebAssembly renderer | n/a | n/a | GPUI WebPlatform + wasm-bindgen |
| Feature-specific docs examples (sorting, filtering, selection, editing, columns, virtualization) | Shared `GridModel` | ✅ same `GridShowcase` modes | ✅ real WASM mode selected via `run(feature)` |
| In-grid column header menu: asc/desc/multi-sort/clear | ✅ | ✅ GPUI Popover on each header | ✅ same native GPUI header (GPU dependent) |
| In-grid typed column filter: text contains/equals, numeric >/</=, Boolean | ✅ | ✅ GPUI Popover + InputState | ✅ same native GPUI header (GPU dependent) |
| In-grid Set Filter (low cardinality): checkboxes, select all, mini-search | ✅ typed OneOf (OR/AND) | 🔄 PR #28, GPUI Checkbox menu | 🔄 pending real WASM E2E |
| In-grid column hide / restore visibility | ✅ | ✅ GPUI Popover, stable keys | ✅ same native GPUI header (GPU dependent) |
| In-grid move left/right, pin-left/unpin, reset presentation | ✅ stable visible-index movement and reset events | ✅ native header actions, pinned boundary guard | ✅ same shared GPUI menu (GPU dependent; E2E covers actions) |

Range selection phase 2 (#5): Shift+arrow keys extend the rectangle from the active GPUI cell; Ctrl/Cmd+A selects all currently visible rows and columns using stable range endpoints rather than allocating one object per selected cell. While a cell is being edited, these shortcuts do not replace the input's normal text editing shortcuts. The same model and keybindings run on desktop and genuine GPUI WASM.

Range selection phase 1 (#5): click a cell to anchor the selection, Shift+click another to extend an inclusive rectangle. The Rust model retains stable row IDs and column keys, computes rectangle coordinates against the current sort/filter/column order, and hides highlighting rather than pointing at the wrong records when an endpoint is filtered out. Selection state drives GPUI cell backgrounds on desktop and in real WASM; drag gestures, multi-row selection and advanced keyboard navigation remain follow-ups.

Editor v2: double-click a visible editable cell or select one and press **F2**; a real Rust GPUI InputState is painted **inside the table cell**. Press Enter to commit; Tab/Shift+Tab commit and move to the next/previous editable cell; Esc discards the draft. There are no showcase-level action buttons. Errors keep the editor active. We intentionally **do not auto-commit on blur**, avoiding IME composition loss. Numeric/boolean parsing and readonly enforcement remain in the Rust core. IME and platform-specific keyboard behavior still require broader coverage, so #4 remains open until fully verified.

Column management (#9 next milestone): the real GPUI header menu now exposes Move Left/Right (disabled at edges or pin boundaries), Pin Left/Unpin, Hide Column, Show All and Reset. Core `move_visible_column` translates displayed positions into full persisted column-state indexes so hidden columns do not corrupt moves. `set_column_pin` keeps left-pinned columns contiguous and emits `ColumnPinChanged`; `reset_column_state` restores schema order, initial pins, width and visibility without clearing filters, sorting, focus or edits. Right-pinning is intentionally **not supported yet** by GPUI Kit 0.7.3 adapter; a searchable column tool panel, resizing persistence and serialized column state remain Issue #9 follow-ups.

Native column-header menus (#7/#8/#9 initial milestone): click the **⋯** in a grid column header (not the showcase toolbar). A GPUI Popover contains sort asc/desc, add to multi-sort, clear sort, typed filter InputState (press Enter for Text Contains / Number Equals, or click an operator button) and operator buttons (text Contains/Equals, numeric >/< /=, Boolean True/False), clear-this-column filter, hide column and restore all columns. All controls call the authoritative typed Rust GridModel, preserve other columns' filter conditions, and refresh GPUI DataTable; invalid numeric values preserve previous filter and show an error. Future work includes full operator dropdowns, set/multi-condition filters, saved column state, keyboard/focus accessibility and comprehensive browser interaction tests.

The [feature documentation website](../web/) uses a separate HTML shell for navigation, API reference, code snippets, and search. **All Live Grid demos run a real Rust GPUI Canvas**. Planned and Core-only pages deliberately show no fake interactive grid. Feature modes are selectable by deep link `?feature=sorting` etc., and grid operations live in the native column header/cell UI rather than an extra demo toolbar. The Virtualization page constructs 50,000 local rows; this is a demonstration, not a performance guarantee.

WASM is **showcase-only**, according to upstream GPUI Kit documentation. Passing `cargo check` is not proof of a running WebGPU/WebGL2 surface: check the browser at the deployed site.

The former JS-only interactive mock was removed from the build. It remains accessible in prior Git commits, but **is not included in the published website**.
