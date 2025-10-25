#!/bin/bash
# Test xdl-charts + xdl-chart-viewer integration

cd /Users/ravindraboddipalli/sources/xdl

# Generate a test chart using xdl-charts
cat > /tmp/test_chart_gen.rs << 'EOF'
use xdl_charts::{ChartConfig, ChartType, Series2D, generate_2d_chart};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = ChartConfig::default();
    config.chart_type = ChartType::Scatter;
    config.title = "Test Scatter Plot from Rust".to_string();
    config.width = 900;
    config.height = 600;

    // Generate some test data
    let x_data: Vec<f64> = (0..50).map(|i| i as f64 / 5.0).collect();
    let y_data: Vec<f64> = x_data.iter().map(|&x| (x * 0.5).sin() + x * 0.1).collect();

    let series = vec![Series2D {
        name: "Test Data".to_string(),
        x_data,
        y_data,
    }];

    let html = generate_2d_chart(&config, &series)?;
    std::fs::write("/tmp/test_chart.html", html)?;
    println!("Chart HTML generated at /tmp/test_chart.html");

    Ok(())
}
EOF

# Compile and run the test
echo "Generating chart HTML..."
rustc --edition 2021 /tmp/test_chart_gen.rs \
    --extern xdl_charts=target/debug/libxdl_charts.rlib \
    --extern serde=target/debug/deps/libserde-*.rlib \
    --extern serde_json=target/debug/deps/libserde_json-*.rlib \
    --extern anyhow=target/debug/deps/libanyhow-*.rlib \
    -L target/debug/deps \
    -o /tmp/test_chart_gen 2>/dev/null

if [ $? -eq 0 ]; then
    /tmp/test_chart_gen
    echo ""
    echo "Now launching in Tauri viewer..."
    ./target/debug/xdl-chart-viewer -f /tmp/test_chart.html --title "XDL Charts Test"
else
    echo "Compilation failed, showing default demo instead..."
    ./target/debug/xdl-chart-viewer --title "XDL Default Demo"
fi
