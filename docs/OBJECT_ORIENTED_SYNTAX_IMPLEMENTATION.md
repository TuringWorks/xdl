# XDL Object-Oriented Syntax Implementation

**Date**: 2025-12-29
**Status**: ✅ Parser Complete | ✅ Interpreter Complete

---

## Overview

This document describes the implementation of object-oriented syntax in XDL to support DataFrame, Array, String, and other object-based operations using the `->` (Arrow) and `.` (Dot) operators.

## Motivation

XDL DataFrame demo scripts use object-oriented syntax for method calls and field access:

```idl
; Method calls with Arrow operator
df = XDLDATAFRAME_READ_CSV('data.csv')
shape = df->Shape()
columns = df->ColumnNames()

; Array methods
arr = [1.0, 2.0, 3.0, 4.0, 5.0]
sum = arr->Sum()
mean = arr->Mean()
sorted = arr->Sort()

; String methods
str = "Hello World"
upper = str->ToUpper()
length = str->Length()

; Field access with Dot operator
point = {x: 10, y: 20}
x_value = point.x
```

## Implementation Status

### ✅ **Phase 1: Parser Implementation** (COMPLETE)

The parser correctly handles both Arrow (`->`) and Dot (`.`) operators:

- `Token::Arrow` for `->` (method calls)
- `Token::Dot` for `.` (field access)

**Key Features**:

- Supports chained method calls: `df->Filter()->Head(10)`
- Supports method calls with and without parentheses
- Supports mixed operations: `arr->Sort()[0]` (method call + indexing)
- Supports field access: `point.x`

### ✅ **Phase 2: Interpreter Implementation** (COMPLETE)

**File**: `xdl-interpreter/src/evaluator.rs`
**New File**: `xdl-interpreter/src/methods.rs`

The interpreter now dispatches method calls based on object type:

```rust
match obj_val {
    XdlValue::DataFrame(id) => call_dataframe_method(...)
    XdlValue::Object(obj_id) => call_user_method(...)
    XdlValue::Array(arr) => methods::call_array_method(...)
    XdlValue::MultiDimArray { data, shape } => methods::call_multidim_method(...)
    XdlValue::NestedArray(rows) => methods::call_nested_array_method(...)
    XdlValue::String(s) => methods::call_string_method(...)
    XdlValue::Struct(_) => Error (use dot notation for fields)
}
```

---

## Supported Methods

### Array Methods (`arr->Method()`)

| Method | Aliases | Description |
|--------|---------|-------------|
| `Sum()` | `Total` | Sum all elements |
| `Mean()` | `Avg`, `Average` | Arithmetic mean |
| `Min()` | `Minimum` | Minimum value |
| `Max()` | `Maximum` | Maximum value |
| `Sort()` | `Sorted` | Returns sorted array |
| `Reverse()` | `Reversed` | Returns reversed array |
| `Unique()` | `Uniq` | Unique values |
| `Length()` | `Len`, `Count`, `Size` | Element count |
| `Variance()` | `Var` | Sample variance |
| `Stddev()` | `Std` | Standard deviation |
| `Median()` | - | Median value |
| `Skewness()` | - | Statistical skewness |
| `Kurtosis()` | - | Statistical kurtosis |
| `Where()` | - | Non-zero indices |
| `Smooth(n)` | - | Moving average |
| `Shift(n)` | - | Circular shift |
| `Histogram(n)` | - | Compute histogram |

**Example**:
```idl
arr = [1.0, 2.0, 3.0, 4.0, 5.0]
PRINT, arr->Sum()       ; 15.0
PRINT, arr->Mean()      ; 3.0
PRINT, arr->Length()    ; 5
PRINT, arr->Sort()      ; [1.0, 2.0, 3.0, 4.0, 5.0]
PRINT, arr->Reverse()   ; [5.0, 4.0, 3.0, 2.0, 1.0]
```

### String Methods (`str->Method()`)

| Method | Aliases | Description |
|--------|---------|-------------|
| `ToUpper()` | `Upper`, `Uppercase`, `Upcase` | Uppercase conversion |
| `ToLower()` | `Lower`, `Lowercase`, `Lowcase` | Lowercase conversion |
| `Length()` | `Len` | Character count |
| `Trim()` | `Strip` | Remove whitespace (both ends) |
| `LTrim()` | `TrimLeft`, `LStrip` | Remove leading whitespace |
| `RTrim()` | `TrimRight`, `RStrip` | Remove trailing whitespace |
| `Compress()` | - | Compress whitespace |
| `Contains(s)` | - | Check if contains substring (0/1) |
| `IndexOf(s)` | `Find`, `Pos` | Position of substring (-1 if not found) |
| `StartsWith(s)` | - | Check if starts with prefix (0/1) |
| `EndsWith(s)` | - | Check if ends with suffix (0/1) |
| `Split(delim)` | - | Split by delimiter |
| `Substring(start, len)` | `Substr`, `Mid` | Extract portion |
| `Replace(old, new)` | - | Replace occurrences |
| `Equals(s)` | `Eq` | String comparison |
| `Match(pattern)` | `Regex` | Regex matching |

