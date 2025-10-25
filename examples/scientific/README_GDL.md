# Scientific Visualization Demos for GDL

This directory contains scientific visualization demos designed to work with GNU Data Language (GDL).

## Prerequisites

### Install GDL on macOS

```bash
# Install XQuartz (X11 for macOS)
brew install --cask xquartz

# Install GDL with graphics support
brew install gnudatalanguage

# Verify installation
gdl --version
```

### Setup X11 Display

```bash
# Start XQuartz
open -a XQuartz

# Set display environment variable
export DISPLAY=:0

# Test X11 is working
xeyes  # Should show animated eyes
```

## Running the Demos

### 1. Test Graphics Capabilities First

Before running the full demos, test that GDL graphics work:

```bash
cd examples/scientific
gdl test_graphics.xdl
```

This will create 7 test windows to verify:
- Line plots (PLOT)
- Contour plots (CONTOUR)
- Surface plots (SURFACE)
- Image display (TV/TVSCL)
- Multi-panel layouts
- Color tables

### 2. Run Individual Demos

Each demo can be run directly:

```bash
# Fluid dynamics simulation
gdl fluid_dynamics_demo.xdl

# Geophysical seismic data
gdl geophysical_demo.xdl

# Medical CT/MRI imaging
gdl medical_imaging_demo.xdl

# Molecular orbital visualization
gdl molecular_structure_demo.xdl

# Volume comparison tool
gdl comparison_tool_demo.xdl

# Data I/O utilities
gdl data_loading_utils.xdl
```

## Demo Descriptions

### fluid_dynamics_demo.xdl
- Taylor-Green vortex simulation
- Vorticity field computation
- Streamline tracing
- Vector field visualization
- Q-criterion for vortex identification

**Graphics**: 5 windows showing vorticity, velocity vectors, streamlines, 3D surface, Q-criterion

### geophysical_demo.xdl
- Synthetic seismic data generation
- Geological layer modeling
- Fault detection
- Horizon tracking
- Cross-section extraction

**Graphics**: Multiple windows for inline/crossline sections, time slices, fault maps

### medical_imaging_demo.xdl
- Synthetic CT data generation
- Tissue segmentation by Hounsfield Units
- Multiple windowing presets (brain, bone, soft tissue)
- Multi-planar reconstruction (axial, coronal, sagittal)

**Graphics**: CT slice views, histograms, 3D renderings

### molecular_structure_demo.xdl
- Hydrogen atomic orbitals (1s, 2p, 3d)
- Electron density distributions
- H₂ molecular orbital formation
- Radial distribution functions

**Graphics**: 3D orbital surfaces, cross-sections, probability distributions

### comparison_tool_demo.xdl
- Side-by-side volume comparison
- Difference mapping (absolute, relative, signed)
- Statistical metrics (MSE, RMSE, PSNR, SSIM)
- Regional analysis

**Graphics**: Multi-panel comparisons, difference maps, histograms

### data_loading_utils.xdl
- Binary volume I/O (custom XDLV format)
- CSV data handling
- Format conversion utilities
- Error checking and validation

**Graphics**: Visualization of loaded data slices

## Known Limitations

### GDL vs IDL Differences

1. **VIZ3D Commands**: Not available in GDL
   - Replaced with SURFACE for 3D visualization
   - Full volume rendering requires external tools (ParaView, VisIt)

2. **VELOVECT**: May not be available
   - Replaced with manual ARROW plotting for vector fields

3. **FUNCTION Returns**: Some complex return types simplified
   - Procedures with output parameters used instead

4. **Graphics Performance**: GDL may be slower than IDL
   - Reduce grid sizes for faster rendering
   - Example: Change `nx=128` to `nx=64` for testing

## Customization

### Adjust Grid Sizes

For faster execution during testing, reduce array dimensions:

```idl
; In any demo, find lines like:
nx = 128   ; Change to 64 or 32
ny = 128
nz = 64    ; Change to 32
```

### Disable Graphics

To run without graphics (computation only):

```idl
; Add at the start of any script:
SET_PLOT, 'NULL'  ; Disable graphics output
```

### Save Plots to Files

Instead of displaying windows, save to files:

```idl
; Switch to PostScript output
SET_PLOT, 'PS'
DEVICE, FILENAME='output.ps', XSIZE=20, YSIZE=20, /ENCAPSULATED

; Your plotting commands here
PLOT, x, y

; Close and convert to PDF
DEVICE, /CLOSE
!P.FONT = -1
SET_PLOT, 'X'

; Convert PS to PDF using system command
SPAWN, 'ps2pdf output.ps output.pdf'
```

## Troubleshooting

### "Can't open display"

```bash
# Ensure XQuartz is running
open -a XQuartz

# Check DISPLAY variable
echo $DISPLAY  # Should show :0 or similar

# Reset if needed
export DISPLAY=:0
```

### "Undefined procedure/function"

Some GDL installations may be missing certain functions. Check:

```bash
# List available procedures
gdl -e "HELP, /ROUTINES"

# Test specific function
gdl -e "PRINT, ROUTINE_INFO('CONTOUR', /SOURCE)"
```

### Graphics Windows Not Appearing

```bash
# Test X11 connection
xdpyinfo

# Try forcing window display
gdl -e "WINDOW, 0 & PLOT, FINDGEN(10)"
```

### Slow Performance

1. Reduce array sizes (see Customization above)
2. Use fewer contour levels: `NLEVELS=10` instead of `NLEVELS=20`
3. Skip 3D visualizations temporarily
4. Disable shading: Remove `SHADES=` keyword from SURFACE

## Advanced Usage

### Batch Processing

Run all demos in sequence:

```bash
#!/bin/bash
export DISPLAY=:0

for demo in *_demo.xdl; do
    echo "Running $demo..."
    gdl "$demo" 2>&1 | tee "${demo%.xdl}.log"
done
```

### Extract Data for External Visualization

The demos can export data for use with Python/ParaView:

```idl
; In GDL, save array to binary file
OPENW, lun, 'vorticity.dat', /GET_LUN
WRITEU, lun, omega_mag
FREE_LUN, lun

; Write dimensions to text file
OPENW, lun, 'vorticity.txt', /GET_LUN
PRINTF, lun, nx, ny, nz
FREE_LUN, lun
```

Then visualize with Python:

```python
import numpy as np
import matplotlib.pyplot as plt

# Read dimensions
with open('vorticity.txt') as f:
    nx, ny, nz = map(int, f.read().split())

# Read binary data
data = np.fromfile('vorticity.dat', dtype=np.float32)
data = data.reshape((nx, ny, nz))

# Visualize
plt.imshow(data[:, :, nz//2])
plt.colorbar()
plt.show()
```

## Further Resources

- GDL Documentation: https://gnudatalanguage.github.io/
- IDL to GDL Migration: http://gnudatalanguage.sourceforge.net/
- XQuartz: https://www.xquartz.org/

## Support

For issues specific to these demos, check:
1. Syntax errors in the XDL files
2. GDL version compatibility (test with `gdl --version`)
3. X11/XQuartz setup on macOS
4. Available memory for large arrays
