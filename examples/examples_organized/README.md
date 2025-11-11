# XDL Examples - Organized Collection

This directory contains the complete, organized collection of XDL examples, categorized by functionality.

## 📂 Directory Structure

```text
examples_organized/
├── basics/              # Fundamental XDL concepts
├── plotting/            # 2D plotting and visualization
├── visualization_3d/    # 3D rendering and volume visualization
├── machine_learning/    # ML algorithms and data processing
├── scientific/          # Advanced scientific computing
├── matlab/              # MATLAB/Octave compatibility
├── tests_working/       # Working test examples
└── tests_failing/       # Examples needing fixes
```

## 📊 Statistics

- **Total Examples**: 150 files (.xdl, .pro, .m)
- **Working Examples**: 24 (execute successfully)
- **Failing Examples**: 77 (need fixes or dependencies)
- **Visual Examples**: 22 (generate plots/visualizations)
- **Generated Images**: 19 images from 12 examples

## 🚀 Running Examples

### Individual Examples

```bash
# Run any XDL example
xdl basics/01_hello_world.xdl

# Run plotting examples
xdl plotting/03_plotting_basics.xdl
xdl plotting/scatter_demo.xdl
```

### Batch Testing

```bash
# Run the example testing script
./run_examples_test.sh

# Generate images from visual examples
python3 generate_images.py
```

## 📁 Category Descriptions

### Basics (10 examples)

Fundamental XDL concepts including:

- Variable assignment and arithmetic
- Array operations and loops
- Control flow structures
- Basic function calls

### Plotting (11 examples)

2D visualization capabilities:

- Line plots and scatter plots
- Surface plots and contours
- Charting and data visualization
- Plot customization

### Visualization 3D (7 examples)

Advanced 3D rendering:

- Volume rendering
- 3D surface plots
- Interactive 3D visualizations
- VIZ3D library usage

### Machine Learning (1 example)

ML algorithms and data processing:

- K-means clustering
- Neural networks
- Data preprocessing

### Scientific (13 examples)

Advanced scientific applications:

- Fluid dynamics simulations
- Mandelbrot set generation
- Geophysical data processing
- Medical imaging
- Molecular structure visualization

### MATLAB (7 examples)

MATLAB/Octave compatibility:

- Basic mathematical operations
- Plotting functions
- Control flow structures
- Array operations

### Tests Working (24 examples)

Validated test examples that execute successfully.

### Tests Failing (77 examples)

Examples that currently fail execution and may need:

- Bug fixes
- Additional dependencies
- Updated syntax

## 🖼️ Generated Images

Visual examples automatically generate images saved in `examples_images/`:

- 2D plots (line plots, scatter plots)
- Surface plots and contours
- 3D visualizations
- Scientific data visualizations

See the [Examples Gallery](../docs/examples-gallery.md) for visual previews.

## 🔧 Maintenance

### Adding New Examples

1. Place examples in appropriate category directories
2. Run `./run_examples_test.sh` to test execution
3. Run `python3 generate_images.py` to generate images
4. Update documentation as needed

### Updating Categories

Run the organization script to recategorize:

```bash
python3 organize_examples.py
```

## 📋 File Types

- **`.xdl`**: XDL source files (primary format)
- **`.pro`**: IDL/GDL procedure files
- **`.m`**: MATLAB/Octave script files

## 🤝 Contributing

When adding examples:

- Include comments explaining functionality
- Test execution with `xdl`
- Ensure examples are self-contained
- Add appropriate category classification
