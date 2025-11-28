//! Polars DataFrame integration for XDL
//!
//! This module provides high-performance DataFrame operations using Polars,
//! which is significantly faster than Pandas for most operations.

use polars::prelude::*;
use std::collections::HashMap;
use std::sync::Mutex;
use xdl_core::{XdlError, XdlResult, XdlValue};

lazy_static::lazy_static! {
    static ref DATAFRAMES: Mutex<HashMap<String, DataFrame>> = Mutex::new(HashMap::new());
    static ref DF_COUNTER: Mutex<usize> = Mutex::new(0);
}

fn next_df_id() -> String {
    let mut counter = DF_COUNTER.lock().unwrap();
    *counter += 1;
    format!("df_{}", *counter)
}

fn store_dataframe(df: DataFrame) -> String {
    let id = next_df_id();
    let mut storage = DATAFRAMES.lock().unwrap();
    storage.insert(id.clone(), df);
    id
}

fn get_dataframe(id: &str) -> XdlResult<DataFrame> {
    let storage = DATAFRAMES.lock().unwrap();
    storage
        .get(id)
        .cloned()
        .ok_or_else(|| XdlError::RuntimeError(format!("DataFrame not found: {}", id)))
}

/// DF_READ_CSV - Read a CSV file into a DataFrame
pub fn df_read_csv(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "DF_READ_CSV requires at least 1 argument: filename".to_string(),
        ));
    }

    let filename = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::RuntimeError(
                "Filename must be a string".to_string(),
            ))
        }
    };

    let has_header = args
        .get(1)
        .map(|v| match v {
            XdlValue::Byte(b) => *b != 0,
            XdlValue::Long(n) => *n != 0,
            _ => true,
        })
        .unwrap_or(true);

    let delimiter = args
        .get(2)
        .and_then(|v| {
            if let XdlValue::String(s) = v {
                s.chars().next()
            } else {
                None
            }
        })
        .unwrap_or(',');

    let df = CsvReadOptions::default()
        .with_has_header(has_header)
        .map_parse_options(|opts| opts.with_separator(delimiter as u8))
        .try_into_reader_with_file_path(Some(filename.into()))
        .map_err(|e| XdlError::RuntimeError(format!("Failed to create CSV reader: {}", e)))?
        .finish()
        .map_err(|e| XdlError::RuntimeError(format!("Failed to read CSV: {}", e)))?;

    let id = store_dataframe(df);
    Ok(XdlValue::String(id))
}

/// DF_READ_PARQUET - Read a Parquet file
pub fn df_read_parquet(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "DF_READ_PARQUET requires filename".to_string(),
        ));
    }

    let filename = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::RuntimeError(
                "Filename must be a string".to_string(),
            ))
        }
    };

    let file = std::fs::File::open(&filename)
        .map_err(|e| XdlError::RuntimeError(format!("Failed to open file: {}", e)))?;

    let df = ParquetReader::new(file)
        .finish()
        .map_err(|e| XdlError::RuntimeError(format!("Failed to read Parquet: {}", e)))?;

    let id = store_dataframe(df);
    Ok(XdlValue::String(id))
}

/// DF_READ_JSON - Read a JSON file
pub fn df_read_json(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "DF_READ_JSON requires filename".to_string(),
        ));
    }

    let filename = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::RuntimeError(
                "Filename must be a string".to_string(),
            ))
        }
    };

    let file = std::fs::File::open(&filename)
        .map_err(|e| XdlError::RuntimeError(format!("Failed to open file: {}", e)))?;

    let df = JsonReader::new(file)
        .finish()
        .map_err(|e| XdlError::RuntimeError(format!("Failed to read JSON: {}", e)))?;

    let id = store_dataframe(df);
    Ok(XdlValue::String(id))
}

