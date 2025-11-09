---
layout: default
title: GPU Acceleration
nav_order: 5
has_children: true
permalink: /gpu
---

# GPU Acceleration

High-performance GPU computing features in XDL.

## Overview

XDL's AMP (Accelerated Math Processing) module provides GPU acceleration with multiple backend support:

- **CUDA** - NVIDIA GPUs (best performance on NVIDIA hardware)
- **ROCm** - AMD GPUs (optimized for AMD hardware)
- **Vulkan** - Cross-platform GPU compute
- **CPU** - Fallback for systems without GPU support

## Key Features

### Automatic Acceleration

GPU acceleration is transparent - existing code runs faster without changes:

```xdl
; These operations are automatically GPU-accelerated
a = findgen(10000000)
b = findgen(10000000)
c = a + b           ; GPU vector addition
d = sin(a)          ; GPU trigonometric functions
e = a * b + c       ; GPU complex expressions
```

### Performance Gains

Typical speedups on GPU vs CPU:

| Operation | Array Size | CPU Time | GPU Time | Speedup |
|:----------|:-----------|:---------|:---------|:--------|
| Vector Add | 10M | 45ms | 2ms | **22.5x** |
| Sin | 10M | 120ms | 5ms | **24x** |
| Matrix Multiply | 4096x4096 | 850ms | 12ms | **70x** |
| FFT | 1M | 200ms | 8ms | **25x** |

## Documentation

- [GPU Compute Implementation](../GPU_COMPUTE_IMPLEMENTATION) - Technical overview
- [Performance Impact Analysis](../GPU_ACCELERATION_PERFORMANCE_IMPACT) - Benchmarks
- [AMP Multi-Backend](../XDL_AMP_MULTI_BACKEND) - Backend configuration
- [GPU Demo Guide](../README_XDL_AMP_DEMO) - Examples and tutorials

## Supported Operations

### Vector Operations
- Addition, subtraction, multiplication, division
- Element-wise operations
- Vector reductions (sum, min, max, mean)

### Mathematical Functions
- Trigonometric: sin, cos, tan, asin, acos, atan
- Exponential: exp, log, log10, sqrt, pow
- Hyperbolic: sinh, cosh, tanh

### Matrix Operations
- Matrix multiplication
- Matrix transpose
- Matrix inversion
- Eigenvalue decomposition

### Advanced Operations
- FFT (Fast Fourier Transform)
- Convolution
- Correlation
- Image processing

## Backend Selection

GPU backend is selected automatically based on available hardware:

```bash
# Check available GPU backends
xdl --features

# Force specific backend
XDL_GPU_BACKEND=cuda xdl script.xdl
XDL_GPU_BACKEND=rocm xdl script.xdl
XDL_GPU_BACKEND=vulkan xdl script.xdl
XDL_GPU_BACKEND=cpu xdl script.xdl
```

## Profiling

Enable GPU profiling:

```bash
# Enable profiling
XDL_GPU_PROFILE=1 xdl script.xdl

# Detailed profiling
XDL_GPU_PROFILE=verbose xdl script.xdl
```

## Memory Management

XDL automatically manages GPU memory:

- **Automatic transfer** - Data moved to/from GPU as needed
- **Memory pooling** - Efficient reuse of GPU memory
- **Spill to CPU** - Graceful handling of large datasets

## Limitations

Current limitations:

- Maximum array size: 2GB per array
- Some operations fall back to CPU
- Multi-GPU support in development

## Next Steps

- [Quick Start](../getting-started/gpu-demo) - Get started with GPU
- [Performance Guide](../GPU_ACCELERATION_PERFORMANCE_IMPACT) - Optimization tips
- [Technical Details](../GPU_COMPUTE_IMPLEMENTATION) - Implementation details
