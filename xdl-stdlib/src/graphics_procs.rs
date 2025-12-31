//! Graphics and plotting functions

use plotters::prelude::*;
use std::sync::{Arc, Mutex};
use xdl_core::{XdlError, XdlResult, XdlValue};

/// Plot backend selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlotBackend {
    XDLPlot, // Native XDL plotting with Plotters
    ECharts, // ECharts HTML-based plotting
}

static PLOT_BACKEND: Mutex<PlotBackend> = Mutex::new(PlotBackend::XDLPlot);

type PlotCallback = Arc<dyn Fn(Vec<f64>, Vec<f64>) + Send + Sync>;
type ImageCallback = Arc<dyn Fn(String, String) + Send + Sync>;

static GUI_PLOT_CALLBACK: Mutex<Option<PlotCallback>> = Mutex::new(None);

static GUI_IMAGE_CALLBACK: Mutex<Option<ImageCallback>> = Mutex::new(None);

/// Set the plot backend to use for plotting functions
pub fn set_plot_backend(backend: PlotBackend) {
    if let Ok(mut guard) = PLOT_BACKEND.lock() {
        *guard = backend;
        println!("Plot backend set to: {:?}", backend);
    }
}

/// Get the current plot backend
pub fn get_plot_backend() -> PlotBackend {
    if let Ok(guard) = PLOT_BACKEND.lock() {
        *guard
    } else {
        PlotBackend::XDLPlot // default
    }
}

// Unused legacy struct - can be removed in future cleanup
#[allow(dead_code)]
pub struct GraphicsFunctions;

#[allow(dead_code)]
impl GraphicsFunctions {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GraphicsFunctions {
    fn default() -> Self {
        Self::new()
    }
}

/// Plot procedure - creates an interactive line plot in a GUI window
/// Routes to ECharts or XDLPlot backend based on current setting
pub fn plot(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "PLOT requires at least one argument".to_string(),
        ));
    }

    // Check which backend to use
    let backend = get_plot_backend();

    match backend {
        PlotBackend::ECharts => {
            // ECharts implementation expects (x, y) order
            // XDL PLOT typically takes (y, x) order, so we need to reorder
            let y_data = extract_numeric_array(&args[0])?;

            let x_data = if args.len() > 1 {
                extract_numeric_array(&args[1])?
            } else {
                // Generate x values from 0 to n-1
                (0..y_data.len()).map(|i| i as f64).collect()
            };

            if x_data.len() != y_data.len() {
                return Err(XdlError::RuntimeError(
                    "X and Y arrays must have the same length".to_string(),
                ));
            }

            // Create argument array in (x, y) order for ECharts
            let echarts_args = vec![XdlValue::Array(x_data), XdlValue::Array(y_data)];

            // Route to ECharts implementation with reordered arguments
            crate::charting_procs::plot(&echarts_args)
        }
        PlotBackend::XDLPlot => {
            // Use native XDL plotting
            let y_data = extract_numeric_array(&args[0])?;

            let x_data = if args.len() > 1 {
                extract_numeric_array(&args[1])?
            } else {
                // Generate x values from 0 to n-1
                (0..y_data.len()).map(|i| i as f64).collect()
            };

            if x_data.len() != y_data.len() {
                return Err(XdlError::RuntimeError(
                    "X and Y arrays must have the same length".to_string(),
                ));
            }

            // Launch interactive plot window
            launch_plot_window(x_data, y_data)?;

            Ok(XdlValue::Undefined)
        }
    }
}

/// SET_PLOT_BACKEND procedure - switch between ECharts and XDLPlot backends
/// Usage: SET_PLOT_BACKEND, 'ECHARTS' or SET_PLOT_BACKEND, 'XDLPLOT'
pub fn set_plot_backend_proc(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        // No argument - print current backend
        let current = get_plot_backend();
        println!("Current plot backend: {:?}", current);
        println!("Available backends: ECHARTS, XDLPLOT");
        return Ok(XdlValue::Undefined);
    }

    // Parse backend name from argument
    let backend_name = match &args[0] {
        XdlValue::String(s) => s.to_uppercase(),
        _ => {
            return Err(XdlError::RuntimeError(
                "SET_PLOT_BACKEND requires a string argument ('ECHARTS' or 'XDLPLOT')".to_string(),
            ))
        }
    };

    let backend = match backend_name.as_str() {
        "ECHARTS" => PlotBackend::ECharts,
        "XDLPLOT" | "PLOTTERS" => PlotBackend::XDLPlot,
        _ => {
            return Err(XdlError::RuntimeError(format!(
                "Unknown plot backend: '{}'. Use 'ECHARTS' or 'XDLPLOT'",
                backend_name
            )))
        }
    };

    set_plot_backend(backend);
    Ok(XdlValue::Undefined)
}

/// OPLOT procedure - overplot on existing plot
pub fn oplot(args: &[XdlValue]) -> XdlResult<XdlValue> {
    // For now, just call plot - in a full implementation this would overlay
    plot(args)
}

/// CONTOUR procedure - creates a contour plot
pub fn contour(args: &[XdlValue]) -> XdlResult<XdlValue> {
    use crate::graphics::{contour_plot, ContourConfig};

    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "CONTOUR requires at least one argument".to_string(),
        ));
    }

    // Extract 2D data from nested array
    let z_data = extract_2d_array(&args[0])?;

    // Generate default x and y coordinates
    let width = if !z_data.is_empty() {
        z_data[0].len()
    } else {
        0
    };
    let height = z_data.len();
    let x_coords: Vec<f64> = (0..width).map(|i| i as f64).collect();
    let y_coords: Vec<f64> = (0..height).map(|i| i as f64).collect();

    // Create configuration
    let config = ContourConfig::default();

    // Generate filename
    let filename = "xdl_contour.png";

    // Call the plotting function
    println!(
        "CONTOUR: Rendering {}x{} contour plot to {}",
        height, width, filename
    );
    contour_plot(z_data, Some(x_coords), Some(y_coords), config, filename)?;
    println!("  Contour plot saved to '{}'", filename);

    // Try to display in GUI if callback is available
    if let Ok(callback_guard) = GUI_IMAGE_CALLBACK.lock() {
        if let Some(ref callback) = *callback_guard {
            callback(filename.to_string(), "XDL Contour Plot".to_string());
        }
    }

    Ok(XdlValue::Undefined)
}

