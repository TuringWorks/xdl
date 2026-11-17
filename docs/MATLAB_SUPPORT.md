# MATLAB File Support in XDL

## Current status

The XDL CLI detects `.m` files, transpiles the supported subset to XDL, and
executes the result. This path is covered by CLI integration tests. It does not
provide complete MATLAB/Octave language, numerical, toolbox, or project-path
compatibility.

The GUI editor contains a MATLAB-transpilation heuristic for code containing
patterns such as `disp(`, `fprintf(`, or MATLAB `%` comments. Its **File > Open**
dialog currently filters for `.xdl`, and editor execution uses the generic name
`Editor`; direct `.m` file loading through that dialog is therefore not an
established GUI workflow. GUI MATLAB behavior was not verified in the current
compatibility test run.

## CLI usage

```bash
xdl script.m
```

The CLI transpiles the file before running it. To call the transpiler without
executing the result, use the Rust API:

```rust
use xdl_matlab::transpile_matlab_to_xdl;

let xdl_source = transpile_matlab_to_xdl(matlab_source)?;
```

There is no `xdl --transpile` flag.

## Verified subset and limits

The current CLI tests cover selected scalar and array expressions, control
flow, user functions, vector indexing, 2D scalar subscripts, and MATLAB
`length`/`numel` behavior, and `size` dimension queries. The compatibility pages
list exact tested cases and known mismatches. Remaining gaps include general
multiple-output `size`, advanced `find` options and outputs, column-major matrix linear indexing,
multidimensional slices, and reference-output comparisons.

- [Current limitations](MATLAB_LIMITATIONS.md)
- [Detailed MATLAB compatibility map](MATLAB_COMPATIBILITY.md)
- [Selected real-world workflows](MATLAB_REAL_WORLD_SUPPORT.md)
- [Scientific-computing gap analysis](VIBECODER_XDL_SCIENTIFIC_COMPUTING_GAP_ANALYSIS.md)

## Implementation locations

- CLI file detection and execution: `xdl-cli/src/main.rs`
- MATLAB lexer and transpiler: `xdl-matlab/src/`
- MATLAB compatibility helpers: `xdl-stdlib/src/matlab_compat.rs`
- GUI heuristic: `xdl-gui/src/gui.rs`