/// DF_CREATE - Create DataFrame from arrays
pub fn df_create(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::RuntimeError(
            "DF_CREATE requires column_names and data arrays".to_string(),
        ));
    }

    let col_names: Vec<String> = match &args[0] {
        XdlValue::NestedArray(arr) => arr
            .iter()
            .filter_map(|v| {
                if let XdlValue::String(s) = v {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .collect(),
        _ => {
            return Err(XdlError::RuntimeError(
                "Column names must be an array".to_string(),
            ))
        }
    };

    if col_names.len() != args.len() - 1 {
        return Err(XdlError::RuntimeError(
            "Number of column names must match data arrays".to_string(),
        ));
    }

    let mut columns: Vec<Column> = Vec::new();

    for (i, name) in col_names.iter().enumerate() {
        let data = &args[i + 1];
        let series = match data {
            XdlValue::Array(arr) => Series::new(name.into(), arr.as_slice()),
            XdlValue::MultiDimArray { data, .. } => Series::new(name.into(), data.as_slice()),
            _ => {
                return Err(XdlError::RuntimeError(
                    "Column data must be an array".to_string(),
                ))
            }
        };
        columns.push(series.into());
    }

    let df = DataFrame::new(columns)
        .map_err(|e| XdlError::RuntimeError(format!("Failed to create DataFrame: {}", e)))?;

    let id = store_dataframe(df);
    Ok(XdlValue::String(id))
}

/// DF_WRITE_CSV - Write DataFrame to CSV
pub fn df_write_csv(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::RuntimeError(
            "DF_WRITE_CSV requires df_id, filename".to_string(),
        ));
    }

    let df_id = match &args[0] {
        XdlValue::String(s) => s,
        _ => return Err(XdlError::RuntimeError("df_id must be a string".to_string())),
    };

    let filename = match &args[1] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::RuntimeError(
                "Filename must be a string".to_string(),
            ))
        }
    };

    let mut df = get_dataframe(df_id)?;
    let mut file = std::fs::File::create(filename)
        .map_err(|e| XdlError::RuntimeError(format!("Failed to create file: {}", e)))?;

    CsvWriter::new(&mut file)
        .finish(&mut df)
        .map_err(|e| XdlError::RuntimeError(format!("Failed to write CSV: {}", e)))?;

    Ok(XdlValue::Undefined)
}

/// DF_WRITE_PARQUET - Write DataFrame to Parquet
pub fn df_write_parquet(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::RuntimeError(
            "DF_WRITE_PARQUET requires df_id, filename".to_string(),
        ));
    }

    let df_id = match &args[0] {
        XdlValue::String(s) => s,
        _ => return Err(XdlError::RuntimeError("df_id must be a string".to_string())),
    };

    let filename = match &args[1] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::RuntimeError(
                "Filename must be a string".to_string(),
            ))
        }
    };

    let mut df = get_dataframe(df_id)?;
    let file = std::fs::File::create(filename)
        .map_err(|e| XdlError::RuntimeError(format!("Failed to create file: {}", e)))?;

    ParquetWriter::new(file)
        .finish(&mut df)
        .map_err(|e| XdlError::RuntimeError(format!("Failed to write Parquet: {}", e)))?;

    Ok(XdlValue::Undefined)
}

/// DF_HEAD - Get first N rows
pub fn df_head(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError("DF_HEAD requires df_id".to_string()));
    }

    let df_id = match &args[0] {
        XdlValue::String(s) => s,
        _ => return Err(XdlError::RuntimeError("df_id must be a string".to_string())),
    };

    let n = args
        .get(1)
        .map(|v| match v {
            XdlValue::Long(n) => *n as usize,
            _ => 5,
        })
        .unwrap_or(5);

    let df = get_dataframe(df_id)?;
    let result = df.head(Some(n));
    let id = store_dataframe(result);
    Ok(XdlValue::String(id))
}

/// DF_TAIL - Get last N rows
pub fn df_tail(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError("DF_TAIL requires df_id".to_string()));
    }

    let df_id = match &args[0] {
        XdlValue::String(s) => s,
        _ => return Err(XdlError::RuntimeError("df_id must be a string".to_string())),
    };

    let n = args
        .get(1)
        .map(|v| match v {
            XdlValue::Long(n) => *n as usize,
            _ => 5,
        })
        .unwrap_or(5);

    let df = get_dataframe(df_id)?;
    let result = df.tail(Some(n));
    let id = store_dataframe(result);
    Ok(XdlValue::String(id))
}

