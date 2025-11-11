//! GPU-accelerated array operations
//!
//! Automatically uses GPU for large arrays (>= 100K elements)
//! Falls back to CPU for smaller arrays to avoid overhead

use lazy_static::lazy_static;
use ndarray::Array1;
use std::sync::Mutex;
use xdl_amp::{ops::GpuOps, GpuContext};

/// Threshold for GPU acceleration (elements)
/// Arrays with fewer elements use CPU to avoid GPU overhead
const GPU_THRESHOLD: usize = 100_000;

lazy_static! {
    /// Global GPU context - initialized once on first use
    static ref GPU_CONTEXT: Mutex<Option<(GpuContext, GpuOps)>> = Mutex::new(None);
}

/// Initialize GPU context if not already initialized
fn ensure_gpu_initialized() -> bool {
    let mut ctx_guard = match GPU_CONTEXT.lock() {
        Ok(guard) => guard,
        Err(_) => return false,
    };

    if ctx_guard.is_none() {
        // Try to initialize GPU
        match GpuContext::new() {
            Ok(ctx) => {
                let backend_name = ctx.backend_name();
                let ops = GpuOps::new(ctx.device().clone());
                tracing::info!("🚀 GPU acceleration initialized: {}", backend_name);
                *ctx_guard = Some((ctx, ops));
                true
            }
            Err(e) => {
                tracing::warn!("GPU initialization failed: {}. Using CPU fallback.", e);
                false
            }
        }
    } else {
        true
    }
}

/// Check if GPU should be used for given array size
#[inline]
pub fn should_use_gpu(size: usize) -> bool {
    size >= GPU_THRESHOLD
}

/// GPU-accelerated MIN operation
pub fn gpu_min(data: &[f64]) -> Option<f64> {
    if !should_use_gpu(data.len()) {
        return None; // Use CPU for small arrays
    }

    if !ensure_gpu_initialized() {
        return None;
    }

    let ctx_guard = GPU_CONTEXT.lock().ok()?;
    let (_, gpu_ops) = ctx_guard.as_ref()?;

    // Convert f64 to f32 for GPU
    let data_f32: Vec<f32> = data.iter().map(|&x| x as f32).collect();
    let array = Array1::from_vec(data_f32);

    // Compute on GPU
    match gpu_ops.min_1d(&array) {
        Ok(result) => {
            tracing::debug!("✓ GPU MIN: {} elements", data.len());
            Some(result as f64)
        }
        Err(e) => {
            tracing::warn!("GPU MIN failed: {}. Falling back to CPU.", e);
            None
        }
    }
}

/// GPU-accelerated MAX operation
pub fn gpu_max(data: &[f64]) -> Option<f64> {
    if !should_use_gpu(data.len()) {
        return None; // Use CPU for small arrays
    }

    if !ensure_gpu_initialized() {
        return None;
    }

    let ctx_guard = GPU_CONTEXT.lock().ok()?;
    let (_, gpu_ops) = ctx_guard.as_ref()?;

    // Convert f64 to f32 for GPU
    let data_f32: Vec<f32> = data.iter().map(|&x| x as f32).collect();
    let array = Array1::from_vec(data_f32);

    // Compute on GPU
    match gpu_ops.max_1d(&array) {
        Ok(result) => {
            tracing::debug!("✓ GPU MAX: {} elements", data.len());
            Some(result as f64)
        }
        Err(e) => {
            tracing::warn!("GPU MAX failed: {}. Falling back to CPU.", e);
            None
        }
    }
}

/// GPU-accelerated SUM operation (for TOTAL and MEAN)
pub fn gpu_sum(data: &[f64]) -> Option<f64> {
    if !should_use_gpu(data.len()) {
        return None; // Use CPU for small arrays
    }

    if !ensure_gpu_initialized() {
        return None;
    }

    let ctx_guard = GPU_CONTEXT.lock().ok()?;
    let (_, gpu_ops) = ctx_guard.as_ref()?;

    // Convert f64 to f32 for GPU
    let data_f32: Vec<f32> = data.iter().map(|&x| x as f32).collect();
    let array = Array1::from_vec(data_f32);

    // Compute on GPU
    match gpu_ops.sum_1d(&array) {
        Ok(result) => {
            tracing::debug!("✓ GPU SUM: {} elements", data.len());
            Some(result as f64)
        }
        Err(e) => {
            tracing::warn!("GPU SUM failed: {}. Falling back to CPU.", e);
            None
        }
    }
}

/// Check if GPU is available and initialized
pub fn is_gpu_available() -> bool {
    if let Ok(ctx_guard) = GPU_CONTEXT.lock() {
        ctx_guard.is_some()
    } else {
        false
    }
}

/// Get GPU backend name if available
pub fn gpu_backend_name() -> Option<String> {
    let ctx_guard = GPU_CONTEXT.lock().ok()?;
    ctx_guard
        .as_ref()
        .map(|(ctx, _)| ctx.backend_name().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_threshold() {
        assert!(!should_use_gpu(1000));
        assert!(!should_use_gpu(99_999));
        assert!(should_use_gpu(100_000));
        assert!(should_use_gpu(1_000_000));
    }

    #[test]
    fn test_gpu_min() {
        let small_data = vec![1.0, 2.0, 3.0];
        assert!(gpu_min(&small_data).is_none()); // Too small for GPU

        let large_data: Vec<f64> = (0..200_000).map(|i| i as f64).collect();
        // GPU may or may not be available in test environment
        let result = gpu_min(&large_data);
        if result.is_some() {
            assert_eq!(result.unwrap(), 0.0);
        }
    }
}
