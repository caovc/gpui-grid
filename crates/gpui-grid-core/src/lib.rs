//! Core data model for GPUI Grid.
//!
//! Intentionally independent of GPUI: sort/filter/edit/group/CSV are testable
//! without a graphics backend, making the same model usable in batch jobs.
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::{Display, Formatter};

mod edit;
pub use edit::parse_cell_value;
mod state;
mod selection;
pub use state::{CellAddress, ColumnState, GridApi, GridEvent, GridOptions, GridState, PinSide};
pub use selection::{CellRange, SelectionRect};

/// Typed value; use numbers for numeric sorting, not formatted strings.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Text(String),
    Number(f64),
    Boolean(bool),
    Empty,
}

impl Value {
    pub fn text(&self) -> String {
        match self {
            Self::Text(s) => s.clone(),
            Self::Number(n) => {
                if n.fract() == 0.0 { format!("{n:.0}") } else { format!("{n}") }
            }
            Self::Boolean(b) => b.to_string(),
            Self::Empty => String::new(),
        }
    }
    pub fn contains_case_insensitive(&self, search: &str) -> bool {
        self.text().to_lowercase().contains(&search.to_lowercase())
    }
    pub fn as_number(&self) -> Option<f64> {
        match self { Self::Number(n) => Some(*n), _ => None }
    }
}

impl From<&str> for Value { fn from(v: &str) -> Self { Self::Text(v.to_owned()) } }
impl From<String> for Value { fn from(v: String) -> Self { Self::Text(v) } }
impl From<f64> for Value { fn from(v: f64) -> Self { Self::Number(v) } }
impl From<bool> for Value { fn from(v: bool) -> Self { Self::Boolean(v) } }

/// Rows are column-ordered vectors to avoid allocating a map for every cell.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub id: u64,
    pub cells: Vec<Value>,
}
impl Row { pub fn new(id: u64, cells: Vec<Value>) -> Self { Self { id, cells } } }

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DataType { Text, Number, Boolean }

