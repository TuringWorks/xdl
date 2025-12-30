//! Apple MLX backend for high-performance Apple Silicon acceleration
//!
//! MLX is Apple's array framework designed for machine learning on Apple Silicon.
//! It provides:
//! - Unified memory architecture (no explicit CPU/GPU transfers)
//! - Lazy evaluation with JIT compilation
//! - Optimized operations for M1/M2/M3/M4 chips
//! - Complete FFT and linear algebra support

use crate::backend::{GpuBuffer, GpuDevice};
use crate::error::{GpuError, Result};

use mlx_rs::ops;
use mlx_rs::prelude::*;
use mlx_rs::{Array, Device, Dtype, Stream};
use std::sync::Arc;

/// MLX GPU buffer wrapping an MLX Array
#[derive(Debug)]
pub struct MLXBuffer {
    array: Array,
    size: usize,
}

impl MLXBuffer {
    /// Create a new MLX buffer from raw bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        // Interpret as f32 array
        let floats: &[f32] = bytemuck::cast_slice(data);
        let array = Array::from_slice(floats, &[floats.len() as i32])
            .map_err(|e| GpuError::AllocationFailed(format!("MLX array creation failed: {}", e)))?;
        Ok(Self {
            array,
            size: data.len(),
        })
    }

    /// Get the underlying MLX array
    pub fn array(&self) -> &Array {
        &self.array
    }
}

impl GpuBuffer for MLXBuffer {
    fn size(&self) -> usize {
        self.size
    }

    fn read_to_slice(&self, dst: &mut [u8]) -> Result<()> {
        if dst.len() != self.size {
            return Err(GpuError::BufferSizeMismatch {
                expected: self.size,
                actual: dst.len(),
            });
        }

        // Evaluate the array to materialize it
        self.array
            .eval()
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX eval failed: {}", e)))?;

        // Read data back - MLX uses unified memory so this is efficient
        let floats: Vec<f32> = self
            .array
            .as_slice()
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX read failed: {}", e)))?
            .to_vec();

        let bytes: &[u8] = bytemuck::cast_slice(&floats);
        dst.copy_from_slice(bytes);
        Ok(())
    }

    fn write_from_slice(&mut self, src: &[u8]) -> Result<()> {
        if src.len() != self.size {
            return Err(GpuError::BufferSizeMismatch {
                expected: self.size,
                actual: src.len(),
            });
        }

        let floats: &[f32] = bytemuck::cast_slice(src);
        self.array = Array::from_slice(floats, &[floats.len() as i32])
            .map_err(|e| GpuError::AllocationFailed(format!("MLX array creation failed: {}", e)))?;
        Ok(())
    }
}

/// Apple MLX device for GPU-accelerated computation
#[derive(Debug)]
pub struct MLXDevice {
    device: Device,
    stream: Stream,
}

impl MLXDevice {
    /// Create a new MLX device (uses GPU by default on Apple Silicon)
    pub fn new() -> Result<Self> {
        let device = Device::gpu();
        let stream = Stream::new(&device).map_err(|e| GpuError::DeviceNotFound)?;

        Ok(Self { device, stream })
    }

    /// Check if MLX is available on this system
    pub fn is_available() -> bool {
        // MLX is available on Apple Silicon Macs
        #[cfg(target_arch = "aarch64")]
        {
            true
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            false
        }
    }

    /// Create an MLX array from f32 slice with given shape
    fn array_from_slice(&self, data: &[f32], shape: &[i32]) -> Result<Array> {
        Array::from_slice(data, shape)
            .map_err(|e| GpuError::AllocationFailed(format!("MLX array creation failed: {}", e)))
    }

    /// Evaluate and extract f32 values from array
    fn eval_to_vec(&self, array: &Array) -> Result<Vec<f32>> {
        array
            .eval()
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX eval failed: {}", e)))?;

        array
            .as_slice::<f32>()
            .map(|s| s.to_vec())
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX read failed: {}", e)))
    }

    /// Copy evaluated array to output slice
    fn eval_to_slice(&self, array: &Array, output: &mut [f32]) -> Result<()> {
        let result = self.eval_to_vec(array)?;
        if result.len() != output.len() {
            return Err(GpuError::BufferSizeMismatch {
                expected: output.len(),
                actual: result.len(),
            });
        }
        output.copy_from_slice(&result);
        Ok(())
    }
}

