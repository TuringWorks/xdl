# Geophysical Data Visualization Demo - User Guide

## Overview

The geophysical demo (`examples/scientific/geophysical_demo.xdl`) is a comprehensive workflow that demonstrates XDL's capabilities for 3D seismic data analysis and visualization. It generates synthetic seismic data with realistic geological features and provides interactive 3D visualization.

## Features

### Data Generation

- **3D Seismic Cube**: 64×64×32 volume (configurable)
- **Geological Structures**:
  - Anticline (dome) structure for hydrocarbon trapping
  - Normal fault with vertical offset
  - Multiple stratigraphic layers
- **Realistic Physics**: Reflection coefficients, wavelet modeling

### Analysis Capabilities

1. **Section Extraction**
   - Inline sections (constant X)
   - Crossline sections (constant Y)
   - Time slices (constant Z)

2. **Seismic Attributes**
   - Instantaneous amplitude
   - Vertical gradient
   - Horizontal gradient (fault indicator)

3. **Automated Interpretation**
   - Fault detection via gradient analysis
   - Horizon auto-picking
   - Structural interpretation

4. **Statistical Analysis**
   - Amplitude statistics
   - Signal-to-noise ratio
   - Frequency content estimation

5. **3D Volume Visualization**
   - Interactive ray-casting volume renderer
   - Configurable colormap (Rainbow, Viridis, etc.)
   - Real-time rotation, zoom, and pan
   - Threshold and opacity controls

## Running the Demo

### Basic Execution

```bash
# Run the complete demo with 3D visualization
xdl examples/scientific/geophysical_demo.xdl
```

### Environment Variables

```bash
# Use specific VIZ3D backend
VIZ3D_BACKEND=threejs xdl examples/scientific/geophysical_demo.xdl

# Available backends:
# - threejs (default): Three.js WebGL (best compatibility)
# - webgpu: Native WebGPU renderer
# - browser: Browser-based WebGPU
```

## Workflow Steps

### Step 1: Survey Parameters

```text
Survey dimensions:  64 x 64 x 32
Inline spacing:     25.0 m
Crossline spacing:  25.0 m
Time sampling:      4.0 ms
```

### Step 2: Geological Model

Creates stratigraphic layers:

- Layer 1: Shallow sediments (0-30ms)
- Layer 2: Sandstone reservoir with anticline (30-50ms)
- Layer 3: Faulted shale cap (50-70ms)
- Layer 4: Deeper formations (70-128ms)

### Step 3: Seismic Data Generation

Generates synthetic seismic amplitudes with:

- Reflection wavelets at layer boundaries
- Anticline geometry (petroleum trap)
- Normal fault with ~20ms throw
- Realistic noise

### Step 4-6: Section Extraction

Extracts representative sections:

- Inline section at mid-position
- Crossline section through anticline
- Time slice at reservoir level (40ms)

### Step 7: Seismic Attributes

Computes diagnostic attributes:

- Instantaneous amplitude (envelope)
- Vertical gradient (layer boundaries)
- Horizontal gradient (fault indicator)

### Step 8: Fault Detection

- Gradient-based fault probability map
- Identifies fault location and orientation
- Estimates fault throw

### Step 9: Horizon Tracking

- Auto-picks reservoir horizon (40ms ± 10ms)
- Tracks anticline structure
- Computes structural relief

### Step 10: Structural Interpretation

Analyzes geological features:

- Anticline closure area (km²)
- Structural relief (ms)
- Fault characteristics (type, throw, location)

### Step 11: Amplitude Analysis

- Extracts amplitude at reservoir level
- Interprets hydrocarbon indicators
- Classifies reflection strength

### Step 12: Data Quality

Assesses seismic data quality:

- Mean amplitude and standard deviation
- Signal-to-noise ratio
- RMS amplitude
- Frequency content estimation

### Step 13: 3D Volume Visualization ✨

**Interactive 3D rendering** of the seismic cube:

- **Window**: 1280×720 HD display
- **Colormap**: Rainbow (shows amplitude variations)
- **Camera**: Positioned for optimal viewing angle
- **Volume**: Full 64×64×32 seismic cube
- **Features visible**: Anticline and fault structures

## 3D Visualization Controls

When the 3D viewer opens:

### Mouse Controls

- **Left Click + Drag**: Rotate camera around volume
- **Mouse Wheel**: Zoom in/out
- **Right Click + Drag**: Pan camera

### GUI Controls

The viewer includes sliders for:

- **Threshold**: Filter low-amplitude values
- **Opacity**: Adjust transparency
- **Colormap**: Switch color schemes
- **Camera Reset**: Return to default view

### Keyboard Shortcuts

- **ESC**: Close viewer
- **Space**: Pause/resume rotation (if auto-rotating)
- **R**: Reset camera to default position

## Output Example

```text
> Step 13: Launching 3D visualization...

VIZ3D: Initialized (1280x720)
  Volume loaded:  64  x  64  x  32
  Amplitude range: [ -0.1 ,  0.9 ]

  Rendering 3D seismic cube...
  - Blue/negative: Troughs (possible hydrocarbon indicators)
  - Red/positive: Peaks (strong reflections)
  - Structures: Anticline and fault visible in volume

🚀 Launching Three.js volume visualization...
Controls:
  - Left mouse: Rotate camera
  - Mouse wheel: Zoom in/out
  - GUI sliders: Adjust threshold and opacity

✓ Three.js volume visualization launched
  Volume: 64×64×32
  Colormap: RAINBOW

✓ 3D visualization complete!
```

