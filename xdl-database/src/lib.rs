//! XDL Database Connectivity Module
//!
//! Provides unified database access for XDL programs supporting:
//! - PostgreSQL
//! - MySQL
//! - DuckDB
//! - SQLite
//! - ODBC (generic)
//! - Redis
//! - Apache Kafka (streaming)
//!
//! # Example
//! ```xdl
//! ; Create a database object
//! objdb = OBJ_NEW('XDLdbDatabase')
//!
//! ; Connect to PostgreSQL
//! conn_str = 'postgresql://user:password@localhost:5432/dbname'
//! objdb->Connect, CONNECTION=conn_str
//!
//! ; Execute a query
//! recordset = objdb->ExecuteSQL('SELECT * FROM my_table')
//!
//! ; Get data
//! data = recordset->GetData()
//!
//! ; Cleanup
//! recordset->Destroy()
//! objdb->Disconnect()
//! OBJ_DESTROY, objdb
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use xdl_core::{XdlError, XdlResult};

pub mod connection;
pub mod drivers;
pub mod error;
pub mod recordset;

pub use connection::DatabaseConnection;
pub use error::{DatabaseError, DatabaseResult};
pub use recordset::Recordset;

/// Database connection type
#[derive(Debug, Clone, PartialEq)]
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
    DuckDB,
    SQLite,
    ODBC,
    Redis,
    Kafka,
    Unknown,
}

impl DatabaseType {
    /// Parse database type from connection string
    pub fn from_connection_string(conn_str: &str) -> Self {
        let lower = conn_str.to_lowercase();

        if lower.starts_with("postgresql://") || lower.starts_with("postgres://") {
            DatabaseType::PostgreSQL
        } else if lower.starts_with("mysql://") {
            DatabaseType::MySQL
        } else if lower.starts_with("sqlite://")
            || lower.starts_with("sqlite:")
            || lower.contains(".sqlite")
            || lower.contains(":memory:")
        {
            DatabaseType::SQLite
        } else if lower.starts_with("duckdb://")
            || lower.contains(".duckdb")
            || lower.contains(".db")
        {
            DatabaseType::DuckDB
        } else if lower.starts_with("redis://") {
            DatabaseType::Redis
        } else if lower.starts_with("kafka://") {
            DatabaseType::Kafka
        } else if lower.starts_with("driver={") || lower.contains("driver=") {
            DatabaseType::ODBC
        } else {
            DatabaseType::Unknown
        }
    }
}

/// Main database object for XDL
#[derive(Debug)]
pub struct XDLDatabase {
    connection: Option<Arc<RwLock<DatabaseConnection>>>,
    db_type: Option<DatabaseType>,
    connection_string: Option<String>,
    last_error: Option<String>,
}

impl XDLDatabase {
    /// Create a new database object
    pub fn new() -> Self {
        Self {
            connection: None,
            db_type: None,
            connection_string: None,
            last_error: None,
        }
    }

    /// Connect to a database using a connection string
    pub async fn connect(&mut self, connection_string: &str) -> XdlResult<()> {
        // Determine database type from connection string
        let db_type = DatabaseType::from_connection_string(connection_string);

        if db_type == DatabaseType::Unknown {
            return Err(XdlError::RuntimeError(format!(
                "Unable to determine database type from connection string: {}",
                connection_string
            )));
        }

        // Create connection based on type
        let conn = DatabaseConnection::new(connection_string, db_type.clone())
            .await
            .map_err(|e| XdlError::RuntimeError(format!("Connection failed: {}", e)))?;

        self.connection = Some(Arc::new(RwLock::new(conn)));
        self.db_type = Some(db_type);
        self.connection_string = Some(connection_string.to_string());
        self.last_error = None;

        Ok(())
    }

    /// Disconnect from the database
    pub async fn disconnect(&mut self) -> XdlResult<()> {
        if let Some(conn) = &self.connection {
            let mut connection = conn.write().await;
            connection
                .close()
                .await
                .map_err(|e| XdlError::RuntimeError(format!("Disconnect failed: {}", e)))?;
        }

        self.connection = None;
        self.db_type = None;

        Ok(())
    }

