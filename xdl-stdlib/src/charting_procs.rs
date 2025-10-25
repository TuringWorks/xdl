//! Charting procedures using ECharts and Tauri

use std::process::Command;
use xdl_charts::{ChartConfig, ChartType, Series2D, Series3D};
use xdl_core::{XdlError, XdlResult, XdlValue};

/// Extract f64 array from XDL Value
fn extract_f64_array(value: &XdlValue) -> XdlResult<Vec<f64>> {
    match value {
        XdlValue::Array(arr) => Ok(arr.clone()),
        XdlValue::NestedArray(arr) => arr.iter().map(|v| v.to_double()).collect(),
        XdlValue::MultiDimArray { data, .. } => Ok(data.clone()),
        _ => Err(XdlError::RuntimeError("Expected array".to_string())),
    }
}

/// Extract 2D array (matrix) from XDL Value
fn extract_2d_array(value: &XdlValue) -> XdlResult<Vec<Vec<f64>>> {
    match value {
        XdlValue::NestedArray(rows) => rows.iter().map(extract_f64_array).collect(),
        XdlValue::MultiDimArray { data, shape } => {
            if shape.len() != 2 {
                return Err(XdlError::RuntimeError(format!(
                    "Expected 2D array, got {}D",
                    shape.len()
                )));
            }
            let rows = shape[0];
            let cols = shape[1];
            Ok((0..rows)
                .map(|i| data[i * cols..(i + 1) * cols].to_vec())
                .collect())
        }
        _ => Err(XdlError::RuntimeError("Expected 2D array".to_string())),
    }
}

/// Extract string from Value
fn extract_string(value: &XdlValue) -> XdlResult<String> {
    match value {
        XdlValue::String(s) => Ok(s.clone()),
        _ => Err(XdlError::RuntimeError("Expected string".to_string())),
    }
}

/// Launch chart in Tauri viewer
fn launch_chart(html: String, title: &str) -> XdlResult<()> {
    use std::fs;
    use std::io::Write;

    let viewer_path = std::env::current_exe()
        .map_err(|e| XdlError::RuntimeError(format!("Cannot find exe: {}", e)))?
        .parent()
        .ok_or_else(|| XdlError::RuntimeError("Cannot find parent directory".to_string()))?
        .join("xdl-chart-viewer");

    // Write HTML to a temporary file to avoid command-line argument length limits
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("xdl_chart_{}.html", std::process::id()));

    let mut file = fs::File::create(&temp_file)
        .map_err(|e| XdlError::RuntimeError(format!("Failed to create temp file: {}", e)))?;
    file.write_all(html.as_bytes())
        .map_err(|e| XdlError::RuntimeError(format!("Failed to write temp file: {}", e)))?;

    // Launch viewer with temp file path
    Command::new(viewer_path)
        .args(["--html-file", temp_file.to_str().unwrap(), "--title", title])
        .spawn()
        .map_err(|e| XdlError::RuntimeError(format!("Failed to launch chart viewer: {}", e)))?;

    Ok(())
}

/// PLOT procedure - 2D line/scatter plot
pub fn plot(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::RuntimeError(
            "PLOT requires at least 2 arguments: x, y".to_string(),
        ));
    }

    let x_data = extract_f64_array(&args[0])?;
    let y_data = extract_f64_array(&args[1])?;

    if x_data.len() != y_data.len() {
        return Err(XdlError::RuntimeError(
            "X and Y arrays must have same length".to_string(),
        ));
    }

    let title = if args.len() > 2 {
        extract_string(&args[2]).unwrap_or_else(|_| "XDL Plot".to_string())
    } else {
        "XDL Plot".to_string()
    };

    let config = ChartConfig {
        chart_type: ChartType::Line,
        title: title.clone(),
        x_label: Some("X".to_string()),
        y_label: Some("Y".to_string()),
        width: 1024,
        height: 768,
        ..Default::default()
    };

    let series = vec![Series2D {
        name: "Data".to_string(),
        x_data,
        y_data,
    }];

    let html = xdl_charts::generate_2d_chart(&config, &series)
        .map_err(|e| XdlError::RuntimeError(format!("Chart generation failed: {}", e)))?;
    launch_chart(html, &title)?;

    Ok(XdlValue::Undefined)
}

