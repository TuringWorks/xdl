# MATLAB to XDL Plotting Guide

## Overview

MATLAB's plotting syntax is significantly different from XDL's, and many MATLAB plotting features are not supported by the transpiler. This guide shows how to translate MATLAB plotting code to XDL.

## The Challenge

Complex MATLAB plotting code like this will **not work**:

```matlab
x = linspace(0, 2*pi, 100);
y1 = sin(x);
y2 = cos(x);
figure;
plot(x, y1, 'b-');
hold on;
plot(x, y2, 'r--*');
xlabel('x-axis');
ylabel('y-axis');
title('Sine and Cosine');
legend('Sine', 'Cosine');
hold off;
```

**Why it doesn't work:**

- ❌ `linspace()` not implemented
- ❌ `figure`, `hold` not supported
- ❌ `xlabel`, `ylabel` separate commands not supported
- ❌ `legend` not supported
- ❌ Line styles `'b-'`, `'r--*'` not parsed
- ❌ Array operations `sin(x)` where x is array

## The Solution: Write in XDL

Here's the equivalent XDL code that **does work**:

```xdl
; Create x values from 0 to 2*PI
n_points = 100
x = FINDGEN(n_points) * 2.0 * !PI / (n_points - 1)

; Calculate sine and cosine
y_sin = SIN(x)
y_cos = COS(x)

; Plot with labels
PLOT, x, y_sin, title='Sine and Cosine Functions', xtitle='X (radians)', ytitle='Y values'
```

## Key Differences

### 1. Array Generation

**MATLAB:**

```matlab
x = linspace(0, 2*pi, 100);
```

**XDL:**

```xdl
x = FINDGEN(100) * 2.0 * !PI / 99.0
```

### 2. Constants

**MATLAB:**

```matlab
x = 2 * pi;
```

**XDL:**

```xdl
x = 2 * !PI
```

Available: `!PI`, `!E`, `!DTOR`, `!RTOD`

### 3. Plotting Syntax

**MATLAB:**

```matlab
plot(x, y);
xlabel('X Axis');
ylabel('Y Axis');
title('My Plot');
```

**XDL:**

```xdl
PLOT, x, y, title='My Plot', xtitle='X Axis', ytitle='Y Axis'
```

### 4. Multiple Plots

**MATLAB:**

```matlab
figure;
plot(x, y1);
hold on;
plot(x, y2);
hold off;
```

**XDL (Current Limitation):**

```xdl
; Currently, XDL generates one plot per PLOT command
; Multiple curves on same plot not yet supported
PLOT, x, y_sin, title='Sine Function', xtitle='X', ytitle='Sin(X)'
; This would overwrite the previous plot:
; PLOT, x, y_cos, title='Cosine Function', xtitle='X', ytitle='Cos(X)'
```

## Practical Examples

### Example 1: Simple Sine Plot

**MATLAB (.m file):**

```matlab
% Won't work - uses linspace and array operations
x = linspace(0, 6.28, 50);
y = sin(x);
plot(x, y);
title('Sine Wave');
```

**XDL (.xdl file) - Works:**

```xdl
x = FINDGEN(50) * 6.28 / 49.0
y = SIN(x)
PLOT, x, y, title='Sine Wave', xtitle='X', ytitle='sin(x)'
```

### Example 2: Exponential Function

**MATLAB (.m file):**

```matlab
% This actually works! (simple scalar math)
x = 2.0;
y = exp(x);
disp(y);
```

**Output:** `7.389...`

This works because it uses only:

- Scalar variables
- Supported functions (`exp`)
- `disp` for output

### Example 3: What You Can Do in MATLAB Files

For simple calculations, `.m` files work:

```matlab
% examples/matlab/simple_calc.m
a = 5;
b = 10;
c = sqrt(a^2 + b^2);
disp('Hypotenuse:');
disp(c);
```

This transpiles and runs successfully!

## Best Practices

### Use MATLAB Files For

- ✅ Simple scalar calculations
- ✅ Testing individual math functions
- ✅ Basic demonstrations
- ✅ Learning/prototyping

### Use XDL Files For

- ✅ Array operations
- ✅ Plotting
- ✅ Loops
- ✅ Complex programs
- ✅ Production code

## Complete Working Example

See these examples in the `examples/` directory:

**MATLAB (Limited):**

- `examples/matlab/01_simple_math.m` ✅
- `examples/matlab/02_trigonometry.m` ✅
- `examples/matlab/03_simple_operations.m` ✅
- `examples/matlab/04_simple_plot.m` ✅ (demonstrates limitations)

**XDL (Full Featured):**

- `examples/xdl/03_plotting_basics.xdl` ✅
- `examples/xdl/04_trigonometry.xdl` ✅
- `examples/xdl/06_sine_cosine_plot.xdl` ✅

## Migration Path

If you have MATLAB code:

1. **Try It**: Run simple `.m` files through the transpiler
2. **Check Errors**: Note what features aren't supported
3. **Rewrite in XDL**: For arrays, plots, and complex features
4. **Use XDL Syntax**: Take advantage of full XDL capabilities

## Future Work

The transpiler may eventually support:

- `linspace()` function
- Array literal syntax
- Better loop support
- Plot style hints (converted to XDL equivalents)

For now, use native XDL for anything beyond basic scalar math.

## Quick Reference

| MATLAB | XDL Equivalent | Status |
|--------|----------------|--------|
| `linspace(0, 10, 100)` | `FINDGEN(100) * 10.0 / 99.0` | Workaround |
| `pi` | `!PI` | Use system var |
| `plot(x,y)` | `PLOT, x, y` | Different syntax |
| `[1,2,3]` | `FINDGEN(3) + 1` | No direct equiv |
| `sin(array)` | `SIN(array)` | XDL only |
| `disp(x)` | `PRINT, x` | Transpiled ✅ |
| `log(x)` | `ALOG(x)` | Transpiled ✅ |

## See Also

- `docs/MATLAB_LIMITATIONS.md` - Complete limitations list
- `docs/MATLAB_SUPPORT.md` - Supported features
- `examples/README.md` - All examples documentation
