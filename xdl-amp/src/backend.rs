//! GPU backend abstraction layer

use crate::error::Result;
use std::fmt::Debug;

/// Supported GPU/ML acceleration backends
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuBackend {
    /// Apple MLX - High-performance ML framework for Apple Silicon (macOS)
    MLX,
    /// Apple Metal - Low-level GPU (macOS, iOS)
    Metal,
    /// Metal Performance Shaders - Optimized operations (macOS, iOS)
    MetalPerformanceShaders,
    /// CoreML with Apple Neural Engine (macOS, iOS)
    CoreML,
    /// DirectX 12 Compute Shaders (Windows)
    DirectX12,
    /// DirectML - ML acceleration on DirectX (Windows)
    DirectML,
    /// NVIDIA CUDA (Linux, Windows)
    Cuda,
    /// NVIDIA cuDNN - Deep learning library (Linux, Windows)
    CuDNN,
    /// AMD ROCm (Linux)
    ROCm,
    /// OpenCL (cross-platform fallback)
    OpenCL,
    /// Vulkan (cross-platform)
    Vulkan,
    /// ONNX Runtime (cross-platform ML)
    OnnxRuntime,
}

/// GPU buffer handle that stores data on the GPU
pub trait GpuBuffer: Send + Sync + Debug {
    /// Get the size of the buffer in bytes
    fn size(&self) -> usize;

    /// Read data from GPU to CPU
    fn read_to_slice(&self, dst: &mut [u8]) -> Result<()>;

    /// Write data from CPU to GPU
    fn write_from_slice(&mut self, src: &[u8]) -> Result<()>;
}

/// GPU device abstraction
pub trait GpuDevice: Send + Sync + Debug {
    /// Get device name
    fn name(&self) -> &str;

    /// Create a buffer on the GPU
    fn create_buffer(&self, size: usize) -> Result<Box<dyn GpuBuffer>>;

    /// Create a buffer initialized with data
    fn create_buffer_with_data(&self, data: &[u8]) -> Result<Box<dyn GpuBuffer>>;

    /// Element-wise addition: c = a + b
    fn add_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()>;

    /// Element-wise multiplication: c = a * b
    fn mul_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()>;

    /// Element-wise subtraction: c = a - b
    fn sub_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()>;

    /// Element-wise division: c = a / b
    fn div_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()>;

    /// Matrix multiplication: c = a @ b
    fn matmul_f32(
        &self,
        a: &[f32],
        b: &[f32],
        c: &mut [f32],
        m: usize,
        n: usize,
        k: usize,
    ) -> Result<()>;

    /// Compute sine: y = sin(x)
    fn sin_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()>;

    /// Compute cosine: y = cos(x)
    fn cos_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()>;

    /// Compute exponential: y = exp(x)
    fn exp_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()>;

    /// Compute natural logarithm: y = log(x)
    fn log_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()>;

    /// Compute square root: y = sqrt(x)
    fn sqrt_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()>;

    /// Compute power: y = x^p
    fn pow_f32(&self, x: &[f32], p: f32, y: &mut [f32]) -> Result<()>;

    /// Sum reduction: returns sum of all elements
    fn sum_f32(&self, x: &[f32]) -> Result<f32>;

    /// Max reduction: returns maximum element
    fn max_f32(&self, x: &[f32]) -> Result<f32>;

    /// Min reduction: returns minimum element
    fn min_f32(&self, x: &[f32]) -> Result<f32>;

    /// Median: returns the median value (middle element of sorted array)
    fn median_f32(&self, x: &[f32]) -> Result<f32>;

    /// Variance: returns the variance of elements
    fn variance_f32(&self, x: &[f32]) -> Result<f32>;

    /// Standard deviation: returns the standard deviation of elements
    fn stddev_f32(&self, x: &[f32]) -> Result<f32>;

    /// Synchronize device (wait for all operations to complete)
    fn synchronize(&self) -> Result<()>;
}

/// Helper trait to work with ndarray
pub trait GpuArrayOps {
    /// Add two arrays on GPU
    fn gpu_add(&self, other: &Self) -> Result<Self>
    where
        Self: Sized;

    /// Multiply two arrays on GPU
    fn gpu_mul(&self, other: &Self) -> Result<Self>
    where
        Self: Sized;

    /// Compute sine on GPU
    fn gpu_sin(&self) -> Result<Self>
    where
        Self: Sized;

    /// Compute cosine on GPU
    fn gpu_cos(&self) -> Result<Self>
    where
        Self: Sized;

    /// Matrix multiplication on GPU
    fn gpu_matmul(&self, other: &Self) -> Result<Self>
    where
        Self: Sized;
}
