//! Advanced scientific visualization
//!
//! Vector fields, streamlines, volume rendering, isosurfaces,
//! and particle systems for scientific data visualization.

use super::colormap::ColorMap;
use super::state::GRAPHICS_STATE;
use plotters::prelude::*;
use xdl_core::{XdlError, XdlResult};

/// 3D vector field
pub struct VectorField3D {
    pub u: Vec<Vec<Vec<f64>>>, // X component
    pub v: Vec<Vec<Vec<f64>>>, // Y component
    pub w: Vec<Vec<Vec<f64>>>, // Z component
    pub dims: (usize, usize, usize),
}

impl VectorField3D {
    pub fn new(
        u: Vec<Vec<Vec<f64>>>,
        v: Vec<Vec<Vec<f64>>>,
        w: Vec<Vec<Vec<f64>>>,
    ) -> XdlResult<Self> {
        let dims = (u.len(), u[0].len(), u[0][0].len());
        
        // Validate dimensions
        for layer in &u {
            if layer.len() != dims.1 {
                return Err(XdlError::InvalidArgument("Inconsistent dimensions".to_string()));
            }
            for row in layer {
                if row.len() != dims.2 {
                    return Err(XdlError::InvalidArgument("Inconsistent dimensions".to_string()));
                }
            }
        }
        
        Ok(Self { u, v, w, dims })
    }
    
    /// Get vector at position
    pub fn get_vector(&self, i: usize, j: usize, k: usize) -> Option<(f64, f64, f64)> {
        if i < self.dims.0 && j < self.dims.1 && k < self.dims.2 {
            Some((self.u[i][j][k], self.v[i][j][k], self.w[i][j][k]))
        } else {
            None
        }
    }
    
    /// Get magnitude at position
    pub fn get_magnitude(&self, i: usize, j: usize, k: usize) -> Option<f64> {
        self.get_vector(i, j, k)
            .map(|(u, v, w)| (u * u + v * v + w * w).sqrt())
    }
}

/// 2D vector field
pub struct VectorField2D {
    pub u: Vec<Vec<f64>>, // X component
    pub v: Vec<Vec<f64>>, // Y component
    pub dims: (usize, usize),
}

impl VectorField2D {
    pub fn new(u: Vec<Vec<f64>>, v: Vec<Vec<f64>>) -> XdlResult<Self> {
        let dims = (u.len(), u[0].len());
        
        if v.len() != dims.0 || v[0].len() != dims.1 {
            return Err(XdlError::InvalidArgument(
                "U and V components must have same dimensions".to_string(),
            ));
        }
        
        Ok(Self { u, v, dims })
    }
    
    pub fn get_vector(&self, i: usize, j: usize) -> Option<(f64, f64)> {
        if i < self.dims.0 && j < self.dims.1 {
            Some((self.u[i][j], self.v[i][j]))
        } else {
            None
        }
    }
    
    pub fn get_magnitude(&self, i: usize, j: usize) -> Option<f64> {
        self.get_vector(i, j)
            .map(|(u, v)| (u * u + v * v).sqrt())
    }
}

