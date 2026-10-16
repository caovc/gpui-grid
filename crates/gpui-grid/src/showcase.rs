//! Reusable GPUI view for native desktop and WASM web previews.
use crate::{GridDelegate, sample_orders};
use gpui_kit::*;
use gpui_kit::base::StyledExt;
use gpui_kit::component::table::{DataTable, TableState};
use gpui_kit::component::button::Button;

// Diagnostic hook for the browser smoke test. It is updated by the *real*
// GPUI button handlers, so the test verifies UI input instead of calling an
// alternative JavaScript table/filter implementation.
#[cfg(target_family = "wasm")]
thread_local! {
    static PREVIEW_VISIBLE_ROWS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

fn publish_visible_rows(count: usize) {
    #[cfg(target_family = "wasm")]
    PREVIEW_VISIBLE_ROWS.with(|rows| rows.set(count));
    #[cfg(not(target_family = "wasm"))]
    let _ = count;
}

#[cfg(target_family = "wasm")]
pub fn preview_visible_rows() -> usize {
    PREVIEW_VISIBLE_ROWS.with(std::cell::Cell::get)
}


pub struct GridShowcase {
    table: Entity<TableState<GridDelegate>>,
}

impl Render for GridShowcase {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let only_completed = self.table.clone();
        let show_all = self.table.clone();
        div()
            .size_full().flex().flex_col().bg(rgb(0xf7f9fc))
            .child(
                div().flex().items_center().justify_between()
                    .px_6().py_4().bg(rgb(0xffffff))
                    .border_b_1().border_color(rgb(0xe4e9f1))
                    .child(
                        div().flex().flex_col().gap_1()
                            .child(div().text_xl().font_bold().text_color(rgb(0x273550)).child("GPUI Grid"))
                            .child(div().text_sm().text_color(rgb(0x7d899b)).child("Shared Rust GPUI • WebAssembly + Desktop • Virtualized DataTable"))
                    )
                    .child(
                        div().flex().gap_2()
                            .child(Button::new("completed-only").label("Completed only").on_click(move |_, _, cx| {
                                only_completed.update(cx, |table, cx| {
                                    let delegate = table.delegate_mut();
                                    delegate.set_quick_filter("Completed");
                                    publish_visible_rows(delegate.row_count());
                                    table.refresh(cx);
                                });
                            }))
                            .child(Button::new("all-rows").label("All rows").on_click(move |_, _, cx| {
                                show_all.update(cx, |table, cx| {
                                    let delegate = table.delegate_mut();
                                    delegate.set_quick_filter("");
                                    publish_visible_rows(delegate.row_count());
                                    table.refresh(cx);
                                });
                            }))
                    )
            )
            .child(
                div().flex_1().min_h_0().p_4()
                    .child(DataTable::new(&self.table).bordered(true).stripe(true))
            )
    }
}


impl GridShowcase {
    /// A shared Rust view: the native desktop and wasm32 browser run this same code.
    pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
        let data = sample_orders(12_480);
        publish_visible_rows(data.len());
        let table = cx.new(|cx| {
            TableState::new(GridDelegate::new(data), window, cx)
                .cell_selectable(true)
                .row_selectable(true)
                .col_resizable(true)
                .col_movable(true)
        });
        cx.new(|_| Self { table })
    }
}
