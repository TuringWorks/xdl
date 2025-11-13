//! DataFrame - pandas/Spark-style data structure for XDL

use crate::error::{DataFrameError, DataFrameResult};
use crate::series::Series;
use indexmap::IndexMap;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use xdl_core::{XdlResult, XdlValue};

/// DataFrame - A tabular data structure with labeled columns
#[derive(Debug, Clone)]
pub struct DataFrame {
    /// Column data (column_name -> Series)
    columns: IndexMap<String, Series>,
    /// Number of rows
    nrows: usize,
}

impl DataFrame {
    /// Create a new empty DataFrame
    pub fn new() -> Self {
        Self {
            columns: IndexMap::new(),
            nrows: 0,
        }
    }

    /// Create DataFrame from columns
    pub fn from_columns(columns: IndexMap<String, Series>) -> DataFrameResult<Self> {
        if columns.is_empty() {
            return Ok(Self::new());
        }

        // Verify all columns have same length
        let nrows = columns.values().next().unwrap().len();
        for (name, series) in &columns {
            if series.len() != nrows {
                return Err(DataFrameError::DimensionMismatch(format!(
                    "Column '{}' has length {} but expected {}",
                    name,
                    series.len(),
                    nrows
                )));
            }
        }

        Ok(Self { columns, nrows })
    }

    /// Create DataFrame from a HashMap of column names to data vectors
    pub fn from_map(data: HashMap<String, Vec<XdlValue>>) -> DataFrameResult<Self> {
        let mut columns = IndexMap::new();

        for (name, values) in data {
            columns.insert(name, Series::from_vec(values)?);
        }

        Self::from_columns(columns)
    }

    /// Get number of rows
    pub fn nrows(&self) -> usize {
        self.nrows
    }

    /// Get number of columns
    pub fn ncols(&self) -> usize {
        self.columns.len()
    }

    /// Get column names
    pub fn column_names(&self) -> Vec<String> {
        self.columns.keys().cloned().collect()
    }

    /// Get a column by name
    pub fn column(&self, name: &str) -> DataFrameResult<&Series> {
        self.columns
            .get(name)
            .ok_or_else(|| DataFrameError::ColumnNotFound(name.to_string()))
    }

    /// Get a mutable column by name
    pub fn column_mut(&mut self, name: &str) -> DataFrameResult<&mut Series> {
        self.columns
            .get_mut(name)
            .ok_or_else(|| DataFrameError::ColumnNotFound(name.to_string()))
    }

    /// Add a new column
    pub fn add_column(&mut self, name: String, series: Series) -> DataFrameResult<()> {
        if !self.columns.is_empty() && series.len() != self.nrows {
            return Err(DataFrameError::DimensionMismatch(format!(
                "Series has length {} but DataFrame has {} rows",
                series.len(),
                self.nrows
            )));
        }

        if self.columns.is_empty() {
            self.nrows = series.len();
        }

        self.columns.insert(name, series);
        Ok(())
    }

    /// Remove a column
    pub fn remove_column(&mut self, name: &str) -> DataFrameResult<Series> {
        self.columns
            .shift_remove(name)
            .ok_or_else(|| DataFrameError::ColumnNotFound(name.to_string()))
    }

    /// Select specific columns
    pub fn select(&self, column_names: &[&str]) -> DataFrameResult<DataFrame> {
        let mut new_columns = IndexMap::new();

        for name in column_names {
            let series = self.column(name)?.clone();
            new_columns.insert(name.to_string(), series);
        }

        Self::from_columns(new_columns)
    }

