# VIZ3D Interactive Visualization Usage

## Important Limitation

Due to a fundamental limitation in the windowing system (winit), **only one interactive 3D visualization window can be opened per XDL process**. After the first window is closed, attempting to open another window will result in an error.

## Workarounds

### Option 1: Run One Demo at a Time (Recommended)

Run each visualization in a separate XDL execution:

```bash
# Demo 1: Gaussian Blob
./target/release/xdl examples/demo/viz3d_demo1_gaussian.xdl

# Demo 2: Torus
./target/release/xdl examples/demo/viz3d_demo2_torus.xdl

# Demo 3: Turbulent Flow
./target/release/xdl examples/demo/viz3d_demo3_turbulence.xdl

# Demo 4: Spiral Galaxy
./target/release/xdl examples/demo/viz3d_demo4_galaxy.xdl
```

### Option 2: Use Non-Interactive Mode

Remove the `/INTERACTIVE` keyword to just prepare the visualization without opening a window:

```xdl
VIZ3D_RENDER, TITLE='My Visualization'  ; No /INTERACTIVE
```

This validates the data and configuration without creating a window.

### Option 3: Modify the Showcase Script

Comment out the `/INTERACTIVE` keyword on all but one `VIZ3D_RENDER` call in `viz3d_showcase.xdl`:

```xdl
; Only this one will open a window
VIZ3D_RENDER, /INTERACTIVE, TITLE='Demo 1: Gaussian Blob (Rainbow)'

; These will just prepare the data
VIZ3D_RENDER, TITLE='Demo 2: Torus (Viridis)'
VIZ3D_RENDER, TITLE='Demo 3: Turbulent Flow (Plasma)'
VIZ3D_RENDER, TITLE='Demo 4: Spiral Galaxy (Inferno)'
```

## Why This Limitation Exists

The winit event loop (used for window management) can only be created once per process. This is a design decision in winit for cross-platform compatibility and safety. Once the event loop runs and exits, it cannot be recreated.

## Future Solutions

Potential solutions being considered:

1. **Persistent Event Loop**: Keep the event loop alive and swap volume data between visualizations
2. **Multi-window Support**: Handle multiple windows within a single event loop
3. **Separate Process**: Launch each visualization in its own subprocess

These would require significant architectural changes to the visualization system.
