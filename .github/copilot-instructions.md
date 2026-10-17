# GitHub Copilot / agent entry point

Read [repository AGENTS.md](../AGENTS.md) before coding. Then read the relevant subtree `AGENTS.md`, [Architecture](../docs/ARCHITECTURE.md), [Agent workflow](../docs/AGENT_WORKFLOW.md), and [Feature matrix](../docs/FEATURES.md).

**Non-negotiable:** implement interaction in **Rust GPUI grid controls**, not merely toolbar buttons or JavaScript docs UI. Browser demos run the **real compiled GPUI WebAssembly** and must be validated via Chromium Canvas E2E. Core is pure Rust; stable `row_id + column_key`; no AG Grid source/Enterprise code.
