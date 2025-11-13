//! CSV and TSV readers

use crate::dataframe::DataFrame;
use crate::error::{DataFrameError, DataFrameResult};
use crate::series::Series;
use csv::{ReaderBuilder, StringRecord};
use indexmap::IndexMap;
use std::path::Path;
use xdl_core::XdlValue;

/// CSV Reader options
#[derive(Debug, Clone)]
pub struct CsvReaderOptions {
    /// Delimiter character (comma, tab, etc.)
    pub delimiter: u8,
    /// Whether the file has a header row
    pub has_headers: bool,
    /// Skip N rows from the beginning
    pub skip_rows: usize,
    /// Maximum rows to read (None = read all)
    pub max_rows: Option<usize>,
    /// Automatically infer types
    pub infer_types: bool,
}

impl Default for CsvReaderOptions {
    fn default() -> Self {
        Self {
            delimiter: b',',
            has_headers: true,
            skip_rows: 0,
            max_rows: None,
            infer_types: true,
        }
    }
}

impl CsvReaderOptions {
    /// Create options for CSV files
    pub fn csv() -> Self {
        Self::default()
    }

    /// Create options for TSV files
    pub fn tsv() -> Self {
        Self {
            delimiter: b'\t',
            ..Self::default()
        }
    }

    /// Set delimiter
    pub fn with_delimiter(mut self, delimiter: u8) -> Self {
        self.delimiter = delimiter;
        self
    }

    /// Set whether file has headers
    pub fn with_headers(mut self, has_headers: bool) -> Self {
        self.has_headers = has_headers;
        self
    }

    /// Set number of rows to skip
    pub fn with_skip_rows(mut self, skip_rows: usize) -> Self {
        self.skip_rows = skip_rows;
        self
    }

    /// Set maximum rows to read
    pub fn with_max_rows(mut self, max_rows: usize) -> Self {
        self.max_rows = Some(max_rows);
        self
    }

    /// Set type inference
    pub fn with_infer_types(mut self, infer_types: bool) -> Self {
        self.infer_types = infer_types;
        self
    }
}

/// Read CSV file into a DataFrame
pub fn read_csv<P: AsRef<Path>>(path: P, options: CsvReaderOptions) -> DataFrameResult<DataFrame> {
    let mut reader = ReaderBuilder::new()
        .delimiter(options.delimiter)
        .has_headers(options.has_headers)
        .from_path(path)?;

    // Get headers
    let headers = if options.has_headers {
        reader.headers()?.clone()
    } else {
        // Generate column names if no headers
        let first_record = reader
            .records()
            .next()
            .ok_or_else(|| DataFrameError::ParseError("Empty CSV file".to_string()))??;
        let num_cols = first_record.len();
        let mut headers = StringRecord::new();
        for i in 0..num_cols {
            headers.push_field(&format!("col_{}", i));
        }
        headers
    };

    let num_cols = headers.len();

    // Initialize column vectors
    let mut column_data: Vec<Vec<String>> = vec![vec![]; num_cols];

    // Read records
    let mut row_count = 0;
    for (idx, result) in reader.records().enumerate() {
        if idx < options.skip_rows {
            continue;
        }

        if let Some(max) = options.max_rows {
            if row_count >= max {
                break;
            }
        }

        let record = result?;
        for (col_idx, field) in record.iter().enumerate() {
            if col_idx < num_cols {
                column_data[col_idx].push(field.to_string());
            }
        }
        row_count += 1;
    }

    // Convert to DataFrame
    let mut columns = IndexMap::new();
    for (col_idx, header) in headers.iter().enumerate() {
        let col_values = if options.infer_types {
            infer_and_convert_types(&column_data[col_idx])
        } else {
            column_data[col_idx]
                .iter()
                .map(|s| XdlValue::String(s.clone()))
                .collect()
        };

        columns.insert(header.to_string(), Series::from_vec(col_values)?);
    }

    DataFrame::from_columns(columns)
}