#[derive(Clone, Debug, PartialEq)]
pub struct Column {
    pub key: String,
    pub name: String,
    pub width: f32,
    pub data_type: DataType,
    pub editable: bool,
    pub pinned: bool,
}
impl Column {
    pub fn new(key: impl Into<String>, name: impl Into<String>, data_type: DataType) -> Self {
        Self { key: key.into(), name: name.into(), width: 140.0, data_type, editable: true, pinned: false }
    }
    pub fn width(mut self, width: f32) -> Self { self.width = width.max(40.0); self }
    pub fn pinned(mut self) -> Self { self.pinned = true; self }
    pub fn readonly(mut self) -> Self { self.editable = false; self }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sort { pub column: String, pub descending: bool }
impl Sort {
    pub fn ascending(column: impl Into<String>) -> Self { Self { column: column.into(), descending: false } }
    pub fn descending(column: impl Into<String>) -> Self { Self { column: column.into(), descending: true } }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FilterOp {
    Contains, Equals, GreaterThan, LessThan,
    /// Inclusive OR within one column; different columns remain AND.
    /// Filter::value is deliberately Value::Empty for this variant.
    OneOf(Vec<Value>),
}
#[derive(Clone, Debug, PartialEq)]
pub struct Filter { pub column: String, pub op: FilterOp, pub value: Value }
impl Filter {
    /// A checked set of typed values for a categorical column. Rust Core
    /// validates the schema and that this list is nonempty before persisting.
    pub fn one_of(column: impl Into<String>, values: Vec<Value>) -> Self {
        Self { column: column.into(), op: FilterOp::OneOf(values), value: Value::Empty }
    }
}

/// One categorical option together with its frequency in raw rows.
#[derive(Clone, Debug, PartialEq)]
pub struct DistinctValue { pub value: Value, pub count: usize }
#[derive(Clone, Debug, PartialEq)]
pub struct Edit { pub row_id: u64, pub column: String, pub value: Value }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridError {
    DuplicateRowId(u64),
    DuplicateColumn(String),
    MismatchedColumns(u64),
    UnknownColumn(String),
    UnknownRow(u64),
    ReadOnly(String),
    InvalidValue(String),
    InvalidColumnWidth(String),
}
impl Display for GridError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for GridError {}

#[derive(Clone, Debug, PartialEq)]
pub struct RowGroup {
    pub key: String,
    pub row_indices: Vec<usize>,
    pub amount_sum: f64,
}
impl RowGroup { pub fn count(&self) -> usize { self.row_indices.len() } }

/// Owns the data and exposes stable-index query results for the GPUI viewport.
/// Column IDs are authoritative: a renderer may reorder presentation columns freely.
#[derive(Clone, Debug)]
pub struct GridModel {
    pub columns: Vec<Column>,
    rows: Vec<Row>,
    id_to_index: HashMap<u64, usize>,
    /// Single source of truth for presentation and query state.
    pub state: GridState,
    pending_events: Vec<GridEvent>,
}
impl GridModel {
    pub fn new(columns: Vec<Column>, rows: Vec<Row>) -> Result<Self, GridError> {
        let mut keys = HashSet::new();
        for col in &columns {
            if !keys.insert(col.key.clone()) { return Err(GridError::DuplicateColumn(col.key.clone())); }
        }
        let mut id_to_index = HashMap::with_capacity(rows.len());
        for (ix, row) in rows.iter().enumerate() {
            if row.cells.len() != columns.len() { return Err(GridError::MismatchedColumns(row.id)); }
            if id_to_index.insert(row.id, ix).is_some() { return Err(GridError::DuplicateRowId(row.id)); }
        }
        let state = GridState::new(&columns);
        Ok(Self { columns, rows, id_to_index, state, pending_events: Vec::new() })
    }
    pub fn len(&self) -> usize { self.rows.len() }
    pub fn is_empty(&self) -> bool { self.rows.is_empty() }
    pub fn row(&self, index: usize) -> Option<&Row> { self.rows.get(index) }
    pub fn row_by_id(&self, id: u64) -> Option<&Row> { self.id_to_index.get(&id).and_then(|&ix|self.rows.get(ix)) }
    pub fn column_index(&self, key: &str) -> Option<usize> { self.columns.iter().position(|c|c.key==key) }
    pub fn value(&self, row_index: usize, column_key: &str) -> Option<&Value> {
        self.column_index(column_key).and_then(|col|self.rows.get(row_index)?.cells.get(col))
    }
    /// Stateful command façade; operations are validated and eventful.
    pub fn api(&mut self) -> GridApi<'_> { GridApi::new(self) }
    /// Read-only view of authoritative runtime state.
    pub fn state(&self) -> &GridState { &self.state }
    /// Drain ordered events to deliver them to a UI/application event bus.
    pub fn drain_events(&mut self) -> Vec<GridEvent> { std::mem::take(&mut self.pending_events) }

    pub fn set_quick_filter(&mut self, query: impl Into<String>) {
        let next = query.into();
        if self.state.quick_filter != next {
            self.state.quick_filter = next.clone();
            self.pending_events.push(GridEvent::QuickFilterChanged(next));
        }
    }
    /// All filter models are validated before touching GridState; invalid
    /// column/type/value combinations never leave a half-applied view.
    fn validate_filter(&self, filter: &Filter) -> Result<(), GridError> {
        let col_ix = self.column_index(&filter.column)
            .ok_or_else(|| GridError::UnknownColumn(filter.column.clone()))?;
        let schema = self.columns[col_ix].data_type;
        let typed = |value: &Value| -> bool {
            match (schema, value) {
                (DataType::Text, Value::Text(_)) |
                (DataType::Number, Value::Empty) |
                (DataType::Text, Value::Empty) |
                (DataType::Boolean, Value::Empty) |
                (DataType::Boolean, Value::Boolean(_)) => true,
                (DataType::Number, Value::Number(n)) => n.is_finite(),
                _ => false,
            }
        };
        let valid = match (&filter.op, &filter.value) {
            (FilterOp::Contains, Value::Text(_)) => schema == DataType::Text,
            (FilterOp::Equals, value) => typed(value),
            (FilterOp::GreaterThan | FilterOp::LessThan, Value::Number(n)) =>
                schema == DataType::Number && n.is_finite(),
            (FilterOp::OneOf(values), Value::Empty) => {
                // An empty selection matches no rows. It is not equivalent
                // to clearing the filter, which intentionally matches all.
                values.iter().all(typed)
                    && !values.iter().enumerate().any(|(i, v)| values[..i].contains(v))
            },
            _ => false,
        };
        if valid { Ok(()) } else {
            Err(GridError::InvalidValue(format!(
                "invalid filter model for column {}", filter.column
            )))
        }
    }

