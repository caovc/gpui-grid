# Coding Agent Workflow / Definition of Done

## Start each task

1. Read `AGENTS.md`, the subtree `AGENTS.md`, `docs/ARCHITECTURE.md`, `docs/FEATURES.md`, `docs/ROADMAP.md`, and issue/PR comments.
2. Inspect `git status` / GitHub `main` SHA and all recent Actions runs. Do not assume conversation screenshots or old commits are current.
3. For a GUI feature, sketch the full interaction lifecycle: *where does the user click in the grid?* → editor/popover opens → typed input/operator → validation → Core state/event → virtualized repaint → how do they clear/cancel?
4. Choose a small Issue-sized scope; branch `feat/<issue>-<slug>`. Avoid auto-closing broad Issues with only partial functionality.

## Implementation checklist

- [ ] Rust Core schema and failure semantics clear; tests verify typed values, nulls, invalid columns and atomicity.
- [ ] Real reusable GPUI component control in `gpui-grid`, not only buttons in `GridShowcase`.
- [ ] No event reentrancy, focus loss, incorrect propagation or stale viewport indices.
- [ ] Sorting/filtering/dragging/editing not conflicting; column keys/row IDs stable after sorting and filtering.
- [ ] Same Rust view in native and WASM; page JS only routes/loads WASM.
- [ ] Feature docs demo uses the actual in-grid control, has API snippet, and accurate support label.
- [ ] Core Rust tests + macOS check + actual wasm32 build + Node docs tests.
- [ ] Chromium Playwright genuinely clicks GPUI Canvas/menu/input, checks Rust read-only diagnostics, captures screenshot and console/network errors.
- [ ] Performance measured when relevant; never equate row count with 60 FPS.
- [ ] `docs/FEATURES.md`, user-facing docs, and related Issue updated.
- [ ] GitHub Actions all green before merge, and post-merge Pages run green.

## CI recipes

```sh
cargo test -p gpui-grid-core --all-targets
cargo check -p gpui-grid -p gpui-grid-native # macOS runner
./scripts/build-wasm.sh --release
npm --prefix web test
npm --prefix web run build
npm --prefix web install --no-audit --no-fund
./web/node_modules/.bin/playwright install --with-deps chromium
npm --prefix web run test:e2e
```

Failed `wasm32` compilation often involves upstream WebPlatform API changes or missing `StyledExt` / `FluentBuilder` trait imports. Failed Chromium E2E: inspect GitHub job log, `gpui-browser-smoke-evidence` artifact, screenshot, browser console/network; distinguish misleading fixed click coordinates from actual Rust state failure. Avoid deleting a failing test or switching to a JS mock.

## GitHub / communication

- Link Issue(s) in PR description. Distinguish planned vs shipped vs tested, especially browser E2E.
- CI green on feature branch, then merge with expected SHA to avoid racing commits.
- GitHub Pages publishes after `main` pushes that match its workflow paths. A successful build of WASM is **not** necessarily a successful Pages deployment.
- Announce exactly which checks have succeeded, which remain, and whether the PR merged. Do not promise autonomous background development.
- Update Issue with follow-up tasks not yet completed (IME, mouse drag, performance, accessibility, etc.).

## Current priority for GUI gaps

1. Header controls: reusable `render_th` column menu with sorting and typed filter popover (Issues #7, #8), active indicators and clear operations. Avoid another showcase-only toolbar.
2. Column panel: visibility, move, pin, reset via GPUI UI (Issue #9).
3. Range selection: drag/auto-scroll, multi-row, keyboard/IME edge cases (Issues #5, #15).
4. Clipboard + Undo/Redo (Issue #6), then grouping (Issue #10) and perf (Issue #11).

See `docs/ROADMAP.md` for other future milestones.
