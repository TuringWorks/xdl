//! Metal backend for macOS GPU acceleration

use crate::backend::{GpuBuffer, GpuDevice};
use crate::error::{GpuError, Result};
use metal::*;

/// Metal GPU buffer
#[derive(Debug)]
pub struct MetalBuffer {
    buffer: metal::Buffer,
    size: usize,
}

impl GpuBuffer for MetalBuffer {
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

        let contents = self.buffer.contents() as *const u8;
        unsafe {
            std::ptr::copy_nonoverlapping(contents, dst.as_mut_ptr(), self.size);
        }
        Ok(())
    }

    fn write_from_slice(&mut self, src: &[u8]) -> Result<()> {
        if src.len() != self.size {
            return Err(GpuError::BufferSizeMismatch {
                expected: self.size,
                actual: src.len(),
            });
        }

        let contents = self.buffer.contents() as *mut u8;
        unsafe {
            std::ptr::copy_nonoverlapping(src.as_ptr(), contents, self.size);
        }
        Ok(())
    }
}

/// Metal GPU device
#[derive(Debug)]
pub struct MetalDevice {
    device: metal::Device,
    queue: metal::CommandQueue,
    library: metal::Library,
}

impl MetalDevice {
    /// Create a new Metal device
    pub fn new() -> Result<Self> {
        let device = metal::Device::system_default().ok_or(GpuError::DeviceNotFound)?;

        let queue = device.new_command_queue();

        // Compile Metal shaders
        let source = include_str!("shaders/metal_kernels.metal");
        let options = CompileOptions::new();
        let library = device
            .new_library_with_source(source, &options)
            .map_err(|e| GpuError::CompilationFailed(e.to_string()))?;

        Ok(Self {
            device,
            queue,
            library,
        })
    }

    fn execute_kernel(
        &self,
        kernel_name: &str,
        buffers: &[&metal::Buffer],
        grid_size: u64,
    ) -> Result<()> {
        let kernel = self
            .library
            .get_function(kernel_name, None)
            .map_err(|e| GpuError::CompilationFailed(format!("Kernel {}: {}", kernel_name, e)))?;

        let pipeline = self
            .device
            .new_compute_pipeline_state_with_function(&kernel)
            .map_err(|e| GpuError::CompilationFailed(e.to_string()))?;

        let command_buffer = self.queue.new_command_buffer();
        let encoder = command_buffer.new_compute_command_encoder();

        encoder.set_compute_pipeline_state(&pipeline);
        for (i, buffer) in buffers.iter().enumerate() {
            encoder.set_buffer(i as u64, Some(buffer), 0);
        }

        let thread_group_size = MTLSize {
            width: 256.min(grid_size),
            height: 1,
            depth: 1,
        };

        let thread_groups = MTLSize {
            width: grid_size.div_ceil(thread_group_size.width),
            height: 1,
            depth: 1,
        };

        encoder.dispatch_thread_groups(thread_groups, thread_group_size);
        encoder.end_encoding();

        command_buffer.commit();
        command_buffer.wait_until_completed();

        Ok(())
    }
}

impl GpuDevice for MetalDevice {
    fn name(&self) -> &str {
        "Metal"
    }

    fn create_buffer(&self, size: usize) -> Result<Box<dyn GpuBuffer>> {
        let buffer = self
            .device
            .new_buffer(size as u64, MTLResourceOptions::StorageModeShared);

        Ok(Box::new(MetalBuffer { buffer, size }))
    }

    fn create_buffer_with_data(&self, data: &[u8]) -> Result<Box<dyn GpuBuffer>> {
        let buffer = self.device.new_buffer_with_data(
            data.as_ptr() as *const _,
            data.len() as u64,
            MTLResourceOptions::StorageModeShared,
        );

        Ok(Box::new(MetalBuffer {
            buffer,
            size: data.len(),
        }))
    }