    pub fn set_filters(&mut self, filters: Vec<Filter>) -> Result<(), GridError> {
        for filter in &filters { self.validate_filter(filter)?; }
        if self.state.filters != filters {
            self.state.filters = filters.clone();
            self.pending_events.push(GridEvent::FiltersChanged(filters));
        }
        Ok(())
    }
    /// Insert or replace a column's filter while preserving all other columns'
    /// filter conditions. Delegates should call this for a header filter menu,
    /// not `set_filters(vec![...])` which would wipe unrelated filters.
    pub fn upsert_column_filter(&mut self, filter: Filter) -> Result<(), GridError> {
        self.validate_filter(&filter)?;
        let mut next: Vec<Filter> = self.state.filters.iter()
            .filter(|candidate| candidate.column != filter.column)
            .cloned().collect();
        next.push(filter);
        self.set_filters(next)
    }

    /// Set Filter semantics: selected values are OR within the column;
    /// different columns remain AND. Empty means match zero rows, whereas
    /// clear_column_filter removes the predicate and shows all values.
    pub fn set_column_values(&mut self, key: &str, values: Vec<Value>) -> Result<(), GridError> {
        if self.column_index(key).is_none() {
            return Err(GridError::UnknownColumn(key.into()));
        }
        let mut normalized = Vec::with_capacity(values.len());
        for value in values {
            if !normalized.contains(&value) { normalized.push(value); }
        }
        self.upsert_column_filter(Filter::one_of(key, normalized))
    }

    /// Toggle a GPUI checkbox using current authoritative state; never trust
    /// a snapshot from the previous render of the virtualized header.
    pub fn toggle_column_value(&mut self, key: &str, value: Value) -> Result<(), GridError> {
        // Validate before changing an existing filter or losing user state.
        self.validate_filter(&Filter::one_of(key, vec![value.clone()]))?;
        let mut selected = match self.state.filters.iter().find(|f| f.column == key) {
            Some(Filter { op: FilterOp::OneOf(current), .. }) => current.clone(),
            _ => self.distinct_column_values(key, 12)?
                .ok_or_else(|| GridError::InvalidValue(format!(
                    "Set Filter supports up to 12 distinct values: {key}"
                )))?
                .into_iter().map(|item| item.value).collect(),
        };
        if let Some(index) = selected.iter().position(|v| v == &value) {
            selected.remove(index);
        } else {
            selected.push(value);
        }
        self.set_column_values(key, selected)
    }

    /// Distinct raw categorical values with counts, bounded to avoid enormous
    /// header menus. Returns None if the cardinality exceeds max_unique;
    /// callers cache results, never scanning 100k rows in render_th.
    pub fn distinct_column_values(
        &self, key: &str, max_unique: usize,
    ) -> Result<Option<Vec<DistinctValue>>, GridError> {
        let column = self.column_index(key)
            .ok_or_else(|| GridError::UnknownColumn(key.to_owned()))?;
        if max_unique == 0 { return Ok(None); }
        let mut result: Vec<DistinctValue> = Vec::new();
        for row in &self.rows {
            let value = &row.cells[column];
            if let Some(item) = result.iter_mut().find(|item| &item.value == value) {
                item.count += 1;
            } else {
                if result.len() == max_unique { return Ok(None); }
                result.push(DistinctValue { value: value.clone(), count: 1 });
            }
        }
        result.sort_by(|a, b| a.value.text().to_lowercase().cmp(&b.value.text().to_lowercase()));
        Ok(Some(result))
    }

    /// Clear only one column's condition. Unknown columns fail rather than
    /// silently producing a success notification.
    pub fn clear_column_filter(&mut self, key: &str) -> Result<(), GridError> {
        if self.column_index(key).is_none() {
            return Err(GridError::UnknownColumn(key.to_string()));
        }
        let remaining: Vec<Filter> = self.state.filters.iter()
            .filter(|filter| filter.column != key)
            .cloned().collect();
        self.set_filters(remaining)
    }