    /// Execute a SQL query and return a recordset
    pub async fn execute_sql(&self, query: &str) -> XdlResult<Recordset> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| XdlError::RuntimeError("Not connected to database".to_string()))?;

        let connection = conn.read().await;
        let recordset = connection
            .execute(query)
            .await
            .map_err(|e| XdlError::RuntimeError(format!("Query execution failed: {}", e)))?;

        Ok(recordset)
    }

    /// Execute a SQL command (INSERT, UPDATE, DELETE) without returning data
    pub async fn execute_command(&self, command: &str) -> XdlResult<u64> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| XdlError::RuntimeError("Not connected to database".to_string()))?;

        let connection = conn.read().await;
        let rows_affected = connection
            .execute_command(command)
            .await
            .map_err(|e| XdlError::RuntimeError(format!("Command execution failed: {}", e)))?;

        Ok(rows_affected)
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    /// Get the database type
    pub fn database_type(&self) -> Option<&DatabaseType> {
        self.db_type.as_ref()
    }

    /// Get last error message
    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }
}

impl Default for XDLDatabase {
    fn default() -> Self {
        Self::new()
    }
}

/// Global database object registry for XDL object system integration
/// Maps object IDs to database instances
pub struct DatabaseRegistry {
    databases: RwLock<HashMap<usize, Arc<RwLock<XDLDatabase>>>>,
    recordsets: RwLock<HashMap<usize, Arc<RwLock<Recordset>>>>,
    next_id: RwLock<usize>,
}

impl DatabaseRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            databases: RwLock::new(HashMap::new()),
            recordsets: RwLock::new(HashMap::new()),
            next_id: RwLock::new(1),
        }
    }

    /// Register a new database object and return its ID
    pub async fn register_database(&self, db: XDLDatabase) -> usize {
        let mut next_id = self.next_id.write().await;
        let id = *next_id;
        *next_id += 1;

        let mut databases = self.databases.write().await;
        databases.insert(id, Arc::new(RwLock::new(db)));

        id
    }

    /// Get a database by ID
    pub async fn get_database(&self, id: usize) -> Option<Arc<RwLock<XDLDatabase>>> {
        let databases = self.databases.read().await;
        databases.get(&id).cloned()
    }

    /// Remove a database from the registry
    pub async fn unregister_database(&self, id: usize) {
        let mut databases = self.databases.write().await;
        databases.remove(&id);
    }

    /// Register a recordset and return its ID
    pub async fn register_recordset(&self, recordset: Recordset) -> usize {
        let mut next_id = self.next_id.write().await;
        let id = *next_id;
        *next_id += 1;

        let mut recordsets = self.recordsets.write().await;
        recordsets.insert(id, Arc::new(RwLock::new(recordset)));

        id
    }

    /// Get a recordset by ID
    pub async fn get_recordset(&self, id: usize) -> Option<Arc<RwLock<Recordset>>> {
        let recordsets = self.recordsets.read().await;
        recordsets.get(&id).cloned()
    }

    /// Remove a recordset from the registry
    pub async fn unregister_recordset(&self, id: usize) {
        let mut recordsets = self.recordsets.write().await;
        recordsets.remove(&id);
    }
}

impl Default for DatabaseRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Global registry instance
lazy_static::lazy_static! {
    pub static ref GLOBAL_DB_REGISTRY: DatabaseRegistry = DatabaseRegistry::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_type_detection() {
        assert_eq!(
            DatabaseType::from_connection_string("postgresql://localhost/db"),
            DatabaseType::PostgreSQL
        );
        assert_eq!(
            DatabaseType::from_connection_string("mysql://localhost/db"),
            DatabaseType::MySQL
        );
        assert_eq!(
            DatabaseType::from_connection_string("test.duckdb"),
            DatabaseType::DuckDB
        );
        assert_eq!(
            DatabaseType::from_connection_string("redis://localhost:6379"),
            DatabaseType::Redis
        );
        assert_eq!(
            DatabaseType::from_connection_string("DRIVER={PostgreSQL};SERVER=localhost"),
            DatabaseType::ODBC
        );
    }
}
