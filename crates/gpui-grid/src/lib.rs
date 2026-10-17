//! Reusable native data-grid adapter on top of GPUI Kit's virtualized DataTable.
//!
//! Responsibilities:
//! - translate typed row values to native GPUI elements;
//! - expose sorting, cell selection, column resize/move/pin to the host;
//! - keep filtering/editing/grouping in `gpui-grid-core` for composability.
//!
//! This is the first native component milestone. The browser showcase additionally
//! demonstrates controls that are not yet available in this native adapter.
use gpui_grid_core::{GridError, GridModel, Sort, Value};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::*;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::component::table::{
    Column as UiColumn, ColumnSort, TableDelegate, TableState,
};
mod header_controls;
pub mod quartz;
use header_controls::{HeaderErrors, HeaderSpec, column_header};
/// Browser-only, read-only verification of actual GPUI header-menu operations.
#[cfg(target_family = "wasm")]
pub mod header_debug {
    pub fn preview_last_header_action() -> String {
        super::header_controls::preview_last_header_action()
    }
    pub fn preview_open_header_menu() -> String {
        super::header_controls::preview_open_header_menu()
    }
}

pub use gpui_grid_core::{
    CellAddress, Column as GridColumn, ColumnState, DataType, Edit, Filter, FilterOp, parse_cell_value,
    CellRange, SelectionRect, DistinctValue, GridApi, GridEvent, GridOptions, GridState, PinSide, Row, sample_orders,
};

/// Identifies a cell independently of sort order, filters or virtualization.
#[derive(Clone, Debug)]
pub struct CellInfo {
    pub address: CellAddress,
    pub data_type: DataType,
    pub editable: bool,
    pub value: Value,
}

/// The editor is a real GPUI InputState rendered inside a virtualized cell.
/// This shared slot avoids reentering TableState while it emits double-click events.
#[derive(Clone)]
pub(crate) struct InlineEditor {
    pub address: CellAddress,
    pub input: Entity<InputState>,
}
pub(crate) type InlineEditorSlot = Rc<RefCell<Option<InlineEditor>>>;

/// Use `cx.new(|cx| TableState::new(GridDelegate::new(model), window, cx))`
/// to instantiate the GPUI state. `DataTable::new(&entity)` performs native
/// row/column virtualization and keyboard interaction.
pub struct GridDelegate {
    pub model: GridModel,
    presentation: Vec<UiColumn>,
    visible: Vec<usize>,
    selection_rect: Option<SelectionRect>,
    inline_editor: InlineEditorSlot,
    /// Input entities persist across header re-renders and sort/filter operations.
    header_inputs: HashMap<String, Entity<InputState>>,
    /// Bounded cached distinct choices: never scan 50k rows during render_th.
    header_set_values: HashMap<String, Vec<DistinctValue>>,
    header_set_search: HashMap<String, Entity<InputState>>,
    header_subscriptions: Vec<Subscription>,
    header_errors: HeaderErrors,
}

impl GridDelegate {
    /// Presentation cache is regenerated from the authoritative Rust column
    /// state; hiding a column never mutates the typed Row storage.
    fn presentation_for(model: &GridModel) -> Vec<UiColumn> {
        model.state.columns.iter().filter(|col| col.visible).map(|state| {
            let col = &model.columns[model.column_index(&state.key).expect("state column key validated")];
            let base = UiColumn::new(col.key.clone(), col.name.clone())
                .width(px(state.width)).min_width(px(76.0)).max_width(px(580.0)).sortable();
            if state.pin == PinSide::Left { base.fixed_left() } else { base }
        }).collect()
    }

    pub fn set_column_visibility(&mut self, key: &str, visible: bool) -> Result<(), GridError> {
        if !visible && self.presentation.len() <= 1 && self.presentation.iter().any(|col| col.key.as_ref() == key) {
            return Err(GridError::InvalidValue("Cannot hide the final visible column".into()));
        }
        self.model.set_column_visible(key, visible)?;
        self.presentation = Self::presentation_for(&self.model);
        self.refresh_selection_rect();
        Ok(())
    }

