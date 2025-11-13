# SQLite Implementation Guide - Code Examples & Reference Patterns

Based on existing drivers in the codebase, here are specific code patterns and examples for implementing SQLite support.

## 1. Pattern from DuckDB Driver (Synchronous)

The DuckDB driver is synchronous, making it a good reference for SQLite (also synchronous):

```rust
// Current DuckDB implementation pattern
pub struct DuckDBConnection {
    conn: Option<Mutex<Connection>>,
}

impl DuckDBConnection {
    pub async fn connect(connection_string: &str) -> DatabaseResult<Self> {
        let path = connection_string
            .trim_start_matches("duckdb://")
            .trim_start_matches("duckdb:");

        let conn = Connection::open(path).map_err(|e| {
            DatabaseError::connection_error(format!("DuckDB connection failed: {}", e))
        })?;

        Ok(Self {
            conn: Some(Mutex::new(conn)),
        })
    }

    pub async fn execute(&self, query: &str) -> DatabaseResult<Recordset> {
        let conn_mutex = self.conn.as_ref().ok_or(DatabaseError::NotConnected)?;
        let conn = conn_mutex
            .lock()
            .map_err(|e| DatabaseError::query_error(format!("Lock failed: {}", e)))?;

        let mut stmt = conn
            .prepare(query)
            .map_err(|e| DatabaseError::query_error(format!("Prepare failed: {}", e)))?;

        let column_count = stmt.column_count();
        let columns: Vec<ColumnInfo> = (0..column_count)
            .map(|i| ColumnInfo {
                name: stmt
                    .column_name(i)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|_| "unknown".to_string()),
                data_type: "unknown".to_string(),
                ordinal: i,
            })
            .collect();

        let mut rows_data = Vec::new();
        let mut rows = stmt
            .query(params![])
            .map_err(|e| DatabaseError::query_error(format!("Query failed: {}", e)))?;

        while let Some(row) = rows
            .next()
            .map_err(|e| DatabaseError::query_error(format!("Row fetch failed: {}", e)))?
        {
            let mut row_data = Vec::new();

            for i in 0..column_count {
                let val: Result<Option<String>, _> = row.get(i);
                let json_val = match val {
                    Ok(Some(s)) => JsonValue::String(s),
                    Ok(None) => JsonValue::Null,
                    Err(_) => JsonValue::Null,
                };
                row_data.push(json_val);
            }

            rows_data.push(row_data);
        }

        Ok(Recordset::new(columns, rows_data))
    }
}
```

## 2. Proposed SQLite Driver Implementation

Using rusqlite (similar pattern to DuckDB):

