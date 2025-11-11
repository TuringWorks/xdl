# Advanced Visualization Quick Reference

## Overview

XDL now includes advanced scientific visualization capabilities including:

- Scientific colormaps (Viridis, Plasma, Inferno, etc.)
- Digital Elevation Model (DEM) rendering
- Terrain hillshade generation
- Vector field visualization (quiver plots)

These features are implemented in Rust for performance and integrate seamlessly with xdl-gui.

---

## RENDER_COLORMAP

**Purpose**: Render 2D data arrays with scientific colormaps

**Syntax**:

```idl
RENDER_COLORMAP, data, filename
```

**Parameters**:

- `data` - 2D float array (NxM)
- `filename` - Output PNG filename (string)

**Default Colormap**: Viridis (perceptually uniform)

**Example**:

```idl
; Create 2D data
data = FLTARR(50, 50)
FOR i=0, 49 DO FOR j=0, 49 DO $
  data[i,j] = SIN(i/5.0) * COS(j/5.0)

; Render with colormap
RENDER_COLORMAP, data, 'my_colormap.png'
```

**Output**: PNG image with color-coded data values

---

## DEM_RENDER

**Purpose**: Render digital elevation models with terrain colormaps

**Syntax**:

```idl
DEM_RENDER, elevation_data, filename
```

**Parameters**:

- `elevation_data` - 2D float array representing elevation (meters)
- `filename` - Output PNG filename (string)

**Colormap**: Terrain-specific (blue→green→brown→white)

**Example**:

```idl
; Create elevation data (mountains)
elevation = FLTARR(60, 60)
FOR i=0, 59 DO FOR j=0, 59 DO BEGIN
  x = (i - 30.0) / 10.0
  y = (j - 30.0) / 10.0
  elevation[i,j] = 100.0 * EXP(-(x*x + y*y) / 8.0) + 50.0
ENDFOR

; Render as DEM
DEM_RENDER, elevation, 'terrain.png'
```

**Output**: PNG image with elevation-coded colors

---

## HILLSHADE

**Purpose**: Generate hillshade (relief shading) from elevation data

**Syntax**:

```idl
HILLSHADE, elevation_data, filename
```

**Parameters**:

- `elevation_data` - 2D float array representing elevation
- `filename` - Output PNG filename (string)

**Sun Position** (default):

- Azimuth: 315° (northwest)
- Altitude: 45° (above horizon)

**Example**:

```idl
; Using same elevation data from DEM_RENDER
HILLSHADE, elevation, 'hillshade.png'
```

**Output**: Grayscale PNG showing terrain relief

**Use Case**: Combine with DEM for enhanced terrain visualization

---

## QUIVER

**Purpose**: Visualize 2D vector fields with arrow plots

**Syntax**:

```idl
QUIVER, u_component, v_component, filename
```

**Parameters**:

- `u_component` - 2D float array (x-direction velocities)
- `v_component` - 2D float array (y-direction velocities)
- `filename` - Output PNG filename (string)

**Colormap**: Plasma (color-codes vector magnitude)

**Example**:

```idl
; Create vortex vector field
size = 20
u = FLTARR(size, size)
v = FLTARR(size, size)

FOR i=0, size-1 DO FOR j=0, size-1 DO BEGIN
  x = (i - size/2.0)
  y = (j - size/2.0)
  r = SQRT(x*x + y*y) + 0.1
  u[i,j] = -y / r
  v[i,j] = x / r
ENDFOR

; Create quiver plot
QUIVER, u, v, 'vortex.png'
```

**Output**: PNG with arrows showing vector field direction and magnitude

---

## Complete Example Workflow

```idl
PRO demo_workflow

  ; 1. Generate synthetic terrain
  nx = 100
  ny = 100
  terrain = FLTARR(nx, ny)

  FOR i=0, nx-1 DO FOR j=0, ny-1 DO BEGIN
    x = (i - nx/2.0) / 20.0
    y = (j - ny/2.0) / 20.0
    terrain[i,j] = 500.0 * EXP(-(x*x + y*y) / 4.0) + 100.0
  ENDFOR

  ; 2. Visualize elevation
  DEM_RENDER, terrain, 'step1_elevation.png'

  ; 3. Generate hillshade for relief
  HILLSHADE, terrain, 'step2_hillshade.png'

  ; 4. Create wind field over terrain (simplified)
  u_wind = FLTARR(20, 20)
  v_wind = FLTARR(20, 20)

  FOR i=0, 19 DO FOR j=0, 19 DO BEGIN
    ; Simplified: wind flows around high terrain
    u_wind[i,j] = 5.0 + RANDOMU(seed) * 2.0
    v_wind[i,j] = 2.0 + RANDOMU(seed) * 1.0
  ENDFOR

  ; 5. Visualize wind field
  QUIVER, u_wind, v_wind, 'step3_wind.png'

  PRINT, 'Workflow complete!'

END
```

