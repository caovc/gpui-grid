//! Reusable GPUI column-header controls: column-local Popover, sort & typed filtering.
//!
//! This is part of the actual GridDelegate::render_th, NOT the outer
//! documentation website or an example-only toolbar. The actions mutate the
//! authoritative Rust GridModel and refresh the same GPUI DataTable on desktop
//! and WebAssembly.
use crate::{DataType, FilterOp, GridDelegate};
use gpui_kit::*;
use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    Sizable,
    input::{Input, InputState},
    popover::Popover,
    table::TableState,
};
use gpui_kit::prelude::FluentBuilder as _;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub(crate) type HeaderErrors = Rc<RefCell<HashMap<String, String>>>;

// Read-only evidence for real Chromium Canvas tests. The diagnostics are
// maintained only by native GPUI Popover/button callbacks; no JS grid APIs.
#[cfg(target_family = "wasm")]
thread_local! {
    static LAST_HEADER_ACTION: RefCell<String> = const { RefCell::new(String::new()) };
    static OPEN_HEADER_MENU: RefCell<String> = const { RefCell::new(String::new()) };
}

pub(crate) fn record_enter_filter(column_key: &str, accepted: bool, visible_rows: usize) {
    if accepted {
        record_action(format!("filter:{}:enter:{}", column_key, visible_rows));
    } else {
        record_action(format!("error:{}:enter", column_key));
    }
}

fn record_action(action: impl Into<String>) {
    #[cfg(target_family = "wasm")]
    LAST_HEADER_ACTION.with(|last| *last.borrow_mut() = action.into());
    #[cfg(not(target_family = "wasm"))]
    let _ = action;
}

fn record_open_menu(key: &str, open: bool) {
    #[cfg(target_family = "wasm")]
    OPEN_HEADER_MENU.with(|value| *value.borrow_mut() =
        if open { key.to_string() } else { String::new() });
    #[cfg(not(target_family = "wasm"))]
    let _ = (key, open);
}

#[cfg(target_family = "wasm")]
pub(crate) fn preview_last_header_action() -> String {
    LAST_HEADER_ACTION.with(|action| action.borrow().clone())
}
#[cfg(target_family = "wasm")]
pub(crate) fn preview_open_header_menu() -> String {
    OPEN_HEADER_MENU.with(|key| key.borrow().clone())
}

/// No viewport indices or screen positions are retained by a column menu.
pub(crate) struct HeaderSpec {
    pub key: String,
    pub name: SharedString,
    pub data_type: DataType,
    pub filter_active: bool,
    /// One-based sort priority and direction.
    pub sort: Option<(usize, bool)>,
    pub input: Entity<InputState>,
    pub input_focus: Option<FocusHandle>,
    pub table: Entity<TableState<GridDelegate>>,
    pub errors: HeaderErrors,
}

#[derive(Copy, Clone)]
enum SortCommand {
    Asc, Desc, AppendAsc, AppendDesc, Clear,
}

impl SortCommand {
    fn slug(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
            Self::AppendAsc => "add-asc",
            Self::AppendDesc => "add-desc",
            Self::Clear => "clear",
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Asc => "↑ Ascending",
            Self::Desc => "↓ Descending",
            Self::AppendAsc => "+ ↑ Multi-sort",
            Self::AppendDesc => "+ ↓ Multi-sort",
            Self::Clear => "Clear sort",
        }
    }
}

fn sort_button(key: &str, table: &Entity<TableState<GridDelegate>>, command: SortCommand) -> Button {
    let key = key.to_string();
    let target = table.clone();
    Button::new(format!("grid-sort-{}-{}", command.slug(), key))
        .label(command.label())
        .outline()
        .xsmall()
        .on_click(move |_, _, cx: &mut App| {
            let result = target.update(cx, |state, cx| {
                let result = match command {
                    SortCommand::Asc => state.delegate_mut().sort_from_header_menu(&key, false, false),
                    SortCommand::Desc => state.delegate_mut().sort_from_header_menu(&key, true, false),
                    SortCommand::AppendAsc => state.delegate_mut().sort_from_header_menu(&key, false, true),
                    SortCommand::AppendDesc => state.delegate_mut().sort_from_header_menu(&key, true, true),
                    SortCommand::Clear => state.delegate_mut().clear_column_sort(&key),
                };
                if result.is_ok() { state.refresh(cx); }
                result
            });
            if result.is_ok() {
                record_action(format!("sort:{}:{}",
                    key,
                    command.slug()));
            }
            cx.stop_propagation();
        })
}

