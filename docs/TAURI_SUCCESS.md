# ✅ Tauri Integration - SUCCESS

**Branch:** `investigate-charting-webgl`
**Date:** 2025-10-25 (revised after codebase grounding)
**Status:** ✅ `xdl-chart-viewer` builds clean; runtime UX not validated in CI.

> Status note: this doc was originally written as an exception report ("here's what fixed the icon error"). It has been folded into the larger ECharts+Tauri story and revised against the actual code.

---

## What Shipped

The Tauri chart viewer is fully implemented and `cargo check`s clean. The CLI works as described, the runtime UX (window actually appearing) has not been verified in CI — that requires a manual GUI run.

### Final state (verified against source)

**`xdl-chart-viewer/Cargo.toml`:**

```toml
[build-dependencies]
tauri-build = { version = "2.0", features = [] }

[dependencies]
tauri = { version = "2.1", features = ["devtools", "webview-data-url"] }
serde = { workspace = true, features = ["derive"] }
serde_json = "1.0"
clap = { workspace = true, features = ["derive"] }
urlencoding = "2.1"
```

**`xdl-chart-viewer/tauri.conf.json`** (`productName="XDL Chart Viewer"`, `identifier="com.xdl.chart-viewer"`):

```json
"bundle": {
  "active": true,
  "icon": [
    "icons/32x32.png",
    "icons/64x64.png",
    "icons/128x128.png",
    "icons/128x128@2x.png",
    "icons/icon.png",
    "icons/icon.icns",
    "icons/icon.ico"
  ],
  "targets": "all"
}
```

There is no pre-configured window in the conf — the code creates it. `withGlobalTauri=true` and `csp=null`. `frontendDist=""` (empty — HTML is supplied at runtime).

---

## CLI Reference (live)

| Flag | Type | Default | Purpose |
|---|---|---|---|
| `--html-file <path>` / `-f` | `Option<String>` | — | Read HTML from file |
| `--html-content <string>` / `-c` | `Option<String>` | — | HTML string (no base64) |
| `--title <s>` / `-t` | `String` | `"XDL Chart"` | Window title |
| `--width <u32>` / `-w` | `u32` | `1024` | Window width |
| `--height <u32>` / `-H` | `u32` | `768` | Window height |
| `--help-only` | bool | `false` | Print help and exit (used in CI tests) |

### Invocation examples (verified working as code paths)

```bash
# Default demo (sine/cosine chart)
./target/debug/xdl-chart-viewer --title "My Chart"

# From HTML file
./target/debug/xdl-chart-viewer -f chart.html --title "Custom Chart"

# Custom size
./target/debug/xdl-chart-viewer --title "Big Chart" -w 1400 -H 900
```

Output (from `src/main.rs` setup):

```text
XDL Chart Viewer started
Window ID: main
```

### End-to-end test

```rust
use xdl_charts::{ChartConfig, ChartType, Series2D, generate_2d_chart};

let config = ChartConfig {
    chart_type: ChartType::Scatter,
    title: "My Data".to_string(),
    width: 1024,
    height: 768,
    ..Default::default()
};

let series = vec![Series2D {
    name: "Points".to_string(),
    x_data: vec![1.0, 2.0, 3.0, 4.0, 5.0],
    y_data: vec![2.5, 4.2, 3.1, 5.8, 4.9],
}];

let html = generate_2d_chart(&config, &series)?;
std::fs::write("my_chart.html", html)?;
```

```bash
./target/debug/xdl-chart-viewer -f my_chart.html --title "My Chart"
```

---

## Architecture (live)

```text
┌─────────────────────────────────────────┐
│          XDL Script (.xdl or .m)        │
└────────────────┬────────────────────────┘
                 │
                 ↓
┌─────────────────────────────────────────┐
│      xdl-stdlib charting procedures     │
│   (CHART_PLOT, CHART_SCATTER,           │
│    CHART_BAR, SURFACE3D, SCATTER3D,     │
│    CHART_CONTOUR, CHART_SHADE_SURF,     │
│    CHART_PLOT3D)                        │
│   + PLOT (when SET_PLOT_BACKEND is      │
│     'ECHARTS')                          │
└────────────────┬────────────────────────┘
                 │
                 ↓
┌─────────────────────────────────────────┐
│            xdl-charts                   │
│   generate_2d_chart / generate_3d_chart │
│   generate_surface_plot / generate_     │
│   heatmap → HTML string                 │
└────────────────┬────────────────────────┘
                 │
                 ↓
         ┌───────┴────────┐
         │                │
         ↓                ↓
┌────────────────┐  ┌──────────────────┐
│  launch_chart  │  │  standalone test │
│  (in charting_ │  │  (chart viewer   │
│   procs.rs)    │  │   launched       │
│                │  │   directly)      │
└────────┬───────┘  └──────────────────┘
         │
         ▼
  Command::spawn("xdl-chart-viewer",
      ["--html-file", tmpfile, "--title", title])
         │
         ▼
┌─────────────────────────────────────────┐
│   xdl-chart-viewer (Tauri)             │
│   reads HTML, encodes as data: URL,     │
│   opens WebviewWindow via Tauri's      │
│   system WebView (data-URL feature)    │
└─────────────────────────────────────────┘
```

