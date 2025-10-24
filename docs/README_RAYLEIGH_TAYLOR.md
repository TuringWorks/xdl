# Rayleigh-Taylor Instability Demo (XDL)

## Overview

This is a complete Rayleigh-Taylor instability simulation implemented in pure XDL, demonstrating advanced scientific visualization and fluid dynamics computation capabilities.

## What is Rayleigh-Taylor Instability?

The Rayleigh-Taylor instability occurs when a heavy fluid sits on top of a lighter fluid in a gravitational field. Small perturbations at the interface grow exponentially, creating characteristic mushroom-shaped structures as the fluids mix.

### Real-World Applications

- **Astrophysics**: Supernova explosions, stellar evolution
- **Geophysics**: Mantle convection, magma chamber dynamics
- **Engineering**: Combustion processes, inertial confinement fusion
- **Meteorology**: Atmospheric instabilities, cloud formation

## Running the Demo

### Quick Start

```bash
cd /Users/ravindraboddipalli/sources/xdl
./target/release/xdl-gui examples/rayleigh_taylor.xdl
```

Click "Execute" to run the simulation.

### Expected Runtime

- Grid size: 128×128
- Time steps: 6
- Runtime: ~30-60 seconds (depending on hardware)

## Output Files

The simulation generates 8 PNG images:

1. **rayleigh_taylor_t0.png** - Initial state (heavy on top, light below)
2. **rayleigh_taylor_t1.png** - Early development of instability
3. **rayleigh_taylor_t2.png** - Perturbations begin to grow
4. **rayleigh_taylor_t3.png** - Mushroom structures form
5. **rayleigh_taylor_t4.png** - Advanced mixing stage
6. **rayleigh_taylor_t5.png** - Complex turbulent patterns
7. **rayleigh_taylor_t6.png** - Fully developed instability
8. **rayleigh_taylor_velocity.png** - Final velocity field visualization

## Physics Implemented

### 1. Buoyancy Force
```idl
buoy = -g * (rho - rho_mean) / rho_mean * atwood
```
- Heavy fluid (ρ=1) accelerates downward
- Light fluid (ρ=0) rises upward
- Atwood number controls density contrast

### 2. Velocity Advection
```idl
v[idx] = v[idx] + buoy * dt
```
- Forward Euler time integration
- Gravitational acceleration drives flow

### 3. Semi-Lagrangian Advection
```idl
x_back = i - u[idx] * dt
y_back = j - v[idx] * dt
```
- Trace particles backward in time
- Bilinear interpolation for smooth fields
- Unconditionally stable (large time steps possible)

### 4. Viscous Damping
```idl
u[idx] = u[idx] * (1.0 - viscosity)
```
- Simple exponential decay
- Prevents numerical instabilities
- Models physical viscosity

## Parameters

### Physical Parameters
```idl
g = 9.8              ; Gravitational acceleration (m/s²)
atwood = 0.5         ; Atwood number (density contrast)
viscosity = 0.001    ; Kinematic viscosity (m²/s)
```

### Numerical Parameters
```idl
nx = 128             ; Grid resolution (x-direction)
ny = 128             ; Grid resolution (y-direction)
dt = 0.01            ; Time step (seconds)
n_steps = 6          ; Number of visualization snapshots
```

### Initial Perturbations
```idl
interface = 0.6 + 0.05 * (sin(8π x) + 0.5*sin(12π x + 1.5) + 0.3*cos(16π x - 0.5))
```
- Base interface at y=0.6
- 5% amplitude perturbations
- 3 wavelength modes for rich dynamics

## Numerical Methods

### Semi-Lagrangian Advection

**Advantages:**
- Unconditionally stable (no CFL restriction)
- Handles large time steps
- Preserves features well

**Implementation:**
1. Trace particle position backward: `x_back = x - u*dt`
2. Interpolate field value at backward position
3. Assign to current position

### Bilinear Interpolation

For smooth field reconstruction:
```idl
val0 = f[i0,j0]*(1-fx) + f[i1,j0]*fx
val1 = f[i0,j1]*(1-fx) + f[i1,j1]*fx
result = val0*(1-fy) + val1*fy
```

## Code Structure

```
1. Setup (lines 1-44)
   - Parameters
   - Grid initialization

2. Initial Conditions (lines 46-93)
   - Density field setup
   - Multi-mode perturbations
   - Initial visualization

3. Time Stepping Loop (lines 95-209)
   - Buoyancy force calculation
   - Velocity update
   - Viscous damping
   - Semi-Lagrangian advection
   - Visualization output

4. Velocity Field (lines 213-245)
   - Downsample for clarity
   - Quiver plot generation

5. Summary (lines 247-285)
   - Statistics
   - Applications
```