    /// Restore every schema column, preserving the existing presentation order.
    pub fn show_all_columns(&mut self) -> Result<(), GridError> {
        let keys: Vec<String> = self.model.state.columns.iter().map(|col| col.key.clone()).collect();
        for key in keys {
            self.model.set_column_visible(&key, true)?;
        }
        self.presentation = Self::presentation_for(&self.model);
        self.refresh_selection_rect();
        Ok(())
    }

    fn build_header_set_values(model: &GridModel) -> HashMap<String, Vec<DistinctValue>> {
        model.columns.iter().filter_map(|column| {
            let values = model.distinct_column_values(&column.key, 12).ok().flatten()?;
            (values.len() >= 2).then(|| (column.key.clone(), values))
        }).collect()
    }

    pub fn new(model: GridModel) -> Self {
        // GPUI elements are a transient cache; GridModel.state owns the order,
        // width, pin and visibility independently of the virtual viewport.
        let presentation = Self::presentation_for(&model);
        let header_set_values = Self::build_header_set_values(&model);
        let visible = model.view();
        let mut grid = Self {
            model, presentation, visible, selection_rect: None,
            inline_editor: Rc::new(RefCell::new(None)),
            header_inputs: HashMap::new(),
            header_set_values,
            header_set_search: HashMap::new(),
            header_subscriptions: Vec::new(),
            header_errors: Rc::new(RefCell::new(HashMap::new())),
        };
        grid.refresh_selection_rect();
        grid
    }

    /// Reproject stable selection after sorting, filtering or column movement.
    fn refresh_selection_rect(&mut self) {
        let columns = self.presentation.iter().map(|col| col.key.to_string()).collect::<Vec<_>>();
        self.selection_rect = self.model.selected_rect(&self.visible, &columns);
    }

    /// Pointer and keyboard events change the authoritative Rust model.
    pub fn select_address(&mut self, address: Option<CellAddress>, extend: bool) -> Result<(), GridError> {
        self.model.select_cell(address, extend)?;
        self.refresh_selection_rect();
        Ok(())
    }

    pub fn selected_cell_count(&self) -> usize {
        self.selection_rect.map_or(0, |bounds| bounds.cell_count())
    }

    pub(crate) fn inline_editor_slot(&self) -> InlineEditorSlot { self.inline_editor.clone() }

    /// Map a stable address to current display coordinates, if still visible.
    /// Never persist the returned indices across sort/filter changes.
    pub fn display_position(&self, address: &CellAddress) -> Option<(usize, usize)> {
        let row_ix = self.visible.iter().position(|&ix| {
            self.model.row(ix).is_some_and(|row| row.id == address.row_id)
        })?;
        let col_ix = self.presentation.iter().position(|col| col.key.to_string() == address.column_key)?;
        Some((row_ix, col_ix))
    }

    pub fn cell_info_by_address(&self, address: &CellAddress) -> Option<CellInfo> {
        let (row_ix, col_ix) = self.display_position(address)?;
        self.cell_info(row_ix, col_ix)
    }

    pub fn displayed_columns_count(&self) -> usize { self.presentation.len() }

    /// Select all *visible* rows and columns using only the two endpoints.
    /// No per-cell list is built, even for 100k rows or hundreds of columns.
    pub fn select_all_visible(&mut self) -> Result<(), GridError> {
        let end_row = self.visible.len().checked_sub(1);
        let end_col = self.presentation.len().checked_sub(1);
        let next = match (end_row, end_col) {
            (Some(row_ix), Some(col_ix)) => Some(CellRange {
                anchor: self.cell_info(0, 0).expect("valid first cell").address,
                focus: self.cell_info(row_ix, col_ix).expect("valid last cell").address,
            }),
            _ => None,
        };
        self.model.select_range(next)?;
        self.refresh_selection_rect();
        Ok(())
    }

