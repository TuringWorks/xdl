//! Scientific color mapping for data visualization
//!
//! Provides perceptually uniform and colorblind-friendly color schemes
//! based on research in scientific visualization.

use super::state::Color;
use colorous;
use palette::{FromColor, Hsv, Srgb};

/// Color map types for scientific visualization
#[derive(Debug, Clone, PartialEq)]
pub enum ColorMapType {
    // Sequential - single hue
    Viridis,
    Plasma,
    Inferno,
    Magma,
    Cividis,
    
    // Sequential - multi-hue
    Turbo,
    Rainbow,
    Jet,
    
    // Diverging - two contrasting colors
    RdBu,      // Red-Blue
    BrBG,      // Brown-Blue-Green
    PiYG,      // Pink-Yellow-Green
    PRGn,      // Purple-Green
    RdYlBu,    // Red-Yellow-Blue
    
    // Qualitative
    Set1,
    Set2,
    Set3,
    Paired,
    
    // Terrain and Geography
    Terrain,
    Ocean,
    Topography,
    
    // Grayscale
    Greys,
    
    // Custom
    Custom(Vec<Color>),
}

impl Default for ColorMapType {
    fn default() -> Self {
        ColorMapType::Viridis
    }
}

/// Color map for mapping scalar values to colors
pub struct ColorMap {
    map_type: ColorMapType,
    custom_colors: Option<Vec<Color>>,
    reverse: bool,
}

impl ColorMap {
    /// Create a new color map
    pub fn new(map_type: ColorMapType) -> Self {
        Self {
            map_type,
            custom_colors: None,
            reverse: false,
        }
    }
    
    /// Create a custom color map from a list of colors
    pub fn custom(colors: Vec<Color>) -> Self {
        Self {
            map_type: ColorMapType::Custom(colors.clone()),
            custom_colors: Some(colors),
            reverse: false,
        }
    }
    
    /// Reverse the color map
    pub fn reversed(mut self) -> Self {
        self.reverse = true;
        self
    }
    
    /// Map a value in [0, 1] to a color
    pub fn map(&self, value: f64) -> Color {
        let t = if self.reverse {
            1.0 - value.clamp(0.0, 1.0)
        } else {
            value.clamp(0.0, 1.0)
        };
        
        match &self.map_type {
            ColorMapType::Viridis => self.colorous_map(&colorous::VIRIDIS, t),
            ColorMapType::Plasma => self.colorous_map(&colorous::PLASMA, t),
            ColorMapType::Inferno => self.colorous_map(&colorous::INFERNO, t),
            ColorMapType::Magma => self.colorous_map(&colorous::MAGMA, t),
            ColorMapType::Cividis => self.colorous_map(&colorous::CIVIDIS, t),
            ColorMapType::Turbo => self.colorous_map(&colorous::TURBO, t),
            ColorMapType::Rainbow => self.rainbow_map(t),
            ColorMapType::Jet => self.jet_map(t),
            ColorMapType::RdBu => self.colorous_map(&colorous::RED_BLUE, t),
            ColorMapType::BrBG => self.colorous_map(&colorous::BROWN_GREEN, t),
            ColorMapType::PiYG => self.colorous_map(&colorous::PINK_GREEN, t),
            ColorMapType::PRGn => self.colorous_map(&colorous::PURPLE_GREEN, t),
            ColorMapType::RdYlBu => self.colorous_map(&colorous::RED_YELLOW_BLUE, t),
            ColorMapType::Set1 => self.discrete_map(&colorous::SET1, t),
            ColorMapType::Set2 => self.discrete_map(&colorous::SET2, t),
            ColorMapType::Set3 => self.discrete_map(&colorous::SET3, t),
            ColorMapType::Paired => self.discrete_map(&colorous::PAIRED, t),
            ColorMapType::Terrain => self.terrain_map(t),
            ColorMapType::Ocean => self.ocean_map(t),
            ColorMapType::Topography => self.topography_map(t),
            ColorMapType::Greys => self.grey_map(t),
            ColorMapType::Custom(_) => self.custom_map(t),
        }
    }
    
    /// Map using colorous gradient
    fn colorous_map(&self, gradient: &colorous::Gradient, t: f64) -> Color {
        let color = gradient.eval_continuous(t);
        Color::new(color.r, color.g, color.b)
    }
    
    /// Rainbow color map (HSV-based)
    fn rainbow_map(&self, t: f64) -> Color {
        let hue = t * 300.0; // 0-300 degrees (blue to red)
        let hsv = Hsv::new(hue, 1.0, 1.0);
        let rgb: Srgb<f32> = Srgb::from_color(hsv).into_format();
        Color::new(
            (rgb.red * 255.0) as u8,
            (rgb.green * 255.0) as u8,
            (rgb.blue * 255.0) as u8,
        )
    }
    
    /// Map discrete color sets
    fn discrete_map(&self, colors: &[colorous::Color], t: f64) -> Color {
        let idx = (t * colors.len() as f64).floor() as usize;
        let idx = idx.min(colors.len() - 1);
        let c = colors[idx];
        Color::new(c.r, c.g, c.b)
    }
    