    /// Filter rows based on a predicate function
    pub fn filter<F>(&self, predicate: F) -> DataFrameResult<DataFrame>
    where
        F: Fn(usize, &HashMap<String, &XdlValue>) -> bool,
    {
        let mut selected_rows = Vec::new();

        // Find which rows satisfy the predicate
        for row_idx in 0..self.nrows {
            let mut row_map = HashMap::new();
            for (col_name, series) in &self.columns {
                if let Ok(value) = series.get(row_idx) {
                    row_map.insert(col_name.clone(), value);
                }
            }

            if predicate(row_idx, &row_map) {
                selected_rows.push(row_idx);
            }
        }

        // Create new DataFrame with selected rows
        let mut new_columns = IndexMap::new();
        for (col_name, series) in &self.columns {
            let filtered_values: Vec<XdlValue> = selected_rows
                .iter()
                .filter_map(|&idx| series.get(idx).ok().cloned())
                .collect();
            new_columns.insert(col_name.clone(), Series::from_vec(filtered_values)?);
        }

        Self::from_columns(new_columns)
    }

    /// Get a row as a HashMap
    pub fn row(&self, index: usize) -> DataFrameResult<HashMap<String, XdlValue>> {
        if index >= self.nrows {
            return Err(DataFrameError::IndexOutOfBounds(index, self.nrows));
        }

        let mut row = HashMap::new();
        for (col_name, series) in &self.columns {
            row.insert(col_name.clone(), series.get(index)?.clone());
        }

        Ok(row)
    }

    /// Get shape as (nrows, ncols)
    pub fn shape(&self) -> (usize, usize) {
        (self.nrows, self.ncols())
    }

    /// Get DataFrame info summary
    pub fn info(&self) -> String {
        let mut info = String::new();
        info.push_str(&format!(
            "DataFrame: {} rows × {} columns\n",
            self.nrows,
            self.ncols()
        ));
        info.push_str("\nColumns:\n");
        for (name, series) in &self.columns {
            info.push_str(&format!("  {} ({})\n", name, series.dtype()));
        }
        info
    }

    /// Head - get first n rows
    pub fn head(&self, n: usize) -> DataFrameResult<DataFrame> {
        let n = n.min(self.nrows);
        let mut new_columns = IndexMap::new();

        for (col_name, series) in &self.columns {
            new_columns.insert(col_name.clone(), series.head(n)?);
        }

        Self::from_columns(new_columns)
    }

    /// Tail - get last n rows
    pub fn tail(&self, n: usize) -> DataFrameResult<DataFrame> {
        let n = n.min(self.nrows);
        let mut new_columns = IndexMap::new();

        for (col_name, series) in &self.columns {
            new_columns.insert(col_name.clone(), series.tail(n)?);
        }

        Self::from_columns(new_columns)
    }

    /// Describe - get statistical summary
    pub fn describe(&self) -> DataFrameResult<HashMap<String, HashMap<String, f64>>> {
        let mut stats = HashMap::new();

        for (col_name, series) in &self.columns {
            if let Ok(col_stats) = series.describe() {
                stats.insert(col_name.clone(), col_stats);
            }
        }

        Ok(stats)
    }

    /// Convert to JSON representation
    pub fn to_json(&self) -> Vec<JsonValue> {
        let mut rows = Vec::new();

        for row_idx in 0..self.nrows {
            let mut row_obj = serde_json::Map::new();
            for (col_name, series) in &self.columns {
                if let Ok(value) = series.get(row_idx) {
                    row_obj.insert(col_name.clone(), xdl_value_to_json(value));
                }
            }
            rows.push(JsonValue::Object(row_obj));
        }

        rows
    }

    /// Convert to XdlValue (nested array)
    pub fn to_xdl_value(&self) -> XdlResult<XdlValue> {
        let mut rows = Vec::new();

        for row_idx in 0..self.nrows {
            let mut row_values = Vec::new();
            for series in self.columns.values() {
                if let Ok(value) = series.get(row_idx) {
                    row_values.push(value.clone());
                }
            }
            rows.push(XdlValue::NestedArray(row_values));
        }

        Ok(XdlValue::NestedArray(rows))
    }

