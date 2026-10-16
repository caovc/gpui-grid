# Browser E2E checks for the **real GPUI WASM**

Issue: [#2](https://github.com/caovc/gpui-grid/issues/2).

The test uses the genuine `gpui_grid_web_bg.wasm` built from the Rust `GridShowcase` view. The page does not render a JavaScript/DOM replacement grid.

## Local run

```bash
rustup target add wasm32-unknown-unknown
./scripts/build-wasm.sh --release
npm --prefix web install
npm --prefix web run build
./web/node_modules/.bin/playwright install --with-deps chromium
npm --prefix web run test:e2e
```

The script starts a local Node static server on `127.0.0.1:4173`, then:

1. Loads `web/dist` and requires the browser to fetch `gpui_grid_web_bg.wasm` with the correct MIME type and HTTP 200.
2. Waits for the actual GPUI WebPlatform canvas and first rendered frame; screenshots it and checks for non-blank painted pixels (not just a canvas tag).
3. Clicks the **GPUI-rendered** `Completed only` and `All rows` buttons using Canvas pointer events.
4. Checks the resulting row count through `preview_visible_rows()`, a **read-only Rust diagnostic export** updated only by those Rust GPUI button handlers. The test never calls a filtering method from JavaScript.
5. Stores screenshots, console errors, WASM network responses, pixel statistics, and traces as GitHub Actions artifacts.

## CI software GPU behavior

The CI runner has no dedicated GPU. The Chromium job explicitly opts into software GL with `--use-gl=angle --use-angle=swiftshader --enable-unsafe-swiftshader --ignore-gpu-blocklist`. This *lower-security* SwiftShader setting is suitable only for our trusted local test page; it is not recommended for normal browsing. GPUI's browser graphics backend will select WebGPU if available and otherwise attempt WebGL2.

GPUI WebPlatform is experimental. A Rust build and WASM load passing **do not** establish that the canvas rendered correctly. If GPU initialization fails, this test is intentionally **red** with screenshots and console logs rather than silently falling back to a JavaScript table or skipping the test.

The Playwright result validates Chromium with the specified flags on the GitHub-hosted Ubuntu runner. Safari, Firefox, mobile input, IME, and screen reader behavior are not covered; track separately under [#15](https://github.com/caovc/gpui-grid/issues/15).