/// DF_SELECT - Select columns
pub fn df_select(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::RuntimeError(
            "DF_SELECT requires df_id and columns".to_string(),
        ));
    }

    let df_id = match &args[0] {
        XdlValue::String(s) => s,
        _ => return Err(XdlError::RuntimeError("df_id must be a string".to_string())),
    };

    let columns: Vec<String> = args[1..]
        .iter()
        .filter_map(|v| {
            if let XdlValue::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        })
        .collect();

    let df = get_dataframe(df_id)?;

    // Use lazy select with column expressions
    let col_exprs: Vec<Expr> = columns.iter().map(col).collect();
    let result = df
        .lazy()
        .select(col_exprs)
        .collect()
        .map_err(|e| XdlError::RuntimeError(format!("Failed to select: {}", e)))?;

    let id = store_dataframe(result);
    Ok(XdlValue::String(id))
}

/// DF_FILTER - Filter rows using lazy expressions
pub fn df_filter(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 4 {
        return Err(XdlError::RuntimeError(
            "DF_FILTER requires df_id, column, operator, value".to_string(),
        ));
    }

    let df_id = match &args[0] {
        XdlValue::String(s) => s,
        _ => return Err(XdlError::RuntimeError("df_id must be a string".to_string())),
    };

    let column = match &args[1] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::RuntimeError(
                "Column must be a string".to_string(),
            ))
        }
    };

    let operator = match &args[2] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::RuntimeError(
                "Operator must be a string".to_string(),
            ))
        }
    };

    let df = get_dataframe(df_id)?;

    // Use lazy expressions for filtering
    let filter_expr = match &args[3] {
        XdlValue::Long(n) => {
            let val = *n as i64;
            match operator.as_str() {
                "=" | "==" => col(&column).eq(lit(val)),
                "!=" | "<>" => col(&column).neq(lit(val)),
                ">" => col(&column).gt(lit(val)),
                "<" => col(&column).lt(lit(val)),
                ">=" => col(&column).gt_eq(lit(val)),
                "<=" => col(&column).lt_eq(lit(val)),
                _ => {
                    return Err(XdlError::RuntimeError(format!(
                        "Unknown operator: {}",
                        operator
                    )))
                }
            }
        }
        XdlValue::Double(d) => {
            let val = *d;
            match operator.as_str() {
                "=" | "==" => col(&column).eq(lit(val)),
                "!=" | "<>" => col(&column).neq(lit(val)),
                ">" => col(&column).gt(lit(val)),
                "<" => col(&column).lt(lit(val)),
                ">=" => col(&column).gt_eq(lit(val)),
                "<=" => col(&column).lt_eq(lit(val)),
                _ => {
                    return Err(XdlError::RuntimeError(format!(
                        "Unknown operator: {}",
                        operator
                    )))
                }
            }
        }
        XdlValue::String(s) => match operator.as_str() {
            "=" | "==" => col(&column).eq(lit(s.clone())),
            "!=" | "<>" => col(&column).neq(lit(s.clone())),
            _ => {
                return Err(XdlError::RuntimeError(
                    "String only supports = and !=".to_string(),
                ))
            }
        },
        _ => {
            return Err(XdlError::RuntimeError(
                "Unsupported filter value".to_string(),
            ))
        }
    };

    let result = df
        .lazy()
        .filter(filter_expr)
        .collect()
        .map_err(|e| XdlError::RuntimeError(format!("Failed to filter: {}", e)))?;

    let id = store_dataframe(result);
    Ok(XdlValue::String(id))
}

/// DF_SORT - Sort by column
pub fn df_sort(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::RuntimeError(
            "DF_SORT requires df_id, column".to_string(),
        ));
    }

    let df_id = match &args[0] {
        XdlValue::String(s) => s,
        _ => return Err(XdlError::RuntimeError("df_id must be a string".to_string())),
    };

    let column = match &args[1] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::RuntimeError(
                "Column must be a string".to_string(),
            ))
        }
    };

    let descending = args
        .get(2)
        .map(|v| match v {
            XdlValue::Byte(b) => *b != 0,
            XdlValue::Long(n) => *n != 0,
            _ => false,
        })
        .unwrap_or(false);

    let df = get_dataframe(df_id)?;
    let result = df
        .sort(
            [&column],
            SortMultipleOptions::default().with_order_descending(descending),
        )
        .map_err(|e| XdlError::RuntimeError(format!("Failed to sort: {}", e)))?;

    let id = store_dataframe(result);
    Ok(XdlValue::String(id))
}

