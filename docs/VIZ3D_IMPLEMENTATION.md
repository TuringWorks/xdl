# XDL 3D Visualization Implementation Plan

## Overview

This document outlines the implementation of high-fidelity 3D visualization capabilities for XDL, specifically designed to support scientific simulations like Rayleigh-Taylor instability rendering.

## Architecture

### Components

```
┌─────────────────────────────────────────────────────────────┐
│                      XDL User Scripts                        │
│              (.xdl files using VIZ3D_* functions)            │
└───────────────────────────┬─────────────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────────┐
│                    xdl-stdlib (Rust)                         │
│        VIZ3D function bindings (viz3d_init, etc.)            │
└───────────────────────────┬─────────────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────────┐
│                    xdl-viz3d (Rust)                          │
│              WebGPU-based rendering engine                   │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  Volume Renderer  │  Camera  │  Colormaps  │ Shaders │   │
│  └──────────────────────────────────────────────────────┘   │
└───────────────────────────┬─────────────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────────┐
│                      wgpu (WebGPU)                           │
│              Cross-platform GPU abstraction                  │
└───────────────────────────┬─────────────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────────┐
│            Native: Metal/Vulkan/DX12 │ Web: WebGPU          │
└─────────────────────────────────────────────────────────────┘
```

## XDL API Functions

The following functions will be added to `xdl-stdlib` for use in XDL scripts:

### Core Functions

#### `VIZ3D_INIT`
Initialize the 3D visualization system.

```idl
VIZ3D_INIT, WINDOW_SIZE=[width, height], TITLE='Title'
```

**Parameters:**
- `WINDOW_SIZE`: 2-element array [width, height] in pixels
- `TITLE`: Window title string

**Example:**
```idl
VIZ3D_INIT, WINDOW_SIZE=[1280, 720], TITLE='My Visualization'
```

---

#### `VIZ3D_VOLUME`
Upload 3D volume data to GPU.

```idl
VIZ3D_VOLUME, data, DIMENSIONS=[nx, ny, nz], /UPDATE
```

**Parameters:**
- `data`: 3D array of scalar values
- `DIMENSIONS`: 3-element array specifying grid dimensions
- `/UPDATE`: Update existing volume without reallocation

**Example:**
```idl
density = FLTARR(128, 128, 128)
VIZ3D_VOLUME, density, DIMENSIONS=[128, 128, 128]
```

---

#### `VIZ3D_RENDER`
Render the current volume.

```idl
VIZ3D_RENDER, /INTERACTIVE, TITLE='Frame Title'
```

**Parameters:**
- `/INTERACTIVE`: Enable interactive mode (blocks until window closed)
- `TITLE`: Optional frame title

**Example:**
```idl
VIZ3D_RENDER, /INTERACTIVE
```

---

### Configuration Functions

#### `VIZ3D_COLORMAP`
Set the colormap for value-to-color mapping.

```idl
VIZ3D_COLORMAP, name, MIN=min_val, MAX=max_val
```

**Parameters:**
- `name`: Colormap name ('RAINBOW', 'VIRIDIS', 'PLASMA', 'TURBO', 'GRAYSCALE')
- `MIN`: Minimum value for colormap range
- `MAX`: Maximum value for colormap range

**Example:**
```idl
VIZ3D_COLORMAP, 'VIRIDIS', MIN=0.0, MAX=1.0
```

---

#### `VIZ3D_TRANSFER`
Configure transfer function for volume rendering.

```idl
VIZ3D_TRANSFER, DENSITY=data, MODE='mode', ALPHA_SCALE=scale
```

**Parameters:**
- `DENSITY`: Reference density field
- `MODE`: Transfer function mode ('UNIFORM', 'GRADIENT', 'CUSTOM')
- `ALPHA_SCALE`: Opacity scaling factor [0.0-1.0]

**Example:**
```idl
VIZ3D_TRANSFER, DENSITY=density, MODE='GRADIENT', ALPHA_SCALE=0.8
```

---

#### `VIZ3D_CAMERA`
Set camera position and parameters.

```idl
VIZ3D_CAMERA, POSITION=[x,y,z], TARGET=[x,y,z], FOV=degrees
```

**Parameters:**
- `POSITION`: 3-element camera position
- `TARGET`: 3-element look-at target
- `FOV`: Field of view in degrees

**Example:**
```idl
VIZ3D_CAMERA, POSITION=[0, 0, 3], TARGET=[0, 0, 0], FOV=45.0
```

---

#### `VIZ3D_LIGHT`
Configure lighting parameters.

```idl
VIZ3D_LIGHT, POSITION=[x,y,z], INTENSITY=value, COLOR=[r,g,b]
```

**Parameters:**
- `POSITION`: Light position
- `INTENSITY`: Light intensity [0.0-1.0]
- `COLOR`: RGB color values [0-255]

---

### Advanced Functions

#### `VIZ3D_ISOSURFACE`
Extract and render isosurface.

```idl
VIZ3D_ISOSURFACE, data, ISOVALUE=value, COLOR=[r,g,b]
```

**Parameters:**
- `data`: 3D volume data
- `ISOVALUE`: Scalar value for isosurface extraction
- `COLOR`: RGB color for surface

**Example:**
```idl
VIZ3D_ISOSURFACE, density, ISOVALUE=1.0, COLOR=[255, 100, 50]
```

---

## Implementation Steps

### Phase 1: Core Infrastructure (Week 1-2)

1. **Create `xdl-viz3d` crate**
   - Set up Cargo.toml with dependencies
   - Initialize wgpu device and surface
   - Basic window creation with winit

