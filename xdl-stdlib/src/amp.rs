//! XDL AMP (Accelerated Math Processing) Interface
//!
//! Provides XDL script access to GPU and SIMD-optimized operations.

use std::time::Instant;
use xdl_amp::simd_ops;
use xdl_amp::stats::GLOBAL_STATS;
use xdl_core::{XdlError, XdlResult, XdlValue};

/// AMP_INFO() - Get information about available acceleration backends
///
/// Returns a string with acceleration capabilities:
/// - SIMD: Always available (SSE/AVX on x86, NEON on ARM)
/// - GPU: If CUDA, DirectX, Metal, etc. is available
pub fn amp_info(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    let mut info = String::new();

    info.push_str("XDL-AMP Acceleration Information\n");
    info.push_str("================================\n\n");

    // SIMD info
    info.push_str("SIMD Acceleration:\n");
    #[cfg(target_arch = "x86_64")]
    {
        info.push_str("  Architecture: x86_64\n");
        if is_x86_feature_detected!("avx2") {
            info.push_str("  Features: AVX2 (256-bit vectors)\n");
        } else if is_x86_feature_detected!("avx") {
            info.push_str("  Features: AVX (256-bit vectors)\n");
        } else if is_x86_feature_detected!("sse4.1") {
            info.push_str("  Features: SSE4.1 (128-bit vectors)\n");
        } else {
            info.push_str("  Features: SSE2 (128-bit vectors)\n");
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        info.push_str("  Architecture: ARM64\n");
        info.push_str("  Features: NEON (128-bit vectors)\n");
    }
    info.push_str("  Status: ENABLED\n\n");

    // GPU info
    info.push_str("GPU Acceleration:\n");
    #[cfg(target_os = "windows")]
    {
        info.push_str("  Platform: Windows\n");
        info.push_str("  Available backends: DirectX 12");
        #[cfg(feature = "cuda")]
        info.push_str(", CUDA");
        #[cfg(feature = "vulkan")]
        info.push_str(", Vulkan");
        info.push_str("\n");
    }
    #[cfg(target_os = "macos")]
    {
        info.push_str("  Platform: macOS\n");
        info.push_str("  Available backends: Metal, MPS\n");
    }
    #[cfg(target_os = "linux")]
    {
        info.push_str("  Platform: Linux\n");
        info.push_str("  Available backends:");
        #[cfg(feature = "cuda")]
        info.push_str(" CUDA");
        #[cfg(feature = "vulkan")]
        info.push_str(" Vulkan");
        #[cfg(feature = "opencl")]
        info.push_str(" OpenCL");
        info.push_str("\n");
    }

    // Check GPU availability
    match xdl_amp::GpuContext::new() {
        Ok(ctx) => {
            info.push_str(&format!("  Active backend: {}\n", ctx.backend_name()));
            info.push_str("  Status: ENABLED\n");
        }
        Err(e) => {
            info.push_str(&format!("  Status: UNAVAILABLE ({})\n", e));
        }
    }

    Ok(XdlValue::String(info))
}

/// AMP_BACKEND() - Get the name of the active GPU backend
pub fn amp_backend(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    match xdl_amp::GpuContext::new() {
        Ok(ctx) => Ok(XdlValue::String(ctx.backend_name().to_string())),
        Err(_) => Ok(XdlValue::String("CPU (SIMD)".to_string())),
    }
}

/// AMP_STATS() - Get execution statistics
pub fn amp_stats(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    Ok(XdlValue::String(GLOBAL_STATS.format_report()))
}

/// AMP_RESET_STATS() - Reset execution statistics
pub fn amp_reset_stats(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    GLOBAL_STATS.reset();
    Ok(XdlValue::Long(0))
}

// ============================================================================
// SIMD Operations
// ============================================================================

/// AMP_SIMD_ADD(a, b) - SIMD-accelerated element-wise addition
pub fn amp_simd_add(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 2 {
        return Err(XdlError::RuntimeError(
            "AMP_SIMD_ADD requires 2 arguments: a, b".to_string(),
        ));
    }

    let a = extract_f32_array(&args[0])?;
    let b = extract_f32_array(&args[1])?;

    if a.len() != b.len() {
        return Err(XdlError::RuntimeError(
            "Arrays must have the same length".to_string(),
        ));
    }

    let mut c = vec![0.0f32; a.len()];
    simd_ops::add_f32(&a, &b, &mut c);

    Ok(f32_array_to_xdl(&c))
}

/// AMP_SIMD_MUL(a, b) - SIMD-accelerated element-wise multiplication
pub fn amp_simd_mul(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 2 {
        return Err(XdlError::RuntimeError(
            "AMP_SIMD_MUL requires 2 arguments: a, b".to_string(),
        ));
    }

    let a = extract_f32_array(&args[0])?;
    let b = extract_f32_array(&args[1])?;

    if a.len() != b.len() {
        return Err(XdlError::RuntimeError(
            "Arrays must have the same length".to_string(),
        ));
    }

    let mut c = vec![0.0f32; a.len()];
    simd_ops::mul_f32(&a, &b, &mut c);

    Ok(f32_array_to_xdl(&c))
}

/// AMP_SIMD_SUB(a, b) - SIMD-accelerated element-wise subtraction
pub fn amp_simd_sub(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 2 {
        return Err(XdlError::RuntimeError(
            "AMP_SIMD_SUB requires 2 arguments: a, b".to_string(),
        ));
    }

    let a = extract_f32_array(&args[0])?;
    let b = extract_f32_array(&args[1])?;

    if a.len() != b.len() {
        return Err(XdlError::RuntimeError(
            "Arrays must have the same length".to_string(),
        ));
    }

    let mut c = vec![0.0f32; a.len()];
    simd_ops::sub_f32(&a, &b, &mut c);

    Ok(f32_array_to_xdl(&c))
}

/// AMP_SIMD_DIV(a, b) - SIMD-accelerated element-wise division
pub fn amp_simd_div(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 2 {
        return Err(XdlError::RuntimeError(
            "AMP_SIMD_DIV requires 2 arguments: a, b".to_string(),
        ));
    }

    let a = extract_f32_array(&args[0])?;
    let b = extract_f32_array(&args[1])?;

    if a.len() != b.len() {
        return Err(XdlError::RuntimeError(
            "Arrays must have the same length".to_string(),
        ));
    }

    let mut c = vec![0.0f32; a.len()];
    simd_ops::div_f32(&a, &b, &mut c);

    Ok(f32_array_to_xdl(&c))
}

/// AMP_SIMD_SQRT(x) - SIMD-accelerated square root
pub fn amp_simd_sqrt(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::RuntimeError(
            "AMP_SIMD_SQRT requires 1 argument".to_string(),
        ));
    }

    let x = extract_f32_array(&args[0])?;
    let mut y = vec![0.0f32; x.len()];
    simd_ops::sqrt_f32(&x, &mut y);

    Ok(f32_array_to_xdl(&y))
}

