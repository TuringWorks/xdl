# XDL ECharts + Tauri Charting - Final Implementation Status

**Branch:** `investigate-charting-webgl`
**Date:** 2025-10-25 (revised after codebase grounding)
**Status:** ✅ MVP Implemented (Option A — separate Tauri process); clean `cargo check` workspace-wide.

> Status note: this document was originally written speculatively with paths "A vs B" decision points. It has been revised to reflect the actual state of the workspace after the ECharts+Tauri implementation was completed. The browser-first fallback (Path B) was abandoned; the standalone Tauri binary (Path A) is the live runtime.

---

## What Was Successfully Built

### ✅ 1. `xdl-charts` crate (Complete)

**Location:** `xdl-charts/` (`Cargo.toml`, `src/{lib,echarts,templates}.rs`)

**Features (verified in source):**

- `ChartType` enum: `Line`, `Scatter`, `Bar`, `Area`, `Heatmap`, `Scatter3D`, `Surface3D`, `Bar3D`
- ECharts JSON option builders (in `echarts.rs`):
  - `build_2d_option(config, series)` — line/scatter/bar/area
  - `build_3d_option(config, series)` — `scatter3D`
  - `build_surface_option(config, z_data, x_range, y_range)` — `surface`
  - `build_heatmap_option(config, data)` — `[[x, y, value]; N]` triples
- HTML template generation (in `templates.rs`): `create_echarts_html(config, option)` — embeds the option as JSON and injects the right ECharts CDN `<script>` tag, conditionally adding `echarts-gl` for `Scatter3D`/`Surface3D`/`Bar3D`. Uses `data:text/html;charset=utf-8,…` data URLs (no asset server).
- Public entry points (in `lib.rs`): `generate_2d_chart`, `generate_3d_chart`, `generate_surface_plot`, `generate_heatmap`
- Tests: `test_chart_config_default`, `test_series_2d_creation`, `test_chart_type_conversion`, `test_needs_3d_support`, `test_html_generation`

`Cargo.toml` deps: `serde` (with derive), `serde_json`, `anyhow`, `tracing`. **No `tauri` dep here** — this is a pure HTML/JSON generator.

**Usage example (matches live API):**

```rust
use xdl_charts::{ChartConfig, ChartType, Series2D, generate_2d_chart};

let config = ChartConfig {
    chart_type: ChartType::Scatter,
    title: "My Chart".to_string(),
    width: 1024,
    height: 768,
    ..Default::default()
};

let series = vec![Series2D {
    name: "Data".to_string(),
    x_data: vec![1.0, 2.0, 3.0],
    y_data: vec![4.0, 5.0, 6.0],
}];

let html = generate_2d_chart(&config, &series)?;
// html contains a complete standalone HTML page with ECharts
```

### ⚠️ 2. `xdl-desktop-viewer` crate (Near-stub, unused)

**Location:** `xdl-desktop-viewer/src/lib.rs`

This crate was scoped for an "embed inside `xdl-gui`" design (Option B) but was **never wired into the runtime path**. It currently provides:

- `WindowConfig` (title/width/height/resizable/decorations), `Default` impl
- `launch_window(html, config) -> Result<String>` — only enqueues a `PendingWindow`; no Tauri event loop runs here (no `tauri::Builder::default().run(...)`)
- `launch_window_impl` — push-only helper
- `create_window_in_app(app, …)` — requires a host `AppHandle`; nothing calls it
- `launch_or_fallback` — stub; calls a `fallback_to_browser` placeholder that returns `"browser-window"`
- `take_pending_windows()` — drain helper

`PendingWindow` is a private struct (`id`, `html`, `config`); `WindowConfig` is `Serialize`/`Deserialize`. Two `#[test]` items are present (`test_window_config_default`, `test_window_counter_increments`).

**Decision:** Either delete the crate, or wire it into a future "embed in `xdl-gui`" mode. Currently a no-op build artifact.

