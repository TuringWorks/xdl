---
layout: home
title: Home
nav_order: 1
description: "XDL - Extended Data Language: A modern Rust implementation with IDL/MATLAB compatibility"
permalink: /
---

# XDL - Extended Data Language

{: .fs-9 }

A modern Rust implementation of the Extended Data Language (XDL), providing IDL-compatible data analysis and visualization capabilities with GPU acceleration.
{: .fs-6 .fw-300 }

[Get Started](getting-started/quick-start){: .btn .btn-primary .fs-5 .mb-4 .mb-md-0 .mr-2 }
[View on GitHub](https://github.com/TuringWorks/xdl){: .btn .fs-5 .mb-4 .mb-md-0 }

---

{: .new }
> **Active Development - Beta Release** - 100+ functions, 50+ ML features, GPU acceleration, and 60-70% IDL compatibility! 🚀

## 🚀 Getting Started

<div class="code-example" markdown="1">

### Installation

```bash
git clone https://github.com/TuringWorks/xdl
cd xdl
cargo build --release
cargo install --path xdl-cli
```

### Quick Example

```xdl
; Create data and visualize
x = findgen(100)
y = sin(x * !pi / 50)
plot, x, y, title='XDL Plot'
```

</div>

[Installation Guide →](getting-started/installation){: .btn .btn-outline }
[Quick Start →](getting-started/quick-start){: .btn .btn-outline }
[GPU Demo →](getting-started/gpu-demo){: .btn .btn-outline }

---

## 📚 Documentation Sections

### Getting Started

{: .text-delta }

Essential resources to begin using XDL

- [Quick Start Guide](getting-started) - Get up and running in minutes
- [Quick Start](QUICK_START) - Detailed quick start guide
- [Examples Gallery](examples-gallery) - Visual examples with generated plots
- [GPU Acceleration Demo](QUICKSTART_GPU_DEMO) - GPU features walkthrough
- [Graphics Quick Start](QUICKSTART_GRAPHICS) - 2D graphics basics
- [3D Visualization Quick Start](QUICKSTART_VIZ3D) - 3D visualization basics
- [Examples](README) - Sample code and tutorials

### Core Features

{: .text-delta }

Language implementation and core functionality

- [Implementation Status](IMPLEMENTATION_STATUS) - Current implementation progress
- [Control Flow](CONTROL_FLOW_IMPLEMENTATION) - IF/THEN/ELSE, FOR, WHILE loops
- [Array Operations](ARRAY_FEATURES) - N-dimensional array support
- [Multi-dimensional Arrays](MULTIDIM_ARRAY_SUPPORT) - Advanced array features
- [Keyword Arguments](KEYWORD_ARGS_IMPLEMENTATION) - Function keyword arguments

### Graphics & Visualization

{: .text-delta }

Comprehensive 2D and 3D graphics capabilities

#### 2D Graphics

- [Graphics Overview](GRAPHICS_IMPLEMENTATION) - 2D graphics implementation
- [Graphics Quick Reference](GRAPHICS_QUICK_REF) - Command reference
- [Charting Status](CHARTING_FINAL_STATUS) - Chart and plot features
- [ECharts Integration](ECHARTS_INTEGRATION_COMPLETE) - Web-based charting
- [Bezier Curves](BEZIER_IMPLEMENTATION_SUMMARY) - Curve drawing features

#### 3D Visualization

- [3D Visualization Complete](VIZ3D_COMPLETE_FINAL) - Full 3D guide
- [3D Browser Rendering](VIZ3D_BROWSER_GUIDE) - Browser-based 3D
- [Three.js Integration](VIZ3D_THREEJS_COMPLETE) - Three.js backend
- [3D Performance](VIZ3D_PERFORMANCE_IMPROVEMENTS) - Optimization guide
- [Scientific Visualization](SCIENTIFIC_VISUALIZATION_GUIDE) - Scientific plotting

### GPU Acceleration

{: .text-delta }

High-performance GPU computing features

- [GPU Compute Implementation](GPU_COMPUTE_IMPLEMENTATION) - GPU acceleration overview
- [Performance Impact Analysis](GPU_ACCELERATION_PERFORMANCE_IMPACT) - Benchmarks and metrics
- [AMP Multi-Backend](XDL_AMP_MULTI_BACKEND) - Multiple GPU backends
- [GPU Demo Guide](README_XDL_AMP_DEMO) - GPU examples and tutorials

### Compatibility

{: .text-delta }

IDL/GDL and MATLAB compatibility layers

#### IDL/GDL Compatibility

- [IDL Command Status](IDL_COMMAND_STATUS) - Supported IDL commands
- [GDL/IDL Compatibility](GDL_IDL_COMPATIBILITY) - Compatibility layer
- [GDL/IDL Syntax](GDL_IDL_SYNTAX) - Syntax reference
- [Gap Analysis](GDL_XDL_GAP_ANALYSIS) - Feature comparison

#### MATLAB Compatibility

- [MATLAB Support](MATLAB_SUPPORT) - MATLAB compatibility overview
- [MATLAB Plotting](MATLAB_PLOTTING_GUIDE) - MATLAB plot functions
- [MATLAB Compatibility Functions](MATLAB_3D_PLOT_FIX) - LINSPACE, LOGSPACE, REPMAT, SQUEEZE, NDGRID, INTERP1, MESHGRID
- [MATLAB Limitations](MATLAB_LIMITATIONS) - Known limitations
- [Tiled Layout](MATLAB_TILEDLAYOUT) - Tiled layout support

### Advanced Topics

{: .text-delta }

Machine learning, Python integration, and more

- [Machine Learning Overview](ML_COMPLETE_REFERENCE) - ML capabilities and reference
- [Python Integration](PYTHON_INTEGRATION) - Python 3.13 integration
- [Advanced Visualization](ADVANCED_VIZ_INTEGRATION) - Complex visualizations
- [Moving Average](MOVING_AVERAGE) - Signal processing
- [Rayleigh-Taylor Demo](README_RAYLEIGH_TAYLOR) - Physics simulation

### Development

{: .text-delta }

Contributing and building XDL

- [Build Guide](BUILD_SUCCESS) - Building from source
- [Session Summary](SESSION_SUMMARY) - Development updates
- [Validation](VALIDATION_REPORT) - Validation reports
- [Slow Tests](SLOW_TESTS_CHANGES) - Test organization

---

## ✨ Key Features

### Memory Safety

{: .text-delta }
Leveraging Rust's ownership system for safe memory management

### Performance

{: .text-delta }
Native code performance with zero-cost abstractions and GPU acceleration

### Concurrency

{: .text-delta }
Safe parallelism using Rust's async/await and threading primitives

### Interoperability

{: .text-delta }
FFI interfaces to existing scientific libraries and Python 3.13 integration

---

## 🏗️ Architecture

XDL is structured as a Cargo workspace with the following crates:

| Crate | Description |
|:------|:------------|
| `xdl-core` | Core data structures, types, and array operations |
| `xdl-parser` | Lexer and parser for XDL/IDL syntax |
| `xdl-interpreter` | AST interpreter and execution engine |
| `xdl-runtime` | Runtime system with memory management |
| `xdl-stdlib` | Standard library functions |
| `xdl-ffi` | Foreign function interfaces |
| `xdl-cli` | Command-line interface and REPL |
| `xdl-gui` | Graphical user interface |
| `xdl-amp` | GPU acceleration module |
| `xdl-viz3d` | 3D visualization engine |
| `xdl-charts` | Charting library |

---

## 📊 Project Status

| Phase | Status | Description |
|:------|:-------|:------------|
| Phase 1 | ✅ Complete | Foundation & Core Types |
| Phase 2 | ✅ Complete | Parser & Interpreter |
| Phase 3 | ✅ Complete | Standard Library (100+ functions) |
| Phase 4 | ✅ Complete | Graphics & Visualization (50+ procedures) |
| Phase 5 | ✅ Complete | 3D Visualization |
| Phase 6 | ✅ Complete | MATLAB Compatibility (basic to moderate) |
| Phase 7 | ✅ Complete | IDL/GDL Compatibility (60-70% compatible) |
| Phase 8 | ✅ Complete | GPU Acceleration (XDL-AMP) |
| Phase 9 | ✅ Complete | Machine Learning (50+ functions) |
| Phase 10 | 🚧 **Current** | **Compatibility & Bug Fixes** |
| Phase 11 | 📋 Planned | Performance Optimization |

**Current Focus (Phase 10):**

- User-defined procedures (PRO/ENDPRO) - critical missing feature
- Complex number edge cases
- Advanced array indexing improvements
- Test coverage expansion
- Error message improvements

---

## 🤝 Contributing

Contributions are welcome! We appreciate:

- Bug reports and feature requests
- Code contributions via pull requests
- Documentation improvements
- Performance benchmarks and optimizations

See our [Contributing Guide](development#contributing) for more details.

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/TuringWorks/xdl/blob/master/LICENSE) file for details.

---

## 🔗 Links

- [GitHub Repository](https://github.com/TuringWorks/xdl)
- [Issue Tracker](https://github.com/TuringWorks/xdl/issues)
- [Discussions](https://github.com/TuringWorks/xdl/discussions)
