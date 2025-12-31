# XDL Database Quick Start Guide

**Version**: 1.0
**Date**: December 31, 2025

---

## Overview

XDL provides built-in database connectivity supporting multiple database systems through a unified API. This guide covers basic usage patterns for connecting to databases and executing queries.

## Supported Databases

| Database | Status | Connection Prefix |
|----------|--------|-------------------|
| PostgreSQL | ✅ Full | `postgresql://` or `postgres://` |
| MySQL | ✅ Full | `mysql://` |
| DuckDB | ✅ Full | `duckdb://` or file path |
| SQLite | ✅ Full | `sqlite://` or file path |
| Redis | ✅ Full | `redis://` |
| ODBC | ✅ Full | `odbc://` |
| Apache Kafka | ✅ Full | `kafka://` |

## Basic Usage

### Connecting to a Database

```xdl
; Create database object
db = OBJ_NEW('XDLDatabase')

; Connect to PostgreSQL
db->Connect('postgresql://user:password@localhost:5432/mydb')

; Check connection status
IF db->IsConnected() THEN PRINT, 'Connected!'

; Disconnect when done
db->Disconnect()

; Destroy object
OBJ_DESTROY, db
```

### Executing Queries

```xdl
; Create and connect
db = OBJ_NEW('XDLDatabase')
db->Connect('postgresql://user:password@localhost:5432/mydb')

; Execute a SELECT query - returns recordset
rs = db->ExecuteSQL('SELECT * FROM users WHERE active = true')

; Get number of rows
n_rows = rs->RowCount()
PRINT, 'Found ', n_rows, ' users'

; Iterate through results
FOR i = 0, n_rows - 1 DO BEGIN
    name = rs->GetValue(i, 'name')
    email = rs->GetValue(i, 'email')
    PRINT, name, ': ', email
ENDFOR

; Execute INSERT/UPDATE/DELETE - returns affected row count
affected = db->ExecuteCommand('UPDATE users SET last_login = NOW() WHERE id = 1')
PRINT, 'Updated ', affected, ' rows'

; Cleanup
db->Disconnect()
OBJ_DESTROY, db
```

## Database-Specific Examples

### PostgreSQL

```xdl
db = OBJ_NEW('XDLDatabase')
db->Connect('postgresql://postgres:secret@localhost:5432/analytics')

; Create table
db->ExecuteCommand('CREATE TABLE IF NOT EXISTS measurements (id SERIAL PRIMARY KEY, value FLOAT, timestamp TIMESTAMP DEFAULT NOW())')

; Insert data
FOR i = 0, 99 DO BEGIN
    value = RANDOMU(seed) * 100
    db->ExecuteCommand('INSERT INTO measurements (value) VALUES (' + STRING(value) + ')')
ENDFOR

; Query with aggregation
rs = db->ExecuteSQL('SELECT AVG(value) as avg_val, MAX(value) as max_val FROM measurements')
PRINT, 'Average: ', rs->GetValue(0, 'avg_val')
PRINT, 'Maximum: ', rs->GetValue(0, 'max_val')

db->Disconnect()
OBJ_DESTROY, db
```

### DuckDB (In-Memory Analytics)

```xdl
db = OBJ_NEW('XDLDatabase')
db->Connect('duckdb://:memory:')  ; In-memory database

; DuckDB excels at analytical queries
db->ExecuteCommand('CREATE TABLE sales AS SELECT * FROM read_csv_auto("sales_data.csv")')

; Analytical query
rs = db->ExecuteSQL('SELECT region, SUM(amount) as total FROM sales GROUP BY region ORDER BY total DESC')

FOR i = 0, rs->RowCount() - 1 DO BEGIN
    PRINT, rs->GetValue(i, 'region'), ': $', rs->GetValue(i, 'total')
ENDFOR

OBJ_DESTROY, db
```

### SQLite (Local Database)

```xdl
db = OBJ_NEW('XDLDatabase')
db->Connect('sqlite://./local_data.db')

; Create and populate
db->ExecuteCommand('CREATE TABLE IF NOT EXISTS config (key TEXT PRIMARY KEY, value TEXT)')
db->ExecuteCommand("INSERT OR REPLACE INTO config VALUES ('version', '1.0')")

; Read back
rs = db->ExecuteSQL("SELECT value FROM config WHERE key = 'version'")
PRINT, 'Version: ', rs->GetValue(0, 'value')

db->Disconnect()
OBJ_DESTROY, db
```

### MySQL

```xdl
db = OBJ_NEW('XDLDatabase')
db->Connect('mysql://user:password@localhost:3306/myapp')

; Execute query
rs = db->ExecuteSQL('SELECT id, name, created_at FROM products WHERE price > 100')

PRINT, 'Products over $100:'
FOR i = 0, rs->RowCount() - 1 DO BEGIN
    PRINT, '  ', rs->GetValue(i, 'name')
ENDFOR

db->Disconnect()
OBJ_DESTROY, db
```

