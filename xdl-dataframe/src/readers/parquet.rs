//! Parquet file reader

#[cfg(feature = "parquet-support")]
use crate::dataframe::DataFrame;
#[cfg(feature = "parquet-support")]
use crate::error::{DataFrameError, DataFrameResult};
#[cfg(feature = "parquet-support")]
use crate::series::Series;
#[cfg(feature = "parquet-support")]
use arrow::array::*;
#[cfg(feature = "parquet-support")]
use arrow::datatypes::DataType;
#[cfg(feature = "parquet-support")]
use indexmap::IndexMap;
#[cfg(feature = "parquet-support")]
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
#[cfg(feature = "parquet-support")]
use std::fs::File;
#[cfg(feature = "parquet-support")]
use std::path::Path;
#[cfg(feature = "parquet-support")]
use std::sync::Arc;
#[cfg(feature = "parquet-support")]
use xdl_core::XdlValue;

#[cfg(feature = "parquet-support")]
pub fn read_parquet<P: AsRef<Path>>(path: P) -> DataFrameResult<DataFrame> {
    let file = File::open(path)
        .map_err(|e| DataFrameError::ParquetError(format!("Failed to open file: {}", e)))?;

    let builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .map_err(|e| DataFrameError::ParquetError(format!("Failed to create reader: {}", e)))?;

    let reader = builder
        .build()
        .map_err(|e| DataFrameError::ParquetError(format!("Failed to build reader: {}", e)))?;

    let schema = reader.schema();
    let mut columns: IndexMap<String, Vec<XdlValue>> = IndexMap::new();

    // Initialize column vectors
    for field in schema.fields() {
        columns.insert(field.name().clone(), Vec::new());
    }

    // Read all batches
    for batch_result in reader {
        let batch = batch_result
            .map_err(|e| DataFrameError::ParquetError(format!("Failed to read batch: {}", e)))?;

        for (col_idx, field) in schema.fields().iter().enumerate() {
            let col_name = field.name();
            let array = batch.column(col_idx);
            let values = arrow_array_to_xdl_values(array, field.data_type())?;

            if let Some(col_vec) = columns.get_mut(col_name) {
                col_vec.extend(values);
            }
        }
    }

    // Convert to DataFrame
    let mut df_columns = IndexMap::new();
    for (name, values) in columns {
        df_columns.insert(name, Series::from_vec(values)?);
    }

    DataFrame::from_columns(df_columns)
}