    pub fn set_sorts(&mut self, sorts: Vec<Sort>) -> Result<(), GridError> {
        for s in &sorts { if self.column_index(&s.column).is_none() {return Err(GridError::UnknownColumn(s.column.clone()));} }
        if self.state.sorts != sorts {
            self.state.sorts = sorts.clone();
            self.pending_events.push(GridEvent::SortChanged(sorts));
        }
        Ok(())
    }
    /// Move presentation order only; typed row storage remains unchanged.
    pub fn move_column(&mut self, key: &str, to: usize) -> Result<(), GridError> {
        let from = self.state.columns.iter().position(|col| col.key == key)
            .ok_or_else(|| GridError::UnknownColumn(key.to_owned()))?;
        let destination = to.min(self.state.columns.len().saturating_sub(1));
        if from != destination {
            let col = self.state.columns.remove(from);
            self.state.columns.insert(destination, col);
            self.pending_events.push(GridEvent::ColumnMoved {key: key.into(), from, to: destination});
        }
        Ok(())
    }
    pub fn set_column_visible(&mut self, key: &str, visible: bool) -> Result<(), GridError> {
        let column = self.state.columns.iter_mut().find(|col| col.key == key)
            .ok_or_else(|| GridError::UnknownColumn(key.to_owned()))?;
        if column.visible != visible {
            column.visible = visible;
            self.pending_events.push(GridEvent::ColumnVisibilityChanged {key: key.into(), visible});
        }
        Ok(())
    }
    pub fn set_column_width(&mut self, key: &str, width: f32) -> Result<(), GridError> {
        if !width.is_finite() || width < 40.0 || width > 2_000.0 {
            return Err(GridError::InvalidColumnWidth(key.to_owned()));
        }
        let column = self.state.columns.iter_mut().find(|col| col.key == key)
            .ok_or_else(|| GridError::UnknownColumn(key.to_owned()))?;
        if column.width != width {
            column.width = width;
            self.pending_events.push(GridEvent::ColumnWidthChanged {key: key.into(), width});
        }
        Ok(())
    }
    /// Focus is an optional stable row ID + column key, never a sorted index.
    pub fn focus_cell(&mut self, next: Option<CellAddress>) -> Result<(), GridError> {
        if let Some(cell) = &next {
            if !self.id_to_index.contains_key(&cell.row_id) {return Err(GridError::UnknownRow(cell.row_id));}
            if self.column_index(&cell.column_key).is_none() {return Err(GridError::UnknownColumn(cell.column_key.clone()));}
        }
        if self.state.focus != next {
            let previous = std::mem::replace(&mut self.state.focus, next.clone());
            self.pending_events.push(GridEvent::FocusChanged {previous, current: next});
        }
        Ok(())
    }
    pub fn view(&self) -> Vec<usize> {
        let quick=self.state.quick_filter.trim().to_lowercase();
        let selected: Vec<(usize, &Filter)> = self.state.filters.iter().filter_map(|f|self.column_index(&f.column).map(|ix|(ix,f))).collect();
        let mut indices = (0..self.rows.len()).filter(|&ix| {
            let row=&self.rows[ix];
            if !quick.is_empty() && !row.cells.iter().any(|cell|cell.contains_case_insensitive(&quick)) {return false;}
            selected.iter().all(|(col,filter)| {
                let cell=&row.cells[*col];
                match &filter.op {
                    FilterOp::Contains => cell.contains_case_insensitive(&filter.value.text()),
                    FilterOp::Equals => cell==&filter.value,
                    FilterOp::GreaterThan => matches!((cell.as_number(),filter.value.as_number()),(Some(a),Some(b)) if a>b),
                    FilterOp::LessThan => matches!((cell.as_number(),filter.value.as_number()),(Some(a),Some(b)) if a<b),
                    FilterOp::OneOf(values) => values.contains(cell),
                }
            })
        }).collect::<Vec<_>>();
        let sorts: Vec<(usize,bool)> = self.state.sorts.iter().filter_map(|s|self.column_index(&s.column).map(|ix|(ix,s.descending))).collect();
        if !sorts.is_empty() {
            // sort_by is stable; original order survives for equal values.
            indices.sort_by(|&a,&b| {
                for (col,descending) in &sorts {
                    let comparison=compare(&self.rows[a].cells[*col],&self.rows[b].cells[*col]);
                    if comparison!=std::cmp::Ordering::Equal {return if *descending {comparison.reverse()}else{comparison};}
                }
                std::cmp::Ordering::Equal
            });
        }
        indices
    }
    pub fn group_by(&self, indices: &[usize], key: &str, sum_column: &str) -> Result<Vec<RowGroup>,GridError> {
        let group_index=self.column_index(key).ok_or_else(||GridError::UnknownColumn(key.to_owned()))?;
        let sum_index=self.column_index(sum_column).ok_or_else(||GridError::UnknownColumn(sum_column.to_owned()))?;
        let mut groups:BTreeMap<String,RowGroup>=BTreeMap::new();
        for &ix in indices {
            let Some(row)=self.rows.get(ix) else {continue};
            let text=row.cells[group_index].text();
            let group=groups.entry(text.clone()).or_insert(RowGroup {key:text,row_indices:Vec::new(),amount_sum:0.0});
            group.row_indices.push(ix);
            group.amount_sum+=row.cells[sum_index].as_number().unwrap_or(0.0);
        }
        Ok(groups.into_values().collect())
    }
    /// Validate all edits before applying any, so a failed batch is atomic.
    pub fn apply_edits(&mut self, edits: &[Edit]) -> Result<usize, GridError> {
        let mut checked=Vec::with_capacity(edits.len());
        for edit in edits {
            let ix=*self.id_to_index.get(&edit.row_id).ok_or(GridError::UnknownRow(edit.row_id))?;
            let col=self.column_index(&edit.column).ok_or_else(||GridError::UnknownColumn(edit.column.clone()))?;
            let spec=&self.columns[col];
            if !spec.editable {return Err(GridError::ReadOnly(spec.key.clone()));}
            let valid=matches!((&spec.data_type,&edit.value),
                (DataType::Text,Value::Text(_)) | (DataType::Text,Value::Empty) |
                (DataType::Number,Value::Number(_)) | (DataType::Number,Value::Empty) |
                (DataType::Boolean,Value::Boolean(_)) | (DataType::Boolean,Value::Empty));
            if !valid || matches!(&edit.value,Value::Number(n) if !n.is_finite()) {return Err(GridError::InvalidValue(spec.key.clone()));}
            checked.push((ix,col));
        }
        for (edit,(ix,col)) in edits.iter().zip(checked) {self.rows[ix].cells[col]=edit.value.clone();}
        if !edits.is_empty() {
            self.pending_events.push(GridEvent::CellsEdited(edits.iter().map(|edit|
                CellAddress::new(edit.row_id, &edit.column)).collect()));
        }
        Ok(edits.len())
    }
    pub fn export_csv(&self, row_indices: &[usize], column_keys: &[&str]) -> Result<String,GridError> {
        let cols=column_keys.iter().map(|key|self.column_index(key).ok_or_else(||GridError::UnknownColumn((*key).to_owned()))).collect::<Result<Vec<_>,_>>()?;
        let mut csv=String::new();
        for (i,&ix) in cols.iter().enumerate() {if i>0{csv.push(',');}csv.push_str(&csv_quote(&self.columns[ix].name,false));}
        csv.push_str("\r\n");
        for &row_ix in row_indices {
            let Some(row)=self.rows.get(row_ix) else {continue};
            for (i,&col_ix) in cols.iter().enumerate(){
                if i>0{csv.push(',');}
                csv.push_str(&csv_quote(&row.cells[col_ix].text(),matches!(row.cells[col_ix],Value::Text(_))));
            }
            csv.push_str("\r\n");
        }
        Ok(csv)
    }
}
fn compare(a:&Value,b:&Value)->std::cmp::Ordering{
    use std::cmp::Ordering;
    match(a,b){
        (Value::Empty,Value::Empty)=>Ordering::Equal,
        (Value::Empty,_)=>Ordering::Less,(_,Value::Empty)=>Ordering::Greater,
        (Value::Number(a),Value::Number(b))=>a.total_cmp(b),
        (Value::Boolean(a),Value::Boolean(b))=>a.cmp(b),
        (Value::Text(a),Value::Text(b))=>a.to_lowercase().cmp(&b.to_lowercase()),
        (a,b)=>a.text().cmp(&b.text()),
    }
}
fn csv_quote(value:&str,formula_guard:bool)->String{
    let mut safe=value.to_owned();
    if formula_guard && value.trim_start().chars().next().is_some_and(|c| matches!(c, '=' | '+' | '-' | '@' | '\t' | '\r')) {safe.insert(0,'\'');}
    if safe.chars().any(|c| matches!(c, ',' | '"' | '\r' | '\n')){format!("\"{}\"",safe.replace('"',"\"\""))}else{safe}
}
/// Deterministic dataset used by the native GPUI demo and Rust tests.
pub fn sample_orders(count:usize)->GridModel{
    const PEOPLE:&[&str]=&["Sophia Chen","Liam Carter","Olivia Martinez","James Anderson","Emma Thompson","Noah Williams","Ava Patel","Mia Robinson"];
    const COMPANIES:&[&str]=&["Linear","Vercel","Figma","Notion","Stripe","Ramp","Mercury","Retool"];
    const REGIONS:&[&str]=&["North America","Europe","APAC","Latin America","Middle East"];
    const STATUSES:&[&str]=&["Completed","Processing","Pending","Cancelled"];
    let cols=vec![
        Column::new("order_id","Order ID",DataType::Text).width(145.0).readonly().pinned(),
        Column::new("customer","Customer",DataType::Text).width(195.0).pinned(),
        Column::new("company","Company",DataType::Text).width(175.0),
        Column::new("region","Region",DataType::Text).width(170.0),
        Column::new("amount","Revenue",DataType::Number).width(152.0),
        Column::new("growth","Growth %",DataType::Number).width(125.0),
        Column::new("status","Status",DataType::Text).width(145.0),
        Column::new("units","Units",DataType::Number).width(105.0),
        Column::new("score","Health Score",DataType::Number).width(145.0),
        Column::new("verified","Verified",DataType::Boolean).width(112.0),
    ];
    let mut rows=Vec::with_capacity(count);
    for i in 0..count {
        let mut n=(i as u64).wrapping_add(37);
        n^=n>>16;n=n.wrapping_mul(0x045d9f3b);n^=n>>16;n=n.wrapping_mul(0x045d9f3b);n^=n>>16;
        rows.push(Row::new(i as u64+1,vec![
            Value::from(format!("ORD-{:06}",i+10420)),
            Value::from(PEOPLE[(n as usize)%PEOPLE.len()]),
            Value::from(COMPANIES[((n>>3) as usize)%COMPANIES.len()]),
            Value::from(REGIONS[((n>>9) as usize)%REGIONS.len()]),
            Value::from(((((n%182000) as f64)/11.0+840.0)*100.0).round()/100.0),
            Value::from(((n>>8)%600) as f64/10.0-17.0),
            Value::from(STATUSES[((n>>17) as usize)%STATUSES.len()]),
            Value::from(((n>>4)%300+1) as f64),
            Value::from(((n>>6)%76+25) as f64),
            Value::from((n & 1) == 0),
        ]));
    }
    GridModel::new(cols,rows).expect("generated sample data must be valid")
}

#[cfg(test)]
mod tests {
    use super::*;
    fn model()->GridModel{
        GridModel::new(vec![Column::new("name","Name",DataType::Text),Column::new("sum","Sum",DataType::Number),Column::new("id","ID",DataType::Text).readonly()],vec![
            Row::new(1,vec!["Zoe".into(),30.0.into(),"1".into()]),
            Row::new(2,vec!["Ada".into(),20.0.into(),"2".into()]),
            Row::new(3,vec!["Ada".into(),10.0.into(),"3".into()]),
        ]).unwrap()
    }
    #[test]
    fn header_filter_replaces_only_target_column_and_is_atomic() {
        let mut m = model();
        m.upsert_column_filter(Filter {
            column: "name".into(), op: FilterOp::Contains, value: Value::Text("a".into())
        }).unwrap();
        m.upsert_column_filter(Filter {
            column: "sum".into(), op: FilterOp::GreaterThan, value: Value::Number(15.0)
        }).unwrap();
        assert_eq!(m.view(), vec![1]);
        let before = m.state.filters.clone();
        m.drain_events();
        assert!(matches!(m.upsert_column_filter(Filter {
            column: "sum".into(), op: FilterOp::GreaterThan, value: Value::Text("bad".into())
        }), Err(GridError::InvalidValue(_))));
        assert_eq!(m.state.filters, before);
        assert!(m.drain_events().is_empty());
        m.upsert_column_filter(Filter {
            column: "sum".into(), op: FilterOp::LessThan, value: Value::Number(25.0)
        }).unwrap();
        assert_eq!(m.state.filters.len(), 2, "replace, do not append duplicate constraints");
        assert_eq!(m.view(), vec![1, 2]);
        m.clear_column_filter("sum").unwrap();
        assert_eq!(m.state.filters.len(), 1);
        assert_eq!(m.view(), vec![1, 2]);
        assert!(matches!(m.clear_column_filter("unknown"), Err(GridError::UnknownColumn(_))));
    }

