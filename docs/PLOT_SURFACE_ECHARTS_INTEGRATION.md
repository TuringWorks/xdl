# PLOT and SURFACE xdl-charts Integration

**Date:** 2025-10-25 (revised after codebase grounding)

> Status note: this document described an "automatic fallback" design that was not actually implemented as written. The revision below reflects the live state of `xdl-stdlib/src/graphics_procs.rs` and `charting_procs.rs`.

---

## What Actually Changed

### `PLOT` — opt-in ECharts routing (via `SET_PLOT_BACKEND`)

In `xdl-stdlib/src/graphics_procs.rs`:

- `PLOT` now checks `get_plot_backend()` (a `Mutex<PlotBackend>` defaulting to `PlotBackend::XDLPlot`).
- If the backend is `PlotBackend::ECharts`, `PLOT` reorders its `IDL/GDL`-style `(y, x)` arguments into `(x, y)` order, builds `XdlValue::Array(...)` payloads, and forwards to `crate::charting_procs::plot(...)`. The actual Tauri launch happens inside `charting_procs::plot → launch_chart(...)`.
- If the backend is `PlotBackend::XDLPlot` (default), `PLOT` falls through to `launch_plot_window`, which calls a registered GUI callback if one exists, or saves a PNG via plotters as a last resort.
- The backend is switched with a new procedure: `SET_PLOT_BACKEND, 'ECHARTS'` / `SET_PLOT_BACKEND, 'XDLPLOT'` / `SET_PLOT_BACKEND, 'PLOTTERS'` (handled by `set_plot_backend_proc` and dispatched in `xdl-stdlib/src/lib.rs`).

There is **no auto-fallback** in the live code. `SET_PLOT_BACKEND` is the single switch. `PLOT` either goes through ECharts+Tauri or through the legacy XDLPlot path; it does not silently fall back from one to the other inside a single call.

### `SURFACE` and `SHADE_SURF` — not wired to xdl-charts

In `xdl-stdlib/src/graphics_procs.rs`:

- `SURFACE` calls `crate::graphics::surface_plot(z_data, Some(x), Some(y), SurfaceConfig::default(), "xdl_surface.png")` directly. It does **not** route through `charting_procs::surface3d`.
- `SHADE_SURF` similarly calls `surface_plot` directly with `SurfaceConfig { shading: true, ..Default::default() }`. It does **not** route through `charting_procs::shade_surf`.
- Both still use the `plotters`/`egui_plot` path and respect the `GUI_IMAGE_CALLBACK` for display.

If you want ECharts 3D surfaces, call `SURFACE3D` (in `charting_procs`) directly — that proc does go through `xdl-charts` and `xdl-chart-viewer`.

### `PLOT3D` (graphics_procs) vs. `CHART_PLOT3D` (charting_procs)

Two procedures share the name's prefix:

- `PLOT3D` in `graphics_procs.rs` — takes `(x, y, z)`, renders via `crate::graphics::plot_3d(...)` to `xdl_plot3d.png`, fires the image callback. Pure plotters/egui_plot.
- `CHART_PLOT3D` in `charting_procs.rs` — takes `(x, y, z)`, builds an ECharts `scatter3D` series, launches `xdl-chart-viewer`. Title is hard-coded to `"3D Line Plot"`.

---

## Usage (corrected)

```xdl
; Default: PLOT uses XDLPlot backend (plotters → PNG or GUI callback)
x = FINDGEN(100) / 10.0
y = SIN(x)
PLOT, x, y

; Switch to ECharts-backed PLOT (one-shot; persistent for the session)
SET_PLOT_BACKEND, 'ECHARTS'
PLOT, x, y                  ; now launches xdl-chart-viewer

; Back to legacy
SET_PLOT_BACKEND, 'XDLPLOT'
PLOT, x, y                  ; PNG / GUI callback again

; ECharts 3D plots are always explicit, regardless of backend:
SURFACE3D, z, 'Wave'        ; 3D surface via xdl-charts + Tauri
SCATTER3D, x, y, z, 'Pts'   ; 3D scatter via xdl-charts + Tauri
CHART_PLOT3D, x, y, z       ; 3D scatter line via xdl-charts + Tauri (title hard-coded)
```

---

## Why This Design (and Not the "auto-fallback" Originally Proposed)

The "try ECharts → fall back to PNG" path described in the earlier version of this doc was never implemented. The chosen approach is:

- **Explicit backend switch** for `PLOT` (`SET_PLOT_BACKEND`), since IDL semantics for bare `PLOT` mean "static 2D plot", and silently rerouting that to a native Tauri window would be a behavior change that breaks existing scripts.
- **Bare `SURFACE`/`SHADE_SURF` keep their legacy path** (plotters → PNG / GUI image callback), because `graphics::surface_plot` is shared with `DEM_RENDER`, `HILLSHADE`, `QUIVER`, etc., and changing the default there would have side effects.
- **ECharts 3D lives behind explicit names** (`SURFACE3D`, `SCATTER3D`, `CHART_PLOT3D`, `CHART_SHADE_SURF`, `CHART_CONTOUR`), so users opt in deliberately.

