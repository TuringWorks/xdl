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
- two scalar subscripts on simple 2D matrix literals;
- MATLAB `length` and `numel` for scalars, row/column vectors, rectangular
  nested matrices, and shaped numeric arrays;
- MATLAB `size(A)` and `size(A, dim)` for supported scalar, vector, rectangular
  matrix, and shaped numeric values;
- direct `.m` execution through the XDL CLI.

Verification snapshot (2026-11-16):

| Command | Result | Scope |
|---|---:|---|
| `cargo test -p xdl-matlab` | 40 passed | Transpiler, mapping, lexer, and selected syntax tests |
| `cargo test -p xdl-cli --test cli_tests` | 29 passed | CLI smoke/regression tests, including the MATLAB cases above |
| `cargo test --lib -p xdl-stdlib matlab_compat` | 15 passed | Focused MATLAB helper tests |

These counts are test-suite results, not percentages of MATLAB compatibility.

## Known semantic gaps

- **Matrix indexing:** MATLAB column-major linear indexing `A(k)` is not
  implemented for matrix literals represented as nested rows. Multidimensional
  slices, multidimensional `end`, logical/index-vector subscripts, and negative
  step slices are also unsupported or unverified. The tested `A(i,j)` scalar
  case does not cover those forms.
- **Array information:** General multiple-output forms such as `[m,n] = size(A)`
  are not implemented; nonnumeric/object-array size semantics are not verified.
- **Search:** One-input `find(A)` supports tested numeric vectors and
  rectangular matrices through `MATLAB_FIND`, returning one-based linear
  indices. Row vectors preserve row orientation; column vectors and matrices
  return a column. Empty 0×0 input and scalar zero produce 0×0 output. Options,
  multiple outputs, and nonnumeric types remain unsupported or unverified.
- **Multiple values and types:** General multiple-output assignment, cell arrays,
  structs, classes, and anonymous functions are not supported as MATLAB
  equivalents. Multiple-output functions are rejected explicitly.
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
