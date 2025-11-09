# Array and Statistical Functions - Implementation Status

## Overview

This document details the implementation status of array manipulation and statistical functions in XDL, including support for multi-dimensional arrays.

## ✅ Fully Implemented Functions

### Array Generation Functions

| Function | Description | Module | MultiDimArray Support |
|----------|-------------|--------|----------------------|
| `FINDGEN(n)` | Generate floating-point array [0, 1, ..., n-1] | `math.rs` | N/A |
| `INDGEN(n)` | Generate integer array | `math.rs` | N/A |
| `RANDOMU(seed, dims...)` | Uniform random numbers [0, 1] | `math.rs` | ✅ |
| `RANDOMN(seed, dims...)` | Normal random numbers (Gaussian) | `math.rs` | ✅ |
| `FLTARR(dims...)` | Create float array | `array.rs` | ✅ |
| `DBLARR(dims...)` | Create double array | `array.rs` | ✅ |
| `BYTARR(dims...)` | Create byte array | `array.rs` | ✅ |
| `INTARR(dims...)` | Create integer array | `array.rs` | ✅ |
| `LONARR(dims...)` | Create long integer array | `array.rs` | ✅ |

### Array Manipulation Functions

| Function | Description | Module | MultiDimArray Support |
|----------|-------------|--------|----------------------|
| `REFORM(arr, dims...)` | Reshape array | `array.rs` | ✅ |
| `TRANSPOSE(arr)` | Transpose 2D array | `array.rs` | ✅ |
| `ROTATE(arr, dir)` | Rotate 2D array by 90° | `array.rs` | ✅ |
| `SHIFT(arr, offset)` | Circular shift | `array.rs` | ✅ |
| `REVERSE(arr)` | Reverse array order | `array.rs` | ✅ |
| `SORT(arr)` | Sort array elements | `array.rs` | ✅ |
| `REPLICATE(val, dims...)` | Create array filled with value | `array.rs` | ✅ |
| `REBIN(arr, dims...)` | Resize array | `array.rs` | ✅ |
| `CONGRID(arr, dims...)` | Resize with interpolation | `array.rs` | ✅ |
| `INTERPOL(v, n)` | 1D interpolation | `array.rs` | ✅ |
| `MESHGRID(x, y)` | Create coordinate grids | `array.rs` | ✅ |
| `PERMUTE(arr, order)` | Permute dimensions | `array.rs` | ✅ |

### Basic Statistical Functions

| Function | Description | Module | MultiDimArray Support | Notes |
|----------|-------------|--------|----------------------|-------|
| `MIN(arr)` | Minimum value | `array.rs` | ✅ | Updated 2025-11-09 |
| `MAX(arr)` | Maximum value | `array.rs` | ✅ | Updated 2025-11-09 |
| `MEAN(arr)` | Arithmetic mean | `array.rs` | ✅ | Updated 2025-11-09 |
| `TOTAL(arr)` | Sum all elements | `array.rs` | ✅ | Updated 2025-11-09 |
| `MEDIAN(arr)` | Median value | `statistics.rs` | ✅ |
| `VARIANCE(arr)` | Variance | `statistics.rs` | ✅ |
| `STDDEV(arr)` | Standard deviation | `statistics.rs` | ✅ |

### Advanced Statistical Functions

| Function | Description | Module | MultiDimArray Support |
|----------|-------------|--------|----------------------|
| `MOMENT(arr)` | Statistical moments [mean, var, skew, kurt] | `statistics.rs` | ✅ |
| `MEANABSDEV(arr)` | Mean absolute deviation | `statistics.rs` | ✅ |
| `SKEWNESS(arr)` | Skewness | `statistics.rs` | ✅ |
| `KURTOSIS(arr)` | Kurtosis | `statistics.rs` | ✅ |
| `CORRELATE(x, y)` | Correlation coefficient | `statistics.rs` | ✅ |
| `PERCENTILES(arr, p)` | Percentile values | `statistics.rs` | ✅ |
| `ROBUST_MEAN(arr)` | Robust mean (outlier resistant) | `statistics.rs` | ✅ |
| `TRIMMED_MEAN(arr, pct)` | Trimmed mean | `statistics.rs` | ✅ |
| `RESISTANT_MEAN(arr)` | Resistant mean | `statistics.rs` | ✅ |

### Conditional and Filtering Functions

| Function | Description | Module | MultiDimArray Support |
|----------|-------------|--------|----------------------|
| `WHERE(condition)` | Find indices where condition is true | `array.rs` | ✅ |
| `N_ELEMENTS(arr)` | Count array elements | `array.rs` | ✅ |
| `UNIQ(arr)` | Find unique elements | `array.rs` | ✅ |
| `ARRAY_EQUAL(a, b)` | Test array equality | `array.rs` | ✅ |

### Smoothing and Filtering

| Function | Description | Module | MultiDimArray Support |
|----------|-------------|--------|----------------------|
| `SMOOTH(arr, window)` | Moving average (boxcar) | `array.rs` | ✅ |
| `MOVING_AVERAGE(arr, window, mode)` | Moving average with edge modes | `array.rs` | ✅ |
| `WMA(arr, window)` | Weighted moving average | `array.rs` | ✅ |
| `EMA(arr, alpha)` | Exponential moving average | `array.rs` | ✅ |
| `CUMULATIVE_AVERAGE(arr)` | Cumulative mean | `array.rs` | ✅ |
| `MEDIAN_FILTER(arr, window)` | Median filter | `signal.rs` | ✅ |
| `GAUSSIAN_FILTER(arr, sigma)` | Gaussian filter | `image.rs` | ✅ |

