# XDL DataFrame Reference (Polars)

**Version**: 1.0
**Date**: November 2025
**Status**: Complete ✅
**Feature Flag**: `dataframes`

---

## Overview

XDL includes high-performance DataFrame operations powered by [Polars](https://pola.rs/), a blazingly fast DataFrame library written in Rust. Polars provides:

- **Speed**: Often 10-100x faster than Pandas for large datasets
- **Memory Efficiency**: Lazy evaluation and columnar storage
- **Parallel Processing**: Automatic multi-threading
- **Native Integration**: Zero-copy data sharing with XDL arrays

---

## Enabling DataFrames

DataFrames require the `dataframes` feature flag:

```bash
# Build with DataFrame support
cargo build --features dataframes

# Or in Cargo.toml
[dependencies]
xdl-stdlib = { version = "0.1", features = ["dataframes"] }
```

---

## Function Reference

### File I/O Functions

#### `DF_READ_CSV(filename, [has_header], [delimiter])`

Read a CSV file into a DataFrame.

**Parameters:**
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `filename` | string | required | Path to CSV file |
| `has_header` | boolean | `1` (true) | First row contains column names |
| `delimiter` | string | `","` | Field delimiter character |

**Returns:** DataFrame ID (string)

**Example:**
```idl
df = DF_READ_CSV('data.csv')
df = DF_READ_CSV('data.tsv', 1, '\t')
df = DF_READ_CSV('no_header.csv', 0)
```

---

#### `DF_READ_PARQUET(filename)`

Read a Parquet file into a DataFrame.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `filename` | string | Path to Parquet file |

**Returns:** DataFrame ID (string)

**Example:**
```idl
df = DF_READ_PARQUET('data.parquet')
```

---

#### `DF_READ_JSON(filename)`

Read a JSON file into a DataFrame.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `filename` | string | Path to JSON file |

**Returns:** DataFrame ID (string)

**Example:**
```idl
df = DF_READ_JSON('data.json')
```

---

#### `DF_WRITE_CSV(df_id, filename)`

Write a DataFrame to a CSV file.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `df_id` | string | DataFrame ID |
| `filename` | string | Output file path |

**Example:**
```idl
DF_WRITE_CSV, df, 'output.csv'
```

---

#### `DF_WRITE_PARQUET(df_id, filename)`

Write a DataFrame to a Parquet file.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `df_id` | string | DataFrame ID |
| `filename` | string | Output file path |

**Example:**
```idl
DF_WRITE_PARQUET, df, 'output.parquet'
```

---

### DataFrame Creation

#### `DF_CREATE(column_names, data1, data2, ...)`

Create a DataFrame from XDL arrays.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `column_names` | string array | Column names |
| `data1, data2, ...` | arrays | Data for each column |

**Returns:** DataFrame ID (string)

**Example:**
```idl
names = ['x', 'y', 'z']
x_data = FINDGEN(100)
y_data = SIN(x_data)
z_data = COS(x_data)
df = DF_CREATE(names, x_data, y_data, z_data)
```

---

### Data Inspection

#### `DF_HEAD(df_id, [n])`

Get the first N rows.

**Parameters:**
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `df_id` | string | required | DataFrame ID |
| `n` | integer | `5` | Number of rows |

**Returns:** New DataFrame ID

**Example:**
```idl
first_10 = DF_HEAD(df, 10)
```

---

#### `DF_TAIL(df_id, [n])`

Get the last N rows.

**Parameters:**
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `df_id` | string | required | DataFrame ID |
| `n` | integer | `5` | Number of rows |

**Returns:** New DataFrame ID

**Example:**
```idl
last_5 = DF_TAIL(df, 5)
```

---

#### `DF_SHAPE(df_id)`

Get DataFrame dimensions.

**Returns:** Array `[n_rows, n_columns]`

**Example:**
```idl
shape = DF_SHAPE(df)
PRINT, 'Rows:', shape[0], 'Columns:', shape[1]
```

---

#### `DF_COLUMNS(df_id)`

Get column names.

**Returns:** Array of column name strings

**Example:**
```idl
cols = DF_COLUMNS(df)
PRINT, 'Columns:', cols
```

---

#### `DF_DTYPES(df_id)`

Get column data types.

**Returns:** Array of type strings

**Example:**
```idl
types = DF_DTYPES(df)
FOR i = 0, N_ELEMENTS(types)-1 DO PRINT, cols[i], ': ', types[i]
```

---

#### `DF_DESCRIBE(df_id)`

Get summary statistics.

**Returns:** Summary string with shape, columns, and types

**Example:**
```idl
summary = DF_DESCRIBE(df)
PRINT, summary
```

---

#### `DF_PRINT(df_id)`

Get string representation of DataFrame.

**Returns:** Formatted string

**Example:**
```idl
PRINT, DF_PRINT(df)
```

---

### Data Selection

#### `DF_SELECT(df_id, col1, col2, ...)`

Select specific columns.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `df_id` | string | DataFrame ID |
| `col1, col2, ...` | strings | Column names to select |

**Returns:** New DataFrame ID

**Example:**
```idl
subset = DF_SELECT(df, 'name', 'age', 'salary')
```

---

#### `DF_FILTER(df_id, column, operator, value)`

Filter rows based on condition.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `df_id` | string | DataFrame ID |
| `column` | string | Column to filter on |
| `operator` | string | Comparison operator |
| `value` | any | Value to compare |

**Operators:**
| Operator | Description |
|----------|-------------|
| `=`, `==` | Equal |
| `!=`, `<>` | Not equal |
| `>` | Greater than |
| `<` | Less than |
| `>=` | Greater or equal |
| `<=` | Less or equal |

**Returns:** New DataFrame ID

**Example:**
```idl
adults = DF_FILTER(df, 'age', '>=', 18)
sales = DF_FILTER(df, 'category', '=', 'Electronics')
```

---

### Data Transformation

#### `DF_SORT(df_id, column, [descending])`

Sort DataFrame by column.

**Parameters:**
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `df_id` | string | required | DataFrame ID |
| `column` | string | required | Column to sort by |
| `descending` | boolean | `0` (false) | Sort descending |

**Returns:** New DataFrame ID

**Example:**
```idl
sorted_asc = DF_SORT(df, 'price')
sorted_desc = DF_SORT(df, 'price', 1)
```

---

#### `DF_GROUPBY(df_id, group_col, agg_col, agg_func)`

Group by column and aggregate.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `df_id` | string | DataFrame ID |
| `group_col` | string | Column to group by |
| `agg_col` | string | Column to aggregate |
| `agg_func` | string | Aggregation function |

**Aggregation Functions:**
| Function | Description |
|----------|-------------|
| `sum` | Sum of values |
| `mean`, `avg` | Average |
| `min` | Minimum value |
| `max` | Maximum value |
| `count` | Count of values |
| `first` | First value |
| `last` | Last value |
| `std` | Standard deviation |
| `var` | Variance |

**Returns:** New DataFrame ID

**Example:**
```idl
; Average salary by department
avg_salary = DF_GROUPBY(df, 'department', 'salary', 'mean')

; Total sales by category
total_sales = DF_GROUPBY(df, 'category', 'amount', 'sum')
```

---

#### `DF_JOIN(df1_id, df2_id, on_column, [how])`

Join two DataFrames.

**Parameters:**
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `df1_id` | string | required | Left DataFrame |
| `df2_id` | string | required | Right DataFrame |
| `on_column` | string | required | Column to join on |
| `how` | string | `"inner"` | Join type |

**Join Types:**
| Type | Description |
|------|-------------|
| `inner` | Only matching rows |
| `left` | All left rows + matching right |
| `right` | All right rows + matching left |
| `outer`, `full` | All rows from both |

**Returns:** New DataFrame ID

**Example:**
```idl
; Inner join
result = DF_JOIN(orders, customers, 'customer_id')

; Left join
result = DF_JOIN(orders, customers, 'customer_id', 'left')
```

---

### Data Conversion

#### `DF_TO_ARRAY(df_id, column)`

Convert a column to XDL array.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `df_id` | string | DataFrame ID |
| `column` | string | Column name |

**Returns:** XDL array (numeric or string)

**Example:**
```idl
prices = DF_TO_ARRAY(df, 'price')
PRINT, 'Average price:', MEAN(prices)
```

---

### Memory Management

#### `DF_DROP(df_id)`

Remove DataFrame from memory.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `df_id` | string | DataFrame ID |

**Example:**
```idl
DF_DROP, df  ; Free memory
```

---

## Complete Example

```idl
; Read sales data
df = DF_READ_CSV('sales.csv')

; Inspect
PRINT, DF_DESCRIBE(df)
PRINT, DF_PRINT(DF_HEAD(df, 5))

; Filter to electronics category
electronics = DF_FILTER(df, 'category', '=', 'Electronics')

; Group by region and sum sales
regional_sales = DF_GROUPBY(electronics, 'region', 'amount', 'sum')

; Sort by sales descending
sorted = DF_SORT(regional_sales, 'amount', 1)

; Print results
PRINT, 'Top regions by electronics sales:'
PRINT, DF_PRINT(sorted)

; Export to Parquet
DF_WRITE_PARQUET, sorted, 'regional_electronics.parquet'

; Clean up
DF_DROP, df
DF_DROP, electronics
DF_DROP, regional_sales
DF_DROP, sorted
```

---

## Performance Tips

1. **Use Parquet** for large datasets - faster read/write than CSV
2. **Filter early** - reduce data size before transformations
3. **Drop unused DataFrames** - free memory when done
4. **Use lazy evaluation** - Polars optimizes query plans automatically

---

## Data Type Mapping

| Polars Type | XDL Type |
|-------------|----------|
| `Int64`, `Int32` | `Long` |
| `Float64` | `Double` |
| `Float32` | `Float` |
| `String` | `String` |
| `Boolean` | `Byte` |
