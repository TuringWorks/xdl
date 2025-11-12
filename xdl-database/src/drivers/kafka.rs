//! Apache Kafka driver - Streaming data platform
//!
//! Supports Kafka operations including:
//! - Producing messages to topics
//! - Consuming messages from topics
//! - Topic management
//!
//! Special query syntax:
//! - PRODUCE TO topic: message_content
//! - CONSUME FROM topic [LIMIT n] [OFFSET group_id]
//! - CREATE TOPIC topic
//! - LIST TOPICS
//! - DELETE TOPIC topic

use crate::{recordset::ColumnInfo, DatabaseError, DatabaseResult, Recordset};
use rdkafka::{
    admin::{AdminClient, AdminOptions, NewTopic, TopicReplication},
    client::DefaultClientContext,
    config::ClientConfig,
    consumer::{BaseConsumer, Consumer},
    message::{BorrowedMessage, Message},
    producer::{FutureProducer, FutureRecord},
    util::Timeout,
};
use serde_json::Value as JsonValue;
use std::time::Duration;

/// Kafka connection wrapper
#[derive(Debug)]
pub struct KafkaConnection {
    brokers: String,
    producer: Option<FutureProducer>,
    consumer: Option<BaseConsumer>,
    admin: Option<AdminClient<DefaultClientContext>>,
}

impl KafkaConnection {
    /// Connect to Kafka cluster
    ///
    /// Connection string format: kafka://broker1:9092,broker2:9092
    /// Or just: localhost:9092
    pub async fn connect(connection_string: &str) -> DatabaseResult<Self> {
        // Parse broker list from connection string
        let brokers = connection_string
            .trim_start_matches("kafka://")
            .trim_start_matches("kafka:")
            .to_string();

        // Create producer
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &brokers)
            .set("message.timeout.ms", "5000")
            .create()
            .map_err(|e| {
                DatabaseError::connection_error(format!("Failed to create Kafka producer: {}", e))
            })?;

        // Create consumer
        let consumer: BaseConsumer = ClientConfig::new()
            .set("bootstrap.servers", &brokers)
            .set("group.id", "xdl-database-consumer")
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .create()
            .map_err(|e| {
                DatabaseError::connection_error(format!("Failed to create Kafka consumer: {}", e))
            })?;

        // Create admin client
        let admin: AdminClient<DefaultClientContext> = ClientConfig::new()
            .set("bootstrap.servers", &brokers)
            .create()
            .map_err(|e| {
                DatabaseError::connection_error(format!(
                    "Failed to create Kafka admin client: {}",
                    e
                ))
            })?;

