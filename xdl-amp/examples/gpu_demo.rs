// XDL AMP GPU Acceleration Demo
// This example demonstrates the multi-backend GPU acceleration capabilities

use ndarray::{Array1, Array2};
use xdl_amp::{ops::GpuOps, GpuContext};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("XDL AMP GPU Acceleration Demo");
    println!("========================================");
    println!();

    // ========================================================================
    // Part 1: GPU Backend Detection
    // ========================================================================
    println!("1. GPU Backend Detection");
    println!("----------------------------------------");

    let ctx = GpuContext::new()?;
    println!("✓ Active GPU Backend: {}", ctx.backend_name());
    println!();

    let gpu_ops = GpuOps::new(ctx.device().clone());

    // ========================================================================
    // Part 2: Element-wise Operations
    // ========================================================================
    println!("2. Element-wise Array Operations");
    println!("----------------------------------------");

    let size = 100_000;
    println!("Array size: {} elements", size);
    println!();

    // Generate test data
    let a = Array1::from_vec((0..size).map(|i| i as f32).collect());
    let b = Array1::from_vec((0..size).map(|i| (i * 2) as f32).collect());

    // CPU operation
    let start = std::time::Instant::now();
    let c_cpu: Array1<f32> = &a + &b;
    let cpu_time = start.elapsed();
    println!("CPU Time (addition): {:?}", cpu_time);

    // GPU operation
    let start = std::time::Instant::now();
    let c_gpu = gpu_ops.add_1d(&a, &b)?;
    let gpu_time = start.elapsed();
    println!("GPU Time (addition): {:?}", gpu_time);

    // Verify correctness
    let max_error = c_cpu
        .iter()
        .zip(c_gpu.iter())
        .map(|(cpu, gpu)| (cpu - gpu).abs())
        .fold(0.0, f32::max);
    println!("Maximum error: {:.2e}", max_error);

    let speedup = cpu_time.as_secs_f64() / gpu_time.as_secs_f64();
    println!("Speedup: {:.2}x", speedup);
    println!();

    // ========================================================================
    // Part 3: Mathematical Functions
    // ========================================================================
    println!("3. Trigonometric Functions");
    println!("----------------------------------------");

    let x = Array1::from_vec(
        (0..100_000)
            .map(|i| (i as f32) * std::f32::consts::PI / 50_000.0)
            .collect(),
    );

    // CPU version
    let start = std::time::Instant::now();
    let _y_cpu = x.mapv(|v| v.sin());
    let cpu_time = start.elapsed();
    println!("CPU Time (sin): {:?}", cpu_time);

    // GPU version
    let start = std::time::Instant::now();
    let _y_gpu = gpu_ops.sin_1d(&x)?;
    let gpu_time = start.elapsed();
    println!("GPU Time (sin): {:?}", gpu_time);

    let speedup = cpu_time.as_secs_f64() / gpu_time.as_secs_f64();
    println!("Speedup: {:.2}x", speedup);
    println!();

    // ========================================================================
    // Part 4: Multiple Operations
    // ========================================================================
    println!("4. Chained Operations");
    println!("----------------------------------------");

    let data = Array1::from_vec((0..50_000).map(|i| (i as f32) / 50_000.0).collect());

    // CPU version
    let start = std::time::Instant::now();
    let _result_cpu = data.mapv(|x| x.sin() * x.cos() + x.exp());
    let cpu_time = start.elapsed();
    println!("CPU Time (complex expression): {:?}", cpu_time);

    // GPU version
    let start = std::time::Instant::now();
    let sin_result = gpu_ops.sin_1d(&data)?;
    let cos_result = gpu_ops.cos_1d(&data)?;
    let exp_result = gpu_ops.exp_1d(&data)?;
    let mul_result = gpu_ops.mul_1d(&sin_result, &cos_result)?;
    let _result_gpu = gpu_ops.add_1d(&mul_result, &exp_result)?;
    let gpu_time = start.elapsed();
    println!("GPU Time (complex expression): {:?}", gpu_time);

    let speedup = cpu_time.as_secs_f64() / gpu_time.as_secs_f64();
    println!("Speedup: {:.2}x", speedup);
    println!();

    // ========================================================================
    // Part 5: Matrix Operations
    // ========================================================================
    println!("5. Matrix Multiplication");
    println!("----------------------------------------");

    let m = 256;
    let n = 256;
    let k = 256;

    println!("Matrix size: {}x{} × {}x{}", m, k, k, n);

    let mat_a = Array2::from_shape_fn((m, k), |(i, j)| ((i * k + j) as f32) / 1000.0);
    let mat_b = Array2::from_shape_fn((k, n), |(i, j)| ((i * n + j) as f32) / 1000.0);

    // CPU version
    let start = std::time::Instant::now();
    let _result_cpu = mat_a.dot(&mat_b);
    let cpu_time = start.elapsed();
    println!("CPU Time (matmul): {:?}", cpu_time);

    // GPU version
    let start = std::time::Instant::now();
    let _result_gpu = gpu_ops.matmul(&mat_a, &mat_b)?;
    let gpu_time = start.elapsed();
    println!("GPU Time (matmul): {:?}", gpu_time);

    let speedup = cpu_time.as_secs_f64() / gpu_time.as_secs_f64();
    println!("Speedup: {:.2}x", speedup);
    println!();

    // ========================================================================
    // Part 6: Reduction Operations
    // ========================================================================
    println!("6. Reduction Operations");
    println!("----------------------------------------");

    let large_array = Array1::from_vec((0..1_000_000).map(|i| (i as f32) / 1000.0).collect());

    // Sum
    let start = std::time::Instant::now();
    let sum_cpu = large_array.sum();
    let cpu_time = start.elapsed();
    println!("CPU Time (sum): {:?}", cpu_time);

    let start = std::time::Instant::now();
    let sum_gpu = gpu_ops.sum_1d(&large_array)?;
    let gpu_time = start.elapsed();
    println!("GPU Time (sum): {:?}", gpu_time);
    println!("Difference: {:.2e}", (sum_cpu - sum_gpu).abs());

    let speedup = cpu_time.as_secs_f64() / gpu_time.as_secs_f64();
    println!("Speedup: {:.2}x", speedup);
    println!();

    // ========================================================================
    // Part 7: Performance Summary
    // ========================================================================
    println!("========================================");
    println!("Performance Summary");
    println!("========================================");
    println!();
    println!("Backend: {}", ctx.backend_name());
    println!();
    println!("All operations completed successfully!");
    println!("GPU acceleration is working on your system.");
    println!();

    // ========================================================================
    // Part 8: Backend Information
    // ========================================================================
    println!("========================================");
    println!("System Information");
    println!("========================================");
    println!();
    println!("Platform: {}", std::env::consts::OS);
    println!("Architecture: {}", std::env::consts::ARCH);
    println!("GPU Backend: {}", ctx.backend_name());
    println!();

    #[cfg(target_os = "macos")]
    {
        println!("✓ Running on Apple Silicon with unified memory");
        println!("✓ Metal Performance Shaders available");
        println!("✓ Zero-copy CPU-GPU data transfers");
    }

    #[cfg(target_os = "windows")]
    {
        println!("✓ Running on Windows");
        #[cfg(feature = "directml")]
        println!("✓ DirectML (Windows ML) support enabled");
    }

    #[cfg(feature = "cuda")]
    println!("✓ NVIDIA CUDA support enabled");

    #[cfg(feature = "vulkan")]
    println!("✓ Vulkan support enabled");

    #[cfg(feature = "opencl")]
    println!("✓ OpenCL support enabled");

    println!();
    println!("========================================");
    println!("Demo Complete!");
    println!("========================================");

    Ok(())
}
