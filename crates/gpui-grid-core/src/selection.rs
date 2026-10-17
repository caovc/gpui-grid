//! Spreadsheet-style rectangular ranges, keyed by stable row and column IDs.
//!
//! A range persists across sort/filter changes, while its projected rectangle
//! is evaluated against the *current* ordered/visible row and column viewport.
use super::{CellAddress, GridError, GridEvent, GridModel};

/// Anchor/focus remain row IDs + column keys even when the viewport is reordered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CellRange {
    pub anchor: CellAddress,
    pub focus: CellAddress,
}

impl CellRange {
    pub fn single(address: CellAddress) -> Self {
        Self { anchor: address.clone(), focus: address }
    }
}

/// Inclusive viewport coordinates; never persisted as a cell's identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectionRect {
    pub top: usize,
    pub bottom: usize,
    pub left: usize,
    pub right: usize,
}
impl SelectionRect {
    pub fn new(row_a: usize, row_b: usize, col_a: usize, col_b: usize) -> Self {
        Self {
            top: row_a.min(row_b),
            bottom: row_a.max(row_b),
            left: col_a.min(col_b),
            right: col_a.max(col_b),
        }
    }
    pub fn contains(&self, row: usize, col: usize) -> bool {
        row >= self.top && row <= self.bottom && col >= self.left && col <= self.right
    }
    pub fn cell_count(&self) -> usize {
        (self.bottom - self.top + 1).saturating_mul(self.right - self.left + 1)
    }
}

impl GridModel {
    /// Select a cell; `extend` preserves the anchor and moves the focus.
    /// Validates the destination before changing focus or selection.
    pub fn select_cell(&mut self, cell: Option<CellAddress>, extend: bool) -> Result<(), GridError> {
        self.focus_cell(cell.clone())?;
        let next = cell.map(|address| {
            if extend {
                if let Some(old) = &self.state.selection {
                    return CellRange { anchor: old.anchor.clone(), focus: address };
                }
            }
            CellRange::single(address)
        });
        if self.state.selection != next {
            self.state.selection = next.clone();
            self.pending_events.push(GridEvent::SelectionChanged(next));
        }
        Ok(())
    }

    /// Replace a whole rectangular range atomically. Validate both endpoints
    /// first, so a bad row/column never changes focus or emits partial events.
    pub fn select_range(&mut self, next: Option<CellRange>) -> Result<(), GridError> {
        if let Some(range) = &next {
            for address in [&range.anchor, &range.focus] {
                if self.row_by_id(address.row_id).is_none() {
                    return Err(GridError::UnknownRow(address.row_id));
                }
                if self.column_index(&address.column_key).is_none() {
                    return Err(GridError::UnknownColumn(address.column_key.clone()));
                }
            }
        }
        self.focus_cell(next.as_ref().map(|range| range.focus.clone()))?;
        if self.state.selection != next {
            self.state.selection = next.clone();
            self.pending_events.push(GridEvent::SelectionChanged(next));
        }
        Ok(())
    }

