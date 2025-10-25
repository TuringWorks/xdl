//! High-level GPU operations

use crate::backend::GpuDevice;
use crate::error::Result;
use ndarray::{Array1, Array2};
use std::sync::Arc;

/// GPU-accelerated array operations
pub struct GpuOps {
    device: Arc<dyn GpuDevice>,
}

impl GpuOps {
    /// Create new GPU operations instance
    pub fn new(device: Arc<dyn GpuDevice>) -> Self {
        Self { device }
    }

    /// Element-wise addition
    pub fn add_1d(&self, a: &Array1<f32>, b: &Array1<f32>) -> Result<Array1<f32>> {
        if a.len() != b.len() {
            return Err(crate::error::GpuError::BufferSizeMismatch {
                expected: a.len(),
                actual: b.len(),
            });
        }

        let mut result = Array1::zeros(a.len());
        self.device.add_f32(
            a.as_slice().unwrap(),
            b.as_slice().unwrap(),
            result.as_slice_mut().unwrap(),
        )?;
        Ok(result)
    }

    /// Element-wise multiplication
    pub fn mul_1d(&self, a: &Array1<f32>, b: &Array1<f32>) -> Result<Array1<f32>> {
        if a.len() != b.len() {
            return Err(crate::error::GpuError::BufferSizeMismatch {
                expected: a.len(),
                actual: b.len(),
            });
        }

        let mut result = Array1::zeros(a.len());
        self.device.mul_f32(
            a.as_slice().unwrap(),
            b.as_slice().unwrap(),
            result.as_slice_mut().unwrap(),
        )?;
        Ok(result)
    }

    /// Element-wise sine
    pub fn sin_1d(&self, a: &Array1<f32>) -> Result<Array1<f32>> {
        let mut result = Array1::zeros(a.len());
        self.device
            .sin_f32(a.as_slice().unwrap(), result.as_slice_mut().unwrap())?;
        Ok(result)
    }

    /// Element-wise cosine
    pub fn cos_1d(&self, a: &Array1<f32>) -> Result<Array1<f32>> {
        let mut result = Array1::zeros(a.len());
        self.device
            .cos_f32(a.as_slice().unwrap(), result.as_slice_mut().unwrap())?;
        Ok(result)
    }

    /// Element-wise exponential
    pub fn exp_1d(&self, a: &Array1<f32>) -> Result<Array1<f32>> {
        let mut result = Array1::zeros(a.len());
        self.device
            .exp_f32(a.as_slice().unwrap(), result.as_slice_mut().unwrap())?;
        Ok(result)
    }

    /// Matrix multiplication
    pub fn matmul(&self, a: &Array2<f32>, b: &Array2<f32>) -> Result<Array2<f32>> {
        let (m, k1) = (a.nrows(), a.ncols());
        let (k2, n) = (b.nrows(), b.ncols());

        if k1 != k2 {
            return Err(crate::error::GpuError::BufferSizeMismatch {
                expected: k1,
                actual: k2,
            });
        }

        let mut result = Array2::zeros((m, n));

        // Convert to contiguous arrays if needed
        let a_contig = a.as_standard_layout();
        let b_contig = b.as_standard_layout();

        self.device.matmul_f32(
            a_contig.as_slice().unwrap(),
            b_contig.as_slice().unwrap(),
            result.as_slice_mut().unwrap(),
            m,
            n,
            k1,
        )?;

        Ok(result)
    }

    /// Sum all elements
    pub fn sum_1d(&self, a: &Array1<f32>) -> Result<f32> {
        self.device.sum_f32(a.as_slice().unwrap())
    }

    /// Maximum element
    pub fn max_1d(&self, a: &Array1<f32>) -> Result<f32> {
        self.device.max_f32(a.as_slice().unwrap())
    }

    /// Minimum element
    pub fn min_1d(&self, a: &Array1<f32>) -> Result<f32> {
        self.device.min_f32(a.as_slice().unwrap())
    }
}
