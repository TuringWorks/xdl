//! High-level GPU operations with smart dispatch, caching, and statistics

use crate::backend::GpuDevice;
use crate::cache::{CacheConfig, CacheManager};
use crate::dispatch::{DispatchConfig, DispatchTarget, SmartDispatcher, cpu_ops};
use crate::error::Result;
use crate::stats::{ExecutionLayer, OpType, GLOBAL_STATS};
use ndarray::{Array1, Array2};
use std::sync::Arc;
use std::time::Instant;

/// Legacy GPU-only operations (for backwards compatibility)
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

    /// Element-wise subtraction
    pub fn sub_1d(&self, a: &Array1<f32>, b: &Array1<f32>) -> Result<Array1<f32>> {
        if a.len() != b.len() {
            return Err(crate::error::GpuError::BufferSizeMismatch {
                expected: a.len(),
                actual: b.len(),
            });
        }

        let mut result = Array1::zeros(a.len());
        self.device.sub_f32(
            a.as_slice().unwrap(),
            b.as_slice().unwrap(),
            result.as_slice_mut().unwrap(),
        )?;
        Ok(result)
    }

    /// Element-wise division
    pub fn div_1d(&self, a: &Array1<f32>, b: &Array1<f32>) -> Result<Array1<f32>> {
        if a.len() != b.len() {
            return Err(crate::error::GpuError::BufferSizeMismatch {
                expected: a.len(),
                actual: b.len(),
            });
        }

        let mut result = Array1::zeros(a.len());
        self.device.div_f32(
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

/// Accelerated operations with smart dispatch, caching, and statistics
pub struct AcceleratedOps {
    device: Arc<dyn GpuDevice>,
    dispatcher: SmartDispatcher,
    cache: CacheManager,
    backend_name: String,
}

impl AcceleratedOps {
    /// Create new accelerated operations with default config
    pub fn new(device: Arc<dyn GpuDevice>, backend_name: &str) -> Self {
        GLOBAL_STATS.set_backend(backend_name);
        Self {
            device,
            dispatcher: SmartDispatcher::with_defaults(),
            cache: CacheManager::with_defaults(),
            backend_name: backend_name.to_string(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        device: Arc<dyn GpuDevice>,
        backend_name: &str,
        dispatch_config: DispatchConfig,
        cache_config: CacheConfig,
    ) -> Self {
        GLOBAL_STATS.set_backend(backend_name);
        Self {
            device,
            dispatcher: SmartDispatcher::new(dispatch_config),
            cache: CacheManager::new(cache_config),
            backend_name: backend_name.to_string(),
        }
    }

    /// Get the backend name
    pub fn backend_name(&self) -> &str {
        &self.backend_name
    }

    /// Enable or disable statistics collection
    pub fn set_stats_enabled(&self, enabled: bool) {
        GLOBAL_STATS.set_enabled(enabled);
    }

    /// Reset all statistics
    pub fn reset_stats(&self) {
        GLOBAL_STATS.reset();
    }

    /// Print statistics report to stdout
    pub fn print_stats(&self) {
        println!("{}", GLOBAL_STATS.format_report());
    }

    /// Get statistics report as string
    pub fn stats_report(&self) -> String {
        GLOBAL_STATS.format_report()
    }

    /// Get dispatch thresholds info
    pub fn thresholds_info(&self) -> String {
        self.dispatcher.get_thresholds().format()
    }

    /// Force all operations to GPU
    pub fn force_gpu(&mut self) {
        self.dispatcher.set_force_gpu(true);
    }

    /// Force all operations to CPU
    pub fn force_cpu(&mut self) {
        self.dispatcher.set_force_cpu(true);
    }

    /// Clear all caches
    pub fn clear_cache(&self) {
        self.cache.clear_all();
    }

    // =========================================================================
    // Binary Element-wise Operations
    // =========================================================================

    /// Element-wise addition with smart dispatch
    pub fn add_1d(&self, a: &Array1<f32>, b: &Array1<f32>) -> Result<Array1<f32>> {
        if a.len() != b.len() {
            return Err(crate::error::GpuError::BufferSizeMismatch {
                expected: a.len(),
                actual: b.len(),
            });
        }

        let elements = a.len();
        let bytes = elements * std::mem::size_of::<f32>() * 3; // a + b + result
        let target = self.dispatcher.dispatch_elementwise(OpType::Add, elements);

        let start = Instant::now();
        let mut result = Array1::zeros(elements);

        match target {
            DispatchTarget::Gpu => {
                self.device.add_f32(
                    a.as_slice().unwrap(),
                    b.as_slice().unwrap(),
                    result.as_slice_mut().unwrap(),
                )?;
                GLOBAL_STATS.record_op(OpType::Add, start.elapsed(), elements, bytes, ExecutionLayer::GpuCompute);
            }
            DispatchTarget::Cpu => {
                cpu_ops::add_f32(
                    a.as_slice().unwrap(),
                    b.as_slice().unwrap(),
                    result.as_slice_mut().unwrap(),
                );
                GLOBAL_STATS.record_op(OpType::Add, start.elapsed(), elements, bytes, ExecutionLayer::Cpu);
            }
        }

        Ok(result)
    }

    /// Element-wise subtraction with smart dispatch
    pub fn sub_1d(&self, a: &Array1<f32>, b: &Array1<f32>) -> Result<Array1<f32>> {
        if a.len() != b.len() {
            return Err(crate::error::GpuError::BufferSizeMismatch {
                expected: a.len(),
                actual: b.len(),
            });
        }

        let elements = a.len();
        let bytes = elements * std::mem::size_of::<f32>() * 3;
        let target = self.dispatcher.dispatch_elementwise(OpType::Sub, elements);

        let start = Instant::now();
        let mut result = Array1::zeros(elements);

        match target {
            DispatchTarget::Gpu => {
                self.device.sub_f32(
                    a.as_slice().unwrap(),
                    b.as_slice().unwrap(),
                    result.as_slice_mut().unwrap(),
                )?;
                GLOBAL_STATS.record_op(OpType::Sub, start.elapsed(), elements, bytes, ExecutionLayer::GpuCompute);
            }
            DispatchTarget::Cpu => {
                cpu_ops::sub_f32(
                    a.as_slice().unwrap(),
                    b.as_slice().unwrap(),
                    result.as_slice_mut().unwrap(),
                );
                GLOBAL_STATS.record_op(OpType::Sub, start.elapsed(), elements, bytes, ExecutionLayer::Cpu);
            }
        }

        Ok(result)
    }

    /// Element-wise multiplication with smart dispatch
    pub fn mul_1d(&self, a: &Array1<f32>, b: &Array1<f32>) -> Result<Array1<f32>> {
        if a.len() != b.len() {
            return Err(crate::error::GpuError::BufferSizeMismatch {
                expected: a.len(),
                actual: b.len(),
            });
        }

        let elements = a.len();
        let bytes = elements * std::mem::size_of::<f32>() * 3;
        let target = self.dispatcher.dispatch_elementwise(OpType::Mul, elements);

        let start = Instant::now();
        let mut result = Array1::zeros(elements);

        match target {
            DispatchTarget::Gpu => {
                self.device.mul_f32(
                    a.as_slice().unwrap(),
                    b.as_slice().unwrap(),
                    result.as_slice_mut().unwrap(),
                )?;
                GLOBAL_STATS.record_op(OpType::Mul, start.elapsed(), elements, bytes, ExecutionLayer::GpuCompute);
            }
            DispatchTarget::Cpu => {
                cpu_ops::mul_f32(
                    a.as_slice().unwrap(),
                    b.as_slice().unwrap(),
                    result.as_slice_mut().unwrap(),
                );
                GLOBAL_STATS.record_op(OpType::Mul, start.elapsed(), elements, bytes, ExecutionLayer::Cpu);
            }
        }

        Ok(result)
    }

    /// Element-wise division with smart dispatch
    pub fn div_1d(&self, a: &Array1<f32>, b: &Array1<f32>) -> Result<Array1<f32>> {
        if a.len() != b.len() {
            return Err(crate::error::GpuError::BufferSizeMismatch {
                expected: a.len(),
                actual: b.len(),
            });
        }

        let elements = a.len();
        let bytes = elements * std::mem::size_of::<f32>() * 3;
        let target = self.dispatcher.dispatch_elementwise(OpType::Div, elements);

        let start = Instant::now();
        let mut result = Array1::zeros(elements);

        match target {
            DispatchTarget::Gpu => {
                self.device.div_f32(
                    a.as_slice().unwrap(),
                    b.as_slice().unwrap(),
                    result.as_slice_mut().unwrap(),
                )?;
                GLOBAL_STATS.record_op(OpType::Div, start.elapsed(), elements, bytes, ExecutionLayer::GpuCompute);
            }
            DispatchTarget::Cpu => {
                cpu_ops::div_f32(
                    a.as_slice().unwrap(),
                    b.as_slice().unwrap(),
                    result.as_slice_mut().unwrap(),
                );
                GLOBAL_STATS.record_op(OpType::Div, start.elapsed(), elements, bytes, ExecutionLayer::Cpu);
            }
        }

        Ok(result)
    }

    // =========================================================================
    // Unary Element-wise Operations
    // =========================================================================

    /// Element-wise sine with smart dispatch
    pub fn sin_1d(&self, a: &Array1<f32>) -> Result<Array1<f32>> {
        let elements = a.len();
        let bytes = elements * std::mem::size_of::<f32>() * 2;
        let target = self.dispatcher.dispatch_elementwise(OpType::Sin, elements);

        let start = Instant::now();
        let mut result = Array1::zeros(elements);

        match target {
            DispatchTarget::Gpu => {
                self.device.sin_f32(a.as_slice().unwrap(), result.as_slice_mut().unwrap())?;
                GLOBAL_STATS.record_op(OpType::Sin, start.elapsed(), elements, bytes, ExecutionLayer::GpuCompute);
            }
            DispatchTarget::Cpu => {
                cpu_ops::sin_f32(a.as_slice().unwrap(), result.as_slice_mut().unwrap());
                GLOBAL_STATS.record_op(OpType::Sin, start.elapsed(), elements, bytes, ExecutionLayer::Cpu);
            }
        }

        Ok(result)
    }

    /// Element-wise cosine with smart dispatch
    pub fn cos_1d(&self, a: &Array1<f32>) -> Result<Array1<f32>> {
        let elements = a.len();
        let bytes = elements * std::mem::size_of::<f32>() * 2;
        let target = self.dispatcher.dispatch_elementwise(OpType::Cos, elements);

        let start = Instant::now();
        let mut result = Array1::zeros(elements);

        match target {
            DispatchTarget::Gpu => {
                self.device.cos_f32(a.as_slice().unwrap(), result.as_slice_mut().unwrap())?;
                GLOBAL_STATS.record_op(OpType::Cos, start.elapsed(), elements, bytes, ExecutionLayer::GpuCompute);
            }
            DispatchTarget::Cpu => {
                cpu_ops::cos_f32(a.as_slice().unwrap(), result.as_slice_mut().unwrap());
                GLOBAL_STATS.record_op(OpType::Cos, start.elapsed(), elements, bytes, ExecutionLayer::Cpu);
            }
        }

        Ok(result)
    }

    /// Element-wise exponential with smart dispatch
    pub fn exp_1d(&self, a: &Array1<f32>) -> Result<Array1<f32>> {
        let elements = a.len();
        let bytes = elements * std::mem::size_of::<f32>() * 2;
        let target = self.dispatcher.dispatch_elementwise(OpType::Exp, elements);

        let start = Instant::now();
        let mut result = Array1::zeros(elements);

        match target {
            DispatchTarget::Gpu => {
                self.device.exp_f32(a.as_slice().unwrap(), result.as_slice_mut().unwrap())?;
                GLOBAL_STATS.record_op(OpType::Exp, start.elapsed(), elements, bytes, ExecutionLayer::GpuCompute);
            }
            DispatchTarget::Cpu => {
                cpu_ops::exp_f32(a.as_slice().unwrap(), result.as_slice_mut().unwrap());
                GLOBAL_STATS.record_op(OpType::Exp, start.elapsed(), elements, bytes, ExecutionLayer::Cpu);
            }
        }

        Ok(result)
    }

    /// Element-wise log with smart dispatch
    pub fn log_1d(&self, a: &Array1<f32>) -> Result<Array1<f32>> {
        let elements = a.len();
        let bytes = elements * std::mem::size_of::<f32>() * 2;
        let target = self.dispatcher.dispatch_elementwise(OpType::Log, elements);

        let start = Instant::now();
        let mut result = Array1::zeros(elements);

        match target {
            DispatchTarget::Gpu => {
                self.device.log_f32(a.as_slice().unwrap(), result.as_slice_mut().unwrap())?;
                GLOBAL_STATS.record_op(OpType::Log, start.elapsed(), elements, bytes, ExecutionLayer::GpuCompute);
            }
            DispatchTarget::Cpu => {
                cpu_ops::log_f32(a.as_slice().unwrap(), result.as_slice_mut().unwrap());
                GLOBAL_STATS.record_op(OpType::Log, start.elapsed(), elements, bytes, ExecutionLayer::Cpu);
            }
        }

        Ok(result)
    }

    /// Element-wise sqrt with smart dispatch
    pub fn sqrt_1d(&self, a: &Array1<f32>) -> Result<Array1<f32>> {
        let elements = a.len();
        let bytes = elements * std::mem::size_of::<f32>() * 2;
        let target = self.dispatcher.dispatch_elementwise(OpType::Sqrt, elements);

        let start = Instant::now();
        let mut result = Array1::zeros(elements);

        match target {
            DispatchTarget::Gpu => {
                self.device.sqrt_f32(a.as_slice().unwrap(), result.as_slice_mut().unwrap())?;
                GLOBAL_STATS.record_op(OpType::Sqrt, start.elapsed(), elements, bytes, ExecutionLayer::GpuCompute);
            }
            DispatchTarget::Cpu => {
                cpu_ops::sqrt_f32(a.as_slice().unwrap(), result.as_slice_mut().unwrap());
                GLOBAL_STATS.record_op(OpType::Sqrt, start.elapsed(), elements, bytes, ExecutionLayer::Cpu);
            }
        }

        Ok(result)
    }

    // =========================================================================
    // Reduction Operations
    // =========================================================================

    /// Sum reduction with smart dispatch
    pub fn sum_1d(&self, a: &Array1<f32>) -> Result<f32> {
        let elements = a.len();
        let bytes = elements * std::mem::size_of::<f32>();
        let target = self.dispatcher.dispatch_reduction(OpType::Sum, elements);

        let start = Instant::now();
        let result = match target {
            DispatchTarget::Gpu => {
                let r = self.device.sum_f32(a.as_slice().unwrap())?;
                GLOBAL_STATS.record_op(OpType::Sum, start.elapsed(), elements, bytes, ExecutionLayer::GpuCompute);
                r
            }
            DispatchTarget::Cpu => {
                let r = cpu_ops::sum_f32(a.as_slice().unwrap());
                GLOBAL_STATS.record_op(OpType::Sum, start.elapsed(), elements, bytes, ExecutionLayer::Cpu);
                r
            }
        };

        Ok(result)
    }

    /// Max reduction with smart dispatch
    pub fn max_1d(&self, a: &Array1<f32>) -> Result<f32> {
        let elements = a.len();
        let bytes = elements * std::mem::size_of::<f32>();
        let target = self.dispatcher.dispatch_reduction(OpType::Max, elements);

        let start = Instant::now();
        let result = match target {
            DispatchTarget::Gpu => {
                let r = self.device.max_f32(a.as_slice().unwrap())?;
                GLOBAL_STATS.record_op(OpType::Max, start.elapsed(), elements, bytes, ExecutionLayer::GpuCompute);
                r
            }
            DispatchTarget::Cpu => {
                let r = cpu_ops::max_f32(a.as_slice().unwrap());
                GLOBAL_STATS.record_op(OpType::Max, start.elapsed(), elements, bytes, ExecutionLayer::Cpu);
                r
            }
        };

        Ok(result)
    }

    /// Min reduction with smart dispatch
    pub fn min_1d(&self, a: &Array1<f32>) -> Result<f32> {
        let elements = a.len();
        let bytes = elements * std::mem::size_of::<f32>();
        let target = self.dispatcher.dispatch_reduction(OpType::Min, elements);

        let start = Instant::now();
        let result = match target {
            DispatchTarget::Gpu => {
                let r = self.device.min_f32(a.as_slice().unwrap())?;
                GLOBAL_STATS.record_op(OpType::Min, start.elapsed(), elements, bytes, ExecutionLayer::GpuCompute);
                r
            }
            DispatchTarget::Cpu => {
                let r = cpu_ops::min_f32(a.as_slice().unwrap());
                GLOBAL_STATS.record_op(OpType::Min, start.elapsed(), elements, bytes, ExecutionLayer::Cpu);
                r
            }
        };

        Ok(result)
    }

    // =========================================================================
    // Matrix Operations
    // =========================================================================

    /// Matrix multiplication with smart dispatch
    pub fn matmul(&self, a: &Array2<f32>, b: &Array2<f32>) -> Result<Array2<f32>> {
        let (m, k1) = (a.nrows(), a.ncols());
        let (k2, n) = (b.nrows(), b.ncols());

        if k1 != k2 {
            return Err(crate::error::GpuError::BufferSizeMismatch {
                expected: k1,
                actual: k2,
            });
        }

        let k = k1;
        let elements = m * n;
        let bytes = (m * k + k * n + m * n) * std::mem::size_of::<f32>();
        let target = self.dispatcher.dispatch_matmul(m, n, k);

        let start = Instant::now();
        let mut result = Array2::zeros((m, n));

        // Convert to contiguous arrays if needed
        let a_contig = a.as_standard_layout();
        let b_contig = b.as_standard_layout();

        match target {
            DispatchTarget::Gpu => {
                self.device.matmul_f32(
                    a_contig.as_slice().unwrap(),
                    b_contig.as_slice().unwrap(),
                    result.as_slice_mut().unwrap(),
                    m, n, k,
                )?;
                GLOBAL_STATS.record_op(OpType::MatMul, start.elapsed(), elements, bytes, ExecutionLayer::GpuCompute);
            }
            DispatchTarget::Cpu => {
                cpu_ops::matmul_f32(
                    a_contig.as_slice().unwrap(),
                    b_contig.as_slice().unwrap(),
                    result.as_slice_mut().unwrap(),
                    m, n, k,
                );
                GLOBAL_STATS.record_op(OpType::MatMul, start.elapsed(), elements, bytes, ExecutionLayer::Cpu);
            }
        }

        Ok(result)
    }

    // =========================================================================
    // 2D Array Operations
    // =========================================================================

    /// Element-wise addition for 2D arrays
    pub fn add_2d(&self, a: &Array2<f32>, b: &Array2<f32>) -> Result<Array2<f32>> {
        if a.shape() != b.shape() {
            return Err(crate::error::GpuError::BufferSizeMismatch {
                expected: a.len(),
                actual: b.len(),
            });
        }

        let shape = (a.nrows(), a.ncols());
        let a_flat = Array1::from_vec(a.as_standard_layout().iter().cloned().collect());
        let b_flat = Array1::from_vec(b.as_standard_layout().iter().cloned().collect());

        let result_flat = self.add_1d(&a_flat, &b_flat)?;
        Ok(result_flat.into_shape(shape).unwrap())
    }

    /// Element-wise multiplication for 2D arrays
    pub fn mul_2d(&self, a: &Array2<f32>, b: &Array2<f32>) -> Result<Array2<f32>> {
        if a.shape() != b.shape() {
            return Err(crate::error::GpuError::BufferSizeMismatch {
                expected: a.len(),
                actual: b.len(),
            });
        }

        let shape = (a.nrows(), a.ncols());
        let a_flat = Array1::from_vec(a.as_standard_layout().iter().cloned().collect());
        let b_flat = Array1::from_vec(b.as_standard_layout().iter().cloned().collect());

        let result_flat = self.mul_1d(&a_flat, &b_flat)?;
        Ok(result_flat.into_shape(shape).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a GPU backend to be available
    // They will be skipped if no backend is present

    #[test]
    fn test_cpu_ops_directly() {
        let a = Array1::from_vec(vec![1.0f32, 2.0, 3.0, 4.0]);
        let b = Array1::from_vec(vec![5.0f32, 6.0, 7.0, 8.0]);
        let mut c = vec![0.0f32; 4];

        cpu_ops::add_f32(a.as_slice().unwrap(), b.as_slice().unwrap(), &mut c);
        assert_eq!(c, vec![6.0, 8.0, 10.0, 12.0]);
    }

    #[test]
    fn test_dispatch_decisions() {
        let dispatcher = SmartDispatcher::with_defaults();

        // Small array -> CPU
        let target = dispatcher.dispatch_elementwise(OpType::Add, 100);
        assert_eq!(target, DispatchTarget::Cpu);

        // Large array -> GPU
        let target = dispatcher.dispatch_elementwise(OpType::Add, 100_000);
        assert_eq!(target, DispatchTarget::Gpu);
    }
}