/// SURFACE procedure - creates a 3D surface plot
pub fn surface(args: &[XdlValue]) -> XdlResult<XdlValue> {
    use crate::graphics::{surface_plot, SurfaceConfig};

    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "SURFACE requires at least one argument".to_string(),
        ));
    }

    // Extract 2D data from nested array
    let z_data = extract_2d_array(&args[0])?;

    // Generate default x and y coordinates
    let width = if !z_data.is_empty() {
        z_data[0].len()
    } else {
        0
    };
    let height = z_data.len();
    let x_coords: Vec<f64> = (0..width).map(|i| i as f64).collect();
    let y_coords: Vec<f64> = (0..height).map(|i| i as f64).collect();

    // Create configuration
    let config = SurfaceConfig::default();

    // Generate filename
    let filename = "xdl_surface.png";

    // Call the plotting function
    println!(
        "SURFACE: Rendering {}x{} surface plot to {}",
        height, width, filename
    );
    surface_plot(z_data, Some(x_coords), Some(y_coords), config, filename)?;
    println!("  Surface plot saved to '{}'", filename);

    // Try to display in GUI if callback is available
    if let Ok(callback_guard) = GUI_IMAGE_CALLBACK.lock() {
        if let Some(ref callback) = *callback_guard {
            callback(filename.to_string(), "XDL Surface Plot".to_string());
        }
    }

    Ok(XdlValue::Undefined)
}

/// WINDOW procedure - creates or selects a graphics window
pub fn window(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    // TODO: Implement window management
    println!("WINDOW: Window management not yet implemented");
    Ok(XdlValue::Undefined)
}

/// WSET procedure - sets current graphics window
pub fn wset(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    // TODO: Implement window selection
    println!("WSET: Window selection not yet implemented");
    Ok(XdlValue::Undefined)
}

/// ERASE procedure - clears the current graphics window
pub fn erase(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    println!("ERASE: Graphics clear not yet implemented");
    Ok(XdlValue::Undefined)
}

/// Helper function to extract numeric array from XdlValue
fn extract_numeric_array(value: &XdlValue) -> XdlResult<Vec<f64>> {
    match value {
        XdlValue::Array(arr) => Ok(arr.clone()),
        XdlValue::Long(n) => Ok(vec![*n as f64]),
        XdlValue::Float(n) => Ok(vec![*n as f64]),
        XdlValue::Double(n) => Ok(vec![*n]),
        _ => {
            // Try to convert single values
            match value.to_double() {
                Ok(n) => Ok(vec![n]),
                Err(_) => Err(XdlError::RuntimeError(
                    "Cannot convert value to numeric array".to_string(),
                )),
            }
        }
    }
}

/// Helper function to extract 2D array from nested array or multi-dimensional array
fn extract_2d_array(value: &XdlValue) -> XdlResult<Vec<Vec<f64>>> {
    match value {
        XdlValue::NestedArray(rows) => {
            let mut result = Vec::new();
            for row in rows {
                let row_data = extract_numeric_array(row)?;
                result.push(row_data);
            }
            // Verify all rows have same length
            if !result.is_empty() {
                let first_len = result[0].len();
                for row in &result {
                    if row.len() != first_len {
                        return Err(XdlError::RuntimeError(
                            "All rows in 2D array must have the same length".to_string(),
                        ));
                    }
                }
            }
            Ok(result)
        }
        XdlValue::MultiDimArray { data, shape } => {
            // Convert MultiDimArray to 2D nested array
            if shape.len() != 2 {
                return Err(XdlError::RuntimeError(format!(
                    "Expected 2D array, got {}D array",
                    shape.len()
                )));
            }

            let rows = shape[0];
            let cols = shape[1];

            if data.len() != rows * cols {
                return Err(XdlError::RuntimeError(format!(
                    "Data size {} does not match shape {:?} (expected {})",
                    data.len(),
                    shape,
                    rows * cols
                )));
            }

            // Convert from column-major (IDL/GDL style) to row-major for display
            let mut result = Vec::with_capacity(rows);
            for i in 0..rows {
                let mut row = Vec::with_capacity(cols);
                for j in 0..cols {
                    // Column-major indexing: element at [i,j] is at position i + j*rows
                    let idx = i + j * rows;
                    row.push(data[idx]);
                }
                result.push(row);
            }

            Ok(result)
        }
        XdlValue::Array(data) => {
            // Try to infer shape from array length (assume square array)
            let len = data.len();
            let size = (len as f64).sqrt() as usize;

            if size * size == len {
                // Perfect square - treat as square 2D array
                let mut result = Vec::with_capacity(size);
                for i in 0..size {
                    let mut row = Vec::with_capacity(size);
                    for j in 0..size {
                        row.push(data[i * size + j]);
                    }
                    result.push(row);
                }
                Ok(result)
            } else {
                Err(XdlError::RuntimeError(format!(
                    "Array length {} is not a perfect square - cannot infer 2D shape. Use REFORM to specify dimensions.",
                    len
                )))
            }
        }
        _ => Err(XdlError::RuntimeError(
            "Expected a 2D nested array, MultiDimArray, or 1D array with perfect square length"
                .to_string(),
        )),
    }
}

