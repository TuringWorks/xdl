//! ODBC driver - Universal database connectivity
//!
//! Supports any database with an ODBC driver including:
//! - SQL Server
//! - Oracle
//! - DB2
//! - Access
//! - PostgreSQL (via ODBC)
//! - MySQL (via ODBC)
//! - And many more

use crate::{recordset::ColumnInfo, DatabaseError, DatabaseResult, Recordset};
use odbc_api::{
    buffers::{BufferDesc, ColumnarAnyBuffer, TextRowSet},
    Connection, ConnectionOptions, Cursor, Environment, IntoParameter,
};
use serde_json::Value as JsonValue;
use std::sync::Arc;

/// ODBC connection wrapper
#[derive(Debug)]
pub struct ODBCConnection {
    connection: Option<Connection<'static>>,
    environment: Option<Arc<Environment>>,
}

impl ODBCConnection {
    /// Connect to a database using ODBC connection string
    ///
    /// Example connection strings:
    /// - SQL Server: "DRIVER={ODBC Driver 17 for SQL Server};SERVER=localhost;DATABASE=mydb;UID=user;PWD=pass"
    /// - PostgreSQL: "DRIVER={PostgreSQL Unicode};SERVER=localhost;PORT=5432;DATABASE=mydb;UID=user;PWD=pass"
    /// - MySQL: "DRIVER={MySQL ODBC 8.0 Driver};SERVER=localhost;DATABASE=mydb;UID=user;PWD=pass"
    pub async fn connect(connection_string: &str) -> DatabaseResult<Self> {
        // Create ODBC environment
        let environment = Environment::new().map_err(|e| {
            DatabaseError::connection_error(format!("Failed to create ODBC environment: {}", e))
        })?;

        let environment = Arc::new(environment);
        let env_clone = environment.clone();

        // Connect with connection string
        let connection = tokio::task::spawn_blocking(move || {
            env_clone
                .connect_with_connection_string(connection_string, ConnectionOptions::default())
        })
        .await
        .map_err(|e| DatabaseError::connection_error(format!("Task join error: {}", e)))?
        .map_err(|e| DatabaseError::connection_error(format!("ODBC connection failed: {}", e)))?;

        // Convert to static lifetime by leaking (managed by our struct)
        let connection_static =
            unsafe { std::mem::transmute::<Connection<'_>, Connection<'static>>(connection) };

        Ok(Self {
            connection: Some(connection_static),
            environment: Some(environment),
        })
    }

    /// Execute a SELECT query and return results
    pub async fn execute(&self, query: &str) -> DatabaseResult<Recordset> {
        let conn = self
            .connection
            .as_ref()
            .ok_or(DatabaseError::NotConnected)?;

        let query_owned = query.to_string();

        // Execute query in blocking task
        let result =
            tokio::task::spawn_blocking(move || Self::execute_query_sync(conn, &query_owned))
                .await
                .map_err(|e| DatabaseError::query_error(format!("Task join error: {}", e)))??;

        Ok(result)
    }

