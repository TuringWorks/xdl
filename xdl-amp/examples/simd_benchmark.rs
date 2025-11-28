//! Benchmark comparing SIMD vs naive CPU operations
//!
//! Run with: cargo run --example simd_benchmark --release -p xdl-amp

use std::time::Instant;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║              XDL-AMP SIMD BENCHMARK                              ║");
    println!("╠══════════════════════════════════════════════════════════════════╣");

    // Test sizes
    let sizes = [1_000, 10_000, 100_000, 1_000_000, 10_000_000];

    println!("║                                                                  ║");
    println!("║  Element-wise Addition (a + b)                                   ║");
    println!("╠════════════╤═══════════════╤═══════════════╤════════════════════╣");
    println!("║   Elements │  Naive (ms)   │  SIMD (ms)    │  Speedup           ║");
    println!("╠════════════╪═══════════════╪═══════════════╪════════════════════╣");

    for &size in &sizes {
        let a: Vec<f32> = (0..size).map(|i| i as f32).collect();
        let b: Vec<f32> = (0..size).map(|i| (i * 2) as f32).collect();
        let mut c_naive = vec![0.0f32; size];
        let mut c_simd = vec![0.0f32; size];

        // Warm up
        naive_add(&a, &b, &mut c_naive);
        xdl_amp::simd_ops::add_f32(&a, &b, &mut c_simd);

        // Benchmark naive
        let iterations = if size > 1_000_000 { 10 } else { 100 };
        let start = Instant::now();
        for _ in 0..iterations {
            naive_add(&a, &b, &mut c_naive);
        }
        let naive_time = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

        // Benchmark SIMD
        let start = Instant::now();
        for _ in 0..iterations {
            xdl_amp::simd_ops::add_f32(&a, &b, &mut c_simd);
        }
        let simd_time = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

        let speedup = naive_time / simd_time;

        println!(
            "║ {:>10} │ {:>13.4} │ {:>13.4} │ {:>15.2}x  ║",
            format_number(size),
            naive_time,
            simd_time,
            speedup
        );
    }

    println!("╠════════════════════════════════════════════════════════════════════╣");
    println!("║                                                                    ║");
    println!("║  Matrix Multiplication (C = A × B)                                 ║");
    println!("╠════════════╤═══════════════╤═══════════════╤════════════════════╣");
    println!("║   Size     │  Naive (ms)   │  Optimized    │  Speedup           ║");
    println!("╠════════════╪═══════════════╪═══════════════╪════════════════════╣");

    let mat_sizes = [64, 128, 256, 512];

    for &n in &mat_sizes {
        let a: Vec<f32> = (0..n*n).map(|i| (i % 100) as f32 / 100.0).collect();
        let b: Vec<f32> = (0..n*n).map(|i| (i % 100) as f32 / 100.0).collect();
        let mut c_naive = vec![0.0f32; n * n];
        let mut c_opt = vec![0.0f32; n * n];

        // Warm up
        naive_matmul(&a, &b, &mut c_naive, n, n, n);
        xdl_amp::simd_ops::matmul_f32(&a, &b, &mut c_opt, n, n, n);

        // Benchmark naive
        let iterations = if n >= 512 { 3 } else { 10 };
        let start = Instant::now();
        for _ in 0..iterations {
            naive_matmul(&a, &b, &mut c_naive, n, n, n);
        }
        let naive_time = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

        // Benchmark optimized
        let start = Instant::now();
        for _ in 0..iterations {
            xdl_amp::simd_ops::matmul_f32(&a, &b, &mut c_opt, n, n, n);
        }
        let opt_time = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

        let speedup = naive_time / opt_time;

        println!(
            "║ {:>10} │ {:>13.4} │ {:>13.4} │ {:>15.2}x  ║",
            format!("{}x{}", n, n),
            naive_time,
            opt_time,
            speedup
        );
    }

    println!("╠════════════════════════════════════════════════════════════════════╣");
    println!("║                                                                    ║");
    println!("║  Sum Reduction                                                     ║");
    println!("╠════════════╤═══════════════╤═══════════════╤════════════════════╣");
    println!("║   Elements │  Naive (ms)   │  SIMD (ms)    │  Speedup           ║");
    println!("╠════════════╪═══════════════╪═══════════════╪════════════════════╣");

    for &size in &sizes {
        let x: Vec<f32> = (0..size).map(|i| (i % 1000) as f32 / 1000.0).collect();

        // Warm up
        let _ = naive_sum(&x);
        let _ = xdl_amp::simd_ops::sum_f32(&x);

        // Benchmark naive
        let iterations = if size > 1_000_000 { 10 } else { 100 };
        let start = Instant::now();
        for _ in 0..iterations {
            std::hint::black_box(naive_sum(&x));
        }
        let naive_time = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

        // Benchmark SIMD
        let start = Instant::now();
        for _ in 0..iterations {
            std::hint::black_box(xdl_amp::simd_ops::sum_f32(&x));
        }
        let simd_time = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

        let speedup = naive_time / simd_time;

        println!(
            "║ {:>10} │ {:>13.4} │ {:>13.4} │ {:>15.2}x  ║",
            format_number(size),
            naive_time,
            simd_time,
            speedup
        );
    }

    println!("╚══════════════════════════════════════════════════════════════════╝");
}

// Naive implementations for comparison
fn naive_add(a: &[f32], b: &[f32], c: &mut [f32]) {
    for i in 0..a.len() {
        c[i] = a[i] + b[i];
    }
}

fn naive_matmul(a: &[f32], b: &[f32], c: &mut [f32], m: usize, n: usize, k: usize) {
    for i in 0..m {
        for j in 0..n {
            let mut sum = 0.0f32;
            for p in 0..k {
                sum += a[i * k + p] * b[p * n + j];
            }
            c[i * n + j] = sum;
        }
    }
}

fn naive_sum(x: &[f32]) -> f32 {
    x.iter().sum()
}

fn format_number(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{}M", n / 1_000_000)
    } else if n >= 1_000 {
        format!("{}K", n / 1_000)
    } else {
        n.to_string()
    }
}
