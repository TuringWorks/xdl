//! Avro file reader

#[cfg(feature = "avro-support")]
use crate::dataframe::DataFrame;
#[cfg(feature = "avro-support")]
use crate::error::{DataFrameError, DataFrameResult};
#[cfg(feature = "avro-support")]
use crate::series::Series;
#[cfg(feature = "avro-support")]
use apache_avro::types::Value as AvroValue;
#[cfg(feature = "avro-support")]
use apache_avro::Reader;
#[cfg(feature = "avro-support")]
use indexmap::IndexMap;
#[cfg(feature = "avro-support")]
use std::fs::File;
#[cfg(feature = "avro-support")]
use std::path::Path;
#[cfg(feature = "avro-support")]
use xdl_core::XdlValue;

#[cfg(feature = "avro-support")]
pub fn read_avro<P: AsRef<Path>>(path: P) -> DataFrameResult<DataFrame> {
    let file = File::open(path)
        .map_err(|e| DataFrameError::AvroError(format!("Failed to open file: {}", e)))?;

    let reader = Reader::new(file)
        .map_err(|e| DataFrameError::AvroError(format!("Failed to create Avro reader: {}", e)))?;

    let mut rows: Vec<IndexMap<String, XdlValue>> = Vec::new();

    // Read all records
    for record_result in reader {
        let record = record_result
            .map_err(|e| DataFrameError::AvroError(format!("Failed to read record: {}", e)))?;

        let row_data = avro_value_to_map(&record)?;
        rows.push(row_data);
    }

    if rows.is_empty() {
        return Ok(DataFrame::new());
    }

    // Extract column names from first row
    let column_names: Vec<String> = rows[0].keys().cloned().collect();

    // Build column data
    let mut columns = IndexMap::new();
    for col_name in &column_names {
        let col_values: Vec<XdlValue> = rows
            .iter()
            .map(|row| row.get(col_name).cloned().unwrap_or(XdlValue::Undefined))
            .collect();
        columns.insert(col_name.clone(), Series::from_vec(col_values)?);
    }

    DataFrame::from_columns(columns)
}

#[cfg(feature = "avro-support")]
fn avro_value_to_map(value: &AvroValue) -> DataFrameResult<IndexMap<String, XdlValue>> {
    match value {
        AvroValue::Record(fields) => {
            let mut map = IndexMap::new();
            for (name, field_value) in fields {
                map.insert(name.clone(), avro_value_to_xdl(field_value)?);
            }
            Ok(map)
        }
        _ => Err(DataFrameError::AvroError(
            "Expected Avro Record type".to_string(),
        )),
    }
}

#[cfg(feature = "avro-support")]
fn avro_value_to_xdl(value: &AvroValue) -> DataFrameResult<XdlValue> {
    match value {
        AvroValue::Null => Ok(XdlValue::Undefined),
        AvroValue::Boolean(b) => Ok(XdlValue::Long(if *b { 1 } else { 0 })),
        AvroValue::Int(i) => Ok(XdlValue::Long(*i)),
        AvroValue::Long(l) => Ok(XdlValue::Long64(*l)),
        AvroValue::Float(f) => Ok(XdlValue::Float(*f)),
        AvroValue::Double(d) => Ok(XdlValue::Double(*d)),
        AvroValue::String(s) => Ok(XdlValue::String(s.clone())),
        AvroValue::Bytes(b) => {
            // Convert bytes to base64 string
            Ok(XdlValue::String(format!("{:?}", b)))
        }
        AvroValue::Fixed(_size, bytes) => Ok(XdlValue::String(format!("{:?}", bytes))),
        AvroValue::Enum(_idx, symbol) => Ok(XdlValue::String(symbol.clone())),
        AvroValue::Union(_idx, boxed_value) => avro_value_to_xdl(boxed_value),
        AvroValue::Array(arr) => {
            let values: Result<Vec<XdlValue>, DataFrameError> =
                arr.iter().map(avro_value_to_xdl).collect();
            Ok(XdlValue::NestedArray(values?))
        }
        AvroValue::Map(map) => {
            // Convert map to string representation
            let map_str = format!("{:?}", map);
            Ok(XdlValue::String(map_str))
        }
        AvroValue::Record(fields) => {
            // Convert record to string representation
            let record_str = fields
                .iter()
                .map(|(k, v)| format!("{}: {:?}", k, v))
                .collect::<Vec<_>>()
                .join(", ");
            Ok(XdlValue::String(format!("{{{}}}", record_str)))
        }
        _ => Ok(XdlValue::String(format!("{:?}", value))),
    }
}

#[cfg(not(feature = "avro-support"))]
use crate::dataframe::DataFrame;
#[cfg(not(feature = "avro-support"))]
use crate::error::{DataFrameError, DataFrameResult};
#[cfg(not(feature = "avro-support"))]
use std::path::Path;

#[cfg(not(feature = "avro-support"))]
pub fn read_avro<P: AsRef<Path>>(_path: P) -> DataFrameResult<DataFrame> {
    Err(DataFrameError::InvalidOperation(
        "Avro support not enabled. Enable the 'avro-support' feature".to_string(),
    ))
}
