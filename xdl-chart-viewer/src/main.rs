// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::{CommandFactory, Parser};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{State, WebviewUrl, WebviewWindowBuilder};

/// Command line arguments
#[derive(Parser, Debug)]
#[command(name = "xdl-chart-viewer")]
#[command(about = "XDL Chart Viewer - Display charts in native windows")]
#[command(version)]
struct Args {
    /// HTML file to display
    #[arg(short = 'f', long)]
    html_file: Option<String>,

    /// HTML content as base64 string
    #[arg(short = 'c', long)]
    html_content: Option<String>,

    /// Window title
    #[arg(short, long, default_value = "XDL Chart")]
    title: String,

    /// Window width
    #[arg(short, long, default_value_t = 1024)]
    width: u32,

    /// Window height
    #[arg(short = 'H', long, default_value_t = 768)]
    height: u32,

    /// Print help and exit (without launching GUI)
    #[arg(long = "help-only", hide = true)]
    help_only: bool,
}

/// Application state
#[derive(Default)]
struct AppState {
    window_counter: Arc<Mutex<usize>>,
}

/// Chart data structure for IPC
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChartData {
    title: String,
    html_content: String,
    width: u32,
    height: u32,
}

/// Tauri command to create a new chart window
#[tauri::command]
async fn create_chart_window(
    app: tauri::AppHandle,
    data: ChartData,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let window_id = {
        let mut counter = state.window_counter.lock().unwrap();
        *counter += 1;
        format!("chart-{}", *counter)
    };

    // Create data URL from HTML content
    let html_encoded = urlencoding::encode(&data.html_content);
    let data_url = format!("data:text/html;charset=utf-8,{}", html_encoded);

    // Create new window
    WebviewWindowBuilder::new(
        &app,
        &window_id,
        WebviewUrl::External(
            data_url
                .parse()
                .map_err(|e| format!("Invalid URL: {}", e))?,
        ),
    )
    .title(&data.title)
    .inner_size(data.width as f64, data.height as f64)
    .resizable(true)
    .visible(true)
    .build()
    .map_err(|e| format!("Failed to create window: {}", e))?;

    println!("Created chart window: {}", window_id);
    Ok(window_id)
}

fn main() {
    // Handle --help and --version early, before Tauri initialization
    // This allows tests to run without launching a GUI
    let args = match Args::try_parse() {
        Ok(args) => args,
        Err(e) => {
            // This handles --help and --version, printing and exiting
            e.exit();
        }
    };

    // Handle --help-only flag for testing purposes
    if args.help_only {
        Args::command().print_help().unwrap();
        println!();
        return;
    }

    // Prepare initial window data if provided
    let initial_html = if let Some(html_file) = &args.html_file {
        // Read from file
        std::fs::read_to_string(html_file).expect("Failed to read HTML file")
    } else if let Some(html_content) = &args.html_content {
        // Use content directly (no base64 decoding for simplicity)
        html_content.clone()
    } else {
        // Default demo chart
        create_demo_chart_html(&args.title)
    };

    let window_title = args.title.clone();
    let window_width = args.width;
    let window_height = args.height;

    tauri::Builder::default()
        .setup(move |app| {
            let window_id = "main";

            // Encode HTML as data URL
            let html_encoded = urlencoding::encode(&initial_html);
            let data_url = format!("data:text/html;charset=utf-8,{}", html_encoded);

            // Create main window
            WebviewWindowBuilder::new(app, window_id, WebviewUrl::External(data_url.parse()?))
                .title(&window_title)
                .inner_size(window_width as f64, window_height as f64)
                .resizable(true)
                .visible(true)
                .build()?;

            println!("XDL Chart Viewer started");
            println!("Window ID: {}", window_id);

            Ok(())
        })
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![create_chart_window])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Create a demo chart HTML for testing
fn create_demo_chart_html(title: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        body {{
            margin: 0;
            padding: 20px;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            min-height: 100vh;
        }}
        #main {{
            width: 900px;
            height: 600px;
            background-color: white;
            box-shadow: 0 10px 40px rgba(0,0,0,0.3);
            border-radius: 8px;
        }}
        .header {{
            text-align: center;
            color: white;
            margin-bottom: 30px;
        }}
        .header h1 {{
            font-size: 42px;
            margin: 0 0 10px 0;
            text-shadow: 2px 2px 4px rgba(0,0,0,0.2);
        }}
        .header p {{
            font-size: 18px;
            opacity: 0.9;
        }}
    </style>
    <script src="https://cdn.jsdelivr.net/npm/echarts@5/dist/echarts.min.js"></script>
</head>
<body>
    <div class="header">
        <h1>🚀 {title}</h1>
        <p>XDL Chart Viewer - Powered by Tauri + ECharts</p>
    </div>
    <div id="main"></div>

    <script type="text/javascript">
        const chartDom = document.getElementById('main');
        const myChart = echarts.init(chartDom);

        const xData = [];
        const sineData = [];
        const cosineData = [];

        for (let i = 0; i <= 100; i++) {{
            const x = i / 10;
            xData.push(x);
            sineData.push([x, Math.sin(x)]);
            cosineData.push([x, Math.cos(x)]);
        }}

        const option = {{
            title: {{
                text: 'Demo: Trigonometric Functions',
                left: 'center',
                textStyle: {{
                    fontSize: 20
                }}
            }},
            tooltip: {{
                trigger: 'axis',
                axisPointer: {{
                    type: 'cross'
                }}
            }},
            legend: {{
                data: ['sin(x)', 'cos(x)'],
                bottom: 20
            }},
            xAxis: {{
                type: 'value',
                name: 'x',
                nameLocation: 'middle',
                nameGap: 30
            }},
            yAxis: {{
                type: 'value',
                name: 'y',
                nameLocation: 'middle',
                nameGap: 40
            }},
            series: [
                {{
                    name: 'sin(x)',
                    type: 'line',
                    data: sineData,
                    smooth: true,
                    lineStyle: {{
                        color: '#5470c6',
                        width: 3
                    }},
                    itemStyle: {{
                        color: '#5470c6'
                    }}
                }},
                {{
                    name: 'cos(x)',
                    type: 'line',
                    data: cosineData,
                    smooth: true,
                    lineStyle: {{
                        color: '#91cc75',
                        width: 3
                    }},
                    itemStyle: {{
                        color: '#91cc75'
                    }}
                }}
            ],
            toolbox: {{
                feature: {{
                    dataZoom: {{
                        yAxisIndex: 'none'
                    }},
                    restore: {{}},
                    saveAsImage: {{}}
                }},
                right: 20
            }}
        }};

        myChart.setOption(option);

        window.addEventListener('resize', () => {{
            myChart.resize();
        }});

        console.log('XDL Chart initialized successfully');
    </script>
</body>
</html>"#,
        title = title
    )
}
