//! DuckDB database driver

use crate::{recordset::ColumnInfo, DatabaseError, DatabaseResult, Recordset};
use duckdb::{params, Connection};
use serde_json::Value as JsonValue;
use std::sync::Mutex;

/// DuckDB connection
pub struct DuckDBConnection {
    conn: Option<Mutex<Connection>>,
}

impl std::fmt::Debug for DuckDBConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DuckDBConnection")
            .field("conn", &self.conn.is_some())
            .finish()
    }
}

impl DuckDBConnection {
    /// Connect to a DuckDB database
    pub async fn connect(connection_string: &str) -> DatabaseResult<Self> {
        // Extract file path from connection string
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

    /// Execute a command
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
        self.conn = None;
        Ok(())
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        self.conn.is_some()
    }
}
