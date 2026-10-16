//! Browser bootstrap for the *same* GPUI Grid view used by the native app.
//! Actual GPUI WebPlatform / WebGPU or WebGL2; not a JavaScript grid emulator.
#[cfg(target_family = "wasm")]
mod wasm {
    use gpui_grid::GridShowcase;
    use gpui_kit::*;
    use gpui_kit::web::{CanvasFontFallback, WebBackendPreference, WebPlatform};
    use std::{borrow::Cow, cell::RefCell, rc::Rc, sync::Arc};
    use wasm_bindgen::prelude::*;

    thread_local! {
        // Retain the application as GPUI's web run_embedded returns to JS.
        static APPLICATION: RefCell<Option<ApplicationHandle>> = const { RefCell::new(None) };
    }

    /// Read-only test/diagnostic snapshot of the real GPUI grid's current row
    /// count. The counter is maintained by native Rust event handlers.
    #[wasm_bindgen]
    pub fn preview_visible_rows() -> u32 {
        gpui_grid::showcase::preview_visible_rows() as u32
    }

    #[wasm_bindgen]
    pub fn run() -> Result<(), JsValue> {
        console_error_panic_hook::set_once();
        gpui_kit::platform::web_init();
        let platform = Rc::new(WebPlatform::new_with_backend_and_font_fallback(
            false,
            WebBackendPreference::Auto,
            CanvasFontFallback::EmojiAndCjk,
        ));
        let http_client = Arc::new(platform.fetch_http_client());
        let app = Application::with_platform(platform)
            .with_http_client(http_client)
            .with_assets(gpui_kit::assets::Assets::new("https://caovc.github.io/gpui-grid/"));

        let launch = |cx: &mut App| {
            // Web canvas does not automatically see desktop system fonts.
            // These small, OFL-licensed font subsets are retrieved at WASM build time.
            let fonts = vec![
                Cow::Borrowed(include_bytes!("../fonts/Inter-Regular.ttf").as_slice()),
                Cow::Borrowed(include_bytes!("../fonts/IBMPlexSans-Regular.ttf").as_slice()),
                Cow::Borrowed(include_bytes!("../fonts/JetBrainsMono-Regular.ttf").as_slice()),
            ];
            cx.text_system().add_fonts(fonts).expect("GPUI fonts unavailable");
            gpui_kit::init(cx);
            use gpui_kit::component::theme::Theme;
            Theme::update(cx, |theme| {
                theme.font_family = "Inter Variable".into();
                theme.mono_font_family = "JetBrains Mono".into();
            });
            gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
                GridShowcase::new(window, cx)
            }).expect("Failed to open GPUI Grid browser window");
            cx.activate(true);
        };
        APPLICATION.with(|holder| {
            *holder.borrow_mut() = Some(app.run_embedded(launch));
        });
        Ok(())
    }
}
