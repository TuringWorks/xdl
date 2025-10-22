//! 2D plotting implementation

use super::state::{Color, LineStyle, PlotStyle, GRAPHICS_STATE};
use plotters::prelude::*;
use xdl_core::XdlResult;

/// 2D plot configuration
#[derive(Clone)]
pub struct Plot2DConfig {
    pub title: Option<String>,
    pub xtitle: Option<String>,
    pub ytitle: Option<String>,
    pub xrange: Option<(f64, f64)>,
    pub yrange: Option<(f64, f64)>,
    pub xlog: bool,
    pub ylog: bool,
    pub style: PlotStyle,
    pub background: Color,
    pub xstyle: i32,
    pub ystyle: i32,
    pub position: Option<(f64, f64, f64, f64)>,
    pub noerase: bool,
    pub isotropic: bool,
}

impl Default for Plot2DConfig {
    fn default() -> Self {
        Self {
            title: None,
            xtitle: None,
            ytitle: None,
            xrange: None,
            yrange: None,
            xlog: false,
            ylog: false,
            style: PlotStyle::default(),
            background: Color::new(255, 255, 255),
            xstyle: 0,
            ystyle: 0,
            position: None,
            noerase: false,
            isotropic: false,
        }
    }
}

/// Plot 2D line data
pub fn plot_2d(
    x_data: Vec<f64>,
    y_data: Vec<f64>,
    config: Plot2DConfig,
    filename: &str,
) -> XdlResult<()> {
    // Get window dimensions from graphics state
    let (width, height) = {
        let state = GRAPHICS_STATE.lock().unwrap();
        let win = state.get_current_window().unwrap();
        (win.width, win.height)
    };

    // Create drawing area
    let root = BitMapBackend::new(filename, (width, height)).into_drawing_area();
    root.fill(&config.background.to_rgb())?;

    // Calculate data ranges
    let x_min = x_data.iter().fold(f64::INFINITY, |a, &b| a.min(b)).floor();
    let x_max = x_data
        .iter()
        .fold(f64::NEG_INFINITY, |a, &b| a.max(b))
        .ceil();
    let y_min = y_data.iter().fold(f64::INFINITY, |a, &b| a.min(b)).floor();
    let y_max = y_data
        .iter()
        .fold(f64::NEG_INFINITY, |a, &b| a.max(b))
        .ceil();

    // Use configured ranges if provided
    let x_range = config.xrange.unwrap_or((x_min, x_max));
    let y_range = config.yrange.unwrap_or((y_min, y_max));

    // Build chart
    let mut chart = ChartBuilder::on(&root)
        .caption(
            config.title.as_deref().unwrap_or("XDL Plot"),
            ("sans-serif", 30),
        )
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(x_range.0..x_range.1, y_range.0..y_range.1)?;

    // Configure mesh (grid)
    chart
        .configure_mesh()
        .x_desc(config.xtitle.as_deref().unwrap_or("X"))
        .y_desc(config.ytitle.as_deref().unwrap_or("Y"))
        .draw()?;

    // Draw line series
    let line_color = config.style.color.to_rgb();
    let line_width = config.style.thick as u32;

    let line_style = ShapeStyle::from(&line_color).stroke_width(line_width);
    chart.draw_series(LineSeries::new(
        x_data.iter().zip(y_data.iter()).map(|(&x, &y)| (x, y)),
        line_style,
    ))?;

    // Draw symbols if requested
    if config.style.psym != 0 {
        draw_symbols(
            &mut chart,
            &x_data,
            &y_data,
            config.style.psym,
            config.style.symsize,
            &line_color,
        )?;
    }

    root.present()?;
    Ok(())
}

/// Get plotters line style from XDL line style (currently simplified)
fn _get_line_style(_style: LineStyle, thick: f64) -> ShapeStyle {
    let color = BLACK;
    // All line styles use solid for now - plotters has limited line style support
    ShapeStyle::from(&color).stroke_width(thick as u32)
}

/// Draw symbols at data points
fn draw_symbols<DB: DrawingBackend>(
    chart: &mut ChartContext<
        DB,
        Cartesian2d<plotters::coord::types::RangedCoordf64, plotters::coord::types::RangedCoordf64>,
    >,
    x_data: &[f64],
    y_data: &[f64],
    psym: i32,
    symsize: f64,
    color: &RGBColor,
) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>> {
    let size = (symsize * 5.0) as i32;

    let style = ShapeStyle::from(color).filled();

    for (x, y) in x_data.iter().zip(y_data.iter()) {
        match psym {
            1 => {
                // Plus sign
                chart.draw_series(std::iter::once(Cross::new((*x, *y), size, style)))?;
            }
            2 => {
                // Asterisk
                chart.draw_series(std::iter::once(Cross::new((*x, *y), size, style)))?;
            }
            3 => {
                // Dot
                chart.draw_series(std::iter::once(Circle::new((*x, *y), size / 2, style)))?;
            }
            4 => {
                // Diamond
                chart.draw_series(std::iter::once(TriangleMarker::new((*x, *y), size, style)))?;
            }
            5 => {
                // Triangle
                chart.draw_series(std::iter::once(TriangleMarker::new((*x, *y), size, style)))?;
            }
            6 => {
                // Square
                chart.draw_series(std::iter::once(Rectangle::new(
                    [
                        (*x - size as f64 / 100.0, *y - size as f64 / 100.0),
                        (*x + size as f64 / 100.0, *y + size as f64 / 100.0),
                    ],
                    style,
                )))?;
            }
            7 => {
                // X symbol
                chart.draw_series(std::iter::once(Cross::new((*x, *y), size, style)))?;
            }
            _ => {
                // Default to circle
                chart.draw_series(std::iter::once(Circle::new((*x, *y), size, style)))?;
            }
        }
    }
    Ok(())
}