/// Render 2D vector field as arrows (quiver plot)
pub fn render_quiver(
    field: &VectorField2D,
    subsample: usize,
    scale: f64,
    colormap: Option<&ColorMap>,
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
        .caption("Vector Field", ("sans-serif", 30))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(0.0..field.dims.1 as f64, 0.0..field.dims.0 as f64)?;
    
    chart.configure_mesh().draw()?;
    
    // Find magnitude range for coloring
    let mut max_mag: f64 = 0.0;
    for i in (0..field.dims.0).step_by(subsample) {
        for j in (0..field.dims.1).step_by(subsample) {
            if let Some(mag) = field.get_magnitude(i, j) {
                max_mag = max_mag.max(mag);
            }
        }
    }
    
    // Draw arrows
    for i in (0..field.dims.0).step_by(subsample) {
        for j in (0..field.dims.1).step_by(subsample) {
            if let Some((u, v)) = field.get_vector(i, j) {
                let mag = (u * u + v * v).sqrt();
                if mag > 1e-10 {
                    let x0 = j as f64 + 0.5;
                    let y0 = i as f64 + 0.5;
                    let x1 = x0 + u * scale;
                    let y1 = y0 + v * scale;
                    
                    // Color by magnitude
                    let color = if let Some(cmap) = colormap {
                        let t = if max_mag > 0.0 { mag / max_mag } else { 0.5 };
                        let c = cmap.map(t);
                        RGBColor(c.r, c.g, c.b)
                    } else {
                        BLUE
                    };
                    
                    // Draw arrow line
                    chart.draw_series(std::iter::once(PathElement::new(
                        vec![(x0, y0), (x1, y1)],
                        &color,
                    )))?;
                    
                    // Draw arrowhead (simplified)
                    let arrow_len = 0.3;
                    let arrow_angle = 0.5;
                    let angle = v.atan2(u);
                    let ah1_x = x1 - arrow_len * (angle + arrow_angle).cos();
                    let ah1_y = y1 - arrow_len * (angle + arrow_angle).sin();
                    let ah2_x = x1 - arrow_len * (angle - arrow_angle).cos();
                    let ah2_y = y1 - arrow_len * (angle - arrow_angle).sin();
                    
                    chart.draw_series(std::iter::once(PathElement::new(
                        vec![(x1, y1), (ah1_x, ah1_y)],
                        &color,
                    )))?;
                    chart.draw_series(std::iter::once(PathElement::new(
                        vec![(x1, y1), (ah2_x, ah2_y)],
                        &color,
                    )))?;
                }
            }
        }
    }
    
    root.present()?;
    Ok(())
}

/// Streamline integration using Runge-Kutta 4th order
pub fn integrate_streamline(
    field: &VectorField2D,
    start_x: f64,
    start_y: f64,
    step_size: f64,
    max_steps: usize,
) -> Vec<(f64, f64)> {
    let mut streamline = vec![(start_x, start_y)];
    let mut x = start_x;
    let mut y = start_y;
    
    for _ in 0..max_steps {
        // RK4 integration
        let (k1_u, k1_v) = interpolate_vector(field, x, y);
        let (k2_u, k2_v) = interpolate_vector(field, x + 0.5 * step_size * k1_u, y + 0.5 * step_size * k1_v);
        let (k3_u, k3_v) = interpolate_vector(field, x + 0.5 * step_size * k2_u, y + 0.5 * step_size * k2_v);
        let (k4_u, k4_v) = interpolate_vector(field, x + step_size * k3_u, y + step_size * k3_v);
        
        let dx = step_size * (k1_u + 2.0 * k2_u + 2.0 * k3_u + k4_u) / 6.0;
        let dy = step_size * (k1_v + 2.0 * k2_v + 2.0 * k3_v + k4_v) / 6.0;
        
        x += dx;
        y += dy;
        
        // Check bounds
        if x < 0.0 || x >= field.dims.1 as f64 || y < 0.0 || y >= field.dims.0 as f64 {
            break;
        }
        
        // Check if velocity is too small
        let mag = (dx * dx + dy * dy).sqrt();
        if mag < 1e-6 {
            break;
        }
        
        streamline.push((x, y));
    }
    
    streamline
}

/// Bilinear interpolation of vector field
fn interpolate_vector(field: &VectorField2D, x: f64, y: f64) -> (f64, f64) {
    let i = y.floor() as usize;
    let j = x.floor() as usize;
    
    if i >= field.dims.0 - 1 || j >= field.dims.1 - 1 {
        return (0.0, 0.0);
    }
    
    let fx = x - x.floor();
    let fy = y - y.floor();
    
    // Bilinear interpolation
    let u00 = field.u[i][j];
    let u10 = field.u[i + 1][j];
    let u01 = field.u[i][j + 1];
    let u11 = field.u[i + 1][j + 1];
    
    let v00 = field.v[i][j];
    let v10 = field.v[i + 1][j];
    let v01 = field.v[i][j + 1];
    let v11 = field.v[i + 1][j + 1];
    
    let u = (1.0 - fx) * (1.0 - fy) * u00 +
            fx * (1.0 - fy) * u01 +
            (1.0 - fx) * fy * u10 +
            fx * fy * u11;
    
    let v = (1.0 - fx) * (1.0 - fy) * v00 +
            fx * (1.0 - fy) * v01 +
            (1.0 - fx) * fy * v10 +
            fx * fy * v11;
    
    (u, v)
}

