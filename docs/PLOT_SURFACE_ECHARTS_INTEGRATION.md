# PLOT and SURFACE xdl-charts Integration

## Overview

The traditional `PLOT` and `SURFACE` procedures have been enhanced to use the modern xdl-charts/Tauri system by default, with automatic fallback to the original graphics rendering when Tauri is unavailable.

## What Changed

### Before

- `PLOT` and `SURFACE` generated static PNG images
- Required GUI callbacks for interactive display
- Limited interactivity
- 2D surface rendering for 3D plots

### After

- **Default**: Interactive Tauri windows with ECharts rendering
- **Fallback**: Original PNG generation if Tauri unavailable
- Full 3D interactivity with rotation, zoom, pan
- WebGL acceleration for large datasets
- Modern web-based charting

## Usage

No code changes required! Existing scripts continue to work:

```idl
; Simple line plot
x = FINDGEN(100) / 10.0
y = SIN(x)
PLOT, x, y

; Auto-generate X coordinates
y2 = COS(x)
PLOT, y2

; 3D surface plot
z = FLTARR(40, 40)
FOR i=0, 39 DO FOR j=0, 39 DO BEGIN
  z[i,j] = SIN(SQRT((i-20)^2 + (j-20)^2) / 5.0)
ENDFOR
SURFACE, z
```

## Benefits

1. **Better Interactivity**: Tauri windows provide native desktop integration
2. **3D Support**: True 3D rendering with ECharts GL
3. **Performance**: WebGL acceleration for large datasets
4. **Backwards Compatible**: Automatic fallback ensures old scripts work
5. **Modern UI**: Clean, responsive charts with built-in controls

## Fallback Behavior

If xdl-chart-viewer is not available or fails to launch:

- Logs a message to stderr: `"PLOT: xdl-charts unavailable (...), using fallback renderer"`
- Automatically uses original graphics::plot_2d/surface_plot
- Generates PNG files as before
- Works with GUI callbacks if registered

## Architecture

```text
PLOT/SURFACE procedure
  ↓
try_chart_plot / try_surface3d (xdl-charts)
  ↓
  Success? → Open Tauri window → Return
  ↓
  Failure? → Log error → Continue
  ↓
graphics::plot_2d / surface_plot (fallback)
  ↓
Generate PNG file → Try GUI callbacks → Return
```

## Implementation Details

### Code Locations

- **Main procedures**: `xdl-stdlib/src/graphics_procs.rs`
  - `plot_with_keywords()` - Enhanced PLOT procedure
  - `surface()` - Enhanced SURFACE procedure
  - `try_chart_plot()` - xdl-charts helper for PLOT
  - `try_surface3d()` - xdl-charts helper for SURFACE

- **Charting backend**: `xdl-stdlib/src/charting_procs.rs`
  - `plot()` - ECharts line chart generator
  - `surface3d()` - ECharts 3D surface generator

### Key Functions

#### try_chart_plot(x_data, y_data, title) -> Result<(), XdlError>

- Converts arrays to XdlValue
- Calls charting_procs::plot()
- Returns error if unavailable (triggers fallback)

#### try_surface3d(z_data, title) -> Result<(), XdlError>

- Converts 2D array to XdlValue
- Calls charting_procs::surface3d()
- Returns error if unavailable (triggers fallback)

## Testing

Run the test script:

```bash
./target/release/xdl examples/charting/test_plot_surface.xdl
```

Expected output:

- 3 Tauri windows open (2 line plots, 1 surface plot)
- Interactive charts with zoom, pan, rotation
- No fallback messages (if xdl-chart-viewer is available)

## Future Enhancements

Potential improvements:

1. Support keyword arguments (TITLE, XTITLE, YTITLE) in xdl-charts
2. Add CONTOUR procedure support
3. Multi-series plotting in single window
4. Export charts from Tauri window (PNG, SVG, PDF)
5. Real-time data updates

## Compatibility

- ✅ Maintains full backward compatibility
- ✅ Works with existing XDL/IDL scripts
- ✅ No breaking changes
- ✅ Graceful degradation when Tauri unavailable
- ✅ Preserves GUI callback system for custom integrations

## Related Procedures

New charting procedures that use xdl-charts directly:

- `CHART_PLOT` - Direct access to ECharts line plotting
- `CHART_SCATTER` - Scatter plots
- `CHART_BAR` - Bar charts
- `SURFACE3D` - Direct 3D surface plotting
- `SCATTER3D` - 3D scatter plots

These provide the same underlying functionality but with explicit names.
