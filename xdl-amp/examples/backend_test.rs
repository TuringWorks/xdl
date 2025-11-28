// XDL AMP Backend Test Example
// Tests all available GPU backends on the current system

use ndarray::{Array1, Array2};
use xdl_amp::{ops::GpuOps, GpuBackend, GpuContext};

fn test_backend(backend: GpuBackend) {
    println!("\n--- Testing {:?} ---", backend);

    match GpuContext::with_preference(Some(backend)) {
        Ok(ctx) => {
            println!("✓ Backend initialized: {}", ctx.backend_name());

            let gpu_ops = GpuOps::new(ctx.device().clone());

            // Test element-wise operations
            let a = Array1::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
            let b = Array1::from_vec(vec![5.0, 4.0, 3.0, 2.0, 1.0]);

            // Addition
            match gpu_ops.add_1d(&a, &b) {
                Ok(result) => {
                    let expected: Vec<f32> = vec![6.0, 6.0, 6.0, 6.0, 6.0];
                    let passed = result
                        .iter()
                        .zip(expected.iter())
                        .all(|(r, e)| (r - e).abs() < 1e-5);
                    println!("  add_f32: {}", if passed { "PASS" } else { "FAIL" });
                }
                Err(e) => println!("  add_f32: ERROR - {}", e),
            }

            // Multiplication
            match gpu_ops.mul_1d(&a, &b) {
                Ok(result) => {
                    let expected: Vec<f32> = vec![5.0, 8.0, 9.0, 8.0, 5.0];
                    let passed = result
                        .iter()
                        .zip(expected.iter())
                        .all(|(r, e)| (r - e).abs() < 1e-5);
                    println!("  mul_f32: {}", if passed { "PASS" } else { "FAIL" });
                }
                Err(e) => println!("  mul_f32: ERROR - {}", e),
            }

            // Subtraction
            match gpu_ops.sub_1d(&a, &b) {
                Ok(result) => {
                    let expected: Vec<f32> = vec![-4.0, -2.0, 0.0, 2.0, 4.0];
                    let passed = result
                        .iter()
                        .zip(expected.iter())
                        .all(|(r, e)| (r - e).abs() < 1e-5);
                    println!("  sub_f32: {}", if passed { "PASS" } else { "FAIL" });
                }
                Err(e) => println!("  sub_f32: ERROR - {}", e),
            }

            // Division
            match gpu_ops.div_1d(&a, &b) {
                Ok(result) => {
                    let expected: Vec<f32> = vec![0.2, 0.5, 1.0, 2.0, 5.0];
                    let passed = result
                        .iter()
                        .zip(expected.iter())
                        .all(|(r, e)| (r - e).abs() < 1e-5);
                    println!("  div_f32: {}", if passed { "PASS" } else { "FAIL" });
                }
                Err(e) => println!("  div_f32: ERROR - {}", e),
            }

            // Sin
            let x = Array1::from_vec(vec![0.0, std::f32::consts::PI / 2.0, std::f32::consts::PI]);
            match gpu_ops.sin_1d(&x) {
                Ok(result) => {
                    let expected: Vec<f32> = vec![0.0, 1.0, 0.0];
                    let passed = result
                        .iter()
                        .zip(expected.iter())
                        .all(|(r, e)| (r - e).abs() < 1e-5);
                    println!("  sin_f32: {}", if passed { "PASS" } else { "FAIL" });
                }
                Err(e) => println!("  sin_f32: ERROR - {}", e),
            }

            // Cos
            match gpu_ops.cos_1d(&x) {
                Ok(result) => {
                    let expected: Vec<f32> = vec![1.0, 0.0, -1.0];
                    let passed = result
                        .iter()
                        .zip(expected.iter())
                        .all(|(r, e)| (r - e).abs() < 1e-5);
                    println!("  cos_f32: {}", if passed { "PASS" } else { "FAIL" });
                }
                Err(e) => println!("  cos_f32: ERROR - {}", e),
            }

            // Exp
            let y = Array1::from_vec(vec![0.0, 1.0, 2.0]);
            match gpu_ops.exp_1d(&y) {
                Ok(result) => {
                    let expected: Vec<f32> =
                        vec![1.0, std::f32::consts::E, std::f32::consts::E.powi(2)];
                    let passed = result
                        .iter()
                        .zip(expected.iter())
                        .all(|(r, e)| (r - e).abs() < 1e-4);
                    println!("  exp_f32: {}", if passed { "PASS" } else { "FAIL" });
                }
                Err(e) => println!("  exp_f32: ERROR - {}", e),
            }

            // Matrix multiplication
            let mat_a = Array2::from_shape_vec((2, 3), vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap();
            let mat_b =
                Array2::from_shape_vec((3, 2), vec![7.0, 8.0, 9.0, 10.0, 11.0, 12.0]).unwrap();
            match gpu_ops.matmul(&mat_a, &mat_b) {
                Ok(result) => {
                    // Expected: [[58, 64], [139, 154]]
                    let expected =
                        Array2::from_shape_vec((2, 2), vec![58.0, 64.0, 139.0, 154.0]).unwrap();
                    let max_diff = result
                        .iter()
                        .zip(expected.iter())
                        .map(|(r, e)| (r - e).abs())
                        .fold(0.0f32, f32::max);
                    let passed = max_diff < 1e-3;
                    println!(
                        "  matmul:  {} (max_diff: {:.2e})",
                        if passed { "PASS" } else { "FAIL" },
                        max_diff
                    );
                }
                Err(e) => println!("  matmul:  ERROR - {}", e),
            }

            // Sum reduction
            let z = Array1::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
            match gpu_ops.sum_1d(&z) {
                Ok(result) => {
                    let expected = 15.0;
                    let passed = (result - expected).abs() < 1e-5;
                    println!("  sum_f32: {}", if passed { "PASS" } else { "FAIL" });
                }
                Err(e) => println!("  sum_f32: ERROR - {}", e),
            }

            println!("✓ Backend tests completed");
        }
        Err(e) => {
            println!("✗ Backend not available: {}", e);
        }
    }
}

fn main() {
    println!("========================================");
    println!("XDL AMP Backend Test Suite");
    println!("========================================");
    println!();
    println!("Platform: {}", std::env::consts::OS);
    println!("Architecture: {}", std::env::consts::ARCH);

    // Test default backend
    println!("\n=== Default Backend ===");
    match GpuContext::new() {
        Ok(ctx) => {
            println!("Default backend: {}", ctx.backend_name());
        }
        Err(e) => {
            println!("Failed to create default context: {}", e);
        }
    }

    // Test platform-specific backends
    #[cfg(target_os = "macos")]
    {
        println!("\n=== macOS Backends ===");
        test_backend(GpuBackend::Metal);

        #[cfg(feature = "mps")]
        test_backend(GpuBackend::MetalPerformanceShaders);
    }

    #[cfg(target_os = "windows")]
    {
        println!("\n=== Windows Backends ===");
        test_backend(GpuBackend::DirectX12);

        #[cfg(feature = "directml")]
        test_backend(GpuBackend::DirectML);
    }

    // Test cross-platform backends
    println!("\n=== Cross-Platform Backends ===");

    #[cfg(feature = "cuda")]
    test_backend(GpuBackend::Cuda);

    #[cfg(feature = "vulkan")]
    test_backend(GpuBackend::Vulkan);

    #[cfg(feature = "opencl")]
    test_backend(GpuBackend::OpenCL);

    println!("\n========================================");
    println!("Backend Test Complete!");
    println!("========================================");
}
