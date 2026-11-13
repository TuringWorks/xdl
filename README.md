# XDL - Extended Data Language (Rust)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Rust implementation of GNU Data Language (GDL), providing an IDL-inspired
scientific-computing language, runtime, visualization stack, and an evolving
MATLAB/Octave migration path.

## Overview

XDL is an active-development beta, designed to provide:
- **Memory Safety**: Leveraging Rust's ownership system for safe memory management
- **Performance**: Native Rust kernels where available, with an interpreter at the language boundary
- **Concurrency**: Safe parallelism using Rust's async/await and threading primitives
- **Interoperability**: FFI interfaces and integrations with existing scientific libraries

XDL is not yet a drop-in replacement for MATLAB, GNU Octave, Scilab, or
Simulink. Compatibility and production readiness vary substantially by feature.
For the current capability map and the VibeCoder + XDL product roadmap, see
[the scientific-computing gap analysis](docs/VIBECODER_XDL_SCIENTIFIC_COMPUTING_GAP_ANALYSIS.md).

## Architecture

The project is structured as a Cargo workspace with the following crates:

- **`xdl-core`**: Core data structures, types, and array operations (~1,353 lines)
- **`xdl-parser`**: Lexer and parser for XDL/IDL syntax using nom (~2,176 lines)
- **`xdl-interpreter`**: AST interpreter and execution engine (~1,796 lines)
- **`xdl-runtime`**: Runtime system with memory management
- **`xdl-stdlib`**: Standard library functions (math, I/O, graphics, ML) (~13,199 lines)
- **`xdl-ffi`**: Foreign function interfaces to external libraries
- **`xdl-cli`**: Command-line interface and REPL
- **`xdl-gui`**: Graphical user interface components (~1,963 lines)
- **`xdl-charts`**: Charting and visualization library (~584 lines)
- **`xdl-viz3d`**: 3D visualization engine
- **`xdl-viz3d-web`**: Web-based 3D visualization
- **`xdl-viz3d-threejs`**: Three.js 3D visualization web components
- **`xdl-chart-viewer`**: Interactive chart viewer application
- **`xdl-amp`**: Accelerated Math Processing
- **`xdl-lsp`**: Language Server Protocol implementation for IDE support
- **`xdl-matlab`**: MATLAB/Octave transpiler for a tested basic-to-moderate `.m` subset (~2,950 lines; 28 unit tests)
- **`xdl-artifacts`**: Versioned run-manifest and compatibility-status contracts
- **`xdl-gnc-core`**, **`xdl-gnc-simulation`**, **`xdl-gnc-navigation`**, **`xdl-gnc-guidance`**, **`xdl-gnc-control`**: Typed guidance, navigation, and control libraries (in development; no XDL built-ins yet). See [GNC Crates](docs/development/gnc-crates.md)

**Additional Modules (not in workspace):**
- **`xdl-desktop-viewer`**: Desktop visualization viewer

## Features

### Core Data Types
- All XDL/IDL numeric types (BYTE, INT, LONG, FLOAT, DOUBLE, COMPLEX, etc.)
- N-dimensional arrays with efficient indexing
- Structures and pointers
- String handling

### Language Features
- Variables and expressions
- Control flow: IF/THEN/ELSE, FOR, WHILE, REPEAT, FOREACH, BREAK, CONTINUE, RETURN
- Built-in function and procedure dispatch across math, arrays, I/O, graphics, ML, and scientific domains; the registry is evolving and counts are not a compatibility metric
- **User-defined procedures (PRO/ENDPRO)** ✅
- **User-defined functions (FUNCTION/ENDFUNCTION)** ✅
- **GOTO statements with labels** ✅ NEW
- **CASE/SWITCH statements** ✅ NEW (with ELSE and BEGIN/END blocks)
- Array operations and indexing (multi-dimensional arrays)
- Structure definitions

