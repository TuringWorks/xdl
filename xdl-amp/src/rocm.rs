//! AMD ROCm backend for AMD GPU acceleration

use crate::backend::{GpuBuffer, GpuDevice};
use crate::error::{GpuError, Result};

/// ROCm GPU buffer
#[derive(Debug)]
pub struct ROCmBuffer {
    data: Vec<u8>,
    size: usize,
}

impl GpuBuffer for ROCmBuffer {
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
        dst.copy_from_slice(&self.data);
        Ok(())
    }

    fn write_from_slice(&mut self, src: &[u8]) -> Result<()> {
        if src.len() != self.size {
            return Err(GpuError::BufferSizeMismatch {
                expected: self.size,
                actual: src.len(),
            });
        }
        self.data.copy_from_slice(src);
        Ok(())
    }
}

/// ROCm GPU device
#[derive(Debug)]
pub struct ROCmDevice {
    device_name: String,
}

impl ROCmDevice {
    /// Create a new ROCm device
    pub fn new() -> Result<Self> {
        #[cfg(feature = "rocm")]
        {
            // Would initialize ROCm/HIP here
            Ok(Self {
                device_name: "AMD ROCm".to_string(),
            })
        }

        #[cfg(not(feature = "rocm"))]
        {
            Err(GpuError::UnsupportedBackend("ROCm not enabled".to_string()))
        }
    }

    /// Check if ROCm is available
    pub fn is_available() -> bool {
        cfg!(feature = "rocm")
    }
}

impl GpuDevice for ROCmDevice {
    fn name(&self) -> &str {
        &self.device_name
    }

    fn create_buffer(&self, size: usize) -> Result<Box<dyn GpuBuffer>> {
        Ok(Box::new(ROCmBuffer {
            data: vec![0u8; size],
            size,
        }))
    }

    fn create_buffer_with_data(&self, data: &[u8]) -> Result<Box<dyn GpuBuffer>> {
        Ok(Box::new(ROCmBuffer {
            data: data.to_vec(),
            size: data.len(),
        }))
    }

    fn add_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()> {
        for i in 0..a.len() {
            c[i] = a[i] + b[i];
        }
        Ok(())
    }

    fn mul_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()> {
        for i in 0..a.len() {
            c[i] = a[i] * b[i];
        }
        Ok(())
    }

    fn sub_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()> {
        for i in 0..a.len() {
            c[i] = a[i] - b[i];
        }
        Ok(())
    }

    fn div_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()> {
        for i in 0..a.len() {
            c[i] = a[i] / b[i];
        }
        Ok(())
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
        for i in 0..m {
            for j in 0..n {
                let mut sum = 0.0;
                for p in 0..k {
                    sum += a[i * k + p] * b[p * n + j];
                }
                c[i * n + j] = sum;
            }
        }
        Ok(())
    }

    fn sin_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        for i in 0..x.len() {
            y[i] = x[i].sin();
        }
        Ok(())
    }

    fn cos_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        for i in 0..x.len() {
            y[i] = x[i].cos();
        }
        Ok(())
    }

    fn exp_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        for i in 0..x.len() {
            y[i] = x[i].exp();
        }
        Ok(())
    }

    fn log_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        for i in 0..x.len() {
            y[i] = x[i].ln();
        }
        Ok(())
    }

    fn sqrt_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        for i in 0..x.len() {
            y[i] = x[i].sqrt();
        }
        Ok(())
    }

    fn pow_f32(&self, x: &[f32], p: f32, y: &mut [f32]) -> Result<()> {
        for i in 0..x.len() {
            y[i] = x[i].powf(p);
        }
        Ok(())
    }

    fn sum_f32(&self, x: &[f32]) -> Result<f32> {
        Ok(x.iter().sum())
    }

    fn max_f32(&self, x: &[f32]) -> Result<f32> {
        x.iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .ok_or(GpuError::ExecutionFailed("Empty array".to_string()))
    }

    fn min_f32(&self, x: &[f32]) -> Result<f32> {
        x.iter()
            .copied()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .ok_or(GpuError::ExecutionFailed("Empty array".to_string()))
    }

    fn synchronize(&self) -> Result<()> {
        Ok(())
    }
}
