# ✅ ECharts + Tauri Integration - COMPLETE

**Branch:** `investigate-charting-webgl`
**Date:** 2025-10-25 (revised after codebase grounding)
**Status:** ✅ MVP Implemented; `cargo check` clean workspace-wide.

> Status note: this doc was written as a "production ready" announcement before the implementation actually shipped. It has been revised to record the actual state of the code, including the proc count, registration list, and which parts of the original design were simplified.

---

## Summary

Apache ECharts charting library is integrated with a Tauri desktop viewer for XDL. Users can create interactive charts from XDL scripts that open in native Tauri windows. The full pipeline — `xdl-charts` (HTML/JSON generator) → `xdl-chart-viewer` (Tauri binary) — is wired up and builds cleanly.

---

## What Was Built (live in tree)

### 1. `xdl-charts` crate ✅

**Location:** `xdl-charts/` (`Cargo.toml`, `src/{lib,echarts,templates}.rs`)

`ChartType` enum: `Line`, `Scatter`, `Bar`, `Area`, `Heatmap`, `Scatter3D`, `Surface3D`, `Bar3D`. Option builders: `build_2d_option`, `build_3d_option`, `build_surface_option`, `build_heatmap_option`. Template generator: `create_echarts_html(config, option)`. Public entry points: `generate_2d_chart`, `generate_3d_chart`, `generate_surface_plot`, `generate_heatmap`.

**CDN-loaded libraries (no Rust-side JS deps):** `echarts@5` always; `echarts-gl@2` is added for `Scatter3D`/`Surface3D`/`Bar3D` automatically.

### 2. `xdl-chart-viewer` standalone Tauri binary ✅

**Location:** `xdl-chart-viewer/` (`Cargo.toml`, `src/main.rs`, `tauri.conf.json`, `build.rs`, `icons/`)

CLI: `--html-file`/`--html-content`/`--title`/`--width`/`--height`/`--help-only`. Loads HTML via a `data:text/html;charset=utf-8,…` URL with the `webview-data-url` Tauri feature. Default fallback is a built-in sine/cosine ECharts demo (`create_demo_chart_html`). IPC command `create_chart_window` exposed for multi-window use.

### 3. `xdl-desktop-viewer` library ⚠️ Near-stub

**Location:** `xdl-desktop-viewer/src/lib.rs`

Window queue + `create_window_in_app(app_handle, …)` helper. **Not in the runtime path.** Decision pending: delete or wire into `xdl-gui`.

### 4. XDL procedures ✅ (8 procs, not 5)

**Location:** `xdl-stdlib/src/charting_procs.rs`

| Proc fn | XDL name | Min args | Notes |
|---|---|---|---|
| `plot` | `CHART_PLOT` | `x, y` | 2D line; title optional |
| `scatter` | `CHART_SCATTER` | `x, y` | 2D scatter; WebGL if `len > 10_000`; title optional |
| `bar` | `CHART_BAR` | `values` | 2D bar; x = `0..n`; title optional |
| `surface3d` | `SURFACE3D` | `z_matrix` | ECharts GL surface; title optional |
| `scatter3d` | `SCATTER3D` | `x, y, z` | ECharts GL scatter; title optional |
| `contour` | `CHART_CONTOUR` | `z_matrix` | Heatmap; **title hard-coded** `"Contour Plot"` |
| `shade_surf` | `CHART_SHADE_SURF` | `z_matrix` | ECharts GL surface; **title hard-coded** `"Shaded Surface"` |
| `plot3d` | `CHART_PLOT3D` | `x, y, z` | ECharts GL scatter (line-style); **title hard-coded** `"3D Line Plot"` |

The original draft listed only `CHART_PLOT`/`CHART_SCATTER`/`CHART_BAR`/`SURFACE3D`/`SCATTER3D`. The shipped set is the full eight — `CHART_CONTOUR`/`CHART_SHADE_SURF`/`CHART_PLOT3D` were added but their titles are hard-coded (inconsistent with the rest).

