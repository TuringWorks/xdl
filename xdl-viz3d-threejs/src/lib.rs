//! Three.js-based volume rendering for XDL
//!
//! This crate provides volume visualization using Three.js and WebGL,
//! as an alternative to the WebGPU-based xdl-viz3d implementation.

use std::process::Command;
use xdl_core::{XdlError, XdlResult};

pub mod colormaps;
pub mod shaders;
pub mod templates;

/// Launch Three.js volume visualization in Tauri viewer
pub fn launch_visualization(
    volume_data: Vec<f32>,
    dims: [usize; 3],
    colormap: &str,
    title: Option<&str>,
) -> XdlResult<()> {
    let title = title.unwrap_or("3D Volume Visualization");

    // Generate HTML
    let html = templates::generate_volume_html(
        &volume_data,
        dims,
        colormap,
        title,
        0.1, // Default threshold
        0.8, // Default opacity
    );

    // Write to temp file
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("xdl_viz3d_{}.html", std::process::id()));

    std::fs::write(&temp_file, html)
        .map_err(|e| XdlError::RuntimeError(format!("Failed to write volume HTML: {}", e)))?;

    // Find xdl-chart-viewer
    let exe_dir = std::env::current_exe()
        .map_err(|e| XdlError::RuntimeError(format!("Cannot find exe: {}", e)))?
        .parent()
        .ok_or_else(|| XdlError::RuntimeError("Cannot find parent directory".to_string()))?
        .to_path_buf();

    // Try with .exe extension first (Windows), then without
    let viewer_path = if cfg!(windows) {
        let with_exe = exe_dir.join("xdl-chart-viewer.exe");
        if with_exe.exists() {
            with_exe
        } else {
            exe_dir.join("xdl-chart-viewer")
        }
    } else {
        exe_dir.join("xdl-chart-viewer")
    };

    // Launch viewer
    Command::new(viewer_path)
        .args([
            "--html-file",
            temp_file.to_str().unwrap(),
            "--title",
            title,
            "--width",
            "1280",
            "--height",
            "720",
        ])
        .spawn()
        .map_err(|e| XdlError::RuntimeError(format!("Failed to launch viewer: {}", e)))?;

    println!("✓ Three.js volume visualization launched");
    println!("  Volume: {}×{}×{}", dims[0], dims[1], dims[2]);
    println!("  Colormap: {}", colormap);

    Ok(())
}

/// Generate HTML without launching viewer (for testing)
pub fn generate_html(
    volume_data: Vec<f32>,
    dims: [usize; 3],
    colormap: &str,
    title: &str,
) -> String {
    templates::generate_volume_html(&volume_data, dims, colormap, title, 0.1, 0.8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_generation() {
        let volume_data: Vec<f32> = (0..1000).map(|i| i as f32 / 1000.0).collect();
        let dims = [10, 10, 10];

        let html = generate_html(volume_data, dims, "VIRIDIS", "Test Volume");

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("three") || html.contains("THREE")); // Three.js import
        assert!(html.contains("Test Volume"));
        assert!(html.contains("10")); // Dimensions
    }

    #[test]
    fn test_colormap_names() {
        let volume_data: Vec<f32> = vec![0.0; 1000];
        let dims = [10, 10, 10];

        for colormap in &[
            "VIRIDIS",
            "RAINBOW",
            "PLASMA",
            "INFERNO",
            "TURBO",
            "GRAYSCALE",
        ] {
            let html = generate_html(volume_data.clone(), dims, colormap, "Test");
            assert!(html.contains(colormap));
        }
    }
}
