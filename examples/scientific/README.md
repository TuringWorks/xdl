# Scientific Visualization Workflows for XDL

This directory contains comprehensive scientific visualization workflows demonstrating XDL's capabilities for professional data analysis and visualization in various scientific domains.

## Overview

These workflows showcase production-ready visualization pipelines for:
- Medical imaging (CT/MRI)
- Fluid dynamics and CFD
- Molecular/quantum chemistry
- Geophysical/seismic data
- Data format conversion
- Volume comparison and analysis

All workflows use XDL's native syntax (compatible with IDL/GDL) and leverage both 2D plotting and 3D volume rendering capabilities.

## Workflows

### 1. Medical Imaging (`medical_imaging_demo.xdl`)

**Purpose**: CT/MRI data visualization with clinical window presets

**Features**:
- Synthetic CT volume generation with Hounsfield Units (HU)
- Anatomical structure simulation (skull, brain tissue)
- Tissue segmentation (air, soft tissue, bone)
- Multiple window/level presets (brain, bone, soft tissue)
- Multi-planar reconstruction (axial, coronal, sagittal)
- 3D volume rendering
- Histogram analysis

**Usage**:
```bash
xdl medical_imaging_demo.xdl
```

**Key Outputs**:
- 2D slice visualizations in three anatomical planes
- Tissue segmentation masks
- 3D volume renders with tissue-specific colormaps
- Intensity histograms with peak detection
- Statistical summaries by tissue type

**Clinical Applications**:
- Treatment planning
- Surgical navigation
- Diagnostic imaging
- Radiation therapy planning

---

### 2. Fluid Dynamics (`fluid_dynamics_demo.xdl`)

**Purpose**: CFD visualization and flow field analysis

**Features**:
- Taylor-Green vortex analytical solution
- Vorticity field computation (curl of velocity)
- Velocity vector field visualization
- Streamline integration and tracing
- Flow quantities (kinetic energy, enstrophy)
- Q-criterion for vortex identification
- Time evolution tracking

**Usage**:
```bash
xdl fluid_dynamics_demo.xdl
```

**Key Outputs**:
- 2D vorticity contour plots
- Vector field overlays
- Streamline visualizations
- 3D volume rendering of vorticity structures
- Time-series energy decay analysis
- Divergence verification (incompressibility check)

**Research Applications**:
- Turbulence analysis
- Aerodynamic simulations
- Mixing and transport studies
- Vortex dynamics research

---

### 3. Molecular Structure (`molecular_structure_demo.xdl`)

**Purpose**: Quantum chemistry and molecular orbital visualization

**Features**:
- Hydrogen atomic orbitals (1s, 2p, 3d)
- Electron density distributions
- Molecular orbital formation (H₂ molecule)
- Bonding/antibonding orbitals
- Radial distribution functions
- 2D orbital cross-sections
- Energy level diagrams

**Usage**:
```bash
xdl molecular_structure_demo.xdl
```

**Key Outputs**:
- 3D orbital isosurfaces
- Electron probability densities
- Molecular bonding visualization
- Radial distribution plots
- Phase information (orbital lobes)

**Research Applications**:
- Quantum chemistry education
- Molecular modeling
- Chemical bonding analysis
- HOMO-LUMO analysis

---

### 4. Geophysical Data (`geophysical_demo.xdl`)

**Purpose**: Seismic data interpretation and structural geology

**Features**:
- Synthetic 3D seismic cube generation
- Geological layer modeling with structure
- Anticline (dome) feature creation
- Normal fault with vertical offset
- Inline/crossline section extraction
- Time slice visualization
- Automated fault detection
- Horizon auto-picking and tracking

**Usage**:
```bash
xdl geophysical_demo.xdl
```

**Key Outputs**:
- Seismic section displays (inline, crossline)
- Time slice amplitude maps
- Fault probability maps
- Horizon surfaces
- 3D seismic volume rendering
- Structural interpretation reports

**Industry Applications**:
- Oil & gas exploration
- Reservoir characterization
- Structural geology mapping
- Geohazard assessment

---

### 5. Data Loading Utilities (`data_loading_utils.xdl`)

**Purpose**: Scientific data format I/O and conversion

**Features**:
- Custom binary volume format (XDLV)
- CSV/ASCII table data I/O
- Format conversion utilities
- Metadata handling
- Error checking and validation
- Performance benchmarking

**Usage**:
```bash
xdl data_loading_utils.xdl
```

**Supported Formats**:
- **XDLV**: Custom binary with 512-byte header
  - Magic number: 'XDLV'
  - Dimensions, data type, version info
  - Raw float32 data
- **CSV**: Comma-separated values with headers
- Extensible to NetCDF, HDF5, DICOM, SEG-Y

**Key Functions**:
- `WRITE_VOLUME_BINARY` - Save 3D volume to binary
- `READ_VOLUME_BINARY` - Load 3D volume from binary
- `WRITE_CSV` - Export 2D table to CSV
- `READ_CSV` - Import CSV with headers

---

### 6. Comparison Tool (`comparison_tool_demo.xdl`)

**Purpose**: Quantitative volume comparison and change detection

**Features**:
- Side-by-side visualization
- Multiple difference metrics
- Statistical comparison
- Regional analysis
- Change detection thresholds
- Comprehensive reporting

**Usage**:
```bash
xdl comparison_tool_demo.xdl
```

**Metrics Computed**:
- **MSE**: Mean Squared Error
- **RMSE**: Root Mean Squared Error
- **PSNR**: Peak Signal-to-Noise Ratio
- **SSIM**: Structural Similarity Index
- **NCC**: Normalized Cross-Correlation

