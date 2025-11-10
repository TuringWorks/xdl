# XDL - Extended Data Language (Rust Implementation)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A modern Rust implementation of the Extended Data Language (XDL), providing IDL-compatible data analysis and visualization capabilities.

## Overview

XDL is a modern, high-performance Rust implementation of the Extended Data Language (XDL), providing IDL/GDL-compatible scientific computing and visualization capabilities. The project features:

- **Complete Standard Library**: 235+ mathematical, statistical, and I/O functions
- **GPU Acceleration**: Metal Performance Shaders (MPS) backend for 10-50x speedup on numerical operations
- **Advanced Visualization**: 2D/3D plotting with WebGL, WebGPU, and Three.js backends
- **MATLAB Compatibility**: Automatic transpilation of .m files to XDL
- **Memory Safety**: Leveraging Rust's ownership system for safe memory management
- **Performance**: Native code performance with zero-cost abstractions
- **Concurrency**: Safe parallelism using Rust's async/await and threading primitives
- **Interoperability**: FFI interfaces to existing scientific libraries

## Architecture

The project is structured as a Cargo workspace with the following crates:

- **`xdl-core`**: Core data structures, types, and array operations
- **`xdl-parser`**: Lexer and parser for XDL/IDL syntax using nom
- **`xdl-interpreter`**: AST interpreter and execution engine
- **`xdl-runtime`**: Runtime system with memory management
- **`xdl-stdlib`**: Standard library functions (math, I/O, graphics) - 235+ functions
- **`xdl-ffi`**: Foreign function interfaces to external libraries
- **`xdl-cli`**: Command-line interface and REPL
- **`xdl-gui`**: Tauri-based GUI with variable browser and plotting
- **`xdl-matlab`**: MATLAB/Octave transpiler for .m file support
- **`xdl-amp`**: GPU acceleration backend (Metal Performance Shaders)
- **`xdl-chart-viewer`**: ECharts-based 2D charting with WebGL support
- **`xdl-charts`**: Charting library integration
- **`xdl-viz3d`**: 3D visualization with WebGPU backend
- **`xdl-viz3d-threejs`**: Three.js 3D rendering backend
- **`xdl-viz3d-web`**: Web-based 3D visualization
- **`xdl-desktop-viewer`**: Desktop 3D viewer application

## Features

### Core Data Types ✅
- All XDL/IDL numeric types (BYTE, INT, LONG, FLOAT, DOUBLE, COMPLEX, etc.)
- N-dimensional arrays (MultiDimArray) with efficient indexing
- Structures and pointers
- String handling with full Unicode support

### Language Features ✅
- Variables and expressions
- Control flow (IF/THEN/ELSE, FOR, WHILE, FOREACH)
- Functions and procedures
- Array operations and indexing
- Structure definitions
- MATLAB .m file transpilation

### Built-in Functions ✅ (235+ functions)
- **Mathematical functions**: SIN, COS, EXP, LOG, GAMMA, BESSEL, etc.
- **Array functions**: TRANSPOSE, REFORM, CONGRID, HISTOGRAM, etc.
- **Statistical functions**: MEAN, STDDEV, CORRELATE, REGRESS, etc.
- **Signal processing**: FFT, SMOOTH, CONVOL, MEDIAN_FILTER, etc.
- **Image processing**: SOBEL, GAUSSIAN_FILTER, DILATE, ERODE, etc.
- **Linear algebra**: INVERT, DETERM, SVD, EIGENVALUES, etc.
- **I/O operations**: PRINT, READF, WRITEF, FILE operations
- **Time/Date functions**: SYSTIME, JULDAY, CALDAT, etc.
- **String operations**: STRLEN, STRPOS, STRTRIM, etc.
- **GPU-accelerated functions**: MIN, MAX, MEAN, TOTAL (10-50x speedup)

### Visualization & Graphics ✅
- **2D Plotting**: ECharts with WebGL support (60 FPS, 100K+ points)
- **3D Visualization**: WebGPU and Three.js backends for volume rendering
- **Interactive Charts**: Pan, zoom, real-time updates
- **Scientific Demos**: Medical imaging, geophysical data, fluid dynamics

### Advanced Features ✅
- **GPU Acceleration**: Metal Performance Shaders backend
- **MATLAB Compatibility**: Automatic .m file transpilation
- **GUI Application**: Tauri-based desktop app with variable browser
- **Python Integration**: PYTHON_IMPORT, PYTHON_CALL functions
- **Multi-backend Rendering**: WebGL, WebGPU, Three.js support

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
xdl script.xdl

# Run MATLAB file (automatic transpilation)
xdl script.m

