//! XDL Desktop Viewer - Tauri-based visualization window
//!
//! This crate provides a Tauri-based desktop window for displaying
//! visualizations (charts, plots, 3D graphics) in a native application
//! window instead of a web browser.

use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder};

/// Window counter for unique window IDs
static WINDOW_COUNTER: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));

/// Configuration for a visualization window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Window title
    pub title: String,
    /// Window width in pixels
    pub width: f64,
    /// Window height in pixels
    pub height: f64,
    /// Whether the window is resizable
    pub resizable: bool,
    /// Whether to show window decorations (title bar, etc.)
    pub decorations: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "XDL Visualization".to_string(),
            width: 1024.0,
            height: 768.0,
            resizable: true,
            decorations: true,
        }
    }
}

/// Launch a visualization in a desktop window
///
/// # Arguments
/// * `html_content` - The HTML content to display
/// * `config` - Window configuration (optional, uses defaults if None)
///
/// # Returns
/// The window label/ID for reference
pub fn launch_window(html_content: String, config: Option<WindowConfig>) -> Result<String> {
    let config = config.unwrap_or_default();

    // Generate unique window ID
    let window_id = {
        let mut counter = WINDOW_COUNTER.lock().unwrap();
        *counter += 1;
        format!("xdl-viz-{}", *counter)
    };

    tracing::info!("Launching desktop viewer window: {}", window_id);

    // In a real implementation, we would:
    // 1. Initialize Tauri app if not already running
    // 2. Create new window with the HTML content
    // 3. Return window handle

    // For now, we'll use a simpler approach that works with the existing architecture
    launch_window_impl(&window_id, html_content, config)?;

    Ok(window_id)
}

/// Internal implementation that handles the actual window creation
fn launch_window_impl(window_id: &str, html_content: String, config: WindowConfig) -> Result<()> {
    // Store HTML content for the window
    // In production, this would be served via Tauri's asset protocol
    tracing::debug!("Creating window {} with config: {:?}", window_id, config);

    // Note: Tauri requires initialization from main.rs
    // This is a library, so we'll provide a function that can be called
    // from the main application (xdl-gui or xdl CLI with desktop mode)

    // For now, store the content and let the caller handle Tauri initialization
    PENDING_WINDOWS.lock().unwrap().push(PendingWindow {
        id: window_id.to_string(),
        html: html_content,
        config,
    });

    Ok(())
}

/// Pending window data structure
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PendingWindow {
    id: String,
    html: String,
    config: WindowConfig,
}

/// Storage for pending windows (before Tauri app is initialized)
static PENDING_WINDOWS: Lazy<Mutex<Vec<PendingWindow>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Create a window in an existing Tauri app
///
/// This is called by the host application (xdl-gui) when it has a Tauri app handle
pub fn create_window_in_app(
    app: &AppHandle,
    window_id: &str,
    html_content: &str,
    config: &WindowConfig,
) -> Result<()> {
    // Create data URL with the HTML content
    let data_url = format!(
        "data:text/html;charset=utf-8,{}",
        urlencoding::encode(html_content)
    );

    let _window =
        WebviewWindowBuilder::new(app, window_id, WebviewUrl::External(data_url.parse()?))
            .title(&config.title)
            .inner_size(config.width, config.height)
            .resizable(config.resizable)
            .decorations(config.decorations)
            .build()
            .context("Failed to create window")?;

    tracing::info!("Created Tauri window: {}", window_id);

    Ok(())
}

/// Get all pending windows (for batch processing)
pub fn take_pending_windows() -> Vec<PendingWindow> {
    PENDING_WINDOWS.lock().unwrap().drain(..).collect()
}

/// Simple launch function that falls back to browser if Tauri not available
///
/// This provides a seamless fallback mechanism
pub fn launch_or_fallback(html_content: String, config: Option<WindowConfig>) -> Result<String> {
    // Try desktop viewer first
    match launch_window(html_content.clone(), config) {
        Ok(window_id) => Ok(window_id),
        Err(e) => {
            tracing::warn!("Desktop viewer unavailable, falling back to browser: {}", e);
            // Fallback to browser (using xdl-viz3d-web server pattern)
            fallback_to_browser(html_content)
        }
    }
}

/// Fallback to browser-based rendering
fn fallback_to_browser(_html_content: String) -> Result<String> {
    // This would use the existing xdl-viz3d-web infrastructure
    // For now, return a placeholder
    tracing::info!("Using browser fallback");
    Ok("browser-window".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_config_default() {
        let config = WindowConfig::default();
        assert_eq!(config.title, "XDL Visualization");
        assert_eq!(config.width, 1024.0);
        assert_eq!(config.height, 768.0);
        assert!(config.resizable);
        assert!(config.decorations);
    }

    #[test]
    fn test_window_counter_increments() {
        let id1 = {
            let mut counter = WINDOW_COUNTER.lock().unwrap();
            *counter += 1;
            *counter
        };
        let id2 = {
            let mut counter = WINDOW_COUNTER.lock().unwrap();
            *counter += 1;
            *counter
        };
        assert_eq!(id2, id1 + 1);
    }
}