/// Create histogram plot
pub fn histogram_plot(
    data: Vec<f64>,
    nbins: usize,
    config: Plot2DConfig,
    filename: &str,
) -> XdlResult<()> {
    // Calculate histogram
    let min_val = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_val = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let bin_width = (max_val - min_val) / nbins as f64;

    let mut bins = vec![0; nbins];
    for &val in &data {
        let bin = ((val - min_val) / bin_width).floor() as usize;
        if bin < nbins {
            bins[bin] += 1;
        }
    }

    // Get window dimensions
    let (width, height) = {
        let state = GRAPHICS_STATE.lock().unwrap();
        let win = state.get_current_window().unwrap();
        (win.width, win.height)
    };

    // Create drawing area
    let root = BitMapBackend::new(filename, (width, height)).into_drawing_area();
    root.fill(&config.background.to_rgb())?;

    // Build chart
    let max_count = *bins.iter().max().unwrap_or(&1) as f64;
    let mut chart = ChartBuilder::on(&root)
        .caption(
            config.title.as_deref().unwrap_or("Histogram"),
            ("sans-serif", 30),
        )
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(min_val..max_val, 0.0..(max_count * 1.1))?;

    chart.configure_mesh().draw()?;

    // Draw bars
    let bar_color = config.style.color.to_rgb();
    chart.draw_series(bins.iter().enumerate().map(|(i, &count)| {
        let x0 = min_val + i as f64 * bin_width;
        let x1 = x0 + bin_width;
        Rectangle::new(
            [(x0, 0.0), (x1, count as f64)],
            ShapeStyle::from(&bar_color).filled(),
        )
    }))?;

    root.present()?;
    Ok(())
}

/// Create bar plot
pub fn bar_plot(values: Vec<f64>, config: Plot2DConfig, filename: &str) -> XdlResult<()> {
    // Get window dimensions
    let (width, height) = {
        let state = GRAPHICS_STATE.lock().unwrap();
        let win = state.get_current_window().unwrap();
        (win.width, win.height)
    };

    // Create drawing area
    let root = BitMapBackend::new(filename, (width, height)).into_drawing_area();
    root.fill(&config.background.to_rgb())?;

    // Calculate ranges
    let max_val = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)) * 1.1;
    let min_val = values.iter().fold(f64::INFINITY, |a, &b| a.min(b)).min(0.0);

    // Build chart
    let mut chart = ChartBuilder::on(&root)
        .caption(
            config.title.as_deref().unwrap_or("Bar Plot"),
            ("sans-serif", 30),
        )
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..(values.len() as f64), min_val..max_val)?;

    chart.configure_mesh().draw()?;

    // Draw bars
    let bar_color = config.style.color.to_rgb();
    let bar_width = 0.8;

    chart.draw_series(values.iter().enumerate().map(|(i, &val)| {
        let x_center = i as f64 + 0.5;
        let x0 = x_center - bar_width / 2.0;
        let x1 = x_center + bar_width / 2.0;
        Rectangle::new(
            [(x0, 0.0), (x1, val)],
            ShapeStyle::from(&bar_color).filled(),
        )
    }))?;

    root.present()?;
    Ok(())
}

/// Plot with error bars
pub fn plot_with_errors(
    x_data: Vec<f64>,
    y_data: Vec<f64>,
    y_errors: Vec<f64>,
    config: Plot2DConfig,
    filename: &str,
) -> XdlResult<()> {
    // First plot the line
    plot_2d(x_data.clone(), y_data.clone(), config.clone(), filename)?;

    // Then add error bars (re-open the file and overlay)
    // For now, create a new plot with error bars
    let (width, height) = {
        let state = GRAPHICS_STATE.lock().unwrap();
        let win = state.get_current_window().unwrap();
        (win.width, win.height)
    };

    let root = BitMapBackend::new(filename, (width, height)).into_drawing_area();
    root.fill(&config.background.to_rgb())?;

    // Calculate ranges including error bars
    let y_min = y_data
        .iter()
        .zip(y_errors.iter())
        .map(|(y, e)| y - e)
        .fold(f64::INFINITY, |a, b| a.min(b));
    let y_max = y_data
        .iter()
        .zip(y_errors.iter())
        .map(|(y, e)| y + e)
        .fold(f64::NEG_INFINITY, |a, b| a.max(b));

    let x_min = x_data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let x_max = x_data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

    let mut chart = ChartBuilder::on(&root)
        .caption(
            config.title.as_deref().unwrap_or("Plot with Errors"),
            ("sans-serif", 30),
        )
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

    chart.configure_mesh().draw()?;

    // Draw data points with error bars
    let color = config.style.color.to_rgb();
    for ((x, y), err) in x_data.iter().zip(y_data.iter()).zip(y_errors.iter()) {
        // Vertical error bar
        chart.draw_series(std::iter::once(PathElement::new(
            vec![(*x, y - err), (*x, y + err)],
            &color,
        )))?;

        // Caps on error bars
        let cap_width = (x_max - x_min) * 0.01;
        chart.draw_series(std::iter::once(PathElement::new(
            vec![(*x - cap_width, y - err), (*x + cap_width, y - err)],
            &color,
        )))?;
        chart.draw_series(std::iter::once(PathElement::new(
            vec![(*x - cap_width, y + err), (*x + cap_width, y + err)],
            &color,
        )))?;

        // Data point
        chart.draw_series(std::iter::once(Circle::new(
            (*x, *y),
            3,
            ShapeStyle::from(&color).filled(),
        )))?;
    }

    root.present()?;
    Ok(())
}