#[cfg(feature = "parquet-support")]
fn arrow_array_to_xdl_values(
    array: &Arc<dyn Array>,
    data_type: &DataType,
) -> DataFrameResult<Vec<XdlValue>> {
    let mut values = Vec::new();

    match data_type {
        DataType::Boolean => {
            let arr = array
                .as_any()
                .downcast_ref::<BooleanArray>()
                .ok_or_else(|| {
                    DataFrameError::ParquetError("Failed to downcast to BooleanArray".to_string())
                })?;
            for i in 0..arr.len() {
                values.push(if arr.is_null(i) {
                    XdlValue::Undefined
                } else {
                    XdlValue::Long(if arr.value(i) { 1 } else { 0 })
                });
            }
        }
        DataType::Int8 => {
            let arr = array.as_any().downcast_ref::<Int8Array>().ok_or_else(|| {
                DataFrameError::ParquetError("Failed to downcast to Int8Array".to_string())
            })?;
            for i in 0..arr.len() {
                values.push(if arr.is_null(i) {
                    XdlValue::Undefined
                } else {
                    XdlValue::Long(arr.value(i) as i32)
                });
            }
        }
        DataType::Int16 => {
            let arr = array.as_any().downcast_ref::<Int16Array>().ok_or_else(|| {
                DataFrameError::ParquetError("Failed to downcast to Int16Array".to_string())
            })?;
            for i in 0..arr.len() {
                values.push(if arr.is_null(i) {
                    XdlValue::Undefined
                } else {
                    XdlValue::Int(arr.value(i))
                });
            }
        }
        DataType::Int32 => {
            let arr = array.as_any().downcast_ref::<Int32Array>().ok_or_else(|| {
                DataFrameError::ParquetError("Failed to downcast to Int32Array".to_string())
            })?;
            for i in 0..arr.len() {
                values.push(if arr.is_null(i) {
                    XdlValue::Undefined
                } else {
                    XdlValue::Long(arr.value(i))
                });
            }
        }
        DataType::Int64 => {
            let arr = array.as_any().downcast_ref::<Int64Array>().ok_or_else(|| {
                DataFrameError::ParquetError("Failed to downcast to Int64Array".to_string())
            })?;
            for i in 0..arr.len() {
                values.push(if arr.is_null(i) {
                    XdlValue::Undefined
                } else {
                    XdlValue::Long64(arr.value(i))
                });
            }
        }
        DataType::UInt8 | DataType::UInt16 | DataType::UInt32 => {
            // Convert unsigned to signed for XDL compatibility
            for i in 0..array.len() {
                if array.is_null(i) {
                    values.push(XdlValue::Undefined);
                } else {
                    // This is a simplified conversion
                    values.push(XdlValue::Long(i as i32));
                }
            }
        }
        DataType::UInt64 => {
            let arr = array
                .as_any()
                .downcast_ref::<UInt64Array>()
                .ok_or_else(|| {
                    DataFrameError::ParquetError("Failed to downcast to UInt64Array".to_string())
                })?;
            for i in 0..arr.len() {
                values.push(if arr.is_null(i) {
                    XdlValue::Undefined
                } else {
                    XdlValue::Long64(arr.value(i) as i64)
                });
            }
        }
        DataType::Float32 => {
            let arr = array
                .as_any()
                .downcast_ref::<Float32Array>()
                .ok_or_else(|| {
                    DataFrameError::ParquetError("Failed to downcast to Float32Array".to_string())
                })?;
            for i in 0..arr.len() {
                values.push(if arr.is_null(i) {
                    XdlValue::Undefined
                } else {
                    XdlValue::Float(arr.value(i))
                });
            }
        }
        DataType::Float64 => {
            let arr = array
                .as_any()
                .downcast_ref::<Float64Array>()
                .ok_or_else(|| {
                    DataFrameError::ParquetError("Failed to downcast to Float64Array".to_string())
                })?;
            for i in 0..arr.len() {
                values.push(if arr.is_null(i) {
                    XdlValue::Undefined
                } else {
                    XdlValue::Double(arr.value(i))
                });
            }
        }
        DataType::Utf8 => {
            let arr = array
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| {
                    DataFrameError::ParquetError("Failed to downcast to StringArray".to_string())
                })?;
            for i in 0..arr.len() {
                values.push(if arr.is_null(i) {
                    XdlValue::Undefined
                } else {
                    XdlValue::String(arr.value(i).to_string())
                });
            }
        }
        DataType::LargeUtf8 => {
            let arr = array
                .as_any()
                .downcast_ref::<LargeStringArray>()
                .ok_or_else(|| {
                    DataFrameError::ParquetError(
                        "Failed to downcast to LargeStringArray".to_string(),
                    )
                })?;
            for i in 0..arr.len() {
                values.push(if arr.is_null(i) {
                    XdlValue::Undefined
                } else {
                    XdlValue::String(arr.value(i).to_string())
                });
            }
        }
        _ => {
            // For unsupported types, convert to string
            for i in 0..array.len() {
                values.push(if array.is_null(i) {
                    XdlValue::Undefined
                } else {
                    XdlValue::String(format!("{:?}", array))
                });
            }
        }
    }

    Ok(values)
}

#[cfg(not(feature = "parquet-support"))]
use crate::dataframe::DataFrame;
#[cfg(not(feature = "parquet-support"))]
use crate::error::{DataFrameError, DataFrameResult};
#[cfg(not(feature = "parquet-support"))]
use std::path::Path;

#[cfg(not(feature = "parquet-support"))]
pub fn read_parquet<P: AsRef<Path>>(_path: P) -> DataFrameResult<DataFrame> {
    Err(DataFrameError::InvalidOperation(
        "Parquet support not enabled. Enable the 'parquet-support' feature".to_string(),
    ))
}