/// AMP_SIMD_SUM(x) - SIMD-accelerated sum reduction
pub fn amp_simd_sum(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::RuntimeError(
            "AMP_SIMD_SUM requires 1 argument".to_string(),
        ));
    }

    let x = extract_f32_array(&args[0])?;
    let sum = simd_ops::sum_f32(&x);

    Ok(XdlValue::Double(sum as f64))
}

/// AMP_SIMD_MAX(x) - SIMD-accelerated max reduction
pub fn amp_simd_max(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::RuntimeError(
            "AMP_SIMD_MAX requires 1 argument".to_string(),
        ));
    }

    let x = extract_f32_array(&args[0])?;
    let max_val = simd_ops::max_f32(&x);

    Ok(XdlValue::Double(max_val as f64))
}

/// AMP_SIMD_MIN(x) - SIMD-accelerated min reduction
pub fn amp_simd_min(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::RuntimeError(
            "AMP_SIMD_MIN requires 1 argument".to_string(),
        ));
    }

    let x = extract_f32_array(&args[0])?;
    let min_val = simd_ops::min_f32(&x);

    Ok(XdlValue::Double(min_val as f64))
}

/// AMP_SIMD_DOT(a, b) - SIMD-accelerated dot product
pub fn amp_simd_dot(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 2 {
        return Err(XdlError::RuntimeError(
            "AMP_SIMD_DOT requires 2 arguments: a, b".to_string(),
        ));
    }

    let a = extract_f32_array(&args[0])?;
    let b = extract_f32_array(&args[1])?;

    if a.len() != b.len() {
        return Err(XdlError::RuntimeError(
            "Arrays must have the same length".to_string(),
        ));
    }

    let dot = simd_ops::dot_f32(&a, &b);

    Ok(XdlValue::Double(dot as f64))
}

