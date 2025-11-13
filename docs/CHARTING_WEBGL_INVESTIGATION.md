# Charting and WebGL Rendering Investigation

**Branch:** `investigate-charting-webgl`
**Date:** 2025-10-24
**Status:** Investigation Phase

---

## Executive Summary

This investigation evaluates modern JavaScript charting libraries (D3.js, Three.js, Apache ECharts) and WebGL rendering techniques to enhance XDL's visualization capabilities. The goal is to identify opportunities for:

1. Interactive 2D/3D scientific charts
2. High-performance WebGL-based rendering
3. Browser-based visualization enhancements
4. Integration with existing XDL visualization stack

---

## Background

### Current XDL Visualization Stack

XDL currently supports:

- **3D Volume Rendering**: xdl-viz3d-web (WebGPU-based, browser-first, 60 FPS)
- **Scientific Visualization**: Colormap rendering, DEM visualization, hillshade, quiver plots
- **2D Plotting**: plotters crate (Rust-based)
- **GUI Integration**: eframe/egui (native), browser-based HTML

### Gap Analysis

| Capability | Current Status | Desired State |
|------------|----------------|---------------|
| Interactive 2D charts | Limited (static plotters) | Rich, interactive, zoomable |
| 3D surface plots | Basic (plotters) | WebGL-accelerated, rotatable |
| Time series visualization | Basic | Advanced (multi-axis, tooltips) |
| Geographic visualization | GIS features (optional) | Interactive maps, projections |
| Chart animations | None | Smooth transitions, updates |
| Declarative API | Procedural (XDL commands) | Declarative (JSON/config-based) |

---

## Libraries Under Investigation

### 1. D3.js (Data-Driven Documents)

**Version:** 7.x
**License:** BSD 3-Clause
**Size:** ~280 KB (minified)

#### D3.js Strengths

- Industry standard for data visualization
- Extremely flexible and composable
- SVG-based (scalable, crisp rendering)
- Rich ecosystem (d3-geo, d3-scale, d3-axis, etc.)
- Excellent for custom visualizations

#### D3.js Weaknesses

- Steep learning curve
- Performance issues with large datasets (>10K points)
- Requires significant JavaScript knowledge
- No built-in 3D support

#### D3.js Use Cases in XDL

- **Scientific plots**: Scatter, line, area charts with custom axes
- **Geospatial**: Map projections, choropleth maps
- **Network graphs**: Force-directed layouts for molecular structures
- **Custom visualizations**: Parallel coordinates, Sankey diagrams

#### D3.js Integration Strategy

```rust
// xdl-stdlib: Generate D3 visualization
fn d3_chart(data: &Array2<f64>, chart_type: &str, output: &str) {
    let json_data = serde_json::to_string(data)?;
    let html = format!(r#"
        <script src="https://d3js.org/d3.v7.min.js"></script>
        <script>
            const data = {json_data};
            // D3 rendering code...
        </script>
    "#);
    serve_html_and_open(html, output)?;
}
```

---

### 2. Three.js

**Version:** r160+
**License:** MIT
**Size:** ~600 KB (minified)

#### Three.js Strengths

- WebGL-based (GPU-accelerated)
- Rich 3D primitives and materials
- Excellent documentation and community
- Built-in camera controls, lights, shadows
- Supports GLTF, OBJ, STL formats

#### Three.js Weaknesses

- Large bundle size
- Not specialized for scientific visualization
- Requires 3D graphics knowledge
- Overhead for simple 2D charts

#### Three.js Use Cases in XDL

- **3D surface plots**: Mesh geometries with height maps
- **Molecular visualization**: Ball-and-stick models
- **Volume rendering**: Alternative to current WebGPU implementation
- **Particle systems**: Large-scale point clouds (millions of particles)

#### Three.js Integration Strategy

