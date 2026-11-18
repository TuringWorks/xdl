# MATLAB Compatibility: Current Limits

XDL can transpile and execute a tested subset of MATLAB `.m` files. A mapped
function name or a successful run does not establish MATLAB/Octave numerical or
language parity. Validate each migration against reference outputs and rendered
artifacts from the source workflow.

## Verified subset

The current local checks cover:

- scalar expressions, common math calls, array literals, selected control flow,
  and a single-output user function;
- vector scalar and range subscripts for known arrays and function parameters,
  including vector `end`, `:`, and omitted range bounds;
- row-vector scalar/range writes, matrix row/column range writes when the other
  subscript is scalar, and two-subscript scalar matrix writes;
- two scalar subscripts on simple 2D matrix literals;
- rectangular numeric 2D slice reads `A(i,j:k)`, `A(i:j,k)`, `A(:,k)`, and
  `A(i,:)`, preserving row-versus-column result shape;
- nonconjugating transpose `A.'` and real-array conjugating syntax `A'` for a
  directly named array; complex scalar conjugation is supported, while complex
  arrays and compound-expression transpose are rejected or unsupported;
- MATLAB `length` and `numel` for scalars, row/column vectors, rectangular
  nested matrices, and shaped numeric arrays;
- MATLAB `size(A)` and `size(A, dim)` for supported scalar, vector, rectangular
  matrix, and shaped numeric values;
- direct `.m` execution through the XDL CLI;
- case-insensitive `NaN`/`Inf` constants in scalar expressions and numeric array literals, including unary `-Inf`; `isnan`, `isinf`, and `isfinite` scalar/vector classification of NaN and ±Inf is CLI-tested, while shaped-array, complex, and external MATLAB/Octave oracle parity remain unverified;
- rectangular numeric CSV import through `readmatrix(filename)`, with optional explicit `'NumHeaderLines', n`; automatic detection, mixed types, and other import options remain unsupported;
- two-argument `atan2(y, x)` for equal-shape vectors, shaped arrays, and
  rectangular nested matrices, with numeric scalar expansion.
