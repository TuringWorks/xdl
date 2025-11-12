//! Database error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Query execution error: {0}")]
    QueryError(String),

    #[error("Data conversion error: {0}")]
    ConversionError(String),

    #[error("Not connected to database")]
    NotConnected,

    #[error("Unsupported database type: {0}")]
    UnsupportedDatabase(String),

    #[error("Invalid connection string: {0}")]
    InvalidConnectionString(String),

    #[error("Database-specific error: {0}")]
    DatabaseSpecific(String),

    #[cfg(feature = "postgres-support")]
    #[error("PostgreSQL error: {0}")]
    PostgresError(#[from] tokio_postgres::Error),

    #[cfg(feature = "mysql-support")]
    #[error("MySQL error: {0}")]
    MySQLError(String),

    #[cfg(feature = "duckdb-support")]
    #[error("DuckDB error: {0}")]
    DuckDBError(String),

    #[cfg(feature = "odbc-support")]
    #[error("ODBC error: {0}")]
    ODBCError(String),

    #[cfg(feature = "redis-support")]
    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(String),
}

impl DatabaseError {
    pub fn connection_error(msg: impl Into<String>) -> Self {
        Self::ConnectionError(msg.into())
    }

    pub fn query_error(msg: impl Into<String>) -> Self {
        Self::QueryError(msg.into())
    }

    pub fn conversion_error(msg: impl Into<String>) -> Self {
        Self::ConversionError(msg.into())
    }
}

pub type DatabaseResult<T> = Result<T, DatabaseError>;
