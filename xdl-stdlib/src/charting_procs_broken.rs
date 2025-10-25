//! Charting procedures using ECharts and Tauri
//!
//! Provides PLOT, SCATTER, BAR, SURFACE3D and other chart types

use anyhow::{anyhow, Context};
use std::process::Command;
use xdl_charts::{ChartConfig, ChartType, Series2D, Series3D};
use xdl_core::{XdlError, XdlResult, XdlValue};

/// Extract f64 array from XDL Value
fn extract_f64_array(value: &XdlValue) -> anyhow::Result<Vec<f64>> {
    match value {
        XdlValue::Array(arr) => Ok(arr.clone()),
        XdlValue::NestedArray(arr) => {
            arr.iter()
                .map(|v| v.to_double().map_err(|e| anyhow!("Array element error: {}", e)))
                .collect()
        }
        XdlValue::MultiDimArray { data, .. } => Ok(data.clone()),
        _ => Err(XdlError::RuntimeError(format!("Expected array")),
    }
}

/// Extract 2D array (matrix) from XDL Value
fn extract_2d_array(value: &XdlValue) -> anyhow::Result<Vec<Vec<f64>>> {
    match value {
        XdlValue::NestedArray(rows) => {
            rows.iter()
                .map(|row| extract_f64_array(row))
                .collect()
        }
        XdlValue::MultiDimArray { data, shape } => {
            if shape.len() != 2 {
                return Err(XdlError::RuntimeError(format!("Expected 2D array, got {}D", shape.len()));
            }
            let rows = shape[0];
            let cols = shape[1];
            Ok((0..rows)
                .map(|i| {
                    data[i * cols..(i + 1) * cols].to_vec()
                })
                .collect())
        }
        _ => Err(XdlError::RuntimeError(format!("Expected 2D array")),
    }
}

/// Extract string from Value
fn extract_string(value: &XdlValue) -> anyhow::Result<String> {
    match value {
        XdlValue::String(s) => Ok(s.clone()),
        _ => Err(XdlError::RuntimeError(format!("Expected string")),
    }
}

/// Launch chart in Tauri viewer
fn launch_chart(html: String, title: &str) -> anyhow::Result<()> {
    // Find xdl-chart-viewer binary
    let viewer_path = std::env::current_exe()?
        .parent()
        .ok_or_else(|| anyhow!("Cannot find parent directory"))?
        .join("xdl-chart-viewer");

    // Launch viewer with HTML content
    Command::new(viewer_path)
        .args(&["--html-content", &html, "--title", title])
        .spawn()
        .context("Failed to launch chart viewer")?;

    Ok(())
}

/// PLOT procedure - 2D line/scatter plot
///
/// Usage:
///   PLOT, x, y, TITLE='My Plot', TYPE='line'
///   PLOT, x, y  ; defaults to line chart
pub fn plot(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::RuntimeError(format!("PLOT requires at least 2 arguments: x, y"));
    }

    let x_data = extract_f64_array(&args[0])?;
    let y_data = extract_f64_array(&args[1])?;

    if x_data.len() != y_data.len() {
        return Err(XdlError::RuntimeError(format!("X and Y arrays must have same length"));
    }

    // Extract optional parameters (simplified - no keyword args yet)
    let title = if args.len() > 2 {
        extract_string(&args[2]).unwrap_or_else(|_| "XDL Plot".to_string())
    } else {
        "XDL Plot".to_string()
    };

    let chart_type = if args.len() > 3 {
        match extract_string(&args[3])?.to_lowercase().as_str() {
            "line" => ChartType::Line,
            "scatter" => ChartType::Scatter,
            "bar" => ChartType::Bar,
            _ => ChartType::Line,
        }
    } else {
        ChartType::Line
    };

    // Build chart configuration
    let config = ChartConfig {
        chart_type,
        title: title.clone(),
        x_label: Some("X".to_string()),
        y_label: Some("Y".to_string()),
        width: 1024,
        height: 768,
        use_webgl: false,
        ..Default::default()
    };

    let series = vec![Series2D {
        name: "Data".to_string(),
        x_data,
        y_data,
    }];

    let html = xdl_charts::generate_2d_chart(&config, &series)?;
    launch_chart(html, &title)?;

    Ok(XdlValue::Undefined)
}

/// SCATTER procedure - 2D scatter plot
///
/// Usage: SCATTER, x, y, TITLE='Scatter Plot'
pub fn scatter(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(XdlError::RuntimeError(format!("SCATTER requires at least 2 arguments: x, y"));
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
        x_label: Some("X".to_string()),
        y_label: Some("Y".to_string()),
        width: 1024,
        height: 768,
        use_webgl: x_data.len() > 10000, // Use WebGL for large datasets
        ..Default::default()
    };

    let series = vec![Series2D {
        name: "Points".to_string(),
        x_data,
        y_data,
    }];

    let html = xdl_charts::generate_2d_chart(&config, &series)?;
    launch_chart(html, &title)?;

    Ok(XdlValue::Undefined)
}

/// BAR procedure - Bar chart
///
/// Usage: BAR, values, TITLE='Bar Chart'
pub fn bar(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError(format!("BAR requires at least 1 argument: values"));
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
        x_label: Some("Category".to_string()),
        y_label: Some("Value".to_string()),
        width: 1024,
        height: 768,
        ..Default::default()
    };

    let series = vec![Series2D {
        name: "Values".to_string(),
        x_data,
        y_data,
    }];

    let html = xdl_charts::generate_2d_chart(&config, &series)?;
    launch_chart(html, &title)?;

    Ok(XdlValue::Undefined)
}

/// SURFACE3D procedure - 3D surface plot
///
/// Usage: SURFACE3D, z_matrix, TITLE='Surface'
pub fn surface3d(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError(format!("SURFACE3D requires at least 1 argument: z_matrix"));
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
        use_webgl: true, // 3D always uses WebGL
        ..Default::default()
    };

    let x_range = (0.0, cols as f64);
    let y_range = (0.0, rows as f64);

    let html = xdl_charts::generate_surface_plot(&config, &z_data, x_range, y_range)?;
    launch_chart(html, &title)?;

    Ok(XdlValue::Undefined)
}

/// SCATTER3D procedure - 3D scatter plot
///
/// Usage: SCATTER3D, x, y, z, TITLE='3D Scatter'
pub fn scatter3d(args: &[Value]) -> Result<Value> {
    if args.len() < 3 {
        return Err(XdlError::RuntimeError(format!("SCATTER3D requires at least 3 arguments: x, y, z"));
    }

    let x_data = extract_f64_array(&args[0])?;
    let y_data = extract_f64_array(&args[1])?;
    let z_data = extract_f64_array(&args[2])?;

    if x_data.len() != y_data.len() || y_data.len() != z_data.len() {
        return Err(XdlError::RuntimeError(format!("X, Y, and Z arrays must have same length"));
    }

    let title = if args.len() > 3 {
        extract_string(&args[3]).unwrap_or_else(|_| "3D Scatter".to_string())
    } else {
        "3D Scatter".to_string()
    };

    let config = ChartConfig {
        chart_type: ChartType::Scatter3D,
        title: title.clone(),
        x_label: Some("X".to_string()),
        y_label: Some("Y".to_string()),
        z_label: Some("Z".to_string()),
        width: 1024,
        height: 768,
        use_webgl: true,
        ..Default::default()
    };

    // Combine x, y, z into triplets
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

    let html = xdl_charts::generate_3d_chart(&config, &series)?;
    launch_chart(html, &title)?;

    Ok(XdlValue::Undefined)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_f64_array() {
        let arr = Value::Array(vec![
            Value::Float(1.0),
            Value::Float(2.0),
            Value::Int(3),
        ]);
        let result = extract_f64_array(&arr).unwrap();
        assert_eq!(result, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_extract_2d_array() {
        let mat = Value::Array(vec![
            Value::Array(vec![Value::Float(1.0), Value::Float(2.0)]),
            Value::Array(vec![Value::Float(3.0), Value::Float(4.0)]),
        ]);
        let result = extract_2d_array(&mat).unwrap();
        assert_eq!(result, vec![vec![1.0, 2.0], vec![3.0, 4.0]]);
    }
}
