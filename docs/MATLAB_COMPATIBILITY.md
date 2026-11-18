# MATLAB Compatibility Layer - Implementation Summary

## What Was Built

We've implemented a **MATLAB to XDL compatibility layer** that enables loading
and executing a tested subset of MATLAB `.m` files in XDL. It is a migration
foundation, not drop-in MATLAB or toolbox compatibility.

### ✅ Completed Components

#### 1. **MATLAB Lexer** (`xdl-matlab/src/lexer.rs`)

- Tokenizes MATLAB syntax
- Handles `%` comments
- Supports element-wise operators (`.*, ./, .^`)
- Processes strings with `'` delimiters
- Line continuation with `...`
- 500+ lines of robust lexing code

#### 2. **Function Mapping Table** (`xdl-matlab/src/function_map.rs`)

**68 MATLAB function names currently map to XDL names:**

**Array Creation:**

- `zeros` → `FLTARR`
- `ones` → `FLTARR` (with special handling)
- `rand`/`randn` → `RANDOMU`/`RANDOMN`

**Math Functions:**

- `sin, cos, tan, exp, log, sqrt, abs, floor, ceil, round`
- Names are mapped to XDL functions; semantics are not uniformly identical.
  For example, `size` multiple outputs are implemented for known array
  variables, while expression/object shapes remain unsupported. Three-output
  `find` returns nonzero values for the tested numeric shapes; broader type and
  MATLAB-oracle coverage remains open. `length` and `numel` use dedicated
  compatibility functions. See the compatibility matrix and open gaps below.

**Statistics:**

- `mean, median, std, var, min, max, sum`

**Linear Algebra:**

- `transpose, inv, det`

**Plotting:**

- `plot, xlabel, ylabel, title, figure, hold, clf`

**And more...**

#### 3. **MATLAB to XDL Transpiler** (`xdl-matlab/src/transpiler.rs`)

**Syntax Conversions:**

- **Comments**: `%` → `;`
- **Indexing**: 1-based `x(1)` → 0-based `x[0]`
- **Loops**: `for i = 1:10` → `for i = 0, 9`
- **Element-wise ops**: `.*` → `*`, `./` → `/`, `.^` → `^`
- **Functions**: Maps MATLAB function names to XDL
- **Control flow**: `if/elseif/else/end`, `while/end`, `for/end`

**Example Transpilation:**

```matlab
% MATLAB Input
x = zeros(10, 1);
for i = 1:10
    x(i) = sin(i * 0.1);
end
mean_x = mean(x);
```

```idl
; XDL Output
x = FLTARR ( 10 , 1 )
for i = 0, 9
  x [ i ] = SIN ( i * 0.1 )
endfor
mean_x = MEAN ( x )
```

## Architecture

```text
┌─────────────────┐
│   .m file       │
│  (MATLAB code)  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  MATLAB Lexer   │  Tokenize MATLAB syntax
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Transpiler    │  Convert to XDL syntax
│                 │  - Adjust indexing (1→0)
│                 │  - Map functions
│                 │  - Convert operators
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   XDL Code      │  Ready for execution
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ XDL Interpreter │  Execute the code
└─────────────────┘
```

## Usage Examples

### 1. As a Library

```rust
use xdl_matlab::transpile_matlab_to_xdl;

let matlab = "x = zeros(10); y = sin(x);";
let xdl = transpile_matlab_to_xdl(matlab)?;
```

### 2. Load .m Files

```rust
use xdl_matlab::load_matlab_file;
use std::path::Path;

let xdl_code = load_matlab_file(Path::new("script.m"))?;
// Execute with XDL interpreter
```

### 3. XDL CLI Integration

```bash
# Load and execute MATLAB files directly
xdl script.m
```

The CLI has no `--transpile` flag. To obtain transpiled source without
execution, call `xdl_matlab::transpile_matlab_to_xdl` from Rust as shown above.

## Key Features

### ✅ What Works

