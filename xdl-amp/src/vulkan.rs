//! Vulkan backend for cross-platform GPU acceleration

use crate::backend::{GpuBuffer, GpuDevice};
use crate::error::{GpuError, Result};
use ash::vk;
use std::ffi::CStr;
use std::sync::Arc;

// Include compiled SPIR-V shaders
#[cfg(feature = "vulkan")]
include!(concat!(env!("OUT_DIR"), "/shaders.rs"));

/// Vulkan GPU buffer with device memory
#[derive(Debug)]
pub struct VulkanBuffer {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    size: usize,
    device: Arc<VulkanContext>,
}

impl Drop for VulkanBuffer {
    fn drop(&mut self) {
        unsafe {
            self.device.device.destroy_buffer(self.buffer, None);
            self.device.device.free_memory(self.memory, None);
        }
    }
}

impl GpuBuffer for VulkanBuffer {
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

        unsafe {
            let data_ptr = self
                .device
                .device
                .map_memory(
                    self.memory,
                    0,
                    self.size as u64,
                    vk::MemoryMapFlags::empty(),
                )
                .map_err(|e| GpuError::ExecutionFailed(format!("Failed to map memory: {:?}", e)))?;

            std::ptr::copy_nonoverlapping(data_ptr as *const u8, dst.as_mut_ptr(), self.size);

            self.device.device.unmap_memory(self.memory);
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

        unsafe {
            let data_ptr = self
                .device
                .device
                .map_memory(
                    self.memory,
                    0,
                    self.size as u64,
                    vk::MemoryMapFlags::empty(),
                )
                .map_err(|e| GpuError::ExecutionFailed(format!("Failed to map memory: {:?}", e)))?;

            std::ptr::copy_nonoverlapping(src.as_ptr(), data_ptr as *mut u8, self.size);

            self.device.device.unmap_memory(self.memory);
        }

        Ok(())
    }
}

/// Vulkan compute pipeline for a specific operation
struct ComputePipeline {
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_set_layout: vk::DescriptorSetLayout,
}

impl ComputePipeline {
    unsafe fn cleanup(&self, device: &ash::Device) {
        device.destroy_pipeline(self.pipeline, None);
        device.destroy_pipeline_layout(self.pipeline_layout, None);
        device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
    }
}

/// Vulkan instance and device context
struct VulkanContext {
    _entry: ash::Entry,
    instance: ash::Instance,
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    compute_queue: vk::Queue,
    compute_queue_family: u32,
    command_pool: vk::CommandPool,
    descriptor_pool: vk::DescriptorPool,
    pipelines: VulkanPipelines,
}

/// All compute pipelines
struct VulkanPipelines {
    add: ComputePipeline,
    mul: ComputePipeline,
    sub: ComputePipeline,
    div: ComputePipeline,
    sin: ComputePipeline,
    cos: ComputePipeline,
    exp: ComputePipeline,
    log: ComputePipeline,
    sqrt: ComputePipeline,
    pow: ComputePipeline,
    matmul: ComputePipeline,
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().ok();

            self.pipelines.add.cleanup(&self.device);
            self.pipelines.mul.cleanup(&self.device);
            self.pipelines.sub.cleanup(&self.device);
            self.pipelines.div.cleanup(&self.device);
            self.pipelines.sin.cleanup(&self.device);
            self.pipelines.cos.cleanup(&self.device);
            self.pipelines.exp.cleanup(&self.device);
            self.pipelines.log.cleanup(&self.device);
            self.pipelines.sqrt.cleanup(&self.device);
            self.pipelines.pow.cleanup(&self.device);
            self.pipelines.matmul.cleanup(&self.device);

            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);
            self.device.destroy_command_pool(self.command_pool, None);
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

impl std::fmt::Debug for VulkanContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VulkanContext")
            .field("compute_queue_family", &self.compute_queue_family)
            .finish()
    }
}

