# Native GPUI component instructions

Read repository root `AGENTS.md` first. Everything in this subtree is **reusable GPUI component code**, not merely a showcase mock.

## UI interaction rules

- Prefer a control rendered **inside the grid** (`TableDelegate::render_th`, `render_td`, context menu, row header, selection overlay). A top-level `GridShowcase` button is fine for a demo, but **does not count as complete native grid UI**.
- Implement real header actions: per-column sort state, filter indicator and operator input, clear filter, column visibility/pinning; put the trigger on the header. Build common `GridHeaderMenu`/filter/editor components for reuse by other applications. Use real GPUI Kit widgets (e.g. `Popover`, `Button`, `InputState`).
- Type-specific filter controls: text contains/equal, numeric equals/>/<, booleans, empty values; convert user text by schema before a `GridModel` mutation. Bad input must preserve previous filter and display error. Avoid silently swallowing `Result`.
- Non-virtual row/column IDs (`CellAddress`) must survive sort/filter/reorder/hide. Display indices may only be used during viewport rendering.
- Handle mouse and keyboard and focus. Buttons embedded in headers must not accidentally trigger parent column sorting, selection or drag; prevent event propagation where required.
- `TableState::refresh(cx)` after delegate model modifications. Never reenter an emitting TableState from an active `TableEvent` callback — defer by stable IDs.
- Read `gpui-kit 0.7.3` API for `TableDelegate::render_th` (custom header content), `Popover::trigger/content`, `TableState::refresh`, and `TableEvent` instead of guessing APIs.
- Keep expensive `GridModel::view()` out of per-cell render loops. Virtualization is a rendering mechanism, not license to materialize millions of elements.
- Maintain compatibility between normal desktop GPUI and `wasm32`: avoid OS-only APIs in reusable component crate.

## Component acceptance

A feature is complete only when its behavior is accessible from the actual reusable grid, has typed Rust model tests, has a shared Rust WASM example and Playwright Canvas input tests, preserves stable IDs, and updates `docs/FEATURES.md`. Use Issue #7/#8/#9 for header interactions.