### ✅ 3. `xdl-chart-viewer` standalone Tauri binary (Complete, builds and runs)

**Location:** `xdl-chart-viewer/` (`Cargo.toml`, `src/main.rs`, `tauri.conf.json`, `build.rs`, `icons/`)

`Cargo.toml` deps: `tauri = "2.1"` (with `devtools`, `webview-data-url`), `serde`, `serde_json`, `clap` (derive), `urlencoding`. `tauri-build = "2.0"` in `[build-dependencies]`.

CLI surface (`Args` struct in `src/main.rs`):

| Flag | Type | Default | Purpose |
|---|---|---|---|
| `--html-file <path>` / `-f` | `Option<String>` | — | Read HTML from file |
| `--html-content <string>` / `-c` | `Option<String>` | — | HTML string (not base64) |
| `--title <s>` / `-t` | `String` | `"XDL Chart"` | Window title |
| `--width <u32>` / `-w` | `u32` | `1024` | Window width |
| `--height <u32>` / `-H` | `u32` | `768` | Window height |
| `--help-only` | bool | `false` | Print help and exit (used in tests to skip Tauri init) |

Behavior:

- `--help` and `--version` short-circuit before Tauri init (via `Args::try_parse()`'s `Err` arm).
- `--help-only` exits cleanly after `print_help()`.
- If neither `--html-file` nor `--html-content` is supplied, falls back to a built-in demo sine/cosine ECharts chart (`create_demo_chart_html(title)`).
- The HTML is URL-encoded into a `data:text/html;charset=utf-8,…` URL and loaded via `WebviewUrl::External(...)` (Tauri 2 `webview-data-url` feature).
- Tauri command `create_chart_window(app, data, state)` is exposed for IPC-based multi-window creation from a host process; it bumps a per-app `window_counter` and builds another `WebviewWindowBuilder`.

`tauri.conf.json`: `productName="XDL Chart Viewer"`, `identifier="com.xdl.chart-viewer"`, `withGlobalTauri=true`, `csp=null`, `bundle.active=true`, `targets="all"`, icon set under `icons/`. `build.rs` runs `tauri_build::build()`.

**The "icon loading error in Tauri runtime" referenced in the earlier draft of this doc does not block `cargo check` — the binary builds clean. Whether the WebView actually renders correctly at runtime is an end-to-end question that requires a manual GUI run, which has not been performed in CI.**

### ✅ 4. `xdl-stdlib::charting_procs` bridge (Complete, builds clean)

**Location:** `xdl-stdlib/src/charting_procs.rs` (8 procs)

Procedures (all return `XdlValue::Undefined`; the visible side-effect is a spawned chart window):

| Proc fn | XDL name | Chart type | Min args | Title arg position |
|---|---|---|---|---|
| `plot` | `CHART_PLOT` | `Line` (2D) | `x, y` | 3rd (optional) |
| `scatter` | `CHART_SCATTER` | `Scatter` (2D, WebGL if `len > 10_000`) | `x, y` | 3rd (optional) |
| `bar` | `CHART_BAR` | `Bar` (2D; x = `0..n`) | `values` | 2nd (optional) |
| `surface3d` | `SURFACE3D` | `Surface3D` (from 2D matrix) | `z_matrix` | 2nd (optional) |
| `scatter3d` | `SCATTER3D` | `Scatter3D` (x/y/z) | `x, y, z` | 4th (optional) |
| `contour` | `CHART_CONTOUR` | `Heatmap` (from 2D matrix; title hard-coded `"Contour Plot"`) | `z_matrix` | n/a (hard-coded) |
| `shade_surf` | `CHART_SHADE_SURF` | `Surface3D` (alias; title hard-coded `"Shaded Surface"`) | `z_matrix` | n/a (hard-coded) |
| `plot3d` | `CHART_PLOT3D` | `Scatter3D` (x/y/z; title hard-coded `"3D Line Plot"`) | `x, y, z` | n/a (hard-coded) |

Helpers:

- `extract_f64_array(&XdlValue) -> Vec<f64>` — handles `Array` (assumed `Vec<f64>`), `NestedArray` (via `to_double()` per element), `MultiDimArray` (uses the flat `data` field).
- `extract_2d_array(&XdlValue) -> Vec<Vec<f64>>` — handles `NestedArray` (row-wise) and `MultiDimArray` (slice `data[i*cols..(i+1)*cols]` for `shape.len() == 2`).
- `extract_string(&XdlValue) -> String` — only `XdlValue::String` accepted.
- `launch_chart(html, title)`:
  - Resolves `xdl-chart-viewer` next to `std::env::current_exe().parent()`. Adds `.exe` suffix on Windows; falls back to bare name if the suffixed binary isn't found.
  - Writes HTML to `${tempdir}/xdl_chart_<pid>.html` to avoid argv length limits.
  - Spawns: `Command::new(viewer_path).args(["--html-file", path, "--title", title]).spawn()`.

> Note: `launch_chart` is `fn`, not `pub`; the unit tests in `.bak` were never carried over — there are currently **no** `#[test]` items in the live `charting_procs.rs`.

### ✅ 5. Registration in `xdl-stdlib/src/lib.rs` (Complete)

Module is declared privately (`mod charting_procs;`, not `pub mod`). Dispatched from `StandardLibrary::call_procedure_with_keywords` (the `keywords` HashMap is currently ignored — keyword-aware wiring is a TODO):

```rust
"CHART_PLOT"       => charting_procs::plot(args),
"CHART_SCATTER"    => charting_procs::scatter(args),
"CHART_BAR"        => charting_procs::bar(args),
"CHART_CONTOUR"    => charting_procs::contour(args),
"CHART_SHADE_SURF" => charting_procs::shade_surf(args),
"CHART_PLOT3D"     => charting_procs::plot3d(args),
"SURFACE3D"        => charting_procs::surface3d(args),
"SCATTER3D"        => charting_procs::scatter3d(args),
```

> Important divergence from the original "Path B" plan: names are `CHART_*`-prefixed for 2D and most 3D, plus bare `SURFACE3D`/`SCATTER3D`. This **intentionally** avoids collision with the existing GDL/IDL-compatible `graphics_procs`, which already binds bare `PLOT`, `SCATTER`, `BAR`, `SURFACE`, `CONTOUR`, `SHADE_SURF`, `PLOT3D` (and dispatches via `plotters`/`egui_plot`). The legacy bare names remain bound to `graphics_procs::*` and dispatch first — so users must use `CHART_PLOT` to get ECharts.

---

## Architecture (as built)

```text
XDL Script (.xdl or .m)
        │
        ▼
xdl-interpreter
        │
        ▼
StandardLibrary::call_procedure("CHART_PLOT", args)
        │
        ▼
charting_procs::plot(args)
        ├─ extract_f64_array / extract_2d_array / extract_string
        ├─ build ChartConfig + Series2D (or Series3D)
        ├─ xdl_charts::generate_2d_chart(...)  → HTML string
        └─ launch_chart():
              write ${tempdir}/xdl_chart_<pid>.html
              Command::spawn("xdl-chart-viewer",
                  ["--html-file", path, "--title", title])
        │
        ▼
xdl-chart-viewer (separate process)
        ├─ Args::try_parse() (early-exits on --help / --version)
        ├─ read html (file or content or demo fallback)
        ├─ urlencoding::encode → "data:text/html;charset=utf-8,…"
        └─ WebviewWindowBuilder::new(app, "main",
                                     WebviewUrl::External(data_url))…
```

`xdl-desktop-viewer` is **not** in the runtime path. The browser-fallback path via `xdl-viz3d-web` was also not pursued — `xdl-viz3d-web::launch_browser_visualization` only accepts volume data (`Vec<f32>`, `[usize; 3]`) and isn't a general-purpose HTML server, so reusing it would have required widening its API. Skipped.

---

## What Was Originally Decided vs. What Shipped

| Original decision point (this doc's earlier draft) | Final state |
|---|---|
| Path A: fix Tauri icon issue | ✅ Built and `cargo check`s clean; runtime UX not validated in CI |
| Path B: browser-first via `xdl-viz3d-web` | ❌ Not done. `xdl-viz3d-web` is still listed in `xdl-stdlib/Cargo.toml` only because of the legacy 3D viz path; charting itself does not use it |
| Register bare `PLOT`/`SCATTER`/`BAR`/`SURFACE3D` | ⚠️ Different — `CHART_PLOT`/`CHART_SCATTER`/`CHART_BAR`/`SURFACE3D`/`SCATTER3D`/`CHART_CONTOUR`/`CHART_SHADE_SURF`/`CHART_PLOT3D`. Bare `PLOT`/`SCATTER`/`BAR`/`SURFACE`/`CONTOUR`/`SHADE_SURF`/`PLOT3D` continue to dispatch to `graphics_procs::*` (plotters/egui_plot) |
| Long-term: env-var toggle `XDL_USE_DESKTOP_VIEWER=1` | ❌ Not implemented. There is no env-var toggle; there is no fallback chain. `launch_chart` looks next to `current_exe()` only. |
| Decision criteria: "If users prefer desktop UX → Add Tauri support in Phase 3" | ✅ Tauri was adopted as the **only** path (no Phase 1 browser POC) |

---

## File Changes Actually Made

### New crates

1. `xdl-charts/` — ECharts option + HTML generation library
2. `xdl-chart-viewer/` — Standalone Tauri binary (CLI + IPC + demo fallback)
3. `xdl-desktop-viewer/` — Stubbed library (not in runtime path)

### Modified files

- `Cargo.toml` — added `xdl-charts` and `xdl-chart-viewer` to `[workspace].members`
- `xdl-stdlib/Cargo.toml` — added `xdl-charts` dep (and kept `xdl-viz3d-web`, `xdl-viz3d`, `xdl-viz3d-threejs` for legacy 3D viz)
- `xdl-stdlib/src/lib.rs` — declared `mod charting_procs;` and wired 8 dispatch entries
- `xdl-stdlib/src/charting_procs.rs` — new file (8 procs + helpers + `launch_chart`)

### Documentation

- `docs/CHARTING_WEBGL_INVESTIGATION.md` — pre-implementation investigation
- `docs/CHARTING_IMPLEMENTATION_STATUS.md` — implementation plan (revised)
- `docs/CHARTING_FINAL_STATUS.md` — this document
- `docs/PLOT_SURFACE_ECHARTS_INTEGRATION.md`, `docs/ECHARTS_INTEGRATION_COMPLETE.md` — integration write-ups (still need grounding)

### Examples (`examples/charting/`)

- `.xdl`: `echarts_demo.xdl`, `simple_test.xdl`, `minimal_for_test.xdl`, `simple_for_test.xdl`, `test_contour.xdl`, `test_echarts_contour.xdl`, `test_gui_output.xdl`, `test_nested_for.xdl`, `test_plot_surface.xdl`
- `.m`: `matlab_comprehensive.m`, `matlab_plot_array.m`, `matlab_plot_multiple.m`, `matlab_plot_simple.m`, `test_gui_output.m`, `test_matlab_basic.m`, `test_range_with_arithmetic.m`
- Docs: `README.md`, `MATLAB_PLOTTING_TESTS.md`

The placeholder `scatter_demo.xdl` referenced in the earlier draft of this doc **does not exist** in the tree. The actual file is `simple_test.xdl` and `echarts_demo.xdl`.

### Scratch files (NOT compiled, dead in tree)

- `xdl-stdlib/src/charting_procs.rs.bak`
- `xdl-stdlib/src/charting_procs_broken.rs`

Both contain mismatched parens in `format!()`/`XdlError::RuntimeError(format!(...)`) and would not compile if `mod`'d in. Neither is referenced by `xdl-stdlib/src/lib.rs`.

---

## Build Status

```bash
cargo check                         # ✅ clean (only block v0.1.6 future-incompat warning, unrelated)
cargo check -p xdl-charts           # ✅ no errors
cargo check -p xdl-desktop-viewer   # ✅ no errors
cargo check -p xdl-chart-viewer     # ✅ no errors
cargo check -p xdl-stdlib           # ✅ no errors
```

The `xdl-stdlib` package compiles cleanly with the current `charting_procs.rs`.

---

## Testing the Components

### Test `xdl-charts` (Rust)

```rust
use xdl_charts::*;

let config = ChartConfig::default();
let series = vec![Series2D {
    name: "test".to_string(),
    x_data: vec![1.0, 2.0],
    y_data: vec![3.0, 4.0],
}];

let html = generate_2d_chart(&config, &series).unwrap();
std::fs::write("test_chart.html", html).unwrap();
// Open test_chart.html in browser — should show chart
```

### Test `xdl-chart-viewer`

```bash
# With HTML file
./target/debug/xdl-chart-viewer -f test_chart.html --title "My Chart"

# With HTML content
./target/debug/xdl-chart-viewer -c "<html>...</html>" --title "Test"

# Default demo
./target/debug/xdl-chart-viewer --title "Demo"

# Help-only (CI-friendly, no GUI init)
./target/debug/xdl-chart-viewer --help-only
```

### End-to-end XDL test

```bash
./target/debug/xdl examples/charting/simple_test.xdl
# Spawns xdl-chart-viewer with a sine line chart in a native Tauri window
```

---

## Outstanding Work / Known Gaps

### Cleanup

- **Delete scratch files**: `xdl-stdlib/src/charting_procs.rs.bak` and `xdl-stdlib/src/charting_procs_broken.rs`.
- **Decide the fate of `xdl-desktop-viewer`**: unused in the current path. Either delete the crate, or wire it into a future "embed in `xdl-gui`" mode.

### Robustness

- **Binary resolution**: `launch_chart` looks next to `std::env::current_exe()` only. `cargo run`-from-workspace and `cargo install` layouts will not find `xdl-chart-viewer` there. Add a `which`-style fallback (search `PATH`, look in `target/{profile}/`, look next to the host `xdl` binary) or honor an env var (`XDL_CHART_VIEWER`).
- **CLI argument passing**: `xdl-chart-viewer --html-content` is a string flag; the proc uses `--html-file` only. The CLI side supports both, but the proc only uses one.
- **Keyword args**: titles, types, axis labels, XRANGE/YRANGE etc. are currently positional (3rd/4th arg) only. Wiring to `call_procedure_with_keywords` (TITLE=, TYPE=, XRANGE=, YRANGE=, XLABEL=, YLABEL=, ZLABEL=) is not done. The dispatcher already accepts a `keywords` HashMap but `charting_procs` ignores it.
- **Hard-coded titles**: `CHART_CONTOUR`, `CHART_SHADE_SURF`, `CHART_PLOT3D` ignore their 2nd/4th positional arg and use hard-coded titles. Inconsistent with the rest.
- **`CHART_PLOT` TYPE argument**: the `.bak`/broken drafts had a 4th positional arg for `TYPE=` (line/scatter/bar); the live `plot` doesn't honor it.
- **`PLOTADD` / `PLOTSHOW` multi-series accumulator**: not implemented.
- **`extract_f64_array` for `XdlValue::Array`**: the live code does `Ok(arr.clone())` which assumes the variant is already `Vec<f64>`. Worth confirming against `xdl-core` and adding a fallback mapping.

### Verification still pending

- `cargo test -p xdl-stdlib` runs but **no charting tests are present** — extract/launch paths are uncovered.
- End-to-end manual run: build `xdl-chart-viewer`, place next to `xdl`, run `xdl examples/charting/simple_test.xdl`, confirm window opens.
- Examples under `examples/charting/` are written but not validated in CI.
- Files at the repo root that mention `PLOT` (legacy `graphics_procs`), not `CHART_PLOT`: `plot3d_demo.xdl`, `plot_demo.xdl`, `plot_working_demo.xdl`, `test_advanced_arrays.xdl`. Leave as-is — they exercise the legacy pipeline.

### Deliberately not done

- **Browser fallback via `xdl-viz3d-web`** (original Path B). The implementation went with Option A (separate Tauri process). `xdl-viz3d-web::launch_browser_visualization` only accepts volume data and isn't a general-purpose HTML server.
- **Embedding Tauri inside `xdl-gui`** (Option B). `xdl-desktop-viewer` was scoped for this but never wired.
- **CI integration** — charting tests not in CI.

---

## Current State Summary (revised)

| Component | Status | Notes |
|---|---|---|
| `xdl-charts` | ✅ Complete | Pure HTML/JSON generator, no Tauri dep |
| `xdl-desktop-viewer` | ⚠️ Near-stub | Unused; library is fine, no runtime wiring |
| `xdl-chart-viewer` binary | ✅ Complete | CLI + IPC + demo fallback; runtime UX not validated in CI |
| `charting_procs` | ✅ Complete | 8 procs, no `#[test]` items |
| Registration in `call_procedure` | ✅ Complete | 8 entries; `CHART_*`-prefixed + bare `SURFACE3D`/`SCATTER3D` |
| `xdl-stdlib` ↔ `xdl-charts` dep | ✅ Added | |
| Workspace members include new crates | ✅ Added | |
| Example scripts | ✅ Present | In `examples/charting/` |
| End-to-end testing | ⚠️ Manual only | Not in CI |
| Browser fallback | ❌ Skipped | Was Path B in earlier draft |
| Keyword args (`TITLE=`, `TYPE=`, etc.) | ❌ Not implemented | |
| Hard-coded titles for 3 procs | ⚠️ Known limitation | |
| Tests for extract/launch paths | ❌ Not ported | `.bak` had them; not carried over |

---

## Next Steps (revised)

1. **Cleanup pass** — delete `charting_procs.rs.bak` and `charting_procs_broken.rs`; decide fate of `xdl-desktop-viewer` (delete the crate or wire it into `xdl-gui`).
2. **Resolve `xdl-chart-viewer` at runtime** — search `PATH` / honor `XDL_CHART_VIEWER` env / look in `target/{profile}/` siblings of the host `xdl` binary; fail with a clear "xdl-chart-viewer not found" message if missing.
3. **Keyword arguments** — pipe `keywords` through `StandardLibrary::call_procedure_with_keywords` into charting procs so users can write `CHART_PLOT, x, y, TITLE='…', TYPE='scatter'`. Decide whether to reintroduce the 4th-arg `TYPE=` shorthand from the older draft or drop it from the docs.
4. **Tests** — port the unit tests from `.bak` (`extract_f64_array`, `extract_2d_array`) into `charting_procs.rs`, plus an integration test that spawns `xdl-chart-viewer --help-only` to confirm the binary is callable.
5. **Docs** — the README in `examples/charting/` already uses `CHART_PLOT` etc. and is correct; the only remaining doc cleanup is making sure `CHART_CONTOUR`/`CHART_SHADE_SURF`/`CHART_PLOT3D`'s hard-coded titles are either fixed or noted as a known limitation.
6. **(Optional) Browser fallback** — if we want a no-Tauri-build path, generalize `xdl-viz3d-web` to serve arbitrary HTML (it currently only serves volume templates) and add a `launch_chart_browser(html, title)` codepath in `charting_procs` keyed off `XDL_CHART_VIEWER_BACKEND=browser`.