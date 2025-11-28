//! CUDA Backend Verification Example
//!
//! Verifies that CUDA GPU operations produce correct results.
//! Requires NVIDIA GPU with CUDA support.
//!
//! Run with: cargo run --example verify_cuda --release -p xdl-amp --features cuda

use std::time::Instant;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║           XDL-AMP CUDA VERIFICATION                              ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    #[cfg(feature = "cuda")]
    {
        use xdl_amp::cuda::CudaDevice;
        use xdl_amp::backend::GpuDevice;
        use xdl_amp::stats::GLOBAL_STATS;

        // Check CUDA availability
        println!("Checking CUDA availability...");
        if !CudaDevice::is_available() {
            println!("\n  CUDA is NOT available on this system.");
            println!("  Possible reasons:");
            println!("  - No NVIDIA GPU installed");
            println!("  - NVIDIA drivers not installed");
            println!("  - CUDA toolkit not installed");
            return;
        }

        println!("  CUDA is available!\n");

        // Initialize CUDA device
        println!("Initializing CUDA device...");
        let device = match CudaDevice::new() {
            Ok(d) => {
                println!("  Device initialized successfully!\n");
                d
            }
            Err(e) => {
                println!("  Failed to initialize CUDA device: {}", e);
                return;
            }
        };

        // Reset stats
        GLOBAL_STATS.reset();
        GLOBAL_STATS.set_backend("NVIDIA CUDA");

        let mut all_passed = true;

        // Test 1: Element-wise Addition
        print!("Testing CUDA add_f32... ");
        let a = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let b = vec![10.0f32, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0];
        let mut c = vec![0.0f32; 8];
        match device.add_f32(&a, &b, &mut c) {
            Ok(_) => {
                let expected = vec![11.0, 22.0, 33.0, 44.0, 55.0, 66.0, 77.0, 88.0];
                if verify_vec(&c, &expected, 1e-5) {
                    println!("PASSED");
                } else {
                    println!("FAILED (got {:?})", c);
                    all_passed = false;
                }
            }
            Err(e) => {
                println!("ERROR: {}", e);
                all_passed = false;
            }
        }

        // Test 2: Element-wise Subtraction
        print!("Testing CUDA sub_f32... ");
        let mut c = vec![0.0f32; 8];
        match device.sub_f32(&b, &a, &mut c) {
            Ok(_) => {
                let expected = vec![9.0, 18.0, 27.0, 36.0, 45.0, 54.0, 63.0, 72.0];
                if verify_vec(&c, &expected, 1e-5) {
                    println!("PASSED");
                } else {
                    println!("FAILED");
                    all_passed = false;
                }
            }
            Err(e) => {
                println!("ERROR: {}", e);
                all_passed = false;
            }
        }

        // Test 3: Element-wise Multiplication
        print!("Testing CUDA mul_f32... ");
        let a = vec![2.0f32; 100];
        let b = vec![3.0f32; 100];
        let mut c = vec![0.0f32; 100];
        match device.mul_f32(&a, &b, &mut c) {
            Ok(_) => {
                if c.iter().all(|&x| (x - 6.0).abs() < 1e-5) {
                    println!("PASSED");
                } else {
                    println!("FAILED");
                    all_passed = false;
                }
            }
            Err(e) => {
                println!("ERROR: {}", e);
                all_passed = false;
            }
        }

        // Test 4: Element-wise Division
        print!("Testing CUDA div_f32... ");
        let a = vec![10.0f32; 100];
        let b = vec![2.0f32; 100];
        let mut c = vec![0.0f32; 100];
        match device.div_f32(&a, &b, &mut c) {
            Ok(_) => {
                if c.iter().all(|&x| (x - 5.0).abs() < 1e-5) {
                    println!("PASSED");
                } else {
                    println!("FAILED");
                    all_passed = false;
                }
            }
            Err(e) => {
                println!("ERROR: {}", e);
                all_passed = false;
            }
        }

        // Test 5: Square Root
        print!("Testing CUDA sqrt_f32... ");
        let x = vec![4.0f32, 9.0, 16.0, 25.0, 36.0, 49.0, 64.0, 81.0];
        let mut y = vec![0.0f32; 8];
        match device.sqrt_f32(&x, &mut y) {
            Ok(_) => {
                let expected = vec![2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
                if verify_vec(&y, &expected, 1e-5) {
                    println!("PASSED");
                } else {
                    println!("FAILED");
                    all_passed = false;
                }
            }
            Err(e) => {
                println!("ERROR: {}", e);
                all_passed = false;
            }
        }

        // Test 6: Trigonometric Functions
        print!("Testing CUDA sin_f32... ");
        let x = vec![0.0f32, std::f32::consts::PI / 2.0, std::f32::consts::PI];
        let mut y = vec![0.0f32; 3];
        match device.sin_f32(&x, &mut y) {
            Ok(_) => {
                if (y[0] - 0.0).abs() < 1e-5 && (y[1] - 1.0).abs() < 1e-5 && y[2].abs() < 1e-4 {
                    println!("PASSED");
                } else {
                    println!("FAILED (got {:?})", y);
                    all_passed = false;
                }
            }
            Err(e) => {
                println!("ERROR: {}", e);
                all_passed = false;
            }
        }

        print!("Testing CUDA cos_f32... ");
        let mut y = vec![0.0f32; 3];
        match device.cos_f32(&x, &mut y) {
            Ok(_) => {
                if (y[0] - 1.0).abs() < 1e-5 && y[1].abs() < 1e-5 && (y[2] - (-1.0)).abs() < 1e-4 {
                    println!("PASSED");
                } else {
                    println!("FAILED (got {:?})", y);
                    all_passed = false;
                }
            }
            Err(e) => {
                println!("ERROR: {}", e);
                all_passed = false;
            }
        }

        // Test 7: Exponential and Logarithm
        print!("Testing CUDA exp_f32... ");
        let x = vec![0.0f32, 1.0, 2.0];
        let mut y = vec![0.0f32; 3];
        match device.exp_f32(&x, &mut y) {
            Ok(_) => {
                if (y[0] - 1.0).abs() < 1e-5 && (y[1] - std::f32::consts::E).abs() < 1e-4 {
                    println!("PASSED");
                } else {
                    println!("FAILED");
                    all_passed = false;
                }
            }
            Err(e) => {
                println!("ERROR: {}", e);
                all_passed = false;
            }
        }

        print!("Testing CUDA log_f32... ");
        let x = vec![1.0f32, std::f32::consts::E, std::f32::consts::E * std::f32::consts::E];
        let mut y = vec![0.0f32; 3];
        match device.log_f32(&x, &mut y) {
            Ok(_) => {
                if (y[0] - 0.0).abs() < 1e-5 && (y[1] - 1.0).abs() < 1e-4 && (y[2] - 2.0).abs() < 1e-3 {
                    println!("PASSED");
                } else {
                    println!("FAILED");
                    all_passed = false;
                }
            }
            Err(e) => {
                println!("ERROR: {}", e);
                all_passed = false;
            }
        }

        // Test 8: Power Function
        print!("Testing CUDA pow_f32... ");
        let x = vec![2.0f32, 3.0, 4.0, 5.0];
        let mut y = vec![0.0f32; 4];
        match device.pow_f32(&x, 2.0, &mut y) {
            Ok(_) => {
                let expected = vec![4.0, 9.0, 16.0, 25.0];
                if verify_vec(&y, &expected, 1e-4) {
                    println!("PASSED");
                } else {
                    println!("FAILED");
                    all_passed = false;
                }
            }
            Err(e) => {
                println!("ERROR: {}", e);
                all_passed = false;
            }
        }

        // Test 9: Sum Reduction
        print!("Testing CUDA sum_f32... ");
        let x: Vec<f32> = (1..=1000).map(|i| i as f32).collect();
        match device.sum_f32(&x) {
            Ok(sum) => {
                let expected_sum = 500500.0; // n*(n+1)/2 for n=1000
                if (sum - expected_sum).abs() < 1.0 {
                    println!("PASSED (sum = {})", sum);
                } else {
                    println!("FAILED (expected {}, got {})", expected_sum, sum);
                    all_passed = false;
                }
            }
            Err(e) => {
                println!("ERROR: {}", e);
                all_passed = false;
            }
        }

        // Test 10: Max Reduction
        print!("Testing CUDA max_f32... ");
        let mut x = vec![3.0f32, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0, 6.0];
        x.extend(vec![1.0f32; 1000]); // Add more elements
        x[500] = 999.0;
        match device.max_f32(&x) {
            Ok(max_val) => {
                if (max_val - 999.0).abs() < 1e-5 {
                    println!("PASSED (max = {})", max_val);
                } else {
                    println!("FAILED (expected 999.0, got {})", max_val);
                    all_passed = false;
                }
            }
            Err(e) => {
                println!("ERROR: {}", e);
                all_passed = false;
            }
        }

        // Test 11: Min Reduction
        print!("Testing CUDA min_f32... ");
        let mut x = vec![10.0f32; 1000];
        x[500] = -5.0;
        match device.min_f32(&x) {
            Ok(min_val) => {
                if (min_val - (-5.0)).abs() < 1e-5 {
                    println!("PASSED (min = {})", min_val);
                } else {
                    println!("FAILED (expected -5.0, got {})", min_val);
                    all_passed = false;
                }
            }
            Err(e) => {
                println!("ERROR: {}", e);
                all_passed = false;
            }
        }

        // Test 12: Matrix Multiplication
        print!("Testing CUDA matmul_f32... ");
        // A = [[1, 2, 3], [4, 5, 6]] (2x3)
        // B = [[7, 8], [9, 10], [11, 12]] (3x2)
        // C = [[58, 64], [139, 154]] (2x2)
        let a = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0];
        let b = vec![7.0f32, 8.0, 9.0, 10.0, 11.0, 12.0];
        let mut c = vec![0.0f32; 4];
        match device.matmul_f32(&a, &b, &mut c, 2, 2, 3) {
            Ok(_) => {
                let expected = vec![58.0, 64.0, 139.0, 154.0];
                if verify_vec(&c, &expected, 1e-3) {
                    println!("PASSED");
                } else {
                    println!("FAILED (got {:?})", c);
                    all_passed = false;
                }
            }
            Err(e) => {
                println!("ERROR: {}", e);
                all_passed = false;
            }
        }

        // Test 13: Large Matrix Multiplication
        print!("Testing CUDA matmul_f32 (256x256)... ");
        let n = 256;
        let a: Vec<f32> = (0..n*n).map(|i| (i % 10) as f32 / 10.0).collect();
        let b: Vec<f32> = (0..n*n).map(|i| (i % 10) as f32 / 10.0).collect();
        let mut c = vec![0.0f32; n * n];
        let start = Instant::now();
        match device.matmul_f32(&a, &b, &mut c, n, n, n) {
            Ok(_) => {
                let elapsed = start.elapsed();
                // Verify a sample of results
                let sum: f32 = c.iter().sum();
                if sum.is_finite() && sum > 0.0 {
                    println!("PASSED ({:.2}ms, sum={})", elapsed.as_secs_f64() * 1000.0, sum);
                } else {
                    println!("FAILED (invalid result)");
                    all_passed = false;
                }
            }
            Err(e) => {
                println!("ERROR: {}", e);
                all_passed = false;
            }
        }

        // Synchronize to ensure all operations complete
        let _ = device.synchronize();

        println!("\n{}", "=".repeat(70));

        if all_passed {
            println!("\n  All CUDA operations verified successfully!\n");
        } else {
            println!("\n  Some tests FAILED!\n");
        }

        // Print execution statistics
        println!("{}", GLOBAL_STATS.format_report());

        // Performance benchmark
        println!("\n╔══════════════════════════════════════════════════════════════════╗");
        println!("║           CUDA PERFORMANCE BENCHMARK                             ║");
        println!("╚══════════════════════════════════════════════════════════════════╝\n");

        // Large array addition benchmark
        let n = 10_000_000;
        let a: Vec<f32> = (0..n).map(|i| i as f32).collect();
        let b: Vec<f32> = (0..n).map(|i| (n - i) as f32).collect();
        let mut c = vec![0.0f32; n];

        // Warm up
        let _ = device.add_f32(&a, &b, &mut c);

        // Benchmark
        let iterations = 10;
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = device.add_f32(&a, &b, &mut c);
        }
        let _ = device.synchronize();
        let elapsed = start.elapsed();
        let avg_ms = elapsed.as_secs_f64() * 1000.0 / iterations as f64;
        let throughput = (n as f64 * 3.0 * 4.0) / (avg_ms / 1000.0) / 1e9; // GB/s (read a, b, write c)

        println!("  Add (10M elements): {:.2}ms avg, {:.1} GB/s", avg_ms, throughput);

        // MatMul benchmark
        let n = 512;
        let a: Vec<f32> = (0..n*n).map(|i| (i % 100) as f32 / 100.0).collect();
        let b: Vec<f32> = (0..n*n).map(|i| (i % 100) as f32 / 100.0).collect();
        let mut c = vec![0.0f32; n * n];

        // Warm up
        let _ = device.matmul_f32(&a, &b, &mut c, n, n, n);

        // Benchmark
        let iterations = 10;
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = device.matmul_f32(&a, &b, &mut c, n, n, n);
        }
        let _ = device.synchronize();
        let elapsed = start.elapsed();
        let avg_ms = elapsed.as_secs_f64() * 1000.0 / iterations as f64;
        let gflops = (2.0 * (n as f64).powi(3)) / (avg_ms / 1000.0) / 1e9;

        println!("  MatMul (512x512): {:.2}ms avg, {:.1} GFLOPS", avg_ms, gflops);
    }

    #[cfg(not(feature = "cuda"))]
    {
        println!("  CUDA feature is not enabled.");
        println!("  To enable CUDA, rebuild with:");
        println!("    cargo run --example verify_cuda --release -p xdl-amp --features cuda");
    }
}

fn verify_vec(actual: &[f32], expected: &[f32], tolerance: f32) -> bool {
    if actual.len() != expected.len() {
        return false;
    }
    actual.iter().zip(expected.iter()).all(|(a, e)| (a - e).abs() < tolerance)
}
