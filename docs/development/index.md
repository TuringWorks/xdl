---
layout: default
title: Development
nav_order: 8
has_children: true
permalink: /development
---

# Development

Contributing and building XDL.

## Building from Source

### Prerequisites

- Rust 1.70 or later
- Git
- Optional: Python 3.13, GPU drivers

### Build Steps

```bash
# Clone repository
git clone https://github.com/TuringWorks/xdl
cd xdl

# Build all crates
cargo build --release --workspace

# Install CLI
cargo install --path xdl-cli
```

See [Build Guide](../BUILD_SUCCESS) for detailed instructions.

## Testing

### Running Tests

```bash
# Run fast tests (~6 seconds)
cargo test --workspace

# Run slow tests (>10 seconds)
./run_slow_tests.sh

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Test Organization

- **Fast tests** - Unit tests, integration tests (<10s)
- **Slow tests** - Marked with `#[ignore]`, run separately
- **Benchmark tests** - Performance benchmarks

See [Slow Tests Changes](../SLOW_TESTS_CHANGES) for details.

## Code Quality

### Pre-commit Hooks

```bash
# Install pre-commit
pip install pre-commit

# Install hooks
pre-commit install

# Run manually
pre-commit run --all-files
```

Hooks run:
- `cargo fmt` - Code formatting
- `cargo check` - Compilation check
- `cargo clippy` - Linting
- `cargo test` - Tests

### Formatting and Linting

```bash
# Format code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --workspace

# Fix clippy warnings
cargo clippy --workspace --fix
```

## Validation

Validation testing and reports:

- [Validation Report](../VALIDATION_REPORT) - Test results
- [Validation Status](../VALIDATION_STATUS) - Current status

### Running Validation

```bash
# Run validation suite
./scripts/validate.sh

# Generate coverage report
./generate-coverage.sh
```

## Architecture

Project structure and design:

### Crate Organization

```
xdl/
├── xdl-core/          # Core data types and operations
├── xdl-parser/        # Lexer and parser
├── xdl-interpreter/   # Execution engine
├── xdl-runtime/       # Memory and runtime
├── xdl-stdlib/        # Standard library
├── xdl-ffi/          # External interfaces
├── xdl-cli/          # CLI and REPL
├── xdl-gui/          # GUI application
├── xdl-amp/          # GPU acceleration
├── xdl-viz3d/        # 3D visualization
├── xdl-charts/       # Charting library
└── xdl-matlab/       # MATLAB compatibility
```

### Design Principles

- **Memory Safety** - Rust ownership system
- **Zero-Cost Abstractions** - Performance without overhead
- **Modularity** - Separate crates for different features
- **Extensibility** - Plugin architecture
- **Compatibility** - IDL/MATLAB compatibility layers

## Contributing

We welcome contributions!

### How to Contribute

1. **Fork the repository**
2. **Create a feature branch**
   ```bash
   git checkout -b feature/amazing-feature
   ```
3. **Make your changes**
4. **Run tests and formatting**
   ```bash
   cargo fmt --all
   cargo test --workspace
   cargo clippy --workspace
   ```
5. **Commit your changes**
   ```bash
   git commit -m "Add amazing feature"
   ```
6. **Push to your fork**
   ```bash
   git push origin feature/amazing-feature
   ```
7. **Open a Pull Request**

### Contribution Guidelines

- Follow Rust naming conventions
- Add tests for new functionality
- Document public APIs
- Update relevant documentation
- Address clippy warnings
- Keep commits focused and well-described

### Code Style

- Use `cargo fmt` for formatting
- Follow Rust idioms
- Write clear comments
- Use descriptive variable names
- Keep functions focused and small

### Documentation

- Document all public APIs with doc comments
- Include examples in documentation
- Update relevant .md files
- Add entries to CHANGELOG

## Communication

- **Issues** - [GitHub Issues](https://github.com/TuringWorks/xdl/issues)
- **Discussions** - [GitHub Discussions](https://github.com/TuringWorks/xdl/discussions)
- **Pull Requests** - Review and collaboration

## Development Status

See [Session Summary](../SESSION_SUMMARY) for recent development updates.

### Implementation Status

- [Implementation Status](../IMPLEMENTATION_STATUS) - Current progress
- [Implementation Summary](../IMPLEMENTATION_SUMMARY) - Summary overview
- [IDL Command Status](../IDL_COMMAND_STATUS) - IDL compatibility
- [Build Success](../BUILD_SUCCESS) - Build system status
- [Parser and Array Fixes](../PARSER_AND_ARRAY_FIXES) - Recent fixes

## Release Process

1. Update version in Cargo.toml files
2. Update CHANGELOG.md
3. Run full test suite
4. Create git tag
5. Push to GitHub
6. Publish to crates.io (when ready)

## CI/CD

Continuous Integration via GitHub Actions:

- Build on Linux, macOS, Windows
- Run test suite
- Check formatting and linting
- Generate documentation
- Deploy GitHub Pages

See `.github/workflows/` for workflows.

## Performance

### Profiling

```bash
# CPU profiling
cargo install flamegraph
cargo flamegraph --bin xdl

# Memory profiling
cargo install valgrind
valgrind --tool=massif target/release/xdl
```

### Benchmarking

```bash
# Run benchmarks
cargo bench

# Compare benchmarks
cargo bench -- --save-baseline main
git checkout feature
cargo bench -- --baseline main
```

## Documentation Development

This documentation site:

- Built with Jekyll and Just the Docs theme
- Deployed via GitHub Pages
- Auto-updates on push to master

Local development:

```bash
cd docs
bundle install
bundle exec jekyll serve
```