### 5. `PLOT` opt-in via `SET_PLOT_BACKEND` ✅

**Location:** `xdl-stdlib/src/graphics_procs.rs`

- `PlotBackend` enum (`XDLPlot` default, `ECharts` opt-in)
- `SET_PLOT_BACKEND, 'ECHARTS'` / `'XDLPLOT'` / `'PLOTTERS'` switches it
- When set to `ECharts`, `PLOT` reorders IDL `(y, x)` args to `(x, y)` and forwards to `charting_procs::plot`

`SURFACE`/`SHADE_SURF` were **not** routed to ECharts (still plotters → PNG / GUI image callback). See `PLOT_SURFACE_ECHARTS_INTEGRATION.md`.

### 6. Demo scripts ✅

**Location:** `examples/charting/`

`.xdl`: `echarts_demo.xdl`, `simple_test.xdl`, `minimal_for_test.xdl`, `simple_for_test.xdl`, `test_contour.xdl`, `test_echarts_contour.xdl`, `test_gui_output.xdl`, `test_nested_for.xdl`, `test_plot_surface.xdl`.

`.m` (MATLAB-style): `matlab_comprehensive.m`, `matlab_plot_array.m`, `matlab_plot_multiple.m`, `matlab_plot_simple.m`, `test_gui_output.m`, `test_matlab_basic.m`, `test_range_with_arithmetic.m`.

Docs: `README.md`, `MATLAB_PLOTTING_TESTS.md`.

The "5 procedures" claim in the original draft of this doc was wrong — there are 8. The `scatter_demo.xdl` placeholder mentioned in `CHARTING_FINAL_STATUS.md` (older draft) does not exist; the actual file is `simple_test.xdl`.

---

## Installation & Usage

### Build

```bash
cargo build --release

# Binaries in target/release/:
# - xdl
# - xdl-chart-viewer (must be on the same path/folder as xdl for charting_procs to find it)
```

### Running Examples

```bash
./target/release/xdl examples/charting/simple_test.xdl
./target/release/xdl examples/charting/echarts_demo.xdl
```

**Expected behavior:**

- Script executes.
- `Command::spawn("xdl-chart-viewer", …)` is launched; HTML is written to `${tempdir}/xdl_chart_<pid>.html` and read via `--html-file`.
- Native Tauri window opens with an interactive chart.
- Script continues (non-blocking).
- Close window when done.

---

## XDL API Reference

### CHART_PLOT

```xdl
x = FINDGEN(100) / 10.0
y = SIN(x)
CHART_PLOT, x, y, 'Sine Wave'
```

### CHART_SCATTER

```xdl
x = RANDOMU(seed, 100) * 10
y = RANDOMU(seed, 100) * 10
CHART_SCATTER, x, y, 'Random Points'
```

### CHART_BAR

```xdl
values = [23.5, 45.2, 67.8, 34.1, 89.3]
CHART_BAR, values, 'Bar Chart'
```

### SURFACE3D

```xdl
z = FLTARR(50, 50)
FOR i=0, 49 DO FOR j=0, 49 DO $
    z[i,j] = SIN(SQRT(((i-25)/5)^2 + ((j-25)/5)^2))
SURFACE3D, z, '3D Surface'
```

### SCATTER3D

```xdl
x = RANDOMU(seed, 100) * 10
y = RANDOMU(seed, 100) * 10
z = RANDOMU(seed, 100) * 10
SCATTER3D, x, y, z, '3D Points'
```

### CHART_CONTOUR (title hard-coded)

```xdl
z = FLTARR(50, 50)
FOR i=0, 49 DO FOR j=0, 49 DO z[i, j] = SIN(i/5.0) * COS(j/5.0)
CHART_CONTOUR, z       ; title is hard-coded "Contour Plot"
```

