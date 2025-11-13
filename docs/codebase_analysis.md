# XDL Codebase Exploration Summary

## Project Overview

**XDL (Extended Data Language)** is a Rust-based implementation similar to IDL/GDL (Interactive Data Language/GNU Data Language) with multiple specialized modules for different functionality.

### Project Structure

The project is organized as a **Cargo workspace** with 17 member crates:

```
xdl/
├── xdl-core/              # Core data structures and types (XdlValue, etc.)
├── xdl-parser/            # XDL language parser
├── xdl-interpreter/       # Interpreter for parsed XDL code
├── xdl-runtime/           # Runtime environment
├── xdl-stdlib/            # Standard library functions
├── xdl-cli/               # Command-line interface
├── xdl-ffi/               # Foreign Function Interface
├── xdl-gui/               # GUI applications
├── xdl-database/          # DATABASE CONNECTIVITY MODULE (Focus area)
├── xdl-viz3d/             # 3D visualization
├── xdl-viz3d-web/         # 3D visualization web
├── xdl-viz3d-threejs/     # Three.js visualization
├── xdl-charts/            # Charting support
├── xdl-chart-viewer/      # Chart viewer
├── xdl-matlab/            # MATLAB transpiler
├── xdl-amp/               # Accelerated math processing
└── xdl-desktop-viewer/    # Desktop visualization viewer
```

---

## Database Connectivity Implementation

### Location
**Primary Module**: `/Users/ravindraboddipalli/sources/xdl/xdl-database/`

### Current Database Support

The xdl-database crate currently supports **6 database systems**:

1. **PostgreSQL** ✅ (Fully implemented)
   - Driver: `tokio-postgres` with connection pooling via `deadpool-postgres`
   - Feature flag: `postgres-support`
   - Async/await: Full support

2. **MySQL** ✅ (Fully implemented)
   - Driver: `mysql_async` (native async driver)
   - Feature flag: `mysql-support`
   - Async/await: Full support with connection pooling

3. **DuckDB** ✅ (Fully implemented)
   - Driver: `duckdb` crate with bundled support
   - Feature flag: `duckdb-support`
   - Note: Uses synchronous driver wrapped in async context

4. **ODBC** ✅ (Fully implemented)
   - Driver: `odbc-api` v8.0
   - Feature flag: `odbc-support`
   - Supports: SQL Server, Oracle, MySQL, PostgreSQL, and other ODBC-compatible databases

5. **Redis** ✅ (Fully implemented)
   - Driver: `redis` v0.27 with `tokio-comp` feature
   - Feature flag: `redis-support`
   - Note: Key-value store, not relational

6. **Apache Kafka** ✅ (Fully implemented)
   - Driver: `rdkafka` v0.36 with tokio integration
   - Feature flag: `kafka-support`
   - Note: Streaming/messaging platform, not a database

### Dependencies (from Cargo.toml)

```toml
# Async runtime
tokio = { version = "1.0", features = ["full"] }
async-trait = "0.1"

# Database drivers (all optional)
sqlx = { version = "0.8", optional = true }
postgres = { version = "0.19", optional = true }
tokio-postgres = { version = "0.7", features = ["with-serde_json-1"], optional = true }
mysql_async = { version = "0.34", optional = true }
duckdb = { version = "1.1", features = ["bundled"], optional = true }
odbc-api = { version = "8.0", optional = true }
redis = { version = "0.27", features = ["tokio-comp", "connection-manager"], optional = true }
rdkafka = { version = "0.36", features = ["tokio"], optional = true }

# Connection pooling
deadpool = { version = "0.12", optional = true }
deadpool-postgres = { version = "0.14", optional = true }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
thiserror = "1.0"
anyhow = "1.0"
```

### Module Structure

```
xdl-database/src/
├── lib.rs                 # Main module, XDLDatabase and DatabaseRegistry
├── connection.rs          # DatabaseConnection enum (dispatches to drivers)
├── drivers/
│   ├── mod.rs            # Driver module declarations
│   ├── postgres.rs       # PostgreSQL driver implementation
│   ├── mysql.rs          # MySQL driver implementation
│   ├── duckdb.rs         # DuckDB driver implementation
│   ├── odbc.rs           # ODBC driver implementation
│   ├── redis_driver.rs   # Redis driver implementation
│   └── kafka.rs          # Kafka driver implementation
├── recordset.rs          # Results representation and manipulation
└── error.rs              # Error types and conversions
```

---

## Current Architecture & Patterns

### 1. **DatabaseType Enum**
Located in `lib.rs`, automatically detects database type from connection string:

```rust
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
    DuckDB,
    ODBC,
    Redis,
    Kafka,
    Unknown,
}

impl DatabaseType {
    pub fn from_connection_string(conn_str: &str) -> Self {
        // Detects from URL scheme or patterns
        // postgresql:// → PostgreSQL
        // mysql:// → MySQL
        // *.duckdb or *.db → DuckDB
        // redis:// → Redis
        // kafka:// → Kafka
        // DRIVER={...} → ODBC
    }
}
```

### 2. **Abstraction Pattern: DatabaseConnection Enum**
Located in `connection.rs`, provides unified interface:

