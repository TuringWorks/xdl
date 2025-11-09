# Medical Imaging Demo - User Guide

## Overview

The medical imaging demo (`examples/scientific/medical_imaging_demo.xdl`) demonstrates XDL's capabilities for medical image analysis and 3D visualization. It generates a synthetic CT head scan with realistic anatomical structures and provides interactive 3D visualization.

## Features

### Data Generation
- **3D CT Volume**: 64×64×32 voxels (configurable)
- **Anatomical Structures**:
  - Skull (cortical bone)
  - Brain tissue (gray and white matter)
  - Air-filled cavities
- **Hounsfield Units**: Physically realistic HU values
  - Air: -1000 HU
  - Soft tissue: 20-45 HU
  - Bone: 700-3000 HU

### Analysis Capabilities

1. **Tissue Segmentation**
   - Air (HU < -500)
   - Soft tissue (-100 ≤ HU < 200)
   - Bone (HU ≥ 200)

2. **Multi-Planar Reconstruction (MPR)**
   - Axial slices (transverse plane)
   - Coronal slices (frontal plane)
   - Sagittal slices (lateral plane)

3. **CT Windowing**
   - Brain window (center: 40 HU, width: 80 HU)
   - Bone window (center: 500 HU, width: 2000 HU)

4. **Volume Statistics**
   - Mean and standard deviation of HU values
   - Min/max HU range
   - Tissue-specific statistics

5. **Image Quality Assessment**
   - Signal-to-noise ratio (SNR)
   - Dynamic range
   - Voxel count and volume dimensions

6. **3D Volume Visualization** ✨
   - Interactive ray-casting volume renderer
   - Grayscale colormap (standard for medical imaging)
   - Real-time threshold adjustment
   - Multi-tissue visualization

## Running the Demo

### Basic Execution

```bash
# Run the complete demo with 3D visualization
xdl examples/scientific/medical_imaging_demo.xdl
```

### Environment Variables

```bash
# Use specific VIZ3D backend
VIZ3D_BACKEND=threejs xdl examples/scientific/medical_imaging_demo.xdl

# Available backends:
# - threejs (default): Three.js WebGL
# - webgpu: Native WebGPU renderer
# - browser: Browser-based WebGPU
```

## Workflow Steps

### Step 1: Generate Synthetic CT Data
Creates a 64×64×32 volume with:
- Air-filled background (-1000 HU)
- Skull shell (bone: 1400-1600 HU)
- Brain tissue (20-45 HU):
  - Gray matter (outer): ~40 HU
  - White matter (inner): ~30 HU

### Step 2: Tissue Segmentation
Segments volume into three tissue classes:
- **Air**: HU < -500 (~55% of volume)
- **Soft tissue**: -100 ≤ HU < 200 (~33% of volume)
- **Bone**: HU ≥ 200 (~12% of volume)

### Step 3-5: Multi-Planar Reconstruction
Extracts representative slices in three orthogonal planes:
- **Axial**: At z = 16 (mid-head level)
- **Coronal**: At y = 32 (mid-frontal)
- **Sagittal**: At x = 32 (mid-lateral)

### Step 6: CT Windowing Presets
Defines standard clinical window settings:

**Brain Window**:
- Center: 40 HU
- Width: 80 HU
- Range: 0-80 HU
- Purpose: Optimize gray/white matter contrast

**Bone Window**:
- Center: 500 HU
- Width: 2000 HU
- Range: -500 to 1500 HU
- Purpose: Visualize skull fractures and bone detail

### Step 7: Volume Statistics
Computes comprehensive statistics:
- Mean HU value (~-351 HU, air-dominated)
- Standard deviation (~837 HU)
- HU range: -1000 to +1600 HU
- Total voxels: 131,072 (64×64×32)

### Step 8: Tissue-Specific Statistics
Calculates mean HU for each tissue type:
- **Bone**: ~1500 HU
- **Soft tissue**: ~38 HU (brain)
- **Air**: -1000 HU (exact)

### Step 9: Image Quality Metrics
Assesses scan quality:
- Signal-to-noise ratio (SNR)
- Dynamic range: ~2600 HU
- Voxel count and dimensions

### Step 10: 3D Volume Visualization ✨
**Interactive 3D rendering** of the CT head scan:
- **Window**: 1280×720 HD display
- **Colormap**: Grayscale (medical imaging standard)
- **Camera**: Positioned for optimal head viewing
- **Volume**: Full 64×64×32 CT scan
- **Features visible**: Skull, brain tissue, anatomical structure

