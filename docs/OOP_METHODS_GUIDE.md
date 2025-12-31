# XDL Object-Oriented Programming Guide

**Version**: 1.0
**Date**: December 31, 2025

---

## Overview

XDL supports object-oriented programming through the arrow syntax (`->`), allowing you to call methods on arrays, strings, DataFrames, and custom objects. This provides a more intuitive and chainable API for common operations.

## Arrow Syntax

The basic syntax for method calls is:

```xdl
result = object->MethodName(arguments)
```

This is equivalent to calling a function with the object as the first argument:

```xdl
result = MethodName(object, arguments)
```

## Array Methods

XDL arrays support 17+ built-in methods for common operations.

### Statistical Methods

```xdl
arr = FINDGEN(100)

; Basic statistics
sum_val = arr->Sum()           ; Sum of all elements
mean_val = arr->Mean()         ; Arithmetic mean
min_val = arr->Min()           ; Minimum value
max_val = arr->Max()           ; Maximum value
median_val = arr->Median()     ; Median value
stddev_val = arr->Stddev()     ; Standard deviation
variance_val = arr->Variance() ; Variance

PRINT, 'Sum: ', sum_val
PRINT, 'Mean: ', mean_val
PRINT, 'Range: ', min_val, ' to ', max_val
```

### Array Manipulation

```xdl
arr = [3, 1, 4, 1, 5, 9, 2, 6]

; Sorting
sorted = arr->Sort()           ; Returns sorted copy
PRINT, sorted                  ; [1, 1, 2, 3, 4, 5, 6, 9]

; Reversing
reversed = arr->Reverse()      ; Returns reversed copy
PRINT, reversed                ; [6, 2, 9, 5, 1, 4, 1, 3]

; Unique elements
unique = arr->Unique()         ; Returns unique values
PRINT, unique                  ; [1, 2, 3, 4, 5, 6, 9]

; Finding elements
indices = arr->Where(arr GT 4) ; Find indices where condition is true
PRINT, indices                 ; [4, 5, 7]
```

### Shape and Size

```xdl
arr = FLTARR(10, 20, 30)

n = arr->NElements()           ; Total number of elements
PRINT, 'Elements: ', n         ; 6000

dims = arr->Dimensions()       ; Array dimensions
PRINT, 'Shape: ', dims         ; [10, 20, 30]

ndims = arr->NDims()           ; Number of dimensions
PRINT, 'Dimensions: ', ndims   ; 3
```

### Reshaping

```xdl
arr = INDGEN(12)               ; [0, 1, 2, ..., 11]

; Reshape to 2D
matrix = arr->Reshape([3, 4])
PRINT, matrix
; [[0, 1, 2, 3],
;  [4, 5, 6, 7],
;  [8, 9, 10, 11]]

; Transpose
transposed = matrix->Transpose()
PRINT, transposed
; [[0, 4, 8],
;  [1, 5, 9],
;  [2, 6, 10],
;  [3, 7, 11]]

; Flatten back to 1D
flat = transposed->Flatten()
```

### Type Conversion

```xdl
int_arr = INDGEN(5)

; Convert to different types
float_arr = int_arr->ToFloat()
double_arr = int_arr->ToDouble()
string_arr = int_arr->ToString()

PRINT, string_arr              ; ['0', '1', '2', '3', '4']
```

### Complete Array Methods Reference

| Method | Description | Example |
|--------|-------------|---------|
| `->Sum()` | Sum of elements | `arr->Sum()` |
| `->Mean()` | Arithmetic mean | `arr->Mean()` |
| `->Min()` | Minimum value | `arr->Min()` |
| `->Max()` | Maximum value | `arr->Max()` |
| `->Median()` | Median value | `arr->Median()` |
| `->Stddev()` | Standard deviation | `arr->Stddev()` |
| `->Variance()` | Variance | `arr->Variance()` |
| `->Sort()` | Sorted copy | `arr->Sort()` |
| `->Reverse()` | Reversed copy | `arr->Reverse()` |
| `->Unique()` | Unique elements | `arr->Unique()` |
| `->Where(cond)` | Find matching indices | `arr->Where(arr GT 0)` |
| `->NElements()` | Element count | `arr->NElements()` |
| `->Dimensions()` | Array shape | `arr->Dimensions()` |
| `->NDims()` | Number of dimensions | `arr->NDims()` |
| `->Reshape(dims)` | Reshape array | `arr->Reshape([3,4])` |
| `->Transpose()` | Transpose array | `arr->Transpose()` |
| `->Flatten()` | Flatten to 1D | `arr->Flatten()` |

---

## String Methods

XDL strings support 16+ methods for text manipulation.

### Case Conversion

