//! OpenCL backend for cross-platform GPU acceleration

use crate::backend::{GpuBuffer, GpuDevice};
use crate::error::{GpuError, Result};
use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};
use std::sync::Arc;

/// OpenCL GPU buffer
#[derive(Debug)]
pub struct OpenCLBuffer {
    buffer: Buffer<u8>,
    size: usize,
}

impl GpuBuffer for OpenCLBuffer {
    fn size(&self) -> usize {
        self.size
    }

    fn read_to_slice(&self, dst: &mut [u8]) -> Result<()> {
        if dst.len() != self.size {
            return Err(GpuError::BufferSizeMismatch {
                expected: self.size,
                actual: dst.len(),
            });
        }

        self.buffer.read(dst).wait()?;
        Ok(())
    }

    fn write_from_slice(&mut self, src: &[u8]) -> Result<()> {
        if src.len() != self.size {
            return Err(GpuError::BufferSizeMismatch {
                expected: self.size,
                actual: src.len(),
            });
        }

        self.buffer.write(src).enq()?;
        Ok(())
    }
}

/// OpenCL GPU device
#[derive(Debug)]
pub struct OpenCLDevice {
    #[allow(dead_code)]
    context: Context,
    queue: Queue,
    program: Program,
    device_name: String,
}

impl OpenCLDevice {
    /// Create a new OpenCL device
    pub fn new() -> Result<Self> {
        let platform = Platform::default();
        let device = Device::first(platform)?;
        let device_name = device.name()?;

        let context = Context::builder()
            .platform(platform)
            .devices(device)
            .build()?;

        let queue = Queue::new(&context, device, None)?;

        // OpenCL kernel source code
        let source = include_str!("shaders/opencl_kernels.cl");
        let program = Program::builder()
            .devices(device)
            .src(source)
            .build(&context)?;

        Ok(Self {
            context,
            queue,
            program,
            device_name,
        })
    }

    /// Helper to execute a binary operation kernel (a op b -> c)
    fn execute_binary_kernel(
        &self,
        kernel_name: &str,
        a: &[f32],
        b: &[f32],
        c: &mut [f32],
    ) -> Result<()> {
        let len = a.len();
        if b.len() != len || c.len() != len {
            return Err(GpuError::BufferSizeMismatch {
                expected: len,
                actual: b.len().min(c.len()),
            });
        }

        // Create GPU buffers
        let buf_a = Buffer::<f32>::builder()
            .queue(self.queue.clone())
            .len(len)
            .copy_host_slice(a)
            .build()?;

        let buf_b = Buffer::<f32>::builder()
            .queue(self.queue.clone())
            .len(len)
            .copy_host_slice(b)
            .build()?;

        let buf_c = Buffer::<f32>::builder()
            .queue(self.queue.clone())
            .len(len)
            .build()?;

        // Build and execute kernel
        let kernel = Kernel::builder()
            .program(&self.program)
            .name(kernel_name)
            .queue(self.queue.clone())
            .global_work_size(len)
            .arg(&buf_a)
            .arg(&buf_b)
            .arg(&buf_c)
            .build()?;

        unsafe {
            kernel.enq()?;
        }

        // Read result back
        buf_c.read(c).enq()?;
        self.queue.finish()?;

        Ok(())
    }

    /// Helper to execute a unary operation kernel (f(x) -> y)
    fn execute_unary_kernel(&self, kernel_name: &str, x: &[f32], y: &mut [f32]) -> Result<()> {
        let len = x.len();
        if y.len() != len {
            return Err(GpuError::BufferSizeMismatch {
                expected: len,
                actual: y.len(),
            });
        }

        // Create GPU buffers
        let buf_x = Buffer::<f32>::builder()
            .queue(self.queue.clone())
            .len(len)
            .copy_host_slice(x)
            .build()?;

        let buf_y = Buffer::<f32>::builder()
            .queue(self.queue.clone())
            .len(len)
            .build()?;

        // Build and execute kernel
        let kernel = Kernel::builder()
            .program(&self.program)
            .name(kernel_name)
            .queue(self.queue.clone())
            .global_work_size(len)
            .arg(&buf_x)
            .arg(&buf_y)
            .build()?;

        unsafe {
            kernel.enq()?;
        }

        // Read result back
        buf_y.read(y).enq()?;
        self.queue.finish()?;

        Ok(())
    }
}