### CHART_SHADE_SURF (title hard-coded)

```xdl
CHART_SHADE_SURF, z    ; title is hard-coded "Shaded Surface"
```

### CHART_PLOT3D (title hard-coded)

```xdl
CHART_PLOT3D, x, y, z  ; title is hard-coded "3D Line Plot"
```

---

## Features (live)

### Interactive charts

- ✅ Zoom (click and drag)
- ✅ Pan (drag)
- ✅ Rotate (3D charts — `SURFACE3D`, `SCATTER3D`, `CHART_PLOT3D`)
- ✅ Tooltips (hover)
- ✅ Toolbox (dataZoom, restore, saveAsImage)
- ✅ Responsive resize (window resize listener in the generated HTML)

### Performance

- ✅ WebGL opt-in for `CHART_SCATTER` when `len > 10_000`
- ✅ ~100ms chart generation, ~500ms Tauri window launch (the figures quoted in the original draft are unverified)
- ✅ Multiple windows: each call spawns a separate process

### User experience

- ✅ Native windows (not browser tabs)
- ✅ Professional ECharts styling
- ✅ Non-blocking execution (`Command::spawn`, no `wait`)
- ✅ Cross-platform via Tauri's WebView2 / WKWebView / WebKitGTK

---

## Architecture

```text
┌─────────────────────────────────────┐
│     XDL Script (.xdl or .m)         │
└──────────────┬──────────────────────┘
               │
               ↓
┌─────────────────────────────────────┐
│      xdl interpreter/runtime        │
└──────────────┬──────────────────────┘
               │
               ↓
┌─────────────────────────────────────┐
│     xdl-stdlib::charting_procs      │
│  (8 procs incl. CHART_PLOT,         │
│   CHART_SCATTER, SURFACE3D, …)      │
└──────────────┬──────────────────────┘
               │
               ↓
┌─────────────────────────────────────┐
│          xdl-charts                 │
│    (ECharts HTML/JSON generator)    │
└──────────────┬──────────────────────┘
               │
               ↓
┌─────────────────────────────────────┐
│       xdl-chart-viewer              │
│    (Tauri native window)            │
│   ┌───────────────────────────────┐ │
│   │   WebView (system WebView)    │ │
│   │   loads data:text/html… URL   │ │
│   │   CDN: echarts@5, echarts-gl@2│ │
│   └───────────────────────────────┘ │
└─────────────────────────────────────┘
```

---

## Files Created/Modified

### New crates

- `xdl-charts/` — ECharts HTML/JSON generator (~3 source files: `lib.rs`, `echarts.rs`, `templates.rs`; tests in each file)
- `xdl-chart-viewer/` — Standalone Tauri binary (`src/main.rs`, `tauri.conf.json`, `build.rs`, `icons/`, `tests/`)
- `xdl-desktop-viewer/` — Stub library (not wired into runtime)

### Modified files

- `Cargo.toml` — added `xdl-charts`, `xdl-chart-viewer` to `[workspace].members`
- `xdl-stdlib/Cargo.toml` — added `xdl-charts` dep
- `xdl-stdlib/src/lib.rs` — declared `mod charting_procs;` and wired 8 dispatch entries (also added `SET_PLOT_BACKEND` → `graphics_procs::set_plot_backend_proc`)
- `xdl-stdlib/src/charting_procs.rs` — new file (8 procs + helpers)
- `xdl-stdlib/src/graphics_procs.rs` — added `PlotBackend`, `PLOT_BACKEND`, `get/set_plot_backend`, branched `plot()` on backend

### Examples & docs

- `examples/charting/{*.xdl, *.m, README.md, MATLAB_PLOTTING_TESTS.md}`
- `docs/CHARTING_WEBGL_INVESTIGATION.md`
- `docs/CHARTING_IMPLEMENTATION_STATUS.md`
- `docs/CHARTING_FINAL_STATUS.md`
- `docs/PLOT_SURFACE_ECHARTS_INTEGRATION.md`
- `docs/ECHARTS_INTEGRATION_COMPLETE.md` (this file)
- `docs/TAURI_SUCCESS.md`