```rust
pub enum DatabaseConnection {
    PostgreSQL(PostgresConnection),
    MySQL(MySQLConnection),
    DuckDB(DuckDBConnection),
    ODBC(ODBCConnection),
    Redis(RedisConnection),
    Kafka(KafkaConnection),
    Unsupported(String),
}

impl DatabaseConnection {
    pub async fn new(conn_str: &str, db_type: DatabaseType) -> DatabaseResult<Self>
    pub async fn execute(&self, query: &str) -> DatabaseResult<Recordset>
    pub async fn execute_command(&self, command: &str) -> DatabaseResult<u64>
    pub async fn close(&mut self) -> DatabaseResult<()>
    pub async fn is_connected(&self) -> bool
}
```

### 3. **Common Driver Interface**
Each driver implements the same interface pattern:

```rust
pub struct DriverConnection {
    // driver-specific connection state
}

impl DriverConnection {
    pub async fn connect(connection_string: &str) -> DatabaseResult<Self>
    pub async fn execute(&self, query: &str) -> DatabaseResult<Recordset>
    pub async fn execute_command(&self, command: &str) -> DatabaseResult<u64>
    pub async fn close(&mut self) -> DatabaseResult<()>
    pub async fn is_connected(&self) -> bool
}
```

### 4. **Recordset Structure**
Unified result representation:

```rust
pub struct Recordset {
    columns: Vec<ColumnInfo>,  // Column metadata
    rows: Vec<Vec<JsonValue>>, // Row data in JSON format
    current_row: usize,         // Navigation pointer
}

pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub ordinal: usize,
}
```

### 5. **Error Handling**
Unified error type with database-specific variants:

```rust
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Query execution error: {0}")]
    QueryError(String),

    // ... more variants including driver-specific ones
}

pub type DatabaseResult<T> = Result<T, DatabaseError>;
```

### 6. **Global Registry Pattern**
For XDL object system integration:

```rust
pub struct DatabaseRegistry {
    databases: RwLock<HashMap<usize, Arc<RwLock<XDLDatabase>>>>,
    recordsets: RwLock<HashMap<usize, Arc<RwLock<Recordset>>>>,
    next_id: RwLock<usize>,
}

// Global instance
lazy_static::lazy_static! {
    pub static ref GLOBAL_DB_REGISTRY: DatabaseRegistry = DatabaseRegistry::new();
}
```

---

## SQLite Support - Recommended Implementation

### Why SQLite?

**SQLite is NOT currently supported** despite being one of the most popular embedded databases. It would be valuable to add because:

1. **Embedded database** - No server required, perfect for embedded/desktop scenarios
2. **Wide compatibility** - Works on all platforms (Linux, macOS, Windows)
3. **Zero-config** - File-based, simple connection strings
4. **Popular** - Widely used in applications
5. **Lightweight** - Minimal dependencies

### Recommended Location for SQLite Support

**File Path**: `/Users/ravindraboddipalli/sources/xdl/xdl-database/src/drivers/sqlite.rs`

### Implementation Strategy

#### Step 1: Add Dependencies to Cargo.toml

```toml
# In [dependencies] section
rusqlite = { version = "0.31", features = ["bundled", "tokio"], optional = true }
# OR async alternative
sqlx = { version = "0.8", features = ["runtime-tokio-native-tls", "sqlite"], optional = true }

# In [features] section
sqlite-support = ["rusqlite"]  # or sqlx with sqlite feature
```

#### Step 2: Create SQLite Driver

**File**: `xdl-database/src/drivers/sqlite.rs`

Following the existing pattern:

```rust
//! SQLite database driver

use crate::{recordset::ColumnInfo, DatabaseError, DatabaseResult, Recordset};
use rusqlite::{Connection, params};
use serde_json::Value as JsonValue;
use std::sync::Mutex;

pub struct SQLiteConnection {
    conn: Option<Mutex<Connection>>,
}

impl SQLiteConnection {
    pub async fn connect(connection_string: &str) -> DatabaseResult<Self> {
        // Parse connection string (e.g., "sqlite:///path/to/db.sqlite")
        // Open connection using rusqlite::Connection::open()
        // Return Self { conn: Some(Mutex::new(conn)) }
    }

    pub async fn execute(&self, query: &str) -> DatabaseResult<Recordset> {
        // Execute SELECT query
        // Extract column names and types
        // Convert rows to Vec<Vec<JsonValue>>
        // Return Recordset
    }

    pub async fn execute_command(&self, command: &str) -> DatabaseResult<u64> {
        // Execute INSERT/UPDATE/DELETE/CREATE commands
        // Return affected row count
    }

    pub async fn close(&mut self) -> DatabaseResult<()> {
        self.conn = None;
        Ok(())
    }

    pub async fn is_connected(&self) -> bool {
        self.conn.is_some()
    }
}
```

#### Step 3: Update DatabaseType Enum

**File**: `xdl-database/src/lib.rs`

