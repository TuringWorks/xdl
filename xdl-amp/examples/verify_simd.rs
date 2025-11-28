//! SIMD Operations Verification Example
//!
//! Verifies that SIMD-optimized CPU operations produce correct results
//! and demonstrates the execution statistics output.
//!
//! Run with: cargo run --example verify_simd --release -p xdl-amp

use xdl_amp::simd_ops;
use xdl_amp::stats::GLOBAL_STATS;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║           XDL-AMP SIMD VERIFICATION                              ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    // Reset stats for clean measurement
    GLOBAL_STATS.reset();
    GLOBAL_STATS.set_backend("CPU (SIMD - AVX/SSE/NEON)");

    let mut all_passed = true;

    // Test 1: Element-wise Addition
    print!("Testing add_f32... ");
    let a = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let b = vec![
        10.0f32, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0,
    ];
    let mut c = vec![0.0f32; 10];
    simd_ops::add_f32(&a, &b, &mut c);
    let expected = vec![11.0, 22.0, 33.0, 44.0, 55.0, 66.0, 77.0, 88.0, 99.0, 110.0];
    if verify_vec(&c, &expected, 1e-6) {
        println!("PASSED");
    } else {
        println!("FAILED");
        all_passed = false;
    }

    // Test 2: Element-wise Subtraction
    print!("Testing sub_f32... ");
    let mut c = vec![0.0f32; 10];
    simd_ops::sub_f32(&b, &a, &mut c);
    let expected = vec![9.0, 18.0, 27.0, 36.0, 45.0, 54.0, 63.0, 72.0, 81.0, 90.0];
    if verify_vec(&c, &expected, 1e-6) {
        println!("PASSED");
    } else {
        println!("FAILED");
        all_passed = false;
    }

    // Test 3: Element-wise Multiplication
    print!("Testing mul_f32... ");
    let a = vec![2.0f32; 100];
    let b = vec![3.0f32; 100];
    let mut c = vec![0.0f32; 100];
    simd_ops::mul_f32(&a, &b, &mut c);
    if c.iter().all(|&x| (x - 6.0).abs() < 1e-6) {
        println!("PASSED");
    } else {
        println!("FAILED");
        all_passed = false;
    }

    // Test 4: Element-wise Division
    print!("Testing div_f32... ");
    let a = vec![10.0f32; 100];
    let b = vec![2.0f32; 100];
    let mut c = vec![0.0f32; 100];
    simd_ops::div_f32(&a, &b, &mut c);
    if c.iter().all(|&x| (x - 5.0).abs() < 1e-6) {
        println!("PASSED");
    } else {
        println!("FAILED");
        all_passed = false;
    }

    // Test 5: Square Root
    print!("Testing sqrt_f32... ");
    let x = vec![4.0f32, 9.0, 16.0, 25.0, 36.0, 49.0, 64.0, 81.0];
    let mut y = vec![0.0f32; 8];
    simd_ops::sqrt_f32(&x, &mut y);
    let expected = vec![2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
    if verify_vec(&y, &expected, 1e-6) {
        println!("PASSED");
    } else {
        println!("FAILED");
        all_passed = false;
    }

    // Test 6: Trigonometric Functions
    print!("Testing sin_f32... ");
    let x = vec![0.0f32, std::f32::consts::PI / 2.0, std::f32::consts::PI];
    let mut y = vec![0.0f32; 3];
    simd_ops::sin_f32(&x, &mut y);
    if (y[0] - 0.0).abs() < 1e-6 && (y[1] - 1.0).abs() < 1e-6 && y[2].abs() < 1e-5 {
        println!("PASSED");
    } else {
        println!("FAILED (got {:?})", y);
        all_passed = false;
    }

    print!("Testing cos_f32... ");
    let mut y = vec![0.0f32; 3];
    simd_ops::cos_f32(&x, &mut y);
    if (y[0] - 1.0).abs() < 1e-6 && y[1].abs() < 1e-6 && (y[2] - (-1.0)).abs() < 1e-5 {
        println!("PASSED");
    } else {
        println!("FAILED (got {:?})", y);
        all_passed = false;
    }

    // Test 7: Exponential and Logarithm
    print!("Testing exp_f32... ");
    let x = vec![0.0f32, 1.0, 2.0];
    let mut y = vec![0.0f32; 3];
    simd_ops::exp_f32(&x, &mut y);
    if (y[0] - 1.0).abs() < 1e-6 && (y[1] - std::f32::consts::E).abs() < 1e-5 {
        println!("PASSED");
    } else {
        println!("FAILED");
        all_passed = false;
    }

    print!("Testing log_f32... ");
    let x = vec![
        1.0f32,
        std::f32::consts::E,
        std::f32::consts::E * std::f32::consts::E,
    ];
    let mut y = vec![0.0f32; 3];
    simd_ops::log_f32(&x, &mut y);
    if (y[0] - 0.0).abs() < 1e-6 && (y[1] - 1.0).abs() < 1e-5 && (y[2] - 2.0).abs() < 1e-4 {
        println!("PASSED");
    } else {
        println!("FAILED");
        all_passed = false;
    }

    // Test 8: Sum Reduction
    print!("Testing sum_f32... ");
    let x: Vec<f32> = (1..=100).map(|i| i as f32).collect();
    let sum = simd_ops::sum_f32(&x);
    let expected_sum = 5050.0; // n*(n+1)/2 for n=100
    if (sum - expected_sum).abs() < 1e-3 {
        println!("PASSED (sum = {})", sum);
    } else {
        println!("FAILED (expected {}, got {})", expected_sum, sum);
        all_passed = false;
    }

    // Test 9: Max/Min Reduction
    print!("Testing max_f32... ");
    let x = vec![3.0f32, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0, 6.0];
    let max_val = simd_ops::max_f32(&x);
    if (max_val - 9.0).abs() < 1e-6 {
        println!("PASSED (max = {})", max_val);
    } else {
        println!("FAILED");
        all_passed = false;
    }

    print!("Testing min_f32... ");
    let min_val = simd_ops::min_f32(&x);
    if (min_val - 1.0).abs() < 1e-6 {
        println!("PASSED (min = {})", min_val);
    } else {
        println!("FAILED");
        all_passed = false;
    }

    // Test 10: Matrix Multiplication
    print!("Testing matmul_f32... ");
    // 2x3 matrix * 3x2 matrix = 2x2 result
    // A = [[1, 2, 3], [4, 5, 6]]
    // B = [[7, 8], [9, 10], [11, 12]]
    // C = [[58, 64], [139, 154]]
    let a = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0];
    let b = vec![7.0f32, 8.0, 9.0, 10.0, 11.0, 12.0];
    let mut c = vec![0.0f32; 4];
    simd_ops::matmul_f32(&a, &b, &mut c, 2, 2, 3);
    let expected = vec![58.0, 64.0, 139.0, 154.0];
    if verify_vec(&c, &expected, 1e-4) {
        println!("PASSED");
    } else {
        println!("FAILED (got {:?})", c);
        all_passed = false;
    }

    // Test 11: Dot Product
    print!("Testing dot_f32... ");
    let a = vec![1.0f32, 2.0, 3.0, 4.0];
    let b = vec![5.0f32, 6.0, 7.0, 8.0];
    let dot = simd_ops::dot_f32(&a, &b);
    let expected_dot = 70.0; // 1*5 + 2*6 + 3*7 + 4*8
    if (dot - expected_dot).abs() < 1e-5 {
        println!("PASSED (dot = {})", dot);
    } else {
        println!("FAILED");
        all_passed = false;
    }

    // Test 12: Fused Multiply-Add
    print!("Testing fma_f32... ");
    let a = vec![2.0f32; 16];
    let b = vec![3.0f32; 16];
    let d = vec![1.0f32; 16];
    let mut c = vec![0.0f32; 16];
    simd_ops::fma_f32(&a, &b, &d, &mut c);
    if c.iter().all(|&x| (x - 7.0).abs() < 1e-6) {
        // 2*3 + 1 = 7
        println!("PASSED");
    } else {
        println!("FAILED");
        all_passed = false;
    }

    // Test 13: AXPY (alpha * x + y)
    print!("Testing axpy_f32... ");
    let x = vec![1.0f32; 16];
    let y = vec![2.0f32; 16];
    let mut z = vec![0.0f32; 16];
    simd_ops::axpy_f32(3.0, &x, &y, &mut z); // 3*1 + 2 = 5
    if z.iter().all(|&x| (x - 5.0).abs() < 1e-6) {
        println!("PASSED");
    } else {
        println!("FAILED");
        all_passed = false;
    }

    // Test 14: Large Array (parallel execution)
    print!("Testing parallel execution (200K elements)... ");
    let n = 200_000;
    let a: Vec<f32> = (0..n).map(|i| i as f32).collect();
    let b: Vec<f32> = (0..n).map(|i| (n - i) as f32).collect();
    let mut c = vec![0.0f32; n];
    simd_ops::add_f32(&a, &b, &mut c);
    // Each element should be n (i + (n-i) = n)
    if c.iter().all(|&x| (x - n as f32).abs() < 1e-3) {
        println!("PASSED");
    } else {
        println!("FAILED");
        all_passed = false;
    }

    println!("\n{}", "=".repeat(70));

    if all_passed {
        println!("\n  All SIMD operations verified successfully!\n");
    } else {
        println!("\n  Some tests FAILED!\n");
    }

    // Print execution statistics
    println!("{}", GLOBAL_STATS.format_report());
}

fn verify_vec(actual: &[f32], expected: &[f32], tolerance: f32) -> bool {
    if actual.len() != expected.len() {
        return false;
    }
    actual
        .iter()
        .zip(expected.iter())
        .all(|(a, e)| (a - e).abs() < tolerance)
}
