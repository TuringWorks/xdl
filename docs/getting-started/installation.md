---
layout: default
title: Installation
parent: Getting Started
nav_order: 2
---

# Installation Guide

Detailed instructions for installing XDL on your system.

## Prerequisites

- **Rust 1.70 or later** - [Install Rust](https://rustup.rs/)
- **Python 3.13.0** (optional) - For Python integration features
- **Git** - For cloning the repository

### Optional Dependencies

For full functionality, you may want:

- **OpenGL/Vulkan** - For GPU acceleration
- **Scientific libraries** - GSL, NetCDF, HDF5

## Building from Source

### 1. Clone the Repository

```bash
git clone https://github.com/TuringWorks/xdl
cd xdl
```

### 2. Build the Project

```bash
# Build all crates
cargo build --release --workspace

# This will take a few minutes on first build
```

### 3. Install the CLI

```bash
cargo install --path xdl-cli
```

### 4. Verify Installation

```bash
xdl --version
```

## Platform-Specific Notes

### Linux

```bash
# Ubuntu/Debian - install development dependencies
sudo apt-get install build-essential pkg-config libssl-dev

# Fedora/RHEL
sudo dnf install gcc openssl-devel
```

### macOS

```bash
# Install Xcode Command Line Tools
xcode-select --install

# Or install via Homebrew
brew install rust
```

### Windows

- Install [Rust via rustup](https://rustup.rs/)
- Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)

## Python Integration Setup

For Python 3.13 integration:

```bash
# Ensure Python 3.13 is installed
python3.13 --version

# XDL will automatically detect and use Python 3.13
```

See [Python Integration Guide]({% link PYTHON_INTEGRATION.md %}) for details.

## GPU Acceleration Setup

For GPU acceleration features:

```bash
# Build with GPU support
cargo build --release --features gpu

# Verify GPU support
xdl --features
```

See [GPU Demo Guide](gpu-demo) for more information.

## Troubleshooting

### Build Errors

If you encounter build errors:

```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

### Python Integration Issues

If Python integration doesn't work:

```bash
# Check Python version
python3 --version

# Ensure Python 3.13 development headers are installed
# On Ubuntu/Debian:
sudo apt-get install python3.13-dev
```

## Next Steps

- [Quick Start Guide](quick-start) - Your first XDL program
- [GPU Demo](gpu-demo) - Try GPU acceleration
- [Examples](examples) - Sample code

## Development Setup

For development and testing:

```bash
# Install pre-commit hooks
pip install pre-commit
pre-commit install

# Run tests
cargo test --workspace

# Run slow tests
./run_slow_tests.sh

# Format and lint
cargo fmt --all
cargo clippy --workspace
```

See [Development Guide]({% link development/index.md %}) for more details.

For complete build documentation, see [BUILD_SUCCESS]({% link BUILD_SUCCESS.md %}).