**Key Outputs**:
- Side-by-side slice comparisons
- Difference maps (absolute, relative, signed)
- Histogram overlays
- 3D volume comparison
- Statistical summary reports

**Applications**:
- Medical: Pre/post treatment comparison
- Quality control: Simulation validation
- Time-series: Evolution tracking
- Algorithm testing: Method benchmarking

---

## Running the Workflows

### Prerequisites

1. **XDL Interpreter** must be installed and in your PATH
2. **Visualization backends**:
   - Three.js (Tauri) for 3D volume rendering
   - Native plotting for 2D graphics

### Execution

**Single workflow**:
```bash
cd examples/scientific/
xdl medical_imaging_demo.xdl
```

**Batch execution**:
```bash
# Run all workflows sequentially
for demo in *.xdl; do
    echo "Running $demo..."
    xdl "$demo"
done
```

### Environment Variables

Configure visualization backend:
```bash
# Use Three.js (default)
export VIZ3D_BACKEND=threejs

# Use native WebGPU
export VIZ3D_BACKEND=webgpu

# Use browser-based
export VIZ3D_BACKEND=browser
```

---

## Technical Details

### Syntax Compatibility

All workflows use **IDL/GDL-compatible syntax**:

- `FOR...DO...ENDFOR` loops
- `IF...THEN...ELSE...ENDIF` conditionals
- `FUNCTION` and `PRO` procedure definitions
- Array indexing with `[]`
- Keywords with `KEYWORD=value` or `/FLAG`
- Comments with `;`

### Data Structures

**3D Volumes**:
```idl
; Create 3D array
volume = FLTARR(nx, ny, nz)

; Access elements
value = volume[i, j, k]

; Extract slice
slice = volume[*, *, k]
```

**Metadata**:
```idl
; Structure definition
metadata = {description: 'CT scan', $
            units: 'HU', $
            spacing: [1.0, 1.0, 2.0]}
```

### Visualization Pipeline

1. **Data Generation/Loading**
2. **Processing** (filtering, segmentation, analysis)
3. **2D Visualization** (plots, contours, images)
4. **3D Visualization** (volume rendering)
5. **Output** (statistics, reports, files)

---

## Extending the Workflows

### Adding New Workflows

1. Create `.xdl` file in `examples/scientific/`
2. Follow existing structure:
   - Header with description
   - Numbered steps with PRINT statements
   - Progressive complexity
   - Summary at end

3. Use standard functions:
   - `FLTARR`, `FINDGEN` for arrays
   - `WINDOW`, `PLOT`, `CONTOUR` for 2D
   - `VIZ3D_*` procedures for 3D

### Custom Data Formats

Extend `data_loading_utils.xdl`:

```idl
FUNCTION READ_MYFORMAT, filename
    ; Your parsing logic here
    OPENR, lun, filename, /GET_LUN
    ; Read header
    ; Read data
    FREE_LUN, lun
    RETURN, data
END
```

### Integration with External Tools

**Python Interop** (via files):
```python
# Generate data in Python
import numpy as np
data = np.random.randn(64, 64, 32)
data.astype('float32').tofile('volume.raw')
```

```idl
; Load in XDL
volume = FLTARR(64, 64, 32)
OPENR, lun, 'volume.raw', /GET_LUN
READU, lun, volume
FREE_LUN, lun
```

---

## Performance Considerations

### Volume Size Guidelines

| Application | Recommended Size | Notes |
|-------------|------------------|-------|
| Interactive 3D | 128³ - 256³ | Real-time rendering |
| Batch processing | 512³ - 1024³ | May require downsampling |
| 2D slices | Unlimited | Fast visualization |

### Optimization Tips

1. **Use subsampling** for large datasets:
   ```idl
   volume_sub = volume[0:*:2, 0:*:2, 0:*:2]
   ```

2. **Extract regions of interest**:
   ```idl
   roi = volume[x0:x1, y0:y1, z0:z1]
   ```

3. **Progressive loading** for large files

4. **Parallel processing** with FOR loops (when available)

---

## Troubleshooting

### Common Issues

**Issue**: "Unknown procedure: VIZ3D_INIT"
- **Solution**: Ensure xdl-stdlib is properly installed

**Issue**: 3D window doesn't open
- **Solution**: Check VIZ3D_BACKEND environment variable

**Issue**: Out of memory
- **Solution**: Reduce volume dimensions or use subsampling

**Issue**: Slow rendering
- **Solution**: Lower grid resolution or use native backend

### Debug Mode

Enable verbose output:
```bash
export XDL_DEBUG=1
xdl your_script.xdl
```

---

## Citation

If you use these workflows in research, please cite:

```
XDL Scientific Visualization Workflows
https://github.com/yourusername/xdl
```

---

## License

MIT License - see LICENSE file for details

---

## Contributing

Contributions welcome! Please:
1. Follow existing code style
2. Add comments and documentation
3. Include example outputs
4. Test with multiple datasets

---

## Support

- **Issues**: GitHub issue tracker
- **Documentation**: `docs/` directory
- **Examples**: `examples/` directory

---

## Roadmap

Future enhancements:
- [ ] NetCDF/HDF5 support
- [ ] DICOM medical imaging format
- [ ] SEG-Y seismic format
- [ ] Interactive GUI controls
- [ ] Real-time data streaming
- [ ] GPU-accelerated processing
- [ ] Machine learning integration
- [ ] Cloud data access (S3, etc.)

---

## Acknowledgments

- Taylor-Green vortex: Classical fluid dynamics benchmark
- Hydrogen orbitals: Quantum mechanics fundamentals
- Medical imaging: Clinical window standards
- Seismic interpretation: Industry best practices

---

**Last Updated**: 2025-10-25
**Version**: 1.0.0
**Author**: XDL Development Team
