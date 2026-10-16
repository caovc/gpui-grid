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
use gpui_kit::*;
use gpui_kit::component::table::{
    Column as UiColumn, ColumnSort, TableDelegate, TableState,
};

pub use gpui_grid_core::{Column as GridColumn, DataType, Edit, Filter, FilterOp, Row, sample_orders};

/// Use `cx.new(|cx| TableState::new(GridDelegate::new(model), window, cx))`
/// to instantiate the GPUI state. `DataTable::new(&entity)` performs native
/// row/column virtualization and keyboard interaction.
pub struct GridDelegate {
    pub model: GridModel,
    presentation: Vec<UiColumn>,
    visible: Vec<usize>,
}

impl GridDelegate {
    pub fn new(model: GridModel) -> Self {
        let presentation = model.columns.iter().map(|col| {
            let base = UiColumn::new(col.key.clone(), col.name.clone())
                .width(px(col.width))
                .min_width(px(76.0))
                .max_width(px(580.0))
                .sortable();
            if col.pinned { base.fixed_left() } else { base }
        }).collect();
        let visible = model.view();
        Self { model, presentation, visible }
    }

    pub fn row_count(&self) -> usize { self.visible.len() }

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

    pub fn rebuild_view(&mut self) { self.visible = self.model.view(); }

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
        let value = self.visible.get(row_ix)
            .and_then(|&index| self.model.value(index, &key))
            .cloned().unwrap_or(Value::Empty);
        // Data stays typed until the last moment, avoiding lexicographic money sort.
        let is_num = matches!(&value, Value::Number(_));
        let label = value.text();
        let base = div().flex().items_center().h_full().px_3().w_full();
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
