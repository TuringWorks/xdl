//! MLX vs Metal vs CPU Performance Benchmark
//!
//! This benchmark compares performance of MLX, Metal, and CPU implementations
//! for various array operations on Apple Silicon.
//!
//! Run with: cargo run --example mlx_benchmark --features mlx --release

use std::time::{Duration, Instant};

#[cfg(all(target_os = "macos", feature = "mlx"))]
use xdl_amp::MLXOps;

use xdl_amp::metal::MetalDevice;
use xdl_amp::backend::GpuDevice;

fn main() {
    println!("========================================");
    println!("  MLX vs Metal vs CPU Benchmark");
    println!("  Apple Silicon Performance Test");
    println!("========================================\n");

    // Test sizes: small, medium, large, very large
    let sizes = [1_000, 10_000, 100_000, 1_000_000, 10_000_000];

    for &size in &sizes {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("  Array Size: {} elements ({:.1} MB)",
                 size,
                 (size * 4) as f64 / 1_000_000.0);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        benchmark_operations(size);
    }

    println!("\n========================================");
    println!("  Benchmark Complete");
    println!("========================================");
}

fn benchmark_operations(size: usize) {
    // Generate test data
    let a: Vec<f32> = (0..size).map(|i| (i as f32) * 0.001).collect();
    let b: Vec<f32> = (0..size).map(|i| ((i + 100) as f32) * 0.001).collect();
    let mut c = vec![0.0f32; size];

    // Warm up
    let _ = cpu_add(&a, &b, &mut c);

    // Benchmark each backend
    let iterations = if size >= 1_000_000 { 5 } else { 20 };

    println!("\n  Operation: Vector Addition (c = a + b)");
    println!("  ─────────────────────────────────────");

    // CPU baseline
    let cpu_time = benchmark_cpu_add(&a, &b, &mut c, iterations);
    println!("  CPU:   {:>10.3} ms  (baseline)", cpu_time.as_secs_f64() * 1000.0);

    // Metal
    match MetalDevice::new() {
        Ok(metal) => {
            let metal_time = benchmark_metal_add(&metal, &a, &b, &mut c, iterations);
            let speedup = cpu_time.as_secs_f64() / metal_time.as_secs_f64();
            println!("  Metal: {:>10.3} ms  ({:.2}x {})",
                     metal_time.as_secs_f64() * 1000.0,
                     speedup.abs(),
                     if speedup > 1.0 { "faster" } else { "slower" });
        }
        Err(e) => println!("  Metal: unavailable ({:?})", e),
    }

    // MLX
    #[cfg(all(target_os = "macos", feature = "mlx"))]
    {
        match MLXOps::new() {
            Ok(mlx) => {
                let mlx_time = benchmark_mlx_add(&mlx, &a, &b, &mut c, iterations);
                let speedup = cpu_time.as_secs_f64() / mlx_time.as_secs_f64();
                println!("  MLX:   {:>10.3} ms  ({:.2}x {})",
                         mlx_time.as_secs_f64() * 1000.0,
                         speedup.abs(),
                         if speedup > 1.0 { "faster" } else { "slower" });
            }
            Err(e) => println!("  MLX:   unavailable ({:?})", e),
        }
    }

    // Matrix multiplication benchmark (if size is square-able)
    let mat_size = (size as f64).sqrt() as usize;
    if mat_size * mat_size <= size && mat_size >= 32 {
        let m = mat_size;
        let n = mat_size;
        let k = mat_size;

        let mat_a: Vec<f32> = (0..(m * k)).map(|i| (i as f32) * 0.001).collect();
        let mat_b: Vec<f32> = (0..(k * n)).map(|i| (i as f32) * 0.001).collect();
        let mut mat_c = vec![0.0f32; m * n];

        println!("\n  Operation: Matrix Multiplication ({}x{} @ {}x{})", m, k, k, n);
        println!("  ─────────────────────────────────────────────────");

        // CPU matmul
        let cpu_time = benchmark_cpu_matmul(&mat_a, &mat_b, &mut mat_c, m, n, k, 3);
        println!("  CPU:   {:>10.3} ms  (baseline)", cpu_time.as_secs_f64() * 1000.0);

        // Metal matmul
        match MetalDevice::new() {
            Ok(metal) => {
                let metal_time = benchmark_metal_matmul(&metal, &mat_a, &mat_b, &mut mat_c, m, n, k, 3);
                let speedup = cpu_time.as_secs_f64() / metal_time.as_secs_f64();
                println!("  Metal: {:>10.3} ms  ({:.2}x {})",
                         metal_time.as_secs_f64() * 1000.0,
                         speedup.abs(),
                         if speedup > 1.0 { "faster" } else { "slower" });
            }
            Err(_) => {}
        }

        // MLX matmul
        #[cfg(all(target_os = "macos", feature = "mlx"))]
        {
            match MLXOps::new() {
                Ok(mlx) => {
                    let mlx_time = benchmark_mlx_matmul(&mlx, &mat_a, &mat_b, &mut mat_c, m, n, k, 3);
                    let speedup = cpu_time.as_secs_f64() / mlx_time.as_secs_f64();
                    println!("  MLX:   {:>10.3} ms  ({:.2}x {})",
                             mlx_time.as_secs_f64() * 1000.0,
                             speedup.abs(),
                             if speedup > 1.0 { "faster" } else { "slower" });
                }
                Err(_) => {}
            }
        }
    }

    // Transcendental functions benchmark
    println!("\n  Operation: sin(x) (element-wise)");
    println!("  ─────────────────────────────────");

    // CPU sin
    let cpu_time = benchmark_cpu_sin(&a, &mut c, iterations);
    println!("  CPU:   {:>10.3} ms  (baseline)", cpu_time.as_secs_f64() * 1000.0);

    // Metal sin
    match MetalDevice::new() {
        Ok(metal) => {
            let metal_time = benchmark_metal_sin(&metal, &a, &mut c, iterations);
            let speedup = cpu_time.as_secs_f64() / metal_time.as_secs_f64();
            println!("  Metal: {:>10.3} ms  ({:.2}x {})",
                     metal_time.as_secs_f64() * 1000.0,
                     speedup.abs(),
                     if speedup > 1.0 { "faster" } else { "slower" });
        }
        Err(_) => {}
    }

    // MLX sin
    #[cfg(all(target_os = "macos", feature = "mlx"))]
    {
        match MLXOps::new() {
            Ok(mlx) => {
                let mlx_time = benchmark_mlx_sin(&mlx, &a, &mut c, iterations);
                let speedup = cpu_time.as_secs_f64() / mlx_time.as_secs_f64();
                println!("  MLX:   {:>10.3} ms  ({:.2}x {})",
                         mlx_time.as_secs_f64() * 1000.0,
                         speedup.abs(),
                         if speedup > 1.0 { "faster" } else { "slower" });
            }
            Err(_) => {}
        }
    }

    // Reduction operations
    println!("\n  Operation: sum(x) (reduction)");
    println!("  ─────────────────────────────");

    // CPU sum
    let cpu_time = benchmark_cpu_sum(&a, iterations);
    println!("  CPU:   {:>10.3} ms  (baseline)", cpu_time.as_secs_f64() * 1000.0);

    // Metal sum
    match MetalDevice::new() {
        Ok(metal) => {
            let metal_time = benchmark_metal_sum(&metal, &a, iterations);
            let speedup = cpu_time.as_secs_f64() / metal_time.as_secs_f64();
            println!("  Metal: {:>10.3} ms  ({:.2}x {})",
                     metal_time.as_secs_f64() * 1000.0,
                     speedup.abs(),
                     if speedup > 1.0 { "faster" } else { "slower" });
        }
        Err(_) => {}
    }

    // MLX sum
    #[cfg(all(target_os = "macos", feature = "mlx"))]
    {
        match MLXOps::new() {
            Ok(mlx) => {
                let mlx_time = benchmark_mlx_sum(&mlx, &a, iterations);
                let speedup = cpu_time.as_secs_f64() / mlx_time.as_secs_f64();
                println!("  MLX:   {:>10.3} ms  ({:.2}x {})",
                         mlx_time.as_secs_f64() * 1000.0,
                         speedup.abs(),
                         if speedup > 1.0 { "faster" } else { "slower" });
            }
            Err(_) => {}
        }
    }
}

