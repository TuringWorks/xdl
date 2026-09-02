# ECharts + Tauri Charting Implementation Status

**Branch:** `investigate-charting-webgl`
**Date:** 2025-10-25 (revised after full codebase grounding)
**Status:** ✅ MVP Implemented (Option A — separate Tauri process); clean `cargo check` workspace-wide.

> Status note: this document was originally written speculatively ("Phase 1 / 🚧 In Progress"). It has been revised to reflect the actual state of the workspace after grounding against the source. Grounded against files at: `xdl-charts/src/{lib,echarts,templates}.rs`, `xdl-chart-viewer/src/main.rs` (+ `tauri.conf.json`, `build.rs`), `xdl-desktop-viewer/src/lib.rs`, `xdl-stdlib/src/{lib,charting_procs}.rs`, `Cargo.toml`, `examples/charting/`, `xdl-viz3d-web/src/lib.rs`.

---

## Current Reality (grounded in the code)

| Component | Status | Where it lives |
|---|---|---|
| `xdl-charts` crate (ECharts option + HTML generation) | ✅ Implemented, builds clean | `xdl-charts/src/{lib,echarts,templates}.rs` |
| `xdl-chart-viewer` standalone Tauri binary | ✅ Implemented (CLI `--html-file`/`--html-content`/`--title`/`--width`/`--height`/`--help-only`; IPC `create_chart_window`; demo HTML fallback) | `xdl-chart-viewer/src/main.rs` + `tauri.conf.json` + `build.rs` |
| `xdl-desktop-viewer` library | ⚠️ Near-stub — `launch_window` only enqueues a `PendingWindow`; no Tauri event loop runs here | `xdl-desktop-viewer/src/lib.rs` |
| `xdl-stdlib::charting_procs` | ✅ Implemented, builds clean (8 procs) | `xdl-stdlib/src/charting_procs.rs` |
| Registration in `StandardLibrary::call_procedure` | ✅ Wired (8 entries) | `xdl-stdlib/src/lib.rs` |
| Example scripts | ✅ Present (`.xdl` + `.m`) | `examples/charting/` |
| Workspace `members` include new crates | ✅ `xdl-charts`, `xdl-chart-viewer` listed | `Cargo.toml` (`[workspace].members`) |
| `xdl-stdlib` depends on `xdl-charts` | ✅ Added (also still depends on `xdl-viz3d-web`, `xdl-viz3d`, `xdl-viz3d-threejs` for legacy 3D viz) | `xdl-stdlib/Cargo.toml` |
| Browser path via `xdl-viz3d-web` for charting | ❌ **Not used** — charting spawns the Tauri binary instead. `xdl-viz3d-web` is still in the dep graph for unrelated 3D viz | n/a |

**Workspace `cargo check` passes** (verified 2025-10-25; one unrelated future-incompat warning from `block v0.1.6`). The `xdl-stdlib` package compiles cleanly with the current `charting_procs.rs`. The two earlier broken attempts (`charting_procs.rs.bak`, `charting_procs_broken.rs`) are leftover scratch files and are **not** part of the build — only the corrected `charting_procs.rs` is referenced by `mod charting_procs;` in `xdl-stdlib/src/lib.rs`.

---

## What Was Actually Implemented

### 1. `xdl-charts` crate (`xdl-charts/Cargo.toml`)

Dependencies: `serde`, `serde_json`, `anyhow`, `tracing`. No `tauri` dep here.

- `ChartType` enum: `Line`, `Scatter`, `Bar`, `Area`, `Heatmap`, `Scatter3D`, `Surface3D`, `Bar3D`
- `ChartConfig` (with `Default`), `Series2D`, `Series3D` data structs
- Builders (in `echarts.rs`):
  - `build_2d_option(config, series)` — line/scatter/bar/area
  - `build_3d_option(config, series)` — scatter3D
  - `build_surface_option(config, z_data, x_range, y_range)` — ECharts `surface` series
  - `build_heatmap_option(config, data)` — `[[x, y, value]; N]` triples
- Template generator (in `templates.rs`): `create_echarts_html(config, option)` — embeds the option as JSON and injects the appropriate ECharts CDN `<script>` tag, conditionally adding `echarts-gl` for `Scatter3D`/`Surface3D`/`Bar3D`. Uses `data:text/html;charset=utf-8,…` data URLs (no asset server).
- Public entry points (in `lib.rs`): `generate_2d_chart`, `generate_3d_chart`, `generate_surface_plot`, `generate_heatmap`
- Tests: `test_chart_config_default`, `test_series_2d_creation`, `test_chart_type_conversion`, `test_needs_3d_support`, `test_html_generation`

### 2. `xdl-chart-viewer` Tauri binary (`xdl-chart-viewer/Cargo.toml`)