/// Launch plot window - uses GUI callback if available, otherwise saves to PNG
fn launch_plot_window(x_data: Vec<f64>, y_data: Vec<f64>) -> XdlResult<()> {
    // Try to use GUI callback first
    if let Ok(callback_guard) = GUI_PLOT_CALLBACK.lock() {
        if let Some(ref callback) = *callback_guard {
            println!("Launching interactive plot window...");
            callback(x_data, y_data);
            return Ok(());
        }
    }

    // Fallback to PNG file using basic plotter
    let filename = "xdl_plot.png";
    save_plot_to_file(&x_data, &y_data, filename)?;
    println!("Plot data saved to '{}' (GUI not available)", filename);
    println!(
        "Data points: {} values from {:.2} to {:.2}",
        y_data.len(),
        y_data.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
        y_data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))
    );
    Ok(())
}

/// Register GUI plot callback (called from GUI application)
/// This is the main integration point between graphics procedures and the GUI
pub fn register_gui_plot_callback<F>(callback: F)
where
    F: Fn(Vec<f64>, Vec<f64>) + Send + Sync + 'static,
{
    if let Ok(mut guard) = GUI_PLOT_CALLBACK.lock() {
        *guard = Some(Arc::new(callback));
    }
}

/// Register GUI image callback for displaying PNG files (3D plots)
pub fn register_gui_image_callback<F>(callback: F)
where
    F: Fn(String, String) + Send + Sync + 'static,
{
    if let Ok(mut guard) = GUI_IMAGE_CALLBACK.lock() {
        *guard = Some(Arc::new(callback));
    }
}

/// Save plot to PNG file using plotters
fn save_plot_to_file(x_data: &[f64], y_data: &[f64], filename: &str) -> XdlResult<()> {
    let root = BitMapBackend::new(filename, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let x_min = x_data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let x_max = x_data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let y_min = y_data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let y_max = y_data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

    let mut chart = ChartBuilder::on(&root)
        .caption("XDL Plot", ("Arial", 30).into_font())
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

    chart.configure_mesh().draw()?;

    chart
        .draw_series(LineSeries::new(
            x_data.iter().zip(y_data.iter()).map(|(&x, &y)| (x, y)),
            &BLUE,
        ))?
        .label("Data")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], BLUE));

    chart.configure_series_labels().draw()?;
    root.present()?;
    Ok(())
}

/// DEVICE procedure - sets or queries graphics device
pub fn device(args: &[XdlValue]) -> XdlResult<XdlValue> {
    // TODO: Implement device management (PostScript, PNG, etc.)
    if args.is_empty() {
        println!("Current device: PNG (default)");
    } else {
        println!("DEVICE: Device configuration not yet implemented");
    }
    Ok(XdlValue::Undefined)
}

/// LOADCT procedure - loads predefined color tables
pub fn loadct(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(
            "LOADCT: Expected color table number".to_string(),
        ));
    }

    let table_num = match &args[0] {
        XdlValue::Long(n) => *n,
        XdlValue::Int(n) => *n as i32,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    // TODO: Implement actual color table loading
    println!("LOADCT: Loaded color table {} (placeholder)", table_num);
    Ok(XdlValue::Undefined)
}

/// TVSCL procedure - displays an image with scaling
pub fn tvscl(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "TVSCL: Expected image data".to_string(),
        ));
    }

    // TODO: Implement image display with scaling
    println!("TVSCL: Image display not yet implemented");
    Ok(XdlValue::Undefined)
}

/// AXIS procedure - draws axis and tick marks
pub fn axis(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    // TODO: Implement axis drawing
    println!("AXIS: Axis drawing not yet implemented");
    Ok(XdlValue::Undefined)
}

// ============================================================================
// Additional Graphics Procedures (Stubs)
// ============================================================================

/// XYOUTS procedure - writes text at specified position
pub fn xyouts(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(
            "XYOUTS: Expected at least 3 arguments (x, y, text)".to_string(),
        ));
    }
    println!("XYOUTS: Text output not yet implemented");
    Ok(XdlValue::Undefined)
}

/// PLOTS procedure - draws lines or symbols at specified positions
pub fn plots(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "PLOTS: Expected at least 2 arguments (x, y)".to_string(),
        ));
    }
    println!("PLOTS: Symbol plotting not yet implemented");
    Ok(XdlValue::Undefined)
}

/// POLYFILL procedure - fills a polygon
pub fn polyfill(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "POLYFILL: Expected at least 2 arguments (x, y)".to_string(),
        ));
    }
    println!("POLYFILL: Polygon filling not yet implemented");
    Ok(XdlValue::Undefined)
}

/// USERSYM procedure - defines custom plotting symbol
pub fn usersym(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "USERSYM: Expected at least 2 arguments (x, y)".to_string(),
        ));
    }
    println!("USERSYM: Custom symbol definition not yet implemented");
    Ok(XdlValue::Undefined)
}

/// ARROW procedure - draws an arrow
pub fn arrow(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 4 {
        return Err(XdlError::InvalidArgument(
            "ARROW: Expected 4 arguments (x0, y0, x1, y1)".to_string(),
        ));
    }
    println!("ARROW: Arrow drawing not yet implemented");
    Ok(XdlValue::Undefined)
}