/// AMP_SIMD_MATMUL(a, b, m, n, k) - SIMD-accelerated matrix multiplication
/// C[m,n] = A[m,k] * B[k,n]
pub fn amp_simd_matmul(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 5 {
        return Err(XdlError::RuntimeError(
            "AMP_SIMD_MATMUL requires 5 arguments: a, b, m, n, k".to_string(),
        ));
    }

    let a = extract_f32_array(&args[0])?;
    let b = extract_f32_array(&args[1])?;
    let m = extract_int(&args[2])? as usize;
    let n = extract_int(&args[3])? as usize;
    let k = extract_int(&args[4])? as usize;

    if a.len() != m * k {
        return Err(XdlError::RuntimeError(format!(
            "Matrix A should have {} elements (m*k), got {}",
            m * k,
            a.len()
        )));
    }
    if b.len() != k * n {
        return Err(XdlError::RuntimeError(format!(
            "Matrix B should have {} elements (k*n), got {}",
            k * n,
            b.len()
        )));
    }

    let mut c = vec![0.0f32; m * n];
    simd_ops::matmul_f32(&a, &b, &mut c, m, n, k);

    Ok(f32_array_to_xdl(&c))
}

// ============================================================================
// GPU Operations (CUDA/DirectX/Metal)
// ============================================================================

/// AMP_GPU_AVAILABLE() - Check if GPU acceleration is available
pub fn amp_gpu_available(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    let available = xdl_amp::GpuContext::new().is_ok();
    Ok(XdlValue::Long(if available { 1 } else { 0 }))
}

/// AMP_GPU_ADD(a, b) - GPU-accelerated element-wise addition
#[cfg(feature = "cuda")]
pub fn amp_gpu_add(args: &[XdlValue]) -> XdlResult<XdlValue> {
    use xdl_amp::backend::GpuDevice;
    use xdl_amp::cuda::CudaDevice;

    if args.len() != 2 {
        return Err(XdlError::RuntimeError(
            "AMP_GPU_ADD requires 2 arguments: a, b".to_string(),
        ));
    }

    let a = extract_f32_array(&args[0])?;
    let b = extract_f32_array(&args[1])?;

    if a.len() != b.len() {
        return Err(XdlError::RuntimeError(
            "Arrays must have the same length".to_string(),
        ));
    }

    let device = CudaDevice::new()
        .map_err(|e| XdlError::RuntimeError(format!("CUDA init failed: {}", e)))?;

    let mut c = vec![0.0f32; a.len()];
    device
        .add_f32(&a, &b, &mut c)
        .map_err(|e| XdlError::RuntimeError(format!("CUDA add failed: {}", e)))?;

    Ok(f32_array_to_xdl(&c))
}

#[cfg(not(feature = "cuda"))]
pub fn amp_gpu_add(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    Err(XdlError::RuntimeError(
        "CUDA feature not enabled. Rebuild with --features cuda".to_string(),
    ))
}

/// AMP_GPU_MATMUL(a, b, m, n, k) - GPU-accelerated matrix multiplication
#[cfg(feature = "cuda")]
pub fn amp_gpu_matmul(args: &[XdlValue]) -> XdlResult<XdlValue> {
    use xdl_amp::backend::GpuDevice;
    use xdl_amp::cuda::CudaDevice;

    if args.len() != 5 {
        return Err(XdlError::RuntimeError(
            "AMP_GPU_MATMUL requires 5 arguments: a, b, m, n, k".to_string(),
        ));
    }

    let a = extract_f32_array(&args[0])?;
    let b = extract_f32_array(&args[1])?;
    let m = extract_int(&args[2])? as usize;
    let n = extract_int(&args[3])? as usize;
    let k = extract_int(&args[4])? as usize;

    let device = CudaDevice::new()
        .map_err(|e| XdlError::RuntimeError(format!("CUDA init failed: {}", e)))?;

    let mut c = vec![0.0f32; m * n];
    device
        .matmul_f32(&a, &b, &mut c, m, n, k)
        .map_err(|e| XdlError::RuntimeError(format!("CUDA matmul failed: {}", e)))?;

    Ok(f32_array_to_xdl(&c))
}