### Scratch files (dead in tree, not compiled)

- `xdl-stdlib/src/charting_procs.rs.bak`
- `xdl-stdlib/src/charting_procs_broken.rs`

---

## Technical Details

### Dependencies added

```toml
# xdl-charts/Cargo.toml
serde = { workspace = true, features = ["derive"] }
serde_json = "1.0"
anyhow = { workspace = true }
tracing = { workspace = true }

# xdl-stdlib/Cargo.toml
xdl-charts = { path = "../xdl-charts" }

# xdl-chart-viewer/Cargo.toml
tauri = { version = "2.1", features = ["devtools", "webview-data-url"] }
serde = { workspace = true, features = ["derive"] }
serde_json = "1.0"
clap = { workspace = true, features = ["derive"] }
urlencoding = "2.1"
[build-dependencies]
tauri-build = { version = "2.0" }
```

### No JavaScript dependencies

All JavaScript libraries (`echarts`, `echarts-gl`) are loaded via CDN; no npm/webpack in the Rust build.

### Binary size impact

`xdl-chart-viewer` adds a Tauri 2 binary to the workspace build. `xdl-charts` itself is a tiny library (~few KB compiled). No JavaScript bundling.

---

## Testing

### Manual testing checklist (from original doc)

- [x] `CHART_PLOT` with arrays — proc compiles; behavior depends on launching `xdl-chart-viewer`
- [x] `CHART_SCATTER` with random data — compiles
- [x] `CHART_BAR` with values — compiles
- [x] `SURFACE3D` with 2D matrix — compiles
- [x] `SCATTER3D` with 3D points — compiles
- [x] Large dataset (15K points) with WebGL — `CHART_SCATTER` sets `use_webgl = true` when `len > 10_000`
- [x] Multiple windows simultaneously — each call spawns a new process
- [x] Window interactions (zoom, pan, rotate) — ECharts toolbox enabled in 2D; 3D uses `grid3D.viewControl`
- [x] Non-blocking execution — `Command::spawn`, no `wait`

### Automated tests

- ✅ `xdl-charts` unit tests (`test_chart_config_default`, `test_series_2d_creation`, `test_chart_type_conversion`, `test_needs_3d_support`, `test_html_generation`)
- ✅ `xdl-desktop-viewer` unit tests (`test_window_config_default`, `test_window_counter_increments`)
- ⚠️ **No** charting tests in `xdl-stdlib` — `extract_f64_array`, `extract_2d_array`, `launch_chart` are not covered

```bash
cargo check --workspace
cargo test --workspace
```

---

## Performance Benchmarks (claim from original draft, not re-verified)

The original draft claimed:

| Chart Type | Data Size | Generation Time | Render Time | FPS |
|------------|-----------|-----------------|-------------|-----|
| Line | 100 points | ~50ms | ~100ms | 60 |
| Scatter | 1K points | ~60ms | ~120ms | 60 |
| Scatter (WebGL) | 15K points | ~200ms | ~500ms | 60 |
| Bar | 50 bars | ~50ms | ~100ms | 60 |
| Surface3D | 50x50 | ~150ms | ~300ms | 45-60 |
| Scatter3D | 100 points | ~100ms | ~200ms | 60 |

These were never measured or published in CI. Treat as unverified.

---

## Comparison to Alternatives (claim from original draft)

| Feature | Browser (viz3d-web) | Tauri (This) | Electron |
|---------|---------------------|--------------|----------|
| Startup | ~1s | ~500ms | ~800ms |
| Memory | ~120 MB | ~80 MB | ~200 MB |
| Bundle Size | 0 MB | ~5 MB | ~200 MB |
| UX | Browser chrome | Native | Native |
| Integration | HTTP server | Direct spawn | Complex |
| Maintenance | Simple | Simple | Complex |

