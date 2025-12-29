//! Metal Performance Shaders (MPS) backend for optimized Apple Silicon operations
//!
//! MPS provides highly optimized implementations of common operations on Apple GPUs.

use crate::backend::{GpuBuffer, GpuDevice};
use crate::error::{GpuError, Result};
use metal::*;

/// MPS GPU buffer
#[derive(Debug)]
pub struct MPSBuffer {
    buffer: metal::Buffer,
    size: usize,
}

impl GpuBuffer for MPSBuffer {
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

        let contents = self.buffer.contents() as *const u8;
        unsafe {
            std::ptr::copy_nonoverlapping(contents, dst.as_mut_ptr(), self.size);
        }
        Ok(())
    }

    fn write_from_slice(&mut self, src: &[u8]) -> Result<()> {
        if src.len() != self.size {
            return Err(GpuError::BufferSizeMismatch {
                expected: self.size,
                actual: src.len(),
            });
        }

        let contents = self.buffer.contents() as *mut u8;
        unsafe {
            std::ptr::copy_nonoverlapping(src.as_ptr(), contents, self.size);
        }
        Ok(())
    }
}

/// Metal Performance Shaders device
#[derive(Debug)]
pub struct MPSDevice {
    device: metal::Device,
    queue: metal::CommandQueue,
}

impl MPSDevice {
    /// Create a new MPS device
    pub fn new() -> Result<Self> {
        let device = metal::Device::system_default().ok_or(GpuError::DeviceNotFound)?;

        let queue = device.new_command_queue();

        Ok(Self { device, queue })
    }

    /// Check if MPS is available
    pub fn is_available() -> bool {
        metal::Device::system_default().is_some()
    }

    /// Execute MPS operation using built-in kernels
    #[allow(dead_code)]
    fn execute_mps_operation<F>(&self, operation: F) -> Result<()>
    where
        F: FnOnce(&metal::CommandBufferRef) -> Result<()>,
    {
        let command_buffer = self.queue.new_command_buffer();
        operation(command_buffer)?;
        command_buffer.commit();
        command_buffer.wait_until_completed();
        Ok(())
    }

    /// Create MPS matrix descriptor
    #[allow(dead_code)]
    fn create_matrix_descriptor(
        &self,
        _rows: usize,
        _cols: usize,
        _data_type: MTLDataType,
    ) -> Result<()> {
        // MPS matrix operations would use MPSMatrixDescriptor
        // This is a placeholder for the actual MPS matrix API
        Ok(())
    }
}

impl GpuDevice for MPSDevice {
    fn name(&self) -> &str {
        "Metal Performance Shaders (MPS)"
    }

    fn create_buffer(&self, size: usize) -> Result<Box<dyn GpuBuffer>> {
        let buffer = self
            .device
            .new_buffer(size as u64, MTLResourceOptions::StorageModeShared);

        Ok(Box::new(MPSBuffer { buffer, size }))
    }

    fn create_buffer_with_data(&self, data: &[u8]) -> Result<Box<dyn GpuBuffer>> {
        let buffer = self.device.new_buffer_with_data(
            data.as_ptr() as *const _,
            data.len() as u64,
            MTLResourceOptions::StorageModeShared,
        );

        Ok(Box::new(MPSBuffer {
            buffer,
            size: data.len(),
        }))
    }

    fn add_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()> {
        // Use optimized MPS vector operations
        // For now, fallback to CPU implementation
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
        // MPS provides highly optimized GEMM via MPSMatrixMultiplication
        // This is a placeholder - real implementation would use MPS matrix operations

        // Naive implementation for now
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

    fn median_f32(&self, x: &[f32]) -> Result<f32> {
        Ok(crate::simd_ops::median_f32(x))
    }

    fn variance_f32(&self, x: &[f32]) -> Result<f32> {
        Ok(crate::simd_ops::variance_f32(x))
    }

    fn stddev_f32(&self, x: &[f32]) -> Result<f32> {
        Ok(crate::simd_ops::stddev_f32(x))
    }

    fn synchronize(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mps_availability() {
        assert!(MPSDevice::is_available());
    }

    #[test]
    fn test_mps_basic_ops() -> Result<()> {
        let device = MPSDevice::new()?;
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![5.0, 6.0, 7.0, 8.0];
        let mut c = vec![0.0; 4];

        device.add_f32(&a, &b, &mut c)?;
        assert_eq!(c, vec![6.0, 8.0, 10.0, 12.0]);

        Ok(())
    }
}
