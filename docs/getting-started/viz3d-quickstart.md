---
layout: default
title: 3D Visualization Quick Start
parent: Getting Started
nav_order: 5
---

# 3D Visualization Quick Start

Get started with 3D visualization in XDL.

For the complete guide, see [QUICKSTART_VIZ3D](../QUICKSTART_VIZ3D).

## Surface Plots

```xdl
; Create 3D surface
x = findgen(50)
y = findgen(50)
z = sin(sqrt((x-25)^2 + (y-25)^2) / 5)

; Display surface
surface, z
```

## 3D Scatter Plots

```xdl
; 3D scatter plot
x = randomn(100)
y = randomn(100)
z = randomn(100)

plot3d, x, y, z, psym=3
```

## Wireframe Plots

```xdl
; Wireframe surface
x = findgen(30)
y = findgen(30)
z = x # y

wireframe, z
```

## Mesh Plots

```xdl
; Mesh plot with color
mesh, z, color_table='rainbow'
```

## Isosurfaces

```xdl
; 3D isosurface
data = make_volume(50, 50, 50)
isosurface, data, threshold=0.5
```

## Interactive Viewing

XDL provides interactive 3D viewing:

- **Browser Mode** - WebGL rendering in browser
- **Desktop Mode** - Native OpenGL rendering
- **Three.js Mode** - Advanced web rendering

```bash
# Launch browser viewer
xdl-viz3d-browser script.xdl

# Launch desktop viewer
xdl-desktop-viewer script.xdl
```

## Animation

```xdl
; Animated surface
for t = 0, 100 do begin
  z = sin(sqrt((x-25)^2 + (y-25)^2) / 5 + t/10.0)
  surface, z
  wait, 0.05
endfor
```

## Next Steps

- [3D Visualization Complete](../VIZ3D_COMPLETE_FINAL) - Full guide
- [3D Browser Guide](../VIZ3D_BROWSER_GUIDE) - Browser rendering
- [Three.js Integration](../VIZ3D_THREEJS_COMPLETE) - Advanced features
- [Scientific Visualization](../SCIENTIFIC_VISUALIZATION_GUIDE) - Scientific plots
