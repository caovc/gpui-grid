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
use std::{cell::RefCell, rc::Rc};
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::*;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::component::table::{
    Column as UiColumn, ColumnSort, TableDelegate, TableState,
};

pub use gpui_grid_core::{
    CellAddress, Column as GridColumn, ColumnState, DataType, Edit, Filter, FilterOp, parse_cell_value,
    CellRange, SelectionRect, GridApi, GridEvent, GridOptions, GridState, PinSide, Row, sample_orders,
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
}

impl GridDelegate {
    pub fn new(model: GridModel) -> Self {
        // GPUI elements are a transient cache; GridModel.state owns the order,
        // width, pin and visibility independently of the virtual viewport.
        let presentation = model.state.columns.iter().filter(|col| col.visible).map(|state| {
            let col = &model.columns[model.column_index(&state.key).expect("state column key validated")];
            let base = UiColumn::new(col.key.clone(), col.name.clone())
                .width(px(state.width))
                .min_width(px(76.0))
                .max_width(px(580.0))
                .sortable();
            if state.pin == PinSide::Left { base.fixed_left() } else { base }
        }).collect();
        let visible = model.view();
        let mut grid = Self {
            model, presentation, visible, selection_rect: None,
            inline_editor: Rc::new(RefCell::new(None)),
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

    /// Find a currently visible cell using stable identities rather than viewport offsets.
    pub fn cell_info_by_address(&self, address: &CellAddress) -> Option<CellInfo> {
        let row_ix = self.visible.iter().position(|&ix| {
            self.model.row(ix).is_some_and(|row| row.id == address.row_id)
        })?;
        let col_ix = self.presentation.iter().position(|col| col.key.to_string() == address.column_key)?;
        self.cell_info(row_ix, col_ix)
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
        self.rebuild_view();
        Ok(())
    }

    pub fn edit(&mut self, edits: &[gpui_grid_core::Edit]) -> Result<usize, GridError> {
        let count = self.model.apply_edits(edits)?;
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
                .bg(rgb(0xf3f6ff)).border_2().border_color(rgb(0x5269e9))
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
            .when(in_range, |element| element.bg(rgb(0xe4efff)));
        match key.as_str() {
            "order_id" => base.text_color(rgb(0x5269e9))
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
            "amount" => base.justify_end().text_color(rgb(0x263854))
                .child(SharedString::from(match value {
                    Value::Number(n) => format!("${n:.2}"),
                    _ => label,
                })).into_any_element(),
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
