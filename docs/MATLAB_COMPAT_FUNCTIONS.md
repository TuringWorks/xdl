# MATLAB Compatibility Functions in XDL

## Overview

XDL includes a growing subset of MATLAB-inspired runtime functions. Each function has its own tested scope; the presence of a MATLAB name does not imply complete MATLAB compatibility. See the compatibility matrix and limitations below.

## Implementation

**Module:** `xdl-stdlib/src/matlab_compat.rs`
**Integration:** Registered in `xdl-stdlib/src/lib.rs`
**Status:** Partial compatibility; implementations are tested within the scopes documented below.

## Array size semantics

Transpiled MATLAB `size(x)` calls `MATLAB_SIZE(x)` and returns the represented
dimension vector. `size(x, dim)` returns the selected dimension, including `1`
for dimensions beyond the represented rank. Row and column vectors, rectangular
nested matrices, and shaped numeric arrays have dedicated shape handling.
For a known array variable, `[d1, ..., dn] = size(A)` is lowered to one
dimension query per output; the final output multiplies the remaining
dimensions, matching MATLAB's behavior for N-D arrays. General expression and
object-array forms are not verified.
This follows MATLAB's documented dimension-vector and selected-dimension forms
for the supported input shapes; see [MathWorks `size`](https://www.mathworks.com/help/matlab/ref/double.size.html).

Transpiled `find(x)`, `find(x,k)`, and `find(x,k,'first'|'last')` calls use
`MATLAB_FIND`. The verified subset returns one-based linear indices for
numeric scalars, vectors, and rectangular 2D matrices. A nonzero scalar returns
the `1x1` index `[1]`; scalar zero and empty `[]` return an empty `0x0` result.
Row-vector inputs preserve row
orientation; column vectors and
matrices return a column. Empty results preserve row-vector orientation and
are columns for other inputs; an empty 0×0 matrix and scalar zero produce a
0×0 result. Positive integer limits and `'first'`/`'last'` direction are also
supported for the tested numeric shapes, with `'last'` indices returned in
ascending order. The two-output form `[row, col] = find(A[,k[,direction]])`
is supported for rectangular numeric 2D matrices with at least two rows and
columns through `MATLAB_FIND_ROWS` and `MATLAB_FIND_COLUMNS`; results are
one-based column vectors, in MATLAB column-major nonzero order. Positive
integer `k` and `'first'`/`'last'` are supported, and `'last'` results remain
in ascending index order. The three-output form `[row, col, linear] = find(A)`
is also supported for this 2D subset; the third output contains one-based
column-major linear indices. Two-output vector cases, four or more outputs,
higher-dimensional inputs, and nonnumeric array types are not implemented or
verified. See [MathWorks `find`](https://www.mathworks.com/help/matlab/ref/find.html).

For a known array variable used with one scalar linear subscript, the
transpiler emits `MATLAB_INDEX(array, index)`. This helper reads numeric vector,
rectangular nested-matrix, and shaped-array values using one-based
column-major indexing. Indexed assignment and vector-valued index arguments are
separate compatibility gaps.

## Numeric CSV import

Transpiled `readmatrix(filename)` uses `MATLAB_READMATRIX` for a strict subset:
headerless rectangular CSV containing only numeric fields. The result is an
XDL nested numeric matrix usable by tested matrix indexing and plotting paths.
Empty files, ragged rows, malformed numbers, and unsupported argument forms
return errors. Header detection, import options, mixed types, dates, and
large-file streaming are not implemented. See MathWorks
[`readmatrix`](https://www.mathworks.com/help/matlab/ref/readmatrix.html).

## Basic plot labels

For the supported single-plot workflow, literal `title('...')`,
`xlabel('...')`, and `ylabel('...')` calls immediately after `plot(x,y)` are
attached to that emitted XDL `PLOT` as `TITLE`, `XTITLE`, and `YTITLE`
keywords. Dynamic label expressions, labels separated from the plot by another
statement, legends, grid/axis controls, and multi-axes graphics are not
supported by this lowering. XDL's plot procedure consumes these keywords in
the native plot-window backend; backend-specific presentation parity remains
limited.

Transpiled MATLAB `length(x)` calls `MATLAB_LENGTH(x)`, which returns the
largest represented dimension (for example, `length` of a 2×3 matrix is 3).
`numel(x)` calls `MATLAB_NUMEL(x)` and returns the total scalar element count
(6 for that matrix). Rectangular nested arrays and shaped numeric arrays
are supported; ragged nested arrays return an error. This is a tested subset,
not a claim of complete MATLAB string, table, or arbitrary object-array parity.
MATLAB specifies `length` as the largest array dimension (zero for empty arrays)
and `numel` as the total element count; see [MathWorks `length`](https://www.mathworks.com/help/matlab/ref/double.length.html)
and [MathWorks `numel`](https://www.mathworks.com/help/matlab/ref/double.numel.html).

The two-input MATLAB `atan2(y, x)` form maps to XDL `ATAN(y, x)`. It applies
elementwise to equal-length numeric vectors, equal-shape numeric arrays, and
rectangular nested matrix literals, and expands a numeric scalar across these
supported shapes. Mismatched array shapes and ragged nested matrices return
errors; MATLAB's broader implicit expansion and complex-input behavior are not
established by these tests. The mapping preserves MATLAB's `y`-then-`x`
argument order and quadrant behavior.
MATLAB also defines `atan2(0,-0)` and `atan2(-0,-0)` as zero; XDL follows
that special case. MATLAB accepts operands with any compatible array sizes,
including row/column singleton-dimension expansion, while XDL's current subset
requires equal shapes or a scalar. See [MathWorks `atan2`](https://www.mathworks.com/help/matlab/ref/double.atan2.html).

For transpose, MATLAB defines `A.'` and `transpose(A)` as nonconjugating and
`A'` as conjugating. XDL implements the former for directly named numeric
vectors and rectangular matrices and rejects the latter. See [MathWorks
`transpose`](https://www.mathworks.com/help/matlab/ref/double.transpose.html).
The scalar results from `MATLAB_LENGTH`, `MATLAB_NUMEL`, and `size(x, dim)` are
XDL `DOUBLE` values, matching MATLAB's numeric dimension/count results.
XDL's single string value cannot distinguish a MATLAB char vector from a string
scalar, so string-array semantics remain outside this verified subset.

## Available Functions

### LINSPACE - Linear Spacing

Generate linearly spaced vector between two values.

**Syntax:**

```xdl
result = LINSPACE(start, stop, n)
```

**Parameters:**

- `start` - Starting value
- `stop` - Ending value (inclusive)
- `n` - Number of points (optional, default: 100)

**Example:**

```xdl
x = LINSPACE(0, 10, 11)
; Result: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
```

**MATLAB Equivalent:**

```matlab
x = linspace(0, 10, 11);
```

---

### LOGSPACE - Logarithmic Spacing

Generate logarithmically spaced vector from 10^start to 10^stop.

**Syntax:**

```xdl
result = LOGSPACE(start, stop, n)
```

**Parameters:**

- `start` - Starting exponent (10^start)
- `stop` - Ending exponent (10^stop)
- `n` - Number of points (optional, default: 50)

**Example:**

```xdl
x = LOGSPACE(0, 3, 4)
; Result: [1, 10, 100, 1000]
```

**MATLAB Equivalent:**

```matlab
x = logspace(0, 3, 4);
```

---

### MESHGRID - Coordinate Matrices

Create 2D coordinate matrices from coordinate vectors.

**Syntax:**

```xdl
result = MESHGRID(x_vec, y_vec)
; Returns [X, Y] as nested array
```

**Parameters:**

- `x_vec` - 1D array for x-coordinates
- `y_vec` - 1D array for y-coordinates

**Returns:**

- Nested array containing [X, Y] where:
  - X is nx×ny matrix with rows = x_vec
  - Y is nx×ny matrix with columns = y_vec

**Example:**

```xdl
x = [1.0, 2.0, 3.0]
y = [4.0, 5.0]
grids = MESHGRID(x, y)
; X = [[1,1], [2,2], [3,3]]
; Y = [[4,5], [4,5], [4,5]]
```

**MATLAB Equivalent:**

```matlab
[X, Y] = meshgrid([1, 2, 3], [4, 5]);
```

**Note:** XDL's MESHGRID is also available in `math.rs` for consistency with existing code.

---

### NDGRID - N-D Coordinate Arrays

Generate N-D coordinate arrays (matrix indexing convention).

**Syntax:**

```xdl
result = NDGRID(x_vec, y_vec)
```

**Parameters:**

- `x_vec` - 1D array for first dimension
- `y_vec` - 1D array for second dimension

**Returns:**

- Nested array containing [X, Y] where:
  - X varies along columns
  - Y varies along rows (transpose of MESHGRID)

**Example:**

```xdl
x = [1.0, 2.0]
y = [3.0, 4.0, 5.0]
grids = NDGRID(x, y)
```

**MATLAB Equivalent:**

```matlab
[X, Y] = ndgrid([1, 2], [3, 4, 5]);
```

**Note:** For 2D case, NDGRID is the transpose of MESHGRID. N-D cases (N > 2) are not yet implemented.

---

### REPMAT - Replicate and Tile Array

Replicate array m times vertically and n times horizontally.

**Syntax:**

```xdl
result = REPMAT(array, m, n)
```

**Parameters:**

- `array` - Input array to replicate
- `m` - Number of vertical replications
- `n` - Number of horizontal replications

**Example:**

```xdl
a = [1.0, 2.0]
b = REPMAT(a, 2, 3)
; Result: [1,2,1,2,1,2,1,2,1,2,1,2] (12 elements)
```

**MATLAB Equivalent:**

```matlab
a = [1, 2];
b = repmat(a, 2, 3);
```

---

### SQUEEZE - Remove Singleton Dimensions

Remove dimensions of size 1 from multi-dimensional array.

**Syntax:**

```xdl
result = SQUEEZE(array)
```

**Parameters:**

- `array` - Multi-dimensional array with singleton dimensions

**Returns:**

- Array with singleton dimensions removed
- Converts to scalar if all dimensions were 1
- Converts to 1D if reduced to single dimension

**Example:**

```xdl
; Assuming arr is 5x1x3 array
; squeezed = SQUEEZE(arr)
; Result: 5x3 array
```

**MATLAB Equivalent:**

```matlab
squeezed = squeeze(arr);
```

---

### INTERP1 - 1D Interpolation

Perform 1D linear or nearest-neighbor interpolation.

**Syntax:**

```xdl
result = INTERP1(x, y, xi, [method])
```

**Parameters:**

- `x` - Finite, strictly monotonic sample coordinates (ascending or descending)
- `y` - 1D sample values corresponding to `x`; must have the same length
- `xi` - Scalar or vector query points
- `method` - Optional string: `'linear'` (default) or `'nearest'`

**Returns:**

- Array of interpolated values; out-of-range queries return `NaN`

**Example:**

```xdl
xp = [0.0, 1.0, 2.0, 3.0]
yp = [0.0, 1.0, 4.0, 9.0]
xi = [0.5, 1.5, 2.5]
yi = INTERP1(xp, yp, xi)
; Result: [0.5, 2.5, 6.5]
```

**MATLAB Equivalent:**

```matlab
xp = [0, 1, 2, 3];
yp = [0, 1, 4, 9];
xi = [0.5, 1.5, 2.5];
yi = interp1(xp, yp, xi);
```

**Features:**

- Linear interpolation between points
- Out-of-range queries return `NaN` (no extrapolation option is implemented)
- Nearest-neighbor ties choose the next input sample; descending-axis tie
  parity against MATLAB/Octave has not been checked
- Handles exact matches
- Supports scalar or array xi

---

## Usage in XDL Programs

### Direct Usage

```xdl
; Create linearly spaced data
x = LINSPACE(0, 2*3.14159, 100)
y = SIN(x)

; Create coordinate grids for 3D plotting
grids = MESHGRID(x, x)

; Interpolate data
xi = LINSPACE(0, 10, 50)
yi = INTERP1(x_data, y_data, xi)
```

### MATLAB Transpilation

The MATLAB transpiler automatically converts MATLAB function calls to these XDL functions:

**MATLAB Code:**

```matlab
x = linspace(0, 10, 11);
y = logspace(0, 3, 4);
[X, Y] = meshgrid(x, y);
```

**Transpiled XDL Code:**

```xdl
x = LINSPACE(0, 10, 11)
y = LOGSPACE(0, 3, 4)
[X, Y] = MESHGRID(x, y)
```

---

## Implementation Details

### Module Structure

**File:** `xdl-stdlib/src/matlab_compat.rs`

```rust
pub mod matlab_compat {
    pub fn matlab_length(args: &[XdlValue]) -> XdlResult<XdlValue>
    pub fn matlab_numel(args: &[XdlValue]) -> XdlResult<XdlValue>
    pub fn matlab_size(args: &[XdlValue]) -> XdlResult<XdlValue>
    pub fn linspace(args: &[XdlValue]) -> XdlResult<XdlValue>
    pub fn logspace(args: &[XdlValue]) -> XdlResult<XdlValue>
    pub fn ndgrid(args: &[XdlValue]) -> XdlResult<XdlValue>
    pub fn repmat(args: &[XdlValue]) -> XdlResult<XdlValue>
    pub fn squeeze(args: &[XdlValue]) -> XdlResult<XdlValue>
    pub fn interp1(args: &[XdlValue]) -> XdlResult<XdlValue>
}
```

### Registration

Functions are registered in `xdl-stdlib/src/lib.rs`:

```rust
// MATLAB compatibility functions
"MATLAB_LENGTH" => matlab_compat::matlab_length(args),
"MATLAB_NUMEL" => matlab_compat::matlab_numel(args),
"MATLAB_SIZE" => matlab_compat::matlab_size(args),
"MATLAB_SIZE_TAIL" => matlab_compat::matlab_size_tail(args),
"MATLAB_FIND" => matlab_compat::matlab_find(args),
"MATLAB_INDEX" => matlab_compat::matlab_index(args),
"LINSPACE" => matlab_compat::linspace(args),
"LOGSPACE" => matlab_compat::logspace(args),
"REPMAT" => matlab_compat::repmat(args),
"SQUEEZE" => matlab_compat::squeeze(args),
"NDGRID" => matlab_compat::ndgrid(args),
"INTERP1" => matlab_compat::interp1(args),
```

### Type Support

Accepted value shapes and scalar types vary by function. For example,
`INTERP1` currently accepts one-dimensional `Array` samples, while
`MATLAB_LENGTH`, `MATLAB_NUMEL`, `MATLAB_FIND`, and `MATLAB_INDEX` handle their
documented rectangular and shaped numeric subsets. Each helper validates its
own argument count and supported types; this does not imply uniform support
across the module.

---

## Testing

### Unit Tests

The focused tests in `matlab_compat.rs` cover selected behavior; passing them
does not establish MATLAB/Octave parity.

```bash
cargo test --lib -p xdl-stdlib matlab_compat
```

**Most recent focused result (2026-11-16):**

```text
running 28 tests
test result: ok. 28 passed; 0 failed
```

### CLI Integration Evidence

`cargo test -p xdl-cli --test cli_tests` exercises MATLAB source through
transpilation and execution. It currently includes focused cases for indexing,
`length`/`numel`, `size`, `find` first/last limits, named-array transpose, and
elementwise/scalar-expanded `atan2`. These are executable
smoke/regression tests, not comparisons against MATLAB reference outputs.
```

---

## Comparison with IDL/GDL

These functions **extend beyond** standard IDL/GDL capabilities:

| Function | IDL/GDL | XDL | Notes |
|----------|---------|-----|-------|
| LINSPACE | ❌ | ✅ | MATLAB-inspired feature |
| LOGSPACE | ❌ | ✅ | MATLAB-inspired feature |
| MESHGRID | ❌ | ✅ | MATLAB-inspired feature |
| NDGRID | ❌ | ✅ | MATLAB-inspired feature |
| REPMAT | ❌ | ✅ | MATLAB-inspired feature |
| SQUEEZE | ❌ | ✅ | MATLAB-inspired feature |
| INTERP1 | Partial | ✅ | Enhanced MATLAB-style implementation |

**IDL/GDL Workarounds:**

- IDL uses `FINDGEN` + arithmetic for linspace
- IDL uses `REBIN` for some repmat operations
- IDL uses nested loops for meshgrid
- IDL has `INTERPOLATE` but different syntax than MATLAB's `interp1`

**XDL Advantage:**
These functions provide a cleaner, more intuitive MATLAB-compatible API while maintaining full IDL/GDL compatibility for existing code.

---

## Performance Considerations

### Memory Efficiency

- **LINSPACE/LOGSPACE**: O(n) memory allocation
- **MESHGRID/NDGRID**: O(nx × ny) memory allocation
- **REPMAT**: O(m × n × len) memory allocation
- **INTERP1**: O(n) interpolation with linear search

### Optimizations

- Pre-allocated vectors with `Vec::with_capacity()`
- Efficient array slicing with `extend_from_slice()`
- Zero-copy operations where possible
- Inline arithmetic for common cases

### Future Enhancements

- GPU acceleration for large arrays
- Parallel processing for MESHGRID/NDGRID
- Cubic spline interpolation for INTERP1
- N-D support for NDGRID (currently 2D only)
- Additional interpolation methods (cubic, spline, etc.)

---

## Known Limitations

### Current Limitations

1. **NDGRID**: Only 2D case implemented (N > 2 returns NotImplemented error)
2. **INTERP1**: Only linear and nearest methods are implemented; MATLAB's
   implicit sample coordinates, matrix-valued samples, shape preservation,
   cubic/spline methods, and explicit extrapolation arguments remain unsupported
3. **MESHGRID**: Runtime returns a nested pair; MATLAB's general multiple-output
   assignment is unsupported, and the transpiler special case is not conformance
   tested
4. **SIZE**: Single-output `size(A)` and `size(A, dim)` use `MATLAB_SIZE`;
   `[d1,...,dn]=size(A)` on a known array variable uses `MATLAB_SIZE` plus
   `MATLAB_SIZE_TAIL`. Other expression/object-array forms and nonnumeric
   semantics remain unsupported or unverified.
5. **FIND**: one-output `find(A[,k[,direction]])` uses `MATLAB_FIND` for tested
   numeric vectors and rectangular 2D matrices. `[row,col]=find(A)` lowers to
   `MATLAB_FIND_ROWS` and `MATLAB_FIND_COLUMNS` for rectangular numeric 2D
   matrices with at least two rows and columns. Three or more outputs and
   broader type/order conformance remain unsupported or unverified.
6. **Type conversions**: Limited automatic type promotion in some cases

### Planned Enhancements

- [ ] N-D NDGRID support (N > 2)
- [ ] Cubic spline interpolation in INTERP1
- [ ] Matrix-valued samples and shape-preserving results in INTERP1
- [ ] GPU acceleration for large array operations
- [ ] Extrapolation options in INTERP1

---

## Compatibility Matrix

| MATLAB Function | XDL Function | Status | Notes |
|----------------|--------------|--------|-------|
| `length()` | `MATLAB_LENGTH()` | ⚠️ Partial | Largest represented dimension for scalars, vectors, rectangular nested arrays, and shaped numeric arrays; ragged arrays error |
| `numel()` | `MATLAB_NUMEL()` | ⚠️ Partial | Counts scalar elements for vectors, rectangular nested arrays, and shaped numeric arrays; ragged arrays error |
| `size()` | `MATLAB_SIZE()` / `MATLAB_SIZE_TAIL()` | ⚠️ Partial | Dimension vector and selected dimension for supported scalar/numeric shapes; multiple outputs supported for known array variables |
| `find()` | `MATLAB_FIND()`; `MATLAB_FIND_ROWS()` / `MATLAB_FIND_COLUMNS()` / `MATLAB_FIND_INDICES()` | ⚠️ Partial | One-output numeric vectors/matrices and two-/three-output forms on rectangular 2D matrices with at least 2 rows and columns; four or more outputs unsupported |
| `atan2(y, x)` | `ATAN(y, x)` | ⚠️ Partial | Same-shape vectors/matrices and scalar expansion are tested; complex inputs and broader implicit expansion are unverified |
| `transpose(A)` / `A.'` | `TRANSPOSE(A)` | ⚠️ Partial | Rectangular nested matrices and row/column vectors transpose; postfix syntax currently requires a named array; conjugating transpose is rejected |
| `linspace()` | `LINSPACE()` | ⚠️ Partial | Common vector generation is implemented; full MATLAB edge-case parity is not established |
| `logspace()` | `LOGSPACE()` | ⚠️ Partial | Common vector generation is implemented; full MATLAB edge-case parity is not established |
| `meshgrid()` | `MESHGRID()` | ⚠️ Partial | Runtime and transpiler use constrained output handling; full two-output MATLAB behavior is not established |
| `ndgrid()` | `NDGRID()` | ⚠️ Partial | 2D only, N-D planned |
| `repmat()` | `REPMAT()` | ⚠️ Partial | Replication implemented; full MATLAB shape and type parity is not established |
| `squeeze()` | `SQUEEZE()` | ⚠️ Partial | Singleton-dimension reduction implemented; full MATLAB shape parity is not established |
| `interp1()` | `INTERP1()` | ⚠️ Partial | Vector samples; linear/nearest; NaN out of range; advanced methods and shape parity remain open |

**Legend:**

- ✅ Full: Complete compatibility with MATLAB
- ⚠️ Partial / known mismatch: scoped support exists, but parity is incomplete or a semantic difference is documented
- ❌ None: Not yet implemented

---

## Examples

### Scientific Computing

```xdl
; Generate frequency response data
freq = LOGSPACE(0, 3, 100)  ; 1 to 1000 Hz
gain = 20 * ALOG10(freq)    ; dB scale

; Create smooth interpolation
x_sparse = [0, 1, 2, 3, 4, 5]
y_sparse = [0, 1, 4, 9, 16, 25]
x_dense = LINSPACE(0, 5, 50)
y_dense = INTERP1(x_sparse, y_sparse, x_dense)
```

### Data Visualization

```xdl
; Create 3D surface data
x = LINSPACE(-2, 2, 50)
y = LINSPACE(-2, 2, 50)
grids = MESHGRID(x, y)
; Extract X and Y from nested array for plotting
```

### Array Manipulation

```xdl
; Replicate pattern
pattern = [1, 0, 1, 0]
tiled = REPMAT(pattern, 5, 3)  ; Create larger pattern

; Remove singleton dimensions
data_4d = FLTARR(10, 1, 20, 1)
data_2d = SQUEEZE(data_4d)  ; Result: 10x20 array
```

---

## Files Modified

### New Files

- `xdl-stdlib/src/matlab_compat.rs` - MATLAB compatibility function implementations

### Modified Files

- `xdl-stdlib/src/lib.rs` - Module declaration and function registration
- `README.md` - Documentation update
- `docs/index.md` - Documentation index update

---

## Build and Installation

The MATLAB compatibility functions are included in standard XDL builds:

```bash
# Build XDL with MATLAB compatibility
cargo build --release

# Run tests
cargo test --lib -p xdl-stdlib matlab_compat

# Install
cargo install --path xdl-cli
```

---

## See Also

- [MATLAB 3D Plot Fix](MATLAB_3D_PLOT_FIX.md) - Details on meshgrid and 3D plotting
- [MATLAB Support](MATLAB_SUPPORT.md) - Overall MATLAB compatibility
- [MATLAB Limitations](MATLAB_LIMITATIONS.md) - Known limitations and workarounds

---

## Summary

XDL provides a partial MATLAB-inspired runtime layer with a separately tested
scope for each function. The transpiler dispatches supported MATLAB names to
these helpers, and unsupported shapes or methods should be documented and
reported rather than treated as full MATLAB parity. The current evidence is
unit and CLI test coverage; it does not establish production readiness or
complete MATLAB/Octave conformance.