#[cfg(not(feature = "cuda"))]
pub fn amp_gpu_matmul(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    Err(XdlError::RuntimeError(
        "CUDA feature not enabled. Rebuild with --features cuda".to_string(),
    ))
}

// ============================================================================
// Benchmarking
// ============================================================================

/// AMP_BENCHMARK(operation, size) - Benchmark an operation
/// Operations: "add", "mul", "sum", "matmul"
pub fn amp_benchmark(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::RuntimeError(
            "AMP_BENCHMARK requires at least 2 arguments: operation, size".to_string(),
        ));
    }

    let op = match &args[0] {
        XdlValue::String(s) => s.to_uppercase(),
        _ => {
            return Err(XdlError::RuntimeError(
                "Operation must be a string".to_string(),
            ))
        }
    };

    let size = extract_int(&args[1])? as usize;
    let iterations = if args.len() > 2 {
        extract_int(&args[2])? as usize
    } else {
        100
    };

    let mut result = String::new();
    result.push_str(&format!(
        "Benchmarking {} with {} elements, {} iterations\n",
        op, size, iterations
    ));

    match op.as_str() {
        "ADD" => {
            let a: Vec<f32> = (0..size).map(|i| i as f32).collect();
            let b: Vec<f32> = (0..size).map(|i| (size - i) as f32).collect();
            let mut c = vec![0.0f32; size];

            // Warmup
            simd_ops::add_f32(&a, &b, &mut c);

            let start = Instant::now();
            for _ in 0..iterations {
                simd_ops::add_f32(&a, &b, &mut c);
            }
            let elapsed = start.elapsed();
            let avg_us = elapsed.as_secs_f64() * 1_000_000.0 / iterations as f64;
            let throughput = (size as f64 * 3.0 * 4.0) / (avg_us / 1_000_000.0) / 1e9;

            result.push_str(&format!("  Average time: {:.2} µs\n", avg_us));
            result.push_str(&format!("  Throughput: {:.2} GB/s\n", throughput));
        }
        "MUL" => {
            let a: Vec<f32> = (0..size).map(|i| i as f32 / 1000.0).collect();
            let b: Vec<f32> = (0..size).map(|i| (size - i) as f32 / 1000.0).collect();
            let mut c = vec![0.0f32; size];

            simd_ops::mul_f32(&a, &b, &mut c);

            let start = Instant::now();
            for _ in 0..iterations {
                simd_ops::mul_f32(&a, &b, &mut c);
            }
            let elapsed = start.elapsed();
            let avg_us = elapsed.as_secs_f64() * 1_000_000.0 / iterations as f64;
            let throughput = (size as f64 * 3.0 * 4.0) / (avg_us / 1_000_000.0) / 1e9;

            result.push_str(&format!("  Average time: {:.2} µs\n", avg_us));
            result.push_str(&format!("  Throughput: {:.2} GB/s\n", throughput));
        }
        "SUM" => {
            let x: Vec<f32> = (0..size).map(|i| (i % 1000) as f32 / 1000.0).collect();

            let _ = simd_ops::sum_f32(&x);

            let start = Instant::now();
            for _ in 0..iterations {
                std::hint::black_box(simd_ops::sum_f32(&x));
            }
            let elapsed = start.elapsed();
            let avg_us = elapsed.as_secs_f64() * 1_000_000.0 / iterations as f64;
            let throughput = (size as f64 * 4.0) / (avg_us / 1_000_000.0) / 1e9;

            result.push_str(&format!("  Average time: {:.2} µs\n", avg_us));
            result.push_str(&format!("  Throughput: {:.2} GB/s\n", throughput));
        }
        "MATMUL" => {
            let n = (size as f64).sqrt() as usize;
            let a: Vec<f32> = (0..n * n).map(|i| (i % 100) as f32 / 100.0).collect();
            let b: Vec<f32> = (0..n * n).map(|i| (i % 100) as f32 / 100.0).collect();
            let mut c = vec![0.0f32; n * n];

            simd_ops::matmul_f32(&a, &b, &mut c, n, n, n);

            let mat_iterations = iterations.min(10);
            let start = Instant::now();
            for _ in 0..mat_iterations {
                simd_ops::matmul_f32(&a, &b, &mut c, n, n, n);
            }
            let elapsed = start.elapsed();
            let avg_ms = elapsed.as_secs_f64() * 1000.0 / mat_iterations as f64;
            let gflops = (2.0 * (n as f64).powi(3)) / (avg_ms / 1000.0) / 1e9;

            result.push_str(&format!("  Matrix size: {}x{}\n", n, n));
            result.push_str(&format!("  Average time: {:.2} ms\n", avg_ms));
            result.push_str(&format!("  Performance: {:.2} GFLOPS\n", gflops));
        }
        _ => {
            return Err(XdlError::RuntimeError(format!(
                "Unknown operation: {}. Use ADD, MUL, SUM, or MATMUL",
                op
            )));
        }
    }

    Ok(XdlValue::String(result))
}

