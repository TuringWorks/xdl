# XDL Scientific Visualization Guide

## Overview

XDL now includes enterprise-grade scientific visualization capabilities including:

1. **Advanced Color Mapping** - Perceptually uniform, colorblind-friendly color schemes
2. **GIS and Cartography** - Map projections, coordinate transformations, geographic visualization
3. **Terrain Visualization** - Digital Elevation Models (DEM), hillshade, 3D terrain rendering
4. **Scientific Visualization** - Vector fields, streamlines, volume rendering, isosurfaces
5. **Multi-Format Export** - PNG, SVG, and interactive HTML output

---

## 1. Color Mapping (`colormap.rs`)

### Overview
Provides scientifically-validated color schemes based on research in perceptual uniformity and accessibility.

### Available Color Maps

#### Sequential (Single Hue)
- **Viridis** - Default, perceptually uniform
- **Plasma** - Purple to yellow
- **Inferno** - Black to white through orange
- **Magma** - Black to white through purple/pink
- **Cividis** - Optimized for colorblind viewers

#### Sequential (Multi-Hue)
- **Turbo** - Improved rainbow
- **Rainbow** - Traditional HSV-based
- **Jet** - Classic (not perceptually uniform)

#### Diverging
- **RdBu** - Red-Blue diverging
- **BrBG** - Brown-Blue-Green
- **PiYG** - Pink-Yellow-Green
- **PRGn** - Purple-Green
- **RdYlBu** - Red-Yellow-Blue

#### Terrain
- **Terrain** - Elevation coloring (blue→green→brown→white)
- **Ocean** - Bathymetry (deep blue to light blue)
- **Topography** - Combined ocean + terrain

### Usage Example

```rust
use xdl_stdlib::graphics::colormap::{ColorMap, ColorMapType, viridis};

// Use built-in color map
let cmap = viridis();
let color = cmap.map(0.75); // Get color at 75% through the map

// Create custom color map
let colors = vec![
    Color::new(255, 0, 0),    // Red
    Color::new(255, 255, 0),  // Yellow
    Color::new(0, 255, 0),    // Green
];
let custom_cmap = ColorMap::custom(colors);

// Reverse a color map
let reversed = viridis().reversed();

// Generate discrete color table
let color_table = cmap.generate_table(256);
```

---

## 2. GIS and Cartography (`gis.rs`)

### Supported Map Projections

#### Cylindrical
- **Mercator** - Conformal, preserves angles
- **MillerCylindrical** - Compromise projection
- **PlateCarree** - Equirectangular

#### Conic
- **LambertConformal** - Used for aeronautical charts
- **AlbersEqualArea** - Preserves area

#### Azimuthal
- **Stereographic** - Perspective projection
- **Orthographic** - Globe-like view
- **Gnomonic** - Great circles as straight lines
- **AzimuthalEquidistant** - Preserves distances from center

#### Pseudocylindrical
- **Mollweide** - Equal-area
- **Sinusoidal** - Equal-area
- **Robinson** - Compromise projection

### Usage Example

```rust
use xdl_stdlib::graphics::gis::{MapProjection, ProjectionType, CoastlineData};

// Create a Mercator projection centered on Greenwich
let mut projection = MapProjection::new(
    ProjectionType::Mercator,
    (0.0, 0.0)  // (lon, lat)
)?;

// Set map limits (lon_min, lat_min, lon_max, lat_max)
projection.set_limits((-180.0, -90.0, 180.0, 90.0));

// Project a coordinate
let (x, y) = projection.project(-74.0, 40.7).unwrap(); // NYC

// Load coastline data
let coastlines = CoastlineData::load_world();
// Or from GeoJSON
let coastlines = CoastlineData::from_geojson(geojson_string)?;

// Draw map with coastlines
draw_map(&projection, &coastlines, "world_map.png")?;

// Draw graticule (grid lines)
draw_graticule(&projection, 30.0, 30.0, "graticule.png")?;

// Plot data on map
let lons = vec![-74.0, 0.0, 139.7]; // NYC, London, Tokyo
let lats = vec![40.7, 51.5, 35.7];
let values = vec![8.3, 8.9, 13.5]; // Population in millions

map_scatter(&projection, &lons, &lats, Some(&values), Some(&viridis()), "cities.png")?;
```

