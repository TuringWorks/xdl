# XDL - Extended Data Language (Rust)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A modern Rust implementation of Extended Data Language (XDL), providing IDL-compatible data analysis and visualization capabilities.

## Overview

This is a complete rewrite of XDL in Rust, designed to provide:
- **Memory Safety**: Leveraging Rust's ownership system for safe memory management
- **Performance**: Native code performance with zero-cost abstractions
- **Concurrency**: Safe parallelism using Rust's async/await and threading primitives
- **Interoperability**: FFI interfaces to existing scientific libraries

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

**Additional Modules (not in workspace):**
- **`xdl-matlab`**: MATLAB transpilation support (~83,000 line transpiler)
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
- Built-in function calls (100+ functions)
- Array operations and indexing (multi-dimensional arrays)
- Structure definitions
- **Not yet supported**: User-defined procedures (PRO/ENDPRO), GOTO/labels

### Built-in Functions (100+ functions implemented)
- **Mathematical functions**: SIN, COS, TAN, EXP, ALOG, SQRT, FFT, etc.
- **Array functions**: TRANSPOSE, REFORM, REVERSE, SORT, WHERE, etc.
- **Array creation**: FINDGEN, INDGEN, FLTARR, DBLARR, RANDOMU, etc.
- **Statistics**: MEAN, VARIANCE, STDDEV, MEDIAN, MOMENT, KURTOSIS, etc.
- **I/O operations**: PRINT, OPENR, OPENW, READF, WRITEF, etc.
- **Python 3.13 integration**: PYTHON_IMPORT, PYTHON_CALL, PYTHON_CALL_KW (PyO3-based)
- **Graphics and plotting**: 50+ procedures (PLOT, SURFACE, CONTOUR, CHART_*, SURFACE3D, etc.)
- **Machine learning**: 50+ functions (neural networks, K-means, SVM, cross-validation, activation functions, optimizers)
- **Linear algebra**: INVERT, DETERM, SVDC, LA_EIGENVAL, LUDC, etc.
- **Image processing**: CONVOL, DILATE, ERODE, SOBEL, GAUSSIAN_FILTER, etc.
- **Signal processing**: A_CORRELATE, C_CORRELATE, DIGITAL_FILTER, HILBERT, etc.
- **String functions**: STRLEN, STRPOS, STRMID, STRUPCASE, STRING, etc.
- **MATLAB transpilation**: Working transpiler for basic to moderate complexity .m files
- **MATLAB compatibility**: LINSPACE, LOGSPACE, REPMAT, SQUEEZE, NDGRID, INTERP1, MESHGRID
- **GPU acceleration**: XDL-AMP module with multi-backend support

## Known Limitations

While XDL provides substantial functionality, the following features are not yet fully implemented:

### Language Features
- **User-defined procedures**: PRO/ENDPRO syntax is not yet supported (this is the most critical missing feature)
- **GOTO statements**: Label-based control flow is not implemented
- **Complex numbers**: Partial support with some type conversion issues
- **Advanced array indexing**: Some edge cases in multi-dimensional array slicing

### Compatibility
- **IDL/GDL compatibility**: Approximately 60-70% compatible with core IDL features
- **MATLAB transpilation**: Works for basic to moderate complexity but fragile with advanced features
- **Example pass rate**: ~64% of test examples pass; 36% fail due to unsupported features or edge cases

### Testing & Quality
- **Test runner**: `xdl test` command is a stub (not yet implemented)
- **Edge cases**: Various edge cases in control flow, type conversions, and array operations
- **Error messages**: Could be more helpful in some scenarios

### Performance
- **Optimization**: Performance tuning is ongoing
- **GPU acceleration**: Implementation depth varies by operation

**Workaround**: For production use cases requiring missing features, consider using GDL or IDL, or contribute to implementing these features in XDL.

## Installation

### Prerequisites
- Rust 1.70 or later
- **Python 3.13.0** for Python integration features
- Optional: Scientific libraries for full functionality

### Building from Source

```bash
git clone https://github.com/TuringWorks/xdl
cd xdl
cargo build --release
```

### Installing with Cargo

```bash
cargo install --path xdl-cli
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

XDL provides seamless integration with Python 3.13.0. See [PYTHON_INTEGRATION.md](PYTHON_INTEGRATION.md) for detailed documentation.

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
4. Run tests and formatting: `cargo fmt --all && cargo test --workspace`
5. Submit a pull request

### Code Style

- Follow Rust naming conventions
- Use `cargo fmt` for consistent formatting
- Address `cargo clippy` warnings
- Add tests for new functionality
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

### Phase 5: Compatibility & Optimization 🚧 **(Current Focus)**
- [x] IDL/GDL compatibility layer (60-70% compatible)
- [x] Legacy code migration tools
- [x] Documentation and examples (150+ examples)
- [ ] User-defined procedures (PRO/ENDPRO) **critical missing feature**
- [ ] Complete complex number support
- [ ] GOTO and label support
- [ ] Performance optimization
- [ ] Extended edge case handling
- [ ] Comprehensive test coverage

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Original GDL team and contributors
- The Rust community
- Scientific computing libraries (GSL, NetCDF, HDF5, etc.)
- TuringWorks development team

## Project Status

🚧 **Active Development - Beta** 🚧

XDL is a functional beta implementation with substantial language support and advanced features. The project totals **~196,000 lines of Rust code** across 145 source files and includes:

- **Core language**: Variables, expressions, control flow (FOR, WHILE, FOREACH, IF/THEN/ELSE)
- **100+ built-in functions**: Math, arrays, statistics, I/O, graphics
- **50+ ML functions**: Neural networks, K-means, SVM, cross-validation, optimizers
- **50+ graphics procedures**: 2D/3D plotting, charting, interactive visualization
- **Python 3.13 integration**: Real PyO3-based integration with PYTHON_IMPORT/PYTHON_CALL
- **MATLAB transpiler**: 83,000-line transpiler supporting basic to moderate .m files
- **GPU acceleration**: XDL-AMP module with multi-backend support
- **Multiple interfaces**: CLI/REPL, GUI (egui), web-based viewers
- **150+ examples**: Demonstrating features across basics, ML, visualization, and scientific computing

**Compatibility**: Approximately **60-70% IDL/GDL compatible** with core features working well. See Known Limitations below for missing features.

Active development continues on completing user-defined procedures, improving edge case handling, and performance optimization. Contributions and feedback are welcome!