impl VulkanContext {
    #[cfg(feature = "vulkan")]
    fn new() -> Result<Self> {
        unsafe {
            // Create Vulkan entry
            let entry = ash::Entry::load().map_err(|e| {
                GpuError::ExecutionFailed(format!("Failed to load Vulkan entry: {:?}", e))
            })?;

            // Create instance
            let app_info = vk::ApplicationInfo::default()
                .application_name(CStr::from_bytes_with_nul_unchecked(b"XDL-AMP\0"))
                .application_version(vk::make_api_version(0, 1, 0, 0))
                .engine_name(CStr::from_bytes_with_nul_unchecked(b"XDL-AMP\0"))
                .engine_version(vk::make_api_version(0, 1, 0, 0))
                .api_version(vk::make_api_version(0, 1, 2, 0));

            let create_info = vk::InstanceCreateInfo::default().application_info(&app_info);

            let instance = entry.create_instance(&create_info, None).map_err(|e| {
                GpuError::ExecutionFailed(format!("Failed to create Vulkan instance: {:?}", e))
            })?;

            // Select physical device
            let physical_devices = instance.enumerate_physical_devices().map_err(|e| {
                GpuError::ExecutionFailed(format!("Failed to enumerate devices: {:?}", e))
            })?;

            let (physical_device, compute_queue_family) = physical_devices
                .iter()
                .find_map(|&pdevice| {
                    let queue_families =
                        instance.get_physical_device_queue_family_properties(pdevice);
                    queue_families
                        .iter()
                        .enumerate()
                        .find(|(_, props)| props.queue_flags.contains(vk::QueueFlags::COMPUTE))
                        .map(|(index, _)| (pdevice, index as u32))
                })
                .ok_or_else(|| {
                    GpuError::ExecutionFailed("No suitable Vulkan device found".to_string())
                })?;

            // Create logical device
            let queue_priorities = [1.0];
            let queue_create_info = vk::DeviceQueueCreateInfo::default()
                .queue_family_index(compute_queue_family)
                .queue_priorities(&queue_priorities);

            let device_create_info = vk::DeviceCreateInfo::default()
                .queue_create_infos(std::slice::from_ref(&queue_create_info));

            let device = instance
                .create_device(physical_device, &device_create_info, None)
                .map_err(|e| {
                    GpuError::ExecutionFailed(format!("Failed to create device: {:?}", e))
                })?;

            let compute_queue = device.get_device_queue(compute_queue_family, 0);

            // Create command pool
            let command_pool_info = vk::CommandPoolCreateInfo::default()
                .queue_family_index(compute_queue_family)
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

            let command_pool = device
                .create_command_pool(&command_pool_info, None)
                .map_err(|e| {
                    GpuError::ExecutionFailed(format!("Failed to create command pool: {:?}", e))
                })?;

            // Create descriptor pool
            let pool_sizes = [vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1000,
            }];

            let descriptor_pool_info = vk::DescriptorPoolCreateInfo::default()
                .max_sets(100)
                .pool_sizes(&pool_sizes);

            let descriptor_pool = device
                .create_descriptor_pool(&descriptor_pool_info, None)
                .map_err(|e| {
                    GpuError::ExecutionFailed(format!("Failed to create descriptor pool: {:?}", e))
                })?;

            // Create pipelines
            let pipelines = VulkanPipelines {
                add: Self::create_binary_pipeline(&device, ADD_SPIRV)?,
                mul: Self::create_binary_pipeline(&device, MUL_SPIRV)?,
                sub: Self::create_binary_pipeline(&device, SUB_SPIRV)?,
                div: Self::create_binary_pipeline(&device, DIV_SPIRV)?,
                sin: Self::create_unary_pipeline(&device, SIN_SPIRV)?,
                cos: Self::create_unary_pipeline(&device, COS_SPIRV)?,
                exp: Self::create_unary_pipeline(&device, EXP_SPIRV)?,
                log: Self::create_unary_pipeline(&device, LOG_SPIRV)?,
                sqrt: Self::create_unary_pipeline(&device, SQRT_SPIRV)?,
                pow: Self::create_pow_pipeline(&device, POW_SPIRV)?,
                matmul: Self::create_matmul_pipeline(&device, MATMUL_SPIRV)?,
            };

            Ok(Self {
                _entry: entry,
                instance,
                physical_device,
                device,
                compute_queue,
                compute_queue_family,
                command_pool,
                descriptor_pool,
                pipelines,
            })
        }
    }

    #[cfg(feature = "vulkan")]
    unsafe fn create_binary_pipeline(
        device: &ash::Device,
        spirv: &[u8],
    ) -> Result<ComputePipeline> {
        // Create descriptor set layout (3 storage buffers)
        let bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(2)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
        ];

        let descriptor_set_layout_info =
            vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
        let descriptor_set_layout = device
            .create_descriptor_set_layout(&descriptor_set_layout_info, None)
            .map_err(|e| {
                GpuError::ExecutionFailed(format!(
                    "Failed to create descriptor set layout: {:?}",
                    e
                ))
            })?;

        // Create pipeline layout
        let layouts = [descriptor_set_layout];
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts);
        let pipeline_layout = device
            .create_pipeline_layout(&pipeline_layout_info, None)
            .map_err(|e| {
                GpuError::ExecutionFailed(format!("Failed to create pipeline layout: {:?}", e))
            })?;

        // Create shader module
        let shader_module_info = vk::ShaderModuleCreateInfo::default().code(
            std::slice::from_raw_parts(spirv.as_ptr() as *const u32, spirv.len() / 4),
        );
        let shader_module = device
            .create_shader_module(&shader_module_info, None)
            .map_err(|e| {
                GpuError::ExecutionFailed(format!("Failed to create shader module: {:?}", e))
            })?;

        // Create compute pipeline
        let entry_name = CStr::from_bytes_with_nul_unchecked(b"main\0");
        let stage_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(shader_module)
            .name(entry_name);

        let pipeline_info = vk::ComputePipelineCreateInfo::default()
            .stage(stage_info)
            .layout(pipeline_layout);

        let pipeline = device
            .create_compute_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
            .map_err(|e| {
                GpuError::ExecutionFailed(format!("Failed to create compute pipeline: {:?}", e.1))
            })?[0];

        device.destroy_shader_module(shader_module, None);

        Ok(ComputePipeline {
            pipeline,
            pipeline_layout,
            descriptor_set_layout,
        })
    }

    #[cfg(feature = "vulkan")]
    unsafe fn create_unary_pipeline(device: &ash::Device, spirv: &[u8]) -> Result<ComputePipeline> {
        // Create descriptor set layout (2 storage buffers)
        let bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
        ];

        let descriptor_set_layout_info =
            vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
        let descriptor_set_layout = device
            .create_descriptor_set_layout(&descriptor_set_layout_info, None)
            .map_err(|e| {
                GpuError::ExecutionFailed(format!(
                    "Failed to create descriptor set layout: {:?}",
                    e
                ))
            })?;

        // Create pipeline layout
        let layouts = [descriptor_set_layout];
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts);
        let pipeline_layout = device
            .create_pipeline_layout(&pipeline_layout_info, None)
            .map_err(|e| {
                GpuError::ExecutionFailed(format!("Failed to create pipeline layout: {:?}", e))
            })?;

        // Create shader module
        let shader_module_info = vk::ShaderModuleCreateInfo::default().code(
            std::slice::from_raw_parts(spirv.as_ptr() as *const u32, spirv.len() / 4),
        );
        let shader_module = device
            .create_shader_module(&shader_module_info, None)
            .map_err(|e| {
                GpuError::ExecutionFailed(format!("Failed to create shader module: {:?}", e))
            })?;

        // Create compute pipeline
        let entry_name = CStr::from_bytes_with_nul_unchecked(b"main\0");
        let stage_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(shader_module)
            .name(entry_name);

        let pipeline_info = vk::ComputePipelineCreateInfo::default()
            .stage(stage_info)
            .layout(pipeline_layout);

        let pipeline = device
            .create_compute_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
            .map_err(|e| {
                GpuError::ExecutionFailed(format!("Failed to create compute pipeline: {:?}", e.1))
            })?[0];

        device.destroy_shader_module(shader_module, None);

        Ok(ComputePipeline {
            pipeline,
            pipeline_layout,
            descriptor_set_layout,
        })
    }

    #[cfg(feature = "vulkan")]
    unsafe fn create_pow_pipeline(device: &ash::Device, spirv: &[u8]) -> Result<ComputePipeline> {
        // Create descriptor set layout (2 storage buffers)
        let bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
        ];

        let descriptor_set_layout_info =
            vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
        let descriptor_set_layout = device
            .create_descriptor_set_layout(&descriptor_set_layout_info, None)
            .map_err(|e| {
                GpuError::ExecutionFailed(format!(
                    "Failed to create descriptor set layout: {:?}",
                    e
                ))
            })?;

        // Create pipeline layout with push constant
        let push_constant_range = vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::COMPUTE)
            .offset(0)
            .size(4); // sizeof(f32)

        let layouts = [descriptor_set_layout];
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&layouts)
            .push_constant_ranges(std::slice::from_ref(&push_constant_range));
        let pipeline_layout = device
            .create_pipeline_layout(&pipeline_layout_info, None)
            .map_err(|e| {
                GpuError::ExecutionFailed(format!("Failed to create pipeline layout: {:?}", e))
            })?;

        // Create shader module
        let shader_module_info = vk::ShaderModuleCreateInfo::default().code(
            std::slice::from_raw_parts(spirv.as_ptr() as *const u32, spirv.len() / 4),
        );
        let shader_module = device
            .create_shader_module(&shader_module_info, None)
            .map_err(|e| {
                GpuError::ExecutionFailed(format!("Failed to create shader module: {:?}", e))
            })?;

        // Create compute pipeline
        let entry_name = CStr::from_bytes_with_nul_unchecked(b"main\0");
        let stage_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(shader_module)
            .name(entry_name);

        let pipeline_info = vk::ComputePipelineCreateInfo::default()
            .stage(stage_info)
            .layout(pipeline_layout);

        let pipeline = device
            .create_compute_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
            .map_err(|e| {
                GpuError::ExecutionFailed(format!("Failed to create compute pipeline: {:?}", e.1))
            })?[0];

        device.destroy_shader_module(shader_module, None);

        Ok(ComputePipeline {
            pipeline,
            pipeline_layout,
            descriptor_set_layout,
        })
    }

    #[cfg(feature = "vulkan")]
    unsafe fn create_matmul_pipeline(
        device: &ash::Device,
        spirv: &[u8],
    ) -> Result<ComputePipeline> {
        // Create descriptor set layout (3 storage buffers)
        let bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()
                .binding(2)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
        ];

        let descriptor_set_layout_info =
            vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
        let descriptor_set_layout = device
            .create_descriptor_set_layout(&descriptor_set_layout_info, None)
            .map_err(|e| {
                GpuError::ExecutionFailed(format!(
                    "Failed to create descriptor set layout: {:?}",
                    e
                ))
            })?;

        // Create pipeline layout with push constants for M, N, K
        let push_constant_range = vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::COMPUTE)
            .offset(0)
            .size(12); // 3 * sizeof(u32)

        let layouts = [descriptor_set_layout];
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&layouts)
            .push_constant_ranges(std::slice::from_ref(&push_constant_range));
        let pipeline_layout = device
            .create_pipeline_layout(&pipeline_layout_info, None)
            .map_err(|e| {
                GpuError::ExecutionFailed(format!("Failed to create pipeline layout: {:?}", e))
            })?;

        // Create shader module
        let shader_module_info = vk::ShaderModuleCreateInfo::default().code(
            std::slice::from_raw_parts(spirv.as_ptr() as *const u32, spirv.len() / 4),
        );
        let shader_module = device
            .create_shader_module(&shader_module_info, None)
            .map_err(|e| {
                GpuError::ExecutionFailed(format!("Failed to create shader module: {:?}", e))
            })?;

        // Create compute pipeline
        let entry_name = CStr::from_bytes_with_nul_unchecked(b"main\0");
        let stage_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(shader_module)
            .name(entry_name);

        let pipeline_info = vk::ComputePipelineCreateInfo::default()
            .stage(stage_info)
            .layout(pipeline_layout);

        let pipeline = device
            .create_compute_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
            .map_err(|e| {
                GpuError::ExecutionFailed(format!("Failed to create compute pipeline: {:?}", e.1))
            })?[0];

        device.destroy_shader_module(shader_module, None);

        Ok(ComputePipeline {
            pipeline,
            pipeline_layout,
            descriptor_set_layout,
        })
    }

    #[cfg(feature = "vulkan")]
    fn create_buffer(
        context: &Arc<VulkanContext>,
        size: usize,
        usage: vk::BufferUsageFlags,
    ) -> Result<VulkanBuffer> {
        unsafe {
            let buffer_info = vk::BufferCreateInfo::default()
                .size(size as u64)
                .usage(usage)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let buffer = context
                .device
                .create_buffer(&buffer_info, None)
                .map_err(|e| {
                    GpuError::ExecutionFailed(format!("Failed to create buffer: {:?}", e))
                })?;

            let mem_requirements = context.device.get_buffer_memory_requirements(buffer);

            let memory_type_index = context
                .find_memory_type(
                    mem_requirements.memory_type_bits,
                    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                )
                .ok_or_else(|| {
                    GpuError::ExecutionFailed("Failed to find suitable memory type".to_string())
                })?;

            let alloc_info = vk::MemoryAllocateInfo::default()
                .allocation_size(mem_requirements.size)
                .memory_type_index(memory_type_index);

            let memory = context
                .device
                .allocate_memory(&alloc_info, None)
                .map_err(|e| {
                    GpuError::ExecutionFailed(format!("Failed to allocate memory: {:?}", e))
                })?;

            context
                .device
                .bind_buffer_memory(buffer, memory, 0)
                .map_err(|e| {
                    GpuError::ExecutionFailed(format!("Failed to bind buffer memory: {:?}", e))
                })?;

            Ok(VulkanBuffer {
                buffer,
                memory,
                size,
                device: Arc::clone(context),
            })
        }
    }

    fn find_memory_type(
        &self,
        type_filter: u32,
        properties: vk::MemoryPropertyFlags,
    ) -> Option<u32> {
        let mem_properties = unsafe {
            self.instance
                .get_physical_device_memory_properties(self.physical_device)
        };

        for i in 0..mem_properties.memory_type_count {
            if (type_filter & (1 << i)) != 0
                && mem_properties.memory_types[i as usize]
                    .property_flags
                    .contains(properties)
            {
                return Some(i);
            }
        }

        None
    }
}