## Modifying the Simulation

### Increase Resolution
```idl
nx = 256
ny = 256
```
Higher resolution captures finer details but runs slower.

### Change Density Contrast
```idl
atwood = 0.8  ; Stronger instability
atwood = 0.2  ; Weaker instability
```

### Adjust Viscosity
```idl
viscosity = 0.01   ; More damping (stable, less detail)
viscosity = 0.0001 ; Less damping (turbulent, more detail)
```

### Different Perturbations
```idl
; Single mode
interface = 0.6 + 0.05 * sin(8.0 * 3.14159 * x)

; Random perturbations
interface = 0.6 + 0.05 * (randomu(seed) - 0.5)
```

### More Time Steps
```idl
n_steps = 12  ; Watch evolution longer
```

## XDL Features Demonstrated

### Array Operations
- Large 1D arrays (16,384 elements for 128×128 grid)
- Array indexing and manipulation
- `reform()` for reshaping

### Control Flow
- Nested `for` loops
- Conditional statements (`if`/`then`/`else`)
- Loop variables

### Mathematical Functions
- Trigonometric: `sin()`, `cos()`
- Array creation: `fltarr()`
- String operations: `strlen()`, `strmid()`

### Advanced Visualization
- `RENDER_COLORMAP` - Density field visualization
- `QUIVER` - Vector field arrows
- Multiple output files

### String Manipulation
```idl
; Dynamic filename generation
filename = 'rayleigh_taylor_t' + string(step) + '.png'

; Remove whitespace
for k=0, len-1 do begin
  char = strmid(filename, k, 1)
  if char ne ' ' then clean_name = clean_name + char
end
```

## Performance Tips

1. **Reduce grid size** for faster testing:
   ```idl
   nx = 64
   ny = 64
   ```

2. **Fewer time steps** for quick preview:
   ```idl
   n_steps = 3
   ```

3. **Increase stride** for velocity visualization:
   ```idl
   stride = 8  ; Fewer arrows, faster rendering
   ```

## Scientific Background

### Growth Rate

The linear growth rate of R-T instability is:
```
γ = sqrt(A * g * k)
```
where:
- A = Atwood number
- g = gravitational acceleration
- k = wavenumber of perturbation

### Atwood Number

```
A = (ρ₂ - ρ₁) / (ρ₂ + ρ₁)
```
- A = 0: No density difference (no instability)
- A = 1: Maximum contrast (fastest growth)

### Characteristic Structures

- **Spikes**: Heavy fluid fingers penetrating downward
- **Bubbles**: Light fluid rising upward
- **Mushroom caps**: Late-stage nonlinear structures
- **Kelvin-Helmholtz rolls**: Secondary instabilities at interfaces

## Comparison with Full Simulation

This is a **simplified demonstration** for educational purposes. A complete simulation would include:

- ✓ Buoyancy force (implemented)
- ✓ Advection (implemented)
- ✗ Pressure projection (not implemented)
- ✗ Incompressibility constraint (not enforced)
- ✗ Surface tension (not included)
- ✗ Multi-material interface tracking (simplified)

For research-grade simulations, see specialized codes like:
- FLASH (astrophysics)
- ATHENA (magnetohydrodynamics)
- Gerris (free-surface flows)

## Related Demos

- **advanced_viz_demo.xdl** - General visualization features
- **advanced_viz_simple.xdl** - Simpler visualization examples
- **rayleigh_taylor_demo.rs** - Rust implementation (more physics)

## Troubleshooting

### Issue: Images not generated
**Solution**: Check that RENDER_COLORMAP and QUIVER procedures are registered. Rebuild with:
```bash
cargo build --release
```

### Issue: Simulation too slow
**Solution**: Reduce grid size or number of time steps

### Issue: Results look unstable
**Solution**: Increase viscosity or decrease time step

### Issue: Not enough mixing
**Solution**: Increase Atwood number or run more time steps

## Further Reading

- **Wikipedia**: "Rayleigh-Taylor instability"
- **Books**:
  - "Turbulence" by P.A. Davidson
  - "Introduction to Hydrodynamic Instabilities" by G. Ricard
- **Papers**:
  - Sharp (1984) - "Overview of R-T instability"
  - Dimonte et al. (2004) - "Comparative study of R-T mixing"

## License

Part of the XDL project. See main repository for license information.

---

**Created**: January 2025
**XDL Version**: 0.1.0
**Visualization**: Viridis colormap (perceptually uniform)
**Status**: ✅ Tested and working