```rust
// xdl-stdlib: Three.js surface plot
fn threejs_surface(z_data: &Array2<f64>, output: &str) {
    // Convert 2D array to vertices
    let vertices = array_to_vertices(z_data);
    let html = format!(r#"
        <script type="importmap">{{
            "imports": {{
                "three": "https://cdn.jsdelivr.net/npm/three@0.160.0/build/three.module.js"
            }}
        }}</script>
        <script type="module">
            import * as THREE from 'three';
            // Three.js scene setup...
        </script>
    "#);
    serve_html_and_open(html, output)?;
}
```

---

### 3. Apache ECharts

**Version:** 5.x
**License:** Apache 2.0
**Size:** ~350 KB (minified)

#### ECharts Strengths

- **Best for XDL**: Balance of power and ease-of-use
- Declarative configuration (JSON-based)
- WebGL renderer built-in (echarts-gl extension)
- Excellent performance (100K+ points)
- Rich chart types (50+ built-in)
- Scientific features: 3D scatter, surface, bar3D

#### ECharts Weaknesses

- Chinese-first documentation (though English is good)
- Less flexible than D3 for custom visualizations
- Some advanced features require extensions

#### ECharts Use Cases in XDL

- **Time series**: Multi-axis, zoomable charts
- **Scientific plots**: Scatter, heatmaps, contours
- **3D plots**: Surface plots, 3D scatter, 3D bar charts
- **Dashboards**: Multi-chart layouts with linked interactions

#### ECharts Integration Strategy (Recommended)

```rust
// xdl-stdlib: ECharts visualization
fn echarts_render(config: &EChartsConfig, output: &str) {
    let config_json = serde_json::to_string(config)?;
    let html = format!(r#"
        <script src="https://cdn.jsdelivr.net/npm/echarts@5/dist/echarts.min.js"></script>
        <script src="https://cdn.jsdelivr.net/npm/echarts-gl@2/dist/echarts-gl.min.js"></script>
        <script>
            const chart = echarts.init(document.getElementById('main'));
            chart.setOption({config_json});
        </script>
    "#);
    serve_html_and_open(html, output)?;
}
```

---

## WebGL Rendering Techniques

### Raw WebGL vs. Abstractions

| Approach | Pros | Cons | Use Case |
|----------|------|------|----------|
| **Raw WebGL** | Maximum performance | Complex, verbose | Volume rendering (current) |
| **Three.js** | Easy 3D, rich features | Bundle size | 3D scientific plots |
| **Regl** | Functional, minimal | Less features | Custom shaders |
| **WebGPU** | Next-gen, Compute shaders | Browser support | Heavy computation |

### Current WebGPU Implementation

XDL already uses WebGPU for volume rendering (xdl-viz3d-web). This is optimal for:

- Volume ray marching
- Compute-heavy shaders
- Low-level GPU control

### Proposed WebGL Use Cases

1. **2D Chart Acceleration** (ECharts GL)
   - Large scatter plots (>100K points)
   - Real-time streaming data
   - Smooth animations

2. **3D Surface Plots** (Three.js or ECharts GL)
   - Mesh rendering with lighting
   - Interactive rotation/zoom
   - Height-based colormaps

3. **Particle Systems** (Three.js Points)
   - Molecular dynamics visualization
   - Astronomy (star fields)
   - Point cloud rendering

---

## Scope of Investigation

### Phase 1: Prototyping (2-3 weeks)

**Deliverables:**

1. ✅ **Branch created**: `investigate-charting-webgl`
2. 📋 **Three prototype implementations:**
   - D3.js: Interactive scatter plot with brush selection
   - Three.js: 3D surface plot from 2D array
   - ECharts: Multi-chart dashboard with WebGL scatter

3. 📋 **Performance benchmarks:**
   - Dataset sizes: 1K, 10K, 100K, 1M points
   - Metrics: Load time, FPS, memory usage
   - Comparison: Static (plotters) vs. WebGL

4. 📋 **Integration POCs:**
   - Rust → JavaScript data serialization
   - HTML template generation
   - Browser server reuse (xdl-viz3d-web pattern)

