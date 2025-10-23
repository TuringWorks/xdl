# XDL-MATLAB: MATLAB Compatibility Layer

This crate provides MATLAB/Octave compatibility for XDL, enabling:

- Loading and executing .m files
- Transpiling MATLAB syntax to XDL
- Mapping MATLAB functions to XDL equivalents

## Features

### Syntax Transpilation

Converts MATLAB code to XDL-compatible syntax:

- **Comments**: `%` → `;`
- **Arrays**: 1-based → 0-based indexing
- **Loops**: `for i = 1:10` → `for i = 0, 9`
- **Element-wise ops**: `.*` → `*`, `./` → `/`, `.^` → `^`

### Function Mapping (~80 functions)

| MATLAB | XDL | Notes |
|--------|-----|-------|
| `zeros(n)` | `FLTARR(n)` | Array creation |
| `sin(x)` | `SIN(x)` | Trig functions |
| `mean(x)` | `MEAN(x)` | Statistics |
| `plot(x, y)` | `PLOT, y, x` | Plotting |
| `disp(x)` | `PRINT, x` | I/O |

See `function_map.rs` for complete list.

## Usage

### As a Library

```rust
use xdl_matlab::transpile_matlab_to_xdl;

let matlab_code = r#"
x = zeros(10, 1);
for i = 1:10
    x(i) = sin(i * 0.1);
end
"#;

let xdl_code = transpile_matlab_to_xdl(matlab_code)?;
println!("{}", xdl_code);
```

### Load .m Files

```rust
use xdl_matlab::load_matlab_file;

let xdl_code = load_matlab_file(Path::new("script.m"))?;
// Execute with XDL interpreter
```

## Example

**Input MATLAB:**
```matlab
% Calculate mean
x = zeros(10, 1);
for i = 1:10
    x(i) = i * 2;
end
mean_x = mean(x);
disp('Mean value:');
disp(mean_x);
```

**Output XDL:**
```idl
; Calculate mean
x = FLTARR ( 10 , 1 )
for i = 0, 9
  x [ i ] = i * 2
endfor
mean_x = MEAN ( x )
PRINT , 'Mean value:'
PRINT , mean_x
```

## Limitations

### Current Implementation

- **Basic syntax only**: Functions, loops, conditionals
- **Simple indexing**: Multi-dimensional arrays need work
- **No cell arrays**: `{}` syntax not yet supported
- **No classes/objects**: OOP features not implemented

### Index Adjustment

MATLAB uses 1-based indexing, XDL uses 0-based:

```matlab
x(1)    % MATLAB: first element
```
```idl
x[0]    % XDL: first element
```

The transpiler automatically adjusts simple numeric indices.

## Roadmap

- [x] Lexer for MATLAB syntax
- [x] Basic transpiler (assignments, loops, functions)
- [x] Function name mapping (~80 functions)
- [ ] Matrix operations and slicing
- [ ] Cell arrays and structures
- [ ] Handle/function handles (@func syntax)
- [ ] MEX file interface
- [ ] Package manager integration
- [ ] Advanced indexing (end, :, etc.)

## Testing

```bash
cargo test -p xdl-matlab
```

## Contributing

Function mappings and syntax rules are in:
- `function_map.rs` - Add new MATLAB→XDL function mappings
- `transpiler.rs` - Extend syntax transpilation rules
- `lexer.rs` - Handle new token types

## License

Same as parent XDL project.
