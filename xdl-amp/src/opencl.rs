//! OpenCL backend for cross-platform GPU acceleration

use crate::backend::{GpuBuffer, GpuDevice};
use crate::error::{GpuError, Result};
use ocl::{Buffer, Context, Device, Platform, Program, Queue};
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

    fn add_f32(&self, _a: &[f32], _b: &[f32], _c: &mut [f32]) -> Result<()> {
        // TODO: Implement using OpenCL kernel
        Err(GpuError::ExecutionFailed(
            "OpenCL add not yet implemented".to_string(),
        ))
    }

    fn mul_f32(&self, _a: &[f32], _b: &[f32], _c: &mut [f32]) -> Result<()> {
        Err(GpuError::ExecutionFailed(
            "OpenCL mul not yet implemented".to_string(),
        ))
    }

    fn sub_f32(&self, _a: &[f32], _b: &[f32], _c: &mut [f32]) -> Result<()> {
        Err(GpuError::ExecutionFailed(
            "OpenCL sub not yet implemented".to_string(),
        ))
    }

    fn div_f32(&self, _a: &[f32], _b: &[f32], _c: &mut [f32]) -> Result<()> {
        Err(GpuError::ExecutionFailed(
            "OpenCL div not yet implemented".to_string(),
        ))
    }

    fn matmul_f32(
        &self,
        _a: &[f32],
        _b: &[f32],
        _c: &mut [f32],
        _m: usize,
        _n: usize,
        _k: usize,
    ) -> Result<()> {
        Err(GpuError::ExecutionFailed(
            "OpenCL matmul not yet implemented".to_string(),
        ))
    }

    fn sin_f32(&self, _x: &[f32], _y: &mut [f32]) -> Result<()> {
        Err(GpuError::ExecutionFailed(
            "OpenCL sin not yet implemented".to_string(),
        ))
    }

    fn cos_f32(&self, _x: &[f32], _y: &mut [f32]) -> Result<()> {
        Err(GpuError::ExecutionFailed(
            "OpenCL cos not yet implemented".to_string(),
        ))
    }

    fn exp_f32(&self, _x: &[f32], _y: &mut [f32]) -> Result<()> {
        Err(GpuError::ExecutionFailed(
            "OpenCL exp not yet implemented".to_string(),
        ))
    }

    fn log_f32(&self, _x: &[f32], _y: &mut [f32]) -> Result<()> {
        Err(GpuError::ExecutionFailed(
            "OpenCL log not yet implemented".to_string(),
        ))
    }

    fn sqrt_f32(&self, _x: &[f32], _y: &mut [f32]) -> Result<()> {
        Err(GpuError::ExecutionFailed(
            "OpenCL sqrt not yet implemented".to_string(),
        ))
    }

    fn pow_f32(&self, _x: &[f32], _p: f32, _y: &mut [f32]) -> Result<()> {
        Err(GpuError::ExecutionFailed(
            "OpenCL pow not yet implemented".to_string(),
        ))
    }

    fn sum_f32(&self, _x: &[f32]) -> Result<f32> {
        Err(GpuError::ExecutionFailed(
            "OpenCL sum not yet implemented".to_string(),
        ))
    }

    fn max_f32(&self, _x: &[f32]) -> Result<f32> {
        Err(GpuError::ExecutionFailed(
            "OpenCL max not yet implemented".to_string(),
        ))
    }

    fn min_f32(&self, _x: &[f32]) -> Result<f32> {
        Err(GpuError::ExecutionFailed(
            "OpenCL min not yet implemented".to_string(),
        ))
    }

    fn synchronize(&self) -> Result<()> {
        self.queue.finish()?;
        Ok(())
    }
}