    /// Sort by column(s)
    pub fn sort_by(&self, column_names: &[&str], ascending: bool) -> DataFrameResult<DataFrame> {
        if column_names.is_empty() {
            return Ok(self.clone());
        }

        // Create index vector
        let mut indices: Vec<usize> = (0..self.nrows).collect();

        // Sort indices based on column values
        indices.sort_by(|&a, &b| {
            for &col_name in column_names {
                if let Ok(series) = self.column(col_name) {
                    if let (Ok(val_a), Ok(val_b)) = (series.get(a), series.get(b)) {
                        let cmp = compare_xdl_values(val_a, val_b);
                        if cmp != std::cmp::Ordering::Equal {
                            return if ascending { cmp } else { cmp.reverse() };
                        }
                    }
                }
            }
            std::cmp::Ordering::Equal
        });

        // Create new DataFrame with sorted rows
        let mut new_columns = IndexMap::new();
        for (col_name, series) in &self.columns {
            let sorted_values: Vec<XdlValue> = indices
                .iter()
                .filter_map(|&idx| series.get(idx).ok().cloned())
                .collect();
            new_columns.insert(col_name.clone(), Series::from_vec(sorted_values)?);
        }

        Self::from_columns(new_columns)
    }

    /// Group by column(s) - returns grouped data for aggregation
    pub fn groupby(&self, column_names: &[&str]) -> DataFrameResult<GroupBy> {
        GroupBy::new(
            self.clone(),
            column_names.iter().map(|s| s.to_string()).collect(),
        )
    }
}

impl Default for DataFrame {
    fn default() -> Self {
        Self::new()
    }
}

/// GroupBy structure for aggregations
#[derive(Debug, Clone)]
pub struct GroupBy {
    dataframe: DataFrame,
    group_columns: Vec<String>,
    groups: HashMap<Vec<String>, Vec<usize>>, // group keys -> row indices
}

impl GroupBy {
    fn new(dataframe: DataFrame, group_columns: Vec<String>) -> DataFrameResult<Self> {
        let mut groups: HashMap<Vec<String>, Vec<usize>> = HashMap::new();

        // Build groups
        for row_idx in 0..dataframe.nrows() {
            let mut key = Vec::new();
            for col_name in &group_columns {
                if let Ok(value) = dataframe.column(col_name)?.get(row_idx) {
                    key.push(value.to_string_repr());
                }
            }

            groups.entry(key).or_default().push(row_idx);
        }

        Ok(Self {
            dataframe,
            group_columns,
            groups,
        })
    }

    /// Count rows in each group
    pub fn count(&self) -> DataFrameResult<DataFrame> {
        let mut columns = IndexMap::new();

        // Add group key columns
        let mut group_keys: Vec<_> = self.groups.keys().collect();
        group_keys.sort();

        for (i, col_name) in self.group_columns.iter().enumerate() {
            let values: Vec<XdlValue> = group_keys
                .iter()
                .map(|key| XdlValue::String(key[i].clone()))
                .collect();
            columns.insert(col_name.clone(), Series::from_vec(values)?);
        }

        // Add count column
        let counts: Vec<XdlValue> = group_keys
            .iter()
            .map(|key| XdlValue::Long(self.groups[*key].len() as i32))
            .collect();
        columns.insert("count".to_string(), Series::from_vec(counts)?);

        DataFrame::from_columns(columns)
    }

    /// Compute mean for numeric columns in each group
    pub fn mean(&self) -> DataFrameResult<DataFrame> {
        self.aggregate("mean", |values| {
            let nums: Vec<f64> = values.iter().filter_map(|v| v.to_double().ok()).collect();
            if nums.is_empty() {
                XdlValue::Undefined
            } else {
                XdlValue::Double(nums.iter().sum::<f64>() / nums.len() as f64)
            }
        })
    }

    /// Compute sum for numeric columns in each group
    pub fn sum(&self) -> DataFrameResult<DataFrame> {
        self.aggregate("sum", |values| {
            let sum: f64 = values.iter().filter_map(|v| v.to_double().ok()).sum();
            XdlValue::Double(sum)
        })
    }

