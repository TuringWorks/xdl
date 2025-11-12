# XDL Database Connectivity Module

Unified database connectivity for XDL supporting multiple database systems including PostgreSQL, MySQL, DuckDB, ODBC, Redis, and Kafka.

## Features

- **Multiple Database Support**
  - ✅ PostgreSQL (fully implemented)
  - ✅ MySQL (fully implemented - native async driver with connection pooling)
  - ✅ DuckDB (fully implemented)
  - ✅ Redis (fully implemented)
  - ✅ ODBC (fully implemented - supports SQL Server, Oracle, MySQL, PostgreSQL, etc.)
  - ✅ Apache Kafka (fully implemented - producer/consumer/admin operations)

- **Async/Await Support** - Built on Tokio for high-performance async operations
- **Connection Pooling** - Efficient connection management (via deadpool)
- **Type-Safe Queries** - Automatic type conversion to XDL types
- **Object-Oriented API** - Familiar IDL/GDL-style object interface

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
xdl-database = { path = "../xdl-database", features = ["postgres-support", "duckdb-support", "redis-support"] }
```

### Available Features

- `postgres-support` - PostgreSQL support
- `mysql-support` - MySQL support
- `duckdb-support` - DuckDB support
- `odbc-support` - ODBC support
- `redis-support` - Redis support
- `kafka-support` - Apache Kafka support
- `all` - Enable all databases

## Usage from XDL

### Basic Example

```xdl
; Create a database object
objdb = OBJ_NEW('XDLdbDatabase')

; Connect to PostgreSQL
conn_str = 'postgresql://user:password@localhost:5432/dbname'
objdb->Connect, CONNECTION=conn_str

; Execute a query
recordset = objdb->ExecuteSQL('SELECT * FROM my_table')

; Get the data
data = recordset->GetData()
PRINT, data

; Get row count
n_rows = recordset->RowCount()
PRINT, 'Number of rows:', n_rows

; Get column names
columns = recordset->ColumnNames()
PRINT, 'Columns:', columns

; Cleanup
recordset->Destroy()
objdb->Disconnect()
OBJ_DESTROY, objdb
```

### MySQL Example

```xdl
; Create database object
objdb = OBJ_NEW('XDLdbDatabase')

; Connect to MySQL
objdb->Connect, CONNECTION='mysql://root:password@localhost:3306/testdb'

; Create a table
objdb->ExecuteCommand, 'CREATE TABLE employees (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(100), salary DECIMAL(10,2))'

; Insert data
objdb->ExecuteCommand, "INSERT INTO employees (name, salary) VALUES ('Alice', 95000.00)"
objdb->ExecuteCommand, "INSERT INTO employees (name, salary) VALUES ('Bob', 75000.00)"

; Query data
recordset = objdb->ExecuteSQL('SELECT * FROM employees ORDER BY salary DESC')
data = recordset->GetData()
PRINT, data

; Cleanup
recordset->Destroy()
objdb->Disconnect()
OBJ_DESTROY, objdb
```

### DuckDB Example

```xdl
; Create database object
objdb = OBJ_NEW('XDLdbDatabase')

; Connect to DuckDB file
objdb->Connect, CONNECTION='my_data.duckdb'

; Create a table
objdb->ExecuteCommand, 'CREATE TABLE users (id INTEGER, name VARCHAR)'

; Insert data
objdb->ExecuteCommand, "INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob')"

; Query data
recordset = objdb->ExecuteSQL('SELECT * FROM users WHERE id > 0')
data = recordset->GetData()
PRINT, data

; Cleanup
recordset->Destroy()
objdb->Disconnect()
OBJ_DESTROY, objdb
```

### Redis Example

```xdl
; Create database object
objdb = OBJ_NEW('XDLdbDatabase')

; Connect to Redis
objdb->Connect, CONNECTION='redis://localhost:6379'

; Set a value
objdb->ExecuteCommand, 'SET mykey myvalue'

; Delete a key
rows_affected = objdb->ExecuteCommand('DEL mykey')
PRINT, 'Rows affected:', rows_affected

; Cleanup
objdb->Disconnect()
OBJ_DESTROY, objdb
```

### ODBC Example (SQL Server)

```xdl
; Create database object
objdb = OBJ_NEW('XDLdbDatabase')

; Connect to SQL Server via ODBC
conn_str = 'DRIVER={ODBC Driver 17 for SQL Server};SERVER=localhost;DATABASE=mydb;UID=user;PWD=pass'
objdb->Connect, CONNECTION=conn_str

; Create table
objdb->ExecuteCommand, 'CREATE TABLE Products (ID INT, Name NVARCHAR(100), Price DECIMAL(10,2))'

; Insert data
objdb->ExecuteCommand, "INSERT INTO Products VALUES (1, 'Laptop', 1299.99)"

; Query data
recordset = objdb->ExecuteSQL('SELECT * FROM Products')
data = recordset->GetData()
PRINT, data