/// Read CSV from string
pub fn read_csv_string(content: &str, options: CsvReaderOptions) -> DataFrameResult<DataFrame> {
    let mut reader = ReaderBuilder::new()
        .delimiter(options.delimiter)
        .has_headers(options.has_headers)
        .from_reader(content.as_bytes());

    let headers = if options.has_headers {
        reader.headers()?.clone()
    } else {
        let first_record = reader
            .records()
            .next()
            .ok_or_else(|| DataFrameError::ParseError("Empty CSV string".to_string()))??;
        let num_cols = first_record.len();
        let mut headers = StringRecord::new();
        for i in 0..num_cols {
            headers.push_field(&format!("col_{}", i));
        }
        headers
    };

    let num_cols = headers.len();
    let mut column_data: Vec<Vec<String>> = vec![vec![]; num_cols];

    let mut row_count = 0;
    for (idx, result) in reader.records().enumerate() {
        if idx < options.skip_rows {
            continue;
        }

        if let Some(max) = options.max_rows {
            if row_count >= max {
                break;
            }
        }

        let record = result?;
        for (col_idx, field) in record.iter().enumerate() {
            if col_idx < num_cols {
                column_data[col_idx].push(field.to_string());
            }
        }
        row_count += 1;
    }

    let mut columns = IndexMap::new();
    for (col_idx, header) in headers.iter().enumerate() {
        let col_values = if options.infer_types {
            infer_and_convert_types(&column_data[col_idx])
        } else {
            column_data[col_idx]
                .iter()
                .map(|s| XdlValue::String(s.clone()))
                .collect()
        };

        columns.insert(header.to_string(), Series::from_vec(col_values)?);
    }

    DataFrame::from_columns(columns)
}

/// Infer types and convert string values to appropriate XdlValue types
fn infer_and_convert_types(values: &[String]) -> Vec<XdlValue> {
    if values.is_empty() {
        return vec![];
    }

    // Try to determine column type
    let mut is_int = true;
    let mut is_float = true;

    for val in values.iter().take(100.min(values.len())) {
        if val.is_empty() {
            continue;
        }

        if is_int && val.parse::<i64>().is_err() {
            is_int = false;
        }

        if is_float && val.parse::<f64>().is_err() {
            is_float = false;
        }

        if !is_int && !is_float {
            break;
        }
    }

    // Convert based on inferred type
    values
        .iter()
        .map(|s| {
            if s.is_empty() {
                return XdlValue::Undefined;
            }

            if is_int {
                if let Ok(i) = s.parse::<i32>() {
                    return XdlValue::Long(i);
                } else if let Ok(i) = s.parse::<i64>() {
                    return XdlValue::Long64(i);
                }
            }

            if is_float {
                if let Ok(f) = s.parse::<f64>() {
                    return XdlValue::Double(f);
                }
            }

            XdlValue::String(s.clone())
        })
        .collect()
}

/// Write DataFrame to CSV file
pub fn write_csv<P: AsRef<Path>>(
    dataframe: &DataFrame,
    path: P,
    delimiter: u8,
) -> DataFrameResult<()> {
    use std::fs::File;

    let file = File::create(path)?;
    let mut writer = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .from_writer(file);

    // Write headers
    writer.write_record(dataframe.column_names())?;

    // Write rows
    for row_idx in 0..dataframe.nrows() {
        let row = dataframe.row(row_idx)?;
        let row_strings: Vec<String> = dataframe
            .column_names()
            .iter()
            .map(|col_name| {
                row.get(col_name)
                    .map(|v| v.to_string_repr())
                    .unwrap_or_default()
            })
            .collect();
        writer.write_record(&row_strings)?;
    }

    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_csv_string() {
        let csv_data = "name,age,city\nAlice,30,NYC\nBob,25,LA\nCarol,35,Chicago";

        let df = read_csv_string(csv_data, CsvReaderOptions::csv()).unwrap();

        assert_eq!(df.nrows(), 3);
        assert_eq!(df.ncols(), 3);
        assert_eq!(df.column_names(), vec!["name", "age", "city"]);
    }

    #[test]
    fn test_read_tsv_string() {
        let tsv_data = "name\tage\tcity\nAlice\t30\tNYC\nBob\t25\tLA";

        let df = read_csv_string(tsv_data, CsvReaderOptions::tsv()).unwrap();

        assert_eq!(df.nrows(), 2);
        assert_eq!(df.ncols(), 3);
    }

    #[test]
    fn test_type_inference() {
        let csv_data = "int_col,float_col,str_col\n1,1.5,hello\n2,2.5,world\n3,3.5,test";

        let df = read_csv_string(csv_data, CsvReaderOptions::csv()).unwrap();

        // Verify columns exist
        assert!(df.column("int_col").is_ok());
        assert!(df.column("float_col").is_ok());
        assert!(df.column("str_col").is_ok());
    }
}
