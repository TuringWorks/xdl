//! Data format readers

pub mod avro;
pub mod csv;
pub mod parquet;

pub use csv::{read_csv, read_csv_string, write_csv, CsvReaderOptions};

#[cfg(feature = "parquet-support")]
pub use parquet::read_parquet;

#[cfg(feature = "avro-support")]
pub use avro::read_avro;