// CPU implementations
fn cpu_add(a: &[f32], b: &[f32], c: &mut [f32]) {
    for i in 0..a.len() {
        c[i] = a[i] + b[i];
    }
}

fn benchmark_cpu_add(a: &[f32], b: &[f32], c: &mut [f32], iterations: usize) -> Duration {
    let start = Instant::now();
    for _ in 0..iterations {
        cpu_add(a, b, c);
    }
    start.elapsed() / iterations as u32
}

fn benchmark_cpu_matmul(a: &[f32], b: &[f32], c: &mut [f32], m: usize, n: usize, k: usize, iterations: usize) -> Duration {
    let start = Instant::now();
    for _ in 0..iterations {
        for i in 0..m {
            for j in 0..n {
                let mut sum = 0.0f32;
                for l in 0..k {
                    sum += a[i * k + l] * b[l * n + j];
                }
                c[i * n + j] = sum;
            }
        }
    }
    start.elapsed() / iterations as u32
}

fn benchmark_cpu_sin(a: &[f32], c: &mut [f32], iterations: usize) -> Duration {
    let start = Instant::now();
    for _ in 0..iterations {
        for i in 0..a.len() {
            c[i] = a[i].sin();
        }
    }
    start.elapsed() / iterations as u32
}

fn benchmark_cpu_sum(a: &[f32], iterations: usize) -> Duration {
    let start = Instant::now();
    for _ in 0..iterations {
        let _sum: f32 = a.iter().sum();
    }
    start.elapsed() / iterations as u32
}

