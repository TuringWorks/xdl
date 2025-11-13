//! Error types for DataFrame operations

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataFrameError {
    #[error("Column not found: {0}")]
    ColumnNotFound(String),

    #[error("Index out of bounds: {0} (size: {1})")]
    IndexOutOfBounds(usize, usize),

    #[error("Dimension mismatch: {0}")]
    DimensionMismatch(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("CSV error: {0}")]
    CsvError(#[from] csv::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[cfg(feature = "parquet-support")]
    #[error("Parquet error: {0}")]
    ParquetError(String),

    #[cfg(feature = "avro-support")]
    #[error("Avro error: {0}")]
    AvroError(String),

    #[error("XDL error: {0}")]
    XdlError(#[from] xdl_core::XdlError),

    #[error("Other error: {0}")]
    Other(String),
}

pub type DataFrameResult<T> = Result<T, DataFrameError>;
