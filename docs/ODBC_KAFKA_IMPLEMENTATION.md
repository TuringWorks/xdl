# ODBC and Kafka Drivers - Implementation Summary

## Overview

Completed full implementations of ODBC and Apache Kafka drivers for the XDL database connectivity module, bringing the total supported databases to 5 fully functional drivers.

## ODBC Driver Implementation

### ODBC Status: ✅ Fully Implemented

### ODBC Technology Stack

- **Crate**: `odbc-api` v8.0
- **Features**: Universal database connectivity via ODBC standard
- **Async Support**: Using `tokio::task::spawn_blocking` for ODBC operations

### Supported Databases (via ODBC)

ODBC provides universal connectivity to virtually any database with an ODBC driver:

1. **SQL Server** - Microsoft SQL Server (all versions)
2. **Oracle** - Oracle Database
3. **IBM DB2** - IBM DB2 databases
4. **MySQL** - MySQL (via MySQL ODBC Driver)
5. **PostgreSQL** - PostgreSQL (via PostgreSQL ODBC Driver)
6. **Microsoft Access** - Access databases
7. **SQLite** - SQLite databases
8. **Sybase** - Sybase ASE
9. **Informix** - IBM Informix
10. **Many others** - Any database with ODBC driver support

### Implementation Highlights

#### Connection Management

```rust
pub async fn connect(connection_string: &str) -> DatabaseResult<Self> {
    // Create ODBC environment
    let environment = Environment::new()?;

    // Connect with connection string in blocking task
    let connection = tokio::task::spawn_blocking(move || {
        env.connect_with_connection_string(
            connection_string,
            ConnectionOptions::default()
        )
    }).await??;

    Ok(Self { connection, environment })
}
```

**Key Features:**

- Async wrapper around blocking ODBC calls
- Automatic environment management
- Support for any ODBC connection string format

#### Query Execution

```rust
pub async fn execute(&self, query: &str) -> DatabaseResult<Recordset> {
    tokio::task::spawn_blocking(move || {
        // Execute query synchronously
        let cursor = conn.execute(query, ())?;

        // Get column metadata
        let columns = extract_column_info(&cursor)?;

        // Fetch all rows using columnar buffer
        let rows = fetch_rows(&cursor)?;

        Ok(Recordset::new(columns, rows))
    }).await?
}
```

**Key Features:**

- Columnar buffer for efficient bulk fetching
- Automatic type detection and conversion
- Intelligent text-to-numeric conversion
- NULL handling

#### Type Conversion Strategy

ODBC types are converted to JSON intermediate format:

| ODBC Type | Conversion Strategy |
|-----------|---------------------|
| CHAR/VARCHAR/TEXT | String → Try parse as number → JSON |
| INTEGER/SMALLINT | Direct to JsonValue::Number |
| BIGINT | Direct to JsonValue::Number |
| REAL/FLOAT/DOUBLE | Direct to JsonValue::Number |
| BOOLEAN | To JsonValue::Bool |
| NULL | JsonValue::Null |
| BINARY | String "(binary data)" |

**Smart Conversion:** Text columns are parsed to detect numeric values:

```rust
if let Ok(num) = text.parse::<i64>() {
    JsonValue::from(num)
} else if let Ok(num) = text.parse::<f64>() {
    JsonValue::from(num)
} else {
    JsonValue::from(text)
}
```

### Connection String Examples

#### SQL Server

```text
DRIVER={ODBC Driver 17 for SQL Server};SERVER=localhost;DATABASE=mydb;UID=sa;PWD=pass;TrustServerCertificate=yes
```

#### PostgreSQL

```text
DRIVER={PostgreSQL Unicode};SERVER=localhost;PORT=5432;DATABASE=mydb;UID=user;PWD=pass
```

#### MySQL

```text
DRIVER={MySQL ODBC 8.0 Driver};SERVER=localhost;DATABASE=mydb;UID=user;PWD=pass
```

#### Oracle

```text
DRIVER={Oracle in OraClient19Home1};DBQ=localhost:1521/ORCL;UID=user;PWD=pass
```

#### SQLite

```text
DRIVER={SQLite3 ODBC Driver};Database=/path/to/database.db
```

### Kafka XDL Usage Example