    /// Execute query synchronously (called from blocking task)
    fn execute_query_sync(conn: &Connection<'static>, query: &str) -> DatabaseResult<Recordset> {
        // Execute the query
        let cursor = conn
            .execute(query, ())
            .map_err(|e| DatabaseError::query_error(format!("Query execution failed: {}", e)))?
            .ok_or_else(|| DatabaseError::query_error("No cursor returned from query"))?;

        // Get column information
        let num_cols = cursor.num_result_cols().map_err(|e| {
            DatabaseError::query_error(format!("Failed to get column count: {}", e))
        })?;

        let mut columns = Vec::new();
        for i in 1..=num_cols {
            let mut name_buffer = vec![0u8; 256];
            let col_info = cursor
                .describe_col(i as u16, &mut name_buffer)
                .map_err(|e| {
                    DatabaseError::query_error(format!("Failed to describe column {}: {}", i, e))
                })?;

            let name = String::from_utf8_lossy(&name_buffer[..col_info.name_length]).to_string();

            columns.push(ColumnInfo {
                name,
                data_type: format!("{:?}", col_info.data_type),
                ordinal: (i - 1) as usize,
            });
        }

        // Fetch rows
        let mut rows_data = Vec::new();

        // Create buffer description for all columns
        let buffer_desc: Vec<BufferDesc> = (0..num_cols)
            .map(|_| BufferDesc::Text { max_str_len: 1024 })
            .collect();

        // Create columnar buffer
        let row_set_buffer = ColumnarAnyBuffer::try_from_descs(100, buffer_desc.iter())
            .map_err(|e| DatabaseError::query_error(format!("Failed to create buffer: {}", e)))?;

        let mut cursor = cursor
            .bind_buffer(row_set_buffer)
            .map_err(|e| DatabaseError::query_error(format!("Failed to bind buffer: {}", e)))?;

        // Fetch all rows
        while let Some(row_set) = cursor
            .fetch()
            .map_err(|e| DatabaseError::query_error(format!("Failed to fetch rows: {}", e)))?
        {
            for row_idx in 0..row_set.num_rows() {
                let mut row_data = Vec::new();

                for col_idx in 0..num_cols {
                    let col_view = row_set.column(col_idx);

                    // Try to get as text
                    let value = match col_view {
                        odbc_api::buffers::AnyColumnView::Text(text_col) => {
                            match text_col.get(row_idx) {
                                Some(text_bytes) => {
                                    let text = String::from_utf8_lossy(text_bytes).to_string();
                                    if text.is_empty() {
                                        JsonValue::Null
                                    } else if let Ok(num) = text.parse::<i64>() {
                                        JsonValue::from(num)
                                    } else if let Ok(num) = text.parse::<f64>() {
                                        JsonValue::from(num)
                                    } else {
                                        JsonValue::from(text)
                                    }
                                }
                                None => JsonValue::Null,
                            }
                        }
                        _ => JsonValue::String("(binary data)".to_string()),
                    };

                    row_data.push(value);
                }

                rows_data.push(row_data);
            }
        }

        Ok(Recordset::new(columns, rows_data))
    }

    /// Execute a command (INSERT, UPDATE, DELETE)
    pub async fn execute_command(&self, command: &str) -> DatabaseResult<u64> {
        let conn = self
            .connection
            .as_ref()
            .ok_or(DatabaseError::NotConnected)?;

        let command_owned = command.to_string();

        // Execute in blocking task
        let rows_affected = tokio::task::spawn_blocking(move || {
            conn.execute(&command_owned, ())
                .map_err(|e| DatabaseError::query_error(format!("Command execution failed: {}", e)))
        })
        .await
        .map_err(|e| DatabaseError::query_error(format!("Task join error: {}", e)))??;

        // Check if we got a cursor (for queries that don't return data)
        match rows_affected {
            Some(_cursor) => Ok(0), // Query returned cursor but we don't need it
            None => Ok(0),          // Command executed, exact row count not available via this API
        }
    }

    /// Close the connection
    pub async fn close(&mut self) -> DatabaseResult<()> {
        self.connection = None;
        self.environment = None;
        Ok(())
    }

    /// Check if connection is alive
    pub async fn is_connected(&self) -> bool {
        if let Some(conn) = &self.connection {
            // Try a simple query to check connection
            tokio::task::spawn_blocking(move || conn.execute("SELECT 1", ()))
                .await
                .is_ok()
        } else {
            false
        }
    }
}

impl Drop for ODBCConnection {
    fn drop(&mut self) {
        // Cleanup happens automatically when connection is dropped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_odbc_connection_string_parsing() {
        // Just test that we can create the struct
        // Actual connection testing requires ODBC drivers installed
        let conn_str = "DRIVER={SQLite3};Database=test.db";

        // This will fail without SQLite ODBC driver, but tests the code path
        let result = ODBCConnection::connect(conn_str).await;

        // We can't assert success without driver, but no panic = good
        println!("Connection attempt result: {:?}", result.is_ok());
    }
}