```rust
//! SQLite database driver
//!
//! Provides async wrapper around rusqlite (synchronous SQLite driver).
//! Supports file-based and in-memory databases.
//!
//! Connection string formats:
//! - sqlite:///path/to/database.sqlite
//! - sqlite://./relative/path.db
//! - sqlite://:memory: (in-memory database)

use crate::{recordset::ColumnInfo, DatabaseError, DatabaseResult, Recordset};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value as JsonValue;
use std::sync::Mutex;

/// SQLite connection
pub struct SQLiteConnection {
    conn: Option<Mutex<Connection>>,
}

impl std::fmt::Debug for SQLiteConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SQLiteConnection")
            .field("conn", &self.conn.is_some())
            .finish()
    }
}

impl SQLiteConnection {
    /// Connect to a SQLite database
    pub async fn connect(connection_string: &str) -> DatabaseResult<Self> {
        // Parse connection string
        let path = parse_sqlite_connection_string(connection_string)?;

        // Open connection
        let conn = Connection::open(&path).map_err(|e| {
            DatabaseError::connection_error(format!("SQLite connection failed: {}", e))
        })?;

        // Enable foreign keys and other pragmas
        conn.execute("PRAGMA foreign_keys = ON", [])
            .map_err(|e| {
                DatabaseError::connection_error(format!("Failed to enable foreign keys: {}", e))
            })?;

        Ok(Self {
            conn: Some(Mutex::new(conn)),
        })
    }

    /// Execute a SELECT query
    pub async fn execute(&self, query: &str) -> DatabaseResult<Recordset> {
        let conn_mutex = self.conn.as_ref().ok_or(DatabaseError::NotConnected)?;
        let conn = conn_mutex
            .lock()
            .map_err(|e| DatabaseError::query_error(format!("Lock failed: {}", e)))?;

        let mut stmt = conn
            .prepare(query)
            .map_err(|e| DatabaseError::query_error(format!("Prepare failed: {}", e)))?;

        let column_count = stmt.column_count();
        let columns: Vec<ColumnInfo> = (0..column_count)
            .map(|i| ColumnInfo {
                name: stmt
                    .column_name(i)
                    .unwrap_or("unknown")
                    .to_string(),
                data_type: "unknown".to_string(), // SQLite doesn't have strong typing
                ordinal: i,
            })
            .collect();

        let mut rows_data = Vec::new();
        let mut rows = stmt
            .query(params![])
            .map_err(|e| DatabaseError::query_error(format!("Query failed: {}", e)))?;

        while let Some(row) = rows
            .next()
            .map_err(|e| DatabaseError::query_error(format!("Row fetch failed: {}", e)))?
        {
            let mut row_data = Vec::new();

            for i in 0..column_count {
                let json_val = match row.get_ref(i) {
                    Ok(val_ref) => {
                        use rusqlite::types::ValueRef;
                        match val_ref {
                            ValueRef::Null => JsonValue::Null,
                            ValueRef::Integer(i) => JsonValue::Number(i.into()),
                            ValueRef::Real(f) => JsonValue::Number(
                                serde_json::Number::from_f64(f)
                                    .unwrap_or_else(|| serde_json::Number::from(0)),
                            ),
                            ValueRef::Text(bytes) => {
                                let s = String::from_utf8_lossy(bytes).to_string();
                                JsonValue::String(s)
                            }
                            ValueRef::Blob(bytes) => {
                                // Convert blob to hex string or base64
                                let hex = hex::encode(bytes);
                                JsonValue::String(format!("blob:{}", hex))
                            }
                        }
                    }
                    Err(_) => JsonValue::Null,
                };
                row_data.push(json_val);
            }

            rows_data.push(row_data);
        }

        Ok(Recordset::new(columns, rows_data))
    }

    /// Execute a command (INSERT, UPDATE, DELETE, CREATE, etc.)
    pub async fn execute_command(&self, command: &str) -> DatabaseResult<u64> {
        let conn_mutex = self.conn.as_ref().ok_or(DatabaseError::NotConnected)?;
        let conn = conn_mutex
            .lock()
            .map_err(|e| DatabaseError::query_error(format!("Lock failed: {}", e)))?;

        let affected = conn
            .execute(command, params![])
            .map_err(|e| DatabaseError::query_error(format!("Command failed: {}", e)))?;

        Ok(affected as u64)
    }

    /// Close the connection
    pub async fn close(&mut self) -> DatabaseResult<()> {
        if let Some(mutex) = self.conn.take() {
            if let Ok(conn) = mutex.into_inner() {
                conn.close()
                    .map_err(|(_, e)| DatabaseError::connection_error(format!("Close failed: {}", e)))?;
            }
        }
        Ok(())
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        self.conn.is_some()
    }
}

/// Parse SQLite connection string
fn parse_sqlite_connection_string(conn_str: &str) -> DatabaseResult<String> {
    let lower = conn_str.to_lowercase();

    // Handle various formats
    if lower.starts_with("sqlite:///") {
        // sqlite:///path/to/db.sqlite → /path/to/db.sqlite
        let path = conn_str.trim_start_matches("sqlite://");
        Ok(path.to_string())
    } else if lower.starts_with("sqlite://") {
        // sqlite://./local.db → ./local.db
        let path = conn_str.trim_start_matches("sqlite://");
        Ok(path.to_string())
    } else if lower.starts_with("sqlite:") {
        // sqlite::memory: → :memory:
        let path = conn_str.trim_start_matches("sqlite:");
        Ok(path.to_string())
    } else if lower.ends_with(".sqlite") || lower.ends_with(".db") {
        // Direct file path
        Ok(conn_str.to_string())
    } else {
        Err(DatabaseError::InvalidConnectionString(format!(
            "Invalid SQLite connection string: {}",
            conn_str
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_connection_strings() {
        assert_eq!(
            parse_sqlite_connection_string("sqlite:///tmp/test.db").unwrap(),
            "/tmp/test.db"
        );
        assert_eq!(
            parse_sqlite_connection_string("sqlite://./local.db").unwrap(),
            "./local.db"
        );
        assert_eq!(
            parse_sqlite_connection_string("sqlite://:memory:").unwrap(),
            ":memory:"
        );
        assert_eq!(
            parse_sqlite_connection_string("test.db").unwrap(),
            "test.db"
        );
    }

    #[tokio::test]
    async fn test_in_memory_connection() {
        let conn = SQLiteConnection::connect("sqlite://:memory:").await;
        assert!(conn.is_ok());
    }
}
```

## 3. Type Conversion Implementation

SQLite to JSON value conversion (more complete than DuckDB):