    fn add_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()> {
        let len = a.len();
        let buf_a = self.create_buffer_with_data(bytemuck::cast_slice(a))?;
        let buf_b = self.create_buffer_with_data(bytemuck::cast_slice(b))?;
        let buf_c = self.create_buffer(len * 4)?;

        let a_metal = unsafe { &*(buf_a.as_ref() as *const dyn GpuBuffer as *const MetalBuffer) };
        let b_metal = unsafe { &*(buf_b.as_ref() as *const dyn GpuBuffer as *const MetalBuffer) };
        let c_metal = unsafe { &*(buf_c.as_ref() as *const dyn GpuBuffer as *const MetalBuffer) };

        self.execute_kernel(
            "add_f32",
            &[&a_metal.buffer, &b_metal.buffer, &c_metal.buffer],
            len as u64,
        )?;

        buf_c.read_to_slice(bytemuck::cast_slice_mut(c))?;
        Ok(())
    }

    fn mul_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()> {
        let len = a.len();
        let buf_a = self.create_buffer_with_data(bytemuck::cast_slice(a))?;
        let buf_b = self.create_buffer_with_data(bytemuck::cast_slice(b))?;
        let buf_c = self.create_buffer(len * 4)?;

        let a_metal = unsafe { &*(buf_a.as_ref() as *const dyn GpuBuffer as *const MetalBuffer) };
        let b_metal = unsafe { &*(buf_b.as_ref() as *const dyn GpuBuffer as *const MetalBuffer) };
        let c_metal = unsafe { &*(buf_c.as_ref() as *const dyn GpuBuffer as *const MetalBuffer) };

        self.execute_kernel(
            "mul_f32",
            &[&a_metal.buffer, &b_metal.buffer, &c_metal.buffer],
            len as u64,
        )?;

        buf_c.read_to_slice(bytemuck::cast_slice_mut(c))?;
        Ok(())
    }

    fn sub_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()> {
        let len = a.len();
        let buf_a = self.create_buffer_with_data(bytemuck::cast_slice(a))?;
        let buf_b = self.create_buffer_with_data(bytemuck::cast_slice(b))?;
        let buf_c = self.create_buffer(len * 4)?;

        let a_metal = unsafe { &*(buf_a.as_ref() as *const dyn GpuBuffer as *const MetalBuffer) };
        let b_metal = unsafe { &*(buf_b.as_ref() as *const dyn GpuBuffer as *const MetalBuffer) };
        let c_metal = unsafe { &*(buf_c.as_ref() as *const dyn GpuBuffer as *const MetalBuffer) };

        self.execute_kernel(
            "sub_f32",
            &[&a_metal.buffer, &b_metal.buffer, &c_metal.buffer],
            len as u64,
        )?;

        buf_c.read_to_slice(bytemuck::cast_slice_mut(c))?;
        Ok(())
    }

    fn div_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()> {
        let len = a.len();
        let buf_a = self.create_buffer_with_data(bytemuck::cast_slice(a))?;
        let buf_b = self.create_buffer_with_data(bytemuck::cast_slice(b))?;
        let buf_c = self.create_buffer(len * 4)?;

        let a_metal = unsafe { &*(buf_a.as_ref() as *const dyn GpuBuffer as *const MetalBuffer) };
        let b_metal = unsafe { &*(buf_b.as_ref() as *const dyn GpuBuffer as *const MetalBuffer) };
        let c_metal = unsafe { &*(buf_c.as_ref() as *const dyn GpuBuffer as *const MetalBuffer) };

        self.execute_kernel(
            "div_f32",
            &[&a_metal.buffer, &b_metal.buffer, &c_metal.buffer],
            len as u64,
        )?;

        buf_c.read_to_slice(bytemuck::cast_slice_mut(c))?;
        Ok(())
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
        // TODO: Implement matrix multiplication
        Err(GpuError::ExecutionFailed(
            "matmul not yet implemented".to_string(),
        ))
    }

    fn sin_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        let len = x.len();
        let buf_x = self.create_buffer_with_data(bytemuck::cast_slice(x))?;
        let buf_y = self.create_buffer(len * 4)?;