**Example**:
```idl
str = "Hello World"
PRINT, str->ToUpper()              ; "HELLO WORLD"
PRINT, str->Length()               ; 11
PRINT, str->Contains('World')      ; 1
PRINT, str->IndexOf('World')       ; 6
PRINT, str->StartsWith('Hello')    ; 1
PRINT, str->Split(' ')             ; ["Hello", "World"]
```

### MultiDimArray Methods (`arr->Method()`)

| Method | Aliases | Description |
|--------|---------|-------------|
| `Sum()` | `Total` | Sum all elements |
| `Mean()` | `Avg`, `Average` | Mean of all elements |
| `Min()` | `Minimum` | Minimum value |
| `Max()` | `Maximum` | Maximum value |
| `Variance()` | `Var` | Sample variance |
| `Stddev()` | `Std` | Standard deviation |
| `Median()` | - | Median value |
| `Shape()` | `Dims`, `Dimensions` | Returns shape array |
| `Ndim()` | `Ndims`, `Rank` | Number of dimensions |
| `Length()` | `Size`, `N_Elements` | Total element count |
| `Flatten()` | `Flat`, `Ravel` | Convert to 1D array |
| `Reshape(dims)` | `Reform` | Change dimensions |
| `Transpose()` | - | Transpose array |
| `Sort()` | `Sorted` | Sort all elements |
| `Reverse()` | `Reversed` | Reverse all elements |

**Example**:
```idl
; Create 2D array
arr = REFORM(FINDGEN(6), 2, 3)
PRINT, arr->Shape()     ; [2.0, 3.0]
PRINT, arr->Ndim()      ; 2
PRINT, arr->Flatten()   ; [0.0, 1.0, 2.0, 3.0, 4.0, 5.0]
PRINT, arr->Sum()       ; 15.0
```

### NestedArray Methods (`matrix->Method()`)

| Method | Aliases | Description |
|--------|---------|-------------|
| `Length()` | `Size`, `Count` | Number of rows |
| `NRows()` | `Rows` | Number of rows |
| `NCols()` | `Cols` | Number of columns |
| `Shape()` | `Dims` | Returns [rows, cols] |
| `Ndim()` | `Rank` | Always 2 |
| `Flatten()` | `Flat`, `Ravel` | Flatten to 1D |
| `Sum()` | `Total` | Sum all elements |
| `Mean()` | `Avg` | Mean of all elements |
| `Min()` | `Minimum` | Minimum value |
| `Max()` | `Maximum` | Maximum value |

### DataFrame Methods (`df->Method()`)

| Method | Aliases | Description |
|--------|---------|-------------|
| `Shape()` | - | Returns [rows, cols] |
| `NRows()` | `Height`, `Len`, `Length` | Row count |
| `NCols()` | `Width` | Column count |
| `ColumnNames()` | `Column_Names`, `Columns` | List of column names |
| `Column(name)` | `Col` | Get column data as array |
| `Row(index)` | - | Get row as struct |
| `Head(n)` | - | First n rows (default 5) |
| `Tail(n)` | - | Last n rows (default 5) |
| `Describe()` | `Info` | DataFrame info string |
| `WriteCSV(file)` | `Write_CSV`, `ToCSV`, `To_CSV` | Write to CSV file |
| `ToJson()` | `To_Json` | Convert to JSON string |
| `Select(col1, col2, ...)` | - | Select columns |
| `SortBy(col, asc)` | `Sort_By`, `Sort` | Sort by column |

**Example**:
```idl
df = XDLDATAFRAME_READ_CSV('data.csv')
PRINT, df->Shape()                  ; [100.0, 5.0]
PRINT, df->NRows()                  ; 100
PRINT, df->ColumnNames()            ; ["id", "name", "age", ...]
PRINT, df->Column('age')            ; [25.0, 30.0, 22.0, ...]
head_df = df->Head(10)              ; First 10 rows
sorted_df = df->SortBy('age', 1)    ; Sort by age ascending
df->WriteCSV('output.csv')          ; Save to file
```

---

## Struct Field Access

