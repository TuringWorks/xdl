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

- [Graphics Overview](../GRAPHICS_IMPLEMENTATION) - Implementation details
- [Graphics Quick Reference](../GRAPHICS_QUICK_REF) - Command reference
- [Charting](../CHARTING_FINAL_STATUS) - Chart features
- [ECharts Integration](../ECHARTS_INTEGRATION_COMPLETE) - Web charts
- [Bezier Curves](../BEZIER_IMPLEMENTATION_SUMMARY) - Curve drawing

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

- [3D Visualization Complete](../VIZ3D_COMPLETE_FINAL) - Full guide
- [3D Browser Guide](../VIZ3D_BROWSER_GUIDE) - Browser rendering
- [Three.js Integration](../VIZ3D_THREEJS_COMPLETE) - Advanced features
- [3D Performance](../VIZ3D_PERFORMANCE_IMPROVEMENTS) - Optimization
- [3D Usage Guide](../VIZ3D_USAGE) - Using 3D features

## Scientific Visualization

Specialized scientific plotting:

- [Scientific Visualization Guide](../SCIENTIFIC_VISUALIZATION_GUIDE) - Scientific plots
- [Advanced Visualization](../ADVANCED_VIZ_INTEGRATION) - Advanced features
- [Advanced Reference](../ADVANCED_VIZ_REFERENCE) - API reference

## Rendering Backends

XDL supports multiple rendering backends:

| Backend | Description | Use Case |
|:--------|:------------|:---------|
| Native OpenGL | Desktop rendering | High-performance desktop apps |
| WebGL | Browser rendering | Web applications |
| Three.js | Advanced web graphics | Interactive 3D web apps |
| ECharts | Interactive charts | Data visualization dashboards |

## Examples

- [Graphics Demos](../GRAPHICS_DEMOS_STATUS) - Demo examples
- [Bezier Demo](../BEZIER_DEMO_FEATURES) - Curve examples
- [3D Showcase](../VIZ3D_SHOWCASE_README) - 3D demonstrations