Dependencies: `tauri` (v2.1, with `devtools`, `webview-data-url`), `serde`, `serde_json`, `clap` (derive), `urlencoding`. `tauri-build` (v2.0) in `[build-dependencies]`.

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
- The HTML is URL-encoded into a `data:text/html;charset=utf-8,…` URL and loaded via `WebviewUrl::External(...)` (Tauri 2 data-URL feature).
- Tauri command `create_chart_window(app, data, state)` is exposed for IPC-based multi-window creation from a host process; it bumps a per-app `window_counter` and builds another `WebviewWindowBuilder`.

`tauri.conf.json`: `productName="XDL Chart Viewer"`, `identifier="com.xdl.chart-viewer"`, `withGlobalTauri=true`, `csp=null`, `bundle.active=true`, `targets="all"`, icon set under `icons/`. `build.rs` runs `tauri_build::build()`.

`xdl-chart-viewer/tests/` directory is present.

### 3. `xdl-stdlib::charting_procs` (the bridge)

Procedures (all return `XdlValue::Undefined`; the visible side-effect is a spawned chart window):

| Proc fn | XDL name (as dispatched) | Chart type | Min args | Title arg position | Other positional args |
|---|---|---|---|---|---|
| `plot` | `CHART_PLOT` | `Line` (2D) | `x, y` | 3rd (optional) | — |
| `scatter` | `CHART_SCATTER` | `Scatter` (2D, `use_webgl` if `len > 10_000`) | `x, y` | 3rd (optional) | — |
| `bar` | `CHART_BAR` | `Bar` (2D; x = `0..n`) | `values` | 2nd (optional) | — |
| `surface3d` | `SURFACE3D` | `Surface3D` (from 2D matrix) | `z_matrix` | 2nd (optional) | — |
| `scatter3d` | `SCATTER3D` | `Scatter3D` (x/y/z) | `x, y, z` | 4th (optional) | — |
| `contour` | `CHART_CONTOUR` | `Heatmap` (from 2D matrix; title hard-coded `"Contour Plot"`) | `z_matrix` | n/a (hard-coded) | — |
| `shade_surf` | `CHART_SHADE_SURF` | `Surface3D` (alias; title hard-coded `"Shaded Surface"`) | `z_matrix` | n/a (hard-coded) | — |
| `plot3d` | `CHART_PLOT3D` | `Scatter3D` (x/y/z; title hard-coded `"3D Line Plot"`) | `x, y, z` | n/a (hard-coded) | — |

Helper utilities:

- `extract_f64_array(&XdlValue) -> Vec<f64>` — handles `XdlValue::Array` (assumed `Vec<f64>`), `XdlValue::NestedArray` (via `to_double()` per element), and `XdlValue::MultiDimArray` (uses the flat `data` field).
- `extract_2d_array(&XdlValue) -> Vec<Vec<f64>>` — handles `NestedArray` (row-wise) and `MultiDimArray` (slice `data[i*cols..(i+1)*cols]` for `shape.len() == 2`).
- `extract_string(&XdlValue) -> String` — only `XdlValue::String` is accepted.
- `launch_chart(html, title)`:
  - Resolves `xdl-chart-viewer` next to `std::env::current_exe().parent()`. Adds `.exe` suffix on Windows; falls back to bare name if the suffixed binary isn't found.
  - Writes HTML to `${tempdir}/xdl_chart_<pid>.html` to avoid argv length limits.
  - Spawns: `Command::new(viewer_path).args(["--html-file", path, "--title", title]).spawn()`.

> Note: `launch_chart` is `fn`, not `pub`; the unit tests in `.bak` were never carried over — there are currently **no** `#[test]` items in the live `charting_procs.rs`.

### 4. Registration in `xdl-stdlib/src/lib.rs`

The module is declared privately: `mod charting_procs;` (not `pub mod`). It is dispatched from `StandardLibrary::call_procedure_with_keywords` (the `keywords` HashMap is currently ignored — all keyword-aware wiring is a TODO). The `call_procedure` entry point delegates here with an empty keyword map.

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

> Important divergence from the original doc: names are `CHART_*`-prefixed for 2D and most 3D, plus bare `SURFACE3D`/`SCATTER3D` for the two 3D helpers. This **intentionally** avoids collision with the existing GDL/IDL-compatible `graphics_procs`, which already binds bare `PLOT`, `SCATTER`, `BAR`, `SURFACE`, `CONTOUR`, `SHADE_SURF`, `PLOT3D` (and dispatches via `plotters`/`egui_plot`). The `CHART_*` prefix routes interactive ECharts to the new procs and leaves the legacy procedural pipeline untouched. The legacy bare names (`PLOT`, `CONTOUR`, `SHADE_SURF`, `PLOT3D`, etc.) remain bound to `graphics_procs::*` and dispatch first — so users must use `CHART_PLOT` to get ECharts.

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

