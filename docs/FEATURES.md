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
| Quick filter (Completed / All) | ✅ | ✅ | ✅ |
| Complex typed filters | ✅ | API only | API only |
| Multi-column sort | ✅ | API only | API only |
| Typed edit transactions | ✅ | ✅ via GPUI editor panel (phase 1) | ✅ via same GPUI editor panel (GPU dependent) |
| CSV export | ✅ | API only | API only |
| Grouping / aggregation | ✅ | UI not wired | UI not wired |
| Excel XLSX, Pivot, server-side row model | ❌ | ❌ | ❌ |
| Browser WebAssembly renderer | n/a | n/a | GPUI WebPlatform + wasm-bindgen |

Editor v1: double-click a visible editable cell or select it and click **Edit cell**; the real GPUI InputState opens above the table. Press Enter or click Save to validate and persist by stable row ID; Cancel leaves the value unchanged. Text, finite numbers, null numeric inputs and boolean true/false are supported. The next editor iteration will add in-cell overlays, F2/Escape/Tab/blur configuration and cross-browser IME checks, so Issue #4 remains open until those are validated.

WASM is **showcase-only**, according to upstream GPUI Kit documentation. Passing `cargo check` is not proof of a running WebGPU/WebGL2 surface: check the browser at the deployed site.

The former JS-only interactive mock was removed from the build. It remains accessible in prior Git commits, but **is not included in the published website**.
