//! XDL DataFrame Module
//!
//! Provides pandas/Spark-style data manipulation capabilities for XDL with support for:
//! - CSV and TSV files
//! - Parquet files
//! - Avro files
//! - Database query results
//!
//! # Features
//!
//! - **DataFrame Operations**: filter, select, groupby, sort, join
//! - **Data I/O**: Read from multiple formats, write to CSV/TSV
//! - **Statistics**: describe, sum, mean, count, value_counts
//! - **Integration**: Works seamlessly with XDL arrays, ML functions, charts, and 3D graphics
//!
//! # Example
//!
//! ```xdl
//! ; Read CSV data
//! df = XDLDataFrame_ReadCSV('data.csv')
//!
//! ; Select columns
//! df_subset = df->Select(['name', 'age', 'city'])
//!
//! ; Filter rows
//! df_filtered = df->Filter(AGE='>30')
//!
//! ; Group and aggregate
//! df_grouped = df->GroupBy(['city'])->Mean()
//!
//! ; Statistical summary
//! stats = df->Describe()
//!
//! ; Export to different format
//! df->WriteTSV, 'output.tsv'
//! ```

#[cfg(feature = "database-integration")]
pub mod database;
pub mod dataframe;
pub mod error;
pub mod readers;
pub mod series;

pub use dataframe::{DataFrame, GroupBy};
pub use error::{DataFrameError, DataFrameResult};
pub use readers::{read_csv, read_csv_string, write_csv, CsvReaderOptions};
pub use series::Series;

#[cfg(feature = "parquet-support")]
pub use readers::read_parquet;

#[cfg(feature = "avro-support")]
pub use readers::read_avro;

#[cfg(feature = "database-integration")]
pub use database::from_recordset;

/// Module version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