```xdl
str = 'Hello World'

upper = str->ToUpper()         ; 'HELLO WORLD'
lower = str->ToLower()         ; 'hello world'

PRINT, upper
PRINT, lower
```

### String Information

```xdl
str = 'Hello World'

len = str->Length()            ; 11
PRINT, 'Length: ', len

; Check content
has_hello = str->Contains('Hello')    ; 1 (true)
starts = str->StartsWith('Hello')     ; 1 (true)
ends = str->EndsWith('World')         ; 1 (true)

PRINT, 'Contains Hello: ', has_hello
```

### Searching

```xdl
str = 'Hello World, Hello Universe'

; Find first occurrence
pos = str->IndexOf('Hello')    ; 0
PRINT, 'First Hello at: ', pos

; Find last occurrence
last_pos = str->LastIndexOf('Hello')  ; 13
PRINT, 'Last Hello at: ', last_pos
```

### Substring and Splitting

```xdl
str = 'Hello World'

; Extract substring
sub = str->Substring(0, 5)     ; 'Hello'
PRINT, sub

; Split string
parts = str->Split(' ')        ; ['Hello', 'World']
PRINT, parts[0]                ; 'Hello'
PRINT, parts[1]                ; 'World'
```

### Trimming and Padding

```xdl
str = '  Hello World  '

; Remove whitespace
trimmed = str->Trim()          ; 'Hello World'
left_trimmed = str->TrimLeft() ; 'Hello World  '
right_trimmed = str->TrimRight() ; '  Hello World'

; Padding
padded = 'Hi'->PadLeft(10)     ; '        Hi'
padded = 'Hi'->PadRight(10)    ; 'Hi        '
```

### Replacing and Matching

```xdl
str = 'Hello World'

; Replace text
replaced = str->Replace('World', 'XDL')
PRINT, replaced                ; 'Hello XDL'

; Regular expression matching
matched = str->Match('W.*d')   ; 1 (true)
PRINT, 'Matches pattern: ', matched
```

### Complete String Methods Reference

| Method | Description | Example |
|--------|-------------|---------|
| `->ToUpper()` | Convert to uppercase | `str->ToUpper()` |
| `->ToLower()` | Convert to lowercase | `str->ToLower()` |
| `->Length()` | String length | `str->Length()` |
| `->Contains(sub)` | Check for substring | `str->Contains('hello')` |
| `->StartsWith(pre)` | Check prefix | `str->StartsWith('He')` |
| `->EndsWith(suf)` | Check suffix | `str->EndsWith('ld')` |
| `->IndexOf(sub)` | Find first occurrence | `str->IndexOf('o')` |
| `->LastIndexOf(sub)` | Find last occurrence | `str->LastIndexOf('o')` |
| `->Substring(start, len)` | Extract substring | `str->Substring(0, 5)` |
| `->Split(delim)` | Split by delimiter | `str->Split(' ')` |
| `->Trim()` | Remove whitespace | `str->Trim()` |
| `->TrimLeft()` | Remove leading whitespace | `str->TrimLeft()` |
| `->TrimRight()` | Remove trailing whitespace | `str->TrimRight()` |
| `->Replace(old, new)` | Replace substring | `str->Replace('a', 'b')` |
| `->Match(pattern)` | Regex match | `str->Match('[0-9]+')` |
| `->PadLeft(len)` | Left pad | `str->PadLeft(10)` |
| `->PadRight(len)` | Right pad | `str->PadRight(10)` |

---

## DataFrame Methods

XDL DataFrames support 15+ methods for data manipulation.

### Basic Operations

```xdl
; Read CSV into DataFrame
df = DF_READ_CSV('data.csv')

; Get dimensions
n_rows = df->NRows()
n_cols = df->NCols()
PRINT, 'Shape: ', n_rows, ' x ', n_cols

; Preview data
df->Head(5)                    ; Print first 5 rows
df->Tail(5)                    ; Print last 5 rows

; Get column names
columns = df->ColumnNames()
PRINT, 'Columns: ', columns
```

### Column Access

```xdl
df = DF_READ_CSV('sales.csv')

; Get single column as array
prices = df->Column('price')
PRINT, 'Average price: ', MEAN(prices)

; Select multiple columns
subset = df->Select(['name', 'price', 'quantity'])
```

### Filtering

```xdl
df = DF_READ_CSV('products.csv')

; Filter rows
expensive = df->Filter(df->Column('price') GT 100)
in_stock = df->Filter(df->Column('quantity') GT 0)

; Combined filters
available_expensive = df->Filter((df->Column('price') GT 100) AND (df->Column('quantity') GT 0))
```

### Sorting

```xdl
df = DF_READ_CSV('users.csv')

; Sort by column
by_name = df->SortBy('name')
by_age_desc = df->SortBy('age', /DESCENDING)

; Sort by multiple columns
sorted = df->SortBy(['department', 'salary'], [0, 1])  ; 0=asc, 1=desc
```