impl GpuDevice for MLXDevice {
    fn name(&self) -> &str {
        "Apple MLX"
    }

    fn create_buffer(&self, size: usize) -> Result<Box<dyn GpuBuffer>> {
        let num_floats = size / std::mem::size_of::<f32>();
        let zeros = vec![0.0f32; num_floats];
        let array = Array::from_slice(&zeros, &[num_floats as i32]).map_err(|e| {
            GpuError::AllocationFailed(format!("MLX buffer creation failed: {}", e))
        })?;

        Ok(Box::new(MLXBuffer { array, size }))
    }

    fn create_buffer_with_data(&self, data: &[u8]) -> Result<Box<dyn GpuBuffer>> {
        Ok(Box::new(MLXBuffer::from_bytes(data)?))
    }

    fn add_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()> {
        let len = a.len() as i32;
        let arr_a = self.array_from_slice(a, &[len])?;
        let arr_b = self.array_from_slice(b, &[len])?;

        let result = ops::add(&arr_a, &arr_b)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX add failed: {}", e)))?;

        self.eval_to_slice(&result, c)
    }

    fn mul_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()> {
        let len = a.len() as i32;
        let arr_a = self.array_from_slice(a, &[len])?;
        let arr_b = self.array_from_slice(b, &[len])?;

        let result = ops::multiply(&arr_a, &arr_b)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX multiply failed: {}", e)))?;

        self.eval_to_slice(&result, c)
    }

    fn sub_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()> {
        let len = a.len() as i32;
        let arr_a = self.array_from_slice(a, &[len])?;
        let arr_b = self.array_from_slice(b, &[len])?;

        let result = ops::subtract(&arr_a, &arr_b)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX subtract failed: {}", e)))?;

        self.eval_to_slice(&result, c)
    }

    fn div_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()> {
        let len = a.len() as i32;
        let arr_a = self.array_from_slice(a, &[len])?;
        let arr_b = self.array_from_slice(b, &[len])?;

        let result = ops::divide(&arr_a, &arr_b)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX divide failed: {}", e)))?;

        self.eval_to_slice(&result, c)
    }

    fn matmul_f32(
        &self,
        a: &[f32],
        b: &[f32],
        c: &mut [f32],
        m: usize,
        n: usize,
        k: usize,
    ) -> Result<()> {
        // A is m x k, B is k x n, C is m x n
        let arr_a = self.array_from_slice(a, &[m as i32, k as i32])?;
        let arr_b = self.array_from_slice(b, &[k as i32, n as i32])?;

        let result = ops::matmul(&arr_a, &arr_b)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX matmul failed: {}", e)))?;

        self.eval_to_slice(&result, c)
    }

    fn sin_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        let len = x.len() as i32;
        let arr_x = self.array_from_slice(x, &[len])?;

        let result = ops::sin(&arr_x)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX sin failed: {}", e)))?;

