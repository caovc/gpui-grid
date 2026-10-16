#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
FONTS="$ROOT/crates/gpui-grid-web/fonts"
mkdir -p "$FONTS" "$ROOT/web/pkg"
# Pin upstream font assets to one reviewed commit; no proprietary assets in git.
REV=f8cf0bd964f9ef2e5b7be79f3a8fc391371feb4a
for font in Inter-Regular.ttf IBMPlexSans-Regular.ttf JetBrainsMono-Regular.ttf; do
  if [[ ! -s "$FONTS/$font" ]]; then
    echo "Fetching web font $font from GPUI Kit ($REV)"
    curl --fail --location --retry 3 --silent --show-error \
      "https://raw.githubusercontent.com/longbridge/gpui-kit/$REV/crates/story-web/fonts/$font" \
      -o "$FONTS/$font"
  fi
done
PROFILE=debug
if [[ "${1:-}" == "--release" ]]; then
  PROFILE=release
  cargo rustc -p gpui-grid-web --target wasm32-unknown-unknown --release -- -C link-arg=-zstack-size=8388608
else
  cargo rustc -p gpui-grid-web --target wasm32-unknown-unknown -- -C link-arg=-zstack-size=8388608
fi
WASM="$ROOT/target/wasm32-unknown-unknown/$PROFILE/gpui_grid_web.wasm"
test -s "$WASM"
wasm-bindgen "$WASM" --out-dir "$ROOT/web/pkg" --target web --no-typescript
test -s "$ROOT/web/pkg/gpui_grid_web_bg.wasm"
echo 'Compiled real GPUI Rust view into web/pkg/gpui_grid_web_bg.wasm'