    #[test]
    fn header_filters_validate_schema_and_keep_other_columns_intact() {
        let mut m = model();
        let expected = Filter { column: "name".into(), op: FilterOp::Equals, value: Value::Text("Ada".into()) };
        m.upsert_column_filter(expected.clone()).unwrap();
        for bad in [
            Filter { column: "name".into(), op: FilterOp::GreaterThan, value: Value::Number(3.0) },
            Filter { column: "sum".into(), op: FilterOp::Contains, value: Value::Text("3".into()) },
            Filter { column: "sum".into(), op: FilterOp::Equals, value: Value::Number(f64::NAN) },
        ] {
            assert!(matches!(m.upsert_column_filter(bad), Err(GridError::InvalidValue(_))));
            assert_eq!(m.state.filters, vec![expected.clone()]);
        }
        m.upsert_column_filter(Filter {
            column: "sum".into(), op: FilterOp::GreaterThan, value: Value::Number(15.0)
        }).unwrap();
        assert_eq!(m.view(), vec![1]);
    }

    #[test]
    fn set_filter_uses_or_within_one_column_and_and_across_columns() {
        let mut model = sample_orders(400);
        model.set_column_values("status", vec!["Completed".into(), "Pending".into()]).unwrap();
        let view = model.view();
        assert!(!view.is_empty() && view.len() < 400);
        assert!(view.iter().all(|&i| matches!(
            model.value(i, "status"),
            Some(Value::Text(s)) if s == "Completed" || s == "Pending"
        )));
        model.upsert_column_filter(Filter {
            column:"amount".into(), op:FilterOp::GreaterThan,
            value:Value::Number(2_000.0)
        }).unwrap();
        assert_eq!(model.state.filters.len(), 2);
        assert!(model.view().iter().all(|&i|
            model.value(i,"amount").and_then(Value::as_number).unwrap_or(0.) > 2_000.));
        model.toggle_column_value("status", "Processing".into()).unwrap();
        assert_eq!(model.state.filters.len(), 2, "other-column predicate preserved");
        assert!(matches!(&model.state.filters.iter().find(|f|f.column=="status").unwrap().op,
            FilterOp::OneOf(v) if v.len() == 3));
        model.toggle_column_value("status", "Pending".into()).unwrap();
        assert!(matches!(&model.state.filters.iter().find(|f|f.column=="status").unwrap().op,
            FilterOp::OneOf(v) if v.len() == 2));
    }

