//! Redis driver

use crate::{DatabaseError, DatabaseResult, Recordset};
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client};

pub struct RedisConnection {
    conn: Option<ConnectionManager>,
}

impl std::fmt::Debug for RedisConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisConnection")
            .field("conn", &self.conn.is_some())
            .finish()
    }
}

impl RedisConnection {
    pub async fn connect(connection_string: &str) -> DatabaseResult<Self> {
        let client = Client::open(connection_string)?;
        let conn = ConnectionManager::new(client).await?;

        Ok(Self { conn: Some(conn) })
    }

    pub async fn execute(&self, _query: &str) -> DatabaseResult<Recordset> {
        // Redis doesn't have traditional SQL queries
        // Return empty recordset for now
        Ok(Recordset::empty())
    }

    pub async fn execute_command(&self, command: &str) -> DatabaseResult<u64> {
        let mut conn = self
            .conn
            .as_ref()
            .ok_or(DatabaseError::NotConnected)?
            .clone();

        // Parse Redis command (simplified)
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(DatabaseError::query_error("Empty command"));
        }

        // Execute command based on type
        match parts[0].to_uppercase().as_str() {
            "SET" if parts.len() >= 3 => {
                let _: () = conn.set(parts[1], parts[2]).await?;
                Ok(1)
            }
            "DEL" if parts.len() >= 2 => {
                let count: u64 = conn.del(parts[1]).await?;
                Ok(count)
            }
            _ => Ok(0),
        }
    }

    pub async fn close(&mut self) -> DatabaseResult<()> {
        self.conn = None;
        Ok(())
    }

    pub async fn is_connected(&self) -> bool {
        self.conn.is_some()
    }
}
