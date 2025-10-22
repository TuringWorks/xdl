//! 3D plotting and contour implementation

use super::plot2d::Plot2DConfig;
use super::state::{Color, GRAPHICS_STATE};
use plotters::prelude::*;
use xdl_core::{XdlError, XdlResult};

/// Contour plot configuration
pub struct ContourConfig {
    pub levels: Option<Vec<f64>>,
    pub nlevels: usize,
    pub fill: bool,
    pub color_table: Option<Vec<Color>>,
    pub config: Plot2DConfig,
}

impl Default for ContourConfig {
    fn default() -> Self {
        Self {
            levels: None,
            nlevels: 10,
            fill: false,
            color_table: None,
            config: Plot2DConfig::default(),
        }
    }
}

/// Create contour plot from 2D data
pub fn contour_plot(
    z_data: Vec<Vec<f64>>,
    x_coords: Option<Vec<f64>>,
    y_coords: Option<Vec<f64>>,
    config: ContourConfig,
    filename: &str,
) -> XdlResult<()> {
    let height = z_data.len();
    let width = if height > 0 { z_data[0].len() } else { 0 };

    if width == 0 || height == 0 {
        return Err(XdlError::InvalidArgument(
            "Contour data must be non-empty".to_string(),
        ));
    }

    // Generate x and y coordinates if not provided
    let x = x_coords.unwrap_or_else(|| (0..width).map(|i| i as f64).collect());
    let y = y_coords.unwrap_or_else(|| (0..height).map(|i| i as f64).collect());

    // Find data range
    let z_min = z_data
        .iter()
        .flat_map(|row| row.iter())
        .fold(f64::INFINITY, |a, &b| a.min(b));
    let z_max = z_data
        .iter()
        .flat_map(|row| row.iter())
        .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

    // Calculate contour levels
    let levels = config.levels.unwrap_or_else(|| {
        (0..config.nlevels)
            .map(|i| z_min + (z_max - z_min) * i as f64 / (config.nlevels - 1) as f64)
            .collect()
    });

    // Get window dimensions
    let (win_width, win_height) = {
        let state = GRAPHICS_STATE.lock().unwrap();
        let win = state.get_current_window().unwrap();
        (win.width, win.height)
    };

    // Create drawing area
    let root = BitMapBackend::new(filename, (win_width, win_height)).into_drawing_area();
    root.fill(&config.config.background.to_rgb())?;

    let x_min = *x.first().unwrap();
    let x_max = *x.last().unwrap();
    let y_min = *y.first().unwrap();
    let y_max = *y.last().unwrap();

    let mut chart = ChartBuilder::on(&root)
        .caption(
            config.config.title.as_deref().unwrap_or("Contour Plot"),
            ("sans-serif", 30),
        )
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

    chart.configure_mesh().draw()?;

    // Draw filled contours or contour lines
    if config.fill {
        // Filled contours - draw rectangles colored by value
        for (i, row) in z_data.iter().enumerate() {
            for (j, &z_val) in row.iter().enumerate() {
                if i + 1 < height && j + 1 < width {
                    let color_idx = ((z_val - z_min) / (z_max - z_min) * 255.0) as u8;
                    let color = RGBColor(color_idx, 100, 255 - color_idx);

                    let x0 = x[j];
                    let x1 = x[j + 1];
                    let y0 = y[i];
                    let y1 = y[i + 1];

                    chart.draw_series(std::iter::once(Rectangle::new(
                        [(x0, y0), (x1, y1)],
                        ShapeStyle::from(&color).filled(),
                    )))?;
                }
            }
        }
    } else {
        // Contour lines using marching squares algorithm (simplified)
        for level in levels {
            let contour_color = RGBColor(0, 0, 255);
            // Simplified: just mark points near the contour level
            for (i, row) in z_data.iter().enumerate() {
                for (j, &z_val) in row.iter().enumerate() {
                    if (z_val - level).abs() < (z_max - z_min) * 0.05 {
                        chart.draw_series(std::iter::once(Circle::new(
                            (x[j], y[i]),
                            2,
                            ShapeStyle::from(&contour_color).filled(),
                        )))?;
                    }
                }
            }
        }
    }

    root.present()?;
    Ok(())
}

/// 3D surface plot configuration
pub struct SurfaceConfig {
    pub ax: f64, // X rotation angle
    pub az: f64, // Z rotation angle
    pub shading: bool,
    pub config: Plot2DConfig,
}

impl Default for SurfaceConfig {
    fn default() -> Self {
        Self {
            ax: 30.0,
            az: 30.0,
            shading: true,
            config: Plot2DConfig::default(),
        }
    }
}

