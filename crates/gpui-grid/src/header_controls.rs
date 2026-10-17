//! Reusable GPUI column-header controls: column-local Popover, sort & typed filtering.
//!
//! This is part of the actual GridDelegate::render_th, NOT the outer
//! documentation website or an example-only toolbar. The actions mutate the
//! authoritative Rust GridModel and refresh the same GPUI DataTable on desktop
//! and WebAssembly.
use crate::{quartz::QUARTZ_LIGHT, DataType, DistinctValue, FilterOp, GridDelegate, Value};
use gpui_kit::*;
use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    checkbox::Checkbox,
    scroll::ScrollableElement as _,
    Disableable as _, Sizable,
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

pub(crate) fn record_set_search_enter(key: &str, valid: bool, visible_rows: usize) {
    record_action(format!(
        "set-search:{}:{}:{}",
        key, if valid { "apply" } else { "error" }, visible_rows
    ));
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
    pub set_values: Option<Vec<DistinctValue>>,
    /// None => all visible options are implicitly selected. Some([]) means none.
    pub set_selected: Option<Vec<Value>>,
    pub set_search: Option<Entity<InputState>>,
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
            let result = target.update(cx, |state, cx| {
                let result = state.delegate_mut().set_column_visibility(&key, false);
                if result.is_ok() { state.refresh(cx); }
                result.map(|_| state.delegate().displayed_columns_count())
            });
            if let Ok(count) = result {
                record_action(format!("column-hide:{}:{}", key, count));
            }
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
            let result = target.update(cx, |state, cx| {
                let result = state.delegate_mut().show_all_columns();
                if result.is_ok() { state.refresh(cx); }
                result.map(|_| state.delegate().displayed_columns_count())
            });
            if let Ok(count) = result {
                record_action(format!("columns-show-all:{}", count));
            }
            cx.stop_propagation();
        })
}


fn matching_set_values(values: &[DistinctValue], query: &str) -> Vec<Value> {
    let query = query.trim().to_lowercase();
    values.iter().filter(|option|
        option.value.text().to_lowercase().contains(&query)
    ).map(|option| option.value.clone()).collect()
}

/// Checkboxes are controlled widgets. Always read the current authoritative
/// Rust model before changing a selection, not a stale rendered checkbox.
fn set_option_checkbox(
    key: &str, table: &Entity<TableState<GridDelegate>>,
    value: &Value, checked: bool,
) -> impl IntoElement {
    let key = key.to_owned();
    let cell = value.clone();
    let target = table.clone();
    Checkbox::new(format!("grid-set-choice-{}-{}", key, cell.text()))
        .checked(checked)
        .label(if matches!(cell, Value::Empty) {
            "(Blanks)".to_owned()
        } else {
            cell.text()
        })
        .small()
        .on_change(move |_, _, cx| {
            let result = target.update(cx, |state, cx| {
                let result = state.delegate_mut().toggle_column_value(&key, cell.clone());
                if result.is_ok() { state.refresh(cx); }
                result.map(|_| state.delegate().row_count())
            });
            if let Ok(count) = result {
                record_action(format!("set:{}:{}:{}", key, cell.text(), count));
            }
            cx.stop_propagation();
        })
}

