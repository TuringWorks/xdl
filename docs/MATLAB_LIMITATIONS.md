# MATLAB Transpiler Limitations

## Overview

The XDL MATLAB transpiler provides basic MATLAB compatibility but has several limitations. This document outlines what works and what doesn't.

## What Works ✅

### Basic Operations
- ✅ Scalar arithmetic: `x = 5 + 3;`
- ✅ Variable assignment: `a = 10;`
- ✅ Comments: `% This is a comment`
- ✅ Basic operators: `+, -, *, /, ^`

### Mathematical Functions
- ✅ Trigonometric: `sin(), cos(), tan(), asin(), acos(), atan()`
- ✅ Exponential/Log: `exp(), sqrt()`
- ✅ Natural log: `log()` → transpiles to `ALOG()`
- ✅ Base-10 log: `log10()` → transpiles to `ALOG10()`
- ✅ Other: `abs(), floor(), ceil(), round()`

### I/O Functions
- ✅ Display output: `disp(value)` → transpiles to `PRINT, value`
- ✅ Nested calls: `disp(sin(0.5))` works correctly

### Control Flow
- ✅ IF/ELSE/ENDIF statements
- ✅ FOR loops with ranges
- ✅ WHILE loops
- ✅ SWITCH/CASE statements
- ✅ BREAK and CONTINUE statements

## What Doesn't Work ❌

### Array/Matrix Operations
- ❌ Array literals: `[1, 2, 3, 4]`
- ❌ Matrix literals: `[1 2; 3 4]`
- ❌ Element-wise operations on arrays: `y = sin(x)` where x is array
- ❌ Array indexing with 1-based indexing
- ❌ Colon operator: `1:10`, `1:2:10`

### Array Generation Functions
- ✅ `linspace(start, end, n)` → transpiles to `LINSPACE()`
- ✅ `logspace()` → transpiles to `LOGSPACE()`
- ⚠️ `zeros()`, `ones()` - Use `REPLICATE(0, n)` or `DBLARR(n)` instead
- ✅ `eye()` → transpiles to `IDENTITY()`
- ✅ `rand()`, `randn()` → transpiles to `RANDOMU()`, `RANDOMN()`

### Plotting Features
- ❌ `figure` - window management
- ❌ `hold on/off` - multiple plots
- ❌ `xlabel`, `ylabel` - axis labels (use XDL keywords instead)
- ❌ `title` - plot title (use XDL keywords instead)
- ❌ `legend` - plot legend
- ❌ Line styles: `'b-'`, `'r--'`, etc.
- ❌ Markers: `'*'`, `'o'`, etc.
- ❌ Plot properties: `LineWidth`, `Marker`, etc.

### Advanced Features
- ❌ User-defined functions
- ❌ Anonymous functions/lambda: `@(x) x^2`
- ❌ Cell arrays: `{1, 2, 'three'}`
- ❌ Structures: `struct.field`
- ❌ Classes and objects
- ❌ String operations beyond basic strings
- ❌ File I/O: `fopen`, `fread`, `fwrite`, etc.
- ❌ `fprintf` with format strings

### Control Flow Limitations
- ✅ `switch/case` statements - Now supported
- ❌ `try/catch` error handling - Not yet implemented
- ✅ Complex FOR loop ranges: `for i = 1:2:10` - Supported
- ✅ `while` loops - Now supported
- ✅ `break`, `continue` - Now supported

## Workarounds

### Instead of Array Literals
MATLAB:
```matlab
x = [1, 2, 3, 4, 5];
```

XDL equivalent (write directly in XDL):
```xdl
x = FINDGEN(5) + 1  ; Creates [1, 2, 3, 4, 5]
```

### Instead of linspace
MATLAB:
```matlab
x = linspace(0, 2*pi, 100);
```

XDL equivalent:
```xdl
x = FINDGEN(100) * 2.0 * !PI / 99.0
```

### Instead of MATLAB Plotting
MATLAB:
```matlab
plot(x, y);
xlabel('X Axis');
ylabel('Y Axis');
title('My Plot');
```

XDL equivalent:
```xdl
PLOT, x, y, title='My Plot', xtitle='X Axis', ytitle='Y Axis'
```

### For Constants
MATLAB:
```matlab
x = 2 * pi;
```

Use XDL system variables:
```xdl
x = 2 * !PI
```

Available constants: `!PI`, `!E`, `!DTOR` (degrees to radians), `!RTOD` (radians to degrees)

## Recommended Approach

For best results with MATLAB files:

1. **Simple Scalar Math**: Use for basic calculations
   ```matlab
   a = 5;
   b = 10;
   c = sin(a) + cos(b);
   disp(c);
   ```

2. **Individual Function Calls**: Test mathematical functions
   ```matlab
   x = 0.5;
   y = sin(x);
   disp(y);
   ```

3. **Avoid Complex Features**: Don't use arrays, loops, or advanced plotting

4. **For Complex Work**: Write directly in XDL
   - Full array support
   - Proper FOR loops
   - Complete plotting with keywords
   - All XDL features available

## Migration Strategy

If you have MATLAB code to migrate:

1. **Simple scripts**: May work with transpiler (test first)
2. **Array operations**: Rewrite in XDL using FINDGEN, FLTARR, etc.
3. **Plotting**: Rewrite using XDL PLOT with keywords
4. **Complex logic**: Rewrite in XDL syntax

## Examples That Work

See `examples/matlab/` for working examples:
- `01_simple_math.m` - Basic arithmetic ✅
- `02_trigonometry.m` - Trig functions ✅
- `03_simple_operations.m` - sqrt, exp, log ✅
- `test_gui_output.m` - Output capture ✅

## Recent Improvements ✅

Enhancements completed in 2025:
1. ✅ `linspace()` and `logspace()` functions
2. ✅ Better FOR loop handling with ranges
3. ✅ WHILE loop support
4. ✅ SWITCH/CASE statement support
5. ✅ BREAK and CONTINUE statements
6. ✅ More complete function mapping (~80 functions)

## Future Improvements

Planned enhancements:
1. Array literal support: `[1, 2, 3]`
2. `try/catch` error handling
3. Plot style parsing (convert to XDL equivalents)
4. Low-level file I/O (`fopen`, `fread`, `fwrite`)

## Getting Help

If you encounter issues:
1. Check this document for known limitations
2. Try rewriting in XDL syntax
3. See `examples/xdl/` for XDL examples
4. Consult `docs/MATLAB_SUPPORT.md` for supported features