/// Vulkan GPU device
pub struct VulkanDevice {
    context: Arc<VulkanContext>,
}

impl std::fmt::Debug for VulkanDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VulkanDevice").finish()
    }
}

impl VulkanDevice {
    /// Create a new Vulkan device
    pub fn new() -> Result<Self> {
        #[cfg(feature = "vulkan")]
        {
            let context = VulkanContext::new()?;
            Ok(Self {
                context: Arc::new(context),
            })
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(GpuError::UnsupportedBackend(
                "Vulkan not enabled".to_string(),
            ))
        }
    }

    /// Check if Vulkan is available
    pub fn is_available() -> bool {
        cfg!(feature = "vulkan")
    }

    #[cfg(feature = "vulkan")]
    fn dispatch_binary_op(
        &self,
        pipeline: &ComputePipeline,
        a: &[f32],
        b: &[f32],
        c: &mut [f32],
    ) -> Result<()> {
        let size = a.len();
        let buffer_size = size * std::mem::size_of::<f32>();

        // Create buffers
        let mut buf_a = VulkanContext::create_buffer(
            &self.context,
            buffer_size,
            vk::BufferUsageFlags::STORAGE_BUFFER,
        )?;
        let mut buf_b = VulkanContext::create_buffer(
            &self.context,
            buffer_size,
            vk::BufferUsageFlags::STORAGE_BUFFER,
        )?;
        let buf_c = VulkanContext::create_buffer(
            &self.context,
            buffer_size,
            vk::BufferUsageFlags::STORAGE_BUFFER,
        )?;

        // Upload data
        buf_a.write_from_slice(bytemuck::cast_slice(a))?;
        buf_b.write_from_slice(bytemuck::cast_slice(b))?;

        // Execute compute shader
        unsafe {
            self.execute_compute(pipeline, &[&buf_a, &buf_b, &buf_c], size, None)?;
        }

        // Read back results
        let mut result_bytes = vec![0u8; buffer_size];
        buf_c.read_to_slice(&mut result_bytes)?;
        c.copy_from_slice(bytemuck::cast_slice(&result_bytes));

        Ok(())
    }