/// Render streamlines
pub fn render_streamlines(
    field: &VectorField2D,
    start_points: &[(f64, f64)],
    step_size: f64,
    max_steps: usize,
    colormap: Option<&ColorMap>,
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
        .caption("Streamlines", ("sans-serif", 30))
        .margin(20)
        .build_cartesian_2d(0.0..field.dims.1 as f64, 0.0..field.dims.0 as f64)?;
    
    chart.configure_mesh().draw()?;
    
    // Draw each streamline
    for (idx, &(sx, sy)) in start_points.iter().enumerate() {
        let streamline = integrate_streamline(field, sx, sy, step_size, max_steps);
        
        if streamline.len() > 1 {
            let color = if let Some(cmap) = colormap {
                let t = idx as f64 / start_points.len() as f64;
                let c = cmap.map(t);
                RGBColor(c.r, c.g, c.b)
            } else {
                BLUE
            };
            
            chart.draw_series(LineSeries::new(
                streamline.into_iter().map(|(x, y)| (x, y)),
                &color,
            ))?;
        }
    }
    
    root.present()?;
    Ok(())
}

/// 3D scalar field for volume rendering
pub struct ScalarField3D {
    pub data: Vec<Vec<Vec<f64>>>,
    pub dims: (usize, usize, usize),
    pub min_val: f64,
    pub max_val: f64,
}

impl ScalarField3D {
    pub fn new(data: Vec<Vec<Vec<f64>>>) -> XdlResult<Self> {
        if data.is_empty() || data[0].is_empty() || data[0][0].is_empty() {
            return Err(XdlError::InvalidArgument("Empty data".to_string()));
        }
        
        let dims = (data.len(), data[0].len(), data[0][0].len());
        
        let mut min_val = f64::INFINITY;
        let mut max_val = f64::NEG_INFINITY;
        
        for layer in &data {
            for row in layer {
                for &val in row {
                    min_val = min_val.min(val);
                    max_val = max_val.max(val);
                }
            }
        }
        
        Ok(Self {
            data,
            dims,
            min_val,
            max_val,
        })
    }
    
    pub fn get_value(&self, i: usize, j: usize, k: usize) -> Option<f64> {
        if i < self.dims.0 && j < self.dims.1 && k < self.dims.2 {
            Some(self.data[i][j][k])
        } else {
            None
        }
    }
    
    pub fn get_normalized(&self, i: usize, j: usize, k: usize) -> Option<f64> {
        self.get_value(i, j, k).map(|val| {
            if self.max_val > self.min_val {
                (val - self.min_val) / (self.max_val - self.min_val)
            } else {
                0.5
            }
        })
    }
}

