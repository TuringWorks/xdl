//! Graphics and plotting functions

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use xdl_core::{XdlError, XdlResult, XdlValue};

static GUI_PLOT_CALLBACK: Mutex<
    Option<Arc<dyn Fn(Vec<f64>, Vec<f64>, String, String, String) + Send + Sync>>,
> = Mutex::new(None);

static GUI_IMAGE_CALLBACK: Mutex<Option<Arc<dyn Fn(String, String) + Send + Sync>>> =
    Mutex::new(None);

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
pub fn plot(args: &[XdlValue]) -> XdlResult<XdlValue> {
    plot_with_keywords(args, &HashMap::new())
}

/// Plot procedure with keyword arguments support
pub fn plot_with_keywords(
    args: &[XdlValue],
    keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    use crate::graphics::{plot_2d, Plot2DConfig};

    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "PLOT requires at least one argument".to_string(),
        ));
    }

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

    // Extract keyword arguments for plot configuration
    let mut config = Plot2DConfig::default();

    // Extract title
    if let Some(title_val) = keywords.get("title").or_else(|| keywords.get("TITLE")) {
        if let XdlValue::String(s) = title_val {
            config.title = Some(s.clone());
        }
    }

    // Extract x-axis title
    if let Some(xtitle_val) = keywords.get("xtitle").or_else(|| keywords.get("XTITLE")) {
        if let XdlValue::String(s) = xtitle_val {
            config.xtitle = Some(s.clone());
        }
    }

    // Extract y-axis title
    if let Some(ytitle_val) = keywords.get("ytitle").or_else(|| keywords.get("YTITLE")) {
        if let XdlValue::String(s) = ytitle_val {
            config.ytitle = Some(s.clone());
        }
    }

    // Generate output filename
    let filename = "xdl_plot.png";

    // Ensure graphics state is initialized - it's already done by default
    // The GRAPHICS_STATE is initialized with window 0 by default

    // Extract labels before moving config
    let title = config
        .title
        .clone()
        .unwrap_or_else(|| "XDL Plot".to_string());
    let xtitle = config.xtitle.clone().unwrap_or_else(|| "X".to_string());
    let ytitle = config.ytitle.clone().unwrap_or_else(|| "Y".to_string());

    // Create the plot using the 2D plotting function
    println!("PLOT: Rendering {} points to {}", x_data.len(), filename);
    plot_2d(x_data.clone(), y_data.clone(), config, filename)?;
    println!("  Plot saved to '{}'", filename);

    // Try to launch interactive plot window if callback is available
    if let Ok(callback_guard) = GUI_PLOT_CALLBACK.lock() {
        if let Some(ref callback) = *callback_guard {
            callback(x_data, y_data, title, xtitle, ytitle);
            return Ok(XdlValue::Undefined);
        }
    }

    // Try to display image in GUI if callback is available
    if let Ok(callback_guard) = GUI_IMAGE_CALLBACK.lock() {
        if let Some(ref callback) = *callback_guard {
            callback(filename.to_string(), "XDL Plot".to_string());
        }
    }

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

/// Helper function to extract 2D array from nested array or MultiDimArray
fn extract_2d_array(value: &XdlValue) -> XdlResult<Vec<Vec<f64>>> {
    match value {
        XdlValue::MultiDimArray { data, shape } => {
            // Handle MultiDimArray from REFORM
            if shape.len() != 2 {
                return Err(XdlError::RuntimeError(format!(
                    "Expected 2D array, got {}D array",
                    shape.len()
                )));
            }

            let height = shape[0];
            let width = shape[1];

            if data.len() != height * width {
                return Err(XdlError::RuntimeError(format!(
                    "Array size {} doesn't match dimensions {}x{}",
                    data.len(),
                    height,
                    width
                )));
            }

            // Convert flat array to 2D Vec<Vec<f64>>
            let mut result = Vec::with_capacity(height);
            for i in 0..height {
                let row_start = i * width;
                let row_end = row_start + width;
                result.push(data[row_start..row_end].to_vec());
            }
            Ok(result)
        }
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
        _ => Err(XdlError::RuntimeError(
            "Expected a 2D nested array or MultiDimArray".to_string(),
        )),
    }
}

/// Register GUI plot callback (called from GUI application)
/// This is the main integration point between graphics procedures and the GUI
pub fn register_gui_plot_callback<F>(callback: F)
where
    F: Fn(Vec<f64>, Vec<f64>, String, String, String) + Send + Sync + 'static,
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
    let mut config = SurfaceConfig::default();
    config.shading = true;

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

// ============================================================================
// ADVANCED VISUALIZATION PROCEDURES
// ============================================================================

