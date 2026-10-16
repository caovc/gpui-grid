//! Native entry point; uses the same Rust/GPUI view as the WASM browser entry.
use gpui_grid::GridShowcase;
use gpui_kit::*;

fn main() {
    gpui_kit::application().with_assets(gpui_kit::assets::Assets).run(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            GridShowcase::new(window, cx)
        }).expect("Unable to open GPUI Grid window");
    });
}