    #[test]
    fn set_filter_rejects_bad_types_and_mutations_atomically() {
        let mut model = sample_orders(30);
        model.set_column_values("status", vec!["Completed".into()]).unwrap();
        model.drain_events();
        let prior = model.state.clone();
        for bad in [
            Filter::one_of("status", vec![Value::Number(3.0)]),
            Filter::one_of("amount", vec![Value::Number(f64::NAN)]),
            Filter::one_of("status", vec![Value::Text("A".into()), Value::Text("A".into())]),
        ] {
            assert!(matches!(model.upsert_column_filter(bad), Err(GridError::InvalidValue(_))));
            assert_eq!(model.state, prior);
            assert!(model.drain_events().is_empty());
        }
        assert!(matches!(model.toggle_column_value("missing","x".into()), Err(GridError::UnknownColumn(_))));
        assert_eq!(model.state, prior);
        model.toggle_column_value("status", "Completed".into()).unwrap();
        assert_eq!(model.view().len(), 0, "unchecking final value selects none");
        assert!(matches!(&model.state.filters.iter().find(|f|f.column=="status").unwrap().op,
            FilterOp::OneOf(v) if v.is_empty()));
        model.clear_column_filter("status").unwrap();
        assert_eq!(model.view().len(), 30, "clear restores all values");
    }

