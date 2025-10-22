//! Image display window for showing generated plot PNG files

use anyhow::Result;
use fltk::{
    app,
    enums::{Color, Event},
    image::PngImage,
    prelude::*,
    window::Window,
};

pub struct ImageWindow {
    window: Window,
}

impl ImageWindow {
    pub fn new(image_path: &str, title: &str) -> Result<Self> {
        // Try to load the PNG image
        let mut img = PngImage::load(image_path)
            .map_err(|e| anyhow::anyhow!("Failed to load image {}: {}", image_path, e))?;

        // Get image dimensions
        let img_width = img.width();
        let img_height = img.height();

        // Add some padding
        let padding = 20;
        let win_width = img_width + padding * 2;
        let win_height = img_height + padding * 2 + 30; // Extra for title bar

        // Create window
        let mut window = Window::new(
            100,
            100,
            win_width,
            win_height,
            title,
        );
        window.set_color(Color::White);
        
        // Scale image if too large
        let max_width = 1200;
        let max_height = 900;
        if img_width > max_width || img_height > max_height {
            let scale_w = max_width as f64 / img_width as f64;
            let scale_h = max_height as f64 / img_height as f64;
            let scale = scale_w.min(scale_h);
            
            let new_width = (img_width as f64 * scale) as i32;
            let new_height = (img_height as f64 * scale) as i32;
            
            img.scale(new_width, new_height, true, true);
            window.set_size(new_width + padding * 2, new_height + padding * 2 + 30);
        }

        // Center image in window
        let x_offset = padding;
        let y_offset = padding;

        // Draw image
        window.draw(move |_| {
            img.draw(x_offset, y_offset, img.width(), img.height());
        });

        // Handle close event
        window.handle(move |win, event| {
            if event == Event::Close {
                win.hide();
                true
            } else {
                false
            }
        });

        window.end();
        window.show();

        Ok(Self { window })
    }

    pub fn show(&mut self) {
        self.window.show();
        // Process events to ensure window is displayed
        while self.window.shown() {
            app::wait();
        }
    }
}
