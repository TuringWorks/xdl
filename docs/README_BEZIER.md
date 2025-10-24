# Bezier Surface Example - Fixed for XDL

## Changes Made

The original `05_complex_bezier.m` file had several features that are not yet supported in XDL:

### Original Issues:
1. **Complex numbers** (`-1-1i`, etc.) - Not supported
2. **Nested function definitions** - Not supported  
3. **MATLAB-specific functions** - `meshgrid`, `subplot`, `figure`, `surf`, `nchoosek` not fully implemented
4. **2D array literal syntax** with `$` continuation - Not supported
5. **Multi-dimensional array indexing** - Limited support

### Solution:
Rewrote the script as `05_bezier_surface.xdl` using XDL-compatible features:

1. **Replaced complex numbers** with real-valued control points
2. **Removed nested functions** - expanded Bernstein polynomial calculations inline
3. **Used 1D arrays** instead of 2D with manual index calculation (`idx = ui * steps + vi`)
4. **Added `reform()`** to reshape 1D array to 2D for surface plotting
5. **Simplified to single surface plot** instead of multiple subplots

### Result:
A working Bezier surface generator that:
- Computes cubic Bernstein basis functions
- Creates a smooth parametric surface from 4x4 control points
- Generates a 3D surface plot
- Provides statistical analysis of the surface

## Usage

```bash
./target/release/xdl examples/matlab/05_bezier_surface.xdl
```

Or from the GUI:
```bash
./target/release/xdl-gui
# Then File -> Open -> examples/matlab/05_bezier_surface.xdl
```

## Output
- Console output with surface statistics
- `xdl_surface.png` - 3D visualization of the Bezier surface

## Technical Notes

The script demonstrates:
- Parametric surface generation
- Cubic Bernstein polynomial blending
- Array manipulation and indexing in XDL
- Surface plotting capabilities
- Statistical analysis functions (mean, stddev, min, max)
