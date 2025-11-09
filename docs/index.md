---
layout: default
title: XDL Documentation
---

# XDL - Extended Data Language

A modern Rust implementation of the Extended Data Language (XDL), providing IDL-compatible data analysis and visualization capabilities.

## Quick Links

- [GitHub Repository](https://github.com/ravituringworks/xdl)
- [Quick Start Guide](QUICK_START.html)
- [Examples](README.html)

## Getting Started

### Installation

```bash
git clone https://github.com/ravituringworks/xdl
cd xdl
cargo build --release
cargo install --path xdl-cli
```

### Quick Start Guides

- [Quick Start](QUICK_START.html) - Basic introduction to XDL
- [GPU Acceleration Demo](QUICKSTART_GPU_DEMO.html) - Getting started with GPU features
- [Graphics Quick Start](QUICKSTART_GRAPHICS.html) - Graphics and plotting basics
- [3D Visualization Quick Start](QUICKSTART_VIZ3D.html) - 3D visualization features

## Documentation

### Core Features

#### Language & Parser
- [Implementation Status](IMPLEMENTATION_STATUS.html) - Current implementation status
- [Control Flow Implementation](CONTROL_FLOW_IMPLEMENTATION.html) - IF/THEN/ELSE, FOR, WHILE, etc.
- [Array Features](ARRAY_FEATURES.html) - Array operations and indexing
- [Multi-dimensional Arrays](MULTIDIM_ARRAY_SUPPORT.html) - N-dimensional array support
- [Keyword Arguments](KEYWORD_ARGS_IMPLEMENTATION.html) - Keyword argument support

#### Graphics & Visualization

##### 2D Graphics
- [Graphics Implementation](GRAPHICS_IMPLEMENTATION.html) - 2D graphics overview
- [Graphics Quick Reference](GRAPHICS_QUICK_REF.html) - Quick reference guide
- [Graphics Demos Status](GRAPHICS_DEMOS_STATUS.html) - Demo examples
- [Charting Implementation](CHARTING_IMPLEMENTATION_STATUS.html) - Chart features
- [ECharts Integration](ECHARTS_INTEGRATION_COMPLETE.html) - Web-based charting

##### 3D Visualization
- [3D Visualization Guide](VIZ3D_COMPLETE_FINAL.html) - Complete 3D visualization guide
- [3D Browser Guide](VIZ3D_BROWSER_GUIDE.html) - Browser-based 3D rendering
- [Three.js Integration](VIZ3D_THREEJS_COMPLETE.html) - Three.js backend
- [3D Quick Start](VIZ3D_QUICK_START.html) - Getting started with 3D
- [3D Usage Guide](VIZ3D_USAGE.html) - Using 3D features
- [3D Showcase](VIZ3D_SHOWCASE_README.html) - Example demonstrations

##### Advanced Visualization
- [Scientific Visualization Guide](SCIENTIFIC_VISUALIZATION_GUIDE.html) - Scientific plotting
- [Advanced Visualization](ADVANCED_VIZ_INTEGRATION.html) - Advanced features
- [Bezier Implementation](BEZIER_IMPLEMENTATION_SUMMARY.html) - Bezier curves
- [Performance Improvements](VIZ3D_PERFORMANCE_IMPROVEMENTS.html) - Optimization tips

#### GPU Acceleration
- [GPU Compute Implementation](GPU_COMPUTE_IMPLEMENTATION.html) - GPU acceleration overview
- [GPU Performance Impact](GPU_ACCELERATION_PERFORMANCE_IMPACT.html) - Performance benchmarks
- [XDL AMP Multi-Backend](XDL_AMP_MULTI_BACKEND.html) - Multi-backend GPU support
- [GPU Demo README](README_XDL_AMP_DEMO.html) - GPU examples

### Compatibility

#### IDL/GDL Compatibility
- [IDL Command Status](IDL_COMMAND_STATUS.html) - IDL command compatibility
- [GDL/IDL Compatibility](GDL_IDL_COMPATIBILITY.html) - GDL compatibility layer
- [GDL/IDL Syntax](GDL_IDL_SYNTAX.html) - Syntax reference
- [GDL/XDL Gap Analysis](GDL_XDL_GAP_ANALYSIS.html) - Feature comparison
- [GDL/XDL Porting Status](GDL_XDL_PORTING_STATUS.html) - Porting progress

#### MATLAB Compatibility
- [MATLAB Compatibility](MATLAB_COMPATIBILITY.html) - MATLAB support overview
- [MATLAB Plotting Guide](MATLAB_PLOTTING_GUIDE.html) - MATLAB plotting
- [MATLAB Support](MATLAB_SUPPORT.html) - Feature support
- [MATLAB Limitations](MATLAB_LIMITATIONS.html) - Known limitations
- [MATLAB Real World Support](MATLAB_REAL_WORLD_SUPPORT.html) - Real-world usage
- [MATLAB Tiled Layout](MATLAB_TILEDLAYOUT.html) - Tiled layout support

### Advanced Topics

#### Machine Learning
- [ML Implementation Plan](ML_IMPLEMENTATION_PLAN.html) - ML roadmap
- [ML Complete Reference](ML_COMPLETE_REFERENCE.html) - ML features
- [ML Status](ML_STATUS.html) - Current ML implementation status
- [ML Advanced Features](ML_ADVANCED_FEATURES_STATUS.html) - Advanced ML capabilities

#### Python Integration
- [Python Integration](PYTHON_INTEGRATION.html) - Python 3.13 integration
- [Python Test Results](PYTHON_TEST_RESULTS.html) - Integration tests

#### Examples & Demos
- [Examples](README.html) - XDL and MATLAB examples
- [Bezier Demo](BEZIER_DEMO_FEATURES.html) - Bezier curve examples
- [Rayleigh-Taylor Simulation](README_RAYLEIGH_TAYLOR.html) - Physics simulation demo

### Development

- [Build Success](BUILD_SUCCESS.html) - Build system guide
- [Session Summary](SESSION_SUMMARY.html) - Development session notes
- [Validation Report](VALIDATION_REPORT.html) - Validation testing
- [Validation Status](VALIDATION_STATUS.html) - Current validation status

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
- Python 3.13 integration (PYTHON_IMPORT, PYTHON_CALL)
- Graphics and plotting

## Architecture

The project is structured as a Cargo workspace with the following crates:

- **xdl-core**: Core data structures, types, and array operations
- **xdl-parser**: Lexer and parser for XDL/IDL syntax
- **xdl-interpreter**: AST interpreter and execution engine
- **xdl-runtime**: Runtime system with memory management
- **xdl-stdlib**: Standard library functions
- **xdl-ffi**: Foreign function interfaces
- **xdl-cli**: Command-line interface and REPL
- **xdl-gui**: Graphical user interface
- **xdl-amp**: GPU acceleration module
- **xdl-viz3d**: 3D visualization engine
- **xdl-charts**: Charting library

## Contributing

Contributions are welcome! Please see the [GitHub repository](https://github.com/ravituringworks/xdl) for more information.

## License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.