`xdl-desktop-viewer` is **not** in the runtime path; it's a near-empty library scoped for an earlier "embed inside host app" design. It contains a `PendingWindow` queue plus a `create_window_in_app(app_handle, …)` helper that requires a host `AppHandle`. Nothing currently calls it from `charting_procs`. Its `launch_or_fallback` is a stub that calls a `fallback_to_browser` placeholder.

---

## Public API (as implemented)

```xdl
; 2D line plot
x = FINDGEN(100)
y = SIN(x / 10.0)
CHART_PLOT, x, y, 'Sine Wave'

; 2D scatter (auto WebGL > 10k points)
x = RANDOMU(seed, 100) * 10
y = RANDOMU(seed, 100) * 10
CHART_SCATTER, x, y, 'Random Scatter'

; Bar chart
CHART_BAR, [10, 20, 15, 35, 28], 'Bar Chart'

; 3D surface from matrix
z = FLTARR(50, 50)
FOR i = 0, 49 DO FOR j = 0, 49 DO z[i, j] = SIN(i/5.0) * COS(j/5.0)
SURFACE3D, z, 'Wave Pattern'

; 3D scatter from x/y/z
SCATTER3D, x, y, z, '3D Scatter'

; Heatmap / contour from 2D matrix (title hard-coded "Contour Plot")
CHART_CONTOUR, z

; Shaded surface (title hard-coded "Shaded Surface")
CHART_SHADE_SURF, z

; 3D line/scatter (title hard-coded "3D Line Plot")
CHART_PLOT3D, x, y, z
```

---

## Outstanding Work / Known Gaps

### Cleanup

- **Delete scratch files**: `xdl-stdlib/src/charting_procs.rs.bak` and `xdl-stdlib/src/charting_procs_broken.rs` — dead scratch files from prior failed attempts. Both contain syntactic errors (mismatched parens in `format!()`/`XdlError::RuntimeError(format!(...)`) and would not compile if `mod`'d in. Neither is referenced.
- **Decide the fate of `xdl-desktop-viewer`**: it is unused in the current path. Either delete the crate, or wire it into a future "embed in `xdl-gui`" mode (Option B). `xdl-desktop-viewer/Cargo.toml` still pulls `tauri v2.1` and is built by the workspace.

### Robustness

- **Binary resolution**: `launch_chart` looks next to `std::env::current_exe()` only. `cargo run`-from-workspace (where `current_exe()` is something like `target/debug/xdl`) and `cargo install` layouts will not find `xdl-chart-viewer` there. Add a `which`-style fallback (search `PATH`, look in `target/{profile}/`, look next to the host `xdl` binary) or honor an env var (`XDL_CHART_VIEWER`).
- **HTML argument length**: the proc already writes to a temp file to dodge argv limits — keep that path.
- **CLI argument passing**: `xdl-chart-viewer --html-content` is a string flag; current proc uses `--html-file` only. The CLI side supports both, but the proc only uses one.
- **Keyword args**: titles, types, axis labels, XRANGE/YRANGE etc. are currently positional (3rd/4th arg) only. Wiring to `call_procedure_with_keywords` (TITLE=, TYPE=, XRANGE=, YRANGE=, XLABEL=, YLABEL=, ZLABEL=) is not done. The dispatcher already accepts a `keywords` HashMap but `charting_procs` ignores it.
- **Hard-coded titles**: `CHART_CONTOUR`, `CHART_SHADE_SURF`, and `CHART_PLOT3D` ignore the second positional arg and use hard-coded titles. Inconsistent with the rest.
- **`CHART_PLOT` TYPE argument**: the `.bak`/broken drafts had a 4th positional arg for `TYPE=` (line/scatter/bar); the live `plot` doesn't honor it. This was a documented feature in the older draft and should be either re-added or explicitly removed from user-facing docs.
- **`PLOTADD` / `PLOTSHOW` multi-series accumulator**: not implemented.
- **`extract_f64_array` for `XdlValue::Array`**: the live code does `Ok(arr.clone())` which assumes the variant is already `Vec<f64>`. The XDL value enum may carry a more general `Array` (e.g., `Vec<XdlValue>`). Worth confirming against `xdl-core` and adding a fallback mapping.

### Verification still pending

