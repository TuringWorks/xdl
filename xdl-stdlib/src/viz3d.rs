//! 3D Volume Visualization functions for XDL
//!
//! Supports multiple rendering backends:
//! - Three.js (WebGL) - Better compatibility, runs in Tauri
//! - WebGPU (Native) - High performance, native window
//! - WebGPU (Browser) - Browser-based, requires modern browser

use std::collections::HashMap;
use std::sync::Mutex;
use xdl_core::{XdlError, XdlResult, XdlValue};

/// Backend selection for VIZ3D rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Viz3DBackend {
    /// Three.js WebGL renderer (Tauri-based)
    ThreeJS,
    /// Native WebGPU renderer (winit window)
    WebGPU,
    /// Browser-based WebGPU renderer
    Browser,
    /// Automatically select best available backend
    Auto,
}

impl Viz3DBackend {
    /// Detect backend from environment variable
    fn from_env() -> Self {
        match std::env::var("VIZ3D_BACKEND")
            .unwrap_or_else(|_| "auto".to_string())
            .to_lowercase()
            .as_str()
        {
            "threejs" | "three" => Self::ThreeJS,
            "webgpu" | "native" => Self::WebGPU,
            "browser" | "web" => Self::Browser,
            _ => Self::Auto,
        }
    }

    /// Resolve Auto to concrete backend
    fn resolve(self) -> Self {
        match self {
            Self::Auto => {
                // Always prefer Three.js (Tauri-based) for best compatibility and consistency
                // Three.js viewer works in both CLI and GUI modes without opening browser
                // Only use Browser mode if explicitly requested via VIZ3D_BACKEND=browser
                if std::env::var("VIZ3D_BROWSER").unwrap_or_default() == "1" {
                    Self::Browser
                } else {
                    Self::ThreeJS // Default to Three.js for all modes
                }
            }
            other => other,
        }
    }
}

// Global state for the 3D visualization
static VIZ3D_STATE: Mutex<Option<Viz3DState>> = Mutex::new(None);

struct Viz3DState {
    // Volume data
    volume_data: Option<Vec<f32>>,
    volume_dims: Option<[usize; 3]>,

    // Colormap
    colormap: String,

    // Camera settings
    camera_position: Option<[f32; 3]>,
    camera_target: Option<[f32; 3]>,
    camera_fov: f32,

    // Rendering settings
    initialized: bool,
    window_size: [u32; 2],
    window_title: String,
}

impl Default for Viz3DState {
    fn default() -> Self {
        Self {
            volume_data: None,
            volume_dims: None,
            colormap: "VIRIDIS".to_string(),
            camera_position: None,
            camera_target: None,
            camera_fov: 45.0,
            initialized: false,
            window_size: [1280, 720],
            window_title: "XDL 3D Visualization".to_string(),
        }
    }
}

/// VIZ3D_INIT - Initialize the 3D visualization system
///
/// Usage: VIZ3D_INIT, WINDOW_SIZE=[width, height], TITLE='title'
pub fn viz3d_init(_args: &[XdlValue], keywords: &HashMap<String, XdlValue>) -> XdlResult<XdlValue> {
    let mut state = VIZ3D_STATE
        .lock()
        .map_err(|e| XdlError::RuntimeError(format!("Failed to lock VIZ3D state: {}", e)))?;

    // Create or reset state
    let mut new_state = Viz3DState::default();

    // Parse WINDOW_SIZE keyword
    if let Some(size_val) = keywords.get("WINDOW_SIZE").or(keywords.get("window_size")) {
        if let XdlValue::Array(arr) = size_val {
            if arr.len() >= 2 {
                new_state.window_size = [arr[0] as u32, arr[1] as u32];
            }
        } else if let XdlValue::NestedArray(arr) = size_val {
            if arr.len() >= 2 {
                if let (Ok(w), Ok(h)) = (arr[0].to_double(), arr[1].to_double()) {
                    new_state.window_size = [w as u32, h as u32];
                }
            }
        }
    }

    // Parse TITLE keyword
    if let Some(XdlValue::String(title)) = keywords.get("TITLE").or(keywords.get("title")) {
        new_state.window_title = title.clone();
    }

    new_state.initialized = true;
    *state = Some(new_state);

    println!(
        "VIZ3D: Initialized ({}x{})",
        state.as_ref().unwrap().window_size[0],
        state.as_ref().unwrap().window_size[1]
    );

    Ok(XdlValue::Undefined)
}