## Key Results

Typical output from the demo:

```text
Key Features Identified:
  • Anticline closure area:  0.996 km²
  • Structural relief:  14.3 ms
  • Normal fault with ~20ms throw
  • Reservoir reflection amplitude:  -0.099

Data Quality:
  • Signal-to-noise ratio:  6.27
  • RMS amplitude:  0.149
  • Dominant frequency: ~80 Hz
```

## Geological Interpretation

### Anticline Structure

The synthetic data includes a classic **4-way closure anticline**:

- Structural trap for hydrocarbons
- ~14ms of structural relief
- Closure area of ~1 km²
- Visible in all three orthogonal views

### Normal Fault

A **normal (extensional) fault** cuts through the structure:

- Vertical throw: ~20ms
- Orientation: North-South strike
- Location: Inline position ~800m
- Creates footwall and hanging wall compartments

### Reservoir Characteristics

The reservoir layer (30-50ms) shows:

- Moderate amplitude reflection
- Consistent across anticline crest
- Potential for hydrocarbon accumulation

## Technical Implementation

### Array Operations

Uses XDL's multi-dimensional array support:

```xdl
seismic = FLTARR(nx, ny, nz)  ; 3D array
min_val = MIN(seismic)         ; Works on entire volume
max_val = MAX(seismic)         ; No manual loops needed
mean_val = MEAN(seismic)       ; Direct computation
```

### VIZ3D Functions Used

```xdl
VIZ3D_INIT, WINDOW_SIZE=[w, h], TITLE='...'
VIZ3D_COLORMAP, 'RAINBOW'
VIZ3D_CAMERA, POSITION=[x,y,z], TARGET=[x,y,z], FOV=45.0
VIZ3D_VOLUME, data, DIMENSIONS=[nx,ny,nz]
VIZ3D_RENDER, /INTERACTIVE, TITLE='...'
```

### Performance

- **Generation**: ~2-3 seconds for 64³ volume
- **Analysis**: ~1 second for all attributes
- **Rendering**: Real-time (60fps) with Three.js

## Customization

### Change Volume Size

```xdl
; Larger volume (higher resolution)
nx = 128
ny = 128
nz = 64

; Smaller volume (faster execution)
nx = 32
ny = 32
nz = 16
```

### Modify Geological Features

```xdl
; Stronger anticline
anticline_relief = 20.0  ; instead of 15.0

; Larger fault offset
fault_offset = 30.0  ; instead of 20.0
```

### Different Colormap

```xdl
VIZ3D_COLORMAP, 'VIRIDIS'  ; or 'PLASMA', 'TURBO', etc.
```

## Applications

This demo illustrates workflows for:

1. **Seismic Interpretation**
   - Structural mapping
   - Fault identification
   - Horizon picking

2. **Petroleum Exploration**
   - Trap identification
   - Reservoir characterization
   - Prospect evaluation

3. **Educational Use**
   - Teaching seismic interpretation
   - Understanding 3D data visualization
   - Learning XDL programming

4. **Algorithm Development**
   - Testing attribute algorithms
   - Validating auto-picking methods
   - Benchmarking visualization

## Related Examples

- `medical_imaging_demo.xdl` - CT/MRI visualization
- `viz3d_demo1_gaussian.xdl` - Simple 3D Gaussian blob
- `viz3d_demo3_turbulence.xdl` - Turbulent flow visualization

## Next Steps

To extend the demo:

1. **Import Real Data**: Replace synthetic data with SEG-Y files
2. **Advanced Attributes**: Add coherence, curvature, etc.
3. **Well Integration**: Incorporate well log data
4. **Quantitative Analysis**: Reservoir property estimation
5. **Time-Lapse**: Compare 4D seismic surveys

## Technical Requirements

- XDL interpreter with VIZ3D support
- ~50MB RAM for 64³ volume
- WebGL-capable GPU (for Three.js backend)
- Modern web browser (embedded Tauri window)

## Troubleshooting

### Visualization doesn't appear

```bash
# Check backend selection
VIZ3D_BACKEND=threejs xdl examples/scientific/geophysical_demo.xdl

# Enable verbose logging
RUST_LOG=debug xdl examples/scientific/geophysical_demo.xdl
```

### Performance issues

- Reduce volume size (nx, ny, nz)
- Use lower opacity in visualization
- Close other GPU-intensive applications

### Colormap errors

Use valid colormap names:

- RAINBOW, VIRIDIS, PLASMA, INFERNO, TURBO, GRAYSCALE

## References

- [Array Functions](ARRAY_STATS_FUNCTIONS.md)
- [VIZ3D Documentation](../xdl-viz3d/README.md)
- [GPU Acceleration](GPU_ACCELERATION_PERFORMANCE_IMPACT.md)
- [3D Visualization Guide](QUICKSTART_VIZ3D.md)

## Author

Generated with XDL - Extended Data Language
