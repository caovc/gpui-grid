# Feature parity / 功能矩阵

This project is **AG Grid-inspired**; no AG Grid Enterprise code or licensed features have been copied. It is not affiliated with or endorsed by AG Grid.

Two implementations have different maturity levels:
- **Native GPUI**: reusable Rust library in `crates/gpui-grid/` + independently tested `gpui-grid-core` engine; delegates rendering and virtualization to GPUI Kit DataTable 0.7.3.
- **Browser showcase**: standalone HTML/CSS/JS that reproduces interactions for review. It is **not GPUI running in WebAssembly** and does not share runtime code with the Rust renderer.

| Function | Browser demo | Native GPUI component |
|---|---|---|
| Quartz-inspired light/dark visual system | ✅ | Basic native skin |
| Virtualized rows / 100,000-row dataset | ✅ | ✅ via GPUI Kit DataTable |
| Column resize, reorder, pin-left | ✅ | ✅ via GPUI Kit DataTable |
| Sorting / multi-column sorting | ✅ | Single-column via DataTable; core supports multi-sort |
| Quick search / field filters | ✅ | Quick filter method + typed core filters |
| Cell focus and arrow navigation | ✅ | ✅ via GPUI Kit DataTable |
| Multi-row checkbox selection | ✅ | Native single/row/cell selection via DataTable |
| Shift-select rectangular range | ✅ | Not yet wired |
| Inline edit / validation / paste | ✅ | Typed edit API; editor UI not yet wired |
| Row grouping / collapse / sums | ✅ | Core group-by; native group UI not yet wired |
| CSV import / export | ✅ | Core CSV export, native import UI not yet wired |
| Context/column menus | ✅ | GPUI Kit baseline context support |
| Responsive preview / design tokens | ✅ | Native resize handled by GPUI |
| Tests | Node built-in test runner | Rust unit tests (run in GitHub Actions) |

## Known limitations

- Browser virtualization is **row-based**, not fully two-dimensional. Native DataTable already virtualizes rows and columns.
- The 100,000-row demo generates **local deterministic data**, not a production remote row model.
- No Excel XLSX, Pivot mode, SSRM, charting, complex aggregation, or accessibility audit is claimed.
- Group collapsing and selection should be tested more deeply before treating the browser demo as production-ready.
- GPUI 0.7.3 is an evolving ecosystem; native CI on macOS validates the adapter against the pinned published APIs. The current environment lacks Rust tooling, so native compilation was not verified locally.
