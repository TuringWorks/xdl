# ECharts + Tauri Charting Implementation Status

**Branch:** `investigate-charting-webgl`
**Date:** 2025-10-25
**Status:** 🚧 In Progress

---

## Implementation Progress

### ✅ Completed (Phase 1)

1. **xdl-desktop-viewer crate** - Tauri-based window management
   - Window configuration API
   - Pending window queue system
   - Browser fallback mechanism
   - Dependencies: `tauri = "2.1"`, `urlencoding = "2.1"`

2. **xdl-charts crate** - ECharts integration
   - Chart types: Line, Scatter, Bar, Area, Heatmap, Scatter3D, Surface3D, Bar3D
   - ECharts option builders (2D, 3D, surface)
   - HTML template generation
   - Automatic 3D library loading (echarts-gl)

### 🚧 In Progress

3. **XDL Procedures in xdl-stdlib**
   - Need to add charting procedures
   - Integration with desktop viewer
   - Data marshalling from XDL arrays

### 📋 TODO

4. **Example scripts**
5. **Workspace configuration**
6. **End-to-end testing**

---

## Architecture Overview

```
XDL Script (.xdl or .m file)
    ↓
XDL Parser/Interpreter (xdl-interpreter)
    ↓
StandardLibrary::call_procedure() (xdl-stdlib)
    ↓
charting_procs::plot() / scatter() / surface3d()
    ↓
┌──────────────────────┬─────────────────────┐
│   xdl-charts         │  xdl-desktop-viewer │
│  (HTML generation)   │  (Tauri window)     │
└──────────────────────┴─────────────────────┘
    ↓                         ↓
ECharts HTML              Tauri Window
(embedded data)        (native desktop window)
```

---

## Next Steps

### Step 1: Add XDL Procedures to xdl-stdlib

Create `xdl-stdlib/src/charting_procs.rs`:

```rust
//! Charting procedures using ECharts + Tauri

use xdl_charts::{ChartConfig, ChartType, Series2D, Series3D};
use xdl_desktop_viewer::{launch_window, WindowConfig};

/// PLOT procedure - 2D line/scatter plot
/// Usage: PLOT, x_data, y_data, TITLE='My Plot', TYPE='scatter'
pub fn plot(args: &[Value]) -> Result<Value> {
    // 1. Extract x_data, y_data from XDL arrays
    // 2. Build Series2D
    // 3. Generate HTML via xdl_charts
    // 4. Launch window via xdl_desktop_viewer
}

/// SURFACE3D procedure - 3D surface plot
/// Usage: SURFACE3D, z_matrix, TITLE='Surface', XRANGE=[0,10], YRANGE=[0,10]
pub fn surface3d(args: &[Value]) -> Result<Value> {
    // Similar pattern for 3D surface
}
```

### Step 2: Register Procedures in xdl-stdlib/src/lib.rs

```rust
// Add to procedure registry
"PLOT" => charting_procs::plot(args),
"SCATTER" => charting_procs::scatter(args),
"SURFACE3D" => charting_procs::surface3d(args),
"BAR" => charting_procs::bar(args),
```

### Step 3: Tauri Integration in xdl-gui

Since xdl-gui already uses eframe/egui, we need to decide:

**Option A: Separate Tauri process** (Recommended for MVP)
- Launch Tauri app as separate process
- Uses `std::process::Command`
- Simpler integration

**Option B: Embed Tauri in xdl-gui**
- More complex, need to run Tauri event loop alongside eframe
- Better UX but harder to implement

For MVP, go with Option A - separate Tauri app binary.

### Step 4: Create Standalone Tauri App

Create `xdl-chart-viewer` binary crate:

```
xdl-chart-viewer/
├── Cargo.toml
├── src/
│   └── main.rs          # Tauri app main
├── tauri.conf.json      # Tauri configuration
└── build.rs             # Tauri build script
```

