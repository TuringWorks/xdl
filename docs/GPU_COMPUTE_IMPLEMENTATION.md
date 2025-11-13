# GPU Compute Implementation for XDL

## Overview

XDL now supports GPU-accelerated computation through the new `xdl-amp` (Accelerated Math Processing) crate. This provides cross-platform GPU acceleration using platform-specific APIs.

## Implementation Date

October 25, 2025

## Architecture

### Multi-Backend Design

```text
┌─────────────────────────────────────┐
│         XDL Applications            │
│    (xdl-stdlib, user code)          │
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
    ┌────────────┼─────────────┬──────────┐
    ▼            ▼             ▼          ▼
┌────────┐  ┌─────────┐  ┌────────-─┐ ┌──────────┐
│ Metal  │  │  CUDA   │  │ OpenCL   │ │DirectX 12│
│ (macOS)│  │(NVIDIA) │  │(Fallback)│ │(Windows) │
└────────┘  └─────────┘  └────────-─┘ └──────────┘
```

### Platform Support

| Platform | Primary Backend | Alternative | Status |
|----------|----------------|-------------|--------|
| macOS    | Metal          | -           | ✅ Fully Implemented |
| Windows  | DirectX 12     | CUDA        | ⏳ Stub |
| Linux    | CUDA           | OpenCL      | ⏳ Stub |

## Components

### 1. Core Abstractions (`backend.rs`)

#### `GpuBackend` enum

Defines available GPU backends:

- `Metal` - Apple Metal (macOS)
- `DirectX12` - DirectX 12 Compute (Windows)
- `Cuda` - NVIDIA CUDA (Linux/Windows)
- `OpenCL` - Cross-platform OpenCL

#### `GpuDevice` trait

Unified interface for GPU operations:

```rust
pub trait GpuDevice: Send + Sync + Debug {
    fn name(&self) -> &str;
    fn create_buffer(&self, size: usize) -> Result<Box<dyn GpuBuffer>>;
    fn add_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()>;
    fn mul_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()>;
    fn sin_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()>;
    // ... more operations
}
```

#### `GpuBuffer` trait

GPU memory management:

```rust
pub trait GpuBuffer: Send + Sync + Debug {
    fn size(&self) -> usize;
    fn read_to_slice(&self, dst: &mut [u8]) -> Result<()>;
    fn write_from_slice(&mut self, src: &[u8]) -> Result<()>;
}
```

### 2. Metal Backend (`metal.rs`) - Fully Implemented

#### Implementation Details

- **Device Discovery**: Uses `metal::Device::system_default()`
- **Shader Compilation**: Compiles Metal Shading Language (MSL) kernels at runtime
- **Buffer Management**: Uses `MTLResourceOptions::StorageModeShared` for unified memory
- **Execution Model**: Command buffers with compute encoders

#### Supported Operations (Metal)

✅ **Implemented:**

- `add_f32` - Element-wise addition
- `mul_f32` - Element-wise multiplication
- `sub_f32` - Element-wise subtraction
- `div_f32` - Element-wise division
- `sin_f32` - Sine function
- `cos_f32` - Cosine function
- `exp_f32` - Exponential function
- `log_f32` - Natural logarithm
- `sqrt_f32` - Square root

⏳ **Planned:**

- `matmul_f32` - Matrix multiplication
- `sum_f32` - Reduction sum
- `max_f32` - Reduction max
- `min_f32` - Reduction min
- `pow_f32` - Power function

#### Metal Shaders (`shaders/metal_kernels.metal`)

Compute kernels written in Metal Shading Language:

```metal
kernel void add_f32(
    device const float* a [[buffer(0)]],
    device const float* b [[buffer(1)]],
    device float* c [[buffer(2)]],
    uint id [[thread_position_in_grid]]
) {
    c[id] = a[id] + b[id];
}
```

### 3. CUDA Backend (`cuda.rs`) - Stub