### Redis (Key-Value Store)

```xdl
db = OBJ_NEW('XDLDatabase')
db->Connect('redis://localhost:6379')

; Set values
db->ExecuteCommand('SET user:1:name "John Doe"')
db->ExecuteCommand('SET user:1:score 100')

; Get values
rs = db->ExecuteSQL('GET user:1:name')
PRINT, 'Name: ', rs->GetValue(0, 'value')

; Increment
db->ExecuteCommand('INCR user:1:score')

db->Disconnect()
OBJ_DESTROY, db
```

## Working with Recordsets

### Column Information

```xdl
rs = db->ExecuteSQL('SELECT * FROM users LIMIT 1')

; Get column names
columns = rs->ColumnNames()
PRINT, 'Columns: ', columns

; Get column count
n_cols = rs->ColumnCount()
PRINT, 'Number of columns: ', n_cols

; Get column types
FOR i = 0, n_cols - 1 DO BEGIN
    PRINT, columns[i], ': ', rs->ColumnType(i)
ENDFOR
```

### Converting to XDL Arrays

```xdl
rs = db->ExecuteSQL('SELECT value FROM measurements')

; Convert entire column to array
values = rs->ToArray('value')
PRINT, 'Mean value: ', MEAN(values)
PRINT, 'Std dev: ', STDDEV(values)

; Convert to 2D array (all columns)
data = rs->ToMatrix()
```

## Error Handling

```xdl
db = OBJ_NEW('XDLDatabase')

; Use CATCH for error handling
CATCH, error_status
IF error_status NE 0 THEN BEGIN
    PRINT, 'Database error: ', !ERROR_STATE.MSG
    IF OBJ_VALID(db) THEN OBJ_DESTROY, db
    RETURN
ENDIF

db->Connect('postgresql://user:pass@localhost:5432/mydb')
rs = db->ExecuteSQL('SELECT * FROM nonexistent_table')  ; Will raise error

CATCH, /CANCEL
db->Disconnect()
OBJ_DESTROY, db
```

## Connection Pooling

For high-performance applications, use connection pooling:

```xdl
; Create pool with max 10 connections
pool = OBJ_NEW('XDLConnectionPool', MAX_CONNECTIONS=10)
pool->Initialize('postgresql://user:pass@localhost:5432/mydb')

; Get connection from pool
db = pool->GetConnection()

; Use connection
rs = db->ExecuteSQL('SELECT * FROM users')

; Return to pool (don't destroy!)
pool->ReleaseConnection(db)

; Cleanup pool when done
OBJ_DESTROY, pool
```

## Best Practices

### 1. Always Clean Up

```xdl
db = OBJ_NEW('XDLDatabase')
db->Connect(connection_string)

; ... do work ...

; Always disconnect and destroy
db->Disconnect()
OBJ_DESTROY, db
```

### 2. Use Parameterized Queries (When Available)

```xdl
; Avoid SQL injection
; BAD: db->ExecuteSQL("SELECT * FROM users WHERE name = '" + user_input + "'")
; GOOD: Use prepared statements when available
```

### 3. Handle Large Result Sets

```xdl
; Use LIMIT and OFFSET for pagination
page_size = 100
page = 0

REPEAT BEGIN
    rs = db->ExecuteSQL('SELECT * FROM large_table LIMIT ' + STRING(page_size) + ' OFFSET ' + STRING(page * page_size))
    IF rs->RowCount() EQ 0 THEN BREAK

    ; Process page
    ; ...

    page = page + 1
ENDREP
```

### 4. Close Recordsets When Done

```xdl
rs = db->ExecuteSQL('SELECT * FROM data')
; Process results
data = rs->ToArray('value')
; Close to free resources
rs->Close()
```

## Building with Database Support

```bash
# Default build includes DuckDB and SQLite
cargo build -p xdl-database

# With PostgreSQL support
cargo build -p xdl-database --features postgres

# With MySQL support
cargo build -p xdl-database --features mysql

# With all databases
cargo build -p xdl-database --features all-databases
```

## See Also

- [DATABASE_MODULE_IMPLEMENTATION.md](DATABASE_MODULE_IMPLEMENTATION.md) - Technical implementation details
- [MYSQL_IMPLEMENTATION.md](MYSQL_IMPLEMENTATION.md) - MySQL-specific features
- [ODBC_KAFKA_IMPLEMENTATION.md](ODBC_KAFKA_IMPLEMENTATION.md) - ODBC and Kafka details

---

**Status**: ✅ Production Ready
**Databases**: 7 supported backends
**API**: Object-oriented with method chaining