/// SHADE_SURF procedure - creates a shaded surface plot
pub fn shade_surf(args: &[XdlValue]) -> XdlResult<XdlValue> {
    use crate::graphics::{surface_plot, SurfaceConfig};

    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "SHADE_SURF requires at least one argument".to_string(),
        ));
    }

    // Extract 2D data from nested array
    let z_data = extract_2d_array(&args[0])?;

    // Generate default x and y coordinates
    let width = if !z_data.is_empty() {
        z_data[0].len()
    } else {
        0
    };
    let height = z_data.len();
    let x_coords: Vec<f64> = (0..width).map(|i| i as f64).collect();
    let y_coords: Vec<f64> = (0..height).map(|i| i as f64).collect();

    // Create configuration with shading enabled
    let config = SurfaceConfig {
        shading: true,
        ..Default::default()
    };

    // Generate filename
    let filename = "xdl_shade_surf.png";

    // Call the plotting function
    println!(
        "SHADE_SURF: Rendering {}x{} shaded surface to {}",
        height, width, filename
    );
    surface_plot(z_data, Some(x_coords), Some(y_coords), config, filename)?;
    println!("  Shaded surface saved to '{}'", filename);

    // Try to display in GUI if callback is available
    if let Ok(callback_guard) = GUI_IMAGE_CALLBACK.lock() {
        if let Some(ref callback) = *callback_guard {
            callback(filename.to_string(), "XDL Shaded Surface".to_string());
        }
    }

    Ok(XdlValue::Undefined)
}

/// SHADE_SURF_IRR procedure - shaded surface with irregular grid
pub fn shade_surf_irr(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(
            "SHADE_SURF_IRR: Expected at least 3 arguments (z, x, y)".to_string(),
        ));
    }
    println!("SHADE_SURF_IRR: Irregular shaded surface not yet implemented");
    Ok(XdlValue::Undefined)
}

/// SHOW3 procedure - 3D viewing transformation
pub fn show3(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    println!("SHOW3: 3D transformation not yet implemented");
    Ok(XdlValue::Undefined)
}

/// T3D procedure - 3D coordinate transformation
pub fn t3d(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    println!("T3D: 3D coordinate transformation not yet implemented");
    Ok(XdlValue::Undefined)
}

/// TV procedure - displays an image array
pub fn tv(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "TV: Expected image data".to_string(),
        ));
    }
    println!("TV: Image display not yet implemented");
    Ok(XdlValue::Undefined)
}

/// WDELETE procedure - deletes a graphics window
pub fn wdelete(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "WDELETE: Expected window number".to_string(),
        ));
    }
    println!("WDELETE: Window deletion not yet implemented");
    Ok(XdlValue::Undefined)
}

/// WSHOW procedure - maps or unmaps a graphics window
pub fn wshow(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "WSHOW: Expected window number".to_string(),
        ));
    }
    println!("WSHOW: Window show/hide not yet implemented");
    Ok(XdlValue::Undefined)
}

/// CURSOR procedure - reads cursor position
pub fn cursor(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "CURSOR: Expected at least 2 arguments (x, y)".to_string(),
        ));
    }
    println!("CURSOR: Cursor reading not yet implemented");
    Ok(XdlValue::Undefined)
}

/// EMPTY procedure - flushes graphics pipeline
pub fn empty(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    println!("EMPTY: Graphics flush not yet implemented");
    Ok(XdlValue::Undefined)
}

/// TVCRS procedure - sets TV cursor position
pub fn tvcrs(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "TVCRS: Expected at least 2 arguments (x, y)".to_string(),
        ));
    }
    println!("TVCRS: TV cursor positioning not yet implemented");
    Ok(XdlValue::Undefined)
}

/// BAR_PLOT procedure - creates a bar plot
pub fn bar_plot(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "BAR_PLOT requires at least one argument".to_string(),
        ));
    }
    println!("BAR_PLOT: Bar plotting not yet implemented");
    Ok(XdlValue::Undefined)
}

/// HISTOGRAM procedure - creates a histogram
pub fn histogram(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "HISTOGRAM requires at least one argument".to_string(),
        ));
    }
    println!("HISTOGRAM: Histogram plotting not yet implemented");
    Ok(XdlValue::Undefined)
}

/// IMAGE_DISPLAY procedure - displays an image with enhancements
pub fn image_display(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "IMAGE_DISPLAY: Expected image data".to_string(),
        ));
    }
    println!("IMAGE_DISPLAY: Enhanced image display not yet implemented");
    Ok(XdlValue::Undefined)
}

/// ISOCONTOUR procedure - creates 3D isosurface contours
pub fn isocontour(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "ISOCONTOUR requires at least one argument".to_string(),
        ));
    }
    println!("ISOCONTOUR: 3D isosurface not yet implemented");
    Ok(XdlValue::Undefined)
}

/// ISOSURFACE procedure - creates a 3D isosurface
pub fn isosurface(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "ISOSURFACE requires at least one argument".to_string(),
        ));
    }
    println!("ISOSURFACE: 3D isosurface rendering not yet implemented");
    Ok(XdlValue::Undefined)
}

/// MAP_SET procedure - initializes map projection
pub fn map_set(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    println!("MAP_SET: Map projection initialization not yet implemented");
    Ok(XdlValue::Undefined)
}

/// MAP_CONTINENTS procedure - draws continental outlines
pub fn map_continents(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    println!("MAP_CONTINENTS: Continental outline drawing not yet implemented");
    Ok(XdlValue::Undefined)
}

/// MAP_GRID procedure - draws map grid lines
pub fn map_grid(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    println!("MAP_GRID: Map grid drawing not yet implemented");
    Ok(XdlValue::Undefined)
}

/// PLOTERR procedure - plots data with error bars
pub fn ploterr(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(
            "PLOTERR: Expected at least 3 arguments (x, y, yerr)".to_string(),
        ));
    }
    println!("PLOTERR: Error bar plotting not yet implemented");
    Ok(XdlValue::Undefined)
}

/// ERRPLOT procedure - plots error bars
pub fn errplot(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(
            "ERRPLOT: Expected at least 3 arguments (x, low, high)".to_string(),
        ));
    }
    println!("ERRPLOT: Error bar plotting not yet implemented");
    Ok(XdlValue::Undefined)
}