        self.eval_to_slice(&result, y)
    }

    fn cos_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        let len = x.len() as i32;
        let arr_x = self.array_from_slice(x, &[len])?;

        let result = ops::cos(&arr_x)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX cos failed: {}", e)))?;

        self.eval_to_slice(&result, y)
    }

    fn exp_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        let len = x.len() as i32;
        let arr_x = self.array_from_slice(x, &[len])?;

        let result = ops::exp(&arr_x)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX exp failed: {}", e)))?;

        self.eval_to_slice(&result, y)
    }

    fn log_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        let len = x.len() as i32;
        let arr_x = self.array_from_slice(x, &[len])?;

        let result = ops::log(&arr_x)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX log failed: {}", e)))?;

        self.eval_to_slice(&result, y)
    }

    fn sqrt_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        let len = x.len() as i32;
        let arr_x = self.array_from_slice(x, &[len])?;

        let result = ops::sqrt(&arr_x)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX sqrt failed: {}", e)))?;

        self.eval_to_slice(&result, y)
    }

    fn pow_f32(&self, x: &[f32], p: f32, y: &mut [f32]) -> Result<()> {
        let len = x.len() as i32;
        let arr_x = self.array_from_slice(x, &[len])?;
        let arr_p = Array::from_float(p).map_err(|e| {
            GpuError::AllocationFailed(format!("MLX scalar creation failed: {}", e))
        })?;

        let result = ops::power(&arr_x, &arr_p)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX power failed: {}", e)))?;

        self.eval_to_slice(&result, y)
    }

    fn sum_f32(&self, x: &[f32]) -> Result<f32> {
        let len = x.len() as i32;
        let arr_x = self.array_from_slice(x, &[len])?;

        let result = ops::sum(&arr_x, None, None)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX sum failed: {}", e)))?;

        let values = self.eval_to_vec(&result)?;
        Ok(values[0])
    }

    fn max_f32(&self, x: &[f32]) -> Result<f32> {
        let len = x.len() as i32;
        let arr_x = self.array_from_slice(x, &[len])?;

        let result = ops::max(&arr_x, None, None)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX max failed: {}", e)))?;

        let values = self.eval_to_vec(&result)?;
        Ok(values[0])
    }

    fn min_f32(&self, x: &[f32]) -> Result<f32> {
        let len = x.len() as i32;
        let arr_x = self.array_from_slice(x, &[len])?;

        let result = ops::min(&arr_x, None, None)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX min failed: {}", e)))?;

        let values = self.eval_to_vec(&result)?;
        Ok(values[0])
    }

    fn median_f32(&self, x: &[f32]) -> Result<f32> {
        // MLX doesn't have direct median - use sort + middle element
        let len = x.len() as i32;
        let arr_x = self.array_from_slice(x, &[len])?;

        let sorted = ops::sort(&arr_x, None)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX sort failed: {}", e)))?;

        let values = self.eval_to_vec(&sorted)?;
        let mid = values.len() / 2;

        if values.len() % 2 == 0 {
            Ok((values[mid - 1] + values[mid]) / 2.0)
        } else {
            Ok(values[mid])
        }
    }

    fn variance_f32(&self, x: &[f32]) -> Result<f32> {
        let len = x.len() as i32;
        let arr_x = self.array_from_slice(x, &[len])?;

        let result = ops::var(&arr_x, None, None, None)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX var failed: {}", e)))?;

        let values = self.eval_to_vec(&result)?;
        Ok(values[0])
    }

    fn stddev_f32(&self, x: &[f32]) -> Result<f32> {
        let len = x.len() as i32;
        let arr_x = self.array_from_slice(x, &[len])?;

        let result = ops::std(&arr_x, None, None, None)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX std failed: {}", e)))?;

        let values = self.eval_to_vec(&result)?;
        Ok(values[0])
    }

    fn synchronize(&self) -> Result<()> {
        // MLX uses lazy evaluation - synchronize by evaluating on the stream
        self.stream
            .synchronize()
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX sync failed: {}", e)))
    }
}

// ============================================================================
// Extended MLX Operations (beyond base GpuDevice trait)
// ============================================================================

impl MLXDevice {
    // ------------------------------------------------------------------------
    // FFT Operations
    // ------------------------------------------------------------------------

    /// 1D Fast Fourier Transform
    pub fn fft_1d(&self, x: &[f32]) -> Result<Vec<f32>> {
        use mlx_rs::fft;

        let len = x.len() as i32;
        let arr_x = self.array_from_slice(x, &[len])?;

        let result = fft::fft(&arr_x, None, None)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX fft failed: {}", e)))?;

        // FFT returns complex - extract real/imag interleaved
        self.eval_to_vec(&result)
    }

    /// 1D Inverse FFT
    pub fn ifft_1d(&self, x: &[f32]) -> Result<Vec<f32>> {
        use mlx_rs::fft;

        let len = x.len() as i32;
        let arr_x = self.array_from_slice(x, &[len])?;

        let result = fft::ifft(&arr_x, None, None)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX ifft failed: {}", e)))?;

        self.eval_to_vec(&result)
    }

