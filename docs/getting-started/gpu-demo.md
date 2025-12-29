---
layout: default
title: GPU Demo
parent: Getting Started
nav_order: 3
---

# GPU Acceleration Demo

Quick introduction to XDL's GPU acceleration capabilities.

For the complete GPU demo guide, see [QUICKSTART_GPU_DEMO.md]({% link QUICKSTART_GPU_DEMO.md %}).

## Overview

XDL provides GPU acceleration through the AMP (Accelerated Math Processing) module, supporting multiple backends:

- **CUDA** - NVIDIA GPUs
- **ROCm** - AMD GPUs
- **Vulkan** - Cross-platform
- **CPU** - Fallback for systems without GPU

## Quick Example

```xdl
; Create large arrays
n = 10000000
a = findgen(n)
b = findgen(n) * 2.0

; GPU-accelerated operations
c = a + b          ; Vector addition
d = sin(a)         ; Trigonometric functions
e = a * b + c      ; Complex expressions

print, 'GPU computation complete!'
```

## Performance Comparison

| Operation | CPU Time | GPU Time | Speedup |
|:----------|:---------|:---------|:--------|
| Vector Add (10M) | 45ms | 2ms | 22.5x |
| Sin (10M) | 120ms | 5ms | 24x |
| Matrix Multiply | 850ms | 12ms | 70x |

## Enabling GPU Acceleration

GPU acceleration is enabled automatically when available:

```bash
# Check GPU support
xdl --features

# Run with GPU profiling
XDL_GPU_PROFILE=1 xdl script.xdl
```

## Next Steps

- [GPU Compute Implementation]({% link GPU_COMPUTE_IMPLEMENTATION.md %}) - Technical details
- [Performance Analysis]({% link GPU_ACCELERATION_PERFORMANCE_IMPACT.md %}) - Benchmarks
- [AMP Multi-Backend]({% link XDL_AMP_MULTI_BACKEND.md %}) - Backend configuration
