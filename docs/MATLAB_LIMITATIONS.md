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
- nonconjugating transpose `A.'` for a directly named array; conjugating
  transpose and compound-expression transpose are rejected with diagnostics;
- MATLAB `length` and `numel` for scalars, row/column vectors, rectangular
  nested matrices, and shaped numeric arrays;
- MATLAB `size(A)` and `size(A, dim)` for supported scalar, vector, rectangular
  matrix, and shaped numeric values;
- direct `.m` execution through the XDL CLI;
- headerless rectangular numeric CSV import through `readmatrix(filename)`;
- two-argument `atan2(y, x)` for equal-shape vectors, shaped arrays, and
  rectangular nested matrices, with numeric scalar expansion.

Verification snapshot (2026-11-16):

| Command | Result | Scope |
|---|---:|---|
| `cargo test -p xdl-matlab` | 50 passed | Transpiler, mapping, lexer, matrix-range and block-write lowering, two-output `find`, postfix nonconjugating transpose, negative array literals, plot-label lowering, and selected syntax tests |
| `cargo test -p xdl-interpreter` | 41 passed | Interpreter behavior including rectangular 2D range reads, shape preservation, ragged-matrix rejection, and shape-checked matrix-block writes |
| `cargo test -p xdl-cli --test cli_tests` | 35 passed | CLI regressions including MATLAB matrix range reads, scalar and block writes, scalar expansion, nonconjugating transpose, and elementwise/scalar-expanded `atan2` |
| `cargo test -p xdl-cli --test matlab_readmatrix` | 1 passed | `.m` imports numeric CSV, indexes it, reads dimensions, selects columns, and creates a non-empty native PNG plot |
| `cargo test -p xdl-cli --test matlab_find_outputs` | 1 passed | `.m` verifies two-output `find` row/column indices in column-major order and empty output dimensions |
| `cargo test --lib -p xdl-stdlib matlab_compat` | 29 passed | Focused MATLAB helper tests, including one- and two-output `find` with optional limits/direction and numeric CSV import errors |

These counts are test-suite results, not percentages of MATLAB compatibility.

## Known semantic gaps

- **Matrix indexing:** Scalar one-subscript reads `A(k)` are routed through
  `MATLAB_INDEX` for tested numeric matrices and use MATLAB column-major order.
  Rectangular numeric 2D row/column slice reads and writes with one scalar
  subscript have focused CLI execution coverage. Two-axis block writes support
  scalar expansion or a rectangular matrix matching the selected shape.
  Nested-array range sources, vector-valued index expressions, single-subscript matrix
  writes, higher-dimensional slices, multidimensional `end`, and
  negative-step slices remain unsupported or unverified. Two-subscript scalar
  reads continue to use the separately tested `A(i,j)` path. A scalar subscript
  embedded directly in `disp(A(k))` is covered for scalar vector and 2D matrix
  reads. Subscript expressions nested in other function arguments remain
  unverified.
- **Array information:** Multiple-output `[d1,...,dn] = size(A)` is supported
  when `A` is a known array variable. General expression, object, and
  nonnumeric-array size semantics are not verified.
- **Search:** `find(A)`, `find(A,k)`, and `find(A,k,'first'|'last')` support
  tested numeric vectors and
  rectangular matrices through `MATLAB_FIND`, returning one-based linear
  indices. Nonzero scalar input returns `[1]`; scalar zero and empty 0×0 input
  produce 0×0 output. Row vectors preserve row orientation; column vectors and
  matrices return a column. Positive integer `k` and `'first'`/`'last'` are
  Two-output `[row,col]=find(A[,k[,direction]])` and three-output
  `[row,col,linear]=find(A[,k[,direction]])` forms return one-based column
  vectors in MATLAB column-major nonzero order for rectangular numeric 2D
  matrices with at least two rows and columns. Positive integer `k` and
  `'first'`/`'last'` selection are supported. Two-output vector cases, four or
  more outputs, higher-dimensional inputs, and nonnumeric types remain
  unsupported or unverified.
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