### Grouping and Aggregation

```xdl
df = DF_READ_CSV('sales.csv')

; Group and aggregate
by_region = df->GroupBy('region')
totals = by_region->Sum('amount')

; Multiple aggregations
summary = df->GroupBy('category')->Aggregate({$
    'total': ['amount', 'sum'],$
    'count': ['amount', 'count'],$
    'average': ['amount', 'mean']$
})
```

### Joining

```xdl
orders = DF_READ_CSV('orders.csv')
customers = DF_READ_CSV('customers.csv')

; Inner join
joined = orders->Join(customers, 'customer_id')

; Left join
all_orders = orders->LeftJoin(customers, 'customer_id')
```

### Complete DataFrame Methods Reference

| Method | Description | Example |
|--------|-------------|---------|
| `->NRows()` | Row count | `df->NRows()` |
| `->NCols()` | Column count | `df->NCols()` |
| `->Head(n)` | First n rows | `df->Head(10)` |
| `->Tail(n)` | Last n rows | `df->Tail(10)` |
| `->ColumnNames()` | Get column names | `df->ColumnNames()` |
| `->Column(name)` | Get column as array | `df->Column('price')` |
| `->Select(cols)` | Select columns | `df->Select(['a','b'])` |
| `->Filter(cond)` | Filter rows | `df->Filter(cond)` |
| `->SortBy(col)` | Sort by column | `df->SortBy('name')` |
| `->GroupBy(col)` | Group by column | `df->GroupBy('type')` |
| `->Join(df2, key)` | Inner join | `df->Join(df2, 'id')` |
| `->LeftJoin(df2, key)` | Left join | `df->LeftJoin(df2, 'id')` |
| `->Describe()` | Summary statistics | `df->Describe()` |
| `->Sample(n)` | Random sample | `df->Sample(100)` |
| `->Drop(cols)` | Drop columns | `df->Drop(['temp'])` |

---

## Structure Field Access

XDL structures support dot notation for field access.

```xdl
; Create structure
point = {x: 10.0, y: 20.0, z: 30.0}

; Access fields
PRINT, point.x                 ; 10.0
PRINT, point.y                 ; 20.0

; Modify fields
point.x = 15.0
PRINT, point.x                 ; 15.0

; Nested structures
person = {$
    name: 'John',$
    address: {$
        city: 'New York',$
        zip: '10001'$
    }$
}

PRINT, person.name             ; 'John'
PRINT, person.address.city     ; 'New York'
```

---

## Custom Objects

Create custom objects using `OBJ_NEW`:

```xdl
; Create database object
db = OBJ_NEW('XDLDatabase')

; Call methods
db->Connect('postgresql://localhost/mydb')
rs = db->ExecuteSQL('SELECT * FROM users')

; Check validity
IF OBJ_VALID(db) THEN BEGIN
    PRINT, 'Object is valid'
    db->Disconnect()
ENDIF

; Destroy when done
OBJ_DESTROY, db
```

### Object Lifecycle

```xdl
; Create
obj = OBJ_NEW('ClassName', arg1, arg2)

; Check validity
valid = OBJ_VALID(obj)         ; 1 if valid, 0 if not

; Get class name
class = OBJ_CLASS(obj)         ; Returns 'ClassName'

; Check inheritance
is_base = OBJ_ISA(obj, 'BaseClass')

; Destroy
OBJ_DESTROY, obj
```

---

## Method Chaining

Methods that return objects can be chained:

```xdl
; Array chaining
result = arr->Sort()->Reverse()->Unique()

; String chaining
cleaned = str->Trim()->ToLower()->Replace(' ', '_')

; DataFrame chaining
summary = df->Filter(df->Column('active'))->GroupBy('region')->Sum('sales')
```

---

## GPU-Accelerated Methods

When GPU acceleration is available, array methods automatically use GPU:

```xdl
; Large array - GPU accelerated
arr = RANDOMU(seed, 10000000)

; These use GPU automatically for large arrays
sum_val = arr->Sum()           ; GPU accelerated
mean_val = arr->Mean()         ; GPU accelerated
min_val = arr->Min()           ; GPU accelerated
max_val = arr->Max()           ; GPU accelerated

; Performance: 10-50x faster for arrays > 10K elements
```

---

## See Also

- [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) - Full function list
- [FUNCTION_REFERENCE.md](FUNCTION_REFERENCE.md) - Function documentation
- [GPU_COMPUTE_IMPLEMENTATION.md](GPU_COMPUTE_IMPLEMENTATION.md) - GPU acceleration

---

**Status**: ✅ Production Ready
**Array Methods**: 17+
**String Methods**: 16+
**DataFrame Methods**: 15+
