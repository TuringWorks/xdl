# VIZ3D Three.js Implementation Plan

**Branch:** `viz3d-threejs`
**Date:** 2025-10-25
**Status:** Planning Phase

---

## Executive Summary

Port VIZ3D_* functions to use Three.js for volume rendering, leveraging the existing Tauri-based architecture from xdl-charts. This provides better browser compatibility and easier deployment compared to WebGPU while maintaining high-quality 3D visualizations.

---

## Goals

1. **Create xdl-viz3d-threejs crate** - Three.js-based volume rendering
2. **Maintain API compatibility** - Same VIZ3D_* procedures
3. **Add backend selection** - Runtime choice between WebGPU/Three.js
4. **Reuse Tauri infrastructure** - Consistent with xdl-charts
5. **Performance comparison** - Benchmark against existing WebGPU implementation

---

## Architecture

```
XDL Script
    ↓
VIZ3D_* procedures (xdl-stdlib)
    ↓
┌─────────────────┴─────────────────┐
│                                   │
xdl-viz3d (WebGPU)    xdl-viz3d-threejs (Three.js)
    ↓                               ↓
Native Window              Tauri Window (xdl-chart-viewer)
(winit + wgpu)            (HTML + Three.js)
```

---

## Implementation Phases

### Phase 1: Core Infrastructure (Week 1)

#### Deliverables:

1. **Create xdl-viz3d-threejs crate**
   - Location: `/xdl-viz3d-threejs/`
   - Dependencies: serde, serde_json
   - Core functions:
     - `generate_threejs_volume_html()` - HTML + Three.js template
     - `create_volume_shader()` - Custom raycasting shader
     - `create_colormap_texture()` - LUT generation
     - `launch_threejs_viewer()` - Integration with xdl-chart-viewer

2. **Three.js Volume Rendering Template**
   ```javascript
   - Data3DTexture for volume data
   - Custom ShaderMaterial for raycasting
   - OrbitControls for camera
   - GUI controls for parameters
   - Colormap texture lookups
   ```

3. **Backend Selection Mechanism**
   ```rust
   // In xdl-stdlib/src/viz3d.rs
   pub enum Viz3DBackend {
       WebGPU,    // xdl-viz3d (native winit)
       ThreeJS,   // xdl-viz3d-threejs (Tauri)
       Auto,      // Select based on environment
   }
   ```

#### Files to Create:
- `xdl-viz3d-threejs/Cargo.toml`
- `xdl-viz3d-threejs/src/lib.rs`
- `xdl-viz3d-threejs/src/templates.rs`
- `xdl-viz3d-threejs/src/shaders.rs`
- `xdl-viz3d-threejs/src/colormaps.rs`

---

### Phase 2: VIZ3D_* Integration (Week 1)

#### Deliverables:

1. **Modify xdl-stdlib/src/viz3d.rs**
   - Add backend detection
   - Route to appropriate renderer
   - Fallback logic: ThreeJS → WebGPU → Error

2. **Environment Variables**
   ```bash
   VIZ3D_BACKEND=threejs    # Force Three.js
   VIZ3D_BACKEND=webgpu     # Force WebGPU
   VIZ3D_BACKEND=auto       # Auto-detect (default)
   ```

3. **Function Mapping**
   | VIZ3D Function | Three.js Implementation |
   |----------------|-------------------------|
   | `VIZ3D_INIT` | Setup scene, camera, renderer |
   | `VIZ3D_VOLUME` | Create Data3DTexture |
   | `VIZ3D_COLORMAP` | Generate LUT texture |
   | `VIZ3D_CAMERA` | Set camera position/FOV |
   | `VIZ3D_RENDER` | Launch Tauri viewer with HTML |
   | `VIZ3D_TRANSFER` | Update shader uniforms |
   | `VIZ3D_LIGHT` | Add scene lights |
   | `VIZ3D_ISOSURFACE` | Marching cubes + Mesh |