### Phase 2: Evaluation (1 week)

**Deliverables:**

1. 📋 **Technical comparison matrix:**
   - Performance scores
   - Feature coverage
   - Integration complexity
   - Bundle size impact

2. 📋 **Recommendation document:**
   - Primary library choice
   - Use case mapping
   - Migration path from plotters

3. 📋 **API design proposal:**
   - XDL procedure signatures
   - Configuration format
   - Example scripts

### Phase 3: Implementation (3-4 weeks, if approved)

**Deliverables:**

1. 📋 **New crate: xdl-charts**
   - Chart configuration structs
   - Template generator
   - Data serialization
   - Server integration (reuse xdl-viz3d-web server)

2. 📋 **XDL procedures:**

   ```xdl
   CHART_INIT, TYPE='scatter', TITLE='My Chart'
   CHART_DATA, x_values, y_values, LABEL='Series 1'
   CHART_CONFIG, /INTERACTIVE, RENDERER='webgl'
   CHART_RENDER, 'output.html'

   ; 3D surface plot
   SURFACE3D, z_matrix, COLORMAP='viridis', /INTERACTIVE

   ; Time series dashboard
   DASHBOARD_INIT, LAYOUT='grid', ROWS=2, COLS=2
   DASHBOARD_CHART, 0, data1, TYPE='line'
   DASHBOARD_CHART, 1, data2, TYPE='scatter'
   DASHBOARD_RENDER, 'dashboard.html'
   ```

3. 📋 **Example scripts:**
   - `examples/charting/scatter_interactive.xdl`
   - `examples/charting/surface3d_webgl.xdl`
   - `examples/charting/timeseries_dashboard.xdl`
   - `examples/charting/performance_comparison.xdl`

4. 📋 **Documentation:**
   - `docs/CHARTING_GUIDE.md`
   - `docs/WEBGL_RENDERING.md`
   - API reference

---

## Success Criteria

### Technical Metrics

- [ ] Render 100K points at 60 FPS (WebGL)
- [ ] Load time < 1s for typical charts
- [ ] Memory usage < 100 MB per chart
- [ ] Bundle size < 1 MB total (gzip)
- [ ] Browser compatibility: Chrome, Edge, Safari, Firefox

### User Experience

- [ ] XDL API is intuitive (similar to IDL/MATLAB)
- [ ] Interactive features work out-of-box (zoom, pan, rotate)
- [ ] Charts are publication-quality
- [ ] Export to PNG/SVG supported
- [ ] Tooltips and legends are automatic

### Integration

- [ ] Reuses existing browser server (xdl-viz3d-web)
- [ ] Works in both xdl CLI and xdl-gui
- [ ] Non-blocking execution
- [ ] Backward compatible with plotters (fallback)

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| **Browser WebGL support** | Low | Medium | Fallback to Canvas 2D |
| **Bundle size bloat** | Medium | Medium | Tree-shaking, CDN links |
| **Performance on large data** | Medium | High | WebWorkers, streaming |
| **API complexity** | Medium | Medium | Sensible defaults, examples |
| **Maintenance burden** | Medium | Medium | Choose well-maintained libs |

---

## Resource Requirements

### Development Time

- **Phase 1 (Prototyping):** 40-60 hours
- **Phase 2 (Evaluation):** 10-15 hours
- **Phase 3 (Implementation):** 80-120 hours
- **Total:** ~130-195 hours (~4-6 weeks)

### Dependencies

```toml
# New dependencies (estimated)
[dependencies]
# Reuse existing
tiny_http = "0.12"        # Already in xdl-viz3d-web
serde_json = "1.0"        # Already in workspace
base64 = "0.22"           # Already in xdl-viz3d-web
webbrowser = "1.0"        # Already in xdl-viz3d-web

# New (minimal)
# None! Charting libraries loaded via CDN in HTML
```

**Note:** All JavaScript libraries will be loaded via CDN, minimizing Rust dependencies and binary size.

