//! Native desktop preview. Browser preview in `web/` is a separate web implementation.
use gpui_grid::{GridDelegate, sample_orders};
use gpui_kit::*;
use gpui_kit::component::table::{DataTable, TableState};
use gpui_kit::component::button::Button;

struct NativeGridApp {
    table: Entity<TableState<GridDelegate>>,
}

impl Render for NativeGridApp {
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
                            .child(div().text_xl().font_semibold().text_color(rgb(0x273550)).child("GPUI Grid"))
                            .child(div().text_sm().text_color(rgb(0x7d899b)).child("Rust native • virtualized DataTable • sorting • selection • pinning"))
                    )
                    .child(
                        div().flex().gap_2()
                            .child(Button::new("completed-only").label("Completed only").on_click(move |_, _, cx| {
                                only_completed.update(cx, |table, cx| {
                                    table.delegate_mut().set_quick_filter("Completed");
                                    table.refresh(cx);
                                });
                            }))
                            .child(Button::new("all-rows").label("All rows").on_click(move |_, _, cx| {
                                show_all.update(cx, |table, cx| {
                                    table.delegate_mut().set_quick_filter("");
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

fn main() {
    gpui_kit::application().with_assets(gpui_kit::assets::Assets).run(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            let table = cx.new(|cx| {
                TableState::new(GridDelegate::new(sample_orders(12_480)), window, cx)
                    .cell_selectable(true)
                    .row_selectable(true)
                    .col_resizable(true)
                    .col_movable(true)
            });
            cx.new(|_| NativeGridApp { table })
        }).expect("Unable to open GPUI Grid window");
    });
}