    /// 2D FFT
    pub fn fft_2d(&self, x: &[f32], rows: usize, cols: usize) -> Result<Vec<f32>> {
        use mlx_rs::fft;

        let arr_x = self.array_from_slice(x, &[rows as i32, cols as i32])?;

        let result = fft::fft2(&arr_x, None, None)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX fft2 failed: {}", e)))?;

        self.eval_to_vec(&result)
    }

    /// Real-valued FFT (more efficient for real input)
    pub fn rfft_1d(&self, x: &[f32]) -> Result<Vec<f32>> {
        use mlx_rs::fft;

        let len = x.len() as i32;
        let arr_x = self.array_from_slice(x, &[len])?;

        let result = fft::rfft(&arr_x, None, None)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX rfft failed: {}", e)))?;

        self.eval_to_vec(&result)
    }

    // ------------------------------------------------------------------------
    // Linear Algebra Operations
    // ------------------------------------------------------------------------

    /// QR decomposition: A = Q @ R
    pub fn qr(&self, a: &[f32], m: usize, n: usize) -> Result<(Vec<f32>, Vec<f32>)> {
        use mlx_rs::linalg;

        let arr_a = self.array_from_slice(a, &[m as i32, n as i32])?;

        let (q, r) = linalg::qr(&arr_a, None)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX qr failed: {}", e)))?;

        let q_vec = self.eval_to_vec(&q)?;
        let r_vec = self.eval_to_vec(&r)?;

        Ok((q_vec, r_vec))
    }

    /// Singular Value Decomposition: A = U @ S @ V^T
    pub fn svd(&self, a: &[f32], m: usize, n: usize) -> Result<(Vec<f32>, Vec<f32>, Vec<f32>)> {
        use mlx_rs::linalg;

        let arr_a = self.array_from_slice(a, &[m as i32, n as i32])?;

        let (u, s, vt) = linalg::svd(&arr_a, None)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX svd failed: {}", e)))?;

        let u_vec = self.eval_to_vec(&u)?;
        let s_vec = self.eval_to_vec(&s)?;
        let vt_vec = self.eval_to_vec(&vt)?;

        Ok((u_vec, s_vec, vt_vec))
    }

    /// Cholesky decomposition: A = L @ L^T
    pub fn cholesky(&self, a: &[f32], n: usize) -> Result<Vec<f32>> {
        use mlx_rs::linalg;

        let arr_a = self.array_from_slice(a, &[n as i32, n as i32])?;

        let result = linalg::cholesky(&arr_a, None, None)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX cholesky failed: {}", e)))?;

        self.eval_to_vec(&result)
    }

    /// Matrix inverse
    pub fn inv(&self, a: &[f32], n: usize) -> Result<Vec<f32>> {
        use mlx_rs::linalg;

        let arr_a = self.array_from_slice(a, &[n as i32, n as i32])?;

        let result = linalg::inv(&arr_a, None)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX inv failed: {}", e)))?;

        self.eval_to_vec(&result)
    }

    /// Solve linear system: A @ x = b
    pub fn solve(&self, a: &[f32], b: &[f32], n: usize) -> Result<Vec<f32>> {
        use mlx_rs::linalg;

        let arr_a = self.array_from_slice(a, &[n as i32, n as i32])?;
        let arr_b = self.array_from_slice(b, &[n as i32])?;

        let result = linalg::solve(&arr_a, &arr_b, None)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX solve failed: {}", e)))?;

        self.eval_to_vec(&result)
    }

    /// Eigenvalues and eigenvectors for symmetric/Hermitian matrices
    pub fn eigh(&self, a: &[f32], n: usize) -> Result<(Vec<f32>, Vec<f32>)> {
        use mlx_rs::linalg;

        let arr_a = self.array_from_slice(a, &[n as i32, n as i32])?;

        let (eigenvalues, eigenvectors) = linalg::eigh(&arr_a, None, None)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX eigh failed: {}", e)))?;

        let evals = self.eval_to_vec(&eigenvalues)?;
        let evecs = self.eval_to_vec(&eigenvectors)?;

        Ok((evals, evecs))
    }