---

## Recommendation Matrix

| Use Case | Recommended Library | Rationale |
|----------|---------------------|-----------|
| **2D scientific plots** | **Apache ECharts** | Best balance of features and ease |
| **3D surface plots** | **ECharts GL** (primary), Three.js (fallback) | Built-in support, good performance |
| **Custom visualizations** | **D3.js** | Maximum flexibility |
| **Large point clouds** | **Three.js** | GPU particle systems |
| **Dashboards** | **Apache ECharts** | Multi-chart coordination |
| **Volume rendering** | **Keep WebGPU** | Already optimal |

### Hybrid Approach (Recommended)

```text
┌─────────────────────────────────────────-──--──┐
│           XDL Visualization Stack              │
├─────────────────────────────────────────---────┤
│  Volume Rendering    →  WebGPU (xdl-viz3d-web) |
│  2D/3D Charts        →  ECharts + ECharts GL.  |
│  Custom Viz          →  D3.js (as needed)      |
│  Static Plots        →  plotters (fallback)    |
│  3D Models           →  Three.js (future)      |
└────────────────────────────────────────────---─┘
```

---

## Desktop Window Options

### Browser vs. Electron vs. Tauri

| Approach | Bundle Size | RAM Usage | Build Complexity | UX Quality |
|----------|-------------|-----------|------------------|------------|
| **Browser** (current) | 0 MB | ~50 MB/tab | Low | Good |
| **Electron** | ~200 MB | ~150 MB/window | High | Excellent |
| **Tauri** | ~5-10 MB | ~50 MB/window | Medium | Excellent |

### Recommended: Tauri Integration

**Why Tauri over Electron:**