/// DF_GROUPBY - Group and aggregate
pub fn df_groupby(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 4 {
        return Err(XdlError::RuntimeError(
            "DF_GROUPBY requires df_id, group_col, agg_col, agg_func".to_string(),
        ));
    }

    let df_id = match &args[0] {
        XdlValue::String(s) => s,
        _ => return Err(XdlError::RuntimeError("df_id must be a string".to_string())),
    };

    let group_col = match &args[1] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::RuntimeError(
                "Group column must be a string".to_string(),
            ))
        }
    };

    let agg_col = match &args[2] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::RuntimeError(
                "Aggregate column must be a string".to_string(),
            ))
        }
    };

    let agg_func = match &args[3] {
        XdlValue::String(s) => s.to_lowercase(),
        _ => {
            return Err(XdlError::RuntimeError(
                "Aggregate function must be a string".to_string(),
            ))
        }
    };

    let df = get_dataframe(df_id)?;

    let agg_expr = match agg_func.as_str() {
        "sum" => col(&agg_col).sum(),
        "mean" | "avg" => col(&agg_col).mean(),
        "min" => col(&agg_col).min(),
        "max" => col(&agg_col).max(),
        "count" => col(&agg_col).count(),
        "first" => col(&agg_col).first(),
        "last" => col(&agg_col).last(),
        "std" => col(&agg_col).std(1),
        "var" => col(&agg_col).var(1),
        _ => {
            return Err(XdlError::RuntimeError(format!(
                "Unknown aggregation: {}",
                agg_func
            )))
        }
    };

    let result = df
        .lazy()
        .group_by([col(&group_col)])
        .agg([agg_expr])
        .collect()
        .map_err(|e| XdlError::RuntimeError(format!("Failed to group by: {}", e)))?;

    let id = store_dataframe(result);
    Ok(XdlValue::String(id))
}

/// DF_JOIN - Join two DataFrames
pub fn df_join(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::RuntimeError(
            "DF_JOIN requires df1_id, df2_id, on_column".to_string(),
        ));
    }

    let df1_id = match &args[0] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::RuntimeError(
                "df1_id must be a string".to_string(),
            ))
        }
    };

    let df2_id = match &args[1] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::RuntimeError(
                "df2_id must be a string".to_string(),
            ))
        }
    };

    let on_col = match &args[2] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::RuntimeError(
                "Join column must be a string".to_string(),
            ))
        }
    };

    let how = args
        .get(3)
        .and_then(|v| {
            if let XdlValue::String(s) = v {
                Some(s.to_lowercase())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "inner".to_string());

    let df1 = get_dataframe(df1_id)?;
    let df2 = get_dataframe(df2_id)?;

    let join_type = match how.as_str() {
        "inner" => JoinType::Inner,
        "left" => JoinType::Left,
        "right" => JoinType::Right,
        "outer" | "full" => JoinType::Full,
        _ => {
            return Err(XdlError::RuntimeError(format!(
                "Unknown join type: {}",
                how
            )))
        }
    };

    let result = df1
        .join(&df2, [&on_col], [&on_col], JoinArgs::new(join_type), None)
        .map_err(|e| XdlError::RuntimeError(format!("Failed to join: {}", e)))?;

    let id = store_dataframe(result);
    Ok(XdlValue::String(id))
}

/// DF_SHAPE - Get shape
pub fn df_shape(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "DF_SHAPE requires df_id".to_string(),
        ));
    }

    let df_id = match &args[0] {
        XdlValue::String(s) => s,
        _ => return Err(XdlError::RuntimeError("df_id must be a string".to_string())),
    };

    let df = get_dataframe(df_id)?;
    let shape = df.shape();
    Ok(XdlValue::Array(vec![shape.0 as f64, shape.1 as f64]))
}

/// DF_COLUMNS - Get column names
pub fn df_columns(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "DF_COLUMNS requires df_id".to_string(),
        ));
    }

    let df_id = match &args[0] {
        XdlValue::String(s) => s,
        _ => return Err(XdlError::RuntimeError("df_id must be a string".to_string())),
    };

    let df = get_dataframe(df_id)?;
    let columns: Vec<XdlValue> = df
        .get_column_names()
        .iter()
        .map(|s| XdlValue::String(s.to_string()))
        .collect();
    Ok(XdlValue::NestedArray(columns))
}