/// VIZ3D_VOLUME - Upload 3D volume data to GPU
///
/// Usage: VIZ3D_VOLUME, data, DIMENSIONS=[nx, ny, nz]
pub fn viz3d_volume(
    args: &[XdlValue],
    keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "VIZ3D_VOLUME requires volume data as first argument".to_string(),
        ));
    }

    let mut state = VIZ3D_STATE
        .lock()
        .map_err(|e| XdlError::RuntimeError(format!("Failed to lock VIZ3D state: {}", e)))?;

    let state = state.as_mut().ok_or_else(|| {
        XdlError::RuntimeError("VIZ3D_INIT must be called before VIZ3D_VOLUME".to_string())
    })?;

    // Extract volume data
    let volume_data = extract_volume_data(&args[0])?;

    // Parse DIMENSIONS keyword
    let dims = if let Some(dims_val) = keywords.get("DIMENSIONS").or(keywords.get("dimensions")) {
        extract_dimensions(dims_val)?
    } else {
        // Try to infer dimensions from array structure
        infer_dimensions(&args[0])?
    };

    // Validate data size matches dimensions
    let expected_size = dims[0] * dims[1] * dims[2];
    if volume_data.len() != expected_size {
        return Err(XdlError::RuntimeError(format!(
            "Volume data size ({}) does not match dimensions ({}x{}x{} = {})",
            volume_data.len(),
            dims[0],
            dims[1],
            dims[2],
            expected_size
        )));
    }

    state.volume_data = Some(volume_data);
    state.volume_dims = Some(dims);

    println!(
        "VIZ3D: Loaded volume {}x{}x{} ({} voxels)",
        dims[0], dims[1], dims[2], expected_size
    );

    Ok(XdlValue::Undefined)
}

/// VIZ3D_COLORMAP - Set colormap for visualization
///
/// Usage: VIZ3D_COLORMAP, 'name', MIN=min_val, MAX=max_val
pub fn viz3d_colormap(
    args: &[XdlValue],
    _keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "VIZ3D_COLORMAP requires colormap name as argument".to_string(),
        ));
    }

    let colormap_name = match &args[0] {
        XdlValue::String(s) => s.to_uppercase(),
        _ => {
            return Err(XdlError::RuntimeError(
                "VIZ3D_COLORMAP colormap name must be a string".to_string(),
            ))
        }
    };

    // Validate colormap name
    let valid_colormaps = [
        "RAINBOW",
        "VIRIDIS",
        "PLASMA",
        "INFERNO",
        "TURBO",
        "GRAYSCALE",
        "GRAY",
    ];
    if !valid_colormaps.contains(&colormap_name.as_str()) {
        return Err(XdlError::RuntimeError(format!(
            "Unknown colormap '{}'. Valid options: {}",
            colormap_name,
            valid_colormaps.join(", ")
        )));
    }

    let mut state = VIZ3D_STATE
        .lock()
        .map_err(|e| XdlError::RuntimeError(format!("Failed to lock VIZ3D state: {}", e)))?;

    let state = state.as_mut().ok_or_else(|| {
        XdlError::RuntimeError("VIZ3D_INIT must be called before VIZ3D_COLORMAP".to_string())
    })?;

    state.colormap = colormap_name.clone();
    println!("VIZ3D: Set colormap to {}", colormap_name);

    Ok(XdlValue::Undefined)
}

/// VIZ3D_CAMERA - Set camera position and parameters
///
/// Usage: VIZ3D_CAMERA, POSITION=[x,y,z], TARGET=[x,y,z], FOV=degrees
pub fn viz3d_camera(
    _args: &[XdlValue],
    keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    let mut state = VIZ3D_STATE
        .lock()
        .map_err(|e| XdlError::RuntimeError(format!("Failed to lock VIZ3D state: {}", e)))?;

    let state = state.as_mut().ok_or_else(|| {
        XdlError::RuntimeError("VIZ3D_INIT must be called before VIZ3D_CAMERA".to_string())
    })?;

    // Parse POSITION keyword
    if let Some(pos_val) = keywords.get("POSITION").or(keywords.get("position")) {
        let pos = extract_float3(pos_val)?;
        state.camera_position = Some(pos);
    }

    // Parse TARGET keyword
    if let Some(target_val) = keywords.get("TARGET").or(keywords.get("target")) {
        let target = extract_float3(target_val)?;
        state.camera_target = Some(target);
    }

    // Parse FOV keyword
    if let Some(XdlValue::Float(fov)) = keywords.get("FOV").or(keywords.get("fov")) {
        state.camera_fov = *fov;
    }

    println!("VIZ3D: Camera configured");
    Ok(XdlValue::Undefined)
}

