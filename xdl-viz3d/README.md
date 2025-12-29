# XDL-VIZ3D: 3D Volume Visualization

High-fidelity 3D visualization engine for scientific simulations using WebGPU.

## Features

- **Volume Ray Marching**: GPU-accelerated volume rendering
- **Interactive Navigation**: Arcball camera with mouse/keyboard controls
- **Scientific Colormaps**: Viridis, Plasma, Rainbow, Turbo, and more
- **Isosurface Extraction**: Render iso-contours of 3D scalar fields
- **Cross-Platform**: Runs on Metal, Vulkan, DirectX 12, and WebGPU
- **Web Export**: Compile to WebAssembly for browser visualization

## Quick Start

### From XDL Scripts

```idl
; rayleigh_taylor.xdl - Create and visualize 3D volume data

; Create 3D volume
size = 64
density = FLTARR(size, size, size)

; Fill with data
FOR i=0, size-1 DO FOR j=0, size-1 DO FOR k=0, size-1 DO $
    density[i,j,k] = EXP(-((i-32)^2 + (j-32)^2 + (k-32)^2) / 400.0)

; Initialize and render
VIZ3D_INIT, WINDOW_SIZE=[1280, 720]
VIZ3D_COLORMAP, 'VIRIDIS'
VIZ3D_VOLUME, density, DIMENSIONS=[size, size, size]
VIZ3D_RENDER, /INTERACTIVE
```

### Run Demo

```bash
# From XDL project root
xdl examples/demo/volume_render_simple.xdl

# Or the full Rayleigh-Taylor simulation
xdl examples/demo/rayleigh_taylor.xdl
```

## Architecture

```
XDL Script (.xdl)
    ↓
xdl-stdlib (VIZ3D_* functions)
    ↓
xdl-viz3d (Rust rendering engine)
    ↓
wgpu (WebGPU abstraction)
    ↓
GPU (Metal/Vulkan/DX12/WebGPU)
```

## API Functions

### Core Functions

- `VIZ3D_INIT` - Initialize visualization system
- `VIZ3D_VOLUME` - Upload volume data to GPU
- `VIZ3D_RENDER` - Render current volume
- `VIZ3D_COLORMAP` - Set colormap
- `VIZ3D_CAMERA` - Configure camera
- `VIZ3D_TRANSFER` - Set transfer function
- `VIZ3D_ISOSURFACE` - Extract and render isosurface

See [docs/VIZ3D_IMPLEMENTATION.md](../docs/VIZ3D_IMPLEMENTATION.md) for full API documentation.

## Examples

### Simple Volume
`examples/demo/volume_render_simple.xdl` - Basic Gaussian blob

### Rayleigh-Taylor Instability
`examples/demo/rayleigh_taylor.xdl` - Full fluid dynamics simulation

## Building

```bash
# Build the crate
cargo build --release -p xdl-viz3d

# Run tests
cargo test -p xdl-viz3d

# Build for WebAssembly
cargo build --target wasm32-unknown-unknown --release -p xdl-viz3d
```

## Implementation Status

- [x] Project structure and Cargo setup
- [x] Demo XDL scripts created
- [x] Implementation plan documented
- [ ] Core rendering engine
- [ ] Camera system
- [ ] Volume ray marching shaders
- [ ] Colormap system
- [ ] stdlib integration
- [ ] WebAssembly export

See TODO list for remaining work.

## Performance

Expected performance on modern GPUs:

| Grid Size | Frame Rate | Memory  |
|-----------|------------|---------|
| 64³       | 60+ FPS    | ~10 MB  |
| 128³      | 60 FPS     | ~50 MB  |
| 256³      | 30 FPS     | ~200 MB |
| 512³      | 15 FPS     | ~1 GB   |

## Dependencies

- `wgpu` - GPU abstraction layer
- `winit` - Cross-platform windowing
- `glam` - Linear algebra
- `bytemuck` - Safe GPU data casting

## Contributing

See main XDL project [Contributing section](../README.md#contributing).

## License

GPL-2.0 (same as XDL project)