    /// Tab navigation follows displayed columns and skips readonly cells.
    pub fn next_editable_cell(&self, address: &CellAddress, backwards: bool) -> Option<CellInfo> {
        let cols = self.presentation.len();
        let total = cols.checked_mul(self.visible.len())?;
        if total < 2 { return None; }
        let row_ix = self.visible.iter().position(|&ix| {
            self.model.row(ix).is_some_and(|row| row.id == address.row_id)
        })?;
        let col_ix = self.presentation.iter().position(|col| col.key.to_string() == address.column_key)?;
        let start = row_ix * cols + col_ix;
        for distance in 1..total {
            let next = if backwards {
                (start + total - distance) % total
            } else {
                (start + distance) % total
            };
            if let Some(info) = self.cell_info(next / cols, next % cols) {
                if info.editable { return Some(info); }
            }
        }
        None
    }

    pub fn row_count(&self) -> usize { self.visible.len() }

    /// Resolve the viewport indices immediately to stable row/column IDs.
    /// The caller must hold on to `address`, not `row_ix`, while editing.
    pub fn cell_info(&self, row_ix: usize, col_ix: usize) -> Option<CellInfo> {
        let source_ix = *self.visible.get(row_ix)?;
        let column_key = self.presentation.get(col_ix)?.key.to_string();
        let storage_col = self.model.column_index(&column_key)?;
        let column = self.model.columns.get(storage_col)?;
        let row = self.model.row(source_ix)?;
        Some(CellInfo {
            address: CellAddress::new(row.id, column_key),
            data_type: column.data_type,
            editable: column.editable,
            value: row.cells.get(storage_col)?.clone(),
        })
    }

    /// Replace the typed filters and recalculate the visible row indices.
    /// When called through a `TableState`, follow with `TableState::refresh(cx)`.
    pub fn set_filters(&mut self, filters: Vec<Filter>) -> Result<(), GridError> {
        self.model.set_filters(filters)?;
        self.rebuild_view();
        Ok(())
    }

    /// Apply one column-local filter from a real GPUI header input. Empty
    /// input clears this column only. Invalid typed values leave every filter
    /// and the current row view unchanged.
    pub fn apply_column_filter(
        &mut self,
        key: &str,
        op: FilterOp,
        raw: &str,
    ) -> Result<(), GridError> {
        let ix = self.model.column_index(key)
            .ok_or_else(|| GridError::UnknownColumn(key.to_string()))?;
        let data_type = self.model.columns[ix].data_type;
        if raw.trim().is_empty() {
            return self.clear_column_filter(key);
        }
        let value = if data_type == DataType::Text {
            Value::Text(raw.to_string())
        } else {
            parse_cell_value(data_type, raw)
                .map_err(|error| GridError::InvalidValue(error.into()))?
        };
        self.model.upsert_column_filter(Filter {
            column: key.to_string(), op, value,
        })?;
        self.rebuild_view();
        Ok(())
    }

    pub fn clear_column_filter(&mut self, key: &str) -> Result<(), GridError> {
        self.model.clear_column_filter(key)?;
        self.rebuild_view();
        Ok(())
    }

    pub fn set_column_values(&mut self, key: &str, values: Vec<Value>) -> Result<(), GridError> {
        self.model.set_column_values(key, values)?;
        self.rebuild_view();
        Ok(())
    }

    pub fn toggle_column_value(&mut self, key: &str, value: Value) -> Result<(), GridError> {
        self.model.toggle_column_value(key, value)?;
        self.rebuild_view();
        Ok(())
    }

    /// Batch-check or uncheck the candidates shown by a Set Filter mini search
    /// without wiping selected options currently hidden by the search query.
    pub fn set_matching_values(
        &mut self, key: &str, matching: &[Value], checked: bool,
    ) -> Result<(), GridError> {
        let options = self.header_set_values.get(key).ok_or_else(|| GridError::InvalidValue(
            format!("Set Filter not available for high-cardinality column {key}")
        ))?;
        if matching.iter().any(|value| !options.iter().any(|x| &x.value == value)) {
            return Err(GridError::InvalidValue("Unknown set filter option".into()));
        }
        let mut selected: Vec<Value> = match self.model.state.filters.iter().find(|f| f.column == key) {
            Some(Filter { op: FilterOp::OneOf(values), .. }) => values.clone(),
            _ => options.iter().map(|item| item.value.clone()).collect(),
        };
        for value in matching {
            if checked {
                if !selected.contains(value) { selected.push(value.clone()); }
            } else {
                selected.retain(|selected| selected != value);
            }
        }
        self.set_column_values(key, selected)
    }