fn filter_button(
    key: &str,
    table: &Entity<TableState<GridDelegate>>,
    input: &Entity<InputState>,
    errors: &HeaderErrors,
    op: FilterOp,
    label: &'static str,
    slug: &'static str,
    preset: Option<&'static str>,
) -> Button {
    let key = key.to_string();
    let target = table.clone();
    let input = input.clone();
    let errors = errors.clone();
    Button::new(format!("grid-filter-{}-{}", slug, key))
        .label(label)
        .outline()
        .xsmall()
        .on_click(move |_, _, cx: &mut App| {
            let raw = preset.map(str::to_string)
                .unwrap_or_else(|| input.read(cx).value().to_string());
            let result = target.update(cx, |state, cx| {
                let result = state.delegate_mut().apply_column_filter(&key, op.clone(), &raw);
                if result.is_ok() { state.refresh(cx); }
                result
            });
            match result {
                Ok(_) => {
                    errors.borrow_mut().remove(&key);
                    let visible = target.read(cx).delegate().row_count();
                    record_action(format!("filter:{}:{}:{}", key, slug, visible));
                }
                Err(error) => {
                    record_action(format!("error:{}:{}", key, slug));
                    errors.borrow_mut().insert(key.clone(), error.to_string());
                    target.update(cx, |_, cx| cx.notify());
                }
            };
            cx.stop_propagation();
        })
}

fn clear_filter_button(
    key: &str,
    table: &Entity<TableState<GridDelegate>>,
    errors: &HeaderErrors,
) -> Button {
    let key = key.to_string();
    let target = table.clone();
    let errors = errors.clone();
    Button::new(format!("grid-filter-clear-{}", key))
        .label("Clear this filter")
        .outline().xsmall()
        .on_click(move |_, _, cx: &mut App| {
            let result = target.update(cx, |state, cx| {
                let result = state.delegate_mut().clear_column_filter(&key);
                if result.is_ok() { state.refresh(cx); }
                result
            });
            match result {
                Ok(_) => {
                    errors.borrow_mut().remove(&key);
                    record_action(format!("filter-clear:{}", key));
                }
                Err(error) => { errors.borrow_mut().insert(key.clone(), error.to_string()); }
            };
            cx.stop_propagation();
        })
}

fn hide_column_button(key: &str, table: &Entity<TableState<GridDelegate>>) -> Button {
    let key = key.to_string();
    let target = table.clone();
    Button::new(format!("grid-hide-column-{}", key))
        .label("Hide column")
        .outline().xsmall()
        .on_click(move |_, _, cx: &mut App| {
            target.update(cx, |state, cx| {
                if state.delegate_mut().set_column_visibility(&key, false).is_ok() {
                    state.refresh(cx);
                }
            });
            record_action(format!("column-hide:{}", key));
            cx.stop_propagation();
        })
}

fn show_columns_button(key: &str, table: &Entity<TableState<GridDelegate>>) -> Button {
    let key = key.to_string();
    let target = table.clone();
    Button::new(format!("grid-show-all-columns-{}", key))
        .label("Show all columns")
        .outline().xsmall()
        .on_click(move |_, _, cx: &mut App| {
            target.update(cx, |state, cx| {
                if state.delegate_mut().show_all_columns().is_ok() {
                    state.refresh(cx);
                }
            });
            record_action("columns-show-all");
            cx.stop_propagation();
        })
}