; Cleanup
recordset->Destroy()
objdb->Disconnect()
OBJ_DESTROY, objdb
```

### Apache Kafka Example

```xdl
; Create database object
objdb = OBJ_NEW('XDLdbDatabase')

; Connect to Kafka
objdb->Connect, CONNECTION='kafka://localhost:9092'

; Create a topic
objdb->ExecuteSQL, 'CREATE TOPIC my-topic'

; Produce messages
objdb->ExecuteSQL, 'PRODUCE TO my-topic: Hello from XDL!'
objdb->ExecuteSQL, 'PRODUCE TO my-topic: Second message'

; Consume messages
recordset = objdb->ExecuteSQL('CONSUME FROM my-topic LIMIT 10')
messages = recordset->GetData()
PRINT, messages

; List topics
topics = objdb->ExecuteSQL('LIST TOPICS')
PRINT, topics->GetData()

; Cleanup
recordset->Destroy()
topics->Destroy()
objdb->Disconnect()
OBJ_DESTROY, objdb
```

### Advanced Query Example

```xdl
; Connect to PostgreSQL
objdb = OBJ_NEW('XDLdbDatabase')
objdb->Connect, CONNECTION='postgresql://localhost/mydb'

; Execute complex query
query = 'SELECT id, name, salary FROM employees WHERE department = ''Engineering'' ORDER BY salary DESC'
recordset = objdb->ExecuteSQL(query)

; Get data as structured columns
data_struct = recordset->GetDataStructured()

; Access individual columns
ids = data_struct.id
names = data_struct.name
salaries = data_struct.salary

; Print results
FOR i = 0, N_ELEMENTS(ids) - 1 DO BEGIN
    PRINT, ids[i], names[i], salaries[i]
ENDFOR

; Cleanup
recordset->Destroy()
objdb->Disconnect()
OBJ_DESTROY, objdb
```

## Connection Strings

### PostgreSQL
```
postgresql://user:password@host:port/database
postgres://user:password@host:port/database
```

### MySQL
```
mysql://user:password@host:port/database
mysql://user:password@host/database  (port defaults to 3306)
mysql://root:pass@localhost:3306/mydb
```

Also compatible with MariaDB and other MySQL-protocol databases.

### DuckDB
```
duckdb://path/to/file.duckdb
/path/to/file.duckdb
file.db
```

### ODBC
```
DRIVER={PostgreSQL Unicode(x64)};SERVER=host;UID=user;PWD=password;DATABASE=dbname;PORT=5432
```

### Redis
```
redis://localhost:6379
redis://:password@localhost:6379/0
```

### Kafka
```
kafka://localhost:9092
kafka://broker1:9092,broker2:9092  (multiple brokers)
localhost:9092                      (simplified format)
```

**Special Kafka Query Syntax:**

Kafka uses a special query syntax since it's a streaming platform:

```xdl
; Topic Management
'LIST TOPICS'
'CREATE TOPIC topic-name'
'DELETE TOPIC topic-name'

; Producer (send messages)
'PRODUCE TO topic-name: message content'
'PRODUCE TO sensors: {"temp":25.5,"humidity":60}'