    #[cfg(feature = "vulkan")]
    fn dispatch_unary_op(
        &self,
        pipeline: &ComputePipeline,
        x: &[f32],
        y: &mut [f32],
    ) -> Result<()> {
        let size = x.len();
        let buffer_size = size * std::mem::size_of::<f32>();

        // Create buffers
        let mut buf_x = VulkanContext::create_buffer(
            &self.context,
            buffer_size,
            vk::BufferUsageFlags::STORAGE_BUFFER,
        )?;
        let buf_y = VulkanContext::create_buffer(
            &self.context,
            buffer_size,
            vk::BufferUsageFlags::STORAGE_BUFFER,
        )?;

        // Upload data
        buf_x.write_from_slice(bytemuck::cast_slice(x))?;

        // Execute compute shader
        unsafe {
            self.execute_compute(pipeline, &[&buf_x, &buf_y], size, None)?;
        }

        // Read back results
        let mut result_bytes = vec![0u8; buffer_size];
        buf_y.read_to_slice(&mut result_bytes)?;
        y.copy_from_slice(bytemuck::cast_slice(&result_bytes));

        Ok(())
    }

    #[cfg(feature = "vulkan")]
    unsafe fn execute_compute(
        &self,
        pipeline: &ComputePipeline,
        buffers: &[&VulkanBuffer],
        work_size: usize,
        push_constant: Option<&[u8]>,
    ) -> Result<()> {
        let device = &self.context.device;

        // Allocate descriptor set
        let layouts = [pipeline.descriptor_set_layout];
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.context.descriptor_pool)
            .set_layouts(&layouts);