### Current capability areas
- **Mathematical functions**: SIN, COS, TAN, SINH, COSH, TANH, EXP, ALOG, SQRT, FFT, etc.
- **Special math**: ERF, ERFC, GAMMA, LNGAMMA, FACTORIAL, BESELJ, BESELY, BESELI, BESELK
- **Type conversion**: BYTE, INT, UINT, LONG, ULONG, LONG64, ULONG64, FLOAT, DOUBLE, FIX
- **Array functions**: TRANSPOSE, REFORM, REVERSE, SORT, WHERE, SHIFT, ROTATE, REPLICATE, etc.
- **Array creation**: FINDGEN, INDGEN, FLTARR, DBLARR, RANDOMU, MAKE_ARRAY, etc.
- **Statistics**: MEAN, VARIANCE, STDDEV, MEDIAN, MOMENT, KURTOSIS, LINFIT, POLY_FIT, etc.
- **Interpolation**: INTERPOL, SPLINE, BILINEAR
- **I/O operations**: PRINT, OPENR, OPENW, READF, WRITEF, FILE_TEST, FILE_INFO, etc.
- **File system**: FILE_SEARCH, FILE_MKDIR, FILE_DELETE, FILE_COPY, FILE_MOVE
- **String functions**: STRLEN, STRPOS, STRMID, STRJOIN, STRSPLIT, STRCMP, STRREPLACE, etc.
- **Time functions**: SYSTIME, JULDAY, CALDAT, TIC, TOC
- **Python integration**: PYTHON_IMPORT, PYTHON_CALL, PYTHON_CALL_KW (PyO3 0.27)
- **Graphics and plotting**: 50+ procedures (PLOT, SURFACE, CONTOUR, CHART_*, SURFACE3D, etc.)
- **Machine learning (XDLML)**: 50+ functions (neural networks, K-means, SVM, cross-validation, activation functions, optimizers)
- **Native ML (Linfa)**: ML_KMEANS_*, ML_LINEAR_*, ML_PCA_*, ML_ACCURACY, ML_MSE, ML_R2_SCORE
- **DataFrames (Polars)**: DF_READ_CSV, DF_FILTER, DF_GROUPBY, DF_JOIN, DF_SORT, etc.
- **Linear algebra**: INVERT, DETERM, SVDC, LA_EIGENVAL, LUDC, etc.
- **Image processing**: CONVOL, DILATE, ERODE, SOBEL, GAUSSIAN_FILTER, etc.
- **Signal processing**: A_CORRELATE, C_CORRELATE, DIGITAL_FILTER, HILBERT, etc.
- **MATLAB/Octave transpilation**: Working transpiler for a tested subset of basic to moderate-complexity `.m` files; advanced syntax is not drop-in compatible
- **MATLAB compatibility**: LINSPACE, LOGSPACE, REPMAT, SQUEEZE, NDGRID, INTERP1, MESHGRID
- **GPU acceleration**: XDL-AMP module with multi-backend support
- **IDE support**: Language Server Protocol (LSP) and VS Code extension

## Known Limitations

While XDL provides substantial functionality, the following areas are not yet
complete or uniformly production-ready:

### Language Features
- **Complex numbers**: Partial support with some type conversion issues
- **Advanced array indexing**: Some edge cases in multi-dimensional array slicing

### Compatibility
- **IDL/GDL compatibility**: Coverage is substantial but is not a formally measured or frozen percentage
- **MATLAB/Octave compatibility**: A passing transpiler test suite covers the implemented subset; it does not imply language or toolbox parity
- **Scientific I/O**: Some FITS, HDF5, and NetCDF paths are placeholders or feature-gated and require end-to-end round-trip tests
- **Model-based simulation**: `xdl-model` and `xdl-sim` are foundations, not a Simulink/Xcos-equivalent product
- **Canonical numerics**: Sparse, chunked, units-aware, frame-aware, and distributed data types are incomplete

### Testing & Quality
- **Test runner**: `xdl test` discovers and executes `.xdl` scripts recursively; `--manifest` emits test counts, SHA-256 source artifacts, successful stdout artifacts, and failure diagnostics, while golden vectors, numerical tolerances, and timeouts are not yet implemented
- **Edge cases**: Various edge cases in control flow, type conversions, and array operations
- **Error messages**: Could be more helpful in some scenarios

### Performance
- **Optimization**: Performance tuning is ongoing
- **GPU acceleration**: Implementation depth varies by operation

For regulated, safety-critical, or production workflows, validate every
required feature and numerical result independently. Do not treat a placeholder
I/O or compatibility path as production support.

## Installation

### Prerequisites
- Rust 1.70 or later
- **Python 3.8-3.12** for Python integration features (set `PYO3_PYTHON` environment variable)
- Optional: Scientific libraries for full functionality

