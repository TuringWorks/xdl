//! Terrain and elevation visualization
//!
//! Digital Elevation Model (DEM) rendering, hillshade generation,
//! and 3D terrain visualization with texture mapping.

use super::colormap::ColorMap;
use super::state::GRAPHICS_STATE;
use plotters::prelude::*;
use xdl_core::{XdlError, XdlResult};

/// Digital Elevation Model data structure
pub struct DigitalElevationModel {
    elevations: Vec<Vec<f64>>,
    width: usize,
    height: usize,
    min_elevation: f64,
    max_elevation: f64,
    cell_size: f64, // meters per cell
}

impl DigitalElevationModel {
    /// Create a DEM from elevation data
    pub fn new(elevations: Vec<Vec<f64>>, cell_size: f64) -> XdlResult<Self> {
        if elevations.is_empty() || elevations[0].is_empty() {
            return Err(XdlError::InvalidArgument(
                "Elevation data cannot be empty".to_string(),
            ));
        }

        let height = elevations.len();
        let width = elevations[0].len();

        let mut min_elev = f64::INFINITY;
        let mut max_elev = f64::NEG_INFINITY;

        for row in &elevations {
            if row.len() != width {
                return Err(XdlError::InvalidArgument(
                    "All rows must have the same width".to_string(),
                ));
            }
            for &elev in row {
                min_elev = min_elev.min(elev);
                max_elev = max_elev.max(elev);
            }
        }

        Ok(Self {
            elevations,
            width,
            height,
            min_elevation: min_elev,
            max_elevation: max_elev,
            cell_size,
        })
    }

    /// Get elevation at grid position
    pub fn get_elevation(&self, x: usize, y: usize) -> Option<f64> {
        if y < self.height && x < self.width {
            Some(self.elevations[y][x])
        } else {
            None
        }
    }

    /// Get normalized elevation [0, 1]
    pub fn get_normalized(&self, x: usize, y: usize) -> Option<f64> {
        self.get_elevation(x, y).map(|elev| {
            if self.max_elevation > self.min_elevation {
                (elev - self.min_elevation) / (self.max_elevation - self.min_elevation)
            } else {
                0.5
            }
        })
    }

    /// Calculate slope at a position (rise/run)
    pub fn calculate_slope(&self, x: usize, y: usize) -> f64 {
        if x == 0 || y == 0 || x >= self.width - 1 || y >= self.height - 1 {
            return 0.0;
        }

        let _z = self.elevations[y][x];
        let z_right = self.elevations[y][x + 1];
        let z_left = self.elevations[y][x - 1];
        let z_up = self.elevations[y - 1][x];
        let z_down = self.elevations[y + 1][x];

        let dz_dx = (z_right - z_left) / (2.0 * self.cell_size);
        let dz_dy = (z_down - z_up) / (2.0 * self.cell_size);

        (dz_dx * dz_dx + dz_dy * dz_dy).sqrt().atan()
    }

    /// Calculate aspect (direction of slope) in radians [0, 2π]
    pub fn calculate_aspect(&self, x: usize, y: usize) -> f64 {
        if x == 0 || y == 0 || x >= self.width - 1 || y >= self.height - 1 {
            return 0.0;
        }

        let z_right = self.elevations[y][x + 1];
        let z_left = self.elevations[y][x - 1];
        let z_up = self.elevations[y - 1][x];
        let z_down = self.elevations[y + 1][x];

        let dz_dx = (z_right - z_left) / (2.0 * self.cell_size);
        let dz_dy = (z_down - z_up) / (2.0 * self.cell_size);

        dz_dy.atan2(dz_dx)
    }

    /// Generate hillshade (shaded relief) with given sun angle
    /// azimuth: sun direction in degrees (0=N, 90=E, 180=S, 270=W)
    /// altitude: sun elevation angle in degrees (0=horizon, 90=overhead)
    pub fn generate_hillshade(&self, azimuth: f64, altitude: f64) -> Vec<Vec<f64>> {
        let az_rad = azimuth.to_radians();
        let alt_rad = altitude.to_radians();

        let mut hillshade = vec![vec![0.0; self.width]; self.height];

        for (y, row) in hillshade.iter_mut().enumerate().take(self.height) {
            for (x, item) in row.iter_mut().enumerate().take(self.width) {
                let slope = self.calculate_slope(x, y);
                let aspect = self.calculate_aspect(x, y);

                // Hillshade formula
                let shade = ((alt_rad.cos() * slope.cos())
                    + (alt_rad.sin() * slope.sin() * (az_rad - aspect).cos()))
                .max(0.0);

                *item = shade;
            }
        }

        hillshade
    }
}