/// DF_DTYPES - Get column types
pub fn df_dtypes(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "DF_DTYPES requires df_id".to_string(),
        ));
    }

    let df_id = match &args[0] {
        XdlValue::String(s) => s,
        _ => return Err(XdlError::RuntimeError("df_id must be a string".to_string())),
    };

    let df = get_dataframe(df_id)?;
    let dtypes: Vec<XdlValue> = df
        .dtypes()
        .iter()
        .map(|dt| XdlValue::String(format!("{:?}", dt)))
        .collect();
    Ok(XdlValue::NestedArray(dtypes))
}

/// DF_DESCRIBE - Get basic statistics (shape, columns, dtypes)
pub fn df_describe(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "DF_DESCRIBE requires df_id".to_string(),
        ));
    }

    let df_id = match &args[0] {
        XdlValue::String(s) => s,
        _ => return Err(XdlError::RuntimeError("df_id must be a string".to_string())),
    };

    let df = get_dataframe(df_id)?;
    let shape = df.shape();
    let cols = df.get_column_names();
    let dtypes: Vec<String> = df.dtypes().iter().map(|dt| format!("{:?}", dt)).collect();

    // Return a summary string since describe() is not available in this version
    let summary = format!(
        "Shape: ({}, {})\nColumns: {:?}\nTypes: {:?}",
        shape.0, shape.1, cols, dtypes
    );
    Ok(XdlValue::String(summary))
}

/// DF_PRINT - Print DataFrame as string
pub fn df_print(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "DF_PRINT requires df_id".to_string(),
        ));
    }

    let df_id = match &args[0] {
        XdlValue::String(s) => s,
        _ => return Err(XdlError::RuntimeError("df_id must be a string".to_string())),
    };

    let df = get_dataframe(df_id)?;
    Ok(XdlValue::String(format!("{}", df)))
}

/// DF_TO_ARRAY - Convert column to array
pub fn df_to_array(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::RuntimeError(
            "DF_TO_ARRAY requires df_id, column".to_string(),
        ));
    }

    let df_id = match &args[0] {
        XdlValue::String(s) => s,
        _ => return Err(XdlError::RuntimeError("df_id must be a string".to_string())),
    };

    let column = match &args[1] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::RuntimeError(
                "Column must be a string".to_string(),
            ))
        }
    };

    let df = get_dataframe(df_id)?;
    let series = df
        .column(&column)
        .map_err(|e| XdlError::RuntimeError(format!("Column not found: {}", e)))?;

    match series.dtype() {
        DataType::Int64 | DataType::Int32 => {
            let arr: Vec<f64> = series
                .i64()
                .map_err(|e| XdlError::RuntimeError(e.to_string()))?
                .into_iter()
                .map(|v| v.unwrap_or(0) as f64)
                .collect();
            Ok(XdlValue::Array(arr))
        }
        DataType::Float64 => {
            let arr: Vec<f64> = series
                .f64()
                .map_err(|e| XdlError::RuntimeError(e.to_string()))?
                .into_iter()
                .map(|v| v.unwrap_or(0.0))
                .collect();
            Ok(XdlValue::Array(arr))
        }
        DataType::Float32 => {
            let arr: Vec<f64> = series
                .f32()
                .map_err(|e| XdlError::RuntimeError(e.to_string()))?
                .into_iter()
                .map(|v| v.unwrap_or(0.0) as f64)
                .collect();
            Ok(XdlValue::Array(arr))
        }
        DataType::String => {
            let arr: Vec<XdlValue> = series
                .str()
                .map_err(|e| XdlError::RuntimeError(e.to_string()))?
                .into_iter()
                .map(|v| XdlValue::String(v.unwrap_or("").to_string()))
                .collect();
            Ok(XdlValue::NestedArray(arr))
        }
        _ => Err(XdlError::RuntimeError(format!(
            "Unsupported type: {:?}",
            series.dtype()
        ))),
    }
}

/// DF_DROP - Remove DataFrame from memory
pub fn df_drop(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError("DF_DROP requires df_id".to_string()));
    }

    let df_id = match &args[0] {
        XdlValue::String(s) => s,
        _ => return Err(XdlError::RuntimeError("df_id must be a string".to_string())),
    };

    let mut storage = DATAFRAMES.lock().unwrap();
    storage.remove(df_id);
    Ok(XdlValue::Undefined)
}