        let x_metal = unsafe { &*(buf_x.as_ref() as *const dyn GpuBuffer as *const MetalBuffer) };
        let y_metal = unsafe { &*(buf_y.as_ref() as *const dyn GpuBuffer as *const MetalBuffer) };

        self.execute_kernel("sin_f32", &[&x_metal.buffer, &y_metal.buffer], len as u64)?;

        buf_y.read_to_slice(bytemuck::cast_slice_mut(y))?;
        Ok(())
    }

    fn cos_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        let len = x.len();
        let buf_x = self.create_buffer_with_data(bytemuck::cast_slice(x))?;
        let buf_y = self.create_buffer(len * 4)?;

        let x_metal = unsafe { &*(buf_x.as_ref() as *const dyn GpuBuffer as *const MetalBuffer) };
        let y_metal = unsafe { &*(buf_y.as_ref() as *const dyn GpuBuffer as *const MetalBuffer) };

        self.execute_kernel("cos_f32", &[&x_metal.buffer, &y_metal.buffer], len as u64)?;

        buf_y.read_to_slice(bytemuck::cast_slice_mut(y))?;
        Ok(())
    }

    fn exp_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        let len = x.len();
        let buf_x = self.create_buffer_with_data(bytemuck::cast_slice(x))?;
        let buf_y = self.create_buffer(len * 4)?;

        let x_metal = unsafe { &*(buf_x.as_ref() as *const dyn GpuBuffer as *const MetalBuffer) };
        let y_metal = unsafe { &*(buf_y.as_ref() as *const dyn GpuBuffer as *const MetalBuffer) };

        self.execute_kernel("exp_f32", &[&x_metal.buffer, &y_metal.buffer], len as u64)?;

        buf_y.read_to_slice(bytemuck::cast_slice_mut(y))?;
        Ok(())
    }

    fn log_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        let len = x.len();
        let buf_x = self.create_buffer_with_data(bytemuck::cast_slice(x))?;
        let buf_y = self.create_buffer(len * 4)?;

        let x_metal = unsafe { &*(buf_x.as_ref() as *const dyn GpuBuffer as *const MetalBuffer) };
        let y_metal = unsafe { &*(buf_y.as_ref() as *const dyn GpuBuffer as *const MetalBuffer) };

        self.execute_kernel("log_f32", &[&x_metal.buffer, &y_metal.buffer], len as u64)?;

        buf_y.read_to_slice(bytemuck::cast_slice_mut(y))?;
        Ok(())
    }

    fn sqrt_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        let len = x.len();
        let buf_x = self.create_buffer_with_data(bytemuck::cast_slice(x))?;
        let buf_y = self.create_buffer(len * 4)?;

        let x_metal = unsafe { &*(buf_x.as_ref() as *const dyn GpuBuffer as *const MetalBuffer) };
        let y_metal = unsafe { &*(buf_y.as_ref() as *const dyn GpuBuffer as *const MetalBuffer) };

        self.execute_kernel("sqrt_f32", &[&x_metal.buffer, &y_metal.buffer], len as u64)?;

        buf_y.read_to_slice(bytemuck::cast_slice_mut(y))?;
        Ok(())
    }

    fn pow_f32(&self, _x: &[f32], _p: f32, _y: &mut [f32]) -> Result<()> {
        Err(GpuError::ExecutionFailed(
            "pow not yet implemented".to_string(),
        ))
    }

    fn sum_f32(&self, _x: &[f32]) -> Result<f32> {
        Err(GpuError::ExecutionFailed(
            "sum not yet implemented".to_string(),
        ))
    }

    fn max_f32(&self, _x: &[f32]) -> Result<f32> {
        Err(GpuError::ExecutionFailed(
            "max not yet implemented".to_string(),
        ))
    }

    fn min_f32(&self, _x: &[f32]) -> Result<f32> {
        Err(GpuError::ExecutionFailed(
            "min not yet implemented".to_string(),
        ))
    }

    fn synchronize(&self) -> Result<()> {
        Ok(())
    }
}