The trade-off is that the legacy `SURFACE` is still 2D-rendered and lacks the true 3D rotation that `SURFACE3D` provides. If we later want to wire `SET_PLOT_BACKEND` to also switch `SURFACE`'s renderer, that's a follow-up.

---

## Architecture (live)

```text
PLOT procedure (graphics_procs.rs)
        │
        ├─ get_plot_backend() == PlotBackend::ECharts ?
        │     │
        │     ▼
        │   reorder args (y, x) → (x, y)
        │   charting_procs::plot(args)
        │     ├─ build ChartConfig + Series2D
        │     ├─ xdl_charts::generate_2d_chart() → HTML
        │     └─ launch_chart():
        │           write ${tempdir}/xdl_chart_<pid>.html
        │           Command::spawn("xdl-chart-viewer",
        │               ["--html-file", path, "--title", title])
        │
        └─ else (XDLPlot default)
              │
              ▼
            launch_plot_window(x, y)
              ├─ GUI_PLOT_CALLBACK → display in GUI
              └─ else save_plot_to_file → xdl_plot.png (plotters)


SURFACE / SHADE_SURF (graphics_procs.rs)
        │
        ▼
   graphics::surface_plot(z, Some(x), Some(y), config, "xdl_surface.png")
        ├─ GUI_IMAGE_CALLBACK → display in GUI
        └─ else: PNG on disk (plotters)


SURFACE3D / SCATTER3D / CHART_* (charting_procs.rs)
        │
        ▼
   xdl_charts::* → HTML → launch_chart() → xdl-chart-viewer (Tauri)
```

---

## Implementation Reference

- **Backend switching**: `xdl-stdlib/src/graphics_procs.rs`
  - `PlotBackend` enum (lines ~10–14)
  - `PLOT_BACKEND: Mutex<PlotBackend>` (default `XDLPlot`)
  - `get_plot_backend()`, `set_plot_backend()`
  - `plot()` (lines ~60–125) — backend dispatch
  - `set_plot_backend_proc()` — `'ECHARTS'`/`'XDLPLOT'`/`'PLOTTERS'` switch
- **Registration**: `xdl-stdlib/src/lib.rs`
  - `"PLOT" => graphics_procs::plot(args)`
  - `"SET_PLOT_BACKEND" => graphics_procs::set_plot_backend_proc(args)`
  - `"SURFACE3D" => charting_procs::surface3d(args)`
  - `"SCATTER3D" => charting_procs::scatter3d(args)`
  - `"CHART_PLOT3D" => charting_procs::plot3d(args)`
  - `"CHART_SHADE_SURF" => charting_procs::shade_surf(args)`
  - `"CHART_CONTOUR" => charting_procs::contour(args)`
  - `"CHART_PLOT" => charting_procs::plot(args)`
  - `"CHART_SCATTER" => charting_procs::scatter(args)`
  - `"CHART_BAR" => charting_procs::bar(args)`
- **Legacy `SURFACE`/`SHADE_SURF`**: still in `graphics_procs.rs`; they call `graphics::surface_plot` directly and ignore `xdl-charts`.

---

## Testing

Run the test script:

```bash
./target/release/xdl examples/charting/test_plot_surface.xdl
```

Expected behavior (live):

- `PLOT, x, y` invokes the **default** `XDLPlot` backend — writes `xdl_plot.png` (or fires the GUI callback) and prints `"Plot data saved to ... (GUI not available)"`.
- `SURFACE, z` writes `xdl_surface.png` via plotters (no Tauri window).
- `CHART_PLOT, …`, `SURFACE3D, …`, `SCATTER3D, …`, etc., spawn `xdl-chart-viewer` Tauri windows.
- `SET_PLOT_BACKEND, 'ECHARTS'` followed by `PLOT, …` does spawn `xdl-chart-viewer`.

---

## Future Enhancements

1. Extend `SET_PLOT_BACKEND` to also switch the renderer for `SURFACE` and `SHADE_SURF`.
2. Surface keyword arguments (`TITLE=`, `XTITLE=`, `YTITLE=`) in `charting_procs` so users can pass `PLOT, x, y, TITLE='Sine'` cleanly.
3. Multi-series plotting in a single chart window (today, each call is a separate window).
4. Export from the Tauri window (PNG, SVG, PDF).
5. Real-time data updates.

---

## Compatibility

- ✅ Maintains full backward compatibility for the default `PLOT`/`SURFACE`/`SHADE_SURF` behavior.
- ✅ Opt-in path to interactive ECharts via `SET_PLOT_BACKEND, 'ECHARTS'` or via the explicit `CHART_*`/`SURFACE3D`/`SCATTER3D` procedures.
- ✅ No breaking changes to existing XDL/IDL scripts.
- ✅ Preserves the GUI callback system for custom integrations.