Prepared for NVIDIA GPU support:

- Uses `cudarc` crate
- Placeholder for CUDA kernel compilation
- Device detection via `CudaDevice::is_available()`

**Status**: Structure in place, needs kernel implementation

### 4. OpenCL Backend (`opencl.rs`) - Stub

Cross-platform fallback:

- Uses `ocl` crate for OpenCL 1.2+ support
- Kernel source in `shaders/opencl_kernels.cl`
- Compatible with AMD, Intel, and NVIDIA GPUs

**Status**: Structure in place, needs kernel execution logic

### 5. DirectX 12 Backend (`directx.rs`) - Stub

Windows native support:

- Will use HLSL compute shaders
- DirectX 12 compute pipeline
- Compatible with all DirectX 12 capable GPUs

**Status**: Placeholder only

### 6. High-Level Operations (`ops.rs`)

Provides convenient ndarray-based API:

```rust
pub struct GpuOps {
    device: Arc<dyn GpuDevice>,
}

impl GpuOps {
    pub fn add_1d(&self, a: &Array1<f32>, b: &Array1<f32>) -> Result<Array1<f32>>;
    pub fn mul_1d(&self, a: &Array1<f32>, b: &Array1<f32>) -> Result<Array1<f32>>;
    pub fn sin_1d(&self, a: &Array1<f32>) -> Result<Array1<f32>>;
    pub fn matmul(&self, a: &Array2<f32>, b: &Array2<f32>) -> Result<Array2<f32>>;
}
```

### 7. Error Handling (`error.rs`)

Comprehensive error types:

```rust
pub enum GpuError {
    DeviceNotFound,
    UnsupportedBackend(String),
    BufferCreationFailed(String),
    CompilationFailed(String),
    ExecutionFailed(String),
    BufferSizeMismatch { expected: usize, actual: usize },
    // Platform-specific errors
    MetalError(String),
    CudaError(String),
    OpenCLError(String),
    DirectXError(String),
}
```

## Usage Examples

### Basic Usage

```rust
use xdl_amp::{GpuContext, ops::GpuOps};
use ndarray::Array1;

// Create GPU context (auto-selects best backend)
let ctx = GpuContext::new()?;
println!("Using: {}", ctx.backend_name());

// Create operations interface
let gpu_ops = GpuOps::new(ctx.device().clone());

// Perform GPU operations
let a = Array1::from_vec(vec![1.0, 2.0, 3.0, 4.0]);
let b = Array1::from_vec(vec![5.0, 6.0, 7.0, 8.0]);
let c = gpu_ops.add_1d(&a, &b)?;
```

### Explicit Backend Selection

```rust
#[cfg(target_os = "macos")]
let ctx = GpuContext::with_preference(Some(GpuBackend::Metal))?;

#[cfg(feature = "cuda")]
let ctx = GpuContext::with_preference(Some(GpuBackend::Cuda))?;
```

## Testing

### Unit Tests

Located in each backend module:

- `lib.rs`: Context creation test
- `metal.rs`: Metal-specific tests (when macOS)
- `cuda.rs`: CUDA-specific tests (when feature enabled)

### Example Programs

**`examples/basic_ops.rs`**
Demonstrates:

- GPU context creation
- Element-wise operations (add, mul)
- Mathematical functions (sin, cos, exp)
- Array operations

Run with:

```bash
cargo run -p xdl-amp --example basic_ops
```

### Verification Results (macOS M-series)

```text
✓ GPU Backend: Metal
Testing GPU operations on arrays of size 1000...

1. Element-wise addition (c = a + b)
   Result (first 5): [0.0, 3.0, 6.0, 9.0, 12.0]

2. Element-wise multiplication (c = a * b)
   Result (first 5): [0.0, 2.0, 8.0, 18.0, 32.0]

3. Sine function
   sin(angles): [0.0, 0.5, 0.707, 0.866, 1.0]

✓ All GPU operations completed successfully!
```

