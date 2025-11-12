//! PostgreSQL database driver

use crate::{recordset::ColumnInfo, DatabaseError, DatabaseResult, Recordset};
use serde_json::Value as JsonValue;
use tokio_postgres::{Client, Config, NoTls};

/// PostgreSQL connection
#[derive(Debug)]
pub struct PostgresConnection {
    client: Option<Client>,
}

impl PostgresConnection {
    /// Connect to a PostgreSQL database
    pub async fn connect(connection_string: &str) -> DatabaseResult<Self> {
        // Parse connection string
        let config: Config = connection_string.parse().map_err(|e| {
            DatabaseError::connection_error(format!("Invalid connection string: {}", e))
        })?;

        // Connect
        let (client, connection) = config.connect(NoTls).await?;

        // Spawn connection task
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("PostgreSQL connection error: {}", e);
            }
        });

        Ok(Self {
            client: Some(client),
        })
    }

    /// Execute a SELECT query
    pub async fn execute(&self, query: &str) -> DatabaseResult<Recordset> {
        let client = self.client.as_ref().ok_or(DatabaseError::NotConnected)?;

        // Execute query
        let rows = client.query(query, &[]).await?;

        if rows.is_empty() {
            return Ok(Recordset::empty());
        }

        // Extract column information
        let columns: Vec<ColumnInfo> = rows[0]
            .columns()
            .iter()
            .enumerate()
            .map(|(i, col)| ColumnInfo {
                name: col.name().to_string(),
                data_type: format!("{:?}", col.type_()),
                ordinal: i,
            })
            .collect();

        // Extract row data
        let mut data_rows = Vec::new();
        for row in rows {
            let mut row_data = Vec::new();

            for i in 0..row.len() {
                let value = postgres_value_to_json(&row, i)?;
                row_data.push(value);
            }

            data_rows.push(row_data);
        }

        Ok(Recordset::new(columns, data_rows))
    }

    /// Execute a command (INSERT, UPDATE, DELETE)
    pub async fn execute_command(&self, command: &str) -> DatabaseResult<u64> {
        let client = self.client.as_ref().ok_or(DatabaseError::NotConnected)?;

        let rows_affected = client.execute(command, &[]).await?;
        Ok(rows_affected)
    }

    /// Close the connection
    pub async fn close(&mut self) -> DatabaseResult<()> {
        self.client = None;
        Ok(())
    }

    /// Check if connection is alive
    pub async fn is_connected(&self) -> bool {
        if let Some(client) = &self.client {
            // Try a simple query
            client.query("SELECT 1", &[]).await.is_ok()
        } else {
            false
        }
    }
}

/// Convert PostgreSQL value to JSON
fn postgres_value_to_json(row: &tokio_postgres::Row, idx: usize) -> DatabaseResult<JsonValue> {
    use tokio_postgres::types::Type;

    let col_type = row.columns()[idx].type_();

    let value = match *col_type {
        Type::BOOL => {
            let v: Option<bool> = row
                .try_get(idx)
                .map_err(|e| DatabaseError::conversion_error(format!("Bool conversion: {}", e)))?;
            v.map(JsonValue::from).unwrap_or(JsonValue::Null)
        }
        Type::INT2 => {
            let v: Option<i16> = row
                .try_get(idx)
                .map_err(|e| DatabaseError::conversion_error(format!("Int2 conversion: {}", e)))?;
            v.map(JsonValue::from).unwrap_or(JsonValue::Null)
        }
        Type::INT4 => {
            let v: Option<i32> = row
                .try_get(idx)
                .map_err(|e| DatabaseError::conversion_error(format!("Int4 conversion: {}", e)))?;
            v.map(JsonValue::from).unwrap_or(JsonValue::Null)
        }
        Type::INT8 => {
            let v: Option<i64> = row
                .try_get(idx)
                .map_err(|e| DatabaseError::conversion_error(format!("Int8 conversion: {}", e)))?;
            v.map(JsonValue::from).unwrap_or(JsonValue::Null)
        }
        Type::FLOAT4 => {
            let v: Option<f32> = row.try_get(idx).map_err(|e| {
                DatabaseError::conversion_error(format!("Float4 conversion: {}", e))
            })?;
            v.map(|f| JsonValue::from(f as f64))
                .unwrap_or(JsonValue::Null)
        }
        Type::FLOAT8 => {
            let v: Option<f64> = row.try_get(idx).map_err(|e| {
                DatabaseError::conversion_error(format!("Float8 conversion: {}", e))
            })?;
            v.map(JsonValue::from).unwrap_or(JsonValue::Null)
        }
        Type::TEXT | Type::VARCHAR => {
            let v: Option<String> = row
                .try_get(idx)
                .map_err(|e| DatabaseError::conversion_error(format!("Text conversion: {}", e)))?;
            v.map(JsonValue::from).unwrap_or(JsonValue::Null)
        }
        _ => {
            // Try to get as string for other types
            let v: Option<String> = row.try_get(idx).ok();
            v.map(JsonValue::from).unwrap_or(JsonValue::Null)
        }
    };

    Ok(value)
}