impl GpuDevice for OpenCLDevice {
    fn name(&self) -> &str {
        &self.device_name
    }

    fn create_buffer(&self, size: usize) -> Result<Box<dyn GpuBuffer>> {
        let buffer = Buffer::<u8>::builder()
            .queue(self.queue.clone())
            .len(size)
            .build()?;

        Ok(Box::new(OpenCLBuffer { buffer, size }))
    }

    fn create_buffer_with_data(&self, data: &[u8]) -> Result<Box<dyn GpuBuffer>> {
        let buffer = Buffer::<u8>::builder()
            .queue(self.queue.clone())
            .len(data.len())
            .copy_host_slice(data)
            .build()?;

        Ok(Box::new(OpenCLBuffer {
            buffer,
            size: data.len(),
        }))
    }

    fn add_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()> {
        self.execute_binary_kernel("add_f32", a, b, c)
    }

    fn mul_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()> {
        self.execute_binary_kernel("mul_f32", a, b, c)
    }

    fn sub_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()> {
        self.execute_binary_kernel("sub_f32", a, b, c)
    }

    fn div_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()> {
        self.execute_binary_kernel("div_f32", a, b, c)
    }

    fn matmul_f32(
        &self,
        a: &[f32],
        b: &[f32],
        c: &mut [f32],
        m: usize,
        n: usize,
        k: usize,
    ) -> Result<()> {
        // Verify dimensions
        if a.len() != m * k || b.len() != k * n || c.len() != m * n {
            return Err(GpuError::BufferSizeMismatch {
                expected: m * k,
                actual: a.len(),
            });
        }

        // Create GPU buffers
        let buf_a = Buffer::<f32>::builder()
            .queue(self.queue.clone())
            .len(m * k)
            .copy_host_slice(a)
            .build()?;

        let buf_b = Buffer::<f32>::builder()
            .queue(self.queue.clone())
            .len(k * n)
            .copy_host_slice(b)
            .build()?;

        let buf_c = Buffer::<f32>::builder()
            .queue(self.queue.clone())
            .len(m * n)
            .build()?;

        // Build and execute kernel with 2D dispatch
        let kernel = Kernel::builder()
            .program(&self.program)
            .name("matmul_f32")
            .queue(self.queue.clone())
            .global_work_size([n, m]) // (columns, rows)
            .arg(&buf_a)
            .arg(&buf_b)
            .arg(&buf_c)
            .arg(m as u32)
            .arg(n as u32)
            .arg(k as u32)
            .build()?;

        unsafe {
            kernel.enq()?;
        }

        // Read result back
        buf_c.read(c).enq()?;
        self.queue.finish()?;

        Ok(())
    }