Also unverified; included for completeness.

---

## Known Limitations (live)

1. **No keyword arguments yet** — `TITLE=`, `TYPE=`, `XRANGE=`, etc. are not wired. The dispatcher in `xdl-stdlib/src/lib.rs` already accepts a `keywords` HashMap, but `charting_procs` ignores it. The original draft's `CHART_PLOT, x, y, TITLE='Title', COLOR='blue'` example doesn't work today.

2. **Single series per chart** — multi-series needs procedure extension. Workaround: multiple calls (multiple windows).

3. **Hard-coded titles** — `CHART_CONTOUR`, `CHART_SHADE_SURF`, `CHART_PLOT3D` ignore their second/fourth positional arg and use hard-coded titles.

4. **Limited customization** — colors, styles are ECharts defaults. Future: add config options.

5. **Binary location dependency** — `xdl-chart-viewer` is resolved next to `std::env::current_exe()`. `cargo run` from the workspace will not find it (the host `xdl` binary lives at `target/debug/xdl`, not at the same level as a built `xdl-chart-viewer`). Workaround: `cargo install`, or build both into the same `target/{profile}/`, or add a `which`-style fallback (TODO).

6. **`xdl-desktop-viewer` is dead weight** — library compiles but is unused.

7. **`SURFACE`/`SHADE_SURF` (graphics_procs) do not route to ECharts** — they remain on plotters. To get ECharts 3D surfaces, use `SURFACE3D` directly.

---

## Future Enhancements

### Short term (next sprint)

1. Wire `keywords` through to `charting_procs` so users can write `CHART_PLOT, x, y, TITLE='…', TYPE='scatter'`.
2. Fix the hard-coded titles on `CHART_CONTOUR`/`CHART_SHADE_SURF`/`CHART_PLOT3D`.
3. Add `which`-style fallback for resolving `xdl-chart-viewer` at runtime.
4. Port the `extract_f64_array`/`extract_2d_array` unit tests from `charting_procs.rs.bak` into the live file.
5. Decide the fate of `xdl-desktop-viewer` (delete or wire in).
6. Delete `charting_procs.rs.bak` and `charting_procs_broken.rs`.

### Medium term

1. Multi-series charts in a single window (`PLOTADD`/`PLOTSHOW` accumulator).
2. Color/style customization via config.
3. Export to PNG/SVG (`toolbox.saveAsImage` is already enabled — needs Tauri-side wiring).
4. Dashboard layouts.

### Long term

1. D3.js integration for custom visualizations (deferred from the original investigation).
2. Three.js for advanced 3D / particle systems (deferred from the original investigation).
3. Real-time data updates and animations.

---

## Troubleshooting

### Charts don't open

```bash
# Verify xdl-chart-viewer works
./target/release/xdl-chart-viewer --title "Test"

# Check if it's co-located with xdl
ls target/release/ | grep xdl-chart-viewer

# Or copy it next to xdl
cp target/release/xdl-chart-viewer target/release/
```

### Script errors

```xdl
; Wrong: arrays different sizes
x = [1, 2, 3]
y = [1, 2]  ; Error!

; Right: same sizes
x = [1, 2, 3]
y = [1, 4, 9]
CHART_PLOT, x, y, 'Test'
```

### Tauri issues

```bash
# Rebuild with fresh icons
cd xdl-chart-viewer
cargo tauri icon source-icon.png
cargo build --release
```

---

## Commands Quick Reference

```bash
cargo build --release --workspace

./target/release/xdl examples/charting/simple_test.xdl
./target/release/xdl examples/charting/echarts_demo.xdl
./target/release/xdl-chart-viewer --title "Test"

cargo fmt --all
cargo test --workspace
cargo check --workspace
```

---

**Status:** ✅ MVP implemented; runtime UX not yet validated in CI.