/// SCATTER procedure
pub fn scatter(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::RuntimeError(
            "SCATTER requires at least 2 arguments: x, y".to_string(),
        ));
    }

    let x_data = extract_f64_array(&args[0])?;
    let y_data = extract_f64_array(&args[1])?;

    let title = if args.len() > 2 {
        extract_string(&args[2]).unwrap_or_else(|_| "Scatter Plot".to_string())
    } else {
        "Scatter Plot".to_string()
    };

    let config = ChartConfig {
        chart_type: ChartType::Scatter,
        title: title.clone(),
        width: 1024,
        height: 768,
        use_webgl: x_data.len() > 10000,
        ..Default::default()
    };

    let series = vec![Series2D {
        name: "Points".to_string(),
        x_data,
        y_data,
    }];

    let html = xdl_charts::generate_2d_chart(&config, &series)
        .map_err(|e| XdlError::RuntimeError(format!("Chart generation failed: {}", e)))?;
    launch_chart(html, &title)?;

    Ok(XdlValue::Undefined)
}

/// BAR procedure
pub fn bar(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "BAR requires at least 1 argument: values".to_string(),
        ));
    }

    let y_data = extract_f64_array(&args[0])?;
    let x_data: Vec<f64> = (0..y_data.len()).map(|i| i as f64).collect();

    let title = if args.len() > 1 {
        extract_string(&args[1]).unwrap_or_else(|_| "Bar Chart".to_string())
    } else {
        "Bar Chart".to_string()
    };

    let config = ChartConfig {
        chart_type: ChartType::Bar,
        title: title.clone(),
        width: 1024,
        height: 768,
        ..Default::default()
    };

    let series = vec![Series2D {
        name: "Values".to_string(),
        x_data,
        y_data,
    }];

    let html = xdl_charts::generate_2d_chart(&config, &series)
        .map_err(|e| XdlError::RuntimeError(format!("Chart generation failed: {}", e)))?;
    launch_chart(html, &title)?;

    Ok(XdlValue::Undefined)
}

/// SURFACE3D procedure
pub fn surface3d(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "SURFACE3D requires at least 1 argument: z_matrix".to_string(),
        ));
    }

    let z_data = extract_2d_array(&args[0])?;
    let title = if args.len() > 1 {
        extract_string(&args[1]).unwrap_or_else(|_| "3D Surface".to_string())
    } else {
        "3D Surface".to_string()
    };

    let rows = z_data.len();
    let cols = if rows > 0 { z_data[0].len() } else { 0 };

    let config = ChartConfig {
        chart_type: ChartType::Surface3D,
        title: title.clone(),
        x_label: Some("X".to_string()),
        y_label: Some("Y".to_string()),
        z_label: Some("Z".to_string()),
        width: 1024,
        height: 768,
        use_webgl: true,
        ..Default::default()
    };

    let x_range = (0.0, cols as f64);
    let y_range = (0.0, rows as f64);

    let html = xdl_charts::generate_surface_plot(&config, &z_data, x_range, y_range)
        .map_err(|e| XdlError::RuntimeError(format!("Chart generation failed: {}", e)))?;
    launch_chart(html, &title)?;

    Ok(XdlValue::Undefined)
}

/// SCATTER3D procedure
pub fn scatter3d(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::RuntimeError(
            "SCATTER3D requires at least 3 arguments: x, y, z".to_string(),
        ));
    }

    let x_data = extract_f64_array(&args[0])?;
    let y_data = extract_f64_array(&args[1])?;
    let z_data = extract_f64_array(&args[2])?;

    if x_data.len() != y_data.len() || y_data.len() != z_data.len() {
        return Err(XdlError::RuntimeError(
            "X, Y, and Z arrays must have same length".to_string(),
        ));
    }

    let title = if args.len() > 3 {
        extract_string(&args[3]).unwrap_or_else(|_| "3D Scatter".to_string())
    } else {
        "3D Scatter".to_string()
    };

    let config = ChartConfig {
        chart_type: ChartType::Scatter3D,
        title: title.clone(),
        width: 1024,
        height: 768,
        use_webgl: true,
        ..Default::default()
    };

    let data: Vec<[f64; 3]> = x_data
        .into_iter()
        .zip(y_data)
        .zip(z_data)
        .map(|((x, y), z)| [x, y, z])
        .collect();

    let series = vec![Series3D {
        name: "Points".to_string(),
        data,
    }];

    let html = xdl_charts::generate_3d_chart(&config, &series)
        .map_err(|e| XdlError::RuntimeError(format!("Chart generation failed: {}", e)))?;
    launch_chart(html, &title)?;

    Ok(XdlValue::Undefined)
}