2. **Implement Camera System**
   - Arcball camera with mouse controls
   - View/projection matrix computation
   - Uniform buffer management

3. **Volume Data Structures**
   - 3D texture allocation
   - Data upload to GPU
   - Format conversions

### Phase 2: Rendering Pipeline (Week 3-4)

4. **Volume Ray Marching Shaders (WGSL)**
   ```wgsl
   @group(0) @binding(0) var<uniform> camera: CameraUniform;
   @group(0) @binding(1) var volume_texture: texture_3d<f32>;
   @group(0) @binding(2) var volume_sampler: sampler;
   @group(0) @binding(3) var colormap: texture_1d<f32>;

   @fragment
   fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
       // Ray marching through volume
       let ray_dir = normalize(in.world_pos - camera.position);
       var color = vec4<f32>(0.0);
       var t = 0.0;

       for (var i = 0; i < 256; i++) {
           let pos = in.world_pos + ray_dir * t;
           let density = textureSample(volume_texture, volume_sampler, pos);
           let mapped_color = textureSample(colormap, volume_sampler, density);

           // Front-to-back compositing
           color += mapped_color * (1.0 - color.a);
           if (color.a > 0.95) { break; }

           t += 0.01;
       }

       return color;
   }
   ```

5. **Scientific Colormaps**
   - Rainbow, Viridis, Plasma, Turbo
   - 1D texture lookup tables
   - Dynamic range mapping

6. **Transfer Functions**
   - Opacity based on density
   - Gradient-based opacity
   - Custom transfer function support

### Phase 3: XDL Integration (Week 5)

7. **stdlib Bindings**
   - Add VIZ3D_* functions to `xdl-stdlib/src/graphics.rs`
   - Handle array data marshalling
   - Error handling and validation

8. **Interpreter Integration**
   - Register viz3d functions
   - Handle blocking/async rendering
   - Memory management

### Phase 4: Demo & Polish (Week 6)

9. **Rayleigh-Taylor Demo**
   - Simplified physics simulation
   - Time evolution visualization
   - Interactive controls

10. **Documentation & Examples**
    - API documentation
    - Tutorial examples
    - Performance optimization guide

### Phase 5: Web Export (Week 7-8)

11. **WebAssembly Support**
    - wasm32 target compilation
    - WebGPU backend
    - HTML/JS wrapper

12. **Web Demo Page**
    - Browser-based visualization
    - No installation required
    - Share via URL

## Technical Challenges & Solutions

### Challenge 1: Large Volume Data
**Problem:** 512³ volumes = 512MB+ of data
**Solution:**
- Streaming texture updates
- Compression (brick-based)
- LOD (Level of Detail) rendering

### Challenge 2: Performance
**Problem:** Real-time rendering of complex volumes
**Solution:**
- Early ray termination
- Empty space skipping
- Adaptive sampling rates

### Challenge 3: Colormap Quality
**Problem:** Scientific colormaps must be perceptually uniform
**Solution:**
- Use established colormaps (Viridis, Turbo)
- Support for custom colormaps
- Automatic contrast adjustment

### Challenge 4: Cross-platform
**Problem:** Different GPU backends (Metal, Vulkan, DX12, WebGPU)
**Solution:**
- wgpu abstraction layer handles this
- Test on all platforms
- Fallback to CPU rendering if needed

## Example Usage

### Simple Gaussian Blob
```idl
; Create data
size = 64
volume = FLTARR(size, size, size)
center = size / 2
FOR i = 0, size-1 DO FOR j = 0, size-1 DO FOR k = 0, size-1 DO $
    volume[i,j,k] = EXP(-((i-center)^2+(j-center)^2+(k-center)^2)/(size/4)^2)

; Visualize
VIZ3D_INIT
VIZ3D_COLORMAP, 'VIRIDIS'
VIZ3D_VOLUME, volume, DIMENSIONS=[size,size,size]
VIZ3D_RENDER, /INTERACTIVE
```

### Rayleigh-Taylor
See `examples/demo/rayleigh_taylor.xdl` for full demo.

## Performance Targets

| Grid Size | Frame Rate | GPU Memory |
|-----------|------------|------------|
| 64³       | 60+ FPS    | < 10 MB    |
| 128³      | 60 FPS     | < 50 MB    |
| 256³      | 30 FPS     | < 200 MB   |
| 512³      | 15 FPS     | < 1 GB     |

## Dependencies

### Rust Crates
- `wgpu` (0.18): GPU abstraction
- `winit` (0.29): Window management
- `glam` (0.25): Math library
- `bytemuck`: Safe casting for GPU buffers

### System Requirements
- GPU with WebGPU support (or Vulkan/Metal/DX12)
- 4GB+ GPU memory recommended for large volumes

## Testing Strategy

1. **Unit Tests**: Individual components (camera, colormaps)
2. **Integration Tests**: Full pipeline with known data
3. **Visual Tests**: Compare rendered output with reference images
4. **Performance Tests**: Benchmark on various hardware

## Future Enhancements

- [ ] Multiple volume support (composite rendering)
- [ ] Vector field visualization (flow lines)
- [ ] Time series animation controls
- [ ] VR/AR support
- [ ] Distributed rendering for supercomputer integration
- [ ] Real-time physics coupling

## References

- [Rayleigh-Taylor Instability](https://en.wikipedia.org/wiki/Rayleigh%E2%80%93Taylor_instability)
- [Volume Rendering Techniques](https://www.cs.unc.edu/~jeffi/papers/RT-VolumeVisualization.pdf)
- [Scientific Color Maps](https://www.kennethmoreland.com/color-advice/)
- [WebGPU Specification](https://www.w3.org/TR/webgpu/)