Structs use dot notation for field access:

```idl
point = {x: 10, y: 20}
PRINT, point.x    ; 10
PRINT, point.y    ; 20
```

User-defined objects also support field access:

```idl
obj = OBJ_NEW('Person', name='John', age=30)
PRINT, obj.name   ; "John"
PRINT, obj.age    ; 30
```

---

## Error Messages

When calling a method on an unsupported type or using an unknown method, helpful error messages are provided:

```
Array method 'xyz'. Available: Sum, Mean, Min, Max, Sort, Reverse,
Unique, Length, Variance, Stddev, Median, Skewness, Kurtosis,
Where, Smooth, Shift, Histogram, Rebin, Congrid
```

```
String method 'xyz'. Available: ToUpper, ToLower, Length, Trim,
LTrim, RTrim, Compress, Contains, IndexOf, StartsWith, EndsWith,
Split, Substring, Replace, Equals, Match
```

---

## Files Modified/Created

### New Files

- ✅ `xdl-interpreter/src/methods.rs` (~450 lines)
  - Type-specific method dispatch for Array, String, MultiDimArray, NestedArray
  - 24 unit tests for all method types

### Modified Files

- ✅ `xdl-interpreter/src/lib.rs`
  - Added `pub mod methods;` export

- ✅ `xdl-interpreter/src/evaluator.rs`
  - Updated MethodCall dispatch to handle Array, String, MultiDimArray, NestedArray
  - Enhanced DataFrame method support with 15+ methods

---

## Architecture

```
┌─────────────────┐
│  Source Code    │ arr->Sum(), str->ToUpper()
│   (*.xdl)       │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│     Lexer       │ Token::Arrow, Token::Dot
│  (lexer.rs)     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│     Parser      │ Expression::MethodCall, Expression::StructRef
│  (parser.rs)    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│      AST        │ MethodCall { object, method, args }
│   (ast.rs)      │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Interpreter    │ Dispatch by object type
│ (evaluator.rs)  │
└────────┬────────┘
         │
    ┌────┴────┬────────────┬─────────────┐
    ▼         ▼            ▼             ▼
┌────────┐ ┌────────┐ ┌─────────┐ ┌───────────┐
│ Array  │ │ String │ │DataFrame│ │User Object│
│methods │ │methods │ │ methods │ │  methods  │
└────────┘ └────────┘ └─────────┘ └───────────┘
    │         │            │             │
    ▼         ▼            ▼             ▼
┌─────────────────────────────────────────────┐
│              xdl-stdlib Functions           │
│  (array.rs, string.rs, statistics.rs)       │
└─────────────────────────────────────────────┘
```

---

## Test Results

All 24 interpreter tests pass:

```
test methods::tests::test_array_sum ... ok
test methods::tests::test_array_mean ... ok
test methods::tests::test_array_length ... ok
test methods::tests::test_array_min_max ... ok
test methods::tests::test_string_toupper ... ok
test methods::tests::test_string_tolower ... ok
test methods::tests::test_string_length ... ok
test methods::tests::test_string_contains ... ok
test methods::tests::test_string_indexof ... ok
test methods::tests::test_string_startswith ... ok
test methods::tests::test_string_endswith ... ok
test methods::tests::test_multidim_shape ... ok
test methods::tests::test_multidim_ndim ... ok
test methods::tests::test_multidim_flatten ... ok
test methods::tests::test_nested_array_shape ... ok
test methods::tests::test_nested_array_flatten ... ok
test methods::tests::test_unknown_method_error ... ok
...

test result: ok. 24 passed; 0 failed
```

---

## Backward Compatibility

✅ **Fully Backward Compatible**

- Existing XDL scripts continue to work unchanged
- Arrow and Dot only activated when explicitly written
- No changes to function call syntax
- No changes to array indexing syntax

---

## Conclusion

The XDL Object-Oriented Syntax is now **fully implemented** and production-ready:

- ✅ Parser: Complete (Arrow and Dot operators)
- ✅ Interpreter: Complete (type-based method dispatch)
- ✅ Array Methods: 17+ methods
- ✅ String Methods: 16+ methods
- ✅ MultiDimArray Methods: 15+ methods
- ✅ NestedArray Methods: 10+ methods
- ✅ DataFrame Methods: 15+ methods
- ✅ Struct Field Access: Working
- ✅ User-defined Object Methods: Working
- ✅ Tests: 24 passing

---

*Implementation Completed*: 2025-12-29
*Parser Status*: ✅ COMPLETE
*Interpreter Status*: ✅ COMPLETE
*Overall Status*: ✅ 100% COMPLETE