#### Files to Modify:
- `xdl-stdlib/src/viz3d.rs`
- `xdl-stdlib/Cargo.toml` (add xdl-viz3d-threejs dependency)
- `Cargo.toml` (workspace member)

---

### Phase 3: Advanced Features (Week 2)

#### Deliverables:

1. **Transfer Functions**
   - Custom opacity curves
   - Multi-channel colormapping
   - Interactive adjustment via GUI

2. **Isosurface Extraction**
   - Implement marching cubes in JavaScript
   - Or use library: https://github.com/mikolalysenko/isosurface
   - Render as Three.js Mesh with lighting

3. **Lighting & Shading**
   - Phong shading for volumes
   - Multiple light sources
   - Shadow mapping (optional)

4. **Performance Optimizations**
   - Adaptive sampling rates
   - LOD for large volumes
   - Web Workers for data processing

#### Files to Create:
- `xdl-viz3d-threejs/src/isosurface.rs`
- `xdl-viz3d-threejs/src/transfer_function.rs`
- `xdl-viz3d-threejs/src/lighting.rs`

---

### Phase 4: Testing & Benchmarking (Week 2)

#### Deliverables:

1. **Test Suite**
   - Port existing VIZ3D demos to use Three.js backend
   - Verify visual parity with WebGPU
   - Test all colormaps, camera controls

2. **Performance Benchmarks**
   | Volume Size | Backend | Load Time | FPS | Memory |
   |-------------|---------|-----------|-----|--------|
   | 32³ | WebGPU | ? | ? | ? |
   | 32³ | Three.js | ? | ? | ? |
   | 64³ | WebGPU | ? | ? | ? |
   | 64³ | Three.js | ? | ? | ? |
   | 128³ | WebGPU | ? | ? | ? |
   | 128³ | Three.js | ? | ? | ? |

3. **Browser Compatibility**
   - Test on Chrome, Firefox, Safari
   - WebGL 2.0 fallback detection
   - Graceful degradation

#### Files to Create:
- `examples/viz3d/threejs_test.xdl`
- `examples/viz3d/backend_comparison.xdl`
- `tests/viz3d_threejs_test.sh`
- `docs/VIZ3D_THREEJS_BENCHMARK.md`

---

## Technical Details

### Three.js Volume Rendering Shader

```glsl
// Fragment shader for volume raycasting
uniform sampler3D u_volume;
uniform sampler2D u_colormap;
uniform float u_threshold;
uniform float u_opacity;
uniform vec3 u_volumeDims;

void main() {
    // Ray setup
    vec3 rayOrigin = cameraPosition;
    vec3 rayDir = normalize(vPosition - cameraPosition);

    // Ray marching
    vec4 color = vec4(0.0);
    float stepSize = 0.01;
    int maxSteps = 128;

    for (int i = 0; i < maxSteps; i++) {
        vec3 pos = rayOrigin + rayDir * float(i) * stepSize;

        // Volume sampling
        float density = texture(u_volume, pos).r;
        if (density < u_threshold) continue;

        // Colormap lookup
        vec4 sampleColor = texture(u_colormap, vec2(density, 0.5));
        sampleColor.a *= u_opacity;

        // Alpha compositing
        color.rgb += (1.0 - color.a) * sampleColor.rgb * sampleColor.a;
        color.a += (1.0 - color.a) * sampleColor.a;

        if (color.a >= 0.95) break;
    }

    gl_FragColor = color;
}
```

### HTML Template Structure