### GeoJSON Support

```rust
// Load GeoJSON features
let geojson = r#"{
  "type": "FeatureCollection",
  "features": [...]
}"#;

let coastlines = CoastlineData::from_geojson(geojson)?;
```

---

## 3. Terrain Visualization (`terrain.rs`)

### Digital Elevation Models (DEM)

```rust
use xdl_stdlib::graphics::terrain::{DigitalElevationModel, render_elevation_map};
use xdl_stdlib::graphics::colormap::terrain;

// Create DEM from elevation data (2D array)
let elevations = vec![
    vec![100.0, 150.0, 200.0, 250.0],
    vec![120.0, 170.0, 220.0, 270.0],
    vec![140.0, 190.0, 240.0, 290.0],
];
let cell_size = 30.0; // meters per cell

let dem = DigitalElevationModel::new(elevations, cell_size)?;
```

### Hillshade Generation

```rust
// Generate hillshade with sun position
let azimuth = 315.0;   // NW direction (degrees)
let altitude = 45.0;   // 45° above horizon

render_hillshade(&dem, azimuth, altitude, "hillshade.png")?;
```

### Elevation Mapping

```rust
// Render elevation with color mapping
let cmap = terrain_colormap();
render_elevation_map(&dem, &cmap, "elevation.png")?;
```

### Shaded Relief (Combined)

```rust
// Combine elevation colors with hillshading
let blend_factor = 0.7; // 0.0 = pure color, 1.0 = pure shading

render_shaded_relief(
    &dem,
    &cmap,
    azimuth,
    altitude,
    blend_factor,
    "shaded_relief.png"
)?;
```

### Contour Lines

```rust
// Generate contour lines at specific elevations
let contour_levels = vec![100.0, 150.0, 200.0, 250.0, 300.0];
generate_contours(&dem, &contour_levels, "contours.png")?;
```

### 3D Terrain Rendering

```rust
// Render 3D terrain with vertical exaggeration
let azimuth = 45.0;           // View azimuth
let elevation = 30.0;         // View elevation
let vertical_exag = 2.0;      // 2x vertical exaggeration

render_terrain_3d(
    &dem,
    &viridis(),
    azimuth,
    elevation,
    vertical_exag,
    "terrain_3d.png"
)?;
```

### DEM Analysis

```rust
// Get terrain metrics
let slope = dem.calculate_slope(10, 10);        // Slope at grid position
let aspect = dem.calculate_aspect(10, 10);      // Aspect (direction)
let elevation = dem.get_elevation(10, 10)?;     // Raw elevation
let normalized = dem.get_normalized(10, 10)?;   // Normalized [0,1]
```

---

## 4. Scientific Visualization (`sciviz.rs`)

### Vector Fields

#### 2D Vector Field Visualization

```rust
use xdl_stdlib::graphics::sciviz::{VectorField2D, render_quiver};

// Create vector field (U and V components)
let u = vec![
    vec![1.0, 0.5, 0.0],
    vec![0.5, 0.0, -0.5],
    vec![0.0, -0.5, -1.0],
];
let v = vec![
    vec![0.0, 0.5, 1.0],
    vec![0.5, 0.0, -0.5],
    vec![1.0, 0.5, 0.0],
];

let field = VectorField2D::new(u, v)?;

// Render as quiver plot (arrows)
let subsample = 1;  // Plot every Nth point
let scale = 2.0;    // Arrow scaling
render_quiver(&field, subsample, scale, Some(&viridis()), "quiver.png")?;
```

#### Streamlines

```rust
use xdl_stdlib::graphics::sciviz::render_streamlines;

// Define starting points for streamlines
let start_points = vec![
    (0.5, 0.5),
    (1.0, 0.5),
    (1.5, 0.5),
];

let step_size = 0.1;
let max_steps = 100;

render_streamlines(
    &field,
    &start_points,
    step_size,
    max_steps,
    Some(&plasma()),
    "streamlines.png"
)?;
```

#### Single Streamline Integration

```rust
use xdl_stdlib::graphics::sciviz::integrate_streamline;

// Integrate a single streamline
let streamline = integrate_streamline(&field, 0.5, 0.5, 0.1, 100);
// Returns: Vec<(f64, f64)> of (x, y) coordinates
```

### Volume Rendering