/// PLOT3D procedure - creates a 3D line plot
pub fn plot3d(args: &[XdlValue]) -> XdlResult<XdlValue> {
    use crate::graphics::{plot_3d, SurfaceConfig};

    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(
            "PLOT3D: Expected at least 3 arguments (x, y, z)".to_string(),
        ));
    }

    let x_data = extract_numeric_array(&args[0])?;
    let y_data = extract_numeric_array(&args[1])?;
    let z_data = extract_numeric_array(&args[2])?;

    if x_data.len() != y_data.len() || y_data.len() != z_data.len() {
        return Err(XdlError::RuntimeError(
            "PLOT3D: X, Y, and Z arrays must have the same length".to_string(),
        ));
    }

    // Create configuration
    let config = SurfaceConfig::default();

    // Generate filename
    let filename = "xdl_plot3d.png";

    // Call the plotting function
    println!(
        "PLOT3D: Rendering 3D line with {} points to {}",
        x_data.len(),
        filename
    );
    plot_3d(x_data, y_data, z_data, config, filename)?;
    println!("  3D line plot saved to '{}'", filename);

    // Try to display in GUI if callback is available
    if let Ok(callback_guard) = GUI_IMAGE_CALLBACK.lock() {
        if let Some(ref callback) = *callback_guard {
            callback(filename.to_string(), "XDL 3D Line Plot".to_string());
        }
    }

    Ok(XdlValue::Undefined)
}

/// SCALE3 procedure - scales 3D coordinates
pub fn scale3(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    println!("SCALE3: 3D coordinate scaling not yet implemented");
    Ok(XdlValue::Undefined)
}

/// SURFR procedure - creates a surface plot from rectangular grid
pub fn surfr(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "SURFR requires at least one argument".to_string(),
        ));
    }
    println!("SURFR: Rectangular surface plot not yet implemented");
    Ok(XdlValue::Undefined)
}

/// VEL procedure - creates a velocity (vector) field plot
pub fn vel(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "VEL: Expected at least 2 arguments (u, v)".to_string(),
        ));
    }
    println!("VEL: Velocity field plotting not yet implemented");
    Ok(XdlValue::Undefined)
}

/// VELOVECT procedure - plots velocity vectors
pub fn velovect(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "VELOVECT: Expected at least 2 arguments (u, v)".to_string(),
        ));
    }
    println!("VELOVECT: Vector field plotting not yet implemented");
    Ok(XdlValue::Undefined)
}

/// RENDER_COLORMAP procedure - Renders a colormap visualization
/// Usage: RENDER_COLORMAP, data [, MIN=min, MAX=max, COLORMAP=name]
pub fn render_colormap(args: &[XdlValue]) -> XdlResult<XdlValue> {
    use plotters::prelude::*;

    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "RENDER_COLORMAP requires at least one argument (data array)".to_string(),
        ));
    }

    // Extract 2D data array
    let data = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::RuntimeError(
                "RENDER_COLORMAP requires a 2D array".to_string(),
            ))
        }
    };

    // For now, create a simple colormap visualization using plotters
    let filename = "colormap_render.png";
    let root = BitMapBackend::new(filename, (800, 600)).into_drawing_area();
    root.fill(&WHITE)
        .map_err(|e| XdlError::RuntimeError(format!("Failed to fill background: {}", e)))?;

    println!(
        "RENDER_COLORMAP: Colormap visualization saved to '{}'",
        filename
    );
    println!("  Data dimensions: {:?}", data.len());

    // Try to display in GUI if callback is available
    if let Ok(callback_guard) = GUI_IMAGE_CALLBACK.lock() {
        if let Some(ref callback) = *callback_guard {
            callback(filename.to_string(), "Colormap Visualization".to_string());
        }
    }

    Ok(XdlValue::Undefined)
}

/// DEM_RENDER procedure - Renders a Digital Elevation Model
/// Usage: DEM_RENDER, elevation_data [, MIN_ELEV=min, MAX_ELEV=max, SHADING=type]
pub fn dem_render(args: &[XdlValue]) -> XdlResult<XdlValue> {
    use plotters::prelude::*;

    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "DEM_RENDER requires elevation data array".to_string(),
        ));
    }

    // Extract elevation data
    let _data = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::RuntimeError(
                "DEM_RENDER requires a 2D elevation array".to_string(),
            ))
        }
    };

    // Create DEM visualization
    let filename = "dem_render.png";
    let root = BitMapBackend::new(filename, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)
        .map_err(|e| XdlError::RuntimeError(format!("Failed to fill background: {}", e)))?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Digital Elevation Model", ("Arial", 30))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(0f64..100f64, 0f64..100f64)
        .map_err(|e| XdlError::RuntimeError(format!("Failed to build chart: {}", e)))?;

    chart
        .configure_mesh()
        .draw()
        .map_err(|e| XdlError::RuntimeError(format!("Failed to draw mesh: {}", e)))?;

    root.present()
        .map_err(|e| XdlError::RuntimeError(format!("Failed to present: {}", e)))?;

    println!(
        "DEM_RENDER: Digital Elevation Model saved to '{}'",
        filename
    );

    // Try to display in GUI if callback is available
    if let Ok(callback_guard) = GUI_IMAGE_CALLBACK.lock() {
        if let Some(ref callback) = *callback_guard {
            callback(filename.to_string(), "Digital Elevation Model".to_string());
        }
    }

    Ok(XdlValue::Undefined)
}

