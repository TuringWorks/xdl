//! HTML template generation for ECharts
//!
//! Creates complete HTML pages with embedded ECharts visualizations

use crate::ChartConfig;
use anyhow::Result;
use serde_json::Value;

/// Create a complete HTML page with ECharts visualization
pub fn create_echarts_html(config: &ChartConfig, option: &Value) -> Result<String> {
    let option_json = serde_json::to_string_pretty(option)?;

    // Choose ECharts script based on whether we need 3D support
    let (echarts_script, echarts_gl_script) = if config.use_webgl || needs_3d_support(config) {
        (
            r#"<script src="https://cdn.jsdelivr.net/npm/echarts@5/dist/echarts.min.js"></script>"#,
            r#"<script src="https://cdn.jsdelivr.net/npm/echarts-gl@2/dist/echarts-gl.min.js"></script>"#,
        )
    } else {
        (
            r#"<script src="https://cdn.jsdelivr.net/npm/echarts@5/dist/echarts.min.js"></script>"#,
            "",
        )
    };

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        body {{
            margin: 0;
            padding: 0;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
            background-color: #ffffff;
            overflow: hidden;
        }}
        #main {{
            width: 100vw;
            height: 100vh;
        }}
    </style>
    {echarts_script}
    {echarts_gl_script}
</head>
<body>
    <div id="main"></div>

    <script type="text/javascript">
        // Initialize ECharts instance
        const chartDom = document.getElementById('main');
        const myChart = echarts.init(chartDom{renderer});

        // Chart option - title will be rendered by ECharts
        const option = {option_json};

        // Set option and display chart
        myChart.setOption(option);

        // Make chart responsive
        window.addEventListener('resize', function() {{
            myChart.resize();
        }});

        // Log for debugging
        console.log('XDL Chart initialized successfully');
        console.log('Chart type:', option.series[0]?.type || 'unknown');
    </script>
</body>
</html>"#,
        title = config.title,
        echarts_script = echarts_script,
        echarts_gl_script = echarts_gl_script,
        renderer = if config.use_webgl {
            ", { renderer: 'canvas' }"
        } else {
            ""
        },
        option_json = option_json,
    );

    Ok(html)
}

/// Check if the chart requires 3D support (ECharts GL)
fn needs_3d_support(config: &ChartConfig) -> bool {
    matches!(
        config.chart_type,
        crate::ChartType::Scatter3D | crate::ChartType::Surface3D | crate::ChartType::Bar3D
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ChartConfig, ChartType};

    #[test]
    fn test_needs_3d_support() {
        let config = ChartConfig {
            chart_type: ChartType::Line,
            ..Default::default()
        };
        assert!(!needs_3d_support(&config));

        let config = ChartConfig {
            chart_type: ChartType::Scatter3D,
            ..Default::default()
        };
        assert!(needs_3d_support(&config));

        let config = ChartConfig {
            chart_type: ChartType::Surface3D,
            ..Default::default()
        };
        assert!(needs_3d_support(&config));
    }

    #[test]
    fn test_html_generation() {
        let config = ChartConfig::default();
        let option = serde_json::json!({
            "series": [{"type": "line", "data": [1, 2, 3]}]
        });

        let html = create_echarts_html(&config, &option).unwrap();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("echarts"));
        assert!(html.contains(&config.title));
    }
}
