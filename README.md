# XDL - Extended Data Language (Rust Implementation)

[![CI](https://github.com/gnudatalanguage/gdl/workflows/CI/badge.svg)](https://github.com/gnudatalanguage/gdl/actions)
[![License: GPL v2](https://img.shields.io/badge/License-GPL%20v2-blue.svg)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html)

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
- **`xdl-stdlib`**: Standard library functions (math, I/O, graphics)
- **`xdl-ffi`**: Foreign function interfaces to external libraries
- **`xdl-cli`**: Command-line interface and REPL

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
- Graphics and plotting (planned)

## Installation

### Prerequisites
- Rust 1.70 or later
- **Python 3.13.0** for Python integration features
- Optional: Scientific libraries for full functionality

### Building from Source

```bash
git clone https://github.com/gnudatalanguage/gdl-rust
cd gdl-rust
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
xdl-rust/
├── xdl-core/          # Core data types and operations
├── xdl-parser/        # Language parser and AST
├── xdl-interpreter/   # Execution engine
├── xdl-runtime/       # Memory and runtime management
├── xdl-stdlib/        # Standard library functions
├── xdl-ffi/          # External library interfaces
└── xdl-cli/          # Command-line interface
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
- [x] Basic parser and AST
- [x] Project structure and build system
- [x] CLI interface and REPL

### Phase 2: Language Implementation 🚧
- [ ] Complete lexer/parser implementation
- [ ] Expression evaluator
- [ ] Statement execution
- [ ] Variable and scope management
- [ ] Function/procedure calls

### Phase 3: Standard Library
- [ ] Mathematical functions
- [ ] Array manipulation
- [ ] I/O operations
- [ ] String functions
- [ ] File handling

### Phase 4: Advanced Features
- [ ] Graphics and plotting
- [ ] External library integration
- [ ] Object-oriented features
- [ ] Optimization and performance

### Phase 5: Compatibility
- [ ] IDL compatibility layer
- [ ] Legacy code migration tools
- [ ] Documentation and examples

## License

This project is licensed under the GNU General Public License v2.0 - see the [COPYING](../COPYING) file for details.

## Acknowledgments

- Original GDL team and contributors
- The Rust community
- Scientific computing libraries (GSL, NetCDF, HDF5, etc.)

## Status

🚧 **Work in Progress** 🚧

This is an active development project. The basic structure is in place, but the interpreter and standard library are still being implemented. Contributions and feedback are welcome!
