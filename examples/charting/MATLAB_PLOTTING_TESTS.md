# MATLAB Plotting Tests for XDL GUI

This directory contains MATLAB test files that demonstrate automatic transpilation and plotting in the XDL GUI.

## Overview

The XDL GUI now automatically detects MATLAB syntax and transpiles it to XDL before execution. This means you can:

- Load `.m` files directly
- Type MATLAB code in the GUI command window
- Use MATLAB plotting functions seamlessly

## Test Files

### 1. `test_gui_output.m`

Basic output test to verify MATLAB transpilation works.

- Tests `disp()` function
- Tests variable display
- No plotting

### 2. `test_matlab_basic.m`

Comprehensive MATLAB syntax test.

- Variable assignments
- Array creation
- Conditionals
- Multiple `disp()` calls
- No plotting

### 3. `matlab_plot_simple.m`

Simple single plot test.

- Creates X and Y data arrays
- Single `plot()` call
- Uses `title()`, `xlabel()`, `ylabel()`

### 4. `matlab_plot_array.m`

Tests plotting with auto-generated X values.

- Single Y array
- GUI should auto-generate X indices
- Tests implicit X-axis generation

### 5. `matlab_plot_multiple.m`

Multiple sequential plots.

- Three separate plots
- Different mathematical functions
- Tests multiple plot windows

### 6. `matlab_comprehensive.m`

Complete feature test.

- Arithmetic operations
- Array creation and ranges
- Mathematical functions (sin, cos, exp)
- Multiple plots
- Conditionals
- Loops
- Extensive output messages

## MATLAB to XDL Function Mapping

The transpiler automatically maps MATLAB functions to XDL equivalents:

| MATLAB | XDL | Description |
|--------|-----|-------------|
| `disp(x)` | `PRINT, x` | Display output |
| `plot(x,y)` | `PLOT, x, y` | Create line plot |
| `xlabel(str)` | `XTITLE, str` | X-axis label |
| `ylabel(str)` | `YTITLE, str` | Y-axis label |
| `title(str)` | `TITLE, str` | Plot title |
| `figure()` | `WINDOW` | Create new figure (ignored in XDL) |
| `clf` | `ERASE` | Clear figure |
| `sin(x)` | `SIN(x)` | Sine function |
| `cos(x)` | `COS(x)` | Cosine function |
| `exp(x)` | `EXP(x)` | Exponential function |
| `[a:b]` | `a:b` | Range notation (stays same) |

## How to Test

### Method 1: Load .m file in GUI

```bash
cargo run --bin xdl-gui
```

Then:

1. File > Open
2. Navigate to `examples/charting/`
3. Select any `.m` file
4. Click Execute

You should see:

```text
=== Executing filename.m ===
✓ Transpiled MATLAB to XDL
✓ Executing with XDL interpreter
...
```

### Method 2: Type MATLAB code directly

1. Launch GUI: `cargo run --bin xdl-gui`
2. Clear the command window
3. Type or paste MATLAB code:

```matlab
% Quick test
x = [0:0.1:10];
y = sin(x);
plot(x, y);
disp('Plot created!');
```

4. Click Execute

The GUI will automatically detect MATLAB syntax (via `%` comments or `disp()` calls) and transpile it.

## Expected Behavior

1. **File Detection**: `.m` extension triggers MATLAB mode
2. **Syntax Detection**: Code with `disp()`, `fprintf()`, or `%` comments triggers MATLAB mode
3. **Transpilation**: MATLAB → XDL conversion happens automatically
4. **Execution**: Transpiled code runs in XDL interpreter
5. **Plotting**: Plot windows appear after execution completes
6. **Output**: Results appear in GUI output pane

## Supported Features

✅ Basic arithmetic and variables
✅ Arrays and ranges `[start:step:end]`
✅ Mathematical functions (sin, cos, exp, sqrt, etc.)
✅ Plotting with `plot(x, y)` or `plot(y)`
✅ Plot labels and titles
✅ Conditionals (if/else/end)
✅ Loops (for/end)
✅ Output with `disp()`
✅ Comments with `%`

## Known Limitations

⚠️ Figure management (`figure`, `clf`, `hold`) is ignored/simplified
⚠️ Subplots not yet implemented
⚠️ Advanced plot types (scatter, bar, etc.) not yet supported
⚠️ 3D plots (surf, mesh) limited support

## Troubleshooting

### "Variable not found" error

- The code might contain syntax the transpiler doesn't recognize
- Check for complex MATLAB-specific features
- Try simplifying the code

### Plots don't appear

- Ensure you're using `plot(x, y)` or `plot(y)`
- Check that data arrays are valid
- Look for transpilation errors in output

### Transpilation failed

- Check the output pane for error messages
- The transpiler will show original code with error marker
- Try breaking complex expressions into simpler steps

## Adding New Test Files

To create a new MATLAB test:

1. Create `.m` file in `examples/charting/`
2. Use MATLAB syntax (comments with `%`, `disp()`, etc.)
3. Test transpilation: load in GUI and execute
4. Document in this README

## Contributing

If you find MATLAB features that don't transpile correctly:

1. Create a minimal test case
2. Check `xdl-matlab/src/function_map.rs` for missing mappings
3. Update the transpiler if needed
4. Add test to this directory
