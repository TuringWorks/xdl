//! Volume renderer with ray marching

use crate::{
    camera::{Camera, CameraUniform},
    colormap::Colormap,
    volume::VolumeData,
};
use anyhow::Result;
use wgpu::{Device, Queue, SurfaceConfiguration, TextureView};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct VolumeParams {
    dimensions: [f32; 4],
    data_min: f32,
    data_max: f32,
    step_size: f32,
    max_steps: u32,
}

/// Volume renderer
pub struct VolumeRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: Option<wgpu::BindGroup>,
    camera_buffer: wgpu::Buffer,
    params_buffer: wgpu::Buffer,
    volume_texture: Option<wgpu::Texture>,
    colormap_texture: wgpu::Texture,
    sampler: wgpu::Sampler,
    current_colormap: Colormap,
}

impl VolumeRenderer {
    /// Create new volume renderer
    pub fn new(device: &Device, config: &SurfaceConfiguration) -> Result<Self> {
        // Load shader
        let shader_source = include_str!("../shaders/volume_raymarch.wgsl");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Volume Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Volume Bind Group Layout"),
            entries: &[
                // Camera uniform
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Volume params
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Volume texture
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                // Volume sampler (NonFiltering for R32Float textures)
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                // Colormap texture
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D1,
                        multisampled: false,
                    },
                    count: None,
                },
                // Colormap sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Volume Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Volume Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Create uniform buffers
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Volume Params Buffer"),
            size: std::mem::size_of::<VolumeParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create sampler (Nearest for R32Float which is not filterable)
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Volume Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Create default colormap (Viridis)
        // Note: We need to get queue from device - using a placeholder texture for now
        // Will be properly initialized when queue is available
        let colormap = Colormap::Viridis;

        let colormap_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Colormap Texture"),
            size: wgpu::Extent3d {
                width: 256,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D1,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        Ok(Self {
            pipeline,
            bind_group_layout,
            bind_group: None,
            camera_buffer,
            params_buffer,
            volume_texture: None,
            colormap_texture,
            sampler,
            current_colormap: colormap,
        })
    }

    /// Create 1D colormap texture
    fn create_colormap_texture(
        device: &Device,
        queue: &Queue,
        colormap: &Colormap,
    ) -> wgpu::Texture {
        let lut = colormap.generate_lut();
        let data: Vec<u8> = lut.iter().flat_map(|c| c.iter().copied()).collect();

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Colormap Texture"),
            size: wgpu::Extent3d {
                width: 256,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D1,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(256 * 4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: 256,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        texture
    }

    /// Update colormap
    pub fn set_colormap(&mut self, device: &Device, queue: &Queue, colormap: Colormap) {
        if colormap != self.current_colormap {
            self.colormap_texture = Self::create_colormap_texture(device, queue, &colormap);
            self.current_colormap = colormap;
            self.bind_group = None; // Force bind group recreation
        }
    }

    /// Initialize colormap data (call after queue is available)
    pub fn init_colormap(&mut self, queue: &Queue) {
        let data: Vec<u8> = self
            .current_colormap
            .generate_lut()
            .iter()
            .flat_map(|c| c.iter().copied())
            .collect();

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.colormap_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(256 * 4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: 256,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Load volume data to GPU
    pub fn load_volume(
        &mut self,
        device: &Device,
        queue: &Queue,
        volume: VolumeData,
    ) -> Result<()> {
        let dims = volume.dimensions();

        // Create 3D texture
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Volume Texture"),
            size: wgpu::Extent3d {
                width: dims[0] as u32,
                height: dims[1] as u32,
                depth_or_array_layers: dims[2] as u32,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Upload data
        let data_bytes = bytemuck::cast_slice(&volume.data);
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data_bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(dims[0] as u32 * 4),
                rows_per_image: Some(dims[1] as u32),
            },
            wgpu::Extent3d {
                width: dims[0] as u32,
                height: dims[1] as u32,
                depth_or_array_layers: dims[2] as u32,
            },
        );

        // Update params
        let data_min = volume.data.iter().copied().fold(f32::INFINITY, f32::min);
        let data_max = volume
            .data
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max);

        let params = VolumeParams {
            dimensions: [dims[0] as f32, dims[1] as f32, dims[2] as f32, 0.0],
            data_min,
            data_max,
            step_size: 0.01,
            max_steps: 512,
        };

        queue.write_buffer(&self.params_buffer, 0, bytemuck::cast_slice(&[params]));

        self.volume_texture = Some(texture);
        self.bind_group = None; // Force recreation with new volume

        Ok(())
    }

    /// Create or update bind group
    fn ensure_bind_group(&mut self, device: &Device) {
        if self.bind_group.is_some() {
            return;
        }

        let volume_texture = self
            .volume_texture
            .as_ref()
            .expect("Volume texture must be loaded before rendering");

        let volume_view = volume_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let colormap_view = self
            .colormap_texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Volume Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&volume_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&colormap_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        self.bind_group = Some(bind_group);
    }

    /// Render the volume
    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        view: &TextureView,
        camera: &Camera,
    ) -> Result<(), wgpu::SurfaceError> {
        // Update camera uniform
        let camera_uniform = camera.uniform_data();
        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );

        // Ensure bind group is created
        self.ensure_bind_group(device);

        let bind_group = self.bind_group.as_ref().unwrap();

        // Create command encoder
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Volume Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Volume Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.draw(0..4, 0..1); // Fullscreen quad
        }

        queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}