```xdl
; Create database object
objdb = OBJ_NEW('XDLdbDatabase')

; Connect to SQL Server
conn_str = 'DRIVER={ODBC Driver 17 for SQL Server};' + $
           'SERVER=localhost;DATABASE=TestDB;UID=sa;PWD=pass'
objdb->Connect, CONNECTION=conn_str

; Create table
objdb->ExecuteCommand, 'CREATE TABLE Products (ID INT, Name NVARCHAR(100), Price DECIMAL(10,2))'

; Insert data
objdb->ExecuteCommand, "INSERT INTO Products VALUES (1, 'Laptop', 1299.99)"
objdb->ExecuteCommand, "INSERT INTO Products VALUES (2, 'Mouse', 29.99)"

; Query data
recordset = objdb->ExecuteSQL('SELECT * FROM Products WHERE Price > 100')
data = recordset->GetData()
n_rows = recordset->RowCount()

PRINT, 'Found ', n_rows, ' products'
PRINT, data

; Cleanup
recordset->Destroy()
objdb->Disconnect()
OBJ_DESTROY, objdb
```

### Advantages of ODBC Driver

1. **Universal Connectivity** - One driver for many databases
2. **Enterprise Support** - Well-tested ODBC drivers from vendors
3. **Legacy System Access** - Connect to older databases
4. **No Database-Specific Code** - Standard SQL works across platforms
5. **Production Ready** - ODBC is mature, stable technology

### Limitations

1. **Driver Required** - ODBC driver must be installed on system
2. **Platform Specific** - Driver availability varies by OS
3. **No Streaming** - Fetches all results into memory
4. **Generic Interface** - May not expose database-specific features

---

## Apache Kafka Driver Implementation

### Kafka Status: ✅ Fully Implemented

### Kafka Technology Stack

- **Crate**: `rdkafka` v0.36
- **Features**: Producer, Consumer, Admin operations
- **Async Support**: Native async/await with Tokio

### Architecture

Kafka driver creates three clients on connection:

1. **Producer** - Send messages to topics
2. **Consumer** - Read messages from topics
3. **Admin Client** - Manage topics and cluster

```rust
pub struct KafkaConnection {
    brokers: String,
    producer: Option<FutureProducer>,
    consumer: Option<BaseConsumer>,
    admin: Option<AdminClient<DefaultClientContext>>,
}
```

### Special Query Syntax

Since Kafka is a streaming platform (not a traditional database), we use special SQL-like syntax:

#### Topic Management

```xdl
; List all topics
recordset = objdb->ExecuteSQL('LIST TOPICS')

; Create a topic
objdb->ExecuteSQL, 'CREATE TOPIC my-topic'

; Delete a topic
objdb->ExecuteSQL, 'DELETE TOPIC my-topic'
```

#### Producer Operations

```xdl
; Send a message
objdb->ExecuteSQL, 'PRODUCE TO topic-name: message content'

; Send JSON data
objdb->ExecuteSQL, 'PRODUCE TO sensors: {"temp":25.5,"humidity":60}'

; Send array data in loop
FOR i = 1, 100 DO BEGIN
    msg = 'Data point ' + STRTRIM(i,2)
    objdb->ExecuteSQL, 'PRODUCE TO data-stream: ' + msg
ENDFOR
```

#### Consumer Operations

```xdl
; Consume messages (default limit 10)
recordset = objdb->ExecuteSQL('CONSUME FROM topic-name LIMIT 10')

; Get messages
messages = recordset->GetData()
payloads = recordset->GetColumn('payload')

; Process each message
FOR i = 0, N_ELEMENTS(payloads)-1 DO BEGIN
    PRINT, 'Message:', payloads[i]
ENDFOR
```

### Implementation Details

#### Producer

```rust
async fn handle_produce(&self, query: &str) -> DatabaseResult<Recordset> {
    // Parse: PRODUCE TO topic: message
    let (topic, message) = parse_produce_query(query)?;

    // Send message
    let record = FutureRecord::to(topic)
        .payload(message)
        .key("xdl-key");

    let (partition, offset) = producer.send(record, timeout).await?;

    // Return delivery confirmation
    Ok(Recordset with status, partition, offset)
}
```

#### Consumer

```rust
async fn handle_consume(&self, query: &str) -> DatabaseResult<Recordset> {
    // Parse: CONSUME FROM topic LIMIT n
    let (topic, limit) = parse_consume_query(query)?;

    // Subscribe to topic
    consumer.subscribe(&[topic])?;

    // Poll for messages
    let mut messages = Vec::new();
    for _ in 0..limit {
        match consumer.poll(timeout) {
            Some(Ok(msg)) => messages.push(convert_message(msg)),
            None => break,
        }
    }

    // Return as recordset with columns:
    // partition, offset, key, payload, timestamp
    Ok(Recordset::new(columns, messages))
}
```

