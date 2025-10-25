# XDL ECharts Integration Examples

This directory contains example scripts demonstrating the ECharts charting integration with Tauri.

## Files

### simple_test.xdl
Quick validation script with a single line plot.
```bash
xdl simple_test.xdl
```

### echarts_demo.xdl
Comprehensive demonstration of all chart types:
1. Line Plot (Sine Wave)
2. Scatter Plot (Random Data)
3. Bar Chart
4. 3D Surface Plot (Ripple Pattern)
5. 3D Scatter Plot (Helix)
6. Multi-Series Plot
7. Large Dataset Scatter (15K points, WebGL)
8. Complex 3D Surface (Saddle Function)

```bash
xdl echarts_demo.xdl
```

## Available Procedures

### CHART_PLOT
2D line plot
```xdl
x = FINDGEN(100)
y = SIN(x / 10.0)
CHART_PLOT, x, y, 'My Plot'
```

### CHART_SCATTER
2D scatter plot with automatic WebGL for large datasets (>10K points)
```xdl
x = RANDOMU(seed, 100) * 10
y = RANDOMU(seed, 100) * 10
CHART_SCATTER, x, y, 'Scatter Plot'
```

### CHART_BAR
Bar chart
```xdl
values = [10, 20, 15, 35, 28]
CHART_BAR, values, 'Bar Chart'
```

### SURFACE3D
3D surface plot from 2D matrix
```xdl
z = FLTARR(50, 50)
FOR i=0, 49 DO FOR j=0, 49 DO $
    z[i,j] = SIN(i/5.0) * COS(j/5.0)
SURFACE3D, z, '3D Surface'
```

### SCATTER3D
3D scatter plot
```xdl
x = RANDOMU(seed, 100) * 10
y = RANDOMU(seed, 100) * 10
z = RANDOMU(seed, 100) * 10
SCATTER3D, x, y, z, '3D Scatter'
```

## Features

- ✅ Native Tauri windows (no browser dependency)
- ✅ Interactive charts (zoom, pan, rotate for 3D)
- ✅ WebGL acceleration for large datasets
- ✅ Professional ECharts styling
- ✅ Non-blocking (script continues while charts are open)
- ✅ Multiple windows support

## Requirements

- XDL with chart integration
- xdl-chart-viewer binary in same directory as xdl binary
- macOS (or Linux/Windows with Tauri support)

## Troubleshooting

**Chart window doesn't open:**
- Check that `xdl-chart-viewer` is in your PATH or next to `xdl` binary
- Try running `xdl-chart-viewer --title Test` manually to verify it works

**Script errors:**
- Ensure arrays have matching dimensions for 2D plots
- Check that 2D arrays (matrices) are properly formatted for SURFACE3D

## Performance

- 2D plots: ~100ms render time
- 3D plots: ~200ms render time
- Large scatter (15K points): ~500ms with WebGL acceleration
- Surface plots: ~300-500ms depending on resolution

## Next Steps

Try modifying the examples to create your own visualizations!