    /// Project a stable range onto the current visible row/column order.
    /// If either endpoint is hidden by a filter or column visibility change,
    /// preserve the stored range but don't render a misleading rectangle.
    pub fn selected_rect(
        &self,
        view: &[usize],
        displayed_columns: &[String],
    ) -> Option<SelectionRect> {
        let range = self.state.selection.as_ref()?;
        let row_position = |id: u64| -> Option<usize> {
            view.iter().position(|&source| self.row(source).is_some_and(|row| row.id == id))
        };
        let column_position = |key: &str| -> Option<usize> {
            displayed_columns.iter().position(|column| column == key)
        };
        Some(SelectionRect::new(
            row_position(range.anchor.row_id)?,
            row_position(range.focus.row_id)?,
            column_position(&range.anchor.column_key)?,
            column_position(&range.focus.column_key)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Column, DataType, GridOptions, Row, Sort, Value};

    fn fixture() -> GridModel {
        GridOptions::new(vec![
            Column::new("a", "A", DataType::Text),
            Column::new("b", "B", DataType::Number),
            Column::new("c", "C", DataType::Text),
        ]).rows(vec![
            Row::new(10, vec![Value::from("alpha"), Value::from(1.0), Value::from("x")]),
            Row::new(20, vec![Value::from("beta"), Value::from(2.0), Value::from("y")]),
            Row::new(30, vec![Value::from("gamma"), Value::from(3.0), Value::from("z")]),
        ]).build().unwrap()
    }

    #[test]
    fn single_then_shift_extend_is_rectangular_and_emits_events() {
        let mut model = fixture();
        let first = CellAddress::new(10, "a");
        model.select_cell(Some(first.clone()), false).unwrap();
        let cols = vec!["a".into(), "b".into(), "c".into()];
        let rect = model.selected_rect(&model.view(), &cols).unwrap();
        assert_eq!(rect.cell_count(), 1);
        assert!(rect.contains(0, 0));
        model.select_cell(Some(CellAddress::new(30, "c")), true).unwrap();
        let rect = model.selected_rect(&model.view(), &cols).unwrap();
        assert_eq!(rect, SelectionRect::new(0, 2, 0, 2));
        assert_eq!(rect.cell_count(), 9);
        assert!(model.drain_events().iter().any(|evt|
            matches!(evt, GridEvent::SelectionChanged(Some(r)) if r.anchor == first)));
    }

    #[test]
    fn reverse_bounds_follow_current_sort_and_column_order() {
        let mut model = fixture();
        model.select_cell(Some(CellAddress::new(10, "a")), false).unwrap();
        model.select_cell(Some(CellAddress::new(20, "c")), true).unwrap();
        model.set_sorts(vec![Sort::descending("b")]).unwrap();
        let cols = vec!["c".into(), "b".into(), "a".into()];
        let view = model.view();
        assert_eq!(view, vec![2, 1, 0]);
        assert_eq!(model.selected_rect(&view, &cols), Some(SelectionRect::new(1, 2, 0, 2)));
        assert_eq!(model.state.selection.as_ref().unwrap().anchor.row_id, 10);
    }

    #[test]
    fn filtered_or_hidden_endpoints_do_not_retarget_the_selection() {
        let mut model = fixture();
        model.select_cell(Some(CellAddress::new(10, "a")), false).unwrap();
        model.select_cell(Some(CellAddress::new(20, "c")), true).unwrap();
        let cols = vec!["a".into(), "b".into(), "c".into()];
        model.set_quick_filter("gamma");
        assert_eq!(model.selected_rect(&model.view(), &cols), None);
        assert_eq!(model.state.selection.as_ref().unwrap().anchor.row_id, 10);
        model.set_quick_filter("");
        let visible_cols = vec!["a".into(), "b".into()];
        assert_eq!(model.selected_rect(&model.view(), &visible_cols), None);
        assert_eq!(model.selected_rect(&model.view(), &cols).unwrap().cell_count(), 6);
    }

    #[test]
    fn selecting_all_rows_uses_only_two_stable_endpoints() {
        let mut model = fixture();
        model.set_sorts(vec![Sort::descending("b")]).unwrap();
        let range = CellRange {
            anchor: CellAddress::new(30, "a"),
            focus: CellAddress::new(10, "c"),
        };
        model.select_range(Some(range.clone())).unwrap();
        assert_eq!(model.state.selection, Some(range));
        assert_eq!(model.selected_rect(&model.view(), &["a".into(), "b".into(), "c".into()]),
            Some(SelectionRect::new(0, 2, 0, 2)));
        assert_eq!(model.state.focus, Some(CellAddress::new(10, "c")));
    }

    #[test]
    fn range_validation_is_atomic_when_focus_is_invalid() {
        let mut model = fixture();
        model.select_cell(Some(CellAddress::new(10, "a")), false).unwrap();
        model.drain_events();
        let before = model.state.clone();
        let bad = CellRange {
            anchor: CellAddress::new(10, "a"),
            focus: CellAddress::new(999, "c"),
        };
        assert_eq!(model.select_range(Some(bad)), Err(GridError::UnknownRow(999)));
        assert_eq!(model.state, before);
        assert!(model.drain_events().is_empty());
    }

    #[test]
    fn invalid_endpoint_does_not_corrupt_range_or_event_log() {
        let mut model = fixture();
        model.select_cell(Some(CellAddress::new(10, "a")), false).unwrap();
        model.drain_events();
        let before = model.state.selection.clone();
        assert_eq!(
            model.select_cell(Some(CellAddress::new(999, "c")), true),
            Err(GridError::UnknownRow(999))
        );
        assert_eq!(model.state.selection, before);
        assert!(model.drain_events().is_empty());
        model.select_cell(None, false).unwrap();
        assert!(model.state.selection.is_none());
        assert!(model.selected_rect(&model.view(), &[]).is_none());
    }
}