        let descriptor_sets = device.allocate_descriptor_sets(&alloc_info).map_err(|e| {
            GpuError::ExecutionFailed(format!("Failed to allocate descriptor sets: {:?}", e))
        })?;
        let descriptor_set = descriptor_sets[0];

        // Update descriptor set
        let mut buffer_infos = Vec::new();

        for (_i, buffer) in buffers.iter().enumerate() {
            buffer_infos.push(
                vk::DescriptorBufferInfo::default()
                    .buffer(buffer.buffer)
                    .offset(0)
                    .range(vk::WHOLE_SIZE),
            );
        }

        let mut write_descriptor_sets = Vec::new();
        for (i, buffer_info) in buffer_infos.iter().enumerate() {
            write_descriptor_sets.push(
                vk::WriteDescriptorSet::default()
                    .dst_set(descriptor_set)
                    .dst_binding(i as u32)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .buffer_info(std::slice::from_ref(buffer_info)),
            );
        }

        device.update_descriptor_sets(&write_descriptor_sets, &[]);

        // Allocate command buffer
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.context.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffers = device.allocate_command_buffers(&alloc_info).map_err(|e| {
            GpuError::ExecutionFailed(format!("Failed to allocate command buffer: {:?}", e))
        })?;
        let command_buffer = command_buffers[0];