### Feature Flags
```bash
# Build with the commonly used optional features (some features require system libraries)
cargo build --features "python,dataframes,ml"

# Build without Python (if Python not available)
cargo build --no-default-features
```

### Building from Source

```bash
git clone https://github.com/TuringWorks/xdl
cd xdl
cargo build --release
```

On macOS, a full workspace build can exhaust the per-process file-descriptor
limit. The Makefile targets therefore default to `ulimit -n 65536`, 2 Cargo
jobs, and no `sccache`. Override with `XDL_CARGO_JOBS`, `XDL_MAX_OPEN_FILES`,
or `XDL_USE_SCCACHE=1`. Plain `cargo build` is unaffected.

### Installing with Cargo

```bash
# Install the CLI interpreter
cargo install --path xdl-cli

# Install the GUI application
cargo install --path xdl-gui
```

### Building Deployable Binaries

To build optimized, standalone binaries that can be distributed and run without Cargo:

```bash
# Build release binaries for all tools
cargo build --release

# The binaries are located at:
# - target/release/xdl        (CLI interpreter)
# - target/release/xdl-gui    (GUI application)

# Copy to a directory in your PATH (e.g., /usr/local/bin on Unix)
# macOS/Linux:
sudo cp target/release/xdl target/release/xdl-gui /usr/local/bin/

# Windows (run as Administrator):
# copy target\release\xdl.exe C:\Windows\System32\
# copy target\release\xdl-gui.exe C:\Windows\System32\
```

After installation, you can run scripts directly:

```bash
# CLI mode (outputs PNG files)
xdl script.xdl

# GUI mode (interactive windows)
xdl-gui script.xdl
```

## Usage

### Command Line

```bash
# Run XDL file
xdl script.pro

# Interactive REPL
xdl

# Execute command directly
xdl -e "print, 'Hello, World!'"

# Parse and show AST
xdl parse script.pro

# Check syntax
xdl check script.pro

# Inspect the current compatibility baseline
xdl compatibility-report --format json

# Measure MATLAB transpilation over a corpus (transpilation only)
xdl compatibility-report --corpus examples/matlab --format json

# Also execute the transpiled XDL (runtime/numerical equivalence still unverified)
xdl compatibility-report --corpus examples/matlab --execute --format json

# Require every corpus case to have a passing declared reference. This is a
# reproducibility gate, not an independent MATLAB/Octave oracle.
xdl compatibility-report --corpus tests/compatibility/matlab --execute --require-reference --format json

# With an adjacent example.m.expected sidecar, verify normalized observed output

# Validate an XDL block-model document
xdl model-check model.json
xdl model-check model.json --svg > model.svg
```

### REPL Commands

```
XDL> print, 'Hello, World!'
XDL> x = findgen(100)
XDL> y = sin(x * !pi / 50)
XDL> .help     ; Show help
XDL> .quit     ; Exit REPL
```

### Python Integration

XDL provides seamless integration with Python. See [docs/PYTHON_INTEGRATION.md](docs/PYTHON_INTEGRATION.md) for detailed documentation.

### Documentation

Comprehensive documentation is available in the `docs/` directory:

- **[Function Reference](docs/FUNCTION_REFERENCE.md)** - Complete list of 220+ functions
- **[LSP & VS Code Extension](docs/LSP_AND_VSCODE.md)** - IDE setup and features
- **[DataFrame Reference](docs/DATAFRAMES_REFERENCE.md)** - Polars DataFrame operations
- **[Linfa ML Reference](docs/LINFA_ML_REFERENCE.md)** - Native Rust ML functions
- **[ML Complete Reference](docs/ML_COMPLETE_REFERENCE.md)** - XDLML neural networks and more
- **[Python Integration](docs/PYTHON_INTEGRATION.md)** - Python interoperability

```xdl
; Import and use Python modules
math_mod = python_import("math")
result = python_call(math_mod, "sqrt", 16.0)
print, "sqrt(16) =", result
```

## Development

### Project Structure

