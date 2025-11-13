# Python 3.13 Integration with XDL

This document describes the Python integration capabilities in XDL using Python 3.13.0.

## Overview

XDL provides seamless integration with Python 3.13.0 through the PyO3 library, allowing you to:

- Import Python modules
- Call Python functions and methods
- Pass data between XDL and Python
- Use Python's extensive scientific computing ecosystem

## Configuration

The Python interpreter path is configured in `.cargo/config.toml`:

```toml
[env]
PYO3_PYTHON = "/Users/ravindraboddipalli/.pyenv/shims/python3"
```

## Available Functions

### `PYTHON_IMPORT(module_name)`

Imports a Python module and returns a module handle.

Example:

```xdl
math_module = python_import("math")
sys_module = python_import("sys")
numpy_module = python_import("numpy")
```

### `PYTHON_CALL(module_handle, function_name, args...)`

Calls a Python function with the given arguments.

Example:

```xdl
; Import math module
math_mod = python_import("math")

; Call math functions
result1 = python_call(math_mod, "sqrt", 16.0)    ; Returns 4.0
result2 = python_call(math_mod, "sin", 0.0)      ; Returns 0.0
result3 = python_call(math_mod, "pi")            ; Returns π
```

### `PYTHON_CALL_KW(module_handle, function_name, args..., kwargs...)`

Calls a Python function with both positional and keyword arguments.

Example:

```xdl
; Call with keyword arguments
result = python_call_kw(numpy_mod, "linspace", 0, 10, "num=50")
```

## Type Conversion

XDL automatically converts between XDL types and Python types:

| XDL Type | Python Type |
|----------|-------------|
| Long     | int         |
| Double   | float       |
| Float    | float       |
| String   | str         |
| Undefined| None        |

## Examples

### Basic Math Operations

```xdl
print, "=== Python Math Integration ==="
math_mod = python_import("math")

; Calculate square root
sqrt_16 = python_call(math_mod, "sqrt", 16.0)
print, "sqrt(16) =", sqrt_16

; Calculate trigonometric functions
sin_pi_2 = python_call(math_mod, "sin", 1.5708)
print, "sin(π/2) =", sin_pi_2
```

### System Information

```xdl
print, "=== Python System Info ==="
sys_mod = python_import("sys")
version = python_call(sys_mod, "version")
print, "Python version:", version
```

### Working with Built-ins

```xdl
print, "=== Python Built-ins ==="
builtins_mod = python_import("builtins")
abs_val = python_call(builtins_mod, "abs", -42)
print, "abs(-42) =", abs_val
```

## Error Handling

If a Python operation fails, XDL will return an appropriate error:

- Module import failures
- Function call errors
- Type conversion errors

## Requirements

- Python 3.13.0 or compatible version
- PyO3 0.22+ for Rust-Python interop
- Standard Python libraries available in your Python installation

## Testing

Run the test script to verify Python integration:

```bash
cargo run --bin xdl test_python_integration.xdl
```

## Performance Notes

- Python calls have some overhead due to the Python C API
- Complex data structures are converted to strings for simplicity
- For heavy numerical computing, consider implementing performance-critical parts in Rust

## Future Enhancements

Planned improvements include:

- NumPy array support
- Pandas DataFrame integration
- Matplotlib plotting integration
- Better error propagation
- Asynchronous Python calls
