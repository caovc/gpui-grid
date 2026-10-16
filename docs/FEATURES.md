# GPUI Grid · Feature truth table

One view is used for **desktop native GPUI and the browser's GPUI WebPlatform**. The Web viewer no longer uses a simulated JavaScript data grid.

| Feature | Rust core | GPUI native view | Browser WASM (same view) |
|---|---|---|---|
| Typed row and column model | ✅ | ✅ | ✅ |
| 12,480 sample rows with virtualized DataTable | ✅ | ✅ | ✅ (browser GPU dependent) |
| Column resize, move, pinned first column | n/a | ✅ GPUI Kit | ✅ GPUI Kit |
| Sort by column | ✅ | ✅ | ✅ |
| Select cell and row | n/a | ✅ | ✅ |
| Quick filter (Completed / All) | ✅ | ✅ | ✅ |
| Complex typed filters | ✅ | API only | API only |
| Multi-column sort | ✅ | API only | API only |
| Typed edit transactions | ✅ | API only | API only |
| CSV export | ✅ | API only | API only |
| Grouping / aggregation | ✅ | UI not wired | UI not wired |
| Excel XLSX, Pivot, server-side row model | ❌ | ❌ | ❌ |
| Browser WebAssembly renderer | n/a | n/a | GPUI WebPlatform + wasm-bindgen |

WASM is **showcase-only**, according to upstream GPUI Kit documentation. Passing `cargo check` is not proof of a running WebGPU/WebGL2 surface: check the browser at the deployed site.

The former JS-only interactive mock was removed from the build. It remains accessible in prior Git commits, but **is not included in the published website**.