    /// Generic aggregation function
    fn aggregate<F>(&self, _agg_name: &str, agg_fn: F) -> DataFrameResult<DataFrame>
    where
        F: Fn(&[XdlValue]) -> XdlValue,
    {
        let mut columns = IndexMap::new();
        let mut group_keys: Vec<_> = self.groups.keys().collect();
        group_keys.sort();

        // Add group key columns
        for (i, col_name) in self.group_columns.iter().enumerate() {
            let values: Vec<XdlValue> = group_keys
                .iter()
                .map(|key| XdlValue::String(key[i].clone()))
                .collect();
            columns.insert(col_name.clone(), Series::from_vec(values)?);
        }

        // Aggregate value columns
        for (col_name, _series) in &self.dataframe.columns {
            if self.group_columns.contains(col_name) {
                continue;
            }

            let values: Vec<XdlValue> = group_keys
                .iter()
                .map(|key| {
                    let indices = &self.groups[*key];
                    let col_values: Vec<XdlValue> = indices
                        .iter()
                        .filter_map(|&idx| {
                            self.dataframe.column(col_name).ok()?.get(idx).ok().cloned()
                        })
                        .collect();
                    agg_fn(&col_values)
                })
                .collect();

            columns.insert(col_name.clone(), Series::from_vec(values)?);
        }

        DataFrame::from_columns(columns)
    }
}

/// Helper function to convert XdlValue to JsonValue
fn xdl_value_to_json(value: &XdlValue) -> JsonValue {
    match value {
        XdlValue::Undefined => JsonValue::Null,
        XdlValue::Int(i) => JsonValue::from(*i),
        XdlValue::Long(l) => JsonValue::from(*l),
        XdlValue::Long64(l) => JsonValue::from(*l),
        XdlValue::Float(f) => JsonValue::from(*f),
        XdlValue::Double(d) => JsonValue::from(*d),
        XdlValue::String(s) => JsonValue::from(s.clone()),
        XdlValue::NestedArray(arr) => JsonValue::Array(arr.iter().map(xdl_value_to_json).collect()),
        _ => JsonValue::String(value.to_string_repr()),
    }
}

/// Helper function to compare XdlValues for sorting
fn compare_xdl_values(a: &XdlValue, b: &XdlValue) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    match (a, b) {
        (XdlValue::Int(a), XdlValue::Int(b)) => a.cmp(b),
        (XdlValue::Long(a), XdlValue::Long(b)) => a.cmp(b),
        (XdlValue::Long64(a), XdlValue::Long64(b)) => a.cmp(b),
        (XdlValue::Float(a), XdlValue::Float(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
        (XdlValue::Double(a), XdlValue::Double(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
        (XdlValue::String(a), XdlValue::String(b)) => a.cmp(b),
        _ => {
            // Try to compare as doubles
            if let (Ok(a_f), Ok(b_f)) = (a.to_double(), b.to_double()) {
                a_f.partial_cmp(&b_f).unwrap_or(Ordering::Equal)
            } else {
                a.to_string_repr().cmp(&b.to_string_repr())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_dataframe() {
        let df = DataFrame::new();
        assert_eq!(df.nrows(), 0);
        assert_eq!(df.ncols(), 0);
    }

    #[test]
    fn test_from_map() {
        let mut data = HashMap::new();
        data.insert(
            "col1".to_string(),
            vec![XdlValue::Long(1), XdlValue::Long(2), XdlValue::Long(3)],
        );
        data.insert(
            "col2".to_string(),
            vec![
                XdlValue::String("a".to_string()),
                XdlValue::String("b".to_string()),
                XdlValue::String("c".to_string()),
            ],
        );

        let df = DataFrame::from_map(data).unwrap();
        assert_eq!(df.nrows(), 3);
        assert_eq!(df.ncols(), 2);
    }

    #[test]
    fn test_select() {
        let mut data = HashMap::new();
        data.insert("col1".to_string(), vec![XdlValue::Long(1)]);
        data.insert("col2".to_string(), vec![XdlValue::Long(2)]);
        data.insert("col3".to_string(), vec![XdlValue::Long(3)]);

        let df = DataFrame::from_map(data).unwrap();
        let selected = df.select(&["col1", "col3"]).unwrap();

        assert_eq!(selected.ncols(), 2);
        assert!(selected.column("col1").is_ok());
        assert!(selected.column("col3").is_ok());
        assert!(selected.column("col2").is_err());
    }
}
