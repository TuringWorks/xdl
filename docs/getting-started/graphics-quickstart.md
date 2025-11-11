---
layout: default
title: Graphics Quick Start
parent: Getting Started
nav_order: 4
---

# Graphics Quick Start

Get started with 2D graphics and plotting in XDL.

For the complete guide, see [QUICKSTART_GRAPHICS.md](../QUICKSTART_GRAPHICS).

## Basic Plotting

```xdl
; Create data
x = findgen(100)
y = sin(x * !pi / 50)

; Simple plot
plot, x, y

; Plot with title and labels
plot, x, y, title='Sine Wave', xtitle='X', ytitle='Y'
```

## Multiple Plots

```xdl
; Multiple curves on one plot
x = findgen(100)
y1 = sin(x * !pi / 50)
y2 = cos(x * !pi / 50)

plot, x, y1, color='blue', title='Trig Functions'
oplot, x, y2, color='red'
```

## Scatter Plots

```xdl
; Scatter plot
x = randomn(100)
y = randomn(100)
plot, x, y, psym=3
```

## Histograms

```xdl
; Create histogram
data = randomn(1000)
hist = histogram(data, nbins=50)
plot, hist
```

## Contour Plots

```xdl
; 2D contour plot
x = findgen(50)
y = findgen(50)
z = sin(x/5) # cos(y/5)
contour, z
```

## Saving Plots

```xdl
; Save plot to file
plot, x, y
save_plot, 'myplot.png'
```

## Next Steps

- [Graphics Overview](../GRAPHICS_IMPLEMENTATION) - Complete graphics guide
- [Graphics Quick Reference](../GRAPHICS_QUICK_REF) - Command reference
- [3D Visualization](viz3d-quickstart) - 3D graphics
- [Charting](../CHARTING_FINAL_STATUS) - Advanced charts