    /// Export exactly the current filtered/sorted rows and current column order.
    /// Formula-like user text is escaped by the core CSV encoder.
    pub fn export_visible_csv(&self) -> Result<String, GridError> {
        let column_keys: Vec<String> = self.presentation.iter().map(|c| c.key.to_string()).collect();
        let keys: Vec<&str> = column_keys.iter().map(String::as_str).collect();
        self.model.export_csv(&self.visible, &keys)
    }

    /// After calling this through `TableState::delegate_mut()`, call
    /// `TableState::refresh(cx)` so the viewport reflects the new row count.
    pub fn set_quick_filter(&mut self, query: impl Into<String>) {
        self.model.set_quick_filter(query);
        self.rebuild_view();
    }

    pub fn set_sort(&mut self, sorts: Vec<Sort>) -> Result<(), GridError> {
        self.model.set_sorts(sorts)?;
        // GPUI Kit's own header sort arrows and our additional priority badge
        // both follow the authoritative Core state, including multi-sort.
        for col in &mut self.presentation {
            let direction = self.model.state.sorts.iter()
                .find(|sort| sort.column == col.key.as_ref());
            col.sort = Some(match direction {
                Some(sort) if sort.descending => ColumnSort::Descending,
                Some(_) => ColumnSort::Ascending,
                None => ColumnSort::Default,
            });
        }
        self.rebuild_view();
        Ok(())
    }

    /// Header menu supports single sort and explicit multi-sort. Normal
    /// click-to-sort in GPUI Kit remains single-column by design.
    pub fn sort_from_header_menu(
        &mut self, key: &str, descending: bool, append: bool,
    ) -> Result<(), GridError> {
        let mut next = if append { self.model.state.sorts.clone() } else { Vec::new() };
        next.retain(|sort| sort.column != key);
        next.push(if descending { Sort::descending(key) } else { Sort::ascending(key) });
        self.set_sort(next)
    }

    pub fn clear_column_sort(&mut self, key: &str) -> Result<(), GridError> {
        let remaining = self.model.state.sorts.iter()
            .filter(|sort| sort.column != key).cloned().collect();
        self.set_sort(remaining)
    }

    pub fn edit(&mut self, edits: &[gpui_grid_core::Edit]) -> Result<usize, GridError> {
        let count = self.model.apply_edits(edits)?;
        // Distinct options are cached, but editing can introduce or remove
        // values. Refresh this cache only on actual data mutations, never on
        // every header render or scroll frame.
        if count > 0 { self.header_set_values = Self::build_header_set_values(&self.model); }
        self.rebuild_view();
        Ok(count)
    }

    pub fn rebuild_view(&mut self) {
        self.visible = self.model.view();
        self.refresh_selection_rect();
    }

    pub fn visible_row(&self, index: usize) -> Option<&gpui_grid_core::Row> {
        self.visible.get(index).and_then(|&original| self.model.row(original))
    }

}

impl TableDelegate for GridDelegate {
    fn columns_count(&self, _cx: &App) -> usize { self.presentation.len() }
    fn rows_count(&self, _cx: &App) -> usize { self.visible.len() }
    fn column(&self, ix: usize, _cx: &App) -> UiColumn { self.presentation[ix].clone() }

