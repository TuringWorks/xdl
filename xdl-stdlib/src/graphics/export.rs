//! Multi-format export for visualizations
//!
//! Support for exporting plots to SVG, PDF, and interactive HTML formats.

use xdl_core::{XdlError, XdlResult};

/// Export format options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    PNG,
    SVG,
    PDF,
    HTML,
}

/// Export configuration
pub struct ExportConfig {
    pub format: ExportFormat,
    pub width: u32,
    pub height: u32,
    pub dpi: u32,
    pub background_color: (u8, u8, u8),
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            format: ExportFormat::PNG,
            width: 800,
            height: 600,
            dpi: 96,
            background_color: (255, 255, 255),
        }
    }
}

impl ExportConfig {
    pub fn new(format: ExportFormat) -> Self {
        Self {
            format,
            ..Default::default()
        }
    }

    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn with_dpi(mut self, dpi: u32) -> Self {
        self.dpi = dpi;
        self
    }

    pub fn with_background(mut self, r: u8, g: u8, b: u8) -> Self {
        self.background_color = (r, g, b);
        self
    }
}

// Note: create_backend is commented out because DrawingBackend is not dyn-safe
// Users should directly instantiate BitMapBackend or SVGBackend as needed
/*
pub fn create_backend(
    filename: &str,
    config: &ExportConfig,
) -> XdlResult<Box<dyn DrawingBackend>> {
    // DrawingBackend trait is not dyn-safe (requires Self: Sized)
    // Use concrete types instead
    unimplemented!()
}
*/

/// Generate HTML wrapper for interactive visualization
pub fn generate_html_wrapper(svg_content: &str, title: &str, width: u32, height: u32) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            display: flex;
            flex-direction: column;
            align-items: center;
            padding: 20px;
            background-color: #f0f0f0;
        }}
        .container {{
            background-color: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        h1 {{
            color: #333;
            margin-top: 0;
        }}
        #plot {{
            max-width: 100%;
            height: auto;
        }}
        .controls {{
            margin-top: 15px;
            padding: 10px;
            background-color: #f8f8f8;
            border-radius: 4px;
        }}
        button {{
            padding: 8px 16px;
            margin: 5px;
            background-color: #4CAF50;
            color: white;
            border: none;
            border-radius: 4px;
            cursor: pointer;
        }}
        button:hover {{
            background-color: #45a049;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>{title}</h1>
        <div id="plot">
            {svg_content}
        </div>
        <div class="controls">
            <button onclick="downloadSVG()">Download SVG</button>
            <button onclick="downloadPNG()">Download PNG</button>
        </div>
    </div>

    <script>
        function downloadSVG() {{
            const svg = document.querySelector('svg');
            const serializer = new XMLSerializer();
            const svgStr = serializer.serializeToString(svg);
            const blob = new Blob([svgStr], {{type: 'image/svg+xml'}});
            const url = URL.createObjectURL(blob);
            const link = document.createElement('a');
            link.href = url;
            link.download = '{title}.svg';
            link.click();
            URL.revokeObjectURL(url);
        }}

        function downloadPNG() {{
            const svg = document.querySelector('svg');
            const canvas = document.createElement('canvas');
            canvas.width = {width};
            canvas.height = {height};
            const ctx = canvas.getContext('2d');
            const data = new XMLSerializer().serializeToString(svg);
            const img = new Image();
            const svgBlob = new Blob([data], {{type: 'image/svg+xml;charset=utf-8'}});
            const url = URL.createObjectURL(svgBlob);

            img.onload = function() {{
                ctx.drawImage(img, 0, 0);
                URL.revokeObjectURL(url);
                canvas.toBlob(function(blob) {{
                    const pngUrl = URL.createObjectURL(blob);
                    const link = document.createElement('a');
                    link.href = pngUrl;
                    link.download = '{title}.png';
                    link.click();
                    URL.revokeObjectURL(pngUrl);
                }});
            }};
            img.src = url;
        }}
    </script>
</body>
</html>"#,
        title = title,
        svg_content = svg_content,
        width = width,
        height = height
    )
}

/// Save HTML file with embedded SVG
pub fn export_to_html(
    svg_filename: &str,
    html_filename: &str,
    title: &str,
    width: u32,
    height: u32,
) -> XdlResult<()> {
    use std::fs;

    // Read SVG content
    let svg_content = fs::read_to_string(svg_filename)
        .map_err(|e| XdlError::IoError(format!("Failed to read SVG: {}", e)))?;

    // Generate HTML wrapper
    let html = generate_html_wrapper(&svg_content, title, width, height);

    // Write HTML file
    fs::write(html_filename, html)
        .map_err(|e| XdlError::IoError(format!("Failed to write HTML: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_config() {
        let config = ExportConfig::new(ExportFormat::SVG)
            .with_size(1024, 768)
            .with_dpi(300);

        assert_eq!(config.format, ExportFormat::SVG);
        assert_eq!(config.width, 1024);
        assert_eq!(config.height, 768);
        assert_eq!(config.dpi, 300);
    }

    #[test]
    fn test_html_generation() {
        let html = generate_html_wrapper("<svg></svg>", "Test Plot", 800, 600);

        assert!(html.contains("Test Plot"));
        assert!(html.contains("<svg></svg>"));
        assert!(html.contains("downloadSVG"));
    }
}