        Ok(Self {
            brokers,
            producer: Some(producer),
            consumer: Some(consumer),
            admin: Some(admin),
        })
    }

    /// Execute a Kafka "query" (special syntax)
    ///
    /// Supported commands:
    /// - PRODUCE TO topic: message
    /// - CONSUME FROM topic LIMIT n
    /// - CREATE TOPIC topic
    /// - LIST TOPICS
    /// - DELETE TOPIC topic
    pub async fn execute(&self, query: &str) -> DatabaseResult<Recordset> {
        let query = query.trim();
        let upper_query = query.to_uppercase();

        if upper_query.starts_with("PRODUCE TO") {
            self.handle_produce(query).await
        } else if upper_query.starts_with("CONSUME FROM") {
            self.handle_consume(query).await
        } else if upper_query.starts_with("LIST TOPICS") {
            self.handle_list_topics().await
        } else if upper_query.starts_with("CREATE TOPIC") {
            self.handle_create_topic(query).await
        } else if upper_query.starts_with("DELETE TOPIC") {
            self.handle_delete_topic(query).await
        } else {
            Err(DatabaseError::query_error(format!(
                "Unsupported Kafka command. Use: PRODUCE TO, CONSUME FROM, LIST TOPICS, CREATE TOPIC, or DELETE TOPIC"
            )))
        }
    }

    /// Handle PRODUCE TO topic: message
    async fn handle_produce(&self, query: &str) -> DatabaseResult<Recordset> {
        let producer = self.producer.as_ref().ok_or(DatabaseError::NotConnected)?;

        // Parse: PRODUCE TO topic: message
        let parts: Vec<&str> = query.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(DatabaseError::query_error(
                "PRODUCE syntax: PRODUCE TO topic: message",
            ));
        }

        let topic_part = parts[0].trim();
        let message = parts[1].trim();

        // Extract topic name
        let topic = topic_part
            .trim_start_matches("PRODUCE TO")
            .trim_start_matches("produce to")
            .trim();

        // Send message
        let record = FutureRecord::to(topic).payload(message).key("xdl-key");

        let delivery_status = producer
            .send(record, Timeout::After(Duration::from_secs(5)))
            .await
            .map_err(|(err, _)| {
                DatabaseError::query_error(format!("Failed to produce message: {}", err))
            })?;

        // Return success info
        let columns = vec![
            ColumnInfo {
                name: "status".to_string(),
                data_type: "text".to_string(),
                ordinal: 0,
            },
            ColumnInfo {
                name: "partition".to_string(),
                data_type: "integer".to_string(),
                ordinal: 1,
            },
            ColumnInfo {
                name: "offset".to_string(),
                data_type: "integer".to_string(),
                ordinal: 2,
            },
        ];

        let rows = vec![vec![
            JsonValue::from("success"),
            JsonValue::from(delivery_status.0),
            JsonValue::from(delivery_status.1),
        ]];

        Ok(Recordset::new(columns, rows))
    }

    /// Handle CONSUME FROM topic [LIMIT n]
    async fn handle_consume(&self, query: &str) -> DatabaseResult<Recordset> {
        let consumer = self.consumer.as_ref().ok_or(DatabaseError::NotConnected)?;

        // Parse: CONSUME FROM topic [LIMIT n]
        let parts: Vec<&str> = query.split_whitespace().collect();
        if parts.len() < 3 {
            return Err(DatabaseError::query_error(
                "CONSUME syntax: CONSUME FROM topic [LIMIT n]",
            ));
        }

        let topic = parts[2];
        let limit = if parts.len() >= 5 && parts[3].to_uppercase() == "LIMIT" {
            parts[4].parse::<usize>().unwrap_or(10)
        } else {
            10
        };

        // Subscribe to topic
        consumer.subscribe(&[topic]).map_err(|e| {
            DatabaseError::query_error(format!("Failed to subscribe to topic: {}", e))
        })?;

        // Collect messages
        let mut messages = Vec::new();

        for _ in 0..limit {
            match consumer.poll(Timeout::After(Duration::from_millis(1000))) {
                Some(Ok(msg)) => {
                    messages.push(Self::message_to_row(&msg)?);
                }
                Some(Err(e)) => {
                    eprintln!("Kafka consumer error: {}", e);
                    break;
                }
                None => {
                    // No more messages within timeout
                    break;
                }
            }
        }

        // Define columns
        let columns = vec![
            ColumnInfo {
                name: "partition".to_string(),
                data_type: "integer".to_string(),
                ordinal: 0,
            },
            ColumnInfo {
                name: "offset".to_string(),
                data_type: "integer".to_string(),
                ordinal: 1,
            },
            ColumnInfo {
                name: "key".to_string(),
                data_type: "text".to_string(),
                ordinal: 2,
            },
            ColumnInfo {
                name: "payload".to_string(),
                data_type: "text".to_string(),
                ordinal: 3,
            },
            ColumnInfo {
                name: "timestamp".to_string(),
                data_type: "integer".to_string(),
                ordinal: 4,
            },
        ];

        Ok(Recordset::new(columns, messages))
    }

    /// Convert Kafka message to row data
    fn message_to_row(msg: &BorrowedMessage) -> DatabaseResult<Vec<JsonValue>> {
        let partition = JsonValue::from(msg.partition());
        let offset = JsonValue::from(msg.offset());

        let key = match msg.key() {
            Some(k) => JsonValue::from(String::from_utf8_lossy(k).to_string()),
            None => JsonValue::Null,
        };

        let payload = match msg.payload() {
            Some(p) => JsonValue::from(String::from_utf8_lossy(p).to_string()),
            None => JsonValue::Null,
        };

        let timestamp = match msg.timestamp().to_millis() {
            Some(ts) => JsonValue::from(ts),
            None => JsonValue::Null,
        };

        Ok(vec![partition, offset, key, payload, timestamp])
    }

    /// Handle LIST TOPICS
    async fn handle_list_topics(&self) -> DatabaseResult<Recordset> {
        let consumer = self.consumer.as_ref().ok_or(DatabaseError::NotConnected)?;

        let metadata = consumer
            .fetch_metadata(None, Timeout::After(Duration::from_secs(5)))
            .map_err(|e| DatabaseError::query_error(format!("Failed to fetch metadata: {}", e)))?;

        let columns = vec![
            ColumnInfo {
                name: "topic".to_string(),
                data_type: "text".to_string(),
                ordinal: 0,
            },
            ColumnInfo {
                name: "partitions".to_string(),
                data_type: "integer".to_string(),
                ordinal: 1,
            },
        ];

        let mut rows = Vec::new();
        for topic in metadata.topics() {
            rows.push(vec![
                JsonValue::from(topic.name()),
                JsonValue::from(topic.partitions().len()),
            ]);
        }

        Ok(Recordset::new(columns, rows))
    }

    /// Handle CREATE TOPIC topic_name
    async fn handle_create_topic(&self, query: &str) -> DatabaseResult<Recordset> {
        let admin = self.admin.as_ref().ok_or(DatabaseError::NotConnected)?;

        // Parse topic name
        let parts: Vec<&str> = query.split_whitespace().collect();
        if parts.len() < 3 {
            return Err(DatabaseError::query_error(
                "CREATE TOPIC syntax: CREATE TOPIC topic_name",
            ));
        }

        let topic_name = parts[2];

        // Create topic
        let new_topic = NewTopic::new(
            topic_name,
            1,                          // num_partitions
            TopicReplication::Fixed(1), // replication_factor
        );

        let results = admin
            .create_topics(&[new_topic], &AdminOptions::new())
            .await
            .map_err(|e| DatabaseError::query_error(format!("Failed to create topic: {}", e)))?;

        // Check result
        let status = if results.is_empty() {
            "success"
        } else {
            match &results[0] {
                Ok(_) => "success",
                Err(e) => {
                    return Err(DatabaseError::query_error(format!(
                        "Topic creation failed: {}",
                        e
                    )))
                }
            }
        };

        let columns = vec![
            ColumnInfo {
                name: "status".to_string(),
                data_type: "text".to_string(),
                ordinal: 0,
            },
            ColumnInfo {
                name: "topic".to_string(),
                data_type: "text".to_string(),
                ordinal: 1,
            },
        ];

        let rows = vec![vec![JsonValue::from(status), JsonValue::from(topic_name)]];

        Ok(Recordset::new(columns, rows))
    }

    /// Handle DELETE TOPIC topic_name
    async fn handle_delete_topic(&self, query: &str) -> DatabaseResult<Recordset> {
        let admin = self.admin.as_ref().ok_or(DatabaseError::NotConnected)?;

        // Parse topic name
        let parts: Vec<&str> = query.split_whitespace().collect();
        if parts.len() < 3 {
            return Err(DatabaseError::query_error(
                "DELETE TOPIC syntax: DELETE TOPIC topic_name",
            ));
        }

        let topic_name = parts[2];

        // Delete topic
        let results = admin
            .delete_topics(&[topic_name], &AdminOptions::new())
            .await
            .map_err(|e| DatabaseError::query_error(format!("Failed to delete topic: {}", e)))?;

        // Check result
        let status = if results.is_empty() {
            "success"
        } else {
            match &results[0] {
                Ok(_) => "success",
                Err(e) => {
                    return Err(DatabaseError::query_error(format!(
                        "Topic deletion failed: {}",
                        e
                    )))
                }
            }
        };

        let columns = vec![
            ColumnInfo {
                name: "status".to_string(),
                data_type: "text".to_string(),
                ordinal: 0,
            },
            ColumnInfo {
                name: "topic".to_string(),
                data_type: "text".to_string(),
                ordinal: 1,
            },
        ];

        let rows = vec![vec![JsonValue::from(status), JsonValue::from(topic_name)]];

        Ok(Recordset::new(columns, rows))
    }

    /// Execute a command (producer operations)
    pub async fn execute_command(&self, command: &str) -> DatabaseResult<u64> {
        // For Kafka, commands are the same as queries
        let result = self.execute(command).await?;
        Ok(result.row_count() as u64)
    }

    /// Close the connection
    pub async fn close(&mut self) -> DatabaseResult<()> {
        self.producer = None;
        self.consumer = None;
        self.admin = None;
        Ok(())
    }

    /// Check if connection is alive
    pub async fn is_connected(&self) -> bool {
        self.producer.is_some() && self.consumer.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kafka_connection_string_parsing() {
        let conn_str = "kafka://localhost:9092";
        let brokers = conn_str.trim_start_matches("kafka://");
        assert_eq!(brokers, "localhost:9092");
    }

    #[test]
    fn test_query_parsing() {
        let query = "PRODUCE TO my-topic: Hello World";
        assert!(query.to_uppercase().starts_with("PRODUCE TO"));

        let query2 = "CONSUME FROM my-topic LIMIT 10";
        assert!(query2.to_uppercase().starts_with("CONSUME FROM"));
    }
}