```rust
// In sqlite.rs driver

use rusqlite::types::ValueRef;

fn sqlite_value_to_json(val_ref: ValueRef) -> JsonValue {
    match val_ref {
        ValueRef::Null => JsonValue::Null,

        ValueRef::Integer(i) => {
            JsonValue::Number(serde_json::Number::from(i))
        }

        ValueRef::Real(f) => {
            serde_json::Number::from_f64(f)
                .map(JsonValue::Number)
                .unwrap_or(JsonValue::Null)
        }

        ValueRef::Text(bytes) => {
            String::from_utf8_lossy(bytes)
                .to_string()
                .into()
        }

        ValueRef::Blob(bytes) => {
            // Option 1: Hex encoding
            format!("blob:{}", hex::encode(bytes)).into()

            // Option 2: Base64 encoding
            // format!("blob:{}", base64::encode(bytes)).into()

            // Option 3: Raw bytes as string
            // String::from_utf8_lossy(bytes).to_string().into()
        }
    }
}
```

## 4. Changes to lib.rs

Adding SQLite support to DatabaseType enum:

```rust
// In xdl-database/src/lib.rs

#[derive(Debug, Clone, PartialEq)]
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
    DuckDB,
    ODBC,
    Redis,
    Kafka,
    SQLite,  // ADD THIS
    Unknown,
}

impl DatabaseType {
    pub fn from_connection_string(conn_str: &str) -> Self {
        let lower = conn_str.to_lowercase();

        // ... existing patterns ...

        // ADD SQLITE DETECTION
        if lower.starts_with("sqlite://")
            || lower.starts_with("sqlite:")
            || lower.ends_with(".sqlite")
            || lower.ends_with(".db") {
            DatabaseType::SQLite
        } else {
            DatabaseType::Unknown
        }
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_type_detection() {
        // ... existing tests ...

        // ADD SQLITE TESTS
        assert_eq!(
            DatabaseType::from_connection_string("sqlite:///tmp/test.db"),
            DatabaseType::SQLite
        );
        assert_eq!(
            DatabaseType::from_connection_string("sqlite://:memory:"),
            DatabaseType::SQLite
        );
        assert_eq!(
            DatabaseType::from_connection_string("test.sqlite"),
            DatabaseType::SQLite
        );
        assert_eq!(
            DatabaseType::from_connection_string("test.db"),
            DatabaseType::SQLite
        );
    }
}
```

## 5. Changes to connection.rs

Adding SQLite to DatabaseConnection enum:

```rust
// In xdl-database/src/connection.rs

#[derive(Debug)]
pub enum DatabaseConnection {
    #[cfg(feature = "postgres-support")]
    PostgreSQL(drivers::postgres::PostgresConnection),

    #[cfg(feature = "mysql-support")]
    MySQL(drivers::mysql::MySQLConnection),

    #[cfg(feature = "duckdb-support")]
    DuckDB(drivers::duckdb::DuckDBConnection),

    #[cfg(feature = "sqlite-support")]  // ADD THIS
    SQLite(drivers::sqlite::SQLiteConnection),  // ADD THIS

    #[cfg(feature = "odbc-support")]
    ODBC(drivers::odbc::ODBCConnection),

    #[cfg(feature = "redis-support")]
    Redis(drivers::redis_driver::RedisConnection),

    #[cfg(feature = "kafka-support")]
    Kafka(drivers::kafka::KafkaConnection),

    Unsupported(String),
}

impl DatabaseConnection {
    pub async fn new(connection_string: &str, db_type: DatabaseType) -> DatabaseResult<Self> {
        match db_type {
            // ... existing cases ...

            #[cfg(feature = "sqlite-support")]  // ADD THIS
            DatabaseType::SQLite => {  // ADD THIS
                let conn = drivers::sqlite::SQLiteConnection::connect(connection_string).await?;
                Ok(DatabaseConnection::SQLite(conn))
            }  // ADD THIS

            _ => Ok(DatabaseConnection::Unsupported(format!("{:?}", db_type))),
        }
    }

    pub async fn execute(&self, query: &str) -> DatabaseResult<Recordset> {
        match self {
            // ... existing cases ...

            #[cfg(feature = "sqlite-support")]  // ADD THIS
            DatabaseConnection::SQLite(conn) => conn.execute(query).await,  // ADD THIS

            DatabaseConnection::Unsupported(db_type) => {
                Err(DatabaseError::UnsupportedDatabase(db_type.clone()))
            }
        }
    }

    pub async fn execute_command(&self, command: &str) -> DatabaseResult<u64> {
        match self {
            // ... existing cases ...

            #[cfg(feature = "sqlite-support")]  // ADD THIS
            DatabaseConnection::SQLite(conn) => conn.execute_command(command).await,  // ADD THIS

            DatabaseConnection::Unsupported(db_type) => {
                Err(DatabaseError::UnsupportedDatabase(db_type.clone()))
            }
        }
    }

    pub async fn close(&mut self) -> DatabaseResult<()> {
        match self {
            // ... existing cases ...

            #[cfg(feature = "sqlite-support")]  // ADD THIS
            DatabaseConnection::SQLite(conn) => conn.close().await,  // ADD THIS

            DatabaseConnection::Unsupported(_) => Ok(()),
        }
    }

    pub async fn is_connected(&self) -> bool {
        match self {
            // ... existing cases ...

            #[cfg(feature = "sqlite-support")]  // ADD THIS
            DatabaseConnection::SQLite(conn) => conn.is_connected().await,  // ADD THIS

            DatabaseConnection::Unsupported(_) => false,
        }
    }
}
```