    /// L2 norm
    pub fn norm(&self, x: &[f32]) -> Result<f32> {
        use mlx_rs::linalg;

        let len = x.len() as i32;
        let arr_x = self.array_from_slice(x, &[len])?;

        let result = linalg::norm(&arr_x, None, None, None)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX norm failed: {}", e)))?;

        let values = self.eval_to_vec(&result)?;
        Ok(values[0])
    }

    // ------------------------------------------------------------------------
    // Convolution Operations
    // ------------------------------------------------------------------------

    /// 1D convolution
    pub fn conv1d(
        &self,
        input: &[f32],
        weight: &[f32],
        in_channels: usize,
        out_channels: usize,
        kernel_size: usize,
        batch_size: usize,
        input_length: usize,
    ) -> Result<Vec<f32>> {
        // Input shape: [batch, length, in_channels]
        // Weight shape: [out_channels, kernel_size, in_channels]
        let arr_input = self.array_from_slice(
            input,
            &[batch_size as i32, input_length as i32, in_channels as i32],
        )?;
        let arr_weight = self.array_from_slice(
            weight,
            &[out_channels as i32, kernel_size as i32, in_channels as i32],
        )?;

        let result = ops::conv1d(&arr_input, &arr_weight, None, None, None, None)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX conv1d failed: {}", e)))?;

        self.eval_to_vec(&result)
    }

    /// 2D convolution
    pub fn conv2d(
        &self,
        input: &[f32],
        weight: &[f32],
        in_channels: usize,
        out_channels: usize,
        kernel_h: usize,
        kernel_w: usize,
        batch_size: usize,
        height: usize,
        width: usize,
    ) -> Result<Vec<f32>> {
        // Input shape: [batch, height, width, in_channels]
        // Weight shape: [out_channels, kernel_h, kernel_w, in_channels]
        let arr_input = self.array_from_slice(
            input,
            &[
                batch_size as i32,
                height as i32,
                width as i32,
                in_channels as i32,
            ],
        )?;
        let arr_weight = self.array_from_slice(
            weight,
            &[
                out_channels as i32,
                kernel_h as i32,
                kernel_w as i32,
                in_channels as i32,
            ],
        )?;

        let result = ops::conv2d(&arr_input, &arr_weight, None, None, None, None)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX conv2d failed: {}", e)))?;

        self.eval_to_vec(&result)
    }

    // ------------------------------------------------------------------------
    // Additional Math Operations
    // ------------------------------------------------------------------------

    /// Element-wise absolute value
    pub fn abs(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        let len = x.len() as i32;
        let arr_x = self.array_from_slice(x, &[len])?;

        let result = ops::abs(&arr_x)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX abs failed: {}", e)))?;

        self.eval_to_slice(&result, y)
    }

    /// Element-wise floor
    pub fn floor(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        let len = x.len() as i32;
        let arr_x = self.array_from_slice(x, &[len])?;

        let result = ops::floor(&arr_x)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX floor failed: {}", e)))?;

        self.eval_to_slice(&result, y)
    }

    /// Element-wise ceil
    pub fn ceil(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        let len = x.len() as i32;
        let arr_x = self.array_from_slice(x, &[len])?;

        let result = ops::ceil(&arr_x)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX ceil failed: {}", e)))?;

        self.eval_to_slice(&result, y)
    }

    /// Element-wise tangent
    pub fn tan(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        let len = x.len() as i32;
        let arr_x = self.array_from_slice(x, &[len])?;

        let result = ops::tan(&arr_x)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX tan failed: {}", e)))?;

        self.eval_to_slice(&result, y)
    }

    /// Element-wise hyperbolic sine
    pub fn sinh(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        let len = x.len() as i32;
        let arr_x = self.array_from_slice(x, &[len])?;

        let result = ops::sinh(&arr_x)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX sinh failed: {}", e)))?;

        self.eval_to_slice(&result, y)
    }

    /// Element-wise hyperbolic cosine
    pub fn cosh(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        let len = x.len() as i32;
        let arr_x = self.array_from_slice(x, &[len])?;

        let result = ops::cosh(&arr_x)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX cosh failed: {}", e)))?;

        self.eval_to_slice(&result, y)
    }