fn select_all_checkbox(
    key: &str, table: &Entity<TableState<GridDelegate>>,
    matching: Vec<Value>, checked: bool,
) -> impl IntoElement {
    let key = key.to_owned();
    let target = table.clone();
    Checkbox::new(format!("grid-set-select-all-{}", key))
        .checked(checked)
        .disabled(matching.is_empty())
        .label("(Select All)")
        .small()
        .on_change(move |state, _, cx| {
            let result = target.update(cx, |table, cx| {
                let result = table.delegate_mut().set_matching_values(&key, &matching, *state);
                if result.is_ok() { table.refresh(cx); }
                result.map(|_| table.delegate().row_count())
            });
            if let Ok(count) = result {
                record_action(format!(
                    "set-all:{}:{}:{}",
                    key, if *state { "checked" } else { "unchecked" }, count
                ));
            }
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
                // ASCII label is visible even with subset WASM fonts. A
                // larger hitbox reduces missed clicks inside the GPUI header.
                .label("...")
                .text()
                .xsmall()
                .w(px(32.0))
                .h(px(27.0))
                .tooltip(format!("{}: sort, filter and columns", title))
        )
        .content(move |_, _, cx| {
            // Keep all essential actions visible even in a short embedded
            // GPUI WebPlatform canvas. Header sort and column visibility live
            // ABOVE the filter editor rather than below a potentially clipped
            // popover surface.
            let mut pane = div().w(px(292.)).flex().flex_col().gap_1().text_sm()
                .bg(rgb(QUARTZ_LIGHT.surface)).text_color(rgb(QUARTZ_LIGHT.foreground))
                .child(div().font_bold().text_color(rgb(0x263854))
                    .child(format!("Column · {}", spec.name)))
                .child(div().flex().flex_wrap().gap_1()
                    .child(sort_button(&spec.key, &spec.table, SortCommand::Asc))
                    .child(sort_button(&spec.key, &spec.table, SortCommand::Desc))
                    .child(sort_button(&spec.key, &spec.table, SortCommand::Clear)))
                .child(div().flex().gap_1()
                    .child(sort_button(&spec.key, &spec.table, SortCommand::AppendAsc))
                    .child(sort_button(&spec.key, &spec.table, SortCommand::AppendDesc)))
                .child(div().flex().gap_1()
                    .child(hide_column_button(&spec.key, &spec.table))
                    .child(show_columns_button(&spec.key, &spec.table)))
                .child(div().border_t_1().border_color(rgb(0xe3e8ef)).pt_1()
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
            pane = pane.child(clear_filter_button(&spec.key, &spec.table, &spec.errors));

            // AG Grid-inspired native Set Filter: mini-search, Select All and
            // checked candidates. This entire subtree is real GPUI, never
            // a JavaScript table nor an AG Grid Enterprise implementation.
            if let (Some(values), Some(search)) = (&spec.set_values, &spec.set_search) {
                let query = search.read(cx).value().to_string();
                let matching = matching_set_values(values, &query);
                let selected = spec.set_selected.as_ref();
                let selected_count = selected.map_or(values.len(), Vec::len);
                let all_matching_checked = !matching.is_empty()
                    && matching.iter().all(|value| selected.is_none_or(|set| set.contains(value)));
                let mut entries = div().flex().flex_col().gap_1()
                    .child(select_all_checkbox(
                        &spec.key, &spec.table, matching.clone(), all_matching_checked
                    ));
                for option in values.iter().filter(|option| matching.contains(&option.value)) {
                    let checked = selected.is_none_or(|set| set.contains(&option.value));
                    entries = entries.child(div().flex().items_center().justify_between().gap_2()
                        .child(set_option_checkbox(
                            &spec.key, &spec.table, &option.value, checked
                        ))
                        .child(div().text_xs().text_color(rgb(QUARTZ_LIGHT.muted))
                            .child(option.count.to_string())));
                }
                pane = pane.child(div()
                    .border_t_1().border_color(rgb(0xe3e8ef)).pt_1()
                    .flex().justify_between().items_center()
                    .child(div().font_bold().text_color(rgb(0x344b63)).child("Set Filter"))
                    .child(div().text_xs().text_color(rgb(0x6a7f98))
                        .child(format!("{selected_count}/{}", values.len()))))
                    .child(Input::new(search)
                        .aria_label(format!("Search {} values", spec.name))
                        .w_full())
                    .child(div().id(format!("grid-set-options:{}", spec.key))
                        .max_h(px(180.0)).overflow_y_scrollbar()
                        .child(entries));
            }
            if let Some(error) = spec.errors.borrow().get(&spec.key).cloned() {
                pane = pane.child(div().text_xs().text_color(rgb(0xbb3551)).child(error));
            }
            pane
        });

    let sort_indicator = sort.map(|(priority, desc)| {
        format!("{}{}", if desc { "↓" } else { "↑" }, priority)
    });
    div().size_full().flex().items_center().justify_between().gap_1()
        .bg(rgb(QUARTZ_LIGHT.header_background))
        .text_color(rgb(QUARTZ_LIGHT.foreground))
        .child(
            div().min_w_0().flex_1().flex().items_center().gap_1()
                .child(name)
                .when_some(sort_indicator, |this, badge| {
                    this.child(div().text_xs().text_color(rgb(0x3468ba)).child(badge))
                })
                .when(filter_active, |this| {
                    this.child(div().text_color(rgb(QUARTZ_LIGHT.accent)).child("●"))
                })
        )
        .child(popover)
}