## 3D Visualization Controls

### Mouse Controls
- **Left Click + Drag**: Rotate camera around head
- **Mouse Wheel**: Zoom in/out
- **Right Click + Drag**: Pan camera

### GUI Controls
The viewer includes sliders for:

**Threshold Slider** (key control for medical imaging):
- **Minimum (-1000 HU)**: Shows everything (air + tissue + bone)
- **-500 HU**: Remove air, show tissue + bone
- **200 HU**: Show only bone (skull)
- **500 HU**: Show dense bone only

**Opacity Slider**:
- Adjust transparency of entire volume
- Lower values: See through structures
- Higher values: Solid rendering

**Usage Tips**:
1. Start with threshold at minimum to see full volume
2. Increase threshold to ~-500 to remove air and see head outline
3. Increase to ~200 to visualize skull structure
4. Adjust opacity to see internal/external features

### Keyboard Shortcuts
- **ESC**: Close viewer
- **R**: Reset camera to default position
- **Space**: Pause/resume auto-rotation (if enabled)

### Clinical View Presets

Use threshold settings to simulate clinical views:

**Soft Tissue View** (threshold: -500 HU):
- Shows brain tissue and skull
- Removes air background
- Good for overall anatomy

**Bone View** (threshold: 200 HU):
- Shows skull only
- Useful for fracture detection
- Visualize bone structure

**Dense Bone View** (threshold: 500 HU):
- Shows only cortical bone
- Highest density structures
- Clear skull outline

## Output Example

```
> Step 10: Launching 3D visualization...

VIZ3D: Initialized (1280x720)
  Volume loaded:  64  x  64  x  32
  HU range: [ -1000 ,  1600 ]

  Rendering 3D CT volume...
  - Dark: Air (-1000 HU)
  - Gray: Soft tissue (20-45 HU)
  - Bright: Bone (700-3000 HU)
  - Structures: Skull and brain tissue visible

  Adjust threshold slider to:
    • Show only bone (threshold > 200 HU)
    • Show soft tissue (threshold > -500 HU)
    • Include air (threshold at minimum)

🚀 Launching Three.js volume visualization...
Controls:
  - Left mouse: Rotate camera
  - Mouse wheel: Zoom in/out
  - GUI sliders: Adjust threshold and opacity

✓ Three.js volume visualization launched
  Volume: 64×64×32
  Colormap: GRAYSCALE

✓ 3D visualization complete!
```

## Key Results

Typical output from the demo:

```
Key Findings:
  • Volume size:  64 x 64 x 32
  • Mean HU:  -351 HU
  • Bone coverage:  12.2%
  • Soft tissue coverage:  33.1%
  • Air coverage:  54.7%
  • SNR:  0.048

Tissue-Specific Mean HU:
  • Bone:  1500 HU (n=16044)
  • Soft tissue:  38 HU (n=43324)
  • Air:  -1000 HU (n=71704)
```

## Medical Interpretation

### Anatomical Features

**Skull Structure**:
- Ellipsoidal shell representing cranium
- Cortical bone with realistic HU values (1400-1600)
- Shell thickness: ~3 voxels (realistic for skull)

**Brain Tissue**:
- Gray matter (outer): ~40 HU (typical)
- White matter (inner): ~30 HU (typical)
- Realistic tissue differentiation
- Spherical geometry inside skull

**Air Spaces**:
- Surrounding volume: -1000 HU (air density)
- Represents extracranial space
- Useful for windowing demonstration

### Clinical Applications

This demo simulates workflows for:

1. **Trauma Imaging**
   - Skull fracture detection (bone window)
   - Hemorrhage identification (brain window)
   - 3D reconstruction for surgical planning

2. **Neurological Assessment**
   - Gray/white matter differentiation
   - Volume measurements
   - Tissue density analysis

3. **Surgical Planning**
   - 3D visualization of anatomy
   - Measurement tools
   - Multi-planar reconstruction

4. **Education and Training**
   - Teaching CT interpretation
   - Understanding Hounsfield units
   - Windowing technique practice

## Technical Implementation

### Array Operations
Uses XDL's multi-dimensional array support:
```xdl
ct_volume = FLTARR(nx, ny, nz)  ; 3D array
min_hu = MIN(ct_volume)         ; Works on entire volume
max_hu = MAX(ct_volume)         ; No manual loops needed
mean_hu = MEAN(ct_volume)       ; Direct computation
stddev_hu = STDDEV(ct_volume)   ; Statistical function
```