    /// Element-wise hyperbolic tangent
    pub fn tanh(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        let len = x.len() as i32;
        let arr_x = self.array_from_slice(x, &[len])?;

        let result = ops::tanh(&arr_x)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX tanh failed: {}", e)))?;

        self.eval_to_slice(&result, y)
    }

    /// Error function
    pub fn erf(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        let len = x.len() as i32;
        let arr_x = self.array_from_slice(x, &[len])?;

        let result = ops::erf(&arr_x)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX erf failed: {}", e)))?;

        self.eval_to_slice(&result, y)
    }

    /// Sigmoid activation
    pub fn sigmoid(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        let len = x.len() as i32;
        let arr_x = self.array_from_slice(x, &[len])?;

        let result = ops::sigmoid(&arr_x)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX sigmoid failed: {}", e)))?;

        self.eval_to_slice(&result, y)
    }

    /// Softmax
    pub fn softmax(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        let len = x.len() as i32;
        let arr_x = self.array_from_slice(x, &[len])?;

        let result = ops::softmax(&arr_x, None)
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX softmax failed: {}", e)))?;

        self.eval_to_slice(&result, y)
    }

    /// Element-wise clip to range [min, max]
    pub fn clip(&self, x: &[f32], min: f32, max: f32, y: &mut [f32]) -> Result<()> {
        let len = x.len() as i32;
        let arr_x = self.array_from_slice(x, &[len])?;
        let arr_min = Array::from_float(min)
            .map_err(|e| GpuError::AllocationFailed(format!("MLX scalar failed: {}", e)))?;
        let arr_max = Array::from_float(max)
            .map_err(|e| GpuError::AllocationFailed(format!("MLX scalar failed: {}", e)))?;

        let result = ops::clip(&arr_x, Some(&arr_min), Some(&arr_max))
            .map_err(|e| GpuError::ExecutionFailed(format!("MLX clip failed: {}", e)))?;

        self.eval_to_slice(&result, y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mlx_availability() {
        // Should be available on Apple Silicon
        #[cfg(target_arch = "aarch64")]
        assert!(MLXDevice::is_available());
    }

    #[test]
    fn test_mlx_basic_ops() -> Result<()> {
        if !MLXDevice::is_available() {
            return Ok(());
        }

        let device = MLXDevice::new()?;
        let a = vec![1.0f32, 2.0, 3.0, 4.0];
        let b = vec![5.0f32, 6.0, 7.0, 8.0];
        let mut c = vec![0.0f32; 4];

        device.add_f32(&a, &b, &mut c)?;
        assert_eq!(c, vec![6.0, 8.0, 10.0, 12.0]);

        device.mul_f32(&a, &b, &mut c)?;
        assert_eq!(c, vec![5.0, 12.0, 21.0, 32.0]);

        Ok(())
    }

    #[test]
    fn test_mlx_reductions() -> Result<()> {
        if !MLXDevice::is_available() {
            return Ok(());
        }

        let device = MLXDevice::new()?;
        let x = vec![1.0f32, 2.0, 3.0, 4.0, 5.0];

        let sum = device.sum_f32(&x)?;
        assert!((sum - 15.0).abs() < 0.001);

        let max = device.max_f32(&x)?;
        assert!((max - 5.0).abs() < 0.001);

        let min = device.min_f32(&x)?;
        assert!((min - 1.0).abs() < 0.001);

        Ok(())
    }

    #[test]
    fn test_mlx_matmul() -> Result<()> {
        if !MLXDevice::is_available() {
            return Ok(());
        }

        let device = MLXDevice::new()?;

        // 2x2 @ 2x2 = 2x2
        // [[1, 2], [3, 4]] @ [[5, 6], [7, 8]] = [[19, 22], [43, 50]]
        let a = vec![1.0f32, 2.0, 3.0, 4.0];
        let b = vec![5.0f32, 6.0, 7.0, 8.0];
        let mut c = vec![0.0f32; 4];

        device.matmul_f32(&a, &b, &mut c, 2, 2, 2)?;

        assert!((c[0] - 19.0).abs() < 0.001);
        assert!((c[1] - 22.0).abs() < 0.001);
        assert!((c[2] - 43.0).abs() < 0.001);
        assert!((c[3] - 50.0).abs() < 0.001);

        Ok(())
    }
}