#### Admin Operations

```rust
async fn handle_create_topic(&self, query: &str) -> DatabaseResult<Recordset> {
    let topic_name = parse_topic_name(query)?;

    let new_topic = NewTopic::new(
        topic_name,
        1,  // partitions
        TopicReplication::Fixed(1)  // replication factor
    );

    admin.create_topics(&[new_topic], &options).await?;

    Ok(Recordset with status)
}
```

### Message Format

Consumed messages are returned as a recordset with these columns:

| Column | Type | Description |
|--------|------|-------------|
| partition | integer | Partition number |
| offset | integer | Message offset |
| key | text | Message key (or NULL) |
| payload | text | Message content |
| timestamp | integer | Message timestamp (milliseconds) |

### XDL Usage Example

```xdl
; Create database object
objdb = OBJ_NEW('XDLdbDatabase')

; Connect to Kafka
objdb->Connect, CONNECTION='kafka://localhost:9092'

; Create topic
objdb->ExecuteSQL, 'CREATE TOPIC sensor-data'

; Produce data stream
FOR i = 1, 100 DO BEGIN
    temperature = 20 + RANDOMU(seed) * 15
    msg = '{"sensor_id":"TEMP01","value":' + STRTRIM(temperature,2) + '}'
    objdb->ExecuteSQL, 'PRODUCE TO sensor-data: ' + msg
    WAIT, 0.1
ENDFOR

; Consume and process
recordset = objdb->ExecuteSQL('CONSUME FROM sensor-data LIMIT 50')
payloads = recordset->GetColumn('payload')

; Analyze stream
temperatures = FLTARR(N_ELEMENTS(payloads))
FOR i = 0, N_ELEMENTS(payloads)-1 DO BEGIN
    ; Parse JSON (simplified)
    temperatures[i] = parse_json_value(payloads[i], 'value')
ENDFOR

avg_temp = MEAN(temperatures)
PRINT, 'Average temperature:', avg_temp

; Cleanup
recordset->Destroy()
objdb->Disconnect()
OBJ_DESTROY, objdb
```

### Use Cases

1. **Real-Time Data Streams** - Sensor data, logs, metrics
2. **Event Sourcing** - Application events, audit logs
3. **Message Queuing** - Async task processing
4. **Data Integration** - ETL pipelines, data lake ingestion
5. **IoT Applications** - Device telemetry, commands

### Advantages

1. **High Throughput** - Millions of messages per second
2. **Durability** - Persistent message storage
3. **Scalability** - Horizontal scaling with partitions
4. **Real-Time** - Low-latency message delivery
5. **Replay** - Can re-read historical messages

### Limitations

1. **Not a Database** - No SQL queries, indexes, or joins
2. **Message Size** - Best for small to medium messages
3. **Setup Required** - Kafka cluster must be running
4. **Learning Curve** - Different paradigm from SQL databases

---

## Integration Status

### Files Modified/Created

**New Implementations:**

1. `xdl-database/src/drivers/odbc.rs` - Full ODBC driver (230 lines)
2. `xdl-database/src/drivers/kafka.rs` - Full Kafka driver (446 lines)

**Examples:**
3. `xdl-database/examples/odbc_sqlserver_example.xdl` - ODBC usage
4. `xdl-database/examples/kafka_streaming_example.xdl` - Kafka usage

**Documentation:**
5. `xdl-database/README.md` - Updated with ODBC and Kafka
6. `docs/ODBC_KAFKA_IMPLEMENTATION.md` - This document

### Feature Matrix

| Database | Status | Query | Commands | Async | Type Conv | Notes |
|----------|--------|-------|----------|-------|-----------|-------|
| PostgreSQL | ✅ | ✅ | ✅ | ✅ | ✅ | Native driver |
| DuckDB | ✅ | ✅ | ✅ | ✅ | ✅ | Embedded analytics |
| Redis | ✅ | ⚠️ | ✅ | ✅ | ✅ | Key-value only |
| **ODBC** | ✅ | ✅ | ✅ | ✅ | ✅ | Universal SQL |
| **Kafka** | ✅ | ✅ | ✅ | ✅ | ✅ | Streaming platform |
| MySQL | ⏳ | - | - | - | - | Native stub ready |

Legend:

- ✅ Fully implemented
- ⚠️ Limited functionality
- ⏳ Stub implementation
- ❌ Not applicable

### Cargo Features

Both drivers are controlled by feature flags:

```toml
[features]
default = ["postgres-support", "duckdb-support", "redis-support"]

odbc-support = ["odbc-api"]
kafka-support = ["rdkafka"]

all = [
    "postgres-support",
    "duckdb-support",
    "redis-support",
    "odbc-support",
    "kafka-support"
]
```

**Enable ODBC:**

```toml
xdl-database = { path = "../xdl-database", features = ["odbc-support"] }
```

**Enable Kafka:**

```toml
xdl-database = { path = "../xdl-database", features = ["kafka-support"] }
```

**Enable All:**

```toml
xdl-database = { path = "../xdl-database", features = ["all"] }
```

---

## Testing

### ODBC Testing

**Prerequisites:**

- ODBC driver manager installed (unixODBC on Linux/Mac, built-in on Windows)
- Specific database ODBC driver installed
- Database server running

**Test Script:**

```bash
# Install ODBC driver (example for PostgreSQL on macOS)
brew install unixodbc
brew install psqlodbc

# List available drivers
odbcinst -q -d

# Test connection
isql -v "DSN=MyDataSource;UID=user;PWD=pass"
```

### Kafka Testing

**Prerequisites:**

- Kafka broker running (or use Docker)
- Default port 9092 accessible

**Quick Start with Docker:**

```bash
# Start Kafka with Docker Compose
docker-compose up -d kafka zookeeper

# Or use confluent-kafka
docker run -p 9092:9092 confluentinc/cp-kafka:latest
```

**Test Connection:**

```bash
# Create topic
kafka-topics --create --topic test --bootstrap-server localhost:9092

# List topics
kafka-topics --list --bootstrap-server localhost:9092
```

---

## Performance Considerations

### ODBC

- **Columnar Fetching**: Uses bulk fetch for efficiency
- **Blocking Operations**: Wrapped in `spawn_blocking` to avoid blocking async runtime
- **Buffer Size**: 100 rows per fetch (configurable)
- **Type Conversion**: Text parsing adds slight overhead

### Kafka

- **Batch Size**: Configure message batch size for throughput
- **Timeout**: 1 second per message poll (configurable)
- **Async Native**: Full async/await, no blocking
- **Memory**: Messages loaded into memory (consider streaming for large volumes)

---

## Error Handling

Both drivers provide comprehensive error handling:

```rust
pub enum DatabaseError {
    #[cfg(feature = "odbc-support")]
    #[error("ODBC error: {0}")]
    ODBCError(String),

    #[error("Kafka error: {0}")]
    KafkaError(String),  // Wraps rdkafka errors

    // ... other error types
}
```

**XDL Error Handling:**

```xdl
CATCH, error
IF error NE 0 THEN BEGIN
    PRINT, 'Database error: ', !ERROR_STATE.MSG
    RETURN
ENDIF

objdb->Connect, CONNECTION=conn_str
```

---

## Future Enhancements

### ODBC

- [ ] Prepared statements support
- [ ] Stored procedure calls
- [ ] Transaction management (BEGIN, COMMIT, ROLLBACK)
- [ ] Connection pooling
- [ ] Streaming result sets for large queries

### Kafka

- [ ] Consumer groups with offset management
- [ ] Exactly-once semantics
- [ ] Schema Registry integration
- [ ] Avro serialization support
- [ ] Partition assignment strategies
- [ ] Kafka Streams integration

---

## Conclusion

Both ODBC and Kafka drivers are now **production-ready** and fully integrated into the XDL database module:

✅ **ODBC** - Provides universal SQL database connectivity

- Supports virtually any database with an ODBC driver
- Enterprise-grade reliability
- Standard SQL interface

✅ **Kafka** - Enables real-time streaming data access

- High-throughput message processing
- Event streaming and data pipelines
- Modern distributed architecture

The XDL database module now supports **5 fully functional database systems**:

1. PostgreSQL (native)
2. DuckDB (embedded)
3. Redis (key-value)
4. ODBC (universal SQL)
5. Apache Kafka (streaming)

This provides XDL users with comprehensive data connectivity options for:

- Traditional SQL databases
- Embedded analytics
- Key-value stores
- Enterprise databases (via ODBC)
- Real-time streaming platforms

All drivers share a unified API, consistent error handling, and async/await support for high-performance data access.