## Recent Updates (2025-11-09)

### MultiDimArray Support for Reduction Functions

Added support for `XdlValue::MultiDimArray` to all array reduction functions:

```rust
// Before: Only supported Array
pub fn min_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    match &args[0] {
        XdlValue::Array(arr) => { /* ... */ }
        val => Ok(val.clone()),
    }
}

// After: Supports both Array and MultiDimArray
pub fn min_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    match &args[0] {
        XdlValue::Array(arr) => { /* ... */ }
        XdlValue::MultiDimArray { data, .. } => { /* ... */ }
        val => Ok(val.clone()),
    }
}
```

**Updated Functions:**
- `MIN()` - Now works with 1D, 2D, and 3D arrays
- `MAX()` - Now works with 1D, 2D, and 3D arrays
- `MEAN()` - Now works with 1D, 2D, and 3D arrays
- `TOTAL()` - Now works with 1D, 2D, and 3D arrays

**Already Supported:**
- `STDDEV()` - Already had MultiDimArray support via `VARIANCE()`
- `VARIANCE()` - Already had MultiDimArray support

## Usage Examples

### Basic Statistics on Multi-Dimensional Arrays

```xdl
; Create a 3D array
nx = 64
ny = 64
nz = 32
data = FLTARR(nx, ny, nz)

; Fill with values
FOR i = 0, nx-1 DO BEGIN
    FOR j = 0, ny-1 DO BEGIN
        FOR k = 0, nz-1 DO BEGIN
            data[i,j,k] = i + j*10.0 + k*100.0
        ENDFOR
    ENDFOR
ENDFOR

; Compute statistics (all work directly on 3D array)
min_val = MIN(data)      ; Minimum value across all elements
max_val = MAX(data)      ; Maximum value across all elements
mean_val = MEAN(data)    ; Mean value
stddev_val = STDDEV(data) ; Standard deviation
total_val = TOTAL(data)  ; Sum of all elements

PRINT, 'Min:   ', min_val
PRINT, 'Max:   ', max_val
PRINT, 'Mean:  ', mean_val
PRINT, 'StdDev:', stddev_val
PRINT, 'Total: ', total_val
```

### Random Data Generation

```xdl
; Generate normally distributed random numbers
seed = 42.0
n = 100
data = RANDOMN(seed, n)

; Compute statistics
PRINT, 'Mean: ', MEAN(data)     ; Should be ~0
PRINT, 'StdDev: ', STDDEV(data) ; Should be ~1

; Generate uniform random numbers [0, 1]
uniform = RANDOMU(seed, n)
PRINT, 'Min: ', MIN(uniform)  ; Should be ~0
PRINT, 'Max: ', MAX(uniform)  ; Should be ~1
```

### Conditional Array Operations

```xdl
; Create test data
data = FINDGEN(100)

; Find indices where data > 50
indices = WHERE(data GT 50.0)
PRINT, 'Found ', N_ELEMENTS(indices), ' values > 50'

; Extract matching values
IF N_ELEMENTS(indices) GT 0 THEN BEGIN
    values = data[indices]
    PRINT, 'Mean of values > 50: ', MEAN(values)
ENDIF
```

## Performance Notes

1. **MultiDimArray Operations**: All reduction functions (`MIN`, `MAX`, `MEAN`, `TOTAL`, `STDDEV`) operate on the flattened data array internally, regardless of dimensionality.

2. **Memory Efficiency**: Multi-dimensional arrays are stored in row-major order with shape metadata, allowing efficient iteration without copying.

3. **WHERE Function**: Returns indices as a 1D array. For multi-dimensional arrays, indices refer to the flattened representation.

## Implementation Details

### File Locations

- **Array Functions**: `xdl-stdlib/src/array.rs`
- **Statistical Functions**: `xdl-stdlib/src/statistics.rs`
- **Math Functions**: `xdl-stdlib/src/math.rs`
- **Function Registry**: `xdl-stdlib/src/lib.rs`

### Type Support

All statistical functions support:
- `XdlValue::Array` - 1D arrays
- `XdlValue::MultiDimArray { data, shape }` - N-dimensional arrays
- Scalar values (treated as single-element arrays)

### Error Handling

Functions return appropriate errors for:
- Empty arrays
- Type mismatches
- Invalid dimensions
- Division by zero (where applicable)

## Testing

Comprehensive test coverage includes:
- 1D array operations
- 2D array operations
- 3D array operations
- Edge cases (empty arrays, single elements)
- Type conversions
- Scientific workflow integration (see `examples/scientific/`)

## Future Enhancements

Potential additions:
- [ ] N-dimensional WHERE support (return N-D indices)
- [ ] Axis-specific operations (e.g., `MEAN(arr, axis=0)`)
- [ ] Weighted statistics
- [ ] Online/streaming statistics
- [ ] GPU acceleration via AMP backend

## See Also

- [Implementation Status](IMPLEMENTATION_STATUS.md)
- [Scientific Visualization Workflows](scientific-viz-workflows/)
- [GPU Acceleration](GPU_ACCELERATION_PERFORMANCE_IMPACT.md)