---

## Features Shipped (verified)

- ✅ Native desktop window via Tauri 2
- ✅ WebView rendering (system WebView, not bundled Chromium)
- ✅ Data URL support (`webview-data-url` feature in `tauri = "2.1"`)
- ✅ Command-line interface (`clap` derive; 6 flags incl. `--help-only`)
- ✅ Built-in demo chart (`create_demo_chart_html` fallback when no HTML provided)
- ✅ IPC command `create_chart_window(app, data, state)` for multi-window use from a host process
- ✅ Cross-platform via Tauri (macOS WKWebView, Windows WebView2, Linux WebKitGTK)

## Features Not Yet Validated

- ⏸ Whether the WebView actually renders correctly at runtime (no GUI run in CI)
- ⏸ Whether Tauri's icon load works on all platforms (no runtime test)
- ⏸ Performance numbers in the original doc (startup, memory, FPS) are unverified

---

## Build / Run Commands

### Basic

```bash
cargo build -p xdl-chart-viewer
cargo build -p xdl-chart-viewer --release
cargo run -p xdl-chart-viewer -- --title "Test"
```

### Development

```bash
cargo watch -x 'build -p xdl-chart-viewer'
cargo check -p xdl-chart-viewer
cargo test -p xdl-chart-viewer
```

---

## Performance (claim from original doc, unverified)

The original draft cited:

- Startup ~500ms
- Chart render < 100ms
- Memory ~80 MB (incl. WebView)
- CPU < 5% idle, < 20% during interaction
- FPS 60 (smooth animations)

These were never measured in CI. Treat as unverified.

### Browser comparison (also unverified)

| Metric | Browser Tab | Tauri Window |
|--------|-------------|--------------|
| Startup | ~1s (server + browser) | ~500ms |
| Memory | ~120 MB | ~80 MB |
| UX | Browser chrome | Native window |
| Integration | HTTP server | Direct spawn |

---

## Troubleshooting

### Window doesn't appear

```bash
# Check if the process is running
ps aux | grep xdl-chart-viewer

# Run with stderr to surface any errors
./target/debug/xdl-chart-viewer --title "Test" 2>&1 | head -20

# Or try the demo (no HTML file needed)
./target/debug/xdl-chart-viewer
```

### Icons missing / mis-built

```bash
cd xdl-chart-viewer
ls -lh icons/
cargo tauri icon source-icon.png  # regenerates
cargo build
```

### Build errors

```bash
cargo clean -p xdl-chart-viewer
cargo build -p xdl-chart-viewer
cargo tauri info   # tauri-cli debug
```

---

## Files Modified

### Configuration

- `xdl-chart-viewer/Cargo.toml` — added `webview-data-url` to Tauri feature list
- `xdl-chart-viewer/tauri.conf.json` — icon set listed under `bundle.icon`, no pre-configured window

### Code

- `xdl-chart-viewer/src/main.rs` — unchanged from earlier draft: clap args, `Args::try_parse()` for early `--help`/`--version` exit, demo fallback, `WebviewWindowBuilder::new(...)` for the main window, `create_chart_window` IPC command

---

## What Was Originally Reported vs. What Shipped

| Original "fix" claim | Actual code state |
|---|---|
| "Added icon configuration in `bundle.icon`" | ✅ Done — icons listed |
| "Removed duplicate window from conf" | ✅ Done — conf has no window block |
| "Enabled `webview-data-url` feature" | ✅ Done — in `tauri = { version = "2.1", features = ["devtools", "webview-data-url"] }` |
| "Runtime icon loading fix" | ⚠️ Not validated — `cargo check` passes but no GUI run |

The reported "duplicated window between conf and code" was a real cleanup — `tauri.conf.json` does not declare any window, and `src/main.rs` creates `"main"` directly via `WebviewWindowBuilder`.

---

## Success Checklist (live)

- [x] Icon set configured in `tauri.conf.json`
- [x] Window created in code (no duplicate declaration)
- [x] `webview-data-url` feature enabled (data-URL HTML loading)
- [x] `cargo check` clean
- [x] Default demo chart implemented (`create_demo_chart_html`)
- [x] Interactive features enabled in generated HTML (ECharts toolbox)
- [x] Command-line arguments parsed (`clap` derive)
- [x] Multiple chart types supported (via `xdl-charts`)
- [x] Ready for XDL stdlib integration (`charting_procs` wires `Command::spawn` here)

## Open Questions

- [ ] Does the WebView render correctly at runtime? (Needs manual test on macOS/Linux/Windows.)
- [ ] Do the bundled icons load correctly? (Needs manual test.)
- [ ] Where does the host `xdl` binary resolve `xdl-chart-viewer` from in `cargo run` / `cargo install` layouts? (Today: `current_exe().parent()` only; likely broken for `cargo run`.)

---

**Status:** ✅ Builds clean; runtime UX pending manual validation.