    /// Jet color map (classic but not perceptually uniform)
    fn jet_map(&self, t: f64) -> Color {
        let r = ((1.5 - 4.0 * (t - 0.75).abs()).clamp(0.0, 1.0) * 255.0) as u8;
        let g = ((1.5 - 4.0 * (t - 0.5).abs()).clamp(0.0, 1.0) * 255.0) as u8;
        let b = ((1.5 - 4.0 * (t - 0.25).abs()).clamp(0.0, 1.0) * 255.0) as u8;
        Color::new(r, g, b)
    }
    
    /// Terrain color map (for elevation data)
    fn terrain_map(&self, t: f64) -> Color {
        if t < 0.0 {
            // Below sea level - deep blue
            Color::new(0, 0, 128)
        } else if t < 0.2 {
            // Shallow water to shore
            self.interpolate_color(
                Color::new(0, 0, 128),
                Color::new(173, 216, 230),
                t / 0.2,
            )
        } else if t < 0.4 {
            // Shore to lowlands (green)
            self.interpolate_color(
                Color::new(173, 216, 230),
                Color::new(34, 139, 34),
                (t - 0.2) / 0.2,
            )
        } else if t < 0.7 {
            // Lowlands to highlands (brown)
            self.interpolate_color(
                Color::new(34, 139, 34),
                Color::new(139, 90, 43),
                (t - 0.4) / 0.3,
            )
        } else {
            // High mountains (white/snow)
            self.interpolate_color(
                Color::new(139, 90, 43),
                Color::new(255, 255, 255),
                (t - 0.7) / 0.3,
            )
        }
    }
    
    /// Ocean color map (for bathymetry)
    fn ocean_map(&self, t: f64) -> Color {
        let dark_blue = Color::new(0, 0, 80);
        let light_blue = Color::new(100, 180, 255);
        self.interpolate_color(dark_blue, light_blue, t)
    }
    
    /// Topography color map (combined terrain + ocean)
    fn topography_map(&self, t: f64) -> Color {
        if t < 0.5 {
            // Ocean depths
            self.ocean_map(t * 2.0)
        } else {
            // Land elevations
            self.terrain_map((t - 0.5) * 2.0)
        }
    }
    
    /// Grayscale color map
    fn grey_map(&self, t: f64) -> Color {
        let intensity = (t * 255.0) as u8;
        Color::new(intensity, intensity, intensity)
    }
    
    /// Custom color map with interpolation
    fn custom_map(&self, t: f64) -> Color {
        if let Some(colors) = &self.custom_colors {
            if colors.is_empty() {
                return Color::new(0, 0, 0);
            }
            if colors.len() == 1 {
                return colors[0];
            }
            
            let scaled = t * (colors.len() - 1) as f64;
            let idx = scaled.floor() as usize;
            let frac = scaled - idx as f64;
            
            if idx >= colors.len() - 1 {
                return colors[colors.len() - 1];
            }
            
            self.interpolate_color(colors[idx], colors[idx + 1], frac)
        } else {
            Color::new(0, 0, 0)
        }
    }
    
    /// Linear interpolation between two colors
    fn interpolate_color(&self, c1: Color, c2: Color, t: f64) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color::new(
            (c1.r as f64 * (1.0 - t) + c2.r as f64 * t) as u8,
            (c1.g as f64 * (1.0 - t) + c2.g as f64 * t) as u8,
            (c1.b as f64 * (1.0 - t) + c2.b as f64 * t) as u8,
        )
    }
    
    /// Generate a discrete color table with n colors
    pub fn generate_table(&self, n: usize) -> Vec<Color> {
        (0..n)
            .map(|i| {
                let t = if n > 1 {
                    i as f64 / (n - 1) as f64
                } else {
                    0.5
                };
                self.map(t)
            })
            .collect()
    }
}

/// Helper function to create common color maps
pub fn viridis() -> ColorMap {
    ColorMap::new(ColorMapType::Viridis)
}

pub fn plasma() -> ColorMap {
    ColorMap::new(ColorMapType::Plasma)
}

pub fn inferno() -> ColorMap {
    ColorMap::new(ColorMapType::Inferno)
}

pub fn turbo() -> ColorMap {
    ColorMap::new(ColorMapType::Turbo)
}

pub fn terrain() -> ColorMap {
    ColorMap::new(ColorMapType::Terrain)
}

pub fn ocean() -> ColorMap {
    ColorMap::new(ColorMapType::Ocean)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_viridis_range() {
        let cmap = viridis();
        let c0 = cmap.map(0.0);
        let c1 = cmap.map(1.0);
        
        // Viridis starts dark purple, ends bright yellow
        assert!(c0.r < 100);
        assert!(c1.r > 200);
    }
    
    #[test]
    fn test_custom_colormap() {
        let colors = vec![
            Color::new(255, 0, 0),
            Color::new(0, 255, 0),
            Color::new(0, 0, 255),
        ];
        let cmap = ColorMap::custom(colors);
        
        let c0 = cmap.map(0.0);
        let c_mid = cmap.map(0.5);
        let c1 = cmap.map(1.0);
        
        assert_eq!(c0, Color::new(255, 0, 0));
        assert_eq!(c_mid, Color::new(0, 255, 0));
        assert_eq!(c1, Color::new(0, 0, 255));
    }
    
    #[test]
    fn test_terrain_map() {
        let cmap = terrain();
        let ocean = cmap.map(0.1);
        let land = cmap.map(0.5);
        let mountain = cmap.map(0.9);
        
        // Ocean should be blue
        assert!(ocean.b > ocean.r);
        // Mountains should be light
        assert!(mountain.r > 200);
    }
}
