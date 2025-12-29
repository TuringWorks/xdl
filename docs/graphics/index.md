---
layout: default
title: Graphics & Visualization
nav_order: 4
has_children: true
permalink: /graphics
---

# Graphics & Visualization

Comprehensive 2D and 3D graphics capabilities in XDL.

## Overview

XDL provides powerful visualization features:

- **2D Graphics** - Line plots, scatter plots, contours, histograms
- **3D Visualization** - Surface plots, wireframes, isosurfaces
- **Web Rendering** - Browser-based graphics with WebGL
- **Scientific Visualization** - Specialized scientific plots
- **Chart Integration** - ECharts for interactive charts

## 2D Graphics

### Basic Plotting

```xdl
; Line plot
plot, x, y

; Scatter plot
plot, x, y, psym=3

; Multiple curves
plot, x, y1
oplot, x, y2, color='red'
```

### Advanced 2D

- [Graphics Overview]({% link GRAPHICS_IMPLEMENTATION.md %}) - Implementation details
- [Graphics Quick Reference]({% link GRAPHICS_QUICK_REF.md %}) - Command reference
- [Charting Final Status]({% link CHARTING_FINAL_STATUS.md %}) - Chart features
- [Charting Implementation]({% link CHARTING_IMPLEMENTATION_STATUS.md %}) - Implementation details
- [ECharts Integration]({% link ECHARTS_INTEGRATION_COMPLETE.md %}) - Web charts
- [Bezier Curves]({% link BEZIER_IMPLEMENTATION_SUMMARY.md %}) - Curve drawing
- [Bezier Demo Features]({% link BEZIER_DEMO_FEATURES.md %}) - Demo examples
- [Graphics Demos]({% link GRAPHICS_DEMOS_STATUS.md %}) - Demo status

## 3D Visualization

### Surface Plots

```xdl
; 3D surface
surface, z

; Wireframe
wireframe, z

; Mesh plot
mesh, z, color_table='rainbow'
```

### 3D Documentation

- [3D Visualization Complete]({% link VIZ3D_COMPLETE_FINAL.md %}) - Full guide
- [3D Browser Guide]({% link VIZ3D_BROWSER_GUIDE.md %}) - Browser rendering
- [Three.js Integration]({% link VIZ3D_THREEJS_COMPLETE.md %}) - Advanced features
- [3D Performance]({% link VIZ3D_PERFORMANCE_IMPROVEMENTS.md %}) - Optimization
- [3D Usage Guide]({% link VIZ3D_USAGE.md %}) - Using 3D features
- [3D Quick Start]({% link VIZ3D_QUICK_START.md %}) - Getting started
- [3D Showcase]({% link VIZ3D_SHOWCASE_README.md %}) - Demonstrations

## Scientific Visualization

Specialized scientific plotting:

- [Scientific Visualization Guide]({% link SCIENTIFIC_VISUALIZATION_GUIDE.md %}) - Scientific plots
- [Advanced Visualization]({% link ADVANCED_VIZ_INTEGRATION.md %}) - Advanced features
- [Advanced Reference]({% link ADVANCED_VIZ_REFERENCE.md %}) - API reference

## Rendering Backends

XDL supports multiple rendering backends:

| Backend | Description | Use Case |
|:--------|:------------|:---------|
| Native OpenGL | Desktop rendering | High-performance desktop apps |
| WebGL | Browser rendering | Web applications |
| Three.js | Advanced web graphics | Interactive 3D web apps |
| ECharts | Interactive charts | Data visualization dashboards |

## Examples

- [Graphics Demos]({% link GRAPHICS_DEMOS_STATUS.md %}) - Demo examples
- [Bezier Demo]({% link BEZIER_DEMO_FEATURES.md %}) - Curve examples
- [3D Showcase]({% link VIZ3D_SHOWCASE_README.md %}) - 3D demonstrations