; Consumer (read messages)
'CONSUME FROM topic-name LIMIT 10'
'CONSUME FROM events LIMIT 100'
```

## API Reference

### XDLdbDatabase Methods

#### Connect
```xdl
objdb->Connect, CONNECTION=connection_string
```
Connects to the database using the specified connection string.

#### Disconnect
```xdl
objdb->Disconnect
```
Disconnects from the database.

#### ExecuteSQL
```xdl
recordset = objdb->ExecuteSQL(query_string)
```
Executes a SELECT query and returns a recordset object.

#### ExecuteCommand
```xdl
rows_affected = objdb->ExecuteCommand(command_string)
```
Executes a command (INSERT, UPDATE, DELETE) and returns the number of rows affected.

#### IsConnected
```xdl
connected = objdb->IsConnected()
```
Returns 1 if connected, 0 otherwise.

#### DatabaseType
```xdl
db_type = objdb->DatabaseType()
```
Returns the type of database currently connected.

### Recordset Methods

#### GetData
```xdl
data = recordset->GetData()
```
Returns all data as a nested array.

#### GetDataStructured
```xdl
data_struct = recordset->GetDataStructured()
```
Returns data as a structure with column names as fields.

#### GetColumn
```xdl
column_data = recordset->GetColumn('column_name')
```
Returns a specific column as an array.

#### RowCount
```xdl
n_rows = recordset->RowCount()
```
Returns the number of rows in the recordset.

#### ColumnCount
```xdl
n_cols = recordset->ColumnCount()
```
Returns the number of columns in the recordset.

#### ColumnNames
```xdl
names = recordset->ColumnNames()
```
Returns an array of column names.

#### Next
```xdl
has_more = recordset->Next()
```
Moves to the next row. Returns 1 if successful, 0 if no more rows.

#### Reset
```xdl
recordset->Reset
```
Resets the cursor to the first row.

#### CurrentRow
```xdl
row_data = recordset->CurrentRow()
```
Returns the current row as a structure.

## Architecture

### Module Structure

```
xdl-database/
├── src/
│   ├── lib.rs                 # Main module, registry, XDLDatabase
│   ├── error.rs               # Error types
│   ├── connection.rs          # Connection enum wrapper
│   ├── recordset.rs           # Query results
│   └── drivers/
│       ├── mod.rs             # Driver exports
│       ├── postgres.rs        # PostgreSQL driver
│       ├── mysql.rs           # MySQL driver
│       ├── duckdb.rs          # DuckDB driver
│       ├── odbc.rs            # ODBC driver
│       ├── redis_driver.rs    # Redis driver
│       └── kafka.rs           # Kafka driver
├── Cargo.toml
└── README.md
```

### Key Components

1. **XDLDatabase** - Main database object
   - Manages connections
   - Executes queries
   - Returns recordsets

2. **DatabaseConnection** - Enum wrapper for different database types
   - Abstracts driver differences
   - Provides unified interface

3. **Recordset** - Query results container
   - Stores rows and columns
   - Provides data access methods
   - Converts to XDL types

4. **DatabaseRegistry** - Global object registry
   - Maps object IDs to database instances
   - Manages object lifecycle
   - Thread-safe with RwLock

5. **Drivers** - Individual database implementations
   - PostgreSQL (tokio-postgres with connection pooling)
   - MySQL (mysql_async with connection pooling)
   - DuckDB (duckdb crate - embedded analytics)
   - Redis (redis crate - key-value store)
   - ODBC (odbc-api - universal SQL connectivity)
   - Kafka (rdkafka - streaming platform)

## Type Conversion

Database types are automatically converted to XDL types:

| Database Type | XDL Type |
|---------------|----------|
| BOOLEAN | Long (0/1) |
| SMALLINT | Int |
| INTEGER | Long |
| BIGINT | Long64 |
| REAL/FLOAT4 | Float |
| DOUBLE/FLOAT8 | Double |
| VARCHAR/TEXT | String |
| NULL | Undefined |

## Error Handling

The module provides comprehensive error handling:

```xdl
objdb = OBJ_NEW('XDLdbDatabase')

; Try to connect
CATCH, error
IF error NE 0 THEN BEGIN
    PRINT, 'Connection failed: ', !ERROR_STATE.MSG
    RETURN
ENDIF

objdb->Connect, CONNECTION='postgresql://localhost/mydb'

; Execute query with error handling
CATCH, error
IF error NE 0 THEN BEGIN
    PRINT, 'Query failed: ', !ERROR_STATE.MSG
    objdb->Disconnect()
    RETURN
ENDIF

recordset = objdb->ExecuteSQL('SELECT * FROM users')
```

## Performance Considerations

1. **Connection Pooling** - Reuse connections for better performance
2. **Async Operations** - All I/O is asynchronous
3. **Batch Operations** - Use transactions for multiple commands
4. **Result Set Size** - Be mindful of large result sets

## Future Enhancements

### Planned Features

1. **Prepared Statements** - For secure parameterized queries (PostgreSQL, MySQL)
2. **Transaction Support** - BEGIN, COMMIT, ROLLBACK
3. **Enhanced Connection Pooling** - More pool configuration options
4. **Streaming Results** - For large datasets
5. **Additional Databases** - MongoDB, Cassandra, ClickHouse

### Example Future API

```xdl
; Prepared statement
stmt = objdb->Prepare('SELECT * FROM users WHERE id = ?')
recordset = stmt->Execute([123])

; Transaction
objdb->BeginTransaction()
objdb->ExecuteCommand('INSERT INTO users VALUES (1, ''Alice'')')
objdb->ExecuteCommand('INSERT INTO users VALUES (2, ''Bob'')')
objdb->Commit()

; Connection pool
pool = OBJ_NEW('XDLdbConnectionPool', SIZE=10)
pool->Connect, CONNECTION='postgresql://localhost/mydb'
conn1 = pool->GetConnection()
conn2 = pool->GetConnection()
```

## Testing

Run tests with:

```bash
cargo test --package xdl-database --all-features
```

## Examples

See the `examples/` directory for more usage examples:

- `postgresql_example.xdl` - PostgreSQL query example
- `mysql_example.xdl` - MySQL CRUD operations and queries
- `duckdb_analytics.xdl` - DuckDB analytics example
- `odbc_sqlserver_example.xdl` - ODBC with SQL Server
- `kafka_streaming_example.xdl` - Kafka streaming operations

## Contributing

To add support for a new database:

1. Add the driver dependency to `Cargo.toml`
2. Create a new driver module in `src/drivers/`
3. Implement the required methods:
   - `connect()`
   - `execute()`
   - `execute_command()`
   - `close()`
   - `is_connected()`
4. Add the driver to `DatabaseConnection` enum
5. Add feature flag support
6. Write tests
7. Update documentation

## License

GPL-2.0 (same as XDL)

## Support

For issues and questions:
- GitHub Issues: https://github.com/gnudatalanguage/gdl/issues
- Documentation: https://docs.gnudatalanguage.com