## Performance Characteristics

### Expected Speedup

| Array Size | Expected Speedup | Notes |
|------------|-----------------|-------|
| < 1K       | 0.5x - 1x       | CPU faster (overhead) |
| 1K - 10K   | 1x - 3x         | Break-even point |
| 10K - 100K | 3x - 10x        | Good speedup |
| > 100K     | 10x - 50x       | Excellent speedup |

### Factors Affecting Performance

1. **Data Transfer Overhead**: CPU ↔ GPU memory transfers
2. **Array Layout**: Contiguous arrays perform better
3. **Operation Complexity**: Complex operations benefit more
4. **GPU Hardware**: Modern GPUs show better speedup

### Optimization Tips

1. **Batch Operations**: Combine multiple operations on GPU
2. **Keep Data on GPU**: Minimize transfers
3. **Use Contiguous Arrays**: Better memory access patterns
4. **Profile First**: Measure before optimizing

## Dependencies

### Core Dependencies

- `ndarray` - Array operations
- `bytemuck` - Safe type casting
- `thiserror` - Error handling

### Platform-Specific

- **macOS**: `metal` 0.29, `objc` 0.2
- **CUDA**: `cudarc` 0.11 (optional)
- **OpenCL**: `ocl` 0.19 (optional)
- **Windows**: `windows` 0.58 (optional)

## Building

### macOS

```bash
cargo build -p xdl-amp
```

### Linux with CUDA

```bash
cargo build -p xdl-amp --features cuda
```

### Linux with OpenCL

```bash
cargo build -p xdl-amp --features opencl
```

### Windows

```bash
cargo build -p xdl-amp
# Or with CUDA:
cargo build -p xdl-amp --features cuda
```

## Future Work

### Short Term

1. ✅ Complete Metal backend implementation
2. ⏳ Implement matrix multiplication (all backends)
3. ⏳ Implement reduction operations (all backends)
4. ⏳ Add comprehensive benchmarks

### Medium Term

1. ⏳ Complete CUDA backend
2. ⏳ Complete OpenCL backend
3. ⏳ Complete DirectX 12 backend
4. ⏳ Add double precision support (f64)
5. ⏳ Integration with xdl-stdlib functions

### Long Term

1. ⏳ Automatic CPU/GPU selection based on array size
2. ⏳ Multi-GPU support
3. ⏳ Async/streaming operations
4. ⏳ Custom kernel support
5. ⏳ FFT operations
6. ⏳ Convolution operations

## Integration with XDL

The GPU backend can be integrated into XDL stdlib functions:

```rust
// In xdl-stdlib
use xdl_amp::{GpuContext, ops::GpuOps};

pub fn gpu_sin(x: &Array1<f32>) -> Result<Array1<f32>> {
    let ctx = GpuContext::new()?;
    let ops = GpuOps::new(ctx.device().clone());
    ops.sin_1d(x)
}
```

## Lessons Learned

1. **Platform Abstraction**: Trait-based design allows easy backend addition
2. **Metal Benefits**: Unified memory on Apple Silicon simplifies memory management
3. **Shader Compilation**: Runtime compilation provides flexibility
4. **Error Handling**: Comprehensive errors help debugging GPU issues
5. **Testing Strategy**: Platform-specific conditional compilation for tests

## References

- [Metal Programming Guide](https://developer.apple.com/metal/)
- [CUDA C++ Programming Guide](https://docs.nvidia.com/cuda/)
- [OpenCL Specification](https://www.khronos.org/opencl/)
- [DirectX 12 Compute](https://docs.microsoft.com/en-us/windows/win32/direct3d12/)

## License

GPL-2.0 (same as XDL)

---

**Implementation Status**: ✅ Metal backend fully operational on macOS
**Next Priority**: Matrix multiplication and reduction operations
**Ready for**: Integration into XDL stdlib functions
