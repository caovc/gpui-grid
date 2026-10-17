//! Public, UI-independent grid configuration, state, and event contracts.
//!
//! The data model is the sole owner of this state. The GPUI TableDelegate
//! caches *presentation* elements only; focus, sort, filter and column order
//! are keyed by stable row IDs and column keys (never viewport indices).
use super::{Column, Edit, Filter, GridError, GridModel, Row, Sort};
use crate::selection::CellRange;

/// Immutable construction options, distinct from runtime [`GridState`].
#[derive(Clone, Debug)]
pub struct GridOptions {
    pub columns: Vec<Column>,
    pub rows: Vec<Row>,
}
impl GridOptions {
    pub fn new(columns: Vec<Column>) -> Self {
        Self { columns, rows: Vec::new() }
    }
    pub fn rows(mut self, rows: Vec<Row>) -> Self {
        self.rows = rows;
        self
    }
    pub fn build(self) -> Result<GridModel, GridError> {
        GridModel::new(self.columns, self.rows)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PinSide {
    None,
    Left,
    Right,
}

/// Persistable column presentation state. 
/// The authoritative column data type/editability remain in `GridModel.columns`.
#[derive(Clone, Debug, PartialEq)]
pub struct ColumnState {
    pub key: String,
    pub width: f32,
    pub visible: bool,
    pub pin: PinSide,
}
impl ColumnState {
    pub fn from_column(col: &Column) -> Self {
        Self {
            key: col.key.clone(),
            width: col.width,
            visible: true,
            pin: if col.pinned { PinSide::Left } else { PinSide::None },
        }
    }
}

/// Stable cell identity across sorting, filtering and viewport recycling.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CellAddress {
    pub row_id: u64,
    pub column_key: String,
}
impl CellAddress {
    pub fn new(row_id: u64, column_key: impl Into<String>) -> Self {
        Self { row_id, column_key: column_key.into() }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GridState {
    pub sorts: Vec<Sort>,
    pub filters: Vec<Filter>,
    pub quick_filter: String,
    pub columns: Vec<ColumnState>,
    pub focus: Option<CellAddress>,
    /// Stable rectangular selection, projected into visible viewport coordinates on demand.
    pub selection: Option<CellRange>,
}
impl GridState {
    pub fn new(columns: &[Column]) -> Self {
        Self {
            sorts: Vec::new(),
            filters: Vec::new(),
            quick_filter: String::new(),
            columns: columns.iter().map(ColumnState::from_column).collect(),
            focus: None,
            selection: None,
        }
    }
    pub fn column(&self, key: &str) -> Option<&ColumnState> {
        self.columns.iter().find(|col| col.key == key)
    }
    pub fn visible_column_keys(&self) -> impl Iterator<Item = &str> {
        self.columns.iter().filter(|col| col.visible).map(|col| col.key.as_str())
    }
}

/// Buffered notifications; drain after each UI operation to feed subscribers.
#[derive(Clone, Debug, PartialEq)]
pub enum GridEvent {
    SortChanged(Vec<Sort>),
    FiltersChanged(Vec<Filter>),
    QuickFilterChanged(String),
    ColumnMoved { key: String, from: usize, to: usize },
    ColumnVisibilityChanged { key: String, visible: bool },
    ColumnWidthChanged { key: String, width: f32 },
    FocusChanged { previous: Option<CellAddress>, current: Option<CellAddress> },
    CellsEdited(Vec<CellAddress>),
    SelectionChanged(Option<CellRange>),
}

/// Small command facade: every mutation runs the same validation as the
/// underlying model and emits exactly the same `GridEvent` changes.
/// GPUI adapters can use `GridDelegate::model.api()` without coupling Core to GPUI.
pub struct GridApi<'a> {
    model: &'a mut GridModel,
}
impl<'a> GridApi<'a> {
    pub(crate) fn new(model: &'a mut GridModel) -> Self { Self { model } }
    pub fn state(&self) -> &GridState { &self.model.state }
    pub fn set_sorts(&mut self, sorts: Vec<Sort>) -> Result<(), GridError> { self.model.set_sorts(sorts) }
    pub fn set_filters(&mut self, filters: Vec<Filter>) -> Result<(), GridError> { self.model.set_filters(filters) }
    pub fn set_quick_filter(&mut self, query: impl Into<String>) { self.model.set_quick_filter(query); }
    pub fn move_column(&mut self, key: &str, to: usize) -> Result<(), GridError> { self.model.move_column(key, to) }
    pub fn set_column_visible(&mut self, key: &str, visible: bool) -> Result<(), GridError> { self.model.set_column_visible(key, visible) }
    pub fn set_column_width(&mut self, key: &str, width: f32) -> Result<(), GridError> { self.model.set_column_width(key, width) }
    pub fn focus_cell(&mut self, cell: Option<CellAddress>) -> Result<(), GridError> { self.model.focus_cell(cell) }
    pub fn select_cell(&mut self, cell: Option<CellAddress>, extend: bool) -> Result<(), GridError> {
        self.model.select_cell(cell, extend)
    }
    pub fn select_range(&mut self, range: Option<CellRange>) -> Result<(), GridError> {
        self.model.select_range(range)
    }
    pub fn edit(&mut self, edits: &[Edit]) -> Result<usize, GridError> { self.model.apply_edits(edits) }
    pub fn drain_events(&mut self) -> Vec<GridEvent> { self.model.drain_events() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DataType, Value};

    fn model() -> GridModel {
        GridOptions::new(vec![
            Column::new("name", "Name", DataType::Text),
            Column::new("score", "Score", DataType::Number),
            Column::new("id", "ID", DataType::Text).readonly(),
        ]).rows(vec![
            Row::new(10, vec![Value::from("Zoe"), Value::from(2.0), Value::from("10")]),
            Row::new(20, vec![Value::from("Ada"), Value::from(4.0), Value::from("20")]),
        ]).build().unwrap()
    }

    #[test]
    fn state_columns_are_keyed_and_stable_after_moves() {
        let mut model = model();
        assert_eq!(model.state.visible_column_keys().collect::<Vec<_>>(), vec!["name", "score", "id"]);
        model.move_column("id", 0).unwrap();
        assert_eq!(model.state.visible_column_keys().collect::<Vec<_>>(), vec!["id", "name", "score"]);
        model.set_column_visible("score", false).unwrap();
        assert_eq!(model.state.visible_column_keys().collect::<Vec<_>>(), vec!["id", "name"]);
        // Presentation order must never mutate typed storage order.
        assert_eq!(model.column_index("name"), Some(0));
        assert_eq!(model.row_by_id(10).unwrap().cells[0], Value::from("Zoe"));
    }

    #[test]
    fn focus_stays_with_row_id_on_sort_and_filter() {
        let mut model = model();
        let focus = CellAddress::new(10, "name");
        model.focus_cell(Some(focus.clone())).unwrap();
        model.set_sorts(vec![Sort::ascending("name")]).unwrap();
        assert_eq!(model.view(), vec![1, 0]);
        assert_eq!(model.state.focus, Some(focus.clone()));
        model.set_quick_filter("ada");
        assert_eq!(model.view(), vec![1]);
        assert_eq!(model.state.focus, Some(focus));
        assert_eq!(model.focus_cell(Some(CellAddress::new(999, "name"))), Err(GridError::UnknownRow(999)));
        assert_eq!(model.focus_cell(Some(CellAddress::new(10, "not-a-column"))), Err(GridError::UnknownColumn("not-a-column".into())));
    }

    #[test]
    fn events_are_ordered_only_after_valid_mutations() {
        let mut model = model();
        {
            let mut api = model.api();
            api.set_sorts(vec![Sort::descending("score")]).unwrap();
            api.set_quick_filter("ada");
            assert!(api.set_column_width("score", f32::NAN).is_err());
            assert!(api.set_filters(vec![Filter { column: "missing".into(), op: crate::FilterOp::Equals, value: Value::Empty }]).is_err());
            assert_eq!(api.drain_events(), vec![
                GridEvent::SortChanged(vec![Sort::descending("score")]),
                GridEvent::QuickFilterChanged("ada".into()),
            ]);
            assert!(api.drain_events().is_empty());
            api.edit(&[Edit { row_id: 10, column: "name".into(), value: Value::from("Changed") }]).unwrap();
            assert_eq!(api.drain_events(), vec![GridEvent::CellsEdited(vec![CellAddress::new(10, "name")])]);
        }
        assert_eq!(model.value(0, "name"), Some(&Value::from("Changed")));
    }

    #[test]
    fn failed_edit_is_atomic_and_emits_no_event() {
        let mut model = model();
        let edits = [
            Edit { row_id: 10, column: "name".into(), value: Value::from("Updated") },
            Edit { row_id: 10, column: "id".into(), value: Value::from("Readonly") },
        ];
        assert!(matches!(model.api().edit(&edits), Err(GridError::ReadOnly(_))));
        assert!(model.drain_events().is_empty());
        assert_eq!(model.value(0, "name"), Some(&Value::from("Zoe")));
    }
}