This will be launched by xdl/xdl-gui when charts are created.

---

## Simplified MVP Plan

Given the complexity, here's a simpler path to get working quickly:

### Phase 1: Browser-First (This Week)

1. **Skip Tauri for now** - Use existing browser pattern like viz3d-web
2. **Reuse HTTP server** from xdl-viz3d-web
3. **Focus on ECharts** integration only

```rust
// In xdl-stdlib/src/charting_procs.rs
use xdl_charts::generate_2d_chart;
use xdl_viz3d_web::launch_browser_visualization; // REUSE!

pub fn plot(args: &[Value]) -> Result<Value> {
    let series = extract_series(args)?;
    let html = generate_2d_chart(&config, &series)?;
    launch_browser_visualization(html)?; // Opens in browser
    Ok(Value::None)
}
```

**Benefits:**
- Works immediately (browser always available)
- Reuses existing, tested infrastructure
- Non-blocking execution
- Multiple charts = multiple tabs

### Phase 2: Add Tauri Later (Next Week)

Once browser version works, add Tauri as optional upgrade:

```rust
pub fn plot(args: &[Value]) -> Result<Value> {
    let html = generate_2d_chart(&config, &series)?;

    if std::env::var("XDL_DESKTOP_VIEWER").is_ok() {
        xdl_desktop_viewer::launch_window(html, None)?;
    } else {
        xdl_viz3d_web::launch_browser_visualization(html)?;
    }

    Ok(Value::None)
}
```

---

## File Changes Required

### xdl-stdlib/Cargo.toml
```toml
[dependencies]
xdl-charts = { path = "../xdl-charts" }
xdl-viz3d-web = { path = "../xdl-viz3d-web" } # Reuse browser server
# xdl-desktop-viewer = { path = "../xdl-desktop-viewer", optional = true }
```

### xdl-stdlib/src/lib.rs
```rust
pub mod charting_procs; // Add new module

// In call_procedure():
"PLOT" => charting_procs::plot(args),
"SCATTER" => charting_procs::scatter(args),
"SURFACE3D" => charting_procs::surface3d(args),
```

---

## Example XDL Usage (Target API)

```xdl
; Simple scatter plot
x = FINDGEN(100)
y = SIN(x / 10.0)
PLOT, x, y, TITLE='Sine Wave', TYPE='scatter'

; 3D surface plot
z = FLTARR(50, 50)
FOR i=0, 49 DO FOR j=0, 49 DO $
    z[i,j] = SIN(i/5.0) * COS(j/5.0)
SURFACE3D, z, TITLE='Wave Pattern', XRANGE=[0,10], YRANGE=[0,10]

; Multiple series
x1 = FINDGEN(50)
y1 = SIN(x1 / 5.0)
y2 = COS(x1 / 5.0)
PLOT, x1, y1, TITLE='Trig Functions', LABEL='sin(x)'
PLOTADD, x1, y2, LABEL='cos(x)'  ; Add second series
PLOTSHOW  ; Display chart
```

---

## Decision: Browser-First MVP

**Recommendation:** Implement browser-first version now, add Tauri later.

**Rationale:**
1. Faster implementation (reuse viz3d-web)
2. Lower risk (proven technology)
3. Works on all platforms immediately
4. Can add Tauri as enhancement later

**Action Items (This Session):**
1. ✅ Create xdl-charts crate
2. ✅ Create xdl-desktop-viewer crate (for later)
3. 🚧 Create charting_procs.rs in xdl-stdlib
4. 🚧 Wire up to browser server (xdl-viz3d-web)
5. 🚧 Test with simple PLOT command

---

## Timeline

- **Now - EOD:** Browser-based PLOT working
- **Tomorrow:** Add SCATTER, SURFACE3D
- **Day 3:** Polish + examples
- **Week 2:** Add Tauri desktop viewer (optional)

---

**Let's proceed with browser-first implementation!**