    fn render_th(
        &mut self, col_ix: usize,
        window: &mut Window, cx: &mut Context<'_, TableState<Self>>,
    ) -> impl IntoElement {
        let presentation = self.presentation[col_ix].clone();
        let key = presentation.key.to_string();
        let data_type = self.model.columns[
            self.model.column_index(&key).expect("displayed column key is in the schema")
        ].data_type;
        let active = self.model.state.filters.iter().any(|filter| filter.column == key);
        let set_values = self.header_set_values.get(&key).cloned();
        let set_selected = self.model.state.filters.iter().find(|f| f.column == key)
            .and_then(|f| match &f.op {
                FilterOp::OneOf(values) => Some(values.clone()),
                _ => None,
            });
        let sort = self.model.state.sorts.iter().enumerate()
            .find(|(_, sort)| sort.column == key)
            .map(|(priority, sort)| (priority + 1, sort.descending));
        // Construct lazily once per stable key. Never allocate an InputState
        // inside the Popover content closure during every repaint.
        let input = if let Some(input) = self.header_inputs.get(&key) {
            input.clone()
        } else {
            let input = cx.new(|cx| InputState::new(window, cx).placeholder("Type filter value..."));
            let column_key = key.clone();
            let errors = self.header_errors.clone();
            // Enter on the focused native GPUI filter InputState applies a
            // sensible default operator. The Popover also exposes explicit
            // Contains/Equals and numeric comparison buttons.
            let sub = cx.subscribe_in(&input, window,
                move |table, input, event: &InputEvent, _, cx| {
                    if !matches!(event, InputEvent::PressEnter { .. }) {
                        return;
                    }
                    let raw = input.read(cx).value().to_string();
                    let op = if data_type == DataType::Text {
                        FilterOp::Contains
                    } else {
                        FilterOp::Equals
                    };
                    let result = table.delegate_mut().apply_column_filter(&column_key, op, &raw);
                    if result.is_ok() {
                        errors.borrow_mut().remove(&column_key);
                    } else if let Err(error) = &result {
                        errors.borrow_mut().insert(column_key.clone(), error.to_string());
                    }
                    let visible = table.delegate().row_count();
                    header_controls::record_enter_filter(&column_key, result.is_ok(), visible);
                    table.refresh(cx);
                },
            );
            self.header_subscriptions.push(sub);
            self.header_inputs.insert(key.clone(), input.clone());
            input
        };
        let set_search = if set_values.is_some() {
            Some(if let Some(input) = self.header_set_search.get(&key) {
                input.clone()
            } else {
                let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search values..."));
                let option_key = key.clone();
                let options = set_values.clone().unwrap_or_default();
                let subscription = cx.subscribe_in(
                    &search, window,
                    move |table, input, event: &InputEvent, _, cx| {
                        match event {
                            InputEvent::Change => {
                                // Mini Filter changes only the displayed
                                // checkbox list, not the actual filtered data.
                                table.refresh(cx);
                            }
                            InputEvent::PressEnter { .. } => {
                                // Excel/AG Grid-like shortcut: Enter selects
                                // all mini-search matches and applies instantly.
                                let query = input.read(cx).value().to_lowercase();
                                let values = options.iter()
                                    .filter(|candidate| candidate.value.text().to_lowercase().contains(&query))
                                    .map(|candidate| candidate.value.clone())
                                    .collect::<Vec<_>>();
                                let result = table.delegate_mut().set_column_values(&option_key, values);
                                let count = table.delegate().row_count();
                                header_controls::record_set_search_enter(
                                    &option_key, result.is_ok(), count
                                );
                                table.refresh(cx);
                            }
                            _ => {}
                        }
                    },
                );
                self.header_subscriptions.push(subscription);
                self.header_set_search.insert(key.clone(), search.clone());
                search
            })
        } else {
            None
        };
        let input_focus = if data_type != DataType::Boolean {
            Some(input.focus_handle(cx))
        } else { None };
        column_header(HeaderSpec {
            key,
            name: presentation.name,
            data_type, filter_active: active, sort,
            set_values, set_selected, set_search,
            input, input_focus, table: cx.entity(), errors: self.header_errors.clone(),
        })
    }