#### 3D Scalar Fields

```rust
use xdl_stdlib::graphics::sciviz::{ScalarField3D, render_volume_mip};

// Create 3D scalar field
let data = vec![
    vec![
        vec![1.0, 2.0, 3.0],
        vec![2.0, 3.0, 4.0],
    ],
    vec![
        vec![3.0, 4.0, 5.0],
        vec![4.0, 5.0, 6.0],
    ],
];

let field = ScalarField3D::new(data)?;
```

#### Maximum Intensity Projection (MIP)

```rust
// Project along Z axis (0=X, 1=Y, 2=Z)
let axis = 2;
render_volume_mip(&field, axis, &inferno(), "volume_mip.png")?;
```

#### Isosurface Extraction

```rust
use xdl_stdlib::graphics::sciviz::render_isosurface_slice;

let isovalue = 4.0;      // Value to extract
let slice_axis = 2;      // Slice along Z
let slice_index = 1;     // Middle slice

render_isosurface_slice(
    &field,
    isovalue,
    slice_axis,
    slice_index,
    "isosurface.png"
)?;
```

### 3D Vector Fields

```rust
use xdl_stdlib::graphics::sciviz::VectorField3D;

let u = vec![/* 3D array */];
let v = vec![/* 3D array */];
let w = vec![/* 3D array */];

let field_3d = VectorField3D::new(u, v, w)?;

// Get vector and magnitude
let (u, v, w) = field_3d.get_vector(5, 5, 5)?;
let magnitude = field_3d.get_magnitude(5, 5, 5)?;
```

---

## 5. Multi-Format Export (`export.rs`)

### Export Formats

```rust
use xdl_stdlib::graphics::export::{ExportConfig, ExportFormat};

// PNG export (default)
let config = ExportConfig::new(ExportFormat::PNG)
    .with_size(1920, 1080)
    .with_dpi(300);

// SVG export (vector graphics)
let config = ExportConfig::new(ExportFormat::SVG)
    .with_size(800, 600);
```

### Interactive HTML Export

```rust
use xdl_stdlib::graphics::export::export_to_html;

// Convert SVG to interactive HTML
export_to_html(
    "plot.svg",
    "plot.html",
    "My Scientific Plot",
    800,
    600
)?;
```

The generated HTML includes:
- Embedded SVG plot
- Download buttons for SVG and PNG
- Responsive layout
- Professional styling

---

## Complete Examples

### Example 1: Climate Data Visualization

```rust
// Load temperature data (lat x lon grid)
let temperatures = load_climate_data("temperature.nc");
let lons = linspace(-180.0, 180.0, temperatures[0].len());
let lats = linspace(-90.0, 90.0, temperatures.len());

// Create map projection
let mut projection = MapProjection::new(ProjectionType::Robinson, (0.0, 0.0))?;
projection.set_limits((-180.0, -90.0, 180.0, 90.0));

// Plot temperature on map
let flat_temps: Vec<f64> = temperatures.into_iter().flatten().collect();
map_scatter(&projection, &lons, &lats, Some(&flat_temps), Some(&turbo()), "climate.png")?;
```

### Example 2: Terrain Analysis

```rust
// Load DEM
let dem = DigitalElevationModel::from_geotiff("terrain.tif")?;

// Generate multiple visualizations
render_elevation_map(&dem, &terrain_colormap(), "elevation.png")?;
render_hillshade(&dem, 315.0, 45.0, "hillshade.png")?;
render_shaded_relief(&dem, &terrain_colormap(), 315.0, 45.0, 0.7, "shaded.png")?;

// Extract contours
let levels: Vec<f64> = (0..10).map(|i| dem.min_elevation + i as f64 * 100.0).collect();
generate_contours(&dem, &levels, "contours.png")?;
```

### Example 3: Fluid Flow Visualization

```rust
// Simulate fluid flow
let (u_field, v_field) = simulate_fluid_flow(100, 100);
let field = VectorField2D::new(u_field, v_field)?;

// Render quiver plot
render_quiver(&field, 5, 2.0, Some(&plasma()), "velocity_field.png")?;

// Generate streamlines
let starts: Vec<(f64, f64)> = (0..20)
    .map(|i| (10.0, i as f64 * 5.0))
    .collect();

render_streamlines(&field, &starts, 0.5, 200, Some(&viridis()), "streamlines.png")?;
```

