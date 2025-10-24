//! Scientific colormaps for volume visualization

/// Available colormap presets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Colormap {
    Rainbow,
    Viridis,
    Plasma,
    Inferno,
    Turbo,
    Grayscale,
}

impl Colormap {
    /// Parse colormap from string
    #[allow(clippy::should_implement_trait)] // Simplified version, not implementing full FromStr trait
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "RAINBOW" => Some(Self::Rainbow),
            "VIRIDIS" => Some(Self::Viridis),
            "PLASMA" => Some(Self::Plasma),
            "INFERNO" => Some(Self::Inferno),
            "TURBO" => Some(Self::Turbo),
            "GRAYSCALE" | "GRAY" => Some(Self::Grayscale),
            _ => None,
        }
    }

    /// Generate colormap lookup table (256 RGBA values)
    pub fn generate_lut(&self) -> Vec<[u8; 4]> {
        let mut lut = Vec::with_capacity(256);

        for i in 0..256 {
            let t = i as f32 / 255.0;
            let color = match self {
                Self::Rainbow => rainbow(t),
                Self::Viridis => viridis(t),
                Self::Plasma => plasma(t),
                Self::Inferno => inferno(t),
                Self::Turbo => turbo(t),
                Self::Grayscale => grayscale(t),
            };
            lut.push(color);
        }

        lut
    }
}

// Colormap implementations
// TODO: Replace these with proper perceptually-uniform colormaps

fn rainbow(t: f32) -> [u8; 4] {
    // Simple rainbow (not perceptually uniform, but colorful)
    let r = (255.0 * (t * 5.0).sin().abs()) as u8;
    let g = (255.0 * ((t * 5.0 + 2.0).sin().abs())) as u8;
    let b = (255.0 * ((t * 5.0 + 4.0).sin().abs())) as u8;
    [r, g, b, 255]
}

fn viridis(t: f32) -> [u8; 4] {
    // Simplified Viridis approximation
    // TODO: Use actual Viridis color values
    let r = (255.0 * (0.267 + 0.005 * t)) as u8;
    let g = (255.0 * (0.005 + 0.55 * t)) as u8;
    let b = (255.0 * (0.329 + 0.5 * t)) as u8;
    [r, g, b, 255]
}

fn plasma(t: f32) -> [u8; 4] {
    // Simplified Plasma approximation
    let r = (255.0 * (0.5 + 0.5 * t)) as u8;
    let g = (255.0 * (0.1 + 0.4 * t)) as u8;
    let b = (255.0 * (0.8 - 0.3 * t)) as u8;
    [r, g, b, 255]
}

fn inferno(t: f32) -> [u8; 4] {
    // Simplified Inferno approximation
    let r = (255.0 * t.powf(0.5)) as u8;
    let g = (255.0 * t.powf(1.5)) as u8;
    let b = (255.0 * t.powf(3.0)) as u8;
    [r, g, b, 255]
}

fn turbo(t: f32) -> [u8; 4] {
    // Simplified Turbo approximation
    let r = (255.0 * (0.13 + 0.87 * (1.0 - (1.0 - t).powf(2.0)))) as u8;
    let g = (255.0 * (0.09 + 0.91 * (4.0 * t * (1.0 - t)).powf(0.5))) as u8;
    let b = (255.0 * (0.14 + 0.86 * (1.0 - t.powf(2.0)))) as u8;
    [r, g, b, 255]
}

fn grayscale(t: f32) -> [u8; 4] {
    let v = (255.0 * t) as u8;
    [v, v, v, 255]
}
