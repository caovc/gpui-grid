# Contributing to GPUI Grid

Thank you for helping develop this independent Rust/GPUI data-grid. This project aims for enterprise-style interactions but is **not affiliated with AG Grid**.

## Read first

- [AGENTS.md](AGENTS.md): binding rules for human and automated contributors
- [Architecture](docs/ARCHITECTURE.md): Core/native/WASM boundaries and event flow
- [Feature matrix](docs/FEATURES.md): actually supported vs Core-only/planned
- [Roadmap](docs/ROADMAP.md): Issues and dependency order

## Set up

Use a Rust toolchain compatible with the `gpui-kit = 0.7.3` workspace and installed platform GPUI prerequisites. Native build checks are run on macOS GitHub-hosted runners.

```sh
cargo test -p gpui-grid-core --all-targets
cargo check -p gpui-grid -p gpui-grid-native
# Real browser build, not a JavaScript mock:
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.121
./scripts/build-wasm.sh --release
npm --prefix web test
npm --prefix web run build
npm --prefix web run dev
```

Browser E2E depends on Playwright Chromium installed with `npm --prefix web install` and `./web/node_modules/.bin/playwright install --with-deps chromium`, and supports a SwiftShader software GPU configuration in CI.

## Pull requests

Create `feat/<issue>-<feature>`, write tests before claiming behavior, update docs and keep PRs focused. For a new grid feature show **which actual GPUI widget interacts with it** (header, popover, editor, row/cell menu, selection overlay), rather than only an API call or top-level sample button. Include native/macOS, wasm32, browser Canvas test evidence, and explicit unimplemented edge cases. Use the repository PR template.

## License and privacy

Do not copy licensed AG Grid Enterprise code or assets. Use synthetic data; never commit secrets or organization datasets. The repository license is MIT; check upstream GPUI/fonts terms separately.