# Interactive REPL
xdl

# Execute command directly
xdl -e "print, 'Hello, World!'"

# Parse and show AST
xdl parse script.xdl

# Check syntax
xdl check script.xdl
```

### GUI Application

```bash
# Launch desktop GUI
xdl-gui

# Features:
# - Variable browser
# - Code editor with syntax highlighting
# - Interactive plotting
# - MATLAB .m file support
```

### REPL Commands

```
XDL> print, 'Hello, World!'
XDL> x = findgen(100)
XDL> y = sin(x * !pi / 50)
XDL> plot, x, y, title='Sine Wave'
XDL> .help     ; Show help
XDL> .quit     ; Exit REPL
```

### MATLAB Compatibility

XDL can execute MATLAB .m files directly:

```bash
xdl examples/matlab/01_simple_math.m
```

See [MATLAB_SUPPORT.md](docs/MATLAB_SUPPORT.md) for details.

### Python Integration

XDL provides seamless integration with Python 3.13.0. See [PYTHON_INTEGRATION.md](docs/PYTHON_INTEGRATION.md) for detailed documentation.

```xdl
; Import and use Python modules
math_mod = python_import("math")
result = python_call(math_mod, "sqrt", 16.0)
print, "sqrt(16) =", result
```

### GPU Acceleration

GPU-accelerated functions automatically use Metal Performance Shaders on macOS:

```xdl
; GPU-accelerated operations (10-50x faster)
x = randomu(seed, 1000000)
min_val = min(x)    ; GPU accelerated
mean_val = mean(x)  ; GPU accelerated
total_val = total(x); GPU accelerated
```

## Development

### Project Structure

```
xdl/
├── xdl-core/          # Core data types and operations
├── xdl-parser/        # Language parser and AST
├── xdl-interpreter/   # Execution engine
├── xdl-runtime/       # Memory and runtime management
├── xdl-stdlib/        # Standard library functions
├── xdl-ffi/           # External library interfaces
└── xdl-cli/           # Command-line interface
```

### Building and Testing

```bash
# Check all crates
cargo check --workspace

# Build everything
cargo build --workspace

# Run tests (fast tests only, completes in ~6 seconds)
cargo test --workspace

# Run slow tests (tests that take >10 seconds)
./run_slow_tests.sh
# Or manually:
# cargo test -- --ignored

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
- [x] Basic parser and AST
- [x] Project structure and build system
- [x] CLI interface and REPL

### Phase 2-18: Standard Library Implementation ✅
- [x] Complete lexer/parser implementation
- [x] Expression evaluator and statement execution
- [x] Variable and scope management
- [x] Function/procedure calls
- [x] Mathematical functions (Phase 6)
- [x] Array manipulation (Phase 5)
- [x] Statistics (Phase 7)
- [x] String operations (Phase 8)
- [x] File I/O (Phase 9)
- [x] Linear algebra (Phase 12)
- [x] Image processing (Phase 13)
- [x] Time/date functions (Phase 14)
- [x] Type conversion (Phase 15)
- [x] Data structures (Phase 16)
- [x] Complex numbers (Phase 17)
- [x] System control (Phase 18)
- [x] Signal processing (Phase 11)

### Phase 19-21: Advanced Features ✅
- [x] Graphics and plotting (ECharts, WebGL)
- [x] 3D visualization (WebGPU, Three.js)
- [x] GPU acceleration (Metal Performance Shaders)
- [x] External library integration (Python, scientific libs)
- [x] MATLAB compatibility layer
- [x] GUI application (Tauri-based)

### Phase 22: Performance & Optimization 🚧
- [x] GPU acceleration for core functions
- [ ] Extended GPU acceleration (FFT, convolution, etc.)
- [ ] Multi-GPU support
- [ ] Performance profiling and optimization

### Phase 23: Ecosystem & Compatibility ✅
- [x] IDL/GDL compatibility layer
- [x] MATLAB migration tools
- [x] Comprehensive documentation
- [x] Scientific examples and demos
- [x] Medical imaging workflows
- [x] Geophysical data visualization

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Original GDL team and contributors
- The Rust community
- Scientific computing libraries (GSL, NetCDF, HDF5, etc.)

## Status

🎉 **Production Ready** 🎉

XDL is a fully functional scientific computing platform with:
- Complete standard library (235+ functions)
- GPU acceleration for high performance
- Advanced 2D/3D visualization
- MATLAB compatibility
- GUI application
- Comprehensive documentation and examples

The core implementation is stable and ready for scientific workflows. Active development continues with performance optimizations and additional features.
