#!/usr/bin/env python3

import os
import shutil
import json

# Load the results from our testing
with open('examples_output/success_files.txt', 'r') as f:
    success_files = [line.strip().replace('SUCCESS: ', '') for line in f if line.strip()]

with open('examples_output/failed_files.txt', 'r') as f:
    failed_files = [line.strip().replace('FAILED: ', '').split(' ')[0] for line in f if line.strip()]

with open('examples_output/visual_files.txt', 'r') as f:
    visual_files = [line.strip().replace('VISUAL: ', '') for line in f if line.strip()]

# Categorization logic
categories = {
    'basics': [],
    'plotting': [],
    'visualization_3d': [],
    'machine_learning': [],
    'scientific': [],
    'matlab': [],
    'tests_working': [],
    'tests_failing': []
}

# Function to categorize a file
def categorize_file(filepath):
    basename = os.path.basename(filepath)
    dirname = os.path.dirname(filepath)

    # Check if it's a test file
    if 'test' in dirname or 'test' in basename.lower():
        if filepath in success_files:
            return 'tests_working'
        else:
            return 'tests_failing'

    # MATLAB files
    if filepath.endswith('.m'):
        return 'matlab'

    # Machine learning examples
    if 'ml_' in basename or 'machine' in basename.lower() or 'kmeans' in basename.lower():
        return 'machine_learning'

    # 3D visualization
    if 'viz3d' in basename.lower() or '3d' in basename.lower() or 'volume' in basename.lower() or 'torus' in basename.lower() or 'galaxy' in basename.lower():
        return 'visualization_3d'

    # Scientific examples
    if 'scientific' in dirname or 'fluid' in basename.lower() or 'rayleigh' in basename.lower() or 'mandelbrot' in basename.lower() or 'geophysical' in basename.lower() or 'medical' in basename.lower():
        return 'scientific'

    # Plotting examples
    if filepath in visual_files or 'plot' in basename.lower() or 'chart' in dirname or 'contour' in basename.lower() or 'surface' in basename.lower():
        return 'plotting'

    # Basic examples
    if basename.startswith(('01_', '02_', '03_', '04_', '05_', '06_')) or 'hello' in basename.lower() or 'basic' in basename.lower():
        return 'basics'

    # Default to basics
    return 'basics'

# Process all files
all_files = []
for root, dirs, files in os.walk('examples'):
    for file in files:
        if file.endswith(('.xdl', '.pro', '.m')):
            filepath = os.path.join(root, file)
            all_files.append(filepath)

for root, dirs, files in os.walk('tests'):
    for file in files:
        if file.endswith(('.xdl', '.pro', '.m')):
            filepath = os.path.join(root, file)
            all_files.append(filepath)

# Remove duplicates
all_files = list(set(all_files))

# Categorize and copy
for filepath in all_files:
    category = categorize_file(filepath)
    categories[category].append(filepath)

    # Copy to organized directory
    dest_dir = f'examples_organized/{category}'
    os.makedirs(dest_dir, exist_ok=True)

    # Create a relative path for the destination
    rel_path = os.path.relpath(filepath)
    dest_file = os.path.join(dest_dir, os.path.basename(filepath))

    # Handle duplicates by adding a suffix
    counter = 1
    original_dest = dest_file
    while os.path.exists(dest_file):
        name, ext = os.path.splitext(original_dest)
        dest_file = f"{name}_{counter}{ext}"
        counter += 1

    shutil.copy2(filepath, dest_file)
    print(f"Copied {filepath} -> {dest_file}")

# Create README files for each category
readme_templates = {
    'basics': """# Basic XDL Examples

These examples demonstrate fundamental XDL concepts:

- Variable assignment and basic operations
- Control flow (loops, conditionals)
- Array operations
- Basic mathematical functions

## Examples
""",
    'plotting': """# Plotting and Visualization Examples

Examples demonstrating 2D plotting capabilities:

- Line plots and scatter plots
- Surface plots and contours
- Charting and data visualization
- Basic plotting functions

## Examples
""",
    'visualization_3d': """# 3D Visualization Examples

Advanced 3D rendering and visualization:

- Volume rendering
- 3D surface plots
- Interactive 3D visualizations
- VIZ3D library usage

## Examples
""",
    'machine_learning': """# Machine Learning Examples

ML algorithms and data processing:

- K-means clustering
- Neural networks (RNN, CNN)
- Cross-validation
- Data preprocessing and normalization

## Examples
""",
    'scientific': """# Scientific Computing Examples

Advanced scientific applications:

- Fluid dynamics simulations
- Mandelbrot set generation
- Geophysical data processing
- Medical imaging
- Molecular structure visualization

## Examples
""",
    'matlab': """# MATLAB Compatibility Examples

Examples demonstrating MATLAB/Octave compatibility:

- Basic mathematical operations
- Plotting functions
- Control flow structures
- Array operations

## Examples
""",
    'tests_working': """# Working Test Examples

Test files that execute successfully:

- Unit tests for various XDL features
- Integration tests
- Performance tests
- Compatibility tests

## Examples
""",
    'tests_failing': """# Failing Test Examples

Test files that currently fail execution:

- These may need fixes or additional dependencies
- Useful for debugging and development

## Examples
"""
}

for category, files in categories.items():
    readme_path = f'examples_organized/{category}/README.md'
    with open(readme_path, 'w') as f:
        f.write(readme_templates[category])
        for filepath in sorted(files):
            basename = os.path.basename(filepath)
            f.write(f"- [{basename}]({basename})\n")

# Create summary
summary = {
    'total_files': len(all_files),
    'categories': {cat: len(files) for cat, files in categories.items()},
    'working_examples': len([f for f in all_files if f in success_files]),
    'failing_examples': len([f for f in all_files if f in failed_files]),
    'visual_examples': len([f for f in all_files if f in visual_files])
}

with open('examples_organized/summary.json', 'w') as f:
    json.dump(summary, f, indent=2)

print("Examples organization complete!")
print(f"Total files: {summary['total_files']}")
print(f"Working examples: {summary['working_examples']}")
print(f"Failing examples: {summary['failing_examples']}")
print(f"Visual examples: {summary['visual_examples']}")