- Written in Rust (fits XDL's stack)
- 20-40x smaller bundle size
- Uses system WebView (no bundled Chromium)
- Same UX as Electron
- Active development, growing ecosystem

**Implementation:**

```toml
# New crate: xdl-desktop-viewer
[dependencies]
tauri = "2.0"                 # Desktop window framework
tauri-plugin-window = "2.0"  # Window management
```

**Usage Pattern:**

```rust
// xdl-stdlib: Launch in desktop window or browser
fn render_chart(html: &str, mode: RenderMode) {
    match mode {
        RenderMode::Desktop => xdl_desktop_viewer::launch(html),
        RenderMode::Browser => xdl_viz3d_web::launch_browser(html),
    }
}
```

**User Control:**

```xdl
; Use desktop window (if available)
CHART_RENDER, 'output.html', /DESKTOP

; Use browser (fallback)
CHART_RENDER, 'output.html', /BROWSER

; Auto-detect (desktop preferred)
CHART_RENDER, 'output.html'  ; Uses desktop if xdl-desktop-viewer installed
```

### Phase 1 Addition: Desktop Window Prototype

**Additional Deliverable:**

- 📋 **Tauri POC**: Simple desktop window displaying ECharts
  - Basic Tauri app (~100 lines Rust)
  - Opens chart in native window
  - Compare UX vs. browser
  - Measure resource usage

**Decision Criteria:**

- If users prefer desktop UX → Add Tauri support in Phase 3
- If browser is sufficient → Keep browser-only (simpler)
- Offer both options (feature flag: `--features desktop-viewer`)

---

## Next Steps

### Immediate Actions (This Investigation)

1. ✅ Create branch: `investigate-charting-webgl`
2. 📋 **Set up prototype directory:**

   ```bash
   mkdir -p prototypes/charting
   mkdir -p prototypes/charting/d3js
   mkdir -p prototypes/charting/threejs
   mkdir -p prototypes/charting/echarts
   ```

3. 📋 **Create test data generators:**
   - Small dataset (1K points)
   - Medium dataset (10K points)
   - Large dataset (100K points)
   - 2D surface (100x100 grid)

4. 📋 **Build three HTML prototypes:**
   - Standalone HTML files (no Rust yet)
   - Focus on core features
   - Measure performance

5. 📋 **Document findings:**
   - Performance comparison table
   - Feature comparison matrix
   - Integration complexity assessment

### Decision Point

After Phase 1 prototyping (~2-3 weeks), present findings and get approval to proceed with Phase 3 implementation.

---

## Related Work

### XDL Visualization History

- ✅ **VIZ3D**: WebGPU volume rendering (production)
- ✅ **Advanced Viz**: Colormap, DEM, hillshade, quiver (production)
- ✅ **plotters**: 2D static charts (production)
- 🚧 **This investigation**: Interactive charts (exploration)

### Future Vision

The ultimate goal is a unified, browser-first visualization system:

```xdl
; Simple API
PLOT, x, y, /INTERACTIVE        ; Opens in browser
SURFACE, z_matrix               ; 3D surface in browser
VOLUME, volume_data             ; Volume rendering (current)
DASHBOARD, charts, LAYOUT='2x2' ; Multi-chart layout

; All visualizations:
; - Open in browser tabs
; - GPU-accelerated where beneficial
; - Interactive (zoom, pan, rotate)
; - Export to PNG/SVG/HTML
; - Non-blocking execution
```

---

## Appendix A: Example Prototypes

### D3.js Scatter Plot

```html
<!DOCTYPE html>
<html>
<head>
    <script src="https://d3js.org/d3.v7.min.js"></script>
</head>
<body>
    <svg width="800" height="600" id="chart"></svg>
    <script>
        const data = [...]; // XDL data here
        const svg = d3.select("#chart");
        // D3 scatter plot code...
    </script>
</body>
</html>
```

### Three.js Surface

```html
<!DOCTYPE html>
<html>
<head>
    <script type="importmap">
        { "imports": { "three": "https://cdn.jsdelivr.net/npm/three@0.160.0/build/three.module.js" }}
    </script>
</head>
<body>
    <canvas id="canvas"></canvas>
    <script type="module">
        import * as THREE from 'three';
        // Three.js surface plot code...
    </script>
</body>
</html>
```

### ECharts 3D Scatter

```html
<!DOCTYPE html>
<html>
<head>
    <script src="https://cdn.jsdelivr.net/npm/echarts@5/dist/echarts.min.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/echarts-gl@2/dist/echarts-gl.min.js"></script>
</head>
<body>
    <div id="main" style="width: 800px; height: 600px;"></div>
    <script>
        const chart = echarts.init(document.getElementById('main'));
        chart.setOption({
            grid3D: {},
            xAxis3D: {},
            yAxis3D: {},
            zAxis3D: {},
            series: [{
                type: 'scatter3D',
                data: [...], // XDL data
            }]
        });
    </script>
</body>
</html>
```

---

## Appendix B: Performance Benchmarking Plan

### Test Matrix

| Library | Chart Type | Points | FPS Target | Memory Target |
|---------|-----------|--------|------------|---------------|
| D3.js | Scatter | 1K | 60 | < 50 MB |
| D3.js | Scatter | 10K | 30 | < 100 MB |
| D3.js | Scatter | 100K | ⚠️ Expected failure | - |
| Three.js | Scatter (Points) | 100K | 60 | < 100 MB |
| Three.js | Scatter (Points) | 1M | 30 | < 500 MB |
| ECharts (Canvas) | Scatter | 10K | 60 | < 100 MB |
| ECharts (WebGL) | Scatter | 100K | 60 | < 200 MB |
| ECharts GL | Surface | 100x100 | 60 | < 150 MB |
| Three.js | Surface | 100x100 | 60 | < 150 MB |

### Metrics to Collect

1. **Initial load time** (ms)
2. **First render time** (ms)
3. **FPS during interaction** (avg/min)
4. **Memory usage** (MB, via Chrome DevTools)
5. **Bundle size** (KB, minified + gzip)

---

**Status:** 📋 Investigation phase initiated
**Next Review:** After Phase 1 prototyping completion
