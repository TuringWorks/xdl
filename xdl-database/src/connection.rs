//! Database connection management

use crate::drivers;
use crate::{DatabaseError, DatabaseResult, DatabaseType, Recordset};

/// Database connection enum that wraps different database drivers
#[derive(Debug)]
pub enum DatabaseConnection {
    #[cfg(feature = "postgres-support")]
    PostgreSQL(drivers::postgres::PostgresConnection),

    #[cfg(feature = "mysql-support")]
    MySQL(drivers::mysql::MySQLConnection),

    #[cfg(feature = "duckdb-support")]
    DuckDB(drivers::duckdb::DuckDBConnection),

    #[cfg(feature = "odbc-support")]
    ODBC(drivers::odbc::ODBCConnection),

    #[cfg(feature = "redis-support")]
    Redis(drivers::redis_driver::RedisConnection),

    #[cfg(feature = "kafka-support")]
    Kafka(drivers::kafka::KafkaConnection),

    Unsupported(String),
}

impl DatabaseConnection {
    /// Create a new connection based on connection string and database type
    pub async fn new(connection_string: &str, db_type: DatabaseType) -> DatabaseResult<Self> {
        match db_type {
            #[cfg(feature = "postgres-support")]
            DatabaseType::PostgreSQL => {
                let conn =
                    drivers::postgres::PostgresConnection::connect(connection_string).await?;
                Ok(DatabaseConnection::PostgreSQL(conn))
            }

            #[cfg(feature = "mysql-support")]
            DatabaseType::MySQL => {
                let conn = drivers::mysql::MySQLConnection::connect(connection_string).await?;
                Ok(DatabaseConnection::MySQL(conn))
            }

            #[cfg(feature = "duckdb-support")]
            DatabaseType::DuckDB => {
                let conn = drivers::duckdb::DuckDBConnection::connect(connection_string).await?;
                Ok(DatabaseConnection::DuckDB(conn))
            }

            #[cfg(feature = "odbc-support")]
            DatabaseType::ODBC => {
                let conn = drivers::odbc::ODBCConnection::connect(connection_string).await?;
                Ok(DatabaseConnection::ODBC(conn))
            }

            #[cfg(feature = "redis-support")]
            DatabaseType::Redis => {
                let conn =
                    drivers::redis_driver::RedisConnection::connect(connection_string).await?;
                Ok(DatabaseConnection::Redis(conn))
            }

            #[cfg(feature = "kafka-support")]
            DatabaseType::Kafka => {
                let conn = drivers::kafka::KafkaConnection::connect(connection_string).await?;
                Ok(DatabaseConnection::Kafka(conn))
            }

            _ => Ok(DatabaseConnection::Unsupported(format!("{:?}", db_type))),
        }
    }

    /// Execute a query and return results
    pub async fn execute(&self, query: &str) -> DatabaseResult<Recordset> {
        match self {
            #[cfg(feature = "postgres-support")]
            DatabaseConnection::PostgreSQL(conn) => conn.execute(query).await,

            #[cfg(feature = "mysql-support")]
            DatabaseConnection::MySQL(conn) => conn.execute(query).await,

            #[cfg(feature = "duckdb-support")]
            DatabaseConnection::DuckDB(conn) => conn.execute(query).await,

            #[cfg(feature = "odbc-support")]
            DatabaseConnection::ODBC(conn) => conn.execute(query).await,

            #[cfg(feature = "redis-support")]
            DatabaseConnection::Redis(conn) => conn.execute(query).await,

            #[cfg(feature = "kafka-support")]
            DatabaseConnection::Kafka(conn) => conn.execute(query).await,

            DatabaseConnection::Unsupported(db_type) => {
                Err(DatabaseError::UnsupportedDatabase(db_type.clone()))
            }
        }
    }

    /// Execute a command (INSERT, UPDATE, DELETE) and return rows affected
    pub async fn execute_command(&self, command: &str) -> DatabaseResult<u64> {
        match self {
            #[cfg(feature = "postgres-support")]
            DatabaseConnection::PostgreSQL(conn) => conn.execute_command(command).await,

            #[cfg(feature = "mysql-support")]
            DatabaseConnection::MySQL(conn) => conn.execute_command(command).await,

            #[cfg(feature = "duckdb-support")]
            DatabaseConnection::DuckDB(conn) => conn.execute_command(command).await,

            #[cfg(feature = "odbc-support")]
            DatabaseConnection::ODBC(conn) => conn.execute_command(command).await,

            #[cfg(feature = "redis-support")]
            DatabaseConnection::Redis(conn) => conn.execute_command(command).await,

            #[cfg(feature = "kafka-support")]
            DatabaseConnection::Kafka(conn) => conn.execute_command(command).await,

            DatabaseConnection::Unsupported(db_type) => {
                Err(DatabaseError::UnsupportedDatabase(db_type.clone()))
            }
        }
    }

    /// Close the connection
    pub async fn close(&mut self) -> DatabaseResult<()> {
        match self {
            #[cfg(feature = "postgres-support")]
            DatabaseConnection::PostgreSQL(conn) => conn.close().await,

            #[cfg(feature = "mysql-support")]
            DatabaseConnection::MySQL(conn) => conn.close().await,

            #[cfg(feature = "duckdb-support")]
            DatabaseConnection::DuckDB(conn) => conn.close().await,

            #[cfg(feature = "odbc-support")]
            DatabaseConnection::ODBC(conn) => conn.close().await,

            #[cfg(feature = "redis-support")]
            DatabaseConnection::Redis(conn) => conn.close().await,

            #[cfg(feature = "kafka-support")]
            DatabaseConnection::Kafka(conn) => conn.close().await,

            DatabaseConnection::Unsupported(_) => Ok(()),
        }
    }

    /// Check if connection is alive
    pub async fn is_connected(&self) -> bool {
        match self {
            #[cfg(feature = "postgres-support")]
            DatabaseConnection::PostgreSQL(conn) => conn.is_connected().await,

            #[cfg(feature = "mysql-support")]
            DatabaseConnection::MySQL(conn) => conn.is_connected().await,

            #[cfg(feature = "duckdb-support")]
            DatabaseConnection::DuckDB(conn) => conn.is_connected().await,

            #[cfg(feature = "odbc-support")]
            DatabaseConnection::ODBC(conn) => conn.is_connected().await,

            #[cfg(feature = "redis-support")]
            DatabaseConnection::Redis(conn) => conn.is_connected().await,

            #[cfg(feature = "kafka-support")]
            DatabaseConnection::Kafka(conn) => conn.is_connected().await,

            DatabaseConnection::Unsupported(_) => false,
        }
    }
}