### Example 4: Atmospheric Data

```rust
// 3D atmospheric pressure field
let pressure_data = load_atmospheric_data("pressure.nc");
let field = ScalarField3D::new(pressure_data)?;

// Render different views
render_volume_mip(&field, 0, &inferno(), "pressure_x.png")?;  // X projection
render_volume_mip(&field, 1, &inferno(), "pressure_y.png")?;  // Y projection
render_volume_mip(&field, 2, &inferno(), "pressure_z.png")?;  // Z projection

// Extract pressure isosurface at 1013 hPa
render_isosurface_slice(&field, 1013.0, 2, 50, "isobar_1013.png")?;
```

---

## Best Practices

### Color Map Selection

1. **Sequential data** (e.g., elevation, temperature): Use `viridis`, `plasma`, or `inferno`
2. **Diverging data** (e.g., anomalies, differences): Use `RdBu` or `BrBG`
3. **Cyclic data** (e.g., phase, direction): Use `rainbow` or custom cyclic map
4. **Terrain**: Use `terrain` or `topography`
5. **Ocean depth**: Use `ocean`
6. **Accessibility**: Prefer `viridis` or `cividis` for colorblind-friendly visualization

### Performance Considerations

1. **Large DEMs**: Subsample before rendering for preview
2. **Vector fields**: Use appropriate `subsample` parameter in quiver plots
3. **Streamlines**: Limit `max_steps` and number of start points
4. **Volume rendering**: Consider slicing large 3D volumes

### Projection Selection

1. **World maps**: Robinson, Mollweide, or Plate Carrée
2. **Continental scale**: Lambert Conformal or Albers Equal Area
3. **Navigation**: Mercator
4. **Polar regions**: Stereographic
5. **Distance measurements**: Azimuthal Equidistant

---

## Integration with Python

XDL's Python integration allows using specialized libraries:

```xdl
; Use cartopy for advanced map features
cartopy = python_import("cartopy")
result = python_call(cartopy, "reproject_data", data, projection)

; Use matplotlib's basemap
basemap = python_import("mpl_toolkits.basemap")

; Use geopandas for vector data
gpd = python_import("geopandas")
```

---

## Implementation Status & Roadmap

### ✅ Completed Features

1. **Volume Rendering**
    - ✅ Ray marching algorithm (WebGPU)
    - ✅ Transfer functions with opacity
    - ✅ Medical imaging volumes (CT, MRI)
    - ✅ Geophysical data volumes
    - ✅ Real-time interaction (60 FPS)

2. **Interactive 2D/3D Plots**
    - ✅ WebGL 3D rendering (Three.js)
    - ✅ Zoom/pan/rotate controls
    - ✅ Surface plots with lighting
    - ✅ Isosurface extraction
    - ✅ Scientific colormaps

3. **Geophysical Visualization**
    - ✅ 3D earth models
    - ✅ Seismic data rendering
    - ✅ Contour plots
    - ✅ Interactive exploration

4. **Medical Imaging**
    - ✅ CT volume rendering
    - ✅ Anatomical structures
    - ✅ Windowing and normalization
    - ✅ 3D visualization workflows

### 🔄 Future Enhancements

1. **Enhanced GIS**
    - Natural Earth dataset integration
    - Shapefile direct loading
    - PostGIS connection

2. **Advanced Terrain**
    - Full Marching Squares contour generation
    - Viewshed analysis
    - Watershed delineation

3. **Particle Systems**
    - Particle tracing for fluid dynamics
    - Animation support
    - GPU-accelerated simulation

4. **Advanced Rendering**
    - Multi-channel volume rendering
    - Advanced lighting models
    - Ray tracing integration
   - Data tooltips

---

## References

- **Color Maps**: [Matplotlib Colormaps](https://matplotlib.org/stable/tutorials/colors/colormaps.html)
- **Projections**: [PROJ Projection Library](https://proj.org/)
- **GeoJSON**: [GeoJSON Specification](https://geojson.org/)
- **Scientific Visualization**: Wikipedia entry on [Scientific Visualization](https://en.wikipedia.org/wiki/Scientific_visualization)

---

**XDL Scientific Visualization - Making Data Beautiful and Insightful** 🌍📊🗺️