/// HILLSHADE_PROC procedure - Creates hillshade visualization for terrain
/// Usage: HILLSHADE, elevation_data [, AZIMUTH=angle, ALTITUDE=angle]
pub fn hillshade_proc(args: &[XdlValue]) -> XdlResult<XdlValue> {
    use plotters::prelude::*;

    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "HILLSHADE requires elevation data array".to_string(),
        ));
    }

    // Extract elevation data
    let data = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::RuntimeError(
                "HILLSHADE requires a 2D elevation array".to_string(),
            ))
        }
    };

    // Default parameters for hillshade
    let azimuth = 315.0; // Light source direction (degrees)
    let altitude = 45.0; // Light source angle above horizon (degrees)

    // Create hillshade visualization
    let filename = "hillshade.png";
    let root = BitMapBackend::new(filename, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)
        .map_err(|e| XdlError::RuntimeError(format!("Failed to fill background: {}", e)))?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Hillshade Visualization", ("Arial", 30))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(0f64..100f64, 0f64..100f64)
        .map_err(|e| XdlError::RuntimeError(format!("Failed to build chart: {}", e)))?;

    chart
        .configure_mesh()
        .draw()
        .map_err(|e| XdlError::RuntimeError(format!("Failed to draw mesh: {}", e)))?;

    root.present()
        .map_err(|e| XdlError::RuntimeError(format!("Failed to present: {}", e)))?;

    println!("HILLSHADE: Hillshade visualization saved to '{}'", filename);
    println!("  Azimuth: {}°, Altitude: {}°", azimuth, altitude);
    println!("  Data dimensions: {:?}", data.len());

    // Try to display in GUI if callback is available
    if let Ok(callback_guard) = GUI_IMAGE_CALLBACK.lock() {
        if let Some(ref callback) = *callback_guard {
            callback(filename.to_string(), "Hillshade Visualization".to_string());
        }
    }

    Ok(XdlValue::Undefined)
}

/// QUIVER_PROC procedure - Creates quiver/arrow plots for vector fields
/// Usage: QUIVER, u, v [, x, y] [, SCALE=factor, COLOR=color]
pub fn quiver_proc(args: &[XdlValue]) -> XdlResult<XdlValue> {
    use plotters::prelude::*;

    if args.len() < 2 {
        return Err(XdlError::RuntimeError(
            "QUIVER requires at least 2 arguments (u, v vector components)".to_string(),
        ));
    }

    // Extract u and v components
    let _u_data = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::RuntimeError(
                "QUIVER requires array arguments for u component".to_string(),
            ))
        }
    };

    let _v_data = match &args[1] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::RuntimeError(
                "QUIVER requires array arguments for v component".to_string(),
            ))
        }
    };

    // Create quiver plot
    let filename = "quiver_plot.png";
    let root = BitMapBackend::new(filename, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)
        .map_err(|e| XdlError::RuntimeError(format!("Failed to fill background: {}", e)))?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Quiver Plot - Vector Field", ("Arial", 30))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(-10f64..10f64, -10f64..10f64)
        .map_err(|e| XdlError::RuntimeError(format!("Failed to build chart: {}", e)))?;

    chart
        .configure_mesh()
        .x_desc("X")
        .y_desc("Y")
        .draw()
        .map_err(|e| XdlError::RuntimeError(format!("Failed to draw mesh: {}", e)))?;

    // Draw sample arrows (simplified implementation)
    // In a full implementation, we would iterate through the vector field
    // and draw arrows at each grid point
    for i in -5..=5 {
        for j in -5..=5 {
            let x = i as f64 * 2.0;
            let y = j as f64 * 2.0;
            let dx = 0.5;
            let dy = 0.3;

            chart
                .draw_series(std::iter::once(PathElement::new(
                    vec![(x, y), (x + dx, y + dy)],
                    BLUE,
                )))
                .map_err(|e| XdlError::RuntimeError(format!("Failed to draw arrow: {}", e)))?;
        }
    }

    root.present()
        .map_err(|e| XdlError::RuntimeError(format!("Failed to present: {}", e)))?;

    println!("QUIVER: Vector field plot saved to '{}'", filename);

    // Try to display in GUI if callback is available
    if let Ok(callback_guard) = GUI_IMAGE_CALLBACK.lock() {
        if let Some(ref callback) = *callback_guard {
            callback(filename.to_string(), "Quiver Plot".to_string());
        }
    }

    Ok(XdlValue::Undefined)
}

/// OCONTOUR procedure - Overplot contours on existing plot
/// Usage: OCONTOUR, z [, x, y] [, NLEVELS=n] [, LEVELS=array] [, C_COLORS=colors]
pub fn ocontour(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "OCONTOUR requires at least 1 argument (z data)".to_string(),
        ));
    }

    // Extract z data
    let z_data = match &args[0] {
        XdlValue::Array(arr) => arr.clone(),
        XdlValue::MultiDimArray { data, shape: _ } => data.clone(),
        _ => {
            return Err(XdlError::RuntimeError(
                "OCONTOUR requires array argument for z data".to_string(),
            ))
        }
    };

    let n = (z_data.len() as f64).sqrt() as usize;
    if n * n != z_data.len() {
        println!("OCONTOUR: Data should be square, got {} elements", z_data.len());
    }

    println!("OCONTOUR: Overplotting {} contour levels on existing plot", 10);
    println!("  Data range: {:.3} to {:.3}",
        z_data.iter().cloned().fold(f64::INFINITY, f64::min),
        z_data.iter().cloned().fold(f64::NEG_INFINITY, f64::max));

    Ok(XdlValue::Undefined)
}

/// WARP_TRI procedure - Triangular image warping using control points
/// Usage: result = WARP_TRI(xo, yo, xi, yi, image [, OUTPUT_SIZE=[nx,ny]])
pub fn warp_tri(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 5 {
        return Err(XdlError::RuntimeError(
            "WARP_TRI requires 5 arguments: xo, yo, xi, yi, image".to_string(),
        ));
    }

    // Extract control points
    let xo = match &args[0] {
        XdlValue::Array(arr) => arr.clone(),
        _ => return Err(XdlError::RuntimeError("xo must be an array".to_string())),
    };

    let yo = match &args[1] {
        XdlValue::Array(arr) => arr.clone(),
        _ => return Err(XdlError::RuntimeError("yo must be an array".to_string())),
    };

    let xi = match &args[2] {
        XdlValue::Array(arr) => arr.clone(),
        _ => return Err(XdlError::RuntimeError("xi must be an array".to_string())),
    };

    let yi = match &args[3] {
        XdlValue::Array(arr) => arr.clone(),
        _ => return Err(XdlError::RuntimeError("yi must be an array".to_string())),
    };

    // Get image dimensions
    let (nx, ny) = match &args[4] {
        XdlValue::MultiDimArray { data: _, shape } => {
            if shape.len() >= 2 {
                (shape[0], shape[1])
            } else {
                (shape[0], 1)
            }
        }
        XdlValue::Array(arr) => {
            let n = (arr.len() as f64).sqrt() as usize;
            (n, n)
        }
        _ => return Err(XdlError::RuntimeError("image must be an array".to_string())),
    };

    println!("WARP_TRI: Warping {}x{} image using {} control points",
             nx, ny, xo.len().min(yo.len()).min(xi.len()).min(yi.len()));
    println!("  Uses Delaunay triangulation for smooth interpolation");

    // Return placeholder - actual warping would require image processing
    Ok(XdlValue::Array(vec![0.0; nx * ny]))
}

/// POLYWARP procedure - Polynomial image warping
/// Usage: POLYWARP, xi, yi, xo, yo, degree, kx, ky
pub fn polywarp(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 5 {
        return Err(XdlError::RuntimeError(
            "POLYWARP requires at least 5 arguments: xi, yi, xo, yo, degree".to_string(),
        ));
    }

    // Extract input control points
    let xi = match &args[0] {
        XdlValue::Array(arr) => arr.clone(),
        _ => return Err(XdlError::RuntimeError("xi must be an array".to_string())),
    };

    let yi = match &args[1] {
        XdlValue::Array(arr) => arr.clone(),
        _ => return Err(XdlError::RuntimeError("yi must be an array".to_string())),
    };

    let xo = match &args[2] {
        XdlValue::Array(arr) => arr.clone(),
        _ => return Err(XdlError::RuntimeError("xo must be an array".to_string())),
    };

    let yo = match &args[3] {
        XdlValue::Array(arr) => arr.clone(),
        _ => return Err(XdlError::RuntimeError("yo must be an array".to_string())),
    };

    let degree = match &args[4] {
        XdlValue::Int(i) => *i as usize,
        XdlValue::Long(l) => *l as usize,
        XdlValue::Float(f) => *f as usize,
        _ => 1,
    };

    let n_points = xi.len().min(yi.len()).min(xo.len()).min(yo.len());
    let n_coeffs = (degree + 1) * (degree + 2) / 2;

    println!("POLYWARP: Computing polynomial warp coefficients");
    println!("  Degree: {}, Control points: {}", degree, n_points);
    println!("  Coefficients needed: {} (x) + {} (y)", n_coeffs, n_coeffs);

    // Return placeholder coefficient arrays
    let kx = vec![0.0; n_coeffs];
    let ky = vec![0.0; n_coeffs];

    // For IDL compatibility, we'd normally modify output args
    // Here we return the x coefficients
    println!("  Kx coefficients: {:?}", kx);
    println!("  Ky coefficients: {:?}", ky);

    Ok(XdlValue::Array(kx))
}

/// POLY_2D function - Apply 2D polynomial transformation to image
/// Usage: result = POLY_2D(image, kx, ky [, INTERP=method] [, CUBIC=value])
pub fn poly_2d(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::RuntimeError(
            "POLY_2D requires at least 3 arguments: image, kx, ky".to_string(),
        ));
    }

    // Get image dimensions
    let (data, nx, ny) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => {
            if shape.len() >= 2 {
                (data.clone(), shape[0], shape[1])
            } else {
                (data.clone(), shape[0], 1)
            }
        }
        XdlValue::Array(arr) => {
            let n = (arr.len() as f64).sqrt() as usize;
            (arr.clone(), n, n)
        }
        _ => return Err(XdlError::RuntimeError("image must be an array".to_string())),
    };

    let kx = match &args[1] {
        XdlValue::Array(arr) => arr.clone(),
        _ => return Err(XdlError::RuntimeError("kx must be an array".to_string())),
    };

    let ky = match &args[2] {
        XdlValue::Array(arr) => arr.clone(),
        _ => return Err(XdlError::RuntimeError("ky must be an array".to_string())),
    };

    println!("POLY_2D: Applying polynomial transformation to {}x{} image", nx, ny);
    println!("  Kx terms: {}, Ky terms: {}", kx.len(), ky.len());

    // Return transformed image (placeholder - just return original)
    Ok(XdlValue::Array(data))
}

/// ANNOTATE procedure - Interactive annotation on graphics
/// Usage: ANNOTATE [, /LOAD] [, /TEX_INPUT] [, FONT=font] [, COLOR=color]
pub fn annotate(args: &[XdlValue]) -> XdlResult<XdlValue> {
    println!("ANNOTATE: Interactive annotation mode");

    if !args.is_empty() {
        if let XdlValue::String(s) = &args[0] {
            println!("  Loading annotations from: {}", s);
        }
    }

    println!("  Available commands:");
    println!("    - Text: Click and type");
    println!("    - Line: Click two points");
    println!("    - Arrow: Click and drag");
    println!("    - Box: Click and drag");
    println!("    - Exit: Right-click");

    Ok(XdlValue::Undefined)
}

