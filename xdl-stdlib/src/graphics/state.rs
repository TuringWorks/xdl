//! Graphics state management
//!
//! Manages the current graphics state including windows, colors, line styles, etc.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

lazy_static::lazy_static! {
    /// Global graphics state
    pub static ref GRAPHICS_STATE: Arc<Mutex<GraphicsState>> =
        Arc::new(Mutex::new(GraphicsState::new()));
}

/// Plot style options
#[derive(Debug, Clone)]
pub struct PlotStyle {
    pub color: Color,
    pub linestyle: LineStyle,
    pub thick: f64,
    pub psym: i32,
    pub symsize: f64,
}

impl Default for PlotStyle {
    fn default() -> Self {
        Self {
            color: Color::new(0, 0, 0), // Black
            linestyle: LineStyle::Solid,
            thick: 1.0,
            psym: 0, // No symbol
            symsize: 1.0,
        }
    }
}

/// Line style enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineStyle {
    Solid = 0,
    Dotted = 1,
    Dashed = 2,
    DashDot = 3,
    DashDotDot = 4,
    LongDashes = 5,
}

impl From<i32> for LineStyle {
    fn from(val: i32) -> Self {
        match val {
            1 => LineStyle::Dotted,
            2 => LineStyle::Dashed,
            3 => LineStyle::DashDot,
            4 => LineStyle::DashDotDot,
            5 => LineStyle::LongDashes,
            _ => LineStyle::Solid,
        }
    }
}

/// RGB Color
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn from_index(index: usize, color_table: &ColorTable) -> Self {
        color_table.get_color(index)
    }

    pub fn to_rgb(&self) -> plotters::prelude::RGBColor {
        plotters::prelude::RGBColor(self.r, self.g, self.b)
    }
}

/// Color table management
#[derive(Debug, Clone)]
pub struct ColorTable {
    red: Vec<u8>,
    green: Vec<u8>,
    blue: Vec<u8>,
}

impl ColorTable {
    pub fn new() -> Self {
        Self::grayscale()
    }

    /// Create grayscale color table
    pub fn grayscale() -> Self {
        let mut red = vec![0u8; 256];
        let mut green = vec![0u8; 256];
        let mut blue = vec![0u8; 256];

        for i in 0..256 {
            red[i] = i as u8;
            green[i] = i as u8;
            blue[i] = i as u8;
        }

        Self { red, green, blue }
    }

    /// Load predefined color table by number
    pub fn load_table(table_num: i32) -> Self {
        match table_num {
            0 => Self::grayscale(),
            1 => Self::blue_red(),
            2 => Self::blue_white(),
            3 => Self::grn_red_blu_wht(),
            13 => Self::rainbow(),
            _ => Self::grayscale(),
        }
    }

    /// Rainbow color table
    fn rainbow() -> Self {
        let mut red = vec![0u8; 256];
        let mut green = vec![0u8; 256];
        let mut blue = vec![0u8; 256];

        for i in 0..256 {
            let x = i as f64 / 255.0;
            if x < 0.2 {
                blue[i] = 255;
            } else if x < 0.4 {
                blue[i] = ((1.0 - (x - 0.2) / 0.2) * 255.0) as u8;
                green[i] = (((x - 0.2) / 0.2) * 255.0) as u8;
            } else if x < 0.6 {
                green[i] = 255;
            } else if x < 0.8 {
                green[i] = ((1.0 - (x - 0.6) / 0.2) * 255.0) as u8;
                red[i] = (((x - 0.6) / 0.2) * 255.0) as u8;
            } else {
                red[i] = 255;
            }
        }

        Self { red, green, blue }
    }

    /// Blue-red color table
    fn blue_red() -> Self {
        let mut red = vec![0u8; 256];
        let green = vec![0u8; 256];
        let mut blue = vec![0u8; 256];

        for i in 0..256 {
            let x = i as f64 / 255.0;
            red[i] = (x * 255.0) as u8;
            blue[i] = ((1.0 - x) * 255.0) as u8;
        }

        Self { red, green, blue }
    }

    /// Blue-white color table
    fn blue_white() -> Self {
        let mut red = vec![0u8; 256];
        let mut green = vec![0u8; 256];
        let mut blue = vec![0u8; 256];

        for i in 0..256 {
            let x = i as f64 / 255.0;
            red[i] = (x * 255.0) as u8;
            green[i] = (x * 255.0) as u8;
            blue[i] = 255;
        }

        Self { red, green, blue }
    }