```
xdl/
├── xdl-core/              # Core data types and operations
├── xdl-parser/            # Language parser and AST
├── xdl-interpreter/       # Execution engine
├── xdl-runtime/           # Memory and runtime management
├── xdl-stdlib/            # Standard library functions
├── xdl-ffi/              # External library interfaces
├── xdl-cli/              # Command-line interface
├── xdl-gui/              # Graphical user interface
├── xdl-charts/           # Charting library
├── xdl-viz3d/            # 3D visualization engine
├── xdl-matlab/           # MATLAB transpilation
├── xdl-amp/              # Accelerated Math Processing
├── xdl-chart-viewer/     # Chart viewer app
├── xdl-desktop-viewer/  # Desktop viewer
├── xdl-viz3d-threejs/    # Three.js components
├── xdl-viz3d-web/        # Web 3D visualization
├── xdl-gnc-*/            # Typed GNC libraries (core, simulation, navigation, guidance, control)
├── examples/             # Example scripts and demos
├── tests/                # Test suite
└── docs/                 # Documentation
```

### Building and Testing

```bash
# Check all crates
cargo check --workspace

# Build everything
cargo build --workspace

# Run tests
cargo test --workspace

# Format code
cargo fmt --all

# Lint code
cargo clippy --workspace
```

### Pre-commit Hooks

This project uses pre-commit hooks to ensure code quality:

```bash
# Install pre-commit
pip install pre-commit

# Install hooks
pre-commit install

# Run manually
pre-commit run --all-files
```

The hooks automatically run:
- `cargo fmt --all` - Code formatting
- `cargo check --workspace` - Compilation check
- `cargo clippy --workspace` - Linting
- `cargo test --workspace` - Tests

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and formatting: `cargo fmt --all && cargo test --workspace --no-fail-fast`
5. Run the consistency checks (below) — they catch what the compiler cannot
6. Submit a pull request

### Read this first

XDL's built-in dispatch is a 754-arm `match` on a string, and the list of
built-in names is maintained by hand in four places. **Most ways to break XDL
produce no compile error.** Before your first change, read:

| File | What it is |
|---|---|
| [AGENTS.md](./AGENTS.md) | The authoritative guide: product matrix, change-surface cookbook, functional style, IDL semantics, performance, verification, test isolation |
| [CLAUDE.md](./CLAUDE.md) | The short entry point, including the cross-cutting change checklist |
| [SOUL.md](./SOUL.md) | What XDL is for, and why the compatibility contract is not negotiable |
| [design-system/](./design-system/README.md) | The shared visual contract across the four render backends |
| [llms.txt](./llms.txt) | Machine-readable language and repo reference |

These apply to human and AI contributors alike. If you use Claude Code, the
project's hooks and the `add-builtin` skill live in `.claude/` and load
automatically.

### Consistency checks

These enforce the contracts nothing in `cargo` checks. CI runs all of them.

```bash
scripts/check-builtin-parity.sh   # stdlib dispatch <-> LSP <-> editor grammar
scripts/check-version-sync.sh     # Cargo.toml <-> tauri.conf.json <-> package.json
scripts/check-docs-links.sh       # relative links AND heading anchors
scripts/run-xdl-suite.sh          # the .xdl language suite (allowlist ratchet)
scripts/run-xdl-suite.sh --survey # every .xdl script, pass/fail/hang
```

### Adding a built-in function

Six files must agree, and only one of them is compiler-checked. The ordered
walk-through is in
[.claude/skills/add-builtin/SKILL.md](./.claude/skills/add-builtin/SKILL.md);
the summary is in [CLAUDE.md](./CLAUDE.md).

### Code Style