    fn render_td(
        &mut self, row_ix: usize, col_ix: usize,
        _window: &mut Window, _cx: &mut Context<'_, TableState<Self>>,
    ) -> impl IntoElement {
        let key = self.presentation[col_ix].key.to_string();
        let in_range = self.selection_rect.is_some_and(|rect| rect.contains(row_ix, col_ix));
        // GPUI paints the actual editor in the visible cell; JS owns no table data.
        let inline_input = self.inline_editor.borrow().as_ref().and_then(|editor| {
            let row_id = self.visible.get(row_ix)
                .and_then(|&index| self.model.row(index))
                .map(|row| row.id)?;
            (editor.address.row_id == row_id && editor.address.column_key == key)
                .then(|| editor.input.clone())
        });
        if let Some(input) = inline_input {
            return div()
                // A closer GPUI key context overrides DataTable's and Root's
                // built-in Tab navigation only while this editor is mounted.
                .key_context("GPUIGridInlineEditor")
                .w_full().h_full().flex().items_center()
                .bg(rgb(quartz::QUARTZ_LIGHT.editor_background))
                .border_2().border_color(rgb(quartz::QUARTZ_LIGHT.editor_border))
                .child(Input::new(&input).appearance(false).bordered(false).flex_1())
                .into_any_element();
        }
        let value = self.visible.get(row_ix)
            .and_then(|&index| self.model.value(index, &key))
            .cloned().unwrap_or(Value::Empty);
        // Data stays typed until the last moment, avoiding lexicographic money sort.
        let is_num = matches!(&value, Value::Number(_));
        let label = value.text();
        let base = div().flex().items_center().h_full().px_3().w_full()
            .text_sm().text_color(rgb(quartz::QUARTZ_LIGHT.foreground))
            .when(in_range, |element|
                element.bg(rgb(quartz::QUARTZ_LIGHT.selected_background)));
        match key.as_str() {
            "order_id" => base.text_color(rgb(quartz::QUARTZ_LIGHT.accent))
                .child(SharedString::from(label)).into_any_element(),
            "status" => {
                let (background, foreground) = match label.as_str() {
                    "Completed" => (0xe7f8ef, 0x147b58),
                    "Processing" => (0xe8f0ff, 0x315bc2),
                    "Pending" => (0xfff3df, 0x9c650d),
                    _ => (0xffe9eb, 0xa83c50),
                };
                base.child(div().rounded_md().px_2().py_1()
                    .bg(rgb(background)).text_color(rgb(foreground))
                    .child(SharedString::from(label))).into_any_element()
            }
            "amount" => base.justify_end().text_color(rgb(quartz::QUARTZ_LIGHT.foreground))
                .child(SharedString::from(match value {
                    Value::Number(n) => format!("${n:.2}"),
                    _ => label,
                })).into_any_element(),
            "growth" => {
                let color = match &value {
                    Value::Number(n) if *n > 0.0 => 0x16815a,
                    Value::Number(n) if *n < 0.0 => 0xae4d4d,
                    _ => quartz::QUARTZ_LIGHT.foreground,
                };
                base.justify_end().text_color(rgb(color))
                    .child(SharedString::from(match value {
                        Value::Number(n) => format!("{n:+.1}%"),
                        _ => label,
                    })).into_any_element()
            }
            _ if is_num => base.justify_end()
                .child(SharedString::from(label)).into_any_element(),
            _ => base.child(SharedString::from(label)).into_any_element(),
        }
    }

    fn perform_sort(
        &mut self, col_ix: usize, sort: ColumnSort,
        _window: &mut Window, _cx: &mut Context<'_, TableState<Self>>,
    ) {
        let column=self.presentation[col_ix].key.to_string();
        let sorts=match sort {
            ColumnSort::Ascending => vec![Sort::ascending(column)],
            ColumnSort::Descending => vec![Sort::descending(column)],
            ColumnSort::Default => vec![],
        };
        let _=self.set_sort(sorts);
    }

    fn move_column(
        &mut self, col_ix: usize, to_ix: usize,
        _window: &mut Window, _cx: &mut Context<'_, TableState<Self>>,
    ) {
        if col_ix >= self.presentation.len() || to_ix >= self.presentation.len() { return; }
        let column=self.presentation.remove(col_ix);
        self.presentation.insert(to_ix,column);
        // Keep the authoritative column state in sync with GPUI's drag order.
        let moved_key = self.presentation[to_ix].key.to_string();
        let _ = self.model.move_column(&moved_key, to_ix);
        self.refresh_selection_rect();
    }

    fn cell_text(&self, row_ix: usize, col_ix: usize, _cx: &App) -> String {
        self.visible.get(row_ix).and_then(|&index| {
            self.model.value(index, &self.presentation[col_ix].key.to_string())
        }).map(Value::text).unwrap_or_default()
    }
}

/// One GPUI Render tree for both native and WebAssembly targets.
pub mod showcase;
pub use showcase::GridShowcase;
