//! ECharts option generation
//!
//! Builds ECharts configuration objects from chart data

use crate::{ChartConfig, ChartType, Series2D, Series3D};
use anyhow::Result;
use serde_json::{json, Value};

/// Build ECharts option for 2D charts
pub fn build_2d_option(config: &ChartConfig, series: &[Series2D]) -> Result<Value> {
    let series_data: Vec<Value> = series
        .iter()
        .map(|s| {
            let data: Vec<Vec<f64>> = s
                .x_data
                .iter()
                .zip(&s.y_data)
                .map(|(x, y)| vec![*x, *y])
                .collect();

            json!({
                "name": s.name,
                "type": chart_type_to_string(config.chart_type),
                "data": data,
                "symbolSize": match config.chart_type {
                    ChartType::Scatter => 8,
                    _ => 4,
                },
                "smooth": matches!(config.chart_type, ChartType::Line | ChartType::Area),
            })
        })
        .collect();

    let option = json!({
        "title": {
            "text": config.title,
            "left": "center",
        },
        "tooltip": {
            "trigger": "axis",
            "axisPointer": {
                "type": "cross"
            }
        },
        "legend": {
            "data": series.iter().map(|s| &s.name).collect::<Vec<_>>(),
            "bottom": 10,
        },
        "xAxis": {
            "type": "value",
            "name": config.x_label.as_deref().unwrap_or("X"),
            "nameLocation": "middle",
            "nameGap": 30,
        },
        "yAxis": {
            "type": "value",
            "name": config.y_label.as_deref().unwrap_or("Y"),
            "nameLocation": "middle",
            "nameGap": 50,
        },
        "series": series_data,
        "toolbox": {
            "feature": {
                "dataZoom": {
                    "yAxisIndex": "none"
                },
                "restore": {},
                "saveAsImage": {}
            }
        }
    });

    Ok(option)
}

/// Build ECharts option for 3D scatter plots
pub fn build_3d_option(config: &ChartConfig, series: &[Series3D]) -> Result<Value> {
    let series_data: Vec<Value> = series
        .iter()
        .map(|s| {
            json!({
                "name": s.name,
                "type": "scatter3D",
                "data": s.data,
                "symbolSize": 5,
                "itemStyle": {
                    "opacity": 0.8
                },
                "emphasis": {
                    "itemStyle": {
                        "color": "#fff"
                    }
                }
            })
        })
        .collect();

    let option = json!({
        "title": {
            "text": config.title,
        },
        "tooltip": {},
        "legend": {
            "data": series.iter().map(|s| &s.name).collect::<Vec<_>>(),
        },
        "xAxis3D": {
            "name": config.x_label.as_deref().unwrap_or("X"),
            "type": "value"
        },
        "yAxis3D": {
            "name": config.y_label.as_deref().unwrap_or("Y"),
            "type": "value"
        },
        "zAxis3D": {
            "name": config.z_label.as_deref().unwrap_or("Z"),
            "type": "value"
        },
        "grid3D": {
            "viewControl": {
                "projection": "perspective",
                "autoRotate": false,
                "distance": 200
            },
            "boxWidth": 100,
            "boxHeight": 100,
            "boxDepth": 100,
        },
        "series": series_data,
    });

    Ok(option)
}

