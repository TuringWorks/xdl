//! XDL 3D Visualization Library
//!
//! High-fidelity 3D visualization for scientific simulations using WebGPU.
//! Supports volume rendering, isosurface extraction, and interactive visualization.

pub mod camera;
pub mod colormap;
pub mod renderer;
pub mod volume;

pub use camera::Camera;
pub use renderer::VolumeRenderer;
pub use volume::{VolumeData, VolumeFormat};

use anyhow::Result;
use std::sync::Arc;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowAttributes, WindowId},
};

/// 3D visualization application state
pub struct Viz3DApp {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    renderer: VolumeRenderer,
    camera: Camera,
    window: Arc<Window>,
}

impl Viz3DApp {
    /// Create a new 3D visualization application with an existing window
    pub async fn new_with_window(window: Arc<Window>) -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to find suitable adapter"))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("XDL 3D Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let size = window.inner_size();
        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let mut renderer = VolumeRenderer::new(&device, &config)?;
        renderer.init_colormap(&queue); // Initialize colormap texture

        let camera = Camera::new(
            glam::Vec3::new(0.0, 0.0, 3.0),
            glam::Vec3::ZERO,
            size.width as f32 / size.height as f32,
        );

        Ok(Self {
            surface,
            device,
            queue,
            config,
            renderer,
            camera,
            window,
        })
    }

    /// Load volume data for visualization
    pub fn load_volume(&mut self, volume: VolumeData) -> Result<()> {
        self.renderer.load_volume(&self.device, &self.queue, volume)
    }

    /// Resize the window
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.camera
                .set_aspect(new_size.width as f32 / new_size.height as f32);
        }
    }

    /// Handle input events
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera.handle_input(event)
    }

    /// Update the application state
    pub fn update(&mut self) {
        self.camera.update();
    }

    /// Render the current frame
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.renderer
            .render(&self.device, &self.queue, &view, &self.camera)?;

        output.present();
        Ok(())
    }

    /// Get device reference
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Get queue reference
    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    /// Get window reference
    pub fn window(&self) -> &Arc<Window> {
        &self.window
    }

    /// Get mutable renderer reference
    pub fn renderer_mut(&mut self) -> &mut VolumeRenderer {
        &mut self.renderer
    }

    /// Set the colormap for volume rendering
    pub fn set_colormap(&mut self, colormap: colormap::Colormap) {
        self.renderer
            .set_colormap(&self.device, &self.queue, colormap);
    }
}

/// Application handler for winit 0.30
struct Viz3DHandler {
    app: Option<Viz3DApp>,
    volume_data: Option<VolumeData>,
    colormap: colormap::Colormap,
    title: String,
}

impl ApplicationHandler for Viz3DHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window on first resume (winit 0.30 pattern)
        if self.app.is_none() {
            println!("Creating visualization window...");

            let window_attrs = WindowAttributes::default()
                .with_title(&self.title)
                .with_inner_size(winit::dpi::PhysicalSize::new(1280, 720));

            let window = match event_loop.create_window(window_attrs) {
                Ok(w) => Arc::new(w),
                Err(e) => {
                    eprintln!("Failed to create window: {}", e);
                    event_loop.exit();
                    return;
                }
            };

            // Create app with the window
            let app = match pollster::block_on(Viz3DApp::new_with_window(window)) {
                Ok(mut a) => {
                    // Load volume if provided
                    if let Some(volume) = self.volume_data.take() {
                        if let Err(e) = a.load_volume(volume) {
                            eprintln!("Failed to load volume: {}", e);
                            event_loop.exit();
                            return;
                        }
                    }
                    a.set_colormap(self.colormap);
                    a
                }
                Err(e) => {
                    eprintln!("Failed to create app: {}", e);
                    event_loop.exit();
                    return;
                }
            };

            app.window().request_redraw();
            self.app = Some(app);
            println!("Window created successfully");
            println!(
                "Window is visible: {}",
                self.app
                    .as_ref()
                    .unwrap()
                    .window()
                    .is_visible()
                    .unwrap_or(false)
            );
        } else if let Some(app) = &self.app {
            app.window().request_redraw();
        }
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: winit::event::StartCause) {
        // Handle new events cycle
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: ()) {
        // Handle custom user events
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        _event: winit::event::DeviceEvent,
    ) {
        // Handle device events if needed
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(app) = self.app.as_mut() else { return };
        if window_id != app.window().id() {
            return;
        }

        if !app.input(&event) {
            match event {
                WindowEvent::CloseRequested => {
                    println!("Close requested - exiting");
                    event_loop.exit();
                }
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => {
                    event_loop.exit();
                }
                WindowEvent::Resized(physical_size) => {
                    app.resize(physical_size);
                }
                WindowEvent::RedrawRequested => {
                    app.update();
                    match app.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            println!("Surface lost, reconfiguring...");
                            app.resize(app.window().inner_size());
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            eprintln!("Out of memory!");
                            event_loop.exit();
                        }
                        Err(e) => {
                            eprintln!("Render error: {:?}", e);
                            event_loop.exit();
                        }
                    }
                    app.window().request_redraw();
                }
                _ => {}
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(app) = &self.app {
            app.window().request_redraw();
        }
    }
}

/// Run the 3D visualization event loop
pub fn run(
    volume_data: Option<VolumeData>,
    colormap: colormap::Colormap,
    title: String,
    event_loop: EventLoop<()>,
) -> Result<()> {
    let mut handler = Viz3DHandler {
        app: None,
        volume_data,
        colormap,
        title,
    };
    event_loop.run_app(&mut handler)?;
    Ok(())
}

use std::sync::Mutex;
use std::sync::OnceLock;

static EVENT_LOOP_CREATED: OnceLock<Mutex<bool>> = OnceLock::new();

/// Launch a 3D visualization window with volume data
///
/// This is a high-level API for creating and running a visualization window.
/// It blocks until the window is closed.
///
/// Note: Due to winit limitations, only one event loop can be created per process.
/// After the first visualization window is closed, subsequent calls will fail.
/// This is a fundamental limitation of the windowing system.
pub fn launch_visualization(
    volume_data: Vec<f32>,
    dimensions: [usize; 3],
    colormap_name: &str,
    title: Option<&str>,
) -> Result<()> {
    use colormap::Colormap;

    // Check if event loop was already created and used
    let already_used = EVENT_LOOP_CREATED
        .get_or_init(|| Mutex::new(false))
        .lock()
        .map(|mut guard| {
            let used = *guard;
            *guard = true;
            used
        })
        .unwrap_or(false);

    if already_used {
        return Err(anyhow::anyhow!(
            "Cannot create multiple visualization windows in the same process. \
            This is a limitation of the windowing system (winit). \
            Please run each visualization in a separate XDL script execution."
        ));
    }

    let event_loop = EventLoop::new()?;

    // Parse colormap from string
    let colormap = match colormap_name.to_uppercase().as_str() {
        "VIRIDIS" => Colormap::Viridis,
        "RAINBOW" => Colormap::Rainbow,
        "PLASMA" => Colormap::Plasma,
        "INFERNO" => Colormap::Inferno,
        "TURBO" => Colormap::Turbo,
        "GRAYSCALE" | "GRAY" => Colormap::Grayscale,
        _ => Colormap::Viridis, // Default
    };

    // Prepare volume data
    let volume = VolumeData::new(volume_data, dimensions);

    // Set title
    let window_title = title.unwrap_or("XDL 3D Visualization").to_string();

    // Run event loop (blocks until window closed)
    // Window will be created in the resumed() callback
    run(Some(volume), colormap, window_title, event_loop)
}