        // Record command buffer
        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        device
            .begin_command_buffer(command_buffer, &begin_info)
            .map_err(|e| {
                GpuError::ExecutionFailed(format!("Failed to begin command buffer: {:?}", e))
            })?;

        device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            pipeline.pipeline,
        );

        device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            pipeline.pipeline_layout,
            0,
            &[descriptor_set],
            &[],
        );

        if let Some(data) = push_constant {
            device.cmd_push_constants(
                command_buffer,
                pipeline.pipeline_layout,
                vk::ShaderStageFlags::COMPUTE,
                0,
                data,
            );
        }

        let group_count_x = ((work_size + 255) / 256) as u32;
        device.cmd_dispatch(command_buffer, group_count_x, 1, 1);

        device.end_command_buffer(command_buffer).map_err(|e| {
            GpuError::ExecutionFailed(format!("Failed to end command buffer: {:?}", e))
        })?;

        // Submit and wait
        let command_buffers = [command_buffer];
        let submit_info = vk::SubmitInfo::default().command_buffers(&command_buffers);

        let fence_info = vk::FenceCreateInfo::default();
        let fence = device
            .create_fence(&fence_info, None)
            .map_err(|e| GpuError::ExecutionFailed(format!("Failed to create fence: {:?}", e)))?;

        device
            .queue_submit(self.context.compute_queue, &[submit_info], fence)
            .map_err(|e| GpuError::ExecutionFailed(format!("Failed to submit queue: {:?}", e)))?;

        device
            .wait_for_fences(&[fence], true, u64::MAX)
            .map_err(|e| GpuError::ExecutionFailed(format!("Failed to wait for fence: {:?}", e)))?;

        device.destroy_fence(fence, None);
        device.free_command_buffers(self.context.command_pool, &[command_buffer]);
        device
            .free_descriptor_sets(self.context.descriptor_pool, &[descriptor_set])
            .ok();

        Ok(())
    }
}

impl GpuDevice for VulkanDevice {
    fn name(&self) -> &str {
        "Vulkan"
    }

    fn create_buffer(&self, size: usize) -> Result<Box<dyn GpuBuffer>> {
        #[cfg(feature = "vulkan")]
        {
            let buffer = VulkanContext::create_buffer(
                &self.context,
                size,
                vk::BufferUsageFlags::STORAGE_BUFFER,
            )?;
            Ok(Box::new(buffer))
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(GpuError::UnsupportedBackend(
                "Vulkan not enabled".to_string(),
            ))
        }
    }

    fn create_buffer_with_data(&self, data: &[u8]) -> Result<Box<dyn GpuBuffer>> {
        #[cfg(feature = "vulkan")]
        {
            let mut buffer = VulkanContext::create_buffer(
                &self.context,
                data.len(),
                vk::BufferUsageFlags::STORAGE_BUFFER,
            )?;
            buffer.write_from_slice(data)?;
            Ok(Box::new(buffer))
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(GpuError::UnsupportedBackend(
                "Vulkan not enabled".to_string(),
            ))
        }
    }

    fn add_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()> {
        #[cfg(feature = "vulkan")]
        {
            self.dispatch_binary_op(&self.context.pipelines.add, a, b, c)
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(GpuError::UnsupportedBackend(
                "Vulkan not enabled".to_string(),
            ))
        }
    }

    fn mul_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()> {
        #[cfg(feature = "vulkan")]
        {
            self.dispatch_binary_op(&self.context.pipelines.mul, a, b, c)
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(GpuError::UnsupportedBackend(
                "Vulkan not enabled".to_string(),
            ))
        }
    }

    fn sub_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()> {
        #[cfg(feature = "vulkan")]
        {
            self.dispatch_binary_op(&self.context.pipelines.sub, a, b, c)
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(GpuError::UnsupportedBackend(
                "Vulkan not enabled".to_string(),
            ))
        }
    }

