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
| Quick filter (Completed / All) | ✅ | ✅ | ✅ |
| Complex typed filters | ✅ | API only | API only |
| Multi-column sort | ✅ | API only | API only |
| Typed edit transactions | ✅ | ✅ GPUI in-cell InputState (phase 2) | ✅ shared GPUI in-cell InputState (GPU dependent) |
| CSV export | ✅ | API only | API only |
| Grouping / aggregation | ✅ | UI not wired | UI not wired |
| Excel XLSX, Pivot, server-side row model | ❌ | ❌ | ❌ |
| Browser WebAssembly renderer | n/a | n/a | GPUI WebPlatform + wasm-bindgen |

Range selection phase 1 (#5): click a cell to anchor the selection, Shift+click another to extend an inclusive rectangle. The Rust model retains stable row IDs and column keys, computes rectangle coordinates against the current sort/filter/column order, and hides highlighting rather than pointing at the wrong records when an endpoint is filtered out. Selection state drives GPUI cell backgrounds on desktop and in real WASM; drag gestures, multi-row selection, Ctrl+A and Shift+arrows are planned follow-ups.

Editor v2: double-click a visible editable cell or select one and press **F2** / click **Edit cell**; a real Rust GPUI InputState is painted **inside the table cell**. Press Enter to commit; Tab/Shift+Tab commit and move to the next/previous editable cell; Esc or Cancel discard the draft. Save and Boolean controls remain available in the toolbar. Errors keep the editor active. We intentionally **do not auto-commit on blur**, avoiding IME composition loss. Numeric/boolean parsing and readonly enforcement remain in the Rust core. IME and platform-specific keyboard behavior still require broader coverage, so #4 remains open until fully verified.

WASM is **showcase-only**, according to upstream GPUI Kit documentation. Passing `cargo check` is not proof of a running WebGPU/WebGL2 surface: check the browser at the deployed site.

The former JS-only interactive mock was removed from the build. It remains accessible in prior Git commits, but **is not included in the published website**.
