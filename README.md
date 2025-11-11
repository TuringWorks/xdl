# XDL - Extended Data Language (Rust Implementation)

[![CI](https://github.com/TuringWorks/xdl/workflows/CI/badge.svg)](https://github.com/TuringWorks/xdl/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A modern Rust implementation of the Extended Data Language (XDL), providing IDL-compatible data analysis and visualization capabilities.

## Overview

This is a complete rewrite of XDL in Rust, designed to provide:
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
- **`xdl-stdlib`**: Standard library functions (math, I/O, graphics, ML)
- **`xdl-ffi`**: Foreign function interfaces to external libraries
- **`xdl-cli`**: Command-line interface and REPL
- **`xdl-gui`**: Graphical user interface components
- **`xdl-charts`**: Charting and visualization library
- **`xdl-viz3d`**: 3D visualization engine
- **`xdl-matlab`**: MATLAB transpilation support
- **`xdl-amp`**: Accelerated Math Processing
- **`xdl-chart-viewer`**: Interactive chart viewer application
- **`xdl-desktop-viewer`**: Desktop visualization viewer
- **`xdl-viz3d-threejs`**: Three.js 3D visualization web components
- **`xdl-viz3d-web`**: Web-based 3D visualization

## Features

### Core Data Types
- All XDL/IDL numeric types (BYTE, INT, LONG, FLOAT, DOUBLE, COMPLEX, etc.)
- N-dimensional arrays with efficient indexing
- Structures and pointers
- String handling

### Language Features
- Variables and expressions
- Control flow (IF/THEN/ELSE, FOR, WHILE, FOREACH)
- Functions and procedures
- Array operations and indexing
- Structure definitions

### Built-in Functions
- Mathematical functions (SIN, COS, EXP, etc.)
- Array functions (TRANSPOSE, REFORM, etc.)
- I/O operations (PRINT, READ, etc.)
- **Python 3.13 integration** (PYTHON_IMPORT, PYTHON_CALL)
- Graphics and plotting (2D/3D charts, WebGL rendering)
- Machine learning functions (K-means, neural networks, cross-validation)
- MATLAB transpilation support
- GPU acceleration support
- Advanced visualization (3D plotting, interactive charts)

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
- [x] MATLAB transpilation support
- [x] GPU acceleration
- [x] Interactive chart viewers
- [x] 3D visualization engines

### Phase 4: User Interfaces ✅
- [x] GUI components
- [x] Desktop viewer applications
- [x] Web-based visualization
- [x] Interactive chart viewers

### Phase 5: Compatibility & Optimization 🚧
- [x] IDL/GDL compatibility layer
- [x] Legacy code migration tools
- [x] Documentation and examples
- [ ] Performance optimization
- [ ] Extended language features

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Original GDL team and contributors
- The Rust community
- Scientific computing libraries (GSL, NetCDF, HDF5, etc.)
- TuringWorks development team

## Status

✅ **Feature Complete** ✅

XDL is a mature implementation with comprehensive language support, advanced visualization capabilities, and extensive library features. The project includes:

- Complete XDL/IDL language implementation
- Advanced 2D/3D visualization and charting
- Machine learning and scientific computing functions
- Python 3.13 integration
- MATLAB transpilation support
- GPU acceleration capabilities
- Multiple user interfaces (CLI, GUI, Web)
- Extensive example library and documentation

Active development continues on performance optimization and extended features. Contributions and feedback are welcome!
