# FINDGEN Function Update

## Overview

Updated FINDGEN (and DINDGEN) to support the full IDL/GDL function signature with multiple dimensions.

## Function Signature

```text
Result = FINDGEN(D1 [, D2, ..., D8] [, INCREMENT=value] [, START=value])
```

## Changes Made

### 1. Multi-Dimensional Support

- **Before**: FINDGEN only accepted a single dimension argument
- **After**: FINDGEN now accepts 1-8 dimension arguments

### 2. Return Type

- **1D arrays**: Returns `XdlValue::Array` (backward compatible)
- **Multi-dimensional arrays**: Returns `XdlValue::MultiDimArray` with shape metadata

### 3. Implementation Details

#### Location

- File: `xdl-stdlib/src/math.rs`
- Functions updated: `findgen()`, `dindgen()`

#### Key Features

- Validates all dimensions are non-negative
- Calculates total size as product of all dimensions
- Generates sequential values starting from 0.0 with increment 1.0
- Preserves shape information for multi-dimensional arrays

### 4. Examples

```xdl
; 1D array (backward compatible)
arr1 = FINDGEN(5)
; Returns: [0.0, 1.0, 2.0, 3.0, 4.0]

; 2D array (3 rows × 4 columns)
arr2 = FINDGEN(3, 4)
; Returns: 3×4 array with values 0-11

; 3D array (2×3×2)
arr3 = FINDGEN(2, 3, 2)
; Returns: 2×3×2 array with values 0-11

; Works with math operations
arr4 = FINDGEN(2, 2) * 2
; Returns: 2×2 array with values [0, 2, 4, 6]
```

### 5. Testing

- Created test script: `test_findgen_simple.xdl`
- Verified backward compatibility with existing code
- Tested multi-dimensional array generation
- Confirmed math operations work with multi-dimensional arrays

## Future Enhancements

### Keyword Arguments (TODO)

The INCREMENT and START keywords are documented but not yet implemented because the XDL evaluator doesn't fully support keyword arguments for functions.

**Planned support:**

```xdl
; Start at 10, increment by 2
arr = FINDGEN(5, START=10, INCREMENT=2)
; Should return: [10.0, 12.0, 14.0, 16.0, 18.0]
```

**Implementation note**: When the evaluator adds keyword support for functions (currently only available for procedures via `call_procedure_with_keywords`), update the `findgen()` function to extract START and INCREMENT from keywords parameter.

### Required Evaluator Changes

Currently in `xdl-interpreter/src/evaluator.rs` (lines 62-65):

```rust
// TODO: Handle keywords
if !keywords.is_empty() {
    return Err(XdlError::NotImplemented("Function keywords".to_string()));
}
```

Once this is implemented, the `findgen()` function can be enhanced to accept keyword arguments.

## Compatibility

### IDL/GDL Compatibility

- ✅ Multiple dimensions (D1-D8)
- ✅ Array generation with sequential values
- ✅ MultiDimArray shape preservation
- ⏳ START keyword (pending evaluator support)
- ⏳ INCREMENT keyword (pending evaluator support)

### Backward Compatibility

- ✅ All existing single-dimension FINDGEN calls work unchanged
- ✅ Return type for 1D arrays remains `XdlValue::Array`
- ✅ No breaking changes to existing code

## Files Modified

1. `xdl-stdlib/src/math.rs`
   - Updated `findgen()` function (lines 367-458)
   - Updated `dindgen()` documentation (lines 460-472)

## Build Status

✅ All compilation successful
✅ Tests passing
✅ Existing examples working correctly
