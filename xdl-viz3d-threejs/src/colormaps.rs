//! Colormap generation for volume rendering
//!
//! Provides standard scientific colormaps as RGB arrays

/// Generate colormap RGB values (256 entries)
pub fn generate_colormap(name: &str) -> Vec<[f32; 3]> {
    match name.to_uppercase().as_str() {
        "VIRIDIS" => viridis(),
        "RAINBOW" => rainbow(),
        "PLASMA" => plasma(),
        "INFERNO" => inferno(),
        "TURBO" => turbo(),
        "GRAYSCALE" | "GRAY" => grayscale(),
        _ => viridis(), // Default
    }
}

/// VIRIDIS colormap (perceptually uniform)
fn viridis() -> Vec<[f32; 3]> {
    let mut colors = Vec::with_capacity(256);
    for i in 0..256 {
        let t = i as f32 / 255.0;
        // Simplified viridis approximation
        let r = 0.267 + 0.735 * t;
        let g = 0.004 + 0.874 * t;
        let b = 0.329 - 0.096 * t + 0.534 * t * t;
        colors.push([r, g, b]);
    }
    colors
}

/// RAINBOW colormap (full spectrum)
fn rainbow() -> Vec<[f32; 3]> {
    let mut colors = Vec::with_capacity(256);
    for i in 0..256 {
        let t = i as f32 / 255.0;
        let h = t * 6.0; // Hue from 0 to 6
        let x = 1.0 - (h % 2.0 - 1.0).abs();

        let (r, g, b) = if h < 1.0 {
            (1.0, x, 0.0)
        } else if h < 2.0 {
            (x, 1.0, 0.0)
        } else if h < 3.0 {
            (0.0, 1.0, x)
        } else if h < 4.0 {
            (0.0, x, 1.0)
        } else if h < 5.0 {
            (x, 0.0, 1.0)
        } else {
            (1.0, 0.0, x)
        };

        colors.push([r, g, b]);
    }
    colors
}

/// PLASMA colormap (perceptually uniform, warm)
fn plasma() -> Vec<[f32; 3]> {
    let mut colors = Vec::with_capacity(256);
    for i in 0..256 {
        let t = i as f32 / 255.0;
        // Simplified plasma approximation
        let r = 0.050 + 0.950 * t;
        let g = 0.029 + 0.971 * (t * t);
        let b = 0.528 - 0.528 * t;
        colors.push([r, g, b]);
    }
    colors
}

/// INFERNO colormap (black to white through fire colors)
fn inferno() -> Vec<[f32; 3]> {
    let mut colors = Vec::with_capacity(256);
    for i in 0..256 {
        let t = i as f32 / 255.0;
        // Simplified inferno approximation
        let r = 0.001 + 0.999 * t;
        let g = if t < 0.5 {
            2.0 * t * t
        } else {
            1.0 - 2.0 * (1.0 - t) * (1.0 - t)
        };
        let b = if t < 0.25 {
            4.0 * t
        } else if t < 0.75 {
            1.0
        } else {
            1.0 - 4.0 * (t - 0.75)
        };
        colors.push([r, g, b]);
    }
    colors
}

/// TURBO colormap (vibrant rainbow)
fn turbo() -> Vec<[f32; 3]> {
    let mut colors = Vec::with_capacity(256);
    for i in 0..256 {
        let t = i as f32 / 255.0;
        // Simplified turbo approximation
        let r = (34.61 + t * (1172.33 - t * (10793.56 - t * 33300.12) + t * 34200.12)) / 255.0;
        let g = (23.31 + t * (557.33 + t * (1225.33 - t * 3574.96))) / 255.0;
        let b = (27.2 + t * (3211.1 - t * 15327.97 + t * 27814.0 - t * 22569.18)) / 255.0;
        colors.push([r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)]);
    }
    colors
}

/// GRAYSCALE colormap (black to white)
fn grayscale() -> Vec<[f32; 3]> {
    let mut colors = Vec::with_capacity(256);
    for i in 0..256 {
        let v = i as f32 / 255.0;
        colors.push([v, v, v]);
    }
    colors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colormap_generation() {
        let viridis = generate_colormap("VIRIDIS");
        assert_eq!(viridis.len(), 256);
        assert_eq!(viridis[0][0], 0.267); // First color

        let rainbow = generate_colormap("RAINBOW");
        assert_eq!(rainbow.len(), 256);

        let grayscale = generate_colormap("GRAYSCALE");
        assert_eq!(grayscale.len(), 256);
        assert_eq!(grayscale[0], [0.0, 0.0, 0.0]); // Black
        assert_eq!(grayscale[255], [1.0, 1.0, 1.0]); // White
    }
}