### VIZ3D Functions Used
```xdl
VIZ3D_INIT, WINDOW_SIZE=[w, h], TITLE='...'
VIZ3D_COLORMAP, 'GRAYSCALE'
VIZ3D_CAMERA, POSITION=[x,y,z], TARGET=[x,y,z], FOV=45.0
VIZ3D_VOLUME, ct_volume, DIMENSIONS=[nx,ny,nz]
VIZ3D_RENDER, /INTERACTIVE, TITLE='...'
```

### Performance
- **Generation**: ~1-2 seconds for 64³ volume
- **Segmentation**: ~0.5 seconds
- **Rendering**: Real-time (60fps) with Three.js
- **Memory**: ~50MB for 64×64×32 float array

## Customization

### Change Volume Size
```xdl
; Higher resolution (more detail)
nx = 128
ny = 128
nz = 64

; Lower resolution (faster)
nx = 32
ny = 32
nz = 16
```

### Modify Anatomical Features
```xdl
; Larger skull
skull_outer_radius = 30.0  ; instead of 28.0

; Thicker skull
skull_inner_radius = 23.0  ; instead of 25.0

; Different tissue densities
gray_matter = 45.0   ; instead of 40.0
white_matter = 35.0  ; instead of 30.0
```

### Different Colormap
```xdl
VIZ3D_COLORMAP, 'VIRIDIS'  ; Color instead of grayscale
VIZ3D_COLORMAP, 'TURBO'    ; Rainbow-like medical colormap
```

## Applications

This demo illustrates workflows for:

1. **Clinical Radiology**
   - CT scan interpretation
   - Multi-planar reconstruction
   - 3D visualization

2. **Medical Education**
   - Teaching CT physics
   - Understanding Hounsfield units
   - Practicing windowing techniques

3. **Research**
   - Image processing algorithm development
   - Segmentation validation
   - Quantitative analysis

4. **Surgical Planning**
   - 3D anatomy visualization
   - Measurement and planning
   - Patient-specific modeling

## Related Examples

- `geophysical_demo.xdl` - Seismic 3D visualization
- `viz3d_demo1_gaussian.xdl` - Simple 3D Gaussian
- `viz3d_showcase.xdl` - Multiple 3D demos

## Next Steps

To extend the demo:

1. **Import DICOM Data**: Load real CT scans
2. **Advanced Segmentation**: Deep learning-based tissue classification
3. **Image Registration**: Align multiple scans
4. **Dose Planning**: Radiation therapy planning
5. **Surface Rendering**: Extract and render skull surface
6. **Measurement Tools**: Distance, volume, density measurements

## Technical Requirements

- XDL interpreter with VIZ3D support
- ~50MB RAM for 64³ volume
- WebGL-capable GPU
- Modern web browser (embedded Tauri window)

## Troubleshooting

### Visualization doesn't appear
```bash
# Verify backend
VIZ3D_BACKEND=threejs xdl examples/scientific/medical_imaging_demo.xdl

# Check logs
RUST_LOG=debug xdl examples/scientific/medical_imaging_demo.xdl
```

### Poor visualization quality
- Increase volume size (nx, ny, nz)
- Adjust threshold to focus on structures of interest
- Use appropriate windowing (brain vs bone)

### Performance issues
- Reduce volume size
- Lower opacity setting
- Close other GPU-intensive applications

## Clinical Context

### Hounsfield Scale Reference
- **Air**: -1000 HU
- **Lung**: -500 HU
- **Fat**: -100 to -50 HU
- **Water**: 0 HU
- **Soft tissue**: 20-70 HU
- **Blood**: 30-45 HU
- **Muscle**: 10-40 HU
- **Gray matter**: 37-45 HU
- **White matter**: 20-30 HU
- **Bone**: 700-3000 HU

### Standard CT Windows
- **Brain**: C=40, W=80
- **Subdural**: C=75, W=215
- **Stroke**: C=40, W=40
- **Bone**: C=500, W=2000
- **Soft tissue**: C=50, W=350
- **Lung**: C=-600, W=1500

## References

- [Array Functions](ARRAY_STATS_FUNCTIONS.md)
- [VIZ3D Documentation](../xdl-viz3d/README.md)
- [Geophysical Demo Guide](GEOPHYSICAL_DEMO_GUIDE.md)
- [GPU Acceleration](GPU_ACCELERATION_PERFORMANCE_IMPACT.md)

## Author

Generated with XDL - Extended Data Language