---

## Tips and Best Practices

### Data Range

- **RENDER_COLORMAP**: Automatically normalizes to [0, 1]
- **DEM_RENDER**: Best for elevation data (meters/feet)
- **HILLSHADE**: Works with any elevation units
- **QUIVER**: Automatically scales arrows

### Performance

- These functions are implemented in Rust for speed
- Can handle arrays up to ~1000x1000 efficiently
- Larger arrays will work but may take longer

### File Output

- All procedures output PNG format
- Images are automatically displayed in xdl-gui
- Files are saved to current working directory

### Integration with Existing Code

```idl
; Combine with PLOT for multi-panel displays
PLOT, x, y, TITLE='Time Series'

; Generate 2D analysis
analysis_data = FLTARR(50, 50)
; ... compute analysis ...
RENDER_COLORMAP, analysis_data, 'analysis.png'

; Compare with contour plot
CONTOUR, analysis_data
```

---

## Color Schemes

### Available in Rust Implementation

Current procedures use:

- **RENDER_COLORMAP**: Viridis (perceptually uniform, colorblind-friendly)
- **DEM_RENDER**: Terrain (blue→green→brown→white)
- **HILLSHADE**: Grayscale (black→white)
- **QUIVER**: Plasma (purple→pink→yellow)

### Future Enhancements

Additional colormaps available in underlying implementation:

- Inferno, Magma, Cividis, Turbo
- Ocean, Jet, Hot, Cool
- Diverging: RdBu, BrBG, PiYG, PRGn
- Discrete: Set1, Set2, Set3, Paired

**Note: To enable additional colormaps, add keyword parameter support to procedures*

---

## Common Use Cases

### 1. Climate/Weather Data

```idl
; Temperature field
RENDER_COLORMAP, temperature, 'temp_map.png'

; Wind vectors
QUIVER, u_wind, v_wind, 'wind_field.png'
```

### 2. Topography Analysis

```idl
; Elevation
DEM_RENDER, elevation, 'topo.png'

; Relief
HILLSHADE, elevation, 'relief.png'
```

### 3. Fluid Dynamics

```idl
; Velocity field
RENDER_COLORMAP, velocity_magnitude, 'velocity.png'

; Flow vectors
QUIVER, vx, vy, 'flow.png'
```

### 4. Scientific Data Analysis

```idl
; 2D experimental data
RENDER_COLORMAP, experiment_data, 'results.png'

; Gradient field
QUIVER, dx_data, dy_data, 'gradients.png'
```

---

## Running the Demo

```bash
# From xdl directory
cd examples
xdl-gui advanced_viz_demo.xdl
```

Or load in xdl-gui and execute interactively.

---

## Troubleshooting

### Issue: "Unknown procedure" error

**Solution**: Rebuild xdl-stdlib with `cargo build --release`

### Issue: Images not displaying

**Solution**: Check that xdl-gui GUI_IMAGE_CALLBACK is registered

### Issue: Array dimension mismatch

**Solution**: Ensure 2D arrays are properly created with FLTARR(nx, ny)

### Issue: QUIVER u/v size mismatch

**Solution**: Both u and v must have same dimensions

---

## Advanced Topics

### Underlying Rust Modules

The procedures use these modules in `xdl-stdlib/src/graphics/`:

- `colormap.rs` - 25+ scientific color schemes
- `terrain.rs` - DEM class with slope/aspect/contour methods
- `sciviz.rs` - Vector fields, streamlines, volume rendering
- `export.rs` - Multi-format export (PNG, SVG, HTML)

### Documentation

- Full API docs: `docs/SCIENTIFIC_VISUALIZATION_GUIDE.md`
- GIS features: `docs/GIS_SETUP.md` (optional, requires PROJ library)

### Contributing

To add new procedures:

1. Implement Rust function in `graphics_procs.rs`
2. Register in `lib.rs` `call_procedure()` match statement
3. Add example to `examples/`
4. Update this reference

---

## Related Commands

- `PLOT` - 2D line plots
- `CONTOUR` - Contour plots
- `SURFACE` - 3D surface plots
- `TV`/`TVSCL` - Image display
- `IMAGE_DISPLAY` - Advanced image display

---

Last updated: January 2025
