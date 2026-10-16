//! Core data model for GPUI Grid.
//!
//! Intentionally independent of GPUI: sort/filter/edit/group/CSV are testable
//! without a graphics backend, making the same model usable in batch jobs.
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::{Display, Formatter};

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
pub enum FilterOp { Contains, Equals, GreaterThan, LessThan }
#[derive(Clone, Debug, PartialEq)]
pub struct Filter { pub column: String, pub op: FilterOp, pub value: Value }
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
    pub quick_filter: String,
    pub filters: Vec<Filter>,
    pub sorts: Vec<Sort>,
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
        Ok(Self { columns, rows, id_to_index, quick_filter: String::new(), filters: Vec::new(), sorts: Vec::new() })
    }
    pub fn len(&self) -> usize { self.rows.len() }
    pub fn is_empty(&self) -> bool { self.rows.is_empty() }
    pub fn row(&self, index: usize) -> Option<&Row> { self.rows.get(index) }
    pub fn row_by_id(&self, id: u64) -> Option<&Row> { self.id_to_index.get(&id).and_then(|&ix|self.rows.get(ix)) }
    pub fn column_index(&self, key: &str) -> Option<usize> { self.columns.iter().position(|c|c.key==key) }
    pub fn value(&self, row_index: usize, column_key: &str) -> Option<&Value> {
        self.column_index(column_key).and_then(|col|self.rows.get(row_index)?.cells.get(col))
    }
    pub fn set_quick_filter(&mut self, query: impl Into<String>) { self.quick_filter=query.into(); }
    pub fn set_filters(&mut self, filters: Vec<Filter>) -> Result<(), GridError> {
        for f in &filters { if self.column_index(&f.column).is_none() {return Err(GridError::UnknownColumn(f.column.clone()));} }
        self.filters = filters; Ok(())
    }
    pub fn set_sorts(&mut self, sorts: Vec<Sort>) -> Result<(), GridError> {
        for s in &sorts { if self.column_index(&s.column).is_none() {return Err(GridError::UnknownColumn(s.column.clone()));} }
        self.sorts = sorts; Ok(())
    }
    pub fn view(&self) -> Vec<usize> {
        let quick=self.quick_filter.trim().to_lowercase();
        let selected: Vec<(usize, &Filter)> = self.filters.iter().filter_map(|f|self.column_index(&f.column).map(|ix|(ix,f))).collect();
        let mut indices = (0..self.rows.len()).filter(|&ix| {
            let row=&self.rows[ix];
            if !quick.is_empty() && !row.cells.iter().any(|cell|cell.contains_case_insensitive(&quick)) {return false;}
            selected.iter().all(|(col,filter)| {
                let cell=&row.cells[*col];
                match filter.op {
                    FilterOp::Contains => cell.contains_case_insensitive(&filter.value.text()),
                    FilterOp::Equals => cell==&filter.value,
                    FilterOp::GreaterThan => matches!((cell.as_number(),filter.value.as_number()),(Some(a),Some(b)) if a>b),
                    FilterOp::LessThan => matches!((cell.as_number(),filter.value.as_number()),(Some(a),Some(b)) if a<b),
                }
            })
        }).collect::<Vec<_>>();
        let sorts: Vec<(usize,bool)> = self.sorts.iter().filter_map(|s|self.column_index(&s.column).map(|ix|(ix,s.descending))).collect();
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
}