    fn sin_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        self.execute_unary_kernel("sin_f32", x, y)
    }

    fn cos_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        self.execute_unary_kernel("cos_f32", x, y)
    }

    fn exp_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        self.execute_unary_kernel("exp_f32", x, y)
    }

    fn log_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        self.execute_unary_kernel("log_f32", x, y)
    }

    fn sqrt_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        self.execute_unary_kernel("sqrt_f32", x, y)
    }

    fn pow_f32(&self, x: &[f32], p: f32, y: &mut [f32]) -> Result<()> {
        let len = x.len();
        if y.len() != len {
            return Err(GpuError::BufferSizeMismatch {
                expected: len,
                actual: y.len(),
            });
        }

        // Create GPU buffers
        let buf_x = Buffer::<f32>::builder()
            .queue(self.queue.clone())
            .len(len)
            .copy_host_slice(x)
            .build()?;

        let buf_y = Buffer::<f32>::builder()
            .queue(self.queue.clone())
            .len(len)
            .build()?;

        // Build and execute kernel with scalar parameter
        let kernel = Kernel::builder()
            .program(&self.program)
            .name("pow_f32")
            .queue(self.queue.clone())
            .global_work_size(len)
            .arg(&buf_x)
            .arg(p)
            .arg(&buf_y)
            .build()?;

        unsafe {
            kernel.enq()?;
        }

        // Read result back
        buf_y.read(y).enq()?;
        self.queue.finish()?;

        Ok(())
    }

    fn sum_f32(&self, x: &[f32]) -> Result<f32> {
        let len = x.len();
        if len == 0 {
            return Ok(0.0);
        }

        // Create GPU buffer
        let buf_x = Buffer::<f32>::builder()
            .queue(self.queue.clone())
            .len(len)
            .copy_host_slice(x)
            .build()?;

        // For reduction, we use a work-group based approach
        // Each work-group reduces its portion, then we finish on CPU
        let work_group_size = 256;
        let num_groups = (len + work_group_size - 1) / work_group_size;

        let buf_partial = Buffer::<f32>::builder()
            .queue(self.queue.clone())
            .len(num_groups)
            .build()?;

        let kernel = Kernel::builder()
            .program(&self.program)
            .name("sum_reduce_f32")
            .queue(self.queue.clone())
            .global_work_size(num_groups * work_group_size)
            .local_work_size(work_group_size)
            .arg(&buf_x)
            .arg(&buf_partial)
            .arg(len as u32)
            .build()?;

        unsafe {
            kernel.enq()?;
        }

        // Read partial results and sum on CPU
        let mut partial = vec![0.0f32; num_groups];
        buf_partial.read(&mut partial).enq()?;
        self.queue.finish()?;

        Ok(partial.iter().sum())
    }

    fn max_f32(&self, x: &[f32]) -> Result<f32> {
        let len = x.len();
        if len == 0 {
            return Err(GpuError::ExecutionFailed("Empty array".to_string()));
        }

        // Create GPU buffer
        let buf_x = Buffer::<f32>::builder()
            .queue(self.queue.clone())
            .len(len)
            .copy_host_slice(x)
            .build()?;

        let work_group_size = 256;
        let num_groups = (len + work_group_size - 1) / work_group_size;

        let buf_partial = Buffer::<f32>::builder()
            .queue(self.queue.clone())
            .len(num_groups)
            .build()?;

        let kernel = Kernel::builder()
            .program(&self.program)
            .name("max_reduce_f32")
            .queue(self.queue.clone())
            .global_work_size(num_groups * work_group_size)
            .local_work_size(work_group_size)
            .arg(&buf_x)
            .arg(&buf_partial)
            .arg(len as u32)
            .build()?;

        unsafe {
            kernel.enq()?;
        }

        // Read partial results and find max on CPU
        let mut partial = vec![f32::NEG_INFINITY; num_groups];
        buf_partial.read(&mut partial).enq()?;
        self.queue.finish()?;

        Ok(partial.iter().cloned().fold(f32::NEG_INFINITY, f32::max))
    }

    fn min_f32(&self, x: &[f32]) -> Result<f32> {
        let len = x.len();
        if len == 0 {
            return Err(GpuError::ExecutionFailed("Empty array".to_string()));
        }

        // Create GPU buffer
        let buf_x = Buffer::<f32>::builder()
            .queue(self.queue.clone())
            .len(len)
            .copy_host_slice(x)
            .build()?;

        let work_group_size = 256;
        let num_groups = (len + work_group_size - 1) / work_group_size;

        let buf_partial = Buffer::<f32>::builder()
            .queue(self.queue.clone())
            .len(num_groups)
            .build()?;

        let kernel = Kernel::builder()
            .program(&self.program)
            .name("min_reduce_f32")
            .queue(self.queue.clone())
            .global_work_size(num_groups * work_group_size)
            .local_work_size(work_group_size)
            .arg(&buf_x)
            .arg(&buf_partial)
            .arg(len as u32)
            .build()?;

        unsafe {
            kernel.enq()?;
        }

        // Read partial results and find min on CPU
        let mut partial = vec![f32::INFINITY; num_groups];
        buf_partial.read(&mut partial).enq()?;
        self.queue.finish()?;

        Ok(partial.iter().cloned().fold(f32::INFINITY, f32::min))
    }

    fn median_f32(&self, x: &[f32]) -> Result<f32> {
        Ok(crate::simd_ops::median_f32(x))
    }

    fn variance_f32(&self, x: &[f32]) -> Result<f32> {
        Ok(crate::simd_ops::variance_f32(x))
    }

    fn stddev_f32(&self, x: &[f32]) -> Result<f32> {
        Ok(crate::simd_ops::stddev_f32(x))
    }

    fn synchronize(&self) -> Result<()> {
        self.queue.finish()?;
        Ok(())
    }
}
