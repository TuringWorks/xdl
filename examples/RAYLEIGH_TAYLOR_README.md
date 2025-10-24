# Rayleigh-Taylor Instability Visualization Demo

## Overview

This demonstration showcases XDL's advanced scientific visualization capabilities through a simulation of the **Rayleigh-Taylor instability** - a classic fluid dynamics phenomenon where a dense fluid sits atop a lighter fluid, creating characteristic mushroom-shaped structures as they mix.

## What is Rayleigh-Taylor Instability?

The Rayleigh-Taylor (RT) instability occurs when:
- A heavier fluid is positioned above a lighter fluid
- Small perturbations at the interface grow exponentially
- The heavy fluid "falls" through the light fluid in finger-like plumes
- Beautiful, turbulent mixing patterns emerge

This phenomenon is observed in:
- **Supernovae** - when stellar material is accelerated outward
- **Inertial Confinement Fusion** - affecting fuel compression
- **Oceanography** - thermocline mixing
- **Atmospheric science** - cloud formation
- **Industrial processes** - mixing and separation

## Features Demonstrated

This demo showcases:

### 1. **Multiple Perceptually Uniform Colormaps**
- **Viridis** - Default, excellent for scientific data
- **Plasma** - Purple to yellow gradient
- **Turbo** - Improved rainbow colormap
- **Inferno** - Black through orange to white

### 2. **Density Field Visualization**
- High-resolution (200x200 grid) fluid simulation
- Real-time evolution of mixing interface
- Color-coded density values

### 3. **Vector Field Visualization**
- Velocity field quiver plots (arrows)
- Color-coded by velocity magnitude
- Subsampled for clarity

### 4. **Multi-Frame Time Series**
- 6 timesteps: 0, 50, 100, 200, 400, 800 steps
- Shows instability growth over time
- Captures transition from smooth interface to turbulent mixing

### 5. **Side-by-Side Comparisons**
- 4-panel colormap comparison images
- Demonstrates advantages of perceptually uniform colormaps
- Shows same data with different color schemes

## Generated Visualizations

The demo generates 30 images total:

### Per Timestep (6 timesteps × 5 images = 30 images):
1. **`rt_viridis_XXXX.png`** - Density field with Viridis colormap
2. **`rt_plasma_XXXX.png`** - Density field with Plasma colormap
3. **`rt_turbo_XXXX.png`** - Density field with Turbo colormap
4. **`rt_velocity_XXXX.png`** - Velocity field quiver plot
5. **`rt_comparison_XXXX.png`** - 4-panel colormap comparison

Plus:
- **`rt_initial_comparison.png`** - Initial state comparison

## Running the Demo

### Prerequisites

Ensure you have Rust and the XDL dependencies installed:

```bash
cd /path/to/xdl
cargo build --release
```

### Execute the Visualization

```bash
# From the xdl root directory
cargo run --release --example rayleigh_taylor_viz
```

### Expected Output

```
Rayleigh-Taylor Instability Visualization Demo
================================================

Initializing simulation (200x200 grid)...
Initial density stats: (1.0, 2.0, 1.5)

Rendering initial state with multiple colormaps...
  ✓ Saved: rt_initial_comparison.png

Simulating 50 steps...
Density stats: (1.0123, 1.9877, 1.5)
Rendering frame 50 visualizations:
  ✓ Saved: rt_viridis_0050.png
  ✓ Saved: rt_plasma_0050.png
  ✓ Saved: rt_turbo_0050.png
  ✓ Saved: rt_velocity_0050.png
  ✓ Saved: rt_comparison_0050.png

[... continues for each timestep ...]

✅ Visualization complete!

Generated 30 images showcasing:
  • Density field evolution with Viridis, Plasma, Turbo, and Inferno colormaps
  • Velocity field quiver plots
  • Side-by-side colormap comparisons

The Rayleigh-Taylor instability demonstrates:
  • Heavy fluid (top) falling through light fluid (bottom)
  • Mushroom-shaped plumes forming
  • Complex fluid mixing dynamics
  • Perceptually uniform colormap advantages
```

## Simulation Details

### Physics
- **Grid**: 200 × 200 cells
- **Time step**: 0.01
- **Gravity**: 0.1
- **Viscosity**: 0.001
- **Atwood number**: 0.5 (density contrast parameter)
- **Initial perturbation**: 4-wavelength sinusoidal disturbance

### Numerical Method
- **Advection**: Semi-Lagrangian scheme with bilinear interpolation
- **Diffusion**: Explicit finite difference (Laplacian)
- **Boundary conditions**: No-slip walls
- **Buoyancy**: Density-driven force proportional to local density gradient

### Simplifications
This is a *demonstrative* simulation, not a full Navier-Stokes solver:
- Simplified pressure solve (omitted for demo)
- No vorticity confinement
- Basic viscous diffusion
- Sufficient to show RT instability growth and mixing

## Analyzing the Results

### What to Look For

1. **Initial State (t=0)**
   - Clear interface between heavy (top) and light (bottom) fluids
   - Small sinusoidal perturbation visible

