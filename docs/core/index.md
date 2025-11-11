---
layout: default
title: Core Features
nav_order: 3
has_children: true
permalink: /core
---

# Core Features

Language implementation and core functionality of XDL.

## Language Features

XDL provides a complete implementation of IDL-compatible language features:

- **Variables and Expressions** - Dynamic typing, numeric and string variables
- **Control Flow** - IF/THEN/ELSE, FOR, WHILE, FOREACH loops
- **Functions and Procedures** - User-defined and built-in functions
- **Array Operations** - N-dimensional arrays with efficient indexing
- **Structures** - Complex data types
- **Keyword Arguments** - Named function parameters

## Documentation

- [Implementation Status](../IMPLEMENTATION_STATUS) - Current progress
- [Control Flow](../CONTROL_FLOW_IMPLEMENTATION) - Loops and conditionals
- [Array Features](../ARRAY_FEATURES) - Array operations
- [Multi-dimensional Arrays](../MULTIDIM_ARRAY_SUPPORT) - N-D arrays
- [Keyword Arguments](../KEYWORD_ARGS_IMPLEMENTATION) - Function keywords
- [Nested Functions Plan](../NESTED_FUNCTIONS_PLAN) - Function nesting support

## Data Types

XDL supports all IDL/GDL data types:

| Type | Description | Example |
|:-----|:------------|:--------|
| BYTE | 8-bit unsigned integer | 255B |
| INT | 16-bit signed integer | 32767 |
| LONG | 32-bit signed integer | 2147483647L |
| FLOAT | 32-bit floating point | 3.14 |
| DOUBLE | 64-bit floating point | 3.14D |
| COMPLEX | Complex numbers | COMPLEX(1, 2) |
| STRING | Character strings | 'Hello' |

## Array Operations

Powerful array manipulation:

```xdl
; Create arrays
a = findgen(100)
b = fltarr(10, 10)

; Array indexing
subset = a[0:49]
row = b[*, 0]

; Array operations
c = a + b
d = transpose(a)
e = reform(a, 10, 10)
```

## Control Flow

Complete control flow support:

```xdl
; IF/THEN/ELSE
if condition then begin
  ; statements
endif else begin
  ; statements
endelse

; FOR loops
for i = 0, 99 do begin
  ; statements
endfor

; WHILE loops
while condition do begin
  ; statements
endwhile
```

## Functions

Built-in and user-defined functions:

```xdl
; Mathematical functions
y = sin(x)
z = exp(log(x))

; Array functions
mean_val = mean(array)
sorted = sort(array)

; User-defined functions
function square, x
  return, x * x
end
```