/// RENDER_COLORMAP procedure - renders data with specified colormap
/// Usage: RENDER_COLORMAP, data, filename, COLORMAP='viridis'
pub fn render_colormap(args: &[XdlValue]) -> XdlResult<XdlValue> {
    use crate::graphics::colormap::{ColorMap, ColorMapType};
    use plotters::prelude::*;

    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "RENDER_COLORMAP: Expected at least 2 arguments (data, filename)".to_string(),
        ));
    }

    let data_2d = extract_2d_array(&args[0])?;
    let filename = match &args[1] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::InvalidArgument(
                "Filename must be a string".to_string(),
            ))
        }
    };

    // Default colormap
    let colormap = ColorMap::new(ColorMapType::Viridis);

    // Find min/max of data
    let mut min_val = f64::INFINITY;
    let mut max_val = f64::NEG_INFINITY;
    for row in &data_2d {
        for &val in row {
            min_val = min_val.min(val);
            max_val = max_val.max(val);
        }
    }

    let height = data_2d.len();
    let width = if height > 0 { data_2d[0].len() } else { 0 };

    // Create drawing
    let root = BitMapBackend::new(&filename, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Colormap Visualization", ("sans-serif", 30))
        .margin(20)
        .build_cartesian_2d(0.0..width as f64, 0.0..height as f64)?;

    chart.configure_mesh().draw()?;

    // Draw colored rectangles
    for (y, row) in data_2d.iter().enumerate() {
        for (x, &val) in row.iter().enumerate() {
            let normalized = if max_val > min_val {
                (val - min_val) / (max_val - min_val)
            } else {
                0.5
            };

            let color = colormap.map(normalized);
            let rgb = RGBColor(color.r, color.g, color.b);

            chart.draw_series(std::iter::once(Rectangle::new(
                [(x as f64, y as f64), (x as f64 + 1.0, y as f64 + 1.0)],
                ShapeStyle::from(&rgb).filled(),
            )))?;
        }
    }

    root.present()?;

    println!("RENDER_COLORMAP: Saved to {}", filename);

    // Try to display in GUI
    if let Ok(callback_guard) = GUI_IMAGE_CALLBACK.lock() {
        if let Some(ref callback) = *callback_guard {
            callback(filename.clone(), "Colormap Visualization".to_string());
        }
    }

    Ok(XdlValue::Undefined)
}

/// DEM_RENDER procedure - renders digital elevation model
/// Usage: DEM_RENDER, elevation_data, filename
pub fn dem_render(args: &[XdlValue]) -> XdlResult<XdlValue> {
    use crate::graphics::colormap::terrain;
    use crate::graphics::terrain::{render_elevation_map, DigitalElevationModel};

    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "DEM_RENDER: Expected at least 2 arguments (elevation_data, filename)".to_string(),
        ));
    }

    let elevations = extract_2d_array(&args[0])?;
    let filename = match &args[1] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::InvalidArgument(
                "Filename must be a string".to_string(),
            ))
        }
    };

    let dem = DigitalElevationModel::new(elevations, 30.0)?;
    let colormap = terrain();

    render_elevation_map(&dem, &colormap, &filename)?;

    println!("DEM_RENDER: Elevation map saved to {}", filename);

    // Try to display in GUI
    if let Ok(callback_guard) = GUI_IMAGE_CALLBACK.lock() {
        if let Some(ref callback) = *callback_guard {
            callback(filename, "Digital Elevation Model".to_string());
        }
    }

    Ok(XdlValue::Undefined)
}

/// HILLSHADE procedure - generates hillshade from elevation data
/// Usage: HILLSHADE, elevation_data, filename, AZIMUTH=315, ALTITUDE=45
pub fn hillshade_proc(args: &[XdlValue]) -> XdlResult<XdlValue> {
    use crate::graphics::terrain::{render_hillshade, DigitalElevationModel};

    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "HILLSHADE: Expected at least 2 arguments (elevation_data, filename)".to_string(),
        ));
    }

    let elevations = extract_2d_array(&args[0])?;
    let filename = match &args[1] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::InvalidArgument(
                "Filename must be a string".to_string(),
            ))
        }
    };

    let dem = DigitalElevationModel::new(elevations, 30.0)?;

    // Default sun position
    let azimuth = 315.0; // NW
    let altitude = 45.0; // 45 degrees above horizon

    render_hillshade(&dem, azimuth, altitude, &filename)?;

    println!("HILLSHADE: Hillshade saved to {}", filename);

    // Try to display in GUI
    if let Ok(callback_guard) = GUI_IMAGE_CALLBACK.lock() {
        if let Some(ref callback) = *callback_guard {
            callback(filename, "Hillshade".to_string());
        }
    }

    Ok(XdlValue::Undefined)
}

/// QUIVER procedure - creates quiver (arrow) plot from vector field
/// Usage: QUIVER, u, v, filename
pub fn quiver_proc(args: &[XdlValue]) -> XdlResult<XdlValue> {
    use crate::graphics::colormap::plasma;
    use crate::graphics::sciviz::{render_quiver, VectorField2D};

    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(
            "QUIVER: Expected at least 3 arguments (u, v, filename)".to_string(),
        ));
    }

    let u = extract_2d_array(&args[0])?;
    let v = extract_2d_array(&args[1])?;
    let filename = match &args[2] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::InvalidArgument(
                "Filename must be a string".to_string(),
            ))
        }
    };

    let field = VectorField2D::new(u, v)?;
    let colormap = plasma();

    render_quiver(&field, 5, 2.0, Some(&colormap), &filename)?;

    println!("QUIVER: Vector field plot saved to {}", filename);

    // Try to display in GUI
    if let Ok(callback_guard) = GUI_IMAGE_CALLBACK.lock() {
        if let Some(ref callback) = *callback_guard {
            callback(filename, "Vector Field".to_string());
        }
    }

    Ok(XdlValue::Undefined)
}