    fn div_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()> {
        #[cfg(feature = "vulkan")]
        {
            self.dispatch_binary_op(&self.context.pipelines.div, a, b, c)
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(GpuError::UnsupportedBackend(
                "Vulkan not enabled".to_string(),
            ))
        }
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
        #[cfg(feature = "vulkan")]
        {
            let size_a = m * k * std::mem::size_of::<f32>();
            let size_b = k * n * std::mem::size_of::<f32>();
            let size_c = m * n * std::mem::size_of::<f32>();

            // Create buffers
            let mut buf_a = VulkanContext::create_buffer(
                &self.context,
                size_a,
                vk::BufferUsageFlags::STORAGE_BUFFER,
            )?;
            let mut buf_b = VulkanContext::create_buffer(
                &self.context,
                size_b,
                vk::BufferUsageFlags::STORAGE_BUFFER,
            )?;
            let buf_c = VulkanContext::create_buffer(
                &self.context,
                size_c,
                vk::BufferUsageFlags::STORAGE_BUFFER,
            )?;

            // Upload data
            buf_a.write_from_slice(bytemuck::cast_slice(a))?;
            buf_b.write_from_slice(bytemuck::cast_slice(b))?;

            // Execute compute shader with 2D dispatch
            unsafe {
                let device = &self.context.device;
                let pipeline = &self.context.pipelines.matmul;

                // Allocate descriptor set
                let layouts = [pipeline.descriptor_set_layout];
                let alloc_info = vk::DescriptorSetAllocateInfo::default()
                    .descriptor_pool(self.context.descriptor_pool)
                    .set_layouts(&layouts);

                let descriptor_sets =
                    device.allocate_descriptor_sets(&alloc_info).map_err(|e| {
                        GpuError::ExecutionFailed(format!(
                            "Failed to allocate descriptor sets: {:?}",
                            e
                        ))
                    })?;
                let descriptor_set = descriptor_sets[0];

                // Update descriptor set
                let buffer_infos = [
                    vk::DescriptorBufferInfo::default()
                        .buffer(buf_a.buffer)
                        .offset(0)
                        .range(vk::WHOLE_SIZE),
                    vk::DescriptorBufferInfo::default()
                        .buffer(buf_b.buffer)
                        .offset(0)
                        .range(vk::WHOLE_SIZE),
                    vk::DescriptorBufferInfo::default()
                        .buffer(buf_c.buffer)
                        .offset(0)
                        .range(vk::WHOLE_SIZE),
                ];

                let write_descriptor_sets = [
                    vk::WriteDescriptorSet::default()
                        .dst_set(descriptor_set)
                        .dst_binding(0)
                        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                        .buffer_info(std::slice::from_ref(&buffer_infos[0])),
                    vk::WriteDescriptorSet::default()
                        .dst_set(descriptor_set)
                        .dst_binding(1)
                        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                        .buffer_info(std::slice::from_ref(&buffer_infos[1])),
                    vk::WriteDescriptorSet::default()
                        .dst_set(descriptor_set)
                        .dst_binding(2)
                        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                        .buffer_info(std::slice::from_ref(&buffer_infos[2])),
                ];

                device.update_descriptor_sets(&write_descriptor_sets, &[]);

                // Allocate command buffer
                let alloc_info = vk::CommandBufferAllocateInfo::default()
                    .command_pool(self.context.command_pool)
                    .level(vk::CommandBufferLevel::PRIMARY)
                    .command_buffer_count(1);

                let command_buffers =
                    device.allocate_command_buffers(&alloc_info).map_err(|e| {
                        GpuError::ExecutionFailed(format!(
                            "Failed to allocate command buffer: {:?}",
                            e
                        ))
                    })?;
                let command_buffer = command_buffers[0];

                // Record command buffer
                let begin_info = vk::CommandBufferBeginInfo::default()
                    .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

                device
                    .begin_command_buffer(command_buffer, &begin_info)
                    .map_err(|e| {
                        GpuError::ExecutionFailed(format!(
                            "Failed to begin command buffer: {:?}",
                            e
                        ))
                    })?;

                device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::COMPUTE,
                    pipeline.pipeline,
                );

                device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::COMPUTE,
                    pipeline.pipeline_layout,
                    0,
                    &[descriptor_set],
                    &[],
                );

                // Push constants for M, N, K
                let dims = [m as u32, n as u32, k as u32];
                device.cmd_push_constants(
                    command_buffer,
                    pipeline.pipeline_layout,
                    vk::ShaderStageFlags::COMPUTE,
                    0,
                    bytemuck::cast_slice(&dims),
                );

                // Dispatch with 2D workgroups (16x16 threads each)
                let group_count_x = ((n + 15) / 16) as u32;
                let group_count_y = ((m + 15) / 16) as u32;
                device.cmd_dispatch(command_buffer, group_count_x, group_count_y, 1);

                device.end_command_buffer(command_buffer).map_err(|e| {
                    GpuError::ExecutionFailed(format!("Failed to end command buffer: {:?}", e))
                })?;

                // Submit and wait
                let command_buffers_array = [command_buffer];
                let submit_info = vk::SubmitInfo::default().command_buffers(&command_buffers_array);

                let fence_info = vk::FenceCreateInfo::default();
                let fence = device.create_fence(&fence_info, None).map_err(|e| {
                    GpuError::ExecutionFailed(format!("Failed to create fence: {:?}", e))
                })?;

                device
                    .queue_submit(self.context.compute_queue, &[submit_info], fence)
                    .map_err(|e| {
                        GpuError::ExecutionFailed(format!("Failed to submit queue: {:?}", e))
                    })?;

                device
                    .wait_for_fences(&[fence], true, u64::MAX)
                    .map_err(|e| {
                        GpuError::ExecutionFailed(format!("Failed to wait for fence: {:?}", e))
                    })?;

                device.destroy_fence(fence, None);
                device.free_command_buffers(self.context.command_pool, &[command_buffer]);
                device
                    .free_descriptor_sets(self.context.descriptor_pool, &[descriptor_set])
                    .ok();
            }