    /// Green-red-blue-white color table
    fn grn_red_blu_wht() -> Self {
        let mut red = vec![0u8; 256];
        let mut green = vec![0u8; 256];
        let mut blue = vec![0u8; 256];

        for i in 0..256 {
            let x = i as f64 / 255.0;
            if x < 0.33 {
                green[i] = (x / 0.33 * 255.0) as u8;
            } else if x < 0.67 {
                green[i] = ((1.0 - (x - 0.33) / 0.34) * 255.0) as u8;
                red[i] = (((x - 0.33) / 0.34) * 255.0) as u8;
            } else {
                red[i] = 255;
                blue[i] = (((x - 0.67) / 0.33) * 255.0) as u8;
                green[i] = blue[i];
            }
        }

        Self { red, green, blue }
    }

    pub fn get_color(&self, index: usize) -> Color {
        let idx = index.min(255);
        Color::new(self.red[idx], self.green[idx], self.blue[idx])
    }
}

impl Default for ColorTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Window information
#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub id: i32,
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub visible: bool,
    pub xpos: i32,
    pub ypos: i32,
}

impl WindowInfo {
    pub fn new(id: i32) -> Self {
        Self {
            id,
            width: 640,
            height: 480,
            title: format!("XDL Graphics Window {}", id),
            visible: true,
            xpos: 100 + id * 50,
            ypos: 100 + id * 50,
        }
    }
}

/// Device types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceType {
    X,    // X11 window
    Win,  // Windows
    Mac,  // macOS
    Ps,   // PostScript
    Null, // No output
    Z,    // Z-buffer
    Png,  // PNG file
    Svg,  // SVG file
}

impl Default for DeviceType {
    fn default() -> Self {
        DeviceType::Png
    }
}

/// Main graphics state
pub struct GraphicsState {
    pub windows: HashMap<i32, WindowInfo>,
    pub current_window: i32,
    pub device: DeviceType,
    pub color_table: ColorTable,
    pub plot_style: PlotStyle,
    pub background_color: Color,
    pub char_size: f64,
    pub char_thick: f64,
    pub clip: Option<(f64, f64, f64, f64)>, // (x0, y0, x1, y1)
    pub xrange: Option<(f64, f64)>,
    pub yrange: Option<(f64, f64)>,
    pub zrange: Option<(f64, f64)>,
    pub position: Option<(f64, f64, f64, f64)>, // Normalized plot position
    pub t3d_matrix: Option<[[f64; 4]; 4]>,      // 3D transformation matrix
}

impl GraphicsState {
    pub fn new() -> Self {
        let mut windows = HashMap::new();
        windows.insert(0, WindowInfo::new(0));

        Self {
            windows,
            current_window: 0,
            device: DeviceType::default(),
            color_table: ColorTable::new(),
            plot_style: PlotStyle::default(),
            background_color: Color::new(255, 255, 255), // White
            char_size: 1.0,
            char_thick: 1.0,
            clip: None,
            xrange: None,
            yrange: None,
            zrange: None,
            position: None,
            t3d_matrix: None,
        }
    }

    pub fn create_window(&mut self, id: i32, width: u32, height: u32) {
        let mut win = WindowInfo::new(id);
        win.width = width;
        win.height = height;
        self.windows.insert(id, win);
    }

    pub fn set_current_window(&mut self, id: i32) -> bool {
        if self.windows.contains_key(&id) {
            self.current_window = id;
            true
        } else {
            false
        }
    }

    pub fn delete_window(&mut self, id: i32) -> bool {
        if id == 0 {
            return false; // Cannot delete window 0
        }
        self.windows.remove(&id).is_some()
    }

    pub fn get_current_window(&self) -> Option<&WindowInfo> {
        self.windows.get(&self.current_window)
    }

    pub fn load_color_table(&mut self, table_num: i32) {
        self.color_table = ColorTable::load_table(table_num);
    }

    pub fn reset_ranges(&mut self) {
        self.xrange = None;
        self.yrange = None;
        self.zrange = None;
    }
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self::new()
    }
}