```rust
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
    DuckDB,
    ODBC,
    Redis,
    Kafka,
    SQLite,  // Add this
    Unknown,
}

impl DatabaseType {
    pub fn from_connection_string(conn_str: &str) -> Self {
        // ... existing patterns ...

        // Add SQLite detection
        if lower.starts_with("sqlite://")
            || lower.starts_with("sqlite:")
            || lower.ends_with(".sqlite")
            || lower.ends_with(".db") {
            DatabaseType::SQLite
        } else {
            // ... rest of detection
        }
    }
}
```

#### Step 4: Update DatabaseConnection Enum

**File**: `xdl-database/src/connection.rs`

```rust
pub enum DatabaseConnection {
    PostgreSQL(drivers::postgres::PostgresConnection),
    MySQL(drivers::mysql::MySQLConnection),
    DuckDB(drivers::duckdb::DuckDBConnection),
    ODBC(drivers::odbc::ODBCConnection),
    Redis(drivers::redis_driver::RedisConnection),
    Kafka(drivers::kafka::KafkaConnection),
    SQLite(drivers::sqlite::SQLiteConnection),  // Add this
    Unsupported(String),
}

impl DatabaseConnection {
    pub async fn new(connection_string: &str, db_type: DatabaseType) -> DatabaseResult<Self> {
        match db_type {
            // ... existing cases ...

            #[cfg(feature = "sqlite-support")]
            DatabaseType::SQLite => {
                let conn = drivers::sqlite::SQLiteConnection::connect(connection_string).await?;
                Ok(DatabaseConnection::SQLite(conn))
            }

            _ => Ok(DatabaseConnection::Unsupported(format!("{:?}", db_type))),
        }
    }

    // Update execute, execute_command, close, is_connected methods
    // to include SQLite cases
}
```

#### Step 5: Update Driver Module

**File**: `xdl-database/src/drivers/mod.rs`

```rust
#[cfg(feature = "sqlite-support")]
pub mod sqlite;
```

#### Step 6: Update Error Types

**File**: `xdl-database/src/error.rs`

```rust
#[derive(Error, Debug)]
pub enum DatabaseError {
    // ... existing variants ...

    #[cfg(feature = "sqlite-support")]
    #[error("SQLite error: {0}")]
    SQLiteError(String),
}
```

---

## Connection String Patterns

### Existing Patterns

- **PostgreSQL**: `postgresql://user:password@localhost:5432/dbname` or `postgres://...`
- **MySQL**: `mysql://user:password@host:3306/database`
- **DuckDB**: `duckdb:///path/to/db.duckdb` or any `.duckdb`/`.db` file
- **Redis**: `redis://localhost:6379`
- **Kafka**: `kafka://broker1:9092,broker2:9092`
- **ODBC**: `DRIVER={PostgreSQL};SERVER=localhost`

### Proposed SQLite Patterns

- **SQLite**: `sqlite:///path/to/database.sqlite`
- **SQLite (relative)**: `sqlite://./local.db`
- **SQLite (in-memory)**: `sqlite://:memory:`
- **File detection**: Automatically detect `.sqlite` or `.db` files as SQLite

---

## Usage Example (Once SQLite is Implemented)

```xdl
; Create a database object
objdb = OBJ_NEW('XDLdbDatabase')

; Connect to SQLite (file-based)
objdb->Connect, CONNECTION='sqlite:///path/to/mydata.sqlite'

; Create a table
objdb->ExecuteCommand, 'CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)'

; Insert data
objdb->ExecuteCommand, "INSERT INTO users (name, age) VALUES ('Alice', 30)"
objdb->ExecuteCommand, "INSERT INTO users (name, age) VALUES ('Bob', 25)"

; Query data
recordset = objdb->ExecuteSQL('SELECT * FROM users WHERE age > 25')
data = recordset->GetData()
PRINT, data

; Cleanup
recordset->Destroy()
objdb->Disconnect()
OBJ_DESTROY, objdb
```

---

## Key Files Reference

| File | Purpose |
|------|---------|
| `/xdl-database/Cargo.toml` | Dependencies and features |
| `/xdl-database/src/lib.rs` | Main API, DatabaseType detection, XDLDatabase |
| `/xdl-database/src/connection.rs` | DatabaseConnection enum, dispatcher |
| `/xdl-database/src/drivers/mod.rs` | Driver module declarations |
| `/xdl-database/src/drivers/duckdb.rs` | Reference implementation (simple, synchronous) |
| `/xdl-database/src/drivers/postgres.rs` | Reference implementation (async, type-safe) |
| `/xdl-database/src/drivers/mysql.rs` | Reference implementation (with pooling) |
| `/xdl-database/src/recordset.rs` | Result representation |
| `/xdl-database/src/error.rs` | Error types |

---

## Key Design Principles Observed

1. **Feature-gated drivers** - Each database is optional via Cargo features
2. **Consistent interface** - All drivers implement same async methods
3. **Enum-based dispatch** - DatabaseConnection enum routes to correct driver
4. **Type conversion** - All results converted to JsonValue/XdlValue
5. **Error handling** - Unified DatabaseError with driver-specific variants
6. **Global registry** - For XDL object system integration
7. **Async/await** - All operations use tokio for async execution
8. **Connection strings** - Automatic type detection from URL schemes
