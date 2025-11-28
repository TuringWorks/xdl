//! Smart dispatch logic for GPU vs CPU execution
//!
//! Decides at runtime whether to execute operations on GPU or CPU
//! based on array size, operation type, and historical performance.

use crate::stats::{ExecutionLayer, OpType, GLOBAL_STATS};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Dispatch thresholds configuration
#[derive(Debug, Clone)]
pub struct DispatchConfig {
    /// Minimum elements for GPU dispatch (element-wise ops)
    pub min_gpu_elements_elementwise: usize,
    /// Minimum elements for GPU dispatch (reductions)
    pub min_gpu_elements_reduction: usize,
    /// Minimum matrix dimension for GPU matmul
    pub min_gpu_matmul_dim: usize,
    /// Minimum total elements for GPU matmul
    pub min_gpu_matmul_elements: usize,
    /// Enable adaptive thresholds based on performance
    pub adaptive_thresholds: bool,
    /// Force GPU for all operations (ignore thresholds)
    pub force_gpu: bool,
    /// Force CPU for all operations (ignore thresholds)
    pub force_cpu: bool,
}

impl Default for DispatchConfig {
    fn default() -> Self {
        Self {
            // GPU overhead typically ~10-50μs, so need enough elements to amortize
            min_gpu_elements_elementwise: 10_000,      // ~40KB for f32
            min_gpu_elements_reduction: 50_000,        // Reductions need more elements
            min_gpu_matmul_dim: 64,                    // 64x64 minimum
            min_gpu_matmul_elements: 10_000,           // M*K + K*N elements
            adaptive_thresholds: true,
            force_gpu: false,
            force_cpu: false,
        }
    }
}

/// Dispatch decision result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchTarget {
    Cpu,
    Gpu,
}

/// Smart dispatcher for GPU/CPU decisions
pub struct SmartDispatcher {
    config: DispatchConfig,
    /// Adaptive threshold for element-wise (learned from performance)
    adaptive_elementwise: AtomicUsize,
    /// Adaptive threshold for reductions
    adaptive_reduction: AtomicUsize,
    /// Adaptive threshold for matmul
    adaptive_matmul: AtomicUsize,
}