## 6. Changes to error.rs

Adding SQLite error variant:

```rust
// In xdl-database/src/error.rs

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

    // ... existing variants ...

    #[cfg(feature = "sqlite-support")]  // ADD THIS
    #[error("SQLite error: {0}")]  // ADD THIS
    SQLiteError(String),  // ADD THIS
}
```

## 7. Changes to drivers/mod.rs

```rust
// In xdl-database/src/drivers/mod.rs

#[cfg(feature = "postgres-support")]
pub mod postgres;

#[cfg(feature = "mysql-support")]
pub mod mysql;

#[cfg(feature = "duckdb-support")]
pub mod duckdb;

#[cfg(feature = "sqlite-support")]  // ADD THIS
pub mod sqlite;  // ADD THIS

#[cfg(feature = "odbc-support")]
pub mod odbc;

#[cfg(feature = "redis-support")]
pub mod redis_driver;

#[cfg(feature = "kafka-support")]
pub mod kafka;
```

## 8. Changes to Cargo.toml

```toml
[dependencies]
# ... existing dependencies ...

# SQLite
rusqlite = { version = "0.31", features = ["bundled"], optional = true }
# Optional: for hex encoding of blobs
hex = "0.4"

[features]
default = ["postgres-support", "duckdb-support", "redis-support"]

# Individual database support
postgres-support = ["tokio-postgres", "deadpool-postgres"]
mysql-support = ["mysql_async"]
duckdb-support = ["duckdb"]
sqlite-support = ["rusqlite"]  # ADD THIS
odbc-support = ["odbc-api"]
redis-support = ["redis"]
kafka-support = ["rdkafka"]

# Enable all databases
all = [
    "postgres-support",
    "mysql-support",
    "duckdb-support",
    "sqlite-support",  # ADD THIS
    "odbc-support",
    "redis-support",
    "kafka-support"
]
```

## 9. Usage Example (XDL Code)

```xdl
; Create database object
objdb = OBJ_NEW('XDLdbDatabase')

; Connect to SQLite
objdb->Connect, CONNECTION='sqlite:///./mydata.sqlite'

; Create table
objdb->ExecuteCommand, $
  'CREATE TABLE IF NOT EXISTS products (' + $
  '  id INTEGER PRIMARY KEY,' + $
  '  name TEXT NOT NULL,' + $
  '  price REAL NOT NULL' + $
  ')'

; Insert data
objdb->ExecuteCommand, "INSERT INTO products (name, price) VALUES ('Widget', 19.99)"
objdb->ExecuteCommand, "INSERT INTO products (name, price) VALUES ('Gadget', 29.99)"

; Query data
recordset = objdb->ExecuteSQL('SELECT * FROM products WHERE price > 20')

; Process results
data = recordset->GetData()
PRINT, 'Products over $20:', data

; Get individual column
names = recordset->GetColumn('name')
PRINT, 'Product names:', names

; Cleanup
recordset->Destroy()
objdb->Disconnect()
OBJ_DESTROY, objdb
```

---

## Key Implementation Notes

1. **Synchronous Driver**: SQLite driver is synchronous (rusqlite) wrapped in async interface (like DuckDB)
2. **Simple Connection**: No connection pooling needed (unlike PostgreSQL/MySQL)
3. **Type Handling**: SQLite is dynamically typed; all types map to JSON
4. **Blob Handling**: Binary data converts to hex string with "blob:" prefix
5. **In-Memory Support**: `sqlite://:memory:` creates temporary databases
6. **File-Based**: Default for production use
7. **Connection String Detection**: Auto-detects `.db` and `.sqlite` file extensions