    #[test]
    fn checkboxes_start_all_checked_and_toggle_excludes_one_value() {
        let mut model = sample_orders(500);
        let count = model.view().len();
        model.toggle_column_value("status", Value::from("Pending")).unwrap();
        let selected = model.state.filters.iter().find(|f| f.column == "status").unwrap();
        assert!(matches!(&selected.op, FilterOp::OneOf(v) if v.len()==3));
        assert!(model.view().len() < count);
        assert!(model.view().iter().all(|&i| model.value(i, "status") != Some(&Value::from("Pending"))));
        model.toggle_column_value("status", Value::from("Pending")).unwrap();
        let selected = model.state.filters.iter().find(|f| f.column == "status").unwrap();
        assert!(matches!(&selected.op, FilterOp::OneOf(v) if v.len()==4));
        assert_eq!(model.view().len(), count);
        assert!(
            matches!(
                model.toggle_column_value("order_id", Value::from("ORD-010420")),
                Err(GridError::InvalidValue(_))
            ),
            "high-cardinality Set Filter is not available"
        );
    }

    #[test]
    fn distinct_values_are_typed_counted_and_bounded() {
        let model = sample_orders(800);
        let status = model.distinct_column_values("status", 12).unwrap().unwrap();
        assert_eq!(status.len(), 4);
        assert_eq!(status.iter().map(|item|item.count).sum::<usize>(),800);
        assert!(status.iter().all(|item|matches!(item.value,Value::Text(_))));
        assert!(model.distinct_column_values("order_id", 12).unwrap().is_none());
        assert_eq!(model.distinct_column_values("verified", 12).unwrap().unwrap().len(),2);
        assert!(matches!(model.distinct_column_values("bad_column",12),Err(GridError::UnknownColumn(_))));
    }