/// Render volume as maximum intensity projection (MIP)
pub fn render_volume_mip(
    field: &ScalarField3D,
    axis: usize, // 0=X, 1=Y, 2=Z
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
    
    let (proj_width, proj_height) = match axis {
        0 => (field.dims.1, field.dims.2), // Project along X
        1 => (field.dims.0, field.dims.2), // Project along Y
        _ => (field.dims.0, field.dims.1), // Project along Z
    };
    
    let mut chart = ChartBuilder::on(&root)
        .caption("Volume MIP", ("sans-serif", 30))
        .margin(20)
        .build_cartesian_2d(0.0..proj_width as f64, 0.0..proj_height as f64)?;
    
    // Compute MIP
    for j in 0..proj_height {
        for i in 0..proj_width {
            let mut max_intensity = f64::NEG_INFINITY;
            
            match axis {
                0 => {
                    // Project along X
                    for k in 0..field.dims.0 {
                        if let Some(val) = field.get_value(k, i, j) {
                            max_intensity = max_intensity.max(val);
                        }
                    }
                }
                1 => {
                    // Project along Y
                    for k in 0..field.dims.1 {
                        if let Some(val) = field.get_value(i, k, j) {
                            max_intensity = max_intensity.max(val);
                        }
                    }
                }
                _ => {
                    // Project along Z
                    for k in 0..field.dims.2 {
                        if let Some(val) = field.get_value(i, j, k) {
                            max_intensity = max_intensity.max(val);
                        }
                    }
                }
            }
            
            let norm_val = if field.max_val > field.min_val {
                (max_intensity - field.min_val) / (field.max_val - field.min_val)
            } else {
                0.5
            };
            
            let color = colormap.map(norm_val);
            let rgb = RGBColor(color.r, color.g, color.b);
            
            chart.draw_series(std::iter::once(Rectangle::new(
                [(i as f64, j as f64), (i as f64 + 1.0, j as f64 + 1.0)],
                ShapeStyle::from(&rgb).filled(),
            )))?;
        }
    }
    
    root.present()?;
    Ok(())
}

/// Extract isosurface (slice at constant value) - simplified 2D slice
pub fn render_isosurface_slice(
    field: &ScalarField3D,
    isovalue: f64,
    slice_axis: usize,
    slice_index: usize,
    filename: &str,
) -> XdlResult<()> {
    let (width, height) = {
        let state = GRAPHICS_STATE.lock().unwrap();
        let win = state.get_current_window().unwrap();
        (win.width, win.height)
    };
    
    let root = BitMapBackend::new(filename, (width, height)).into_drawing_area();
    root.fill(&WHITE)?;
    
    let (slice_width, slice_height) = match slice_axis {
        0 => (field.dims.1, field.dims.2),
        1 => (field.dims.0, field.dims.2),
        _ => (field.dims.0, field.dims.1),
    };
    
    let mut chart = ChartBuilder::on(&root)
        .caption(&format!("Isosurface Slice (value={})", isovalue), ("sans-serif", 30))
        .margin(20)
        .build_cartesian_2d(0.0..slice_width as f64, 0.0..slice_height as f64)?;
    
    chart.configure_mesh().draw()?;
    
    // Mark cells where isosurface passes through
    for j in 0..slice_height {
        for i in 0..slice_width {
            let val = match slice_axis {
                0 => field.get_value(slice_index, i, j),
                1 => field.get_value(i, slice_index, j),
                _ => field.get_value(i, j, slice_index),
            };
            
            if let Some(v) = val {
                if (v - isovalue).abs() < (field.max_val - field.min_val) * 0.05 {
                    chart.draw_series(std::iter::once(Circle::new(
                        (i as f64 + 0.5, j as f64 + 0.5),
                        3,
                        ShapeStyle::from(&RED).filled(),
                    )))?;
                }
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
    fn test_vector_field_2d() {
        let u = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let v = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
        
        let field = VectorField2D::new(u, v).unwrap();
        assert_eq!(field.dims, (2, 2));
        
        let (u_val, v_val) = field.get_vector(0, 0).unwrap();
        assert_eq!(u_val, 1.0);
        assert_eq!(v_val, 0.0);
    }
    
    #[test]
    fn test_scalar_field_3d() {
        let data = vec![
            vec![vec![1.0, 2.0], vec![3.0, 4.0]],
            vec![vec![5.0, 6.0], vec![7.0, 8.0]],
        ];
        
        let field = ScalarField3D::new(data).unwrap();
        assert_eq!(field.dims, (2, 2, 2));
        assert_eq!(field.min_val, 1.0);
        assert_eq!(field.max_val, 8.0);
    }
}