2. **Early Growth (t=50-100)**
   - Perturbations begin to grow
   - Fingers starting to form
   - Interface becomes wavy

3. **Linear Growth (t=100-200)**
   - Clear finger/spike formation
   - Heavy fluid fingers penetrating downward
   - Light fluid bubbles rising upward

4. **Nonlinear Regime (t=200-400)**
   - Mushroom-shaped structures
   - Secondary instabilities
   - Vortex roll-up at plume heads

5. **Turbulent Mixing (t=400-800)**
   - Complex, chaotic patterns
   - Extensive mixing region
   - Multiple length scales present

### Colormap Comparison

**Viridis vs. Rainbow/Jet:**
- Viridis maintains perceptual uniformity
- Features remain visible across full range
- No artificial "bands" or "hot spots"
- Better for quantitative analysis

**Plasma:**
- Excellent contrast
- Good for highlighting fine structures
- Purple-yellow gradient intuitive for hot/cold

**Turbo:**
- Improved rainbow
- More perceptually uniform than classic jet
- Good for presentations

**Inferno:**
- Black-body radiation inspired
- Excellent for high dynamic range
- Good for printing

## Technical Implementation

### File Structure
```
examples/
├── rayleigh_taylor_demo.rs      # Simulation engine (241 lines)
├── rayleigh_taylor_viz.rs       # Visualization driver (309 lines)
└── RAYLEIGH_TAYLOR_README.md    # This file
```

### Key Components

**Simulation (`rayleigh_taylor_demo.rs`):**
- `RTSimulation` struct with density and velocity fields
- `step()` - Evolve one timestep
- `simulate(n)` - Run n timesteps
- `density_stats()` - Get min/max/average density

**Visualization (`rayleigh_taylor_viz.rs`):**
- `render_density_field()` - Render with any colormap
- `render_velocity_field()` - Quiver plot of velocities
- `render_comparison()` - 4-panel comparison image
- Colormap implementations: viridis, plasma, turbo, inferno

## Educational Value

This demo illustrates:

1. **Fluid Dynamics Fundamentals**
   - Instability growth
   - Buoyancy-driven flow
   - Turbulent mixing

2. **Scientific Visualization Best Practices**
   - Importance of colormap choice
   - Perceptual uniformity
   - Multi-modal representation (density + velocity)

3. **Computational Methods**
   - Finite difference methods
   - Semi-Lagrangian advection
   - Eulerian vs. Lagrangian frameworks

4. **XDL Capabilities**
   - High-quality scientific visualization
   - Flexible colormap system
   - Vector field rendering
   - Animation frame generation

## Extending the Demo

### Modifications to Try

1. **Different Initial Conditions:**
   ```rust
   // In RTSimulation::new(), modify:
   let wavelength = width as f64 / 6.0;  // More wavelengths
   let perturbation_amplitude = 10.0;     // Larger perturbation
   ```

2. **Different Parameters:**
   ```rust
   viscosity: 0.01,      // More viscous fluid
   gravity: 0.2,         // Stronger gravity
   atwood_number: 0.8,   // Greater density contrast
   ```

3. **Higher Resolution:**
   ```rust
   let mut sim = RTSimulation::new(400, 400);  // 4x more grid points
   ```

4. **More Timesteps:**
   ```rust
   let timesteps = [0, 25, 50, 100, 200, 400, 600, 800, 1000, 1200];
   ```

5. **Additional Visualizations:**
   - Add streamline plots
   - Add vorticity visualization
   - Create animated GIF
   - Export to HTML with slider control

## Performance

**On a typical modern laptop:**
- Initialization: < 0.1 seconds
- Per timestep: ~10ms (200×200 grid)
- Per visualization: ~0.5 seconds
- Total runtime: ~20-30 seconds

**Scaling:**
- Simulation: O(N²) per step
- Visualization: O(N²) per frame
- Memory: ~10 MB for 200×200 grid

## Scientific Accuracy

**What's Accurate:**
- Qualitative RT instability behavior
- Growth of perturbations
- Mushroom structure formation
- Velocity field patterns

**What's Simplified:**
- No incompressibility constraint (divergence-free velocity)
- No pressure solve
- Basic boundary conditions
- 2D instead of 3D

**For Research-Grade Simulations, Consider:**
- Full Navier-Stokes solver (e.g., using XDL's Python integration with PyFR, Basilisk, or ATHENA)
- Higher-order numerical schemes
- Adaptive mesh refinement
- 3D domains

## References

1. **Rayleigh, Lord** (1883). "Investigation of the character of the equilibrium of an incompressible heavy fluid of variable density"
2. **Taylor, G.I.** (1950). "The instability of liquid surfaces when accelerated"
3. **Chandrasekhar, S.** (1961). "Hydrodynamic and Hydromagnetic Stability"
4. **Sharp, D.H.** (1984). "An overview of Rayleigh-Taylor instability"

## Support

For questions or issues:
- Check XDL documentation: `docs/SCIENTIFIC_VISUALIZATION_GUIDE.md`
- Review examples: `examples/`
- Report bugs: XDL issue tracker

---

**Enjoy exploring the beautiful physics of fluid instabilities with XDL!** 🌊💫
