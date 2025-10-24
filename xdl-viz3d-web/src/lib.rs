//! Browser-based 3D Volume Visualization
//!
//! This crate provides WebGPU-accelerated volume rendering in the browser,
//! solving the native event loop limitations and providing a superior user experience.

use anyhow::Result;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::JoinHandle;

mod server;
mod template;

pub use server::VizServer;

// Global registry to keep server threads alive
static SERVER_HANDLES: OnceLock<Mutex<Vec<JoinHandle<()>>>> = OnceLock::new();

/// Launch a volume visualization in the browser
///
/// This creates a local HTTP server, generates an HTML page with embedded volume data,
/// and opens it in the default browser. Multiple visualizations can run simultaneously
/// in different tabs.
pub fn launch_browser_visualization(
    volume_data: Vec<f32>,
    dimensions: [usize; 3],
    colormap: &str,
    title: Option<&str>,
) -> Result<String> {
    // Create server on random available port
    let server = VizServer::new()?;
    let port = server.port();
    let url = format!("http://localhost:{}", port);

    // Generate HTML page with embedded volume data
    let html = template::generate_volume_viewer(
        &volume_data,
        dimensions,
        colormap,
        title.unwrap_or("XDL 3D Visualization"),
    );

    // Start server in detached background thread
    let server = Arc::new(Mutex::new(server));
    let html_clone = html.clone();

    // Spawn thread that keeps server running
    let handle = std::thread::Builder::new()
        .name(format!("viz-server-{}", port))
        .spawn(move || {
            if let Ok(server) = server.lock() {
                server.serve_html(html_clone);
            }
        })
        .expect("Failed to spawn server thread");

    // Store handle in global registry to keep thread alive
    SERVER_HANDLES
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .unwrap()
        .push(handle);

    // Give server a moment to start
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Open browser (non-blocking)
    println!("🌐 Launching visualization at: {}", url);
    if let Err(e) = webbrowser::open(&url) {
        eprintln!("⚠️  Could not auto-open browser: {}", e);
        println!("   Please open this URL manually: {}", url);
    }

    Ok(url)
}

/// Wait for all server threads to finish
/// This keeps the process alive so servers can continue serving
pub fn wait_for_servers() {
    let handles = SERVER_HANDLES.get();
    if let Some(handles_mutex) = handles {
        if let Ok(mut handles) = handles_mutex.lock() {
            if !handles.is_empty() {
                println!("\n📡 {} visualization server(s) running", handles.len());
                println!("   Keep this process running to view visualizations");
                println!("   Press Ctrl+C to stop all servers\n");

                // Wait for all handles
                while let Some(handle) = handles.pop() {
                    let _ = handle.join();
                }
            }
        }
    }
}
