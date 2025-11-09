---
layout: default
title: Examples
parent: Getting Started
nav_order: 6
---

# Examples

Sample XDL code and tutorials.

For the complete examples documentation, see [README.md](../README).

## XDL Examples

### Hello World

```xdl
; Basic introduction to XDL
print, 'Hello, XDL!'

x = 42
print, 'The answer is:', x

; Arithmetic
y = x * 2 + 10
print, 'Calculation:', y
```

### Arrays and Loops

```xdl
; Create array
arr = findgen(10)
print, 'Array:', arr

; Loop through array
for i = 0, 9 do begin
  arr[i] = arr[i] * 2
  print, 'Element', i, ':', arr[i]
endfor
```

### Trigonometry

```xdl
; Trigonometric functions
x = findgen(100) * !pi / 50
y_sin = sin(x)
y_cos = cos(x)
y_tan = tan(x)

; Plot results
plot, x, y_sin, title='Sine Wave'
oplot, x, y_cos, color='red'
```

### Conditionals

```xdl
; If/Then/Else
x = 42

if x gt 50 then begin
  print, 'x is greater than 50'
endif else begin
  print, 'x is less than or equal to 50'
endelse
```

## MATLAB Examples

XDL can run MATLAB code:

### Simple Math

```matlab
% MATLAB code running in XDL
x = 10;
y = 20;
z = x + y;
fprintf('Result: %d\n', z);
```

### Matrix Operations

```matlab
% Matrix operations
A = [1, 2; 3, 4];
B = [5, 6; 7, 8];
C = A * B;
disp(C);
```

## Scientific Examples

### Rayleigh-Taylor Simulation

See [Rayleigh-Taylor Demo](../advanced/rayleigh-taylor) for a complete physics simulation example.

### Bezier Curves

See [Bezier Demo](../graphics/bezier) for curve drawing examples.

### GPU Computation

See [GPU Demo](gpu-demo) for GPU-accelerated examples.

## Running Examples

```bash
# Run XDL file
xdl examples/xdl/01_hello_world.xdl

# Run MATLAB file (auto-transpiled)
xdl examples/matlab/01_simple_math.m

# Interactive REPL
xdl
```

## Next Steps

- [Core Features](../core) - Language reference
- [Graphics](../graphics) - Visualization features
- [Advanced Topics](../advanced) - Complex examples