- **Syntax transpilation** for common constructs
- **Colon vectors** for whole expressions, `start:stop` and `start:step:stop`, with literal ranges bounded at transpile time and runtime-valued bounds validated as finite real scalars (including one-element numeric arrays); ranges are inclusive, direction-aware, and limited to 1,000,000 elements. Colon expressions inside subscripts remain on the separate MATLAB indexing translation path.
- **Postfix transpose** for nonconjugating `A.'` and conjugating `A'` on directly named arrays. `A'` supports real numeric scalars, vectors, and 2D arrays, plus conjugation of a complex scalar; complex arrays and compound-expression transpose remain unsupported
- **Function-name mappings** for 69 MATLAB names; a mapping does not assert behavioral parity
- **Index adjustment** for known source variables and function parameters, including one-based MATLAB linear scalar reads, 2D scalar subscripts, vector slices with positive constant step, vector `end`, `:`, and omitted bounds
- **2D matrix slice reads** for rectangular numeric matrices: `A(i,j:k)`, `A(i:j,k)`, `A(:,k)`, and `A(i,:)`, preserving row (`1×N`) versus column (`N×1`) result shape
- **Linear element reads** for one scalar subscript on tested numeric row/column vectors, rectangular matrices, and shaped numeric arrays through `MATLAB_INDEX`
- **Vector index reads** for tracked numeric arrays through `MATLAB_INDEX_VECTOR`; use MATLAB column-major order, preserve requested indices and duplicates, and accept empty index vectors. Flat XDL results cannot retain MATLAB row/column result orientation.
- **Array dimensions** for single-output and multiple-output `size(A)` on supported scalar and numeric array variables, plus `size(A, dim)`
- **Array reshape** for numeric scalars, vectors, rectangular nested arrays, and shaped arrays with explicit nonnegative integer dimensions via `reshape(A,d1,d2,...)`, preserving MATLAB column-major element order in shaped XDL arrays; target size is capped at 16,777,216 elements
- **Array search** for one-output `find(A[,k[,direction]])` on tested numeric vectors and matrices; two-output `[row,col]` on vectors and rectangular matrices; and three-output `[row,col,v]` returning nonzero values, with optional `k`/direction. Shaped and nested N-D runtime cases use MATLAB column-major traversal; `.m` end-to-end fixtures cover vectors, 2D matrices, and a 2×2×2 array created with `reshape`.
- **CSV import** for rectangular numeric files through transpiled `readmatrix(filename)`, optionally skipping an explicit `'NumHeaderLines', n`; automatic header detection, mixed-type inference, and other import options remain unsupported
- **Finite-value check** through `isfinite`, using XDL `FINITE` for supported real-valued scalar, vector, and shaped-array inputs; complex and MATLAB-oracle parity remain unverified
- **Non-finite values**: case-insensitive `NaN` and `Inf` constants lower to explicit IEEE-754 runtime values; unary `-Inf` produces negative infinity. `isnan`, `isinf`, and `isfinite` use XDL scalar/array classifiers. Complex-array and independent MATLAB/Octave oracle parity remain unverified.
- **Matrix left division** for direct named-array operands on finite real-valued square and rectangular systems with vector or matrix RHS; square singular systems error, rectangular systems use SVD least-squares/minimum-norm solving, while sparse, complex, and elementwise left division remain unsupported
- **Loop conversion** (for, while)
- **Conditional statements** (if/elseif/else)
- **Comments** preserved
- **Element-wise operators** converted
- **String literals** handled

### 🚧 Limitations (Future Work)

- **Multidimensional slicing**: only the listed 2D read forms are supported for rectangular numeric matrices. Two-axis block writes support scalar expansion or a rectangular matrix with the selected shape. Ragged or nonnumeric nested rows are rejected explicitly; higher-dimensional slices remain unsupported.
- **Cell arrays**: `{}` syntax needs implementation
- **Advanced indexing**: matrix index arrays, nested-array index expressions, single-subscript matrix assignment, multidimensional `end`, logical subscripts, and negative-step slice assignment. Row/column range writes are tested when the other subscript is scalar; two-axis block writes accept scalar expansion or a rectangular matrix matching the selected block shape. Gather outputs are flattened because XDL does not retain index-vector orientation.
- **Advanced `find` forms**: four or more output subscripts, nonnumeric inputs, and complex/sparse arrays
- **Data import**: `readmatrix` accepts a filename and strict numeric CSV, optionally with `'NumHeaderLines', n`; automatic header detection, `detectImportOptions`, mixed types, dates, and other file formats are not supported
- **Multiple function outputs**: functions declaring more than one output are rejected with an explicit diagnostic; `[d1,...,dn]=size(A)` is a supported assignment special case
- **MEX files**: Binary extension support
- **Classes/OOP**: MATLAB classes not supported
- **Anonymous functions**: `@(x) x^2` syntax
- **Toolbox functions**: Need per-toolbox mapping

## Testing

### Unit Tests ✅

The crate includes focused unit tests:

```bash
cargo test -p xdl-matlab
```

**Test Results:**

- ✅ **53/53 crate tests passing** (pass rate for this suite, not a language-coverage percentage; verified 2026-11-17)
- **Coverage**: Lexer, transpiler, function mapping, control flow, arrays, math

### Integration Tests ✅

MATLAB .m files can be executed directly:

```bash
# Execute MATLAB files directly
xdl examples/matlab/01_simple_math.m
xdl examples/matlab/02_trigonometry.m

# Run the selected MATLAB execution fixtures
./tests/test_matlab_execution.sh
```