            // Read back results
            let mut result_bytes = vec![0u8; size_c];
            buf_c.read_to_slice(&mut result_bytes)?;
            c.copy_from_slice(bytemuck::cast_slice(&result_bytes));

            Ok(())
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(GpuError::UnsupportedBackend(
                "Vulkan not enabled".to_string(),
            ))
        }
    }

    fn sin_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        #[cfg(feature = "vulkan")]
        {
            self.dispatch_unary_op(&self.context.pipelines.sin, x, y)
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(GpuError::UnsupportedBackend(
                "Vulkan not enabled".to_string(),
            ))
        }
    }

    fn cos_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        #[cfg(feature = "vulkan")]
        {
            self.dispatch_unary_op(&self.context.pipelines.cos, x, y)
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(GpuError::UnsupportedBackend(
                "Vulkan not enabled".to_string(),
            ))
        }
    }

    fn exp_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        #[cfg(feature = "vulkan")]
        {
            self.dispatch_unary_op(&self.context.pipelines.exp, x, y)
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(GpuError::UnsupportedBackend(
                "Vulkan not enabled".to_string(),
            ))
        }
    }

    fn log_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        #[cfg(feature = "vulkan")]
        {
            self.dispatch_unary_op(&self.context.pipelines.log, x, y)
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(GpuError::UnsupportedBackend(
                "Vulkan not enabled".to_string(),
            ))
        }
    }

    fn sqrt_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        #[cfg(feature = "vulkan")]
        {
            self.dispatch_unary_op(&self.context.pipelines.sqrt, x, y)
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(GpuError::UnsupportedBackend(
                "Vulkan not enabled".to_string(),
            ))
        }
    }

    fn pow_f32(&self, x: &[f32], p: f32, y: &mut [f32]) -> Result<()> {
        #[cfg(feature = "vulkan")]
        {
            let size = x.len();
            let buffer_size = size * std::mem::size_of::<f32>();

            let mut buf_x = VulkanContext::create_buffer(
                &self.context,
                buffer_size,
                vk::BufferUsageFlags::STORAGE_BUFFER,
            )?;
            let buf_y = VulkanContext::create_buffer(
                &self.context,
                buffer_size,
                vk::BufferUsageFlags::STORAGE_BUFFER,
            )?;

            buf_x.write_from_slice(bytemuck::cast_slice(x))?;

            unsafe {
                self.execute_compute(
                    &self.context.pipelines.pow,
                    &[&buf_x, &buf_y],
                    size,
                    Some(bytemuck::bytes_of(&p)),
                )?;
            }

            let mut result_bytes = vec![0u8; buffer_size];
            buf_y.read_to_slice(&mut result_bytes)?;
            y.copy_from_slice(bytemuck::cast_slice(&result_bytes));

            Ok(())
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(GpuError::UnsupportedBackend(
                "Vulkan not enabled".to_string(),
            ))
        }
    }

    fn sum_f32(&self, x: &[f32]) -> Result<f32> {
        Ok(x.iter().sum())
    }

    fn max_f32(&self, x: &[f32]) -> Result<f32> {
        x.iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .ok_or(GpuError::ExecutionFailed("Empty array".to_string()))
    }

    fn min_f32(&self, x: &[f32]) -> Result<f32> {
        x.iter()
            .copied()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .ok_or(GpuError::ExecutionFailed("Empty array".to_string()))
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
        #[cfg(feature = "vulkan")]
        {
            unsafe {
                self.context.device.device_wait_idle().map_err(|e| {
                    GpuError::ExecutionFailed(format!("Failed to synchronize device: {:?}", e))
                })?;
            }
            Ok(())
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(GpuError::UnsupportedBackend(
                "Vulkan not enabled".to_string(),
            ))
        }
    }
}
