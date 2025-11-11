# XDL Graphics Quick Reference

## Available Demo Scripts

### 1. `plot_demo.xdl` - Simple 2D Plot
Basic sine wave demonstration.
```bash
cargo run --release --bin xdl-gui examples/plot_demo.xdl
```

### 2. `plot_working_demo.xdl` - Comprehensive 2D Tests
Tests all 2D plotting features with 5 different plots.
```bash
cargo run --release --bin xdl-gui examples/plot_working_demo.xdl
```

### 3. `plot3d_demo.xdl` - 3D Plotting Tests  
Tests 3D data structures with SURFACE, CONTOUR, SHADE_SURF, and PLOT3D.
```bash
cargo run --release --bin xdl examples/plot3d_demo.xdl
```

## Quick Examples

### 2D Line Plot
```xdl
x = FINDGEN(50) / 5.0
y = SIN(x)
PLOT, y, x
```

### 3D Surface Plot
```xdl
z = [[1, 2, 3, 2, 1], [2, 4, 6, 4, 2], [3, 6, 9, 6, 3]]
SURFACE, z
```

### Array Math
```xdl
x = FINDGEN(10)
y = COS(x * 2.0)       ; Array multiplication and COS
z = EXP(-x)             ; Array negation and EXP
w = x + SIN(x)          ; Array addition
```

## Supported Procedures

### ✅ Fully Working
- `PLOT` - 2D line plots with GUI/PNG output
- `SURFACE` - Parses 2D arrays for 3D surfaces
- `CONTOUR` - Parses 2D arrays for contours
- `SHADE_SURF` - Parses 2D arrays for shaded surfaces  
- `PLOT3D` - Parses 3 1D arrays for 3D lines

### ⚠️ Registered (Stubs)
See `GRAPHICS_DEMOS_STATUS.md` for complete list of 40+ registered procedures.

## Supported Math Functions (with Array Support)

- `SIN`, `COS`, `TAN` - Trigonometric functions
- `ASIN`, `ACOS`, `ATAN` - Inverse trig functions
- `EXP` - Exponential
- `SQRT` - Square root
- `ABS` - Absolute value
- `FINDGEN` - Generate float arrays

## Array Operations

### Arithmetic
```xdl
a = [1, 2, 3]
b = [4, 5, 6]
c = a + b        ; [5, 7, 9]
d = a * 2.0      ; [2, 4, 6]
e = a / 2.0      ; [0.5, 1.0, 1.5]
```

### Unary
```xdl
x = [1, 2, 3]
neg_x = -x       ; [-1, -2, -3]
```

### Element-wise Functions
```xdl
angles = FINDGEN(10) * 0.1
sines = SIN(angles)      ; Sin of each element
exps = EXP(-angles)      ; Exp of each negative element
```

## Nested Arrays for 2D Data

```xdl
; 3x3 matrix
matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]

; Access rows
row1 = matrix[0]         ; [1, 2, 3]

; Access elements
elem = matrix[1, 1]      ; 5

; Use in 3D plots
SURFACE, matrix
```

## Common Patterns

### Plotting Waves
```xdl
t = FINDGEN(100) / 10.0
y = SIN(t)
PLOT, y, t
```

### Exponential Decay
```xdl
t = FINDGEN(50) / 5.0
y = EXP(-t)
PLOT, y, t
```

### Combined Functions
```xdl
x = FINDGEN(80) / 8.0
y = SIN(x) + COS(x * 2.0)
PLOT, y, x
```

### 3D Parametric Curves
```xdl
t = FINDGEN(50) / 5.0
x = COS(t)
y = SIN(t)
z = t / 2.0
PLOT3D, x, y, z
```

## Tips

1. **GUI vs CLI:** Use `xdl-gui` for interactive plots, `xdl` for PNG output
2. **Array Size:** Keep demo arrays small (< 100 elements) for quick testing
3. **Nested Arrays:** All rows must have same length for 2D data
4. **Math Functions:** All support both scalars and arrays
5. **Negation:** Use parentheses for clarity: `y = EXP(-(x))` or `y = EXP(-x)`

## Troubleshooting

### "Unknown procedure: PLOT"
Make sure you're running the correct binary:
```bash
# GUI mode
cargo run --release --bin xdl-gui script.xdl

# CLI mode
cargo run --release --bin xdl script.xdl
```

### "Type mismatch" errors
Check that math functions are using correct types. All should work with arrays now.

### "Expected a 2D nested array"
For SURFACE/CONTOUR/SHADE_SURF, use nested array literals:
```xdl
z = [[1,2,3], [4,5,6], [7,8,9]]  ; Correct
z = FLTARR(3, 3)                  ; Not yet implemented
```

## See Also

- `GRAPHICS_DEMOS_STATUS.md` - Detailed status of all procedures
- `GRAPHICS_FIXES_SUMMARY.md` - Recent fixes and changes
- `QUICKSTART_GRAPHICS.md` - Complete graphics guide
- `GRAPHICS_IMPLEMENTATION.md` - Technical implementation details