```html
<!DOCTYPE html>
<html>
<head>
    <title>{{title}}</title>
    <script type="importmap">{
        "imports": {
            "three": "https://cdn.jsdelivr.net/npm/three@0.161.0/build/three.module.js",
            "three/addons/": "https://cdn.jsdelivr.net/npm/three@0.161.0/examples/jsm/"
        }
    }</script>
</head>
<body>
    <canvas id="canvas"></canvas>
    <div id="controls">
        <!-- lil-gui controls -->
    </div>
    <script type="module">
        import * as THREE from 'three';
        import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
        import { GUI } from 'three/addons/libs/lil-gui.module.min.js';

        // Volume data embedded as JSON
        const volumeData = {{volume_data_json}};
        const dims = {{dimensions}};

        // Scene setup
        const scene = new THREE.Scene();
        const camera = new THREE.PerspectiveCamera(45, window.innerWidth / window.innerHeight, 0.1, 1000);
        const renderer = new THREE.WebGLRenderer({ canvas: document.getElementById('canvas'), antialias: true });
        const controls = new OrbitControls(camera, renderer.domElement);

        // Volume rendering setup
        // ... (shader, texture, mesh creation)

        // GUI controls
        const gui = new GUI();
        gui.add(params, 'threshold', 0, 1).name('Threshold');
        gui.add(params, 'opacity', 0, 1).name('Opacity');
        gui.add(params, 'colormap', colormaps).name('Colormap');

        // Animation loop
        function animate() {
            requestAnimationFrame(animate);
            renderer.render(scene, camera);
        }
        animate();
    </script>
</body>
</html>
```

---

## Success Criteria

- ✅ All VIZ3D_* procedures work with Three.js backend
- ✅ Visual quality comparable to WebGPU (within 10%)
- ✅ Performance acceptable for 64³ volumes (>30 FPS)
- ✅ Browser compatibility (Chrome, Firefox, Safari)
- ✅ Seamless backend switching via environment variable
- ✅ All existing demos run without modification
- ✅ Documentation updated with Three.js backend info

---

## Dependencies

### Rust Crates
- `serde`, `serde_json` - JSON serialization
- `xdl-core` - XDL types
- `xdl-desktop-viewer` - Tauri window management (existing)

### JavaScript Libraries
- Three.js r161+ (CDN)
- lil-gui (for controls)
- OrbitControls (Three.js addon)

---

## Migration Path

### For Users

1. **No code changes required** - Existing VIZ3D_* scripts work
2. **Optional backend selection**:
   ```bash
   # Use Three.js
   VIZ3D_BACKEND=threejs ./target/release/xdl script.xdl

   # Use WebGPU (original)
   VIZ3D_BACKEND=webgpu ./target/release/xdl script.xdl

   # Auto-detect (default)
   ./target/release/xdl script.xdl
   ```

3. **Fallback behavior**:
   - Try Three.js first (better compatibility)
   - Fall back to WebGPU if Three.js unavailable
   - Error if both fail

### For Developers

1. **Import xdl-viz3d-threejs**:
   ```rust
   use xdl_viz3d_threejs::generate_volume_html;
   ```

2. **Backend-agnostic API**:
   ```rust
   match backend {
       Viz3DBackend::WebGPU => xdl_viz3d::launch_visualization(...),
       Viz3DBackend::ThreeJS => xdl_viz3d_threejs::launch_visualization(...),
       _ => auto_select_backend(...),
   }
   ```

---

## Timeline

| Week | Phase | Deliverables |
|------|-------|--------------|
| 1 | Phase 1 | Core infrastructure, basic volume rendering |
| 1 | Phase 2 | VIZ3D_* integration, backend selection |
| 2 | Phase 3 | Advanced features (transfer functions, isosurface) |
| 2 | Phase 4 | Testing, benchmarking, documentation |

**Total Time:** 2 weeks

---

## Next Steps

1. ✅ Create branch `viz3d-threejs`
2. ✅ Create this planning document
3. 📋 Create xdl-viz3d-threejs crate skeleton
4. 📋 Implement basic Three.js volume rendering template
5. 📋 Integrate with VIZ3D_INIT and VIZ3D_RENDER
6. 📋 Test with existing demos
7. 📋 Add advanced features
8. 📋 Benchmark and document

---

**Status:** 📋 Ready to begin Phase 1
**Branch:** `viz3d-threejs`
**Estimated Completion:** 2025-11-08
