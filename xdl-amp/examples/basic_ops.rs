use ndarray::Array1;
use xdl_amp::{ops::GpuOps, GpuContext};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("XDL AMP - GPU Accelerated Math Processing Demo\n");

    // Create GPU context (automatically selects best backend for platform)
    let ctx = GpuContext::new()?;
    println!("✓ GPU Backend: {}\n", ctx.backend_name());

    // Create GPU operations handler
    let gpu_ops = GpuOps::new(ctx.device().clone());

    // Test data
    let size = 1000;
    let a = Array1::from_vec((0..size).map(|i| i as f32).collect());
    let b = Array1::from_vec((0..size).map(|i| (i * 2) as f32).collect());

    println!("Testing GPU operations on arrays of size {}...\n", size);

    // Element-wise addition
    println!("1. Element-wise addition (c = a + b)");
    let c = gpu_ops.add_1d(&a, &b)?;
    println!(
        "   Result (first 5): {:?}",
        &c.as_slice().unwrap()[..5.min(size)]
    );

    // Element-wise multiplication
    println!("\n2. Element-wise multiplication (c = a * b)");
    let c = gpu_ops.mul_1d(&a, &b)?;
    println!(
        "   Result (first 5): {:?}",
        &c.as_slice().unwrap()[..5.min(size)]
    );

    // Trigonometric functions
    let angles = Array1::from_vec(vec![
        0.0,
        std::f32::consts::PI / 6.0,
        std::f32::consts::PI / 4.0,
        std::f32::consts::PI / 3.0,
        std::f32::consts::PI / 2.0,
    ]);

    println!("\n3. Sine function");
    let sin_result = gpu_ops.sin_1d(&angles)?;
    println!("   Input angles: {:?}", angles);
    println!("   sin(angles): {:?}", sin_result);

    println!("\n4. Cosine function");
    let cos_result = gpu_ops.cos_1d(&angles)?;
    println!("   cos(angles): {:?}", cos_result);

    // Exponential
    let small_vals = Array1::from_vec(vec![0.0, 1.0, 2.0, 3.0]);
    println!("\n5. Exponential function");
    let exp_result = gpu_ops.exp_1d(&small_vals)?;
    println!("   Input: {:?}", small_vals);
    println!("   exp(input): {:?}", exp_result);

    println!("\n✓ All GPU operations completed successfully!");

    Ok(())
}
