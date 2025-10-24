# GIS Feature Setup Guide

## Overview

The GIS (Geographic Information System) features in XDL require the **PROJ** coordinate transformation library to be installed on your system. This is an optional feature that can be enabled when needed.

## Current Status

By default, GIS features are **disabled** to avoid build dependencies. The following features are available:

- ✅ **Always Available** (no extra dependencies):
  - Color mapping (viridis, plasma, turbo, inferno, terrain)
  - Terrain visualization (DEM, hillshade, 3D rendering)
  - Scientific visualization (vector fields, streamlines, volume rendering)
  - Export (PNG, SVG, HTML)

- 🗺️ **GIS Features** (requires `gis` feature flag + PROJ library):
  - Map projections (Mercator, Lambert, Albers, etc.)
  - Coordinate transformations
  - GeoJSON support
  - Coastline rendering
  - Map scatter plots

## Installing PROJ Library

### macOS (Homebrew)

```bash
brew install proj
```

### macOS (MacPorts)

```bash
sudo port install proj
```

### Linux (Ubuntu/Debian)

```bash
sudo apt-get install libproj-dev proj-bin
```

### Linux (Fedora/RHEL)

```bash
sudo dnf install proj proj-devel
```

### Linux (Arch)

```bash
sudo pacman -S proj
```

### Verify Installation

```bash
# Check if PROJ is installed
proj --version

# Check if pkg-config can find it
pkg-config --modversion proj
```

## Building XDL with GIS Support

Once PROJ is installed, enable the `gis` feature:

```bash
cd /path/to/xdl

# Build with GIS support
cargo build --release --features gis

# Or build with all optional features
cargo build --release --features all-features

# Run tests with GIS
cargo test --features gis
```

## Using GIS Features

### In Rust Code

```rust
#[cfg(feature = "gis")]
use xdl_stdlib::graphics::{MapProjection, ProjectionType};

#[cfg(feature = "gis")]
fn create_map() -> Result<(), Box<dyn Error>> {
    let mut projection = MapProjection::new(
        ProjectionType::Mercator,
        (0.0, 0.0)
    )?;
    
    projection.set_limits((-180.0, -90.0, 180.0, 90.0));
    
    // Use projection...
    Ok(())
}
```

### In Cargo.toml

```toml
[dependencies]
xdl-stdlib = { path = "../xdl-stdlib", features = ["gis"] }
```

## Troubleshooting

### Error: "pkg-config unable to find existing libproj installation"

**Solution**: Install PROJ library (see above) and ensure `pkg-config` can find it:

```bash
# Set PKG_CONFIG_PATH if needed
export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH

# Verify
pkg-config --modversion proj
```

### Error: "CMake Error at cmake/Ccache.cmake:10"

**Solution**: This occurs during automatic PROJ compilation. Installing PROJ via your system package manager (see above) avoids this issue.

### macOS: PROJ installed but not found

**Solution**: Set the `PKG_CONFIG_PATH` to your Homebrew location:

```bash
# For Intel Macs
export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH

# For Apple Silicon Macs
export PKG_CONFIG_PATH=/opt/homebrew/lib/pkgconfig:$PKG_CONFIG_PATH

# Add to ~/.zshrc to make permanent
echo 'export PKG_CONFIG_PATH=/opt/homebrew/lib/pkgconfig:$PKG_CONFIG_PATH' >> ~/.zshrc
```

## Feature Flags Summary

XDL provides the following feature flags:

- **`gis`** - Enable GIS and map projection features (requires PROJ)
- **`scientific-io`** - Enable NetCDF and HDF5 support (requires system libraries)
- **`all-features`** - Enable all optional features

### Building without GIS

The default build works without PROJ:

```bash
# Standard build (no GIS)
cargo build --release

# This will work and provide:
# - Color mapping
# - Terrain visualization
# - Scientific visualization
# - Vector fields
# - Export features
```

## Alternative: Python Integration

If you need GIS features but don't want to install PROJ, you can use XDL's Python integration:

```xdl
; Use Python's cartopy or geopandas for GIS
cartopy = python_import("cartopy")
gpd = python_import("geopandas")

; Perform map operations in Python
result = python_call(cartopy, "transform_points", points, projection)
```

This requires Python 3.13+ with the relevant packages installed:

```bash
pip install cartopy geopandas
```

## Documentation

For full GIS API documentation, see:
- `docs/SCIENTIFIC_VISUALIZATION_GUIDE.md` - Complete guide
- `xdl-stdlib/src/graphics/gis.rs` - Source code with examples

## Support

If you encounter issues:
1. Verify PROJ installation: `proj --version`
2. Check pkg-config: `pkg-config --modversion proj`
3. Ensure PROJ >= 9.2.0 is installed
4. Set `PKG_CONFIG_PATH` if needed
5. File an issue if problems persist

---

**Note**: GIS features are completely optional. XDL's core visualization capabilities (colormaps, terrain, vector fields, etc.) work without any external dependencies.