// Metal implementations
fn benchmark_metal_add(metal: &MetalDevice, a: &[f32], b: &[f32], c: &mut [f32], iterations: usize) -> Duration {
    // Warm up
    let _ = metal.add_f32(a, b, c);

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = metal.add_f32(a, b, c);
    }
    start.elapsed() / iterations as u32
}

fn benchmark_metal_matmul(metal: &MetalDevice, a: &[f32], b: &[f32], c: &mut [f32], m: usize, n: usize, k: usize, iterations: usize) -> Duration {
    // Warm up
    let _ = metal.matmul_f32(a, b, c, m, n, k);

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = metal.matmul_f32(a, b, c, m, n, k);
    }
    start.elapsed() / iterations as u32
}

fn benchmark_metal_sin(metal: &MetalDevice, a: &[f32], c: &mut [f32], iterations: usize) -> Duration {
    // Warm up
    let _ = metal.sin_f32(a, c);

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = metal.sin_f32(a, c);
    }
    start.elapsed() / iterations as u32
}

fn benchmark_metal_sum(metal: &MetalDevice, a: &[f32], iterations: usize) -> Duration {
    // Warm up
    let _ = metal.sum_f32(a);

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = metal.sum_f32(a);
    }
    start.elapsed() / iterations as u32
}

// MLX implementations
#[cfg(all(target_os = "macos", feature = "mlx"))]
fn benchmark_mlx_add(mlx: &MLXOps, a: &[f32], b: &[f32], c: &mut [f32], iterations: usize) -> Duration {
    // Warm up
    let _ = mlx.add_f32(a, b, c);

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = mlx.add_f32(a, b, c);
    }
    start.elapsed() / iterations as u32
}

#[cfg(all(target_os = "macos", feature = "mlx"))]
fn benchmark_mlx_matmul(mlx: &MLXOps, a: &[f32], b: &[f32], c: &mut [f32], m: usize, n: usize, k: usize, iterations: usize) -> Duration {
    // Warm up
    let _ = mlx.matmul_f32(a, b, c, m, n, k);

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = mlx.matmul_f32(a, b, c, m, n, k);
    }
    start.elapsed() / iterations as u32
}

#[cfg(all(target_os = "macos", feature = "mlx"))]
fn benchmark_mlx_sin(mlx: &MLXOps, a: &[f32], c: &mut [f32], iterations: usize) -> Duration {
    // Warm up
    let _ = mlx.sin_f32(a, c);

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = mlx.sin_f32(a, c);
    }
    start.elapsed() / iterations as u32
}

#[cfg(all(target_os = "macos", feature = "mlx"))]
fn benchmark_mlx_sum(mlx: &MLXOps, a: &[f32], iterations: usize) -> Duration {
    // Warm up
    let _ = mlx.sum_f32(a);

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = mlx.sum_f32(a);
    }
    start.elapsed() / iterations as u32
}