/// Build ECharts option for surface plots
pub fn build_surface_option(
    config: &ChartConfig,
    z_data: &[Vec<f64>],
    x_range: (f64, f64),
    y_range: (f64, f64),
) -> Result<Value> {
    // Generate grid data for surface plot
    let x_count = if !z_data.is_empty() {
        z_data[0].len()
    } else {
        0
    };
    let y_count = z_data.len();

    let mut surface_data: Vec<Vec<f64>> = Vec::new();
    for (i, row) in z_data.iter().enumerate() {
        let y = y_range.0 + (y_range.1 - y_range.0) * i as f64 / (y_count - 1) as f64;
        for (j, &z) in row.iter().enumerate() {
            let x = x_range.0 + (x_range.1 - x_range.0) * j as f64 / (x_count - 1) as f64;
            surface_data.push(vec![x, y, z]);
        }
    }

    let option = json!({
        "title": {
            "text": config.title,
        },
        "tooltip": {},
        "visualMap": {
            "show": true,
            "dimension": 2,
            "min": z_data.iter().flat_map(|row| row.iter()).cloned().fold(f64::INFINITY, f64::min),
            "max": z_data.iter().flat_map(|row| row.iter()).cloned().fold(f64::NEG_INFINITY, f64::max),
            "inRange": {
                "color": [
                    "#313695", "#4575b4", "#74add1", "#abd9e9", "#e0f3f8",
                    "#ffffbf", "#fee090", "#fdae61", "#f46d43", "#d73027", "#a50026"
                ]
            }
        },
        "xAxis3D": {
            "name": config.x_label.as_deref().unwrap_or("X"),
            "type": "value",
            "min": x_range.0,
            "max": x_range.1,
        },
        "yAxis3D": {
            "name": config.y_label.as_deref().unwrap_or("Y"),
            "type": "value",
            "min": y_range.0,
            "max": y_range.1,
        },
        "zAxis3D": {
            "name": config.z_label.as_deref().unwrap_or("Z"),
            "type": "value"
        },
        "grid3D": {
            "viewControl": {
                "projection": "perspective",
                "autoRotate": false,
                "distance": 200,
                "alpha": 30,
                "beta": 40,
            },
            "boxWidth": 100,
            "boxHeight": 100,
            "boxDepth": 100,
        },
        "series": [{
            "type": "surface",
            "wireframe": {
                "show": false
            },
            "shading": "color",
            "data": surface_data,
        }]
    });

    Ok(option)
}

/// Build ECharts option for heatmap/contour plots
pub fn build_heatmap_option(config: &ChartConfig, data: &[[f64; 3]]) -> Result<Value> {
    // data format: [[x, y, value], ...]
    let option = json!({
        "title": {
            "text": config.title,
            "left": "center",
        },
        "tooltip": {
            "position": "top"
        },
        "grid": {
            "height": "50%",
            "top": "10%"
        },
        "xAxis": {
            "type": "category",
            "name": config.x_label.as_deref().unwrap_or("X"),
            "splitArea": {
                "show": true
            }
        },
        "yAxis": {
            "type": "category",
            "name": config.y_label.as_deref().unwrap_or("Y"),
            "splitArea": {
                "show": true
            }
        },
        "visualMap": {
            "min": data.iter().map(|d| d[2]).fold(f64::INFINITY, f64::min),
            "max": data.iter().map(|d| d[2]).fold(f64::NEG_INFINITY, f64::max),
            "calculable": true,
            "orient": "horizontal",
            "left": "center",
            "bottom": "15%",
            "inRange": {
                "color": [
                    "#313695", "#4575b4", "#74add1", "#abd9e9", "#e0f3f8",
                    "#ffffbf", "#fee090", "#fdae61", "#f46d43", "#d73027", "#a50026"
                ]
            }
        },
        "series": [{
            "name": "Contour",
            "type": "heatmap",
            "data": data,
            "label": {
                "show": false
            },
            "emphasis": {
                "itemStyle": {
                    "shadowBlur": 10,
                    "shadowColor": "rgba(0, 0, 0, 0.5)"
                }
            }
        }]
    });

    Ok(option)
}

/// Convert ChartType to ECharts type string
fn chart_type_to_string(chart_type: ChartType) -> &'static str {
    match chart_type {
        ChartType::Line => "line",
        ChartType::Scatter => "scatter",
        ChartType::Bar => "bar",
        ChartType::Area => "line", // Area is line with areaStyle
        ChartType::Heatmap => "heatmap",
        ChartType::Scatter3D => "scatter3D",
        ChartType::Surface3D => "surface",
        ChartType::Bar3D => "bar3D",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chart_type_conversion() {
        assert_eq!(chart_type_to_string(ChartType::Line), "line");
        assert_eq!(chart_type_to_string(ChartType::Scatter), "scatter");
        assert_eq!(chart_type_to_string(ChartType::Surface3D), "surface");
    }
}