**Verified Working:**

- ✅ Basic arithmetic and variables
- ✅ Array literals and tested indexing subset: whole-expression colon vectors accept runtime-valued finite scalar bounds and steps; vector scalar/range reads and vector-index gathers preserve linear selection order and duplicates for numeric vectors, nested matrices, and shaped arrays; vector `end`/omitted bounds, 2D scalar subscripts, rectangular numeric 2D reads `A(i,j:k)`, `A(i:j,k)`, `A(:,k)`, and `A(i,:)` preserve row/column slice shape; row-vector scalar/range writes, matrix row/column range writes with a scalar or matching-length numeric vector, two-subscript scalar matrix writes, and two-axis block writes with scalar expansion or a shape-matched rectangular matrix; vector-index result orientation, matrix-valued index arrays, nested-array index expressions, logical subscripts, single-subscript matrix writes, ragged matrix slice reads, and higher-dimensional slicing remain incomplete
- ✅ Function calls (sin, cos, sqrt, etc.)
- ✅ Simple control flow (for, while, if/elseif/else)
- ✅ Direct .m file execution via XDL CLI
- ✅ CLI integration suite: 35 tests passed, including `length`/`numel`, single- and multiple-output `size`, `size(A, dim)`, one- and two-output `find`, scalar matrix linear reads, rectangular 2D matrix range reads, scalar and block matrix writes, nonconjugating transpose, and elementwise `atan2` (verified 2026-11-16)
- ✅ Separate `.m` transpose fixture executes `A'` on a real 2×3 matrix and verifies the 3×2 dimensions and corner values (verified 2026-11-17)
- ✅ Separate multi-output `find` integration test verifies vector, 2D matrix, and reshaped 3D row/column outputs, three-output nonzero values, bounded first/last selection, and empty results (verified 2026-11-17)
- ✅ CSV integration tests: `readmatrix` imports headerless CSV and explicitly skips two header lines, reads/indexes values, selects columns, and produces a non-empty `xdl_plot.png` (verified 2026-11-17)

## Files Created

```text
xdl-matlab/
├── src/
│   ├── lib.rs              # Main library interface
│   ├── lexer.rs            # MATLAB tokenizer (500+ lines)
│   ├── transpiler.rs       # Syntax converter (400+ lines)
│   └── function_map.rs     # Function mappings (100+ lines)
├── Cargo.toml              # Crate configuration
└── README.md               # Documentation

examples/
├── test_matlab.m           # Sample MATLAB file
└── test_matlab_transpiler.rs  # Demo program
```

## Benefits

1. **Access to MATLAB ecosystem**: Run existing .m files
2. **Migration path**: Gradual transition from MATLAB to XDL
3. **Interoperability**: Mix MATLAB and XDL code
4. **Learning tool**: Understand syntax differences
5. **Extensibility**: Easy to add more function mappings

## Next Steps

To move from the current compatibility foundation toward dependable migration:

1. **Conformance**: Maintain a generated syntax/function pass-fail matrix
2. **Diagnostics**: Preserve source maps and report unsupported semantics
3. **More functions**: Expand mappings only with behavioral tests
4. **Advanced syntax**: Matrix operations, slicing, cells, structs, and handles
5. **Testing**: Run a larger real MATLAB/Octave corpus and independent references
6. **Documentation**: Keep examples explicit about supported subset versus parity

## Performance

- **Lexing**: Fast, single-pass tokenization
- **Transpilation**: Lightweight AST-based conversion
- **No extra MATLAB runtime layer**: Transpiles to XDL; execution still uses
  the XDL interpreter or selected native backends

## Maintenance

Adding new function mappings is simple:

```rust
// In function_map.rs
map.insert("newfunc", "XDL_EQUIV");
```

Extending syntax support:

```rust
// In transpiler.rs
TokenKind::NewSyntax => self.transpile_new_syntax()?
```

## Conclusion

We've successfully built a foundational **MATLAB compatibility layer** for XDL that:

✅ Lexes MATLAB syntax
✅ Transpiles to XDL code
✅ Maps 68 MATLAB function names to XDL names; this does not imply semantic equivalence
✅ Handles basic control flow
✅ Adjusts array indexing
✅ Preserves comments

This provides a **solid foundation** for MATLAB/Octave compatibility and can be extended incrementally to support more advanced features.

---

**Status**: ✅ Core infrastructure exists and is exercised by a focused test
subset
**Next**: Expand conformance coverage, improve source maps and diagnostics, and
validate CLI/GUI execution on a larger real-world corpus. This is not seamless
MATLAB compatibility or toolbox parity.
