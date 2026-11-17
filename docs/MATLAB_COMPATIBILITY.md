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

**63 MATLAB function names currently map to XDL names:**

**Array Creation:**

- `zeros` → `FLTARR`
- `ones` → `FLTARR` (with special handling)
- `rand`/`randn` → `RANDOMU`/`RANDOMN`

**Math Functions:**

- `sin, cos, tan, exp, log, sqrt, abs, floor, ceil, round`
- Names are mapped to XDL functions; semantics are not uniformly identical.
  For example, general multiple-output `size` and advanced `find` forms remain
  unsupported, while `length` and `numel` use dedicated compatibility functions. See the compatibility
  matrix and open gaps below.

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
- **Control flow**: `if/else/end`, `while/end`, `for/end`

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
- **Function-name mappings** for 63 MATLAB names; a mapping does not assert behavioral parity
- **Index adjustment** for known source variables and function parameters (1-based → 0-based; scalar indices, 2D matrix scalar subscripts, and vector slices with positive constant step; vector `end`, `:`, and omitted bounds)
- **Array dimensions** for single-output `size(A)` and `size(A, dim)` on supported scalar and numeric array shapes
- **Array search** for one-input `find(A)` on numeric vectors and rectangular 2D matrices, returning one-based indices and preserving row-vector orientation
- **Loop conversion** (for, while)
- **Conditional statements** (if/else)
- **Comments** preserved
- **Element-wise operators** converted
- **String literals** handled

### 🚧 Limitations (Future Work)

- **Multidimensional slicing**: MATLAB matrix row/column selections and `:` across dimensions remain unsupported; only scalar subscripts for simple 2D matrix expressions have executable coverage
- **Cell arrays**: `{}` syntax needs implementation
- **Advanced indexing**: MATLAB column-major linear indexing of matrices, multidimensional `end`, logical/index-vector subscripts, and negative-step slices
- **Advanced `find` forms**: limit/direction options and multiple-output row/column subscripts
- **Multiple function outputs**: functions declaring more than one output are rejected with an explicit diagnostic
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

- ✅ **40/40 crate tests passing** (pass rate for this suite, not a language-coverage percentage; verified 2026-11-16)
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
- ✅ Array literals and tested indexing subset: vector scalar/range indexing, vector `end`/omitted bounds, and 2D scalar subscripts; broader MATLAB matrix indexing remains incomplete
- ✅ Function calls (sin, cos, sqrt, etc.)
- ✅ Simple control flow (for, while, if)
- ✅ Direct .m file execution via XDL CLI
- ✅ CLI integration suite: 28 tests passed, including `length`/`numel`, `size(A)`, and `size(A, dim)` cases (verified 2026-11-16)

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
✅ Maps 63 MATLAB function names to XDL names; this does not imply semantic equivalence
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