/// Create 3D surface plot
pub fn surface_plot(
    z_data: Vec<Vec<f64>>,
    x_coords: Option<Vec<f64>>,
    y_coords: Option<Vec<f64>>,
    config: SurfaceConfig,
    filename: &str,
) -> XdlResult<()> {
    let height = z_data.len();
    let width = if height > 0 { z_data[0].len() } else { 0 };

    if width == 0 || height == 0 {
        return Err(XdlError::InvalidArgument(
            "Surface data must be non-empty".to_string(),
        ));
    }

    // Generate coordinates if not provided
    let x = x_coords.unwrap_or_else(|| (0..width).map(|i| i as f64).collect());
    let y = y_coords.unwrap_or_else(|| (0..height).map(|i| i as f64).collect());

    // Find Z range
    let z_min = z_data
        .iter()
        .flat_map(|row| row.iter())
        .fold(f64::INFINITY, |a, &b| a.min(b));
    let z_max = z_data
        .iter()
        .flat_map(|row| row.iter())
        .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

    // Get window dimensions
    let (win_width, win_height) = {
        let state = GRAPHICS_STATE.lock().unwrap();
        let win = state.get_current_window().unwrap();
        (win.width, win.height)
    };

    // Create drawing area
    let root = BitMapBackend::new(filename, (win_width, win_height)).into_drawing_area();
    root.fill(&config.config.background.to_rgb())?;

    // Build 3D chart context
    let x_min = *x.first().unwrap();
    let x_max = *x.last().unwrap();
    let y_min = *y.first().unwrap();
    let y_max = *y.last().unwrap();

    let mut chart = ChartBuilder::on(&root)
        .caption(
            config.config.title.as_deref().unwrap_or("3D Surface"),
            ("sans-serif", 30),
        )
        .margin(20)
        .build_cartesian_3d(x_min..x_max, z_min..z_max, y_min..y_max)?;

    chart.with_projection(|mut pb| {
        pb.yaw = config.az * std::f64::consts::PI / 180.0;
        pb.pitch = config.ax * std::f64::consts::PI / 180.0;
        pb.into_matrix()
    });

    chart.configure_axes().draw()?;

    // Draw surface as a mesh
    for i in 0..height - 1 {
        for j in 0..width - 1 {
            let z00 = z_data[i][j];
            let z01 = z_data[i][j + 1];
            let z10 = z_data[i + 1][j];
            let z11 = z_data[i + 1][j + 1];

            // Color by height
            let avg_z = (z00 + z01 + z10 + z11) / 4.0;
            let color_val = ((avg_z - z_min) / (z_max - z_min) * 200.0) as u8;
            let color = RGBColor(color_val, 100, 255 - color_val);

            // Draw two triangles to form a quad
            let poly_style = ShapeStyle::from(&color).filled();
            chart.draw_series(std::iter::once(Polygon::new(
                vec![
                    (x[j], z00, y[i]),
                    (x[j + 1], z01, y[i]),
                    (x[j], z10, y[i + 1]),
                ],
                poly_style,
            )))?;

            chart.draw_series(std::iter::once(Polygon::new(
                vec![
                    (x[j + 1], z01, y[i]),
                    (x[j + 1], z11, y[i + 1]),
                    (x[j], z10, y[i + 1]),
                ],
                poly_style,
            )))?;

            // Draw wireframe if not shaded
            if !config.shading {
                chart.draw_series(std::iter::once(PathElement::new(
                    vec![
                        (x[j], z00, y[i]),
                        (x[j + 1], z01, y[i]),
                        (x[j + 1], z11, y[i + 1]),
                        (x[j], z10, y[i + 1]),
                        (x[j], z00, y[i]),
                    ],
                    BLACK,
                )))?;
            }
        }
    }

    root.present()?;
    Ok(())
}

/// Create simple 3D line plot
pub fn plot_3d(
    x_data: Vec<f64>,
    y_data: Vec<f64>,
    z_data: Vec<f64>,
    config: SurfaceConfig,
    filename: &str,
) -> XdlResult<()> {
    if x_data.len() != y_data.len() || y_data.len() != z_data.len() {
        return Err(XdlError::InvalidArgument(
            "X, Y, and Z arrays must have the same length".to_string(),
        ));
    }

    let x_min = x_data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let x_max = x_data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let y_min = y_data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let y_max = y_data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let z_min = z_data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let z_max = z_data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

    let (win_width, win_height) = {
        let state = GRAPHICS_STATE.lock().unwrap();
        let win = state.get_current_window().unwrap();
        (win.width, win.height)
    };

    let root = BitMapBackend::new(filename, (win_width, win_height)).into_drawing_area();
    root.fill(&config.config.background.to_rgb())?;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            config.config.title.as_deref().unwrap_or("3D Plot"),
            ("sans-serif", 30),
        )
        .margin(20)
        .build_cartesian_3d(x_min..x_max, z_min..z_max, y_min..y_max)?;

    chart.with_projection(|mut pb| {
        pb.yaw = config.az * std::f64::consts::PI / 180.0;
        pb.pitch = config.ax * std::f64::consts::PI / 180.0;
        pb.into_matrix()
    });

    chart.configure_axes().draw()?;

    // Draw 3D line
    let points: Vec<(f64, f64, f64)> = x_data
        .iter()
        .zip(z_data.iter())
        .zip(y_data.iter())
        .map(|((&x, &z), &y)| (x, z, y))
        .collect();

    chart.draw_series(LineSeries::new(points, &BLUE))?;

    root.present()?;
    Ok(())
}