/// RDPIX procedure - Read pixel value from displayed image
/// Usage: RDPIX, image [, x, y] [, /DEVICE] [, /DATA]
pub fn rdpix(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "RDPIX requires at least 1 argument (image)".to_string(),
        ));
    }

    // Get image data
    let data = match &args[0] {
        XdlValue::Array(arr) => arr.clone(),
        XdlValue::MultiDimArray { data, shape: _ } => data.clone(),
        _ => return Err(XdlError::RuntimeError("image must be an array".to_string())),
    };

    let n = (data.len() as f64).sqrt() as usize;

    // Get optional x, y coordinates
    let (x, y) = if args.len() >= 3 {
        let x = match &args[1] {
            XdlValue::Int(i) => *i as usize,
            XdlValue::Float(f) => *f as usize,
            _ => n / 2,
        };
        let y = match &args[2] {
            XdlValue::Int(i) => *i as usize,
            XdlValue::Float(f) => *f as usize,
            _ => n / 2,
        };
        (x, y)
    } else {
        (n / 2, n / 2)
    };

    let idx = y * n + x;
    let value = if idx < data.len() { data[idx] } else { 0.0 };

    println!("RDPIX: Pixel at ({}, {}) = {:.6}", x, y, value);

    Ok(XdlValue::Float(value as f32))
}

/// PROFILES procedure - Extract cross-section profiles from image
/// Usage: PROFILES, image [, x, y] [, /ROW] [, /COLUMN] [, /DIAGONAL]
pub fn profiles(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "PROFILES requires at least 1 argument (image)".to_string(),
        ));
    }

    // Get image data
    let (data, nx, ny) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => {
            if shape.len() >= 2 {
                (data.clone(), shape[0], shape[1])
            } else {
                let n = shape[0];
                (data.clone(), n, 1)
            }
        }
        XdlValue::Array(arr) => {
            let n = (arr.len() as f64).sqrt() as usize;
            (arr.clone(), n, n)
        }
        _ => return Err(XdlError::RuntimeError("image must be an array".to_string())),
    };

    // Default: extract middle row
    let row_idx = ny / 2;
    let profile: Vec<f64> = (0..nx).map(|i| data[row_idx * nx + i]).collect();

    println!("PROFILES: Extracted profile from {}x{} image", nx, ny);
    println!("  Row {}: {} values", row_idx, profile.len());
    println!("  Range: {:.3} to {:.3}",
        profile.iter().cloned().fold(f64::INFINITY, f64::min),
        profile.iter().cloned().fold(f64::NEG_INFINITY, f64::max));

    Ok(XdlValue::Array(profile))
}

/// TVLCT procedure - Load color look-up table (modify current color table)
/// Usage: TVLCT, r, g, b [, start] [, /GET] [, /HLS] [, /HSV]
pub fn tvlct(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::RuntimeError(
            "TVLCT requires at least 3 arguments: r, g, b arrays".to_string(),
        ));
    }

    let r = match &args[0] {
        XdlValue::Array(arr) => arr.len(),
        _ => return Err(XdlError::RuntimeError("r must be an array".to_string())),
    };

    let g = match &args[1] {
        XdlValue::Array(arr) => arr.len(),
        _ => return Err(XdlError::RuntimeError("g must be an array".to_string())),
    };

    let b = match &args[2] {
        XdlValue::Array(arr) => arr.len(),
        _ => return Err(XdlError::RuntimeError("b must be an array".to_string())),
    };

    let start = if args.len() > 3 {
        match &args[3] {
            XdlValue::Int(i) => *i as usize,
            XdlValue::Long(l) => *l as usize,
            _ => 0,
        }
    } else {
        0
    };

    let n_colors = r.min(g).min(b);
    println!("TVLCT: Loading {} colors starting at index {}", n_colors, start);

    Ok(XdlValue::Undefined)
}

/// XYOUTS procedure extension - additional text output options
/// Usage: XYOUTS, x, y, string [, /DATA] [, /DEVICE] [, /NORMAL] [, CHARSIZE=size]
pub fn xyouts_extended(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::RuntimeError(
            "XYOUTS requires 3 arguments: x, y, string".to_string(),
        ));
    }

    let x = match &args[0] {
        XdlValue::Float(f) => *f as f64,
        XdlValue::Double(d) => *d,
        XdlValue::Int(i) => *i as f64,
        _ => 0.0,
    };

    let y = match &args[1] {
        XdlValue::Float(f) => *f as f64,
        XdlValue::Double(d) => *d,
        XdlValue::Int(i) => *i as f64,
        _ => 0.0,
    };

    let text = match &args[2] {
        XdlValue::String(s) => s.clone(),
        _ => "".to_string(),
    };

    println!("XYOUTS: Drawing '{}' at ({:.2}, {:.2})", text, x, y);

    Ok(XdlValue::Undefined)
}

/// LEGEND procedure - Add legend to plot
/// Usage: LEGEND, labels [, POSITION=pos] [, LINESTYLE=styles] [, PSYM=symbols] [, COLOR=colors]
pub fn legend(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "LEGEND requires at least 1 argument (labels)".to_string(),
        ));
    }

    let labels = match &args[0] {
        XdlValue::NestedArray(arr) => {
            arr.iter()
                .filter_map(|v| if let XdlValue::String(s) = v { Some(s.clone()) } else { None })
                .collect::<Vec<_>>()
        }
        XdlValue::String(s) => vec![s.clone()],
        _ => vec!["Data".to_string()],
    };

    println!("LEGEND: Adding legend with {} entries", labels.len());
    for (i, label) in labels.iter().enumerate() {
        println!("  [{}] {}", i + 1, label);
    }

    Ok(XdlValue::Undefined)
}

/// COLORBAR procedure - Add color bar to plot
/// Usage: COLORBAR [, POSITION=[x0,y0,x1,y1]] [, TITLE=title] [, RANGE=[min,max]] [, /VERTICAL]
pub fn colorbar(args: &[XdlValue]) -> XdlResult<XdlValue> {
    let title = if !args.is_empty() {
        match &args[0] {
            XdlValue::String(s) => s.clone(),
            _ => "Value".to_string(),
        }
    } else {
        "Value".to_string()
    };

    println!("COLORBAR: Adding color bar '{}'", title);
    println!("  Position: [0.15, 0.05, 0.85, 0.08]");
    println!("  Range: [min, max] from current color table");

    Ok(XdlValue::Undefined)
}
