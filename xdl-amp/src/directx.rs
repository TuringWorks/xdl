//! DirectX 12 Compute Shader backend for Windows GPU acceleration
//!
//! This backend delegates operations to DirectML for GPU-accelerated computation.
//! DirectML provides optimized implementations that work on any DirectX 12-capable GPU.

use crate::backend::{GpuBuffer, GpuDevice};
use crate::error::{GpuError, Result};

#[cfg(all(target_os = "windows", feature = "directml"))]
use crate::directml::DirectMLDevice;

/// DirectX 12 GPU buffer - delegates to DirectML buffer implementation
#[derive(Debug)]
pub struct DirectXBuffer {
    data: Vec<u8>,
    size: usize,
}

impl GpuBuffer for DirectXBuffer {
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

/// DirectX 12 GPU device
///
/// When DirectML feature is enabled, delegates all operations to DirectML for
/// optimal GPU acceleration. Otherwise falls back to CPU SIMD operations.
#[derive(Debug)]
pub struct DirectXDevice {
    device_name: String,
    #[cfg(all(target_os = "windows", feature = "directml"))]
    dml_device: DirectMLDevice,
}

impl DirectXDevice {
    /// Create a new DirectX 12 device
    pub fn new() -> Result<Self> {
        #[cfg(all(target_os = "windows", feature = "directml"))]
        {
            // Use DirectML for GPU-accelerated operations
            let dml_device = DirectMLDevice::new()?;
            Ok(Self {
                device_name: "DirectX 12 (via DirectML)".to_string(),
                dml_device,
            })
        }

        #[cfg(not(all(target_os = "windows", feature = "directml")))]
        {
            Ok(Self {
                device_name: "DirectX 12 (CPU fallback)".to_string(),
            })
        }
    }
}

impl GpuDevice for DirectXDevice {
    fn name(&self) -> &str {
        &self.device_name
    }

    fn create_buffer(&self, size: usize) -> Result<Box<dyn GpuBuffer>> {
        Ok(Box::new(DirectXBuffer {
            data: vec![0u8; size],
            size,
        }))
    }

    fn create_buffer_with_data(&self, data: &[u8]) -> Result<Box<dyn GpuBuffer>> {
        Ok(Box::new(DirectXBuffer {
            data: data.to_vec(),
            size: data.len(),
        }))
    }

    fn add_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()> {
        #[cfg(all(target_os = "windows", feature = "directml"))]
        {
            self.dml_device.add_f32(a, b, c)
        }
        #[cfg(not(all(target_os = "windows", feature = "directml")))]
        {
            crate::simd_ops::add_f32(a, b, c);
            Ok(())
        }
    }

    fn mul_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()> {
        #[cfg(all(target_os = "windows", feature = "directml"))]
        {
            self.dml_device.mul_f32(a, b, c)
        }
        #[cfg(not(all(target_os = "windows", feature = "directml")))]
        {
            crate::simd_ops::mul_f32(a, b, c);
            Ok(())
        }
    }

    fn sub_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()> {
        #[cfg(all(target_os = "windows", feature = "directml"))]
        {
            self.dml_device.sub_f32(a, b, c)
        }
        #[cfg(not(all(target_os = "windows", feature = "directml")))]
        {
            crate::simd_ops::sub_f32(a, b, c);
            Ok(())
        }
    }

    fn div_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()> {
        #[cfg(all(target_os = "windows", feature = "directml"))]
        {
            self.dml_device.div_f32(a, b, c)
        }
        #[cfg(not(all(target_os = "windows", feature = "directml")))]
        {
            crate::simd_ops::div_f32(a, b, c);
            Ok(())
        }
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
        #[cfg(all(target_os = "windows", feature = "directml"))]
        {
            self.dml_device.matmul_f32(a, b, c, m, n, k)
        }
        #[cfg(not(all(target_os = "windows", feature = "directml")))]
        {
            crate::simd_ops::matmul_f32(a, b, c, m, n, k);
            Ok(())
        }
    }

    fn sin_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        #[cfg(all(target_os = "windows", feature = "directml"))]
        {
            self.dml_device.sin_f32(x, y)
        }
        #[cfg(not(all(target_os = "windows", feature = "directml")))]
        {
            crate::simd_ops::sin_f32(x, y);
            Ok(())
        }
    }

    fn cos_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        #[cfg(all(target_os = "windows", feature = "directml"))]
        {
            self.dml_device.cos_f32(x, y)
        }
        #[cfg(not(all(target_os = "windows", feature = "directml")))]
        {
            crate::simd_ops::cos_f32(x, y);
            Ok(())
        }
    }

    fn exp_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        #[cfg(all(target_os = "windows", feature = "directml"))]
        {
            self.dml_device.exp_f32(x, y)
        }
        #[cfg(not(all(target_os = "windows", feature = "directml")))]
        {
            crate::simd_ops::exp_f32(x, y);
            Ok(())
        }
    }

    fn log_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        #[cfg(all(target_os = "windows", feature = "directml"))]
        {
            self.dml_device.log_f32(x, y)
        }
        #[cfg(not(all(target_os = "windows", feature = "directml")))]
        {
            crate::simd_ops::log_f32(x, y);
            Ok(())
        }
    }

    fn sqrt_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        #[cfg(all(target_os = "windows", feature = "directml"))]
        {
            self.dml_device.sqrt_f32(x, y)
        }
        #[cfg(not(all(target_os = "windows", feature = "directml")))]
        {
            crate::simd_ops::sqrt_f32(x, y);
            Ok(())
        }
    }

    fn pow_f32(&self, x: &[f32], p: f32, y: &mut [f32]) -> Result<()> {
        #[cfg(all(target_os = "windows", feature = "directml"))]
        {
            self.dml_device.pow_f32(x, p, y)
        }
        #[cfg(not(all(target_os = "windows", feature = "directml")))]
        {
            crate::simd_ops::pow_f32(x, p, y);
            Ok(())
        }
    }

    fn sum_f32(&self, x: &[f32]) -> Result<f32> {
        #[cfg(all(target_os = "windows", feature = "directml"))]
        {
            self.dml_device.sum_f32(x)
        }
        #[cfg(not(all(target_os = "windows", feature = "directml")))]
        {
            Ok(crate::simd_ops::sum_f32(x))
        }
    }

    fn max_f32(&self, x: &[f32]) -> Result<f32> {
        #[cfg(all(target_os = "windows", feature = "directml"))]
        {
            self.dml_device.max_f32(x)
        }
        #[cfg(not(all(target_os = "windows", feature = "directml")))]
        {
            Ok(crate::simd_ops::max_f32(x))
        }
    }

    fn min_f32(&self, x: &[f32]) -> Result<f32> {
        #[cfg(all(target_os = "windows", feature = "directml"))]
        {
            self.dml_device.min_f32(x)
        }
        #[cfg(not(all(target_os = "windows", feature = "directml")))]
        {
            Ok(crate::simd_ops::min_f32(x))
        }
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
        #[cfg(all(target_os = "windows", feature = "directml"))]
        {
            self.dml_device.synchronize()
        }
        #[cfg(not(all(target_os = "windows", feature = "directml")))]
        {
            Ok(())
        }
    }
}