- named-array `A \ b` for finite real-valued square or rectangular 2D matrices with
  vector or matrix right-hand sides; rectangular solving uses SVD least-squares/
  minimum-norm semantics. Sparse/complex systems, expression operands, and
  elementwise `.\` remain unsupported.

Verification snapshot (2026-11-17):

| Command | Result | Scope |
|---|---:|---|
| `cargo test -p xdl-matlab` | 59 passed (verified 2026-11-17) | Includes compile-time bounded literal and runtime-valued whole-expression colon vectors, vector-index lowering while preserving subscript range semantics, case-insensitive NaN/Inf lowering in scalar expressions and numeric array literals, plus the previously listed syntax and lowering cases |
| `cargo test -p xdl-interpreter` | 41 passed | Interpreter behavior including rectangular 2D range reads, shape preservation, ragged-matrix rejection, and shape-checked matrix-block writes |
| `cargo test -p xdl-cli --test cli_tests` | 35 passed | CLI regressions including MATLAB matrix range reads, scalar and block writes, scalar expansion, nonconjugating transpose, and elementwise/scalar-expanded `atan2` |
| `cargo test -p xdl-cli --test matlab_readmatrix` | 2 passed (verified 2026-11-17) | `.m` imports numeric CSV with and without explicit header skipping, indexes it, selects columns, and creates a non-empty native PNG plot |
| `cargo test -p xdl-cli --test matlab_isfinite` | 1 passed (verified 2026-11-17) | `.m` verifies NaN and ±Inf constants with scalar `isnan`/`isinf`/`isfinite`, plus vector classification masks |
| `cargo test -p xdl-cli --test matlab_find_outputs` | 1 passed | `.m` verifies two-output row/column subscripts for a 2D matrix, row vector, and 2×2×2 reshaped array; three-output values; column-major order; optional limits/directions; and empty output dimensions |
| `cargo test -p xdl-cli --test matlab_elseif` | 1 passed | `.m` executes `if/elseif/else` for each branch choice and verifies exactly the selected branch prints |
| `cargo test -p xdl-cli --test matlab_mldivide` | 1 passed | `.m` solves square and rectangular systems with vector and matrix right-hand sides |
| `cargo test -p xdl-cli --test matlab_ctranspose` | 1 passed | `.m` conjugating-transposes a real 2×3 matrix, checking 3×2 output dimensions and corner values |
| `cargo test -p xdl-cli --test matlab_colon_ranges` | 1 passed | `.m` executes ascending, descending stepped, and direction-mismatch empty literal colon vectors, checking values and size |
| `cargo test -p xdl-cli --test matlab_dynamic_colon` | 1 passed | `.m` executes variable-bound, variable-step, descending, and direction-mismatch empty ranges, checking values and size |
| `cargo test -p xdl-cli --test matlab_vector_indexing` | 1 passed | `.m` gathers by variable and literal index vectors, preserving order and duplicates; an empty colon index returns an empty selection |
| `cargo test --lib -p xdl-stdlib matlab_compat` | 54 passed (verified 2026-11-17) | Focused MATLAB helper tests, including bounded runtime colon vectors and vector-index selection/error cases, IEEE NaN/Inf values, vector/N-D `find`, 3D MATLAB-order reshape, three-output nonzero values, left division, checked transpose, and numeric CSV import/header validation |

These counts are test-suite results, not percentages of MATLAB compatibility.

## Known semantic gaps

- **Matrix indexing:** Scalar one-subscript reads `A(k)` are routed through
  `MATLAB_INDEX` for tested numeric matrices and use MATLAB column-major order.
  Numeric array reads `A(P)` use `MATLAB_INDEX_VECTOR`, preserving requested
  order and duplicates while checking all one-based indices. It reads linear
  index vectors from vectors, rectangular nested matrices, and shaped numeric
  arrays in column-major order. Results are flattened because XDL cannot retain
  MATLAB row/column result orientation. Logical indexing and indexed assignment
  are not supported.
  Rectangular numeric 2D row/column slice reads and writes with one scalar
  subscript have focused CLI execution coverage. Two-axis block writes support
  scalar expansion or a rectangular matrix matching the selected shape.
  Nested-array index expressions, matrix-valued index arrays, single-subscript
  matrix writes, higher-dimensional slices, multidimensional `end`, and
  negative-step slices remain unsupported or unverified. Two-subscript scalar
  reads continue to use the separately tested `A(i,j)` path. A scalar subscript
  embedded directly in `disp(A(k))` is covered for scalar vector and 2D matrix
  reads. Subscript expressions nested in other function arguments remain
  unverified.
- **Array information:** Multiple-output `[d1,...,dn] = size(A)` is supported
  when `A` is a known array variable. General expression, object, and
  nonnumeric-array size semantics are not verified.
- **Search:** `find(A)`, `find(A,k)`, and `find(A,k,'first'|'last')` support
  tested numeric vectors and rectangular matrices through `MATLAB_FIND`,
  returning one-based linear indices. Nonzero scalar input returns `[1]`;
  scalar zero and empty 0×0 input produce 0×0 output. Row vectors preserve row
  orientation; column vectors and matrices return a column. Two-output
  `[row,col]=find(A[,k[,direction]])` supports vector and matrix inputs.
  Shaped/nested N-D runtime inputs use MATLAB column-major traversal and
  collapse dimensions 2..N into `col`. Three-output `[row,col,v]` returns
  nonzero values in `v`, not linear indices. Positive integer `k` and
  `'first'`/`'last'` selection are supported. N-D `.m` fixtures, four or more
  output subscripts, and nonnumeric/complex/sparse types remain unsupported or
  unverified.
- **Multiple values and types:** General multiple-output assignment (apart from
  the `size(A)` special case), cell arrays, structs, classes, and anonymous
  functions are not supported as MATLAB equivalents. Multiple-output user
  functions are rejected explicitly.
- **Interpolation:** `INTERP1` currently supports finite, strictly monotonic
  1D sample vectors with linear or nearest interpolation and NaN outside the
  sample domain. Matrix-valued samples, shape preservation, cubic/spline methods,
  and explicit extrapolation options remain open.
- **Randomness and signal processing:** `rand`/`randn` map to `RANDOMU`/`RANDOMN`,
  and FFT exists, but MATLAB seed, shape, complex-number, and numerical
  conformance have not been established for general workflows.
- **Graphics and I/O:** Plot calls and selected layout commands have mappings or
  special handling. Surface plotting, style/property parity, GUI behavior,
  scientific file formats, and complete workflow outputs need separate tests.
- **Toolboxes and project loading:** Toolbox coverage, multi-file function
  lookup, package/path behavior, and MEX compatibility are not provided as a
  general MATLAB environment.

## Migration guidance

Run the exact `.m` files needed by the project through XDL, then compare
intermediate numerical values, error behavior, and plots with MATLAB or Octave
reference results. Keep unsupported constructs visible and test them before
relying on downstream calculations. For the detailed feature map and current
evidence, see [MATLAB compatibility](MATLAB_COMPATIBILITY.md),
[real-world support](MATLAB_REAL_WORLD_SUPPORT.md), and the
[scientific-computing gap analysis](VIBECODER_XDL_SCIENTIFIC_COMPUTING_GAP_ANALYSIS.md).