- `cargo test -p xdl-stdlib` runs but **no charting tests are present** — extract/launch paths are uncovered.
- End-to-end manual run: build `xdl-chart-viewer`, place next to `xdl`, run `xdl examples/charting/simple_test.xdl`, confirm window opens.
- Examples under `examples/charting/` are written but not validated in CI:
  - `.xdl`: `echarts_demo.xdl`, `simple_test.xdl`, `minimal_for_test.xdl`, `simple_for_test.xdl`, `test_contour.xdl`, `test_echarts_contour.xdl`, `test_gui_output.xdl`, `test_nested_for.xdl`, `test_plot_surface.xdl`
  - `.m`: `matlab_comprehensive.m`, `matlab_plot_array.m`, `matlab_plot_multiple.m`, `matlab_plot_simple.m`, `test_gui_output.m`, `test_matlab_basic.m`, `test_range_with_arithmetic.m`
  - Docs: `README.md`, `MATLAB_PLOTTING_TESTS.md`
- Files at the repo root that mention `PLOT` (legacy `graphics_procs`), not `CHART_PLOT`: `plot3d_demo.xdl`, `plot_demo.xdl`, `plot_working_demo.xdl`, `test_advanced_arrays.xdl`. Leave as-is — they exercise the legacy pipeline.

### Deliberately not done

- **Browser fallback via `xdl-viz3d-web`** (original "Phase 1" idea). The implementation went with Option A (separate Tauri process). `xdl-viz3d-web::launch_browser_visualization` only accepts volume data (`Vec<f32>`, `[usize; 3]`) — it isn't a general-purpose HTML server, so reusing it would have meant widening its API. Skipped.
- **Embedding Tauri inside `xdl-gui`** (Option B). `xdl-desktop-viewer` was scoped for this but never wired.
- **CI integration** — charting tests not in CI.

---

## File Changes Actually Made (vs. the doc's plan)

| Planned (in original doc) | Actual | Notes |
|---|---|---|
| New `xdl-stdlib/src/charting_procs.rs` | ✅ Done | plus two scratch `.bak`/broken variants to remove |
| Register `PLOT`/`SCATTER`/`BAR`/`SURFACE3D` in `call_procedure` | ⚠️ Different — used `CHART_PLOT`/`CHART_SCATTER`/`CHART_BAR`/`SURFACE3D`/`SCATTER3D`/`CHART_CONTOUR`/`CHART_SHADE_SURF`/`CHART_PLOT3D` (avoids collision with existing `graphics_procs`) | intentional |
| `xdl-stdlib/Cargo.toml` adds `xdl-charts` dep | ✅ Done | |
| `xdl-stdlib/Cargo.toml` adds `xdl-viz3d-web` for browser fallback | ❌ Not done — fallback abandoned. `xdl-viz3d-web` is still listed but only because of the legacy 3D viz path; charting itself does not use it | |
| Standalone `xdl-chart-viewer` Tauri crate | ✅ Done, with CLI (`--html-file`/`--html-content`/`--title`/`--width`/`--height`/`--help-only`) + IPC `create_chart_window` + demo fallback | |
| `xdl-desktop-viewer` as primary Tauri runtime | ⚠️ Stubbed out; primary runtime is `xdl-chart-viewer` | |
| Workspace `members` includes `xdl-charts` and `xdl-chart-viewer` | ✅ Done | |
| Examples | ✅ Present in `examples/charting/` (`.xdl`, `.m`, READMEs) | |
| End-to-end testing | ⚠️ Manual run only — not in CI | |

---

## Next Steps (revised)

1. **Cleanup pass** — delete `xdl-stdlib/src/charting_procs.rs.bak` and `xdl-stdlib/src/charting_procs_broken.rs`; decide the fate of `xdl-desktop-viewer` (delete the crate or wire it into `xdl-gui`).
2. **Resolve `xdl-chart-viewer` at runtime** — search `PATH` / honor `XDL_CHART_VIEWER` env / look in `target/{profile}/` siblings of the host `xdl` binary; fail with a clear "xdl-chart-viewer not found" message if missing.
3. **Keyword arguments** — pipe `keywords` through `StandardLibrary::call_procedure_with_keywords` into charting procs so users can write `CHART_PLOT, x, y, TITLE='…', TYPE='scatter'`. Decide whether to reintroduce the 4th-arg `TYPE=` shorthand from the older draft or drop it from the docs.
4. **Tests** — port the unit tests from `.bak` (`extract_f64_array`, `extract_2d_array`) into `charting_procs.rs`, plus an integration test that spawns `xdl-chart-viewer --help-only` to confirm the binary is callable.
5. **Docs** — the README in `examples/charting/` already uses `CHART_PLOT` etc. and is correct; the only remaining doc cleanup is making sure `CHART_CONTOUR`/`CHART_SHADE_SURF`/`CHART_PLOT3D`'s hard-coded titles are either fixed or noted as a known limitation.
6. **(Optional) Browser fallback** — if we want a no-Tauri-build path, generalize `xdl-viz3d-web` to serve arbitrary HTML (it currently only serves volume templates) and add a `launch_chart_browser(html, title)` codepath in `charting_procs` keyed off `XDL_CHART_BACKEND=browser`.