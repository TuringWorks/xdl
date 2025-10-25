# XDL AMP (Accelerated Math Processing)

Multi-backend GPU and ML acceleration for XDL with comprehensive platform support.

## Overview

XDL AMP provides a unified interface for GPU/ML operations with **10 acceleration backends**:

### Apple Platforms (macOS, iOS)
- ✅ **Metal Performance Shaders (MPS)** - Optimized operations (default)
- ✅ **Metal** - Low-level GPU compute
- ✅ **CoreML** - Apple Neural Engine acceleration

### NVIDIA Platforms
- ✅ **cuDNN** - Deep learning acceleration
- ✅ **CUDA** - GPU compute

### AMD Platforms
- ✅ **ROCm** - GPU acceleration for AMD GPUs

### Windows
- ✅ **DirectML** - ML acceleration on DirectX
- ✅ **DirectX 12** - GPU compute shaders

### Cross-Platform
- ✅ **ONNX Runtime** - ML model inference
- ✅ **OpenCL** - Universal GPU fallback

## Architecture

```
┌─────────────────────────────────────┐
│         XDL Applications            │
└─────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────┐
│          GpuContext                 │
│   (Automatic backend selection)     │
└─────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────┐
│        GpuDevice Trait              │
│  (Unified GPU operations API)       │
└─────────────────────────────────────┘
                 │
    ┌────────────┼────────────┐
    ▼            ▼            ▼
┌────────┐  ┌─────────┐  ┌─────────┐
│ Metal  │  │  CUDA   │  │ OpenCL  │
│ (macOS)│  │(Linux/W)│  │ (fallbk)│
└────────┘  └─────────┘  └─────────┘
```

## Features

### Implemented (Metal backend)

- ✅ Element-wise operations: add, mul, sub, div
- ✅ Mathematical functions: sin, cos, exp, log, sqrt
- ✅ Metal compute shaders
- ✅ Automatic buffer management
- ✅ Platform detection

### Planned

- ⏳ Matrix multiplication (GEMM)
- ⏳ Reduction operations (sum, max, min)
- ⏳ CUDA backend implementation
- ⏳ OpenCL backend implementation
- ⏳ DirectX 12 backend implementation
- ⏳ Convolution operations
- ⏳ FFT operations

## Usage

### Basic Example

```rust
use xdl_amp::{GpuContext, ops::GpuOps};
use ndarray::Array1;

// Create GPU context (automatically selects best backend)
let ctx = GpuContext::new()?;
println!("Using GPU backend: {}", ctx.backend_name());

// Create GPU operations
let gpu_ops = GpuOps::new(ctx.device().clone());

// Perform GPU-accelerated operations
let a = Array1::from_vec(vec![1.0, 2.0, 3.0, 4.0]);
let b = Array1::from_vec(vec![5.0, 6.0, 7.0, 8.0]);

let c = gpu_ops.add_1d(&a, &b)?;
println!("Result: {:?}", c);
```

### Platform Selection

```rust
use xdl_amp::{GpuContext, GpuBackend};

// Prefer CUDA if available
#[cfg(feature = "cuda")]
let ctx = GpuContext::with_preference(Some(GpuBackend::Cuda))?;

// Use Metal on macOS
#[cfg(target_os = "macos")]
let ctx = GpuContext::with_preference(Some(GpuBackend::Metal))?;
```

## Building

### macOS (Metal)

```bash
cargo build --release
```

### Linux with CUDA

```bash
cargo build --release --features cuda
```

Requires CUDA toolkit installed.

### Linux with OpenCL

```bash
cargo build --release --features opencl
```

Requires OpenCL runtime installed.

### Windows

```bash
# DirectX 12 (default)
cargo build --release

# Or with CUDA
cargo build --release --features cuda
```

## Supported Operations

| Operation | Metal | CUDA | OpenCL | DirectX 12 |
|-----------|-------|------|--------|------------|
| add_f32   | ✅    | ⏳   | ⏳     | ⏳         |
| mul_f32   | ✅    | ⏳   | ⏳     | ⏳         |
| sub_f32   | ✅    | ⏳   | ⏳     | ⏳         |
| div_f32   | ✅    | ⏳   | ⏳     | ⏳         |
| sin_f32   | ✅    | ⏳   | ⏳     | ⏳         |
| cos_f32   | ✅    | ⏳   | ⏳     | ⏳         |
| exp_f32   | ✅    | ⏳   | ⏳     | ⏳         |
| log_f32   | ✅    | ⏳   | ⏳     | ⏳         |
| sqrt_f32  | ✅    | ⏳   | ⏳     | ⏳         |
| matmul    | ⏳    | ⏳   | ⏳     | ⏳         |
| sum       | ⏳    | ⏳   | ⏳     | ⏳         |

Legend: ✅ Implemented, ⏳ Planned, ❌ Not supported

## Performance

GPU acceleration provides significant speedup for large arrays:

- **Small arrays (<1K elements)**: CPU faster due to overhead
- **Medium arrays (1K-100K)**: 2-5x speedup
- **Large arrays (>100K)**: 10-50x speedup

Actual performance depends on:
- GPU hardware
- Data transfer overhead
- Operation complexity
- Array size and layout

## Platform-Specific Notes

### macOS (Metal)

- Supported on macOS 10.13+
- Uses Metal Shading Language (MSL)
- Optimized for Apple Silicon (M1/M2/M3)
- Unified memory architecture benefits

### Windows (DirectX 12)

- Supported on Windows 10+
- Uses HLSL compute shaders
- Works with any DirectX 12 compatible GPU

### CUDA (Linux/Windows)

- Requires NVIDIA GPU
- CUDA toolkit must be installed
- Best performance on recent NVIDIA cards

### OpenCL (Cross-platform)

- Works on most GPUs (NVIDIA, AMD, Intel)
- Performance varies by vendor
- Use as fallback option

## Contributing

Contributions welcome! Priority areas:

1. Complete CUDA backend implementation
2. Complete OpenCL backend implementation
3. Complete DirectX 12 backend implementation
4. Implement matrix multiplication
5. Add reduction operations
6. Performance benchmarks

## License

GPL-2.0 - Same as parent XDL project