- Follow Rust naming conventions; `cargo fmt` for formatting
- Address `cargo clippy` warnings (CI runs `-D warnings`)
- **Write in a functional style** — pure functions over immutable data, effects
  at the edges, total error handling. See
  [AGENTS.md → Functional Style](./AGENTS.md#functional-style--safe-refactoring)
- **No `.unwrap()` / `.expect()` / `panic!` in library, interpreter, or stdlib
  paths.** A panic there kills the user's whole session and loses their loaded
  data. Return `XdlResult` and propagate with `?`
- **Never substitute a plausible default for missing or failed numeric data.**
  This is a data-analysis language; a wrong-but-plausible number can survive peer
  review, while an error gets fixed the same afternoon
- Add tests for new functionality — in `.xdl`, at the boundary, the way a user
  calls it — and **watch the test fail before you count it as done**
- Document public APIs

## Roadmap

### Phase 1: Foundation ✅
- [x] Core data structures and types
- [x] Complete lexer/parser implementation
- [x] Expression evaluator and statement execution
- [x] Variable and scope management
- [x] Function/procedure calls
- [x] Project structure and build system
- [x] CLI interface and REPL

### Phase 2: Standard Library ✅
- [x] Mathematical functions
- [x] Array manipulation and indexing
- [x] I/O operations and file handling
- [x] String functions
- [x] Graphics and plotting (2D/3D)
- [x] Machine learning functions
- [x] Python 3.13 integration

### Phase 3: Advanced Features ✅
- [x] Graphics and plotting (WebGL, Three.js)
- [x] External library integration
- [x] MATLAB transpilation support (basic to moderate)
- [x] GPU acceleration (XDL-AMP)
- [x] Interactive chart viewers
- [x] 3D visualization engines

### Phase 4: User Interfaces ✅
- [x] GUI components (egui-based)
- [x] Desktop viewer applications
- [x] Web-based visualization
- [x] Interactive chart viewers

### Phase 5: Compatibility, reproducibility, and optimization 🚧 **(Current Focus)**
- [ ] Publish generated IDL/GDL and MATLAB/Octave conformance reports
- [ ] Replace placeholder scientific-I/O paths with tested readers/writers
- [x] Basic recursive `.xdl` script test runner with pass/fail exit status
- [ ] Scientific test evidence, replayable run manifests, and artifact capture
- [x] Legacy code migration foundation
- [x] Documentation and examples
- [x] User-defined procedures (PRO/ENDPRO) ✅
- [x] User-defined functions (FUNCTION/ENDFUNCTION) ✅
- [ ] Complete complex number support
- [x] GOTO and label support
- [ ] Performance optimization
- [ ] Extended edge case handling
- [ ] Comprehensive numerical, workflow, and artifact coverage

### Phase 6: Model-based scientific engineering 🚧
- [ ] Stable typed block/signal graph schema
- [ ] Deterministic fixed-step, multi-rate, continuous, and hybrid simulation
- [ ] Scopes, replay, model diagnostics, and golden-vector testing
- [ ] FMU import/export and native Rust/C execution profiles
- [ ] VibeCoder scientific workspace and model editor integration

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Original GDL team and contributors
- The Rust community
- Scientific computing libraries (GSL, NetCDF, HDF5, etc.)
- TuringWorks development team

## Project Status

🚧 **Active Development — Beta** 🚧

XDL is a functional beta implementation with substantial language support and
several experimental or partially implemented scientific-computing surfaces.
The following is a capability summary, not a claim of toolbox parity:

- **Core language**: Variables, expressions, control flow (FOR, WHILE, FOREACH, IF/THEN/ELSE, CASE/SWITCH)
- **Standard library**: Math, arrays, statistics, I/O, graphics, ML, dataframes, and domain functions; registry counts change as coverage is audited
- **ML**: Neural networks, K-means, SVM, cross-validation, optimizers, and native Linfa-backed paths with uneven depth
- **Native ML (Linfa)**: K-Means, Linear/Logistic Regression, PCA (pure Rust)
- **DataFrames (Polars)**: High-performance data manipulation (10-100x faster than Pandas)
- **50+ graphics procedures**: 2D/3D plotting, charting, interactive visualization
- **Python integration**: PyO3 0.27 integration with PYTHON_IMPORT/PYTHON_CALL
- **IDE support**: Language Server Protocol (LSP) + VS Code extension
- **MATLAB transpiler**: Basic-to-moderate `.m` support with a passing unit-test subset; array/object/toolbox/file-I/O parity remains incomplete
- **GPU acceleration**: XDL-AMP module with multi-backend support (MLX, Metal, CUDA, etc.)
- **Multiple interfaces**: CLI/REPL, GUI (egui), web-based viewers
- **150+ examples**: Demonstrating features across basics, ML, visualization, and scientific computing

**Compatibility**: Core IDL/GDL features are implemented, including
user-defined procedures and functions, GOTO/labels, and CASE/SWITCH. The
project does not currently publish a reproducible formal compatibility
percentage; see Known Limitations and the [gap analysis](docs/VIBECODER_XDL_SCIENTIFIC_COMPUTING_GAP_ANALYSIS.md)
for the remaining gaps.

Active development continues on improving edge case handling and performance optimization. Contributions and feedback are welcome!
