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
> **Phase 8 Complete** - GPU Acceleration with AMP Multi-Backend Support! 🚀

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

- [Quick Start Guide](getting-started/quick-start) - Get up and running in minutes
- [Installation](getting-started/installation) - Detailed installation instructions
- [GPU Acceleration Demo](getting-started/gpu-demo) - GPU features walkthrough
- [Graphics Quick Start](getting-started/graphics-quickstart) - 2D graphics basics
- [3D Visualization Quick Start](getting-started/viz3d-quickstart) - 3D visualization basics
- [Examples](getting-started/examples) - Sample code and tutorials

### Core Features
{: .text-delta }

Language implementation and core functionality

- [Implementation Status](core/implementation-status) - Current implementation progress
- [Language Features](core/language-features) - Syntax and language constructs
- [Control Flow](core/control-flow) - IF/THEN/ELSE, FOR, WHILE loops
- [Array Operations](core/arrays) - N-dimensional array support
- [Multi-dimensional Arrays](core/multidim-arrays) - Advanced array features
- [Keyword Arguments](core/keyword-args) - Function keyword arguments

### Graphics & Visualization
{: .text-delta }

Comprehensive 2D and 3D graphics capabilities

#### 2D Graphics
- [Graphics Overview](graphics/overview) - 2D graphics implementation
- [Graphics Quick Reference](graphics/quick-ref) - Command reference
- [Charting](graphics/charting) - Chart and plot features
- [ECharts Integration](graphics/echarts) - Web-based charting
- [Bezier Curves](graphics/bezier) - Curve drawing features

#### 3D Visualization
- [3D Visualization Complete](graphics/viz3d-complete) - Full 3D guide
- [3D Browser Rendering](graphics/viz3d-browser) - Browser-based 3D
- [Three.js Integration](graphics/viz3d-threejs) - Three.js backend
- [3D Performance](graphics/viz3d-performance) - Optimization guide
- [Scientific Visualization](graphics/scientific-viz) - Scientific plotting

### GPU Acceleration
{: .text-delta }

High-performance GPU computing features

- [GPU Compute Implementation](gpu/compute) - GPU acceleration overview
- [Performance Impact Analysis](gpu/performance) - Benchmarks and metrics
- [AMP Multi-Backend](gpu/amp-multibackend) - Multiple GPU backends
- [GPU Demo Guide](gpu/demo) - GPU examples and tutorials

### Compatibility
{: .text-delta }

IDL/GDL and MATLAB compatibility layers

#### IDL/GDL Compatibility
- [IDL Command Status](compatibility/idl-commands) - Supported IDL commands
- [GDL/IDL Compatibility](compatibility/gdl-idl) - Compatibility layer
- [GDL/IDL Syntax](compatibility/gdl-idl-syntax) - Syntax reference
- [Gap Analysis](compatibility/gap-analysis) - Feature comparison

#### MATLAB Compatibility
- [MATLAB Support](compatibility/matlab) - MATLAB compatibility overview
- [MATLAB Plotting](compatibility/matlab-plotting) - MATLAB plot functions
- [MATLAB Limitations](compatibility/matlab-limitations) - Known limitations
- [Tiled Layout](compatibility/matlab-tiledlayout) - Tiled layout support

### Advanced Topics
{: .text-delta }

Machine learning, Python integration, and more

- [Machine Learning](advanced/ml-overview) - ML capabilities and roadmap
- [Python Integration](advanced/python) - Python 3.13 integration
- [Advanced Visualization](advanced/advanced-viz) - Complex visualizations
- [Moving Average](advanced/moving-average) - Signal processing
- [Rayleigh-Taylor Demo](advanced/rayleigh-taylor) - Physics simulation

### Development
{: .text-delta }

Contributing and building XDL

- [Build Guide](development/build) - Building from source
- [Testing](development/testing) - Running tests
- [Validation](development/validation) - Validation reports
- [Contributing](development/contributing) - How to contribute
- [Architecture](development/architecture) - System architecture

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
| Phase 3 | ✅ Complete | Standard Library |
| Phase 4 | ✅ Complete | Graphics & Visualization |
| Phase 5 | ✅ Complete | 3D Visualization |
| Phase 6 | ✅ Complete | MATLAB Compatibility |
| Phase 7 | ✅ Complete | IDL/GDL Compatibility |
| Phase 8 | ✅ Complete | GPU Acceleration |
| Phase 9 | 🚧 In Progress | Machine Learning |
| Phase 10 | 📋 Planned | Production Optimization |

---

## 🤝 Contributing

Contributions are welcome! We appreciate:

- Bug reports and feature requests
- Code contributions via pull requests
- Documentation improvements
- Performance benchmarks and optimizations

See our [Contributing Guide](development/contributing) for more details.

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/TuringWorks/xdl/blob/master/LICENSE) file for details.

---

## 🔗 Links

- [GitHub Repository](https://github.com/TuringWorks/xdl)
- [Issue Tracker](https://github.com/TuringWorks/xdl/issues)
- [Discussions](https://github.com/TuringWorks/xdl/discussions)