/// Called by the reusable GPUI TableDelegate::render_th (not by the docs page).
pub(crate) fn column_header(spec: HeaderSpec) -> impl IntoElement {
    let filter_active = spec.filter_active;
    let sort = spec.sort;
    let name = spec.name.clone();
    let title = spec.name.to_string();
    let key = spec.key.clone();
    let pop_id = format!("gpui-grid-column-menu:{}", key);
    let trigger_id = format!("gpui-grid-column-trigger:{}", key);

    let open_key = key.clone();
    let input_focus = spec.input_focus.clone();
    let popover = Popover::new(pop_id)
        .on_open_change(move |open, _, _| record_open_menu(&open_key, *open))
        .when_some(input_focus, |popover, focus| popover.track_focus(&focus))
        // The popover intercepts left mouse-down in GPUI BasePopover before
        // the TableState parent can interpret it as column sorting/dragging.
        .trigger(
            Button::new(trigger_id)
                .label("⋯")
                .text()
                .xsmall()
                .tooltip(format!("{}: sort, filter and columns", title))
        )
        .content(move |_, _, cx| {
            let mut pane = div().w(px(282.)).flex().flex_col().gap_2().text_sm()
                .child(div().font_bold().text_color(rgb(0x263854))
                    .child(format!("Column · {}", spec.name)))
                .child(div().text_xs().text_color(rgb(0x6c7f92))
                    .child("Rust / GPUI column controls"))
                .child(div().flex().flex_wrap().gap_1()
                    .child(sort_button(&spec.key, &spec.table, SortCommand::Asc))
                    .child(sort_button(&spec.key, &spec.table, SortCommand::Desc))
                    .child(sort_button(&spec.key, &spec.table, SortCommand::Clear)))
                .child(div().flex().gap_1()
                    .child(sort_button(&spec.key, &spec.table, SortCommand::AppendAsc))
                    .child(sort_button(&spec.key, &spec.table, SortCommand::AppendDesc)))
                .child(div().border_t_1().border_color(rgb(0xe3e8ef)).pt_2()
                    .font_bold().text_color(rgb(0x344b63))
                    .child("Filter this column"));
            if spec.data_type != DataType::Boolean {
                pane = pane.child(
                    Input::new(&spec.input)
                        .aria_label(format!("Filter {}", spec.name))
                        .w_full()
                );
                pane = match spec.data_type {
                    DataType::Text => pane.child(div().flex().gap_1()
                        .child(filter_button(
                            &spec.key, &spec.table, &spec.input, &spec.errors,
                            FilterOp::Contains, "Contains", "contains", None,
                        ))
                        .child(filter_button(
                            &spec.key, &spec.table, &spec.input, &spec.errors,
                            FilterOp::Equals, "Equals", "equals", None,
                        ))),
                    DataType::Number => pane.child(div().flex().gap_1()
                        .child(filter_button(
                            &spec.key, &spec.table, &spec.input, &spec.errors,
                            FilterOp::GreaterThan, "> Greater", "greater", None,
                        ))
                        .child(filter_button(
                            &spec.key, &spec.table, &spec.input, &spec.errors,
                            FilterOp::LessThan, "< Less", "less", None,
                        ))
                        .child(filter_button(
                            &spec.key, &spec.table, &spec.input, &spec.errors,
                            FilterOp::Equals, "= Equals", "number-equals", None,
                        ))),
                    DataType::Boolean => pane,
                };
            } else {
                pane = pane.child(div().flex().gap_1()
                    .child(filter_button(
                        &spec.key, &spec.table, &spec.input, &spec.errors,
                        FilterOp::Equals, "True", "bool-true", Some("true"),
                    ))
                    .child(filter_button(
                        &spec.key, &spec.table, &spec.input, &spec.errors,
                        FilterOp::Equals, "False", "bool-false", Some("false"),
                    )));
            }
            pane = pane.child(clear_filter_button(&spec.key, &spec.table, &spec.errors))
                .child(div().border_t_1().border_color(rgb(0xe3e8ef)).pt_2()
                    .child(div().flex().gap_1()
                        .child(hide_column_button(&spec.key, &spec.table))
                        .child(show_columns_button(&spec.key, &spec.table))));
            if let Some(error) = spec.errors.borrow().get(&spec.key).cloned() {
                pane = pane.child(div().text_xs().text_color(rgb(0xbb3551)).child(error));
            }
            pane
        });

    let sort_indicator = sort.map(|(priority, desc)| {
        format!("{}{}", if desc { "↓" } else { "↑" }, priority)
    });
    div().size_full().flex().items_center().justify_between().gap_1()
        .child(
            div().min_w_0().flex_1().flex().items_center().gap_1()
                .child(name)
                .when_some(sort_indicator, |this, badge| {
                    this.child(div().text_xs().text_color(rgb(0x3468ba)).child(badge))
                })
                .when(filter_active, |this| {
                    this.child(div().text_color(rgb(0x0f9e87)).child("●"))
                })
        )
        .child(popover)
}
