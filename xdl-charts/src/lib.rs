//! XDL Charts - ECharts-based visualization generation
//!
//! This crate provides chart generation using Apache ECharts,
//! supporting 2D plots, 3D visualizations, and dashboards.

use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod echarts;
pub mod templates;

/// Chart type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChartType {
    /// 2D line chart
    Line,
    /// 2D scatter plot
    Scatter,
    /// Bar chart
    Bar,
    /// Area chart
    Area,
    /// Heatmap
    Heatmap,
    /// 3D scatter plot
    Scatter3D,
    /// 3D surface plot
    Surface3D,
    /// 3D bar chart
    Bar3D,
}

/// Chart configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartConfig {
    /// Chart type
    pub chart_type: ChartType,
    /// Chart title
    pub title: String,
    /// X-axis label
    pub x_label: Option<String>,
    /// Y-axis label
    pub y_label: Option<String>,
    /// Z-axis label (for 3D charts)
    pub z_label: Option<String>,
    /// Chart width in pixels
    pub width: u32,
    /// Chart height in pixels
    pub height: u32,
    /// Use WebGL renderer (for better performance)
    pub use_webgl: bool,
    /// Color scheme/theme
    pub theme: String,
}

impl Default for ChartConfig {
    fn default() -> Self {
        Self {
            chart_type: ChartType::Line,
            title: "XDL Chart".to_string(),
            x_label: None,
            y_label: None,
            z_label: None,
            width: 800,
            height: 600,
            use_webgl: false,
            theme: "default".to_string(),
        }
    }
}

/// Data series for 2D charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Series2D {
    /// Series name
    pub name: String,
    /// X data points
    pub x_data: Vec<f64>,
    /// Y data points
    pub y_data: Vec<f64>,
}

/// Data series for 3D charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Series3D {
    /// Series name
    pub name: String,
    /// Data points as [x, y, z] triplets
    pub data: Vec<[f64; 3]>,
}

/// Generate HTML for a 2D chart
pub fn generate_2d_chart(config: &ChartConfig, series: &[Series2D]) -> Result<String> {
    let echarts_option = echarts::build_2d_option(config, series)?;
    let html = templates::create_echarts_html(config, &echarts_option)?;
    Ok(html)
}

/// Generate HTML for a 3D chart
pub fn generate_3d_chart(config: &ChartConfig, series: &[Series3D]) -> Result<String> {
    let echarts_option = echarts::build_3d_option(config, series)?;
    let html = templates::create_echarts_html(config, &echarts_option)?;
    Ok(html)
}

/// Generate HTML for a surface plot from a 2D matrix
pub fn generate_surface_plot(
    config: &ChartConfig,
    z_data: &[Vec<f64>],
    x_range: (f64, f64),
    y_range: (f64, f64),
) -> Result<String> {
    let echarts_option = echarts::build_surface_option(config, z_data, x_range, y_range)?;
    let html = templates::create_echarts_html(config, &echarts_option)?;
    Ok(html)
}

/// Generate HTML for a heatmap/contour plot
pub fn generate_heatmap(config: &ChartConfig, data: &[[f64; 3]]) -> Result<String> {
    let echarts_option = echarts::build_heatmap_option(config, data)?;
    let html = templates::create_echarts_html(config, &echarts_option)?;
    Ok(html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chart_config_default() {
        let config = ChartConfig::default();
        assert_eq!(config.chart_type, ChartType::Line);
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
    }

    #[test]
    fn test_series_2d_creation() {
        let series = Series2D {
            name: "Test".to_string(),
            x_data: vec![1.0, 2.0, 3.0],
            y_data: vec![4.0, 5.0, 6.0],
        };
        assert_eq!(series.x_data.len(), 3);
        assert_eq!(series.y_data.len(), 3);
    }
}