impl SmartDispatcher {
    pub fn new(config: DispatchConfig) -> Self {
        Self {
            adaptive_elementwise: AtomicUsize::new(config.min_gpu_elements_elementwise),
            adaptive_reduction: AtomicUsize::new(config.min_gpu_elements_reduction),
            adaptive_matmul: AtomicUsize::new(config.min_gpu_matmul_elements),
            config,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(DispatchConfig::default())
    }

    /// Get dispatch target for an element-wise operation
    pub fn dispatch_elementwise(&self, op: OpType, elements: usize) -> DispatchTarget {
        // Check forced settings
        if self.config.force_cpu {
            GLOBAL_STATS.record_dispatch(op, elements, ExecutionLayer::Cpu, "Forced CPU");
            return DispatchTarget::Cpu;
        }
        if self.config.force_gpu {
            GLOBAL_STATS.record_dispatch(op, elements, ExecutionLayer::GpuCompute, "Forced GPU");
            return DispatchTarget::Gpu;
        }

        let threshold = if self.config.adaptive_thresholds {
            self.adaptive_elementwise.load(Ordering::Relaxed)
        } else {
            self.config.min_gpu_elements_elementwise
        };

        if elements >= threshold {
            GLOBAL_STATS.record_dispatch(
                op,
                elements,
                ExecutionLayer::GpuCompute,
                "Above threshold",
            );
            DispatchTarget::Gpu
        } else {
            GLOBAL_STATS.record_dispatch(
                op,
                elements,
                ExecutionLayer::Cpu,
                "Below threshold",
            );
            DispatchTarget::Cpu
        }
    }

    /// Get dispatch target for a reduction operation
    pub fn dispatch_reduction(&self, op: OpType, elements: usize) -> DispatchTarget {
        if self.config.force_cpu {
            GLOBAL_STATS.record_dispatch(op, elements, ExecutionLayer::Cpu, "Forced CPU");
            return DispatchTarget::Cpu;
        }
        if self.config.force_gpu {
            GLOBAL_STATS.record_dispatch(op, elements, ExecutionLayer::GpuCompute, "Forced GPU");
            return DispatchTarget::Gpu;
        }

        let threshold = if self.config.adaptive_thresholds {
            self.adaptive_reduction.load(Ordering::Relaxed)
        } else {
            self.config.min_gpu_elements_reduction
        };

        if elements >= threshold {
            GLOBAL_STATS.record_dispatch(
                op,
                elements,
                ExecutionLayer::GpuCompute,
                "Above reduction threshold",
            );
            DispatchTarget::Gpu
        } else {
            GLOBAL_STATS.record_dispatch(
                op,
                elements,
                ExecutionLayer::Cpu,
                "Below reduction threshold",
            );
            DispatchTarget::Cpu
        }
    }

    /// Get dispatch target for matrix multiplication
    pub fn dispatch_matmul(&self, m: usize, n: usize, k: usize) -> DispatchTarget {
        if self.config.force_cpu {
            GLOBAL_STATS.record_dispatch(OpType::MatMul, m * n, ExecutionLayer::Cpu, "Forced CPU");
            return DispatchTarget::Cpu;
        }
        if self.config.force_gpu {
            GLOBAL_STATS.record_dispatch(OpType::MatMul, m * n, ExecutionLayer::GpuCompute, "Forced GPU");
            return DispatchTarget::Gpu;
        }

        let min_dim = m.min(n).min(k);
        let total_elements = m * k + k * n;

        let threshold = if self.config.adaptive_thresholds {
            self.adaptive_matmul.load(Ordering::Relaxed)
        } else {
            self.config.min_gpu_matmul_elements
        };

        // Need both minimum dimension and minimum total elements
        if min_dim >= self.config.min_gpu_matmul_dim && total_elements >= threshold {
            GLOBAL_STATS.record_dispatch(
                OpType::MatMul,
                m * n,
                ExecutionLayer::GpuCompute,
                "Matrix large enough",
            );
            DispatchTarget::Gpu
        } else {
            let reason = if min_dim < self.config.min_gpu_matmul_dim {
                "Matrix dimension too small"
            } else {
                "Matrix elements below threshold"
            };
            GLOBAL_STATS.record_dispatch(OpType::MatMul, m * n, ExecutionLayer::Cpu, reason);
            DispatchTarget::Cpu
        }
    }

    /// Update adaptive thresholds based on observed performance
    pub fn update_adaptive_threshold(&self, op: OpType, gpu_faster: bool, elements: usize) {
        if !self.config.adaptive_thresholds {
            return;
        }

        let atomic = match op {
            OpType::Add | OpType::Sub | OpType::Mul | OpType::Div |
            OpType::Sin | OpType::Cos | OpType::Exp | OpType::Log |
            OpType::Sqrt | OpType::Pow => &self.adaptive_elementwise,
            OpType::Sum | OpType::Max | OpType::Min => &self.adaptive_reduction,
            OpType::MatMul => &self.adaptive_matmul,
            _ => return,
        };

        let current = atomic.load(Ordering::Relaxed);

        if gpu_faster {
            // GPU was faster, maybe we can lower the threshold
            if elements < current {
                // Found a smaller size where GPU is still faster
                let new_threshold = (current * 9 + elements) / 10; // Smooth adjustment
                atomic.store(new_threshold.max(1000), Ordering::Relaxed);
            }
        } else {
            // CPU was faster, maybe we should raise the threshold
            if elements >= current {
                // GPU wasn't faster even at this size
                let new_threshold = (current * 9 + elements * 2) / 10;
                atomic.store(new_threshold.min(1_000_000), Ordering::Relaxed);
            }
        }
    }

    /// Get current thresholds for display
    pub fn get_thresholds(&self) -> ThresholdInfo {
        ThresholdInfo {
            elementwise: self.adaptive_elementwise.load(Ordering::Relaxed),
            reduction: self.adaptive_reduction.load(Ordering::Relaxed),
            matmul: self.adaptive_matmul.load(Ordering::Relaxed),
            config: self.config.clone(),
        }
    }

    /// Set force GPU mode
    pub fn set_force_gpu(&mut self, force: bool) {
        self.config.force_gpu = force;
        if force {
            self.config.force_cpu = false;
        }
    }

    /// Set force CPU mode
    pub fn set_force_cpu(&mut self, force: bool) {
        self.config.force_cpu = force;
        if force {
            self.config.force_gpu = false;
        }
    }

    /// Get configuration
    pub fn config(&self) -> &DispatchConfig {
        &self.config
    }
}

impl Default for SmartDispatcher {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Information about current thresholds
#[derive(Debug, Clone)]
pub struct ThresholdInfo {
    pub elementwise: usize,
    pub reduction: usize,
    pub matmul: usize,
    pub config: DispatchConfig,
}

impl ThresholdInfo {
    pub fn format(&self) -> String {
        format!(
            "Dispatch Thresholds:\n\
             - Element-wise: {} elements (~{:.1} KB)\n\
             - Reduction: {} elements (~{:.1} KB)\n\
             - MatMul: {} total elements\n\
             - Force GPU: {}\n\
             - Force CPU: {}\n\
             - Adaptive: {}",
            self.elementwise,
            (self.elementwise * 4) as f64 / 1024.0,
            self.reduction,
            (self.reduction * 4) as f64 / 1024.0,
            self.matmul,
            self.config.force_gpu,
            self.config.force_cpu,
            self.config.adaptive_thresholds,
        )
    }
}

/// CPU fallback implementations using SIMD and parallel execution
///
/// Re-exports from simd_ops module which provides:
/// - SIMD-optimized operations using `wide` crate (SSE/AVX/NEON)
/// - Parallel execution via `rayon` for arrays > 100K elements
/// - Cache-efficient matmul via `matrixmultiply` crate
pub mod cpu_ops {
    pub use crate::simd_ops::{
        // Binary element-wise
        add_f32, sub_f32, mul_f32, div_f32,
        // Unary element-wise
        sin_f32, cos_f32, exp_f32, log_f32, sqrt_f32, pow_f32,
        // Reductions
        sum_f32, max_f32, min_f32,
        // Matrix operations
        matmul_f32, matmul_f32_parallel,
        // Fused operations
        fma_f32, axpy_f32, dot_f32,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatch_elementwise() {
        let dispatcher = SmartDispatcher::with_defaults();

        // Small array -> CPU
        assert_eq!(
            dispatcher.dispatch_elementwise(OpType::Add, 100),
            DispatchTarget::Cpu
        );

        // Large array -> GPU
        assert_eq!(
            dispatcher.dispatch_elementwise(OpType::Add, 100_000),
            DispatchTarget::Gpu
        );
    }

    #[test]
    fn test_dispatch_matmul() {
        let dispatcher = SmartDispatcher::with_defaults();

        // Small matrix -> CPU
        assert_eq!(dispatcher.dispatch_matmul(10, 10, 10), DispatchTarget::Cpu);

        // Large matrix -> GPU
        assert_eq!(
            dispatcher.dispatch_matmul(256, 256, 256),
            DispatchTarget::Gpu
        );
    }

    #[test]
    fn test_force_modes() {
        let mut dispatcher = SmartDispatcher::with_defaults();

        dispatcher.set_force_gpu(true);
        assert_eq!(
            dispatcher.dispatch_elementwise(OpType::Add, 10),
            DispatchTarget::Gpu
        );

        dispatcher.set_force_cpu(true);
        assert_eq!(
            dispatcher.dispatch_elementwise(OpType::Add, 1_000_000),
            DispatchTarget::Cpu
        );
    }

    #[test]
    fn test_cpu_ops() {
        let a = vec![1.0f32, 2.0, 3.0, 4.0];
        let b = vec![5.0f32, 6.0, 7.0, 8.0];
        let mut c = vec![0.0f32; 4];

        cpu_ops::add_f32(&a, &b, &mut c);
        assert_eq!(c, vec![6.0, 8.0, 10.0, 12.0]);

        cpu_ops::mul_f32(&a, &b, &mut c);
        assert_eq!(c, vec![5.0, 12.0, 21.0, 32.0]);
    }
}
