//! Database integration - convert between DataFrame and database Recordset

#[cfg(feature = "database-integration")]
use crate::dataframe::DataFrame;
#[cfg(feature = "database-integration")]
use crate::error::DataFrameResult;
#[cfg(feature = "database-integration")]
use crate::series::Series;
#[cfg(feature = "database-integration")]
use indexmap::IndexMap;
#[cfg(feature = "database-integration")]
use serde_json::Value as JsonValue;
#[cfg(feature = "database-integration")]
use xdl_core::XdlValue;
#[cfg(feature = "database-integration")]
use xdl_database::Recordset;

/// Convert a database Recordset to a DataFrame
#[cfg(feature = "database-integration")]
pub fn from_recordset(recordset: &Recordset) -> DataFrameResult<DataFrame> {
    let column_names = recordset.column_names();
    let mut columns = IndexMap::new();

    // Extract each column
    for col_name in &column_names {
        let col_data = recordset.get_column(col_name)?;
        columns.insert(col_name.clone(), Series::from_vec(col_data)?);
    }

    DataFrame::from_columns(columns)
}

/// Convert DataFrame to JSON values (compatible with Recordset format)
#[cfg(feature = "database-integration")]
pub fn to_json_rows(dataframe: &DataFrame) -> Vec<Vec<JsonValue>> {
    let mut rows = Vec::new();

    for row_idx in 0..dataframe.nrows() {
        let mut row = Vec::new();
        for col_name in dataframe.column_names() {
            if let Ok(series) = dataframe.column(&col_name) {
                if let Ok(value) = series.get(row_idx) {
                    row.push(xdl_value_to_json(value));
                }
            }
        }
        rows.push(row);
    }

    rows
}

#[cfg(feature = "database-integration")]
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

#[cfg(not(feature = "database-integration"))]
use crate::dataframe::DataFrame;
#[cfg(not(feature = "database-integration"))]
use crate::error::{DataFrameError, DataFrameResult};

#[cfg(not(feature = "database-integration"))]
pub fn from_recordset(_recordset: &()) -> DataFrameResult<DataFrame> {
    Err(DataFrameError::InvalidOperation(
        "Database integration not enabled. Enable the 'database-integration' feature".to_string(),
    ))
}