    #[test] fn base_view(){let m=model();assert_eq!(m.view(),vec![0,1,2]);assert_eq!(m.value(0,"name"),Some(&Value::from("Zoe")));}
    #[test] fn stable_sort_and_multi_sort(){let mut m=model();m.set_sorts(vec![Sort::ascending("name"),Sort::descending("sum")]).unwrap();assert_eq!(m.view(),vec![1,2,0]);}
    #[test] fn quick_filter_case_insensitive(){let mut m=model();m.set_quick_filter("zoE");assert_eq!(m.view(),vec![0]);}
    #[test] fn numeric_filter(){let mut m=model();m.set_filters(vec![Filter{column:"sum".into(),op:FilterOp::GreaterThan,value:15.0.into()}]).unwrap();assert_eq!(m.view(),vec![0,1]);}
    #[test] fn grouping_sum(){let m=model();let groups=m.group_by(&m.view(),"name","sum").unwrap();assert_eq!(groups[0].key,"Ada");assert_eq!(groups[0].count(),2);assert_eq!(groups[0].amount_sum,30.0);}
    #[test] fn atomic_edits(){let mut m=model();let r=m.apply_edits(&[Edit{row_id:1,column:"name".into(),value:"Changed".into()},Edit{row_id:1,column:"id".into(),value:"NO".into()}]);assert!(matches!(r,Err(GridError::ReadOnly(_))));assert_eq!(m.value(0,"name"),Some(&Value::from("Zoe")));}
    #[test] fn successful_edit(){let mut m=model();assert_eq!(m.apply_edits(&[Edit{row_id:1,column:"sum".into(),value:99.0.into()}]),Ok(1));assert_eq!(m.value(0,"sum"),Some(&Value::from(99.0)));}
    #[test] fn invalid_type(){let mut m=model();assert!(matches!(m.apply_edits(&[Edit{row_id:1,column:"sum".into(),value:"bad".into()}]),Err(GridError::InvalidValue(_))));}
    #[test] fn duplicate_row_rejected(){let r=GridModel::new(vec![Column::new("k","K",DataType::Text)],vec![Row::new(1,vec!["a".into()]),Row::new(1,vec!["b".into()])]);assert!(matches!(r,Err(GridError::DuplicateRowId(1))));}
    #[test] fn csv_escape_and_formula_guard(){let mut m=model();m.apply_edits(&[Edit{row_id:1,column:"name".into(),value:Value::from("=SUM(1,2)")}]).unwrap();let csv=m.export_csv(&[0], &["name","sum"]).unwrap();assert!(csv.contains("\"'=SUM(1,2)\""));}
    #[test] fn generator_large(){let m=sample_orders(10_000);assert_eq!(m.len(),10_000);assert_eq!(m.view().len(),10_000);assert_eq!(m.row_by_id(10000).unwrap().id,10_000);}
    #[test] fn sample_dataset_exposes_boolean_editor_column(){
        let m=sample_orders(4);
        let col=m.column_index("verified").unwrap();
        assert_eq!(m.columns[col].data_type,DataType::Boolean);
        assert!(m.row(0).is_some_and(|row| matches!(&row.cells[col], Value::Boolean(_))));
    }
}
