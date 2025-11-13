//! SQLite database driver

use crate::{recordset::ColumnInfo, DatabaseError, DatabaseResult, Recordset};
use rusqlite::{params, Connection};
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
        // Extract file path from connection string
        // Supports: sqlite:///path/to/file.db, sqlite://:memory:, or direct file paths
        let path = connection_string
            .trim_start_matches("sqlite:///")
            .trim_start_matches("sqlite://")
            .trim_start_matches("sqlite:");

        let conn = Connection::open(path).map_err(|e| {
            DatabaseError::connection_error(format!("SQLite connection failed: {}", e))
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
            .map(|i| {
                let name = stmt.column_name(i).unwrap_or("unknown").to_string();
                ColumnInfo {
                    name,
                    data_type: "unknown".to_string(),
                    ordinal: i,
                }
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
                use rusqlite::types::ValueRef;

                let val = row
                    .get_ref(i)
                    .map_err(|e| DatabaseError::query_error(format!("Get column failed: {}", e)))?;

                let json_val = match val {
                    ValueRef::Null => JsonValue::Null,
                    ValueRef::Integer(i) => JsonValue::Number(i.into()),
                    ValueRef::Real(f) => {
                        if let Some(num) = serde_json::Number::from_f64(f) {
                            JsonValue::Number(num)
                        } else {
                            JsonValue::Null
                        }
                    }
                    ValueRef::Text(s) => {
                        let text = std::str::from_utf8(s).map_err(|e| {
                            DatabaseError::query_error(format!("UTF-8 decode failed: {}", e))
                        })?;
                        JsonValue::String(text.to_string())
                    }
                    ValueRef::Blob(b) => {
                        // Encode binary data as base64
                        JsonValue::String(base64_encode(b))
                    }
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

/// Simple base64 encoding for binary data
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();

    for chunk in data.chunks(3) {
        let mut buf = [0u8; 3];
        for (i, &byte) in chunk.iter().enumerate() {
            buf[i] = byte;
        }

        result.push(CHARS[(buf[0] >> 2) as usize] as char);
        result.push(CHARS[(((buf[0] & 0x03) << 4) | (buf[1] >> 4)) as usize] as char);

        if chunk.len() > 1 {
            result.push(CHARS[(((buf[1] & 0x0f) << 2) | (buf[2] >> 6)) as usize] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(CHARS[(buf[2] & 0x3f) as usize] as char);
        } else {
            result.push('=');
        }
    }

    result
}