/// CONTOUR procedure - 2D contour/heatmap
pub fn contour(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "CONTOUR requires at least 1 argument: z_matrix".to_string(),
        ));
    }

    let z_data = extract_2d_array(&args[0])?;
    let title = "Contour Plot".to_string();

    // Flatten the 2D array for heatmap
    let mut flat_data = Vec::new();
    for (i, row) in z_data.iter().enumerate() {
        for (j, &value) in row.iter().enumerate() {
            flat_data.push([j as f64, i as f64, value]);
        }
    }

    let config = ChartConfig {
        chart_type: ChartType::Heatmap,
        title: title.clone(),
        x_label: Some("X".to_string()),
        y_label: Some("Y".to_string()),
        width: 1024,
        height: 768,
        ..Default::default()
    };

    let html = xdl_charts::generate_heatmap(&config, &flat_data)
        .map_err(|e| XdlError::RuntimeError(format!("Chart generation failed: {}", e)))?;
    launch_chart(html, &title)?;

    Ok(XdlValue::Undefined)
}

/// SHADE_SURF procedure - shaded 3D surface (alias for SURFACE3D)
pub fn shade_surf(args: &[XdlValue]) -> XdlResult<XdlValue> {
    // SHADE_SURF is essentially the same as a 3D surface with shading
    // Just use surface3d with a different default title
    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "SHADE_SURF requires at least 1 argument: z_matrix".to_string(),
        ));
    }

    let z_data = extract_2d_array(&args[0])?;
    let title = "Shaded Surface".to_string();

    let rows = z_data.len();
    let cols = if rows > 0 { z_data[0].len() } else { 0 };

    let config = ChartConfig {
        chart_type: ChartType::Surface3D,
        title: title.clone(),
        x_label: Some("X".to_string()),
        y_label: Some("Y".to_string()),
        z_label: Some("Z".to_string()),
        width: 1024,
        height: 768,
        use_webgl: true,
        ..Default::default()
    };

    let x_range = (0.0, cols as f64);
    let y_range = (0.0, rows as f64);

    let html = xdl_charts::generate_surface_plot(&config, &z_data, x_range, y_range)
        .map_err(|e| XdlError::RuntimeError(format!("Chart generation failed: {}", e)))?;
    launch_chart(html, &title)?;

    Ok(XdlValue::Undefined)
}

/// PLOT3D procedure - 3D line plot
pub fn plot3d(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::RuntimeError(
            "PLOT3D requires at least 3 arguments: x, y, z".to_string(),
        ));
    }

    let x_data = extract_f64_array(&args[0])?;
    let y_data = extract_f64_array(&args[1])?;
    let z_data = extract_f64_array(&args[2])?;

    if x_data.len() != y_data.len() || y_data.len() != z_data.len() {
        return Err(XdlError::RuntimeError(
            "X, Y, and Z arrays must have same length".to_string(),
        ));
    }

    let title = "3D Line Plot".to_string();

    let config = ChartConfig {
        chart_type: ChartType::Scatter3D, // Use scatter3D with line connection
        title: title.clone(),
        width: 1024,
        height: 768,
        use_webgl: true,
        ..Default::default()
    };

    let data: Vec<[f64; 3]> = x_data
        .into_iter()
        .zip(y_data)
        .zip(z_data)
        .map(|((x, y), z)| [x, y, z])
        .collect();

    let series = vec![Series3D {
        name: "Line".to_string(),
        data,
    }];

    let html = xdl_charts::generate_3d_chart(&config, &series)
        .map_err(|e| XdlError::RuntimeError(format!("Chart generation failed: {}", e)))?;
    launch_chart(html, &title)?;

    Ok(XdlValue::Undefined)
}
