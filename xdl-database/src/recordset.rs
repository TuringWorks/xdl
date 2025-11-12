//! Recordset - represents query results

use serde_json::Value as JsonValue;
use std::collections::HashMap;
use xdl_core::{XdlError, XdlResult, XdlValue};

/// Column information
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub ordinal: usize,
}

/// Recordset containing query results
#[derive(Debug, Clone)]
pub struct Recordset {
    columns: Vec<ColumnInfo>,
    rows: Vec<Vec<JsonValue>>,
    current_row: usize,
}

impl Recordset {
    /// Create a new recordset
    pub fn new(columns: Vec<ColumnInfo>, rows: Vec<Vec<JsonValue>>) -> Self {
        Self {
            columns,
            rows,
            current_row: 0,
        }
    }

    /// Create an empty recordset
    pub fn empty() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            current_row: 0,
        }
    }

    /// Get the number of rows
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Get the number of columns
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Get column names
    pub fn column_names(&self) -> Vec<String> {
        self.columns.iter().map(|c| c.name.clone()).collect()
    }

    /// Get column information
    pub fn columns(&self) -> &[ColumnInfo] {
        &self.columns
    }

    /// Move to the next row
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> bool {
        if self.current_row < self.rows.len() {
            self.current_row += 1;
            true
        } else {
            false
        }
    }

    /// Reset to the first row
    pub fn reset(&mut self) {
        self.current_row = 0;
    }

    /// Get current row data as a HashMap
    pub fn current_row(&self) -> Option<HashMap<String, JsonValue>> {
        if self.current_row < self.rows.len() {
            let row = &self.rows[self.current_row];
            let mut map = HashMap::new();

            for (i, value) in row.iter().enumerate() {
                if let Some(col) = self.columns.get(i) {
                    map.insert(col.name.clone(), value.clone());
                }
            }

            Some(map)
        } else {
            None
        }
    }

    /// Get all data as XdlValue (structure array)
    pub fn get_data(&self) -> XdlResult<XdlValue> {
        // Convert recordset to XDL-compatible structure
        // For now, we'll create a nested array where each row is an array of values

        if self.rows.is_empty() {
            return Ok(XdlValue::Undefined);
        }

        // Create array of row arrays
        let mut row_arrays = Vec::new();

        for row in &self.rows {
            let mut row_values = Vec::new();

            for cell in row {
                let xdl_val = json_to_xdl(cell)?;
                row_values.push(xdl_val);
            }

            row_arrays.push(XdlValue::NestedArray(row_values));
        }

        Ok(XdlValue::NestedArray(row_arrays))
    }

    /// Get data as a column-major structure (like IDL structures)
    pub fn get_data_structured(&self) -> XdlResult<HashMap<String, Vec<XdlValue>>> {
        let mut result = HashMap::new();

        // Initialize column vectors
        for col in &self.columns {
            result.insert(col.name.clone(), Vec::new());
        }

        // Fill column vectors
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if let Some(col) = self.columns.get(i) {
                    let xdl_val = json_to_xdl(cell)?;
                    if let Some(col_vec) = result.get_mut(&col.name) {
                        col_vec.push(xdl_val);
                    }
                }
            }
        }

        Ok(result)
    }

    /// Get a specific column as an array
    pub fn get_column(&self, column_name: &str) -> XdlResult<Vec<XdlValue>> {
        let col_index = self
            .columns
            .iter()
            .position(|c| c.name == column_name)
            .ok_or_else(|| XdlError::RuntimeError(format!("Column not found: {}", column_name)))?;

        let mut values = Vec::new();
        for row in &self.rows {
            if let Some(cell) = row.get(col_index) {
                values.push(json_to_xdl(cell)?);
            }
        }

        Ok(values)
    }

    /// Get row at specific index
    pub fn get_row(&self, index: usize) -> Option<&Vec<JsonValue>> {
        self.rows.get(index)
    }

    /// Check if recordset is empty
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// Convert JSON value to XdlValue
fn json_to_xdl(value: &JsonValue) -> XdlResult<XdlValue> {
    match value {
        JsonValue::Null => Ok(XdlValue::Undefined),
        JsonValue::Bool(b) => Ok(XdlValue::Long(if *b { 1 } else { 0 })),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                    Ok(XdlValue::Long(i as i32))
                } else {
                    Ok(XdlValue::Long64(i))
                }
            } else if let Some(f) = n.as_f64() {
                Ok(XdlValue::Double(f))
            } else {
                Ok(XdlValue::Undefined)
            }
        }
        JsonValue::String(s) => Ok(XdlValue::String(s.clone())),
        JsonValue::Array(arr) => {
            let values: Result<Vec<_>, _> = arr.iter().map(json_to_xdl).collect();
            Ok(XdlValue::NestedArray(values?))
        }
        JsonValue::Object(_) => {
            // For objects, convert to string representation
            Ok(XdlValue::String(value.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_recordset() {
        let rs = Recordset::empty();
        assert_eq!(rs.row_count(), 0);
        assert_eq!(rs.column_count(), 0);
        assert!(rs.is_empty());
    }

    #[test]
    fn test_recordset_with_data() {
        let columns = vec![
            ColumnInfo {
                name: "id".to_string(),
                data_type: "integer".to_string(),
                ordinal: 0,
            },
            ColumnInfo {
                name: "name".to_string(),
                data_type: "text".to_string(),
                ordinal: 1,
            },
        ];

        let rows = vec![
            vec![JsonValue::from(1), JsonValue::from("Alice")],
            vec![JsonValue::from(2), JsonValue::from("Bob")],
        ];

        let rs = Recordset::new(columns, rows);

        assert_eq!(rs.row_count(), 2);
        assert_eq!(rs.column_count(), 2);
        assert!(!rs.is_empty());
        assert_eq!(rs.column_names(), vec!["id", "name"]);
    }
}
