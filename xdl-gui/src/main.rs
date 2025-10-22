//! XDL GUI Application
//!
//! A graphical user interface for XDL that provides:
//! - Interactive CLI within the GUI
//! - File operations (open, save, new)
//! - GUI-based plotting
//! - Integrated text editor

use anyhow::Result;
use tracing::info;

mod gui;
mod image_window;
mod plot_window;

use gui::XdlGui;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting XDL GUI Application");

    // Initialize FLTK
    let app = fltk::app::App::default();
    let mut gui = XdlGui::new()?;

    gui.show();

    info!("XDL GUI started successfully");
    app.run()?;

    Ok(())
}