// ============================================================================
// Helper Functions
// ============================================================================

fn extract_f32_array(value: &XdlValue) -> XdlResult<Vec<f32>> {
    match value {
        XdlValue::Array(arr) => Ok(arr.iter().map(|f| *f as f32).collect()),
        XdlValue::MultiDimArray { data, .. } => Ok(data.iter().map(|f| *f as f32).collect()),
        XdlValue::NestedArray(arr) => {
            // Flatten nested array
            arr.iter()
                .map(|v| match v {
                    XdlValue::Double(f) => Ok(*f as f32),
                    XdlValue::Float(f) => Ok(*f),
                    XdlValue::Long(i) => Ok(*i as f32),
                    XdlValue::Long64(i) => Ok(*i as f32),
                    _ => Err(XdlError::RuntimeError(
                        "Array must contain numbers".to_string(),
                    )),
                })
                .collect()
        }
        _ => Err(XdlError::RuntimeError("Expected an array".to_string())),
    }
}

fn extract_int(value: &XdlValue) -> XdlResult<i64> {
    match value {
        XdlValue::Long64(i) => Ok(*i),
        XdlValue::Long(i) => Ok(*i as i64),
        XdlValue::Int(i) => Ok(*i as i64),
        XdlValue::Byte(i) => Ok(*i as i64),
        XdlValue::Float(f) => Ok(*f as i64),
        XdlValue::Double(f) => Ok(*f as i64),
        _ => Err(XdlError::RuntimeError("Expected an integer".to_string())),
    }
}

fn f32_array_to_xdl(arr: &[f32]) -> XdlValue {
    XdlValue::Array(arr.iter().map(|f| *f as f64).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amp_info() {
        let result = amp_info(&[]).unwrap();
        if let XdlValue::String(s) = result {
            assert!(s.contains("SIMD Acceleration"));
        } else {
            panic!("Expected string result");
        }
    }

    #[test]
    fn test_amp_simd_add() {
        let a = XdlValue::Array(vec![1.0, 2.0, 3.0, 4.0]);
        let b = XdlValue::Array(vec![5.0, 6.0, 7.0, 8.0]);
        let result = amp_simd_add(&[a, b]).unwrap();

        if let XdlValue::Array(arr) = result {
            assert!((arr[0] - 6.0).abs() < 1e-6);
            assert!((arr[1] - 8.0).abs() < 1e-6);
            assert!((arr[2] - 10.0).abs() < 1e-6);
            assert!((arr[3] - 12.0).abs() < 1e-6);
        } else {
            panic!("Expected Array result");
        }
    }

    #[test]
    fn test_amp_simd_sum() {
        let x = XdlValue::Array((1..=100).map(|i| i as f64).collect());
        let result = amp_simd_sum(&[x]).unwrap();

        if let XdlValue::Double(sum) = result {
            assert!((sum - 5050.0).abs() < 1.0);
        } else {
            panic!("Expected Double result");
        }
    }
}