/// Render DEM as colored elevation map
pub fn render_elevation_map(
    dem: &DigitalElevationModel,
    colormap: &ColorMap,
    filename: &str,
) -> XdlResult<()> {
    let (width, height) = {
        let state = GRAPHICS_STATE.lock().unwrap();
        let win = state.get_current_window().unwrap();
        (win.width, win.height)
    };

    let root = BitMapBackend::new(filename, (width, height)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Elevation Map", ("sans-serif", 30))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(0.0..dem.width as f64, 0.0..dem.height as f64)?;

    chart.configure_mesh().draw()?;

    // Draw elevation as colored pixels
    for y in 0..dem.height {
        for x in 0..dem.width {
            if let Some(norm_elev) = dem.get_normalized(x, y) {
                let color = colormap.map(norm_elev);
                let rgb = RGBColor(color.r, color.g, color.b);

                chart.draw_series(std::iter::once(Rectangle::new(
                    [(x as f64, y as f64), (x as f64 + 1.0, y as f64 + 1.0)],
                    ShapeStyle::from(&rgb).filled(),
                )))?;
            }
        }
    }

    root.present()?;
    Ok(())
}

/// Render hillshade visualization
pub fn render_hillshade(
    dem: &DigitalElevationModel,
    azimuth: f64,
    altitude: f64,
    filename: &str,
) -> XdlResult<()> {
    let hillshade = dem.generate_hillshade(azimuth, altitude);

    let (width, height) = {
        let state = GRAPHICS_STATE.lock().unwrap();
        let win = state.get_current_window().unwrap();
        (win.width, win.height)
    };

    let root = BitMapBackend::new(filename, (width, height)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Hillshade", ("sans-serif", 30))
        .margin(20)
        .build_cartesian_2d(0.0..dem.width as f64, 0.0..dem.height as f64)?;

    // Draw hillshade as grayscale
    for (y, row) in hillshade.iter().enumerate().take(dem.height) {
        for (x, &shade) in row.iter().enumerate().take(dem.width) {
            let intensity = (shade * 255.0) as u8;
            let color = RGBColor(intensity, intensity, intensity);

            chart.draw_series(std::iter::once(Rectangle::new(
                [(x as f64, y as f64), (x as f64 + 1.0, y as f64 + 1.0)],
                ShapeStyle::from(&color).filled(),
            )))?;
        }
    }

    root.present()?;
    Ok(())
}

/// Render combined elevation + hillshade
pub fn render_shaded_relief(
    dem: &DigitalElevationModel,
    colormap: &ColorMap,
    azimuth: f64,
    altitude: f64,
    blend_factor: f64,
    filename: &str,
) -> XdlResult<()> {
    let hillshade = dem.generate_hillshade(azimuth, altitude);

    let (width, height) = {
        let state = GRAPHICS_STATE.lock().unwrap();
        let win = state.get_current_window().unwrap();
        (win.width, win.height)
    };

    let root = BitMapBackend::new(filename, (width, height)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Shaded Relief", ("sans-serif", 30))
        .margin(20)
        .build_cartesian_2d(0.0..dem.width as f64, 0.0..dem.height as f64)?;

    // Blend elevation color with hillshade
    for (y, row) in hillshade.iter().enumerate().take(dem.height) {
        for (x, &shade) in row.iter().enumerate().take(dem.width) {
            if let Some(norm_elev) = dem.get_normalized(x, y) {
                let base_color = colormap.map(norm_elev);

                // Blend color with hillshade
                let r = (base_color.r as f64 * ((1.0 - blend_factor) + blend_factor * shade)) as u8;
                let g = (base_color.g as f64 * ((1.0 - blend_factor) + blend_factor * shade)) as u8;
                let b = (base_color.b as f64 * ((1.0 - blend_factor) + blend_factor * shade)) as u8;

                let color = RGBColor(r, g, b);

                chart.draw_series(std::iter::once(Rectangle::new(
                    [(x as f64, y as f64), (x as f64 + 1.0, y as f64 + 1.0)],
                    ShapeStyle::from(&color).filled(),
                )))?;
            }
        }
    }

    root.present()?;
    Ok(())
}

/// Generate contour lines from DEM
pub fn generate_contours(
    dem: &DigitalElevationModel,
    levels: &[f64],
    filename: &str,
) -> XdlResult<()> {
    let (width, height) = {
        let state = GRAPHICS_STATE.lock().unwrap();
        let win = state.get_current_window().unwrap();
        (win.width, win.height)
    };

    let root = BitMapBackend::new(filename, (width, height)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Contour Map", ("sans-serif", 30))
        .margin(20)
        .build_cartesian_2d(0.0..dem.width as f64, 0.0..dem.height as f64)?;

    chart.configure_mesh().draw()?;

    // Simple contour detection using marching squares concept
    for &level in levels {
        for y in 0..dem.height - 1 {
            for x in 0..dem.width - 1 {
                // Get elevations of cell corners
                let z00 = dem.get_elevation(x, y);
                let z10 = dem.get_elevation(x + 1, y);
                let z01 = dem.get_elevation(x, y + 1);
                let z11 = dem.get_elevation(x + 1, y + 1);

                if let (Some(z00), Some(z10), Some(z01), Some(z11)) = (z00, z10, z01, z11) {
                    // Check if contour passes through this cell
                    let min_z = z00.min(z10).min(z01).min(z11);
                    let max_z = z00.max(z10).max(z01).max(z11);

                    if level >= min_z && level <= max_z {
                        // Draw a marker (simplified - full marching squares would interpolate edges)
                        chart.draw_series(std::iter::once(Circle::new(
                            (x as f64 + 0.5, y as f64 + 0.5),
                            2,
                            ShapeStyle::from(&BLUE).filled(),
                        )))?;
                    }
                }
            }
        }
    }

    root.present()?;
    Ok(())
}

/// Render 3D terrain surface
pub fn render_terrain_3d(
    dem: &DigitalElevationModel,
    colormap: &ColorMap,
    azimuth: f64,
    elevation_angle: f64,
    vertical_exaggeration: f64,
    filename: &str,
) -> XdlResult<()> {
    let (width, height) = {
        let state = GRAPHICS_STATE.lock().unwrap();
        let win = state.get_current_window().unwrap();
        (win.width, win.height)
    };

    let root = BitMapBackend::new(filename, (width, height)).into_drawing_area();
    root.fill(&WHITE)?;

    let x_range = 0.0..dem.width as f64;
    let y_range = 0.0..dem.height as f64;
    let z_range = dem.min_elevation..(dem.max_elevation * vertical_exaggeration);

    let mut chart = ChartBuilder::on(&root)
        .caption("3D Terrain", ("sans-serif", 30))
        .margin(20)
        .build_cartesian_3d(x_range, z_range, y_range)?;

    chart.with_projection(|mut pb| {
        pb.yaw = azimuth.to_radians();
        pb.pitch = elevation_angle.to_radians();
        pb.into_matrix()
    });

    chart.configure_axes().draw()?;

    // Draw terrain surface as triangles
    for y in 0..dem.height - 1 {
        for x in 0..dem.width - 1 {
            if let (Some(z00), Some(z10), Some(z01), Some(z11)) = (
                dem.get_elevation(x, y),
                dem.get_elevation(x + 1, y),
                dem.get_elevation(x, y + 1),
                dem.get_elevation(x + 1, y + 1),
            ) {
                let z00_scaled = z00 * vertical_exaggeration;
                let z10_scaled = z10 * vertical_exaggeration;
                let z01_scaled = z01 * vertical_exaggeration;
                let z11_scaled = z11 * vertical_exaggeration;

                // Color by average elevation
                let avg_z = (z00 + z10 + z01 + z11) / 4.0;
                let norm_z = if dem.max_elevation > dem.min_elevation {
                    (avg_z - dem.min_elevation) / (dem.max_elevation - dem.min_elevation)
                } else {
                    0.5
                };
                let color = colormap.map(norm_z);
                let rgb = RGBColor(color.r, color.g, color.b);

                // Draw two triangles
                chart.draw_series(std::iter::once(Polygon::new(
                    vec![
                        (x as f64, z00_scaled, y as f64),
                        (x as f64 + 1.0, z10_scaled, y as f64),
                        (x as f64, z01_scaled, y as f64 + 1.0),
                    ],
                    ShapeStyle::from(&rgb).filled(),
                )))?;

                chart.draw_series(std::iter::once(Polygon::new(
                    vec![
                        (x as f64 + 1.0, z10_scaled, y as f64),
                        (x as f64 + 1.0, z11_scaled, y as f64 + 1.0),
                        (x as f64, z01_scaled, y as f64 + 1.0),
                    ],
                    ShapeStyle::from(&rgb).filled(),
                )))?;
            }
        }
    }

    root.present()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dem_creation() {
        let elevations = vec![
            vec![100.0, 110.0, 120.0],
            vec![105.0, 115.0, 125.0],
            vec![110.0, 120.0, 130.0],
        ];

        let dem = DigitalElevationModel::new(elevations, 30.0).unwrap();
        assert_eq!(dem.width, 3);
        assert_eq!(dem.height, 3);
        assert_eq!(dem.min_elevation, 100.0);
        assert_eq!(dem.max_elevation, 130.0);
    }

    #[test]
    fn test_hillshade_generation() {
        let elevations = vec![
            vec![100.0, 110.0, 120.0],
            vec![105.0, 115.0, 125.0],
            vec![110.0, 120.0, 130.0],
        ];

        let dem = DigitalElevationModel::new(elevations, 30.0).unwrap();
        let hillshade = dem.generate_hillshade(315.0, 45.0);

        assert_eq!(hillshade.len(), 3);
        assert_eq!(hillshade[0].len(), 3);
    }
}
