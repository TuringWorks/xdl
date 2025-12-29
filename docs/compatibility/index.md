---
layout: default
title: Compatibility
nav_order: 6
has_children: true
permalink: /compatibility
---

# Compatibility

IDL/GDL and MATLAB compatibility layers.

## Overview

XDL provides compatibility with:

- **IDL** (Interactive Data Language) - Full syntax and function compatibility
- **GDL** (GNU Data Language) - Compatible with GDL extensions
- **MATLAB** - Subset of MATLAB syntax and functions

## IDL/GDL Compatibility

### Supported IDL Commands

XDL implements a comprehensive set of IDL commands and functions:

- **Array Functions** - FINDGEN, FLTARR, INDGEN, etc.
- **Math Functions** - SIN, COS, EXP, LOG, SQRT, etc.
- **I/O Functions** - PRINT, READ, OPENR, WRITEU, etc.
- **Graphics** - PLOT, SURFACE, CONTOUR, etc.
- **Control Flow** - IF/THEN/ELSE, FOR, WHILE, FOREACH

See [IDL Command Status]({% link IDL_COMMAND_STATUS.md %}) for complete list.

### IDL Syntax Support

```xdl
; IDL-compatible syntax
a = findgen(100)
b = sin(a * !pi / 50)
plot, a, b, title='Sine Wave', xtitle='X', ytitle='Y'

; Structures
struct = {name: 'Test', value: 42}

; Keywords
result = function_name(arg1, arg2, KEYWORD=value)
```

### Documentation

- [IDL Command Status]({% link IDL_COMMAND_STATUS.md %}) - Command compatibility
- [GDL/IDL Compatibility]({% link GDL_IDL_COMPATIBILITY.md %}) - Compatibility layer
- [GDL/IDL Syntax]({% link GDL_IDL_SYNTAX.md %}) - Syntax reference
- [Gap Analysis]({% link GDL_XDL_GAP_ANALYSIS.md %}) - Feature comparison
- [Porting Status]({% link GDL_XDL_PORTING_STATUS.md %}) - Porting progress
- [XDL Files Status]({% link XDL_FILES_STATUS.md %}) - File compatibility status

## MATLAB Compatibility

### Supported MATLAB Features

XDL can run many MATLAB programs:

- **Basic Operations** - Arithmetic, arrays, matrices
- **Math Functions** - sin, cos, sqrt, exp, log, etc.
- **Array Functions** - size, length, reshape, transpose
- **Plotting** - plot, scatter, surf, mesh
- **Control Flow** - if/else, for, while

### MATLAB Syntax

```matlab
% MATLAB code running in XDL
x = 1:100;
y = sin(x * pi / 50);
plot(x, y);
title('Sine Wave');
xlabel('X');
ylabel('Y');
```

### Auto-Transpilation

XDL automatically transpiles MATLAB code:

```bash
# Run MATLAB file directly
xdl script.m

# XDL detects .m extension and transpiles automatically
```

### Documentation

- [MATLAB Compatibility]({% link MATLAB_COMPATIBILITY.md %}) - Overview
- [MATLAB Plotting Guide]({% link MATLAB_PLOTTING_GUIDE.md %}) - Plotting functions
- [MATLAB Limitations]({% link MATLAB_LIMITATIONS.md %}) - Known limitations
- [MATLAB Support]({% link MATLAB_SUPPORT.md %}) - Feature support
- [MATLAB Real World]({% link MATLAB_REAL_WORLD_SUPPORT.md %}) - Real-world usage
- [MATLAB Critical Fixes]({% link MATLAB_CRITICAL_FIXES.md %}) - Bug fixes
- [Tiled Layout]({% link MATLAB_TILEDLAYOUT.md %}) - Tiled layout support

## Migration Guide

### From IDL to XDL

Most IDL code runs without changes:

```xdl
; IDL code - works directly in XDL
data = findgen(1000)
result = fft(data)
plot, abs(result)
```

### From MATLAB to XDL

MATLAB code may need minor adjustments:

```matlab
% MATLAB
A = [1, 2, 3];     % May need adjustment

% XDL equivalent
A = [1, 2, 3]      % Works in some cases
```

See [MATLAB Limitations]({% link MATLAB_LIMITATIONS.md %}) for details.

## Compatibility Status

| Feature | IDL | GDL | MATLAB | Status |
|:--------|:----|:----|:-------|:-------|
| Basic Syntax | ✅ | ✅ | ⚠️ | Complete/Partial |
| Arrays | ✅ | ✅ | ✅ | Complete |
| Math Functions | ✅ | ✅ | ✅ | Complete |
| Graphics | ✅ | ✅ | ⚠️ | Mostly Complete |
| I/O | ✅ | ✅ | ⚠️ | In Progress |
| Structures | ✅ | ✅ | ❌ | IDL/GDL Only |
| OOP | ⚠️ | ⚠️ | ❌ | Partial |

Legend: ✅ Complete | ⚠️ Partial | ❌ Not Supported

## Testing

Test your compatibility:

```bash
# Run IDL code
xdl idl_script.pro

# Run GDL code
xdl gdl_script.gdl

# Run MATLAB code
xdl matlab_script.m
```
