# XDL Examples

This directory contains example scripts demonstrating various features of XDL and MATLAB compatibility.

## Directory Structure

```
examples/
├── xdl/        - Native XDL script examples
└── matlab/     - MATLAB .m file examples that are transpiled to XDL
```

## XDL Examples

### 01_hello_world.xdl
Basic introduction to XDL:
- Variable assignment
- PRINT statements
- Basic arithmetic operations

**Run with:**
```bash
xdl examples/xdl/01_hello_world.xdl
```

### 02_arrays_and_loops.xdl
Working with arrays:
- Creating arrays with FINDGEN and FLTARR
- FOR loops with BEGIN...END blocks
- Array indexing and manipulation

**Run with:**
```bash
xdl examples/xdl/02_arrays_and_loops.xdl
```

### 03_plotting_basics.xdl
Basic plotting:
- Creating data arrays
- PLOT command with keyword arguments
- Using title, xtitle, and ytitle

**Run with:**
```bash
xdl examples/xdl/03_plotting_basics.xdl
```

### 04_trigonometry.xdl
Mathematical functions:
- Trigonometric functions (SIN, COS, TAN)
- Working with constants (PI)
- Plotting trigonometric curves

**Run with:**
```bash
xdl examples/xdl/04_trigonometry.xdl
```

### 05_conditionals.xdl
Conditional logic:
- IF/THEN/ELSE statements
- BEGIN...END blocks
- Nested conditionals

**Run with:**
```bash
xdl examples/xdl/05_conditionals.xdl
```

## MATLAB Examples

These examples demonstrate MATLAB code that is automatically transpiled to XDL.

### 01_simple_math.m
Basic MATLAB operations:
- Variable assignment
- Arithmetic operations
- fprintf output

**Run with:**
```bash
xdl examples/matlab/01_simple_math.m
```

### 02_trigonometry.m
MATLAB trigonometric functions:
- sin, cos, tan functions
- Working with angles in radians
- Formatted output

**Run with:**
```bash
xdl examples/matlab/02_trigonometry.m
```

### 03_simple_operations.m
MATLAB mathematical operations:
- Square root (sqrt)
- Exponential (exp)
- Natural logarithm (log)
- Power operations

**Run with:**
```bash
xdl examples/matlab/03_simple_operations.m
```

## Running Examples

### Using the CLI

Run any example file directly:
```bash
# XDL files
xdl examples/xdl/01_hello_world.xdl

# MATLAB files (auto-transpiled)
xdl examples/matlab/01_simple_math.m
```

### Using the GUI

1. Launch the GUI:
   ```bash
   xdl-gui
   ```

2. Go to **File > Open...**

3. Navigate to the `examples/` directory

4. Select any `.xdl` or `.m` file

5. Click **Execute** to run

## Notes

### MATLAB Compatibility

The MATLAB transpiler has some limitations:

- **Array literals**: `[1, 2, 3]` syntax is not yet fully supported
- **Complex expressions**: Some advanced MATLAB features may not work
- **Functions**: User-defined functions have limited support

For best results with MATLAB files:
- Use simple scalar operations
- Use standard mathematical functions
- Avoid complex array literals

### Plotting

When running examples with plots:
- A plot window will open
- The script will pause until you close the window
- Multiple plots can be created in sequence

### Performance

Examples are designed to run quickly. For larger datasets or more complex operations, XDL can handle arrays with thousands of elements efficiently.