/// VIZ3D_RENDER - Render the volume
///
/// Usage: VIZ3D_RENDER, /INTERACTIVE, TITLE='title'
pub fn viz3d_render(
    _args: &[XdlValue],
    keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    let state = VIZ3D_STATE
        .lock()
        .map_err(|e| XdlError::RuntimeError(format!("Failed to lock VIZ3D state: {}", e)))?;

    let state = state.as_ref().ok_or_else(|| {
        XdlError::RuntimeError("VIZ3D_INIT must be called before VIZ3D_RENDER".to_string())
    })?;

    if !state.initialized {
        return Err(XdlError::RuntimeError(
            "VIZ3D not initialized. Call VIZ3D_INIT first".to_string(),
        ));
    }

    if state.volume_data.is_none() {
        return Err(XdlError::RuntimeError(
            "No volume data loaded. Call VIZ3D_VOLUME first".to_string(),
        ));
    }

    // Check for INTERACTIVE keyword
    let interactive = keywords.contains_key("INTERACTIVE") || keywords.contains_key("interactive");

    println!("VIZ3D: Rendering volume (interactive={})", interactive);
    println!(
        "  Volume: {}x{}x{}",
        state.volume_dims.unwrap()[0],
        state.volume_dims.unwrap()[1],
        state.volume_dims.unwrap()[2]
    );
    println!("  Colormap: {}", state.colormap);

    // Get title from keywords
    let title = keywords
        .get("TITLE")
        .or(keywords.get("title"))
        .and_then(|v| match v {
            XdlValue::String(s) => Some(s.as_str()),
            _ => None,
        });

    // Detect and resolve backend
    let backend = Viz3DBackend::from_env().resolve();
    println!("  Backend: {:?}", backend);

    // Route to appropriate backend
    if interactive && backend == Viz3DBackend::ThreeJS {
        // Three.js WebGL rendering (Tauri-based)
        println!("\n🚀 Launching Three.js volume visualization...");
        println!("Controls:");
        println!("  - Left mouse: Rotate camera");
        println!("  - Mouse wheel: Zoom in/out");
        println!("  - GUI sliders: Adjust threshold and opacity\n");

        let result = xdl_viz3d_threejs::launch_visualization(
            state.volume_data.clone().unwrap(),
            state.volume_dims.unwrap(),
            &state.colormap,
            title,
        );

        match result {
            Ok(_) => {
                println!("\n✓ Three.js visualization launched.");
                Ok(XdlValue::Undefined)
            }
            Err(e) => Err(XdlError::RuntimeError(format!(
                "Failed to launch Three.js visualization: {}",
                e
            ))),
        }
    } else if interactive && backend == Viz3DBackend::WebGPU {
        // Native WebGPU window rendering
        println!("\n🎮 Launching native WebGPU visualization...");
        println!("Controls:");
        println!("  - Left mouse button: Rotate camera");
        println!("  - Mouse wheel: Zoom in/out");
        println!("  - ESC: Close window\n");

        let result = xdl_viz3d::launch_visualization(
            state.volume_data.clone().unwrap(),
            state.volume_dims.unwrap(),
            &state.colormap,
            title,
        );

        match result {
            Ok(_) => {
                println!("\nVisualization window closed.");
                Ok(XdlValue::Undefined)
            }
            Err(e) => Err(XdlError::RuntimeError(format!(
                "Failed to launch WebGPU visualization: {}",
                e
            ))),
        }
    } else if interactive && backend == Viz3DBackend::Browser {
        println!("\n🌐 Launching browser-based WebGPU visualization...");
        println!("Controls:");
        println!("  - Left mouse drag: Rotate camera");
        println!("  - Mouse wheel: Zoom in/out");
        println!("  - Use browser controls to adjust parameters\n");

        // Launch browser-based visualization (non-blocking)
        let result = xdl_viz3d_web::launch_browser_visualization(
            state.volume_data.clone().unwrap(),
            state.volume_dims.unwrap(),
            &state.colormap,
            title,
        );

        match result {
            Ok(url) => {
                println!("✓ Visualization available at: {}", url);
                println!("  Server is running - waiting 30 seconds for browser to load...");
                println!("  (You can Ctrl+C to exit early)\n");

                // Wait to give browser time to load the page
                std::thread::sleep(std::time::Duration::from_secs(30));

                println!("\n✓ Page should be loaded. Server will continue in background.\n");
                Ok(XdlValue::Undefined)
            }
            Err(e) => Err(XdlError::RuntimeError(format!(
                "Failed to launch browser visualization: {}",
                e
            ))),
        }
    } else if interactive {
        // In GUI mode, can't open blocking windows
        println!("\n⚠️  Interactive visualization not available in GUI mode.");
        println!("   Use xdl CLI to view interactive 3D windows.");
        println!(
            "   Data prepared: {}x{}x{} volume with {} colormap\n",
            state.volume_dims.unwrap()[0],
            state.volume_dims.unwrap()[1],
            state.volume_dims.unwrap()[2],
            state.colormap
        );
        Ok(XdlValue::Undefined)
    } else {
        // Non-interactive mode - just confirm data is ready
        println!("\nVisualization prepared (non-interactive mode).");
        println!("Note: Use /INTERACTIVE keyword to open 3D window.\n");
        Ok(XdlValue::Undefined)
    }
}

/// VIZ3D_TRANSFER - Configure transfer function
///
/// Usage: VIZ3D_TRANSFER, DENSITY=data, MODE='mode', ALPHA_SCALE=scale
pub fn viz3d_transfer(
    _args: &[XdlValue],
    _keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    // Placeholder - transfer function configuration
    println!("VIZ3D_TRANSFER: [Not yet implemented]");
    Ok(XdlValue::Undefined)
}

/// VIZ3D_LIGHT - Configure lighting
///
/// Usage: VIZ3D_LIGHT, POSITION=[x,y,z], INTENSITY=value
pub fn viz3d_light(
    _args: &[XdlValue],
    _keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    // Placeholder - lighting configuration
    println!("VIZ3D_LIGHT: [Not yet implemented]");
    Ok(XdlValue::Undefined)
}

/// VIZ3D_ISOSURFACE - Extract and render isosurface
///
/// Usage: VIZ3D_ISOSURFACE, data, ISOVALUE=value, COLOR=[r,g,b]
pub fn viz3d_isosurface(
    _args: &[XdlValue],
    _keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    // Placeholder - isosurface extraction
    println!("VIZ3D_ISOSURFACE: [Not yet implemented]");
    Ok(XdlValue::Undefined)
}

// Helper functions

fn extract_volume_data(value: &XdlValue) -> XdlResult<Vec<f32>> {
    // Flatten 3D array into 1D vec
    match value {
        XdlValue::Array(arr) => {
            // Simple 1D f64 array - convert to f32
            Ok(arr.iter().map(|&v| v as f32).collect())
        }
        XdlValue::NestedArray(arr) => {
            // Nested array - recursively flatten
            let mut result = Vec::new();
            for val in arr {
                result.extend(extract_volume_data(val)?);
            }
            Ok(result)
        }
        XdlValue::MultiDimArray { data, .. } => {
            // Multi-dimensional array
            Ok(data.iter().map(|&v| v as f32).collect())
        }
        _ => {
            // Try converting single value
            match value.to_double() {
                Ok(v) => Ok(vec![v as f32]),
                Err(_) => Err(XdlError::RuntimeError(
                    "Volume data must be an array or numeric value".to_string(),
                )),
            }
        }
    }
}

fn extract_dimensions(value: &XdlValue) -> XdlResult<[usize; 3]> {
    match value {
        XdlValue::Array(arr) if arr.len() >= 3 => {
            let mut dims = [0usize; 3];
            for (i, &val) in arr.iter().take(3).enumerate() {
                dims[i] = val as usize;
            }
            Ok(dims)
        }
        XdlValue::NestedArray(arr) if arr.len() >= 3 => {
            let mut dims = [0usize; 3];
            for (i, val) in arr.iter().take(3).enumerate() {
                dims[i] = val.to_double()? as usize;
            }
            Ok(dims)
        }
        _ => Err(XdlError::RuntimeError(
            "DIMENSIONS must be a 3-element array".to_string(),
        )),
    }
}

fn infer_dimensions(_value: &XdlValue) -> XdlResult<[usize; 3]> {
    // Try to infer 3D dimensions from nested array structure
    // This is a simplified version - might not handle all cases
    Err(XdlError::RuntimeError(
        "Could not infer dimensions. Please specify DIMENSIONS=[nx,ny,nz]".to_string(),
    ))
}

fn extract_float3(value: &XdlValue) -> XdlResult<[f32; 3]> {
    match value {
        XdlValue::Array(arr) if arr.len() >= 3 => {
            let mut result = [0.0f32; 3];
            for (i, &val) in arr.iter().take(3).enumerate() {
                result[i] = val as f32;
            }
            Ok(result)
        }
        XdlValue::NestedArray(arr) if arr.len() >= 3 => {
            let mut result = [0.0f32; 3];
            for (i, val) in arr.iter().take(3).enumerate() {
                result[i] = val.to_double()? as f32;
            }
            Ok(result)
        }
        _ => Err(XdlError::RuntimeError(
            "Position/target must be a 3-element array".to_string(),
        )),
    }
}
