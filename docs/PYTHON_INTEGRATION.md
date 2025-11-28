# Python Integration with XDL

**Version**: 1.0
**Date**: November 2025
**Status**: Complete ✅
**Feature Flag**: `python` (enabled by default)

---

## Overview

XDL provides seamless integration with Python through the PyO3 library (v0.27), allowing you to:

- Import Python modules
- Call Python functions and methods
- Pass data between XDL and Python
- Use Python's extensive scientific computing ecosystem
- Handle Python objects with automatic reference management

---

## Configuration

### Setting the Python Interpreter

Set the `PYO3_PYTHON` environment variable to your Python executable:

**Windows:**
```bash
set PYO3_PYTHON=C:/Python312/python.exe
cargo build --features python
```

**Linux/macOS:**
```bash
export PYO3_PYTHON=/usr/bin/python3
cargo build --features python
```

**Or in `.cargo/config.toml`:**
```toml
[env]
PYO3_PYTHON = "C:/Python312/python.exe"  # Windows
# PYO3_PYTHON = "/usr/bin/python3"       # Linux/macOS
```

### Supported Python Versions

- Python 3.8 - 3.12 (recommended)
- Python 3.13+ may work but is less tested

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

- Python 3.8 - 3.12 (recommended)
- PyO3 0.27+ for Rust-Python interop
- Standard Python libraries available in your Python installation

---

## Building with Python Support

Python integration is enabled by default. To explicitly enable:

```bash
cargo build --features python
```

To build without Python:

```bash
cargo build --no-default-features
```

---

## Testing

Run the test script to verify Python integration:

```bash
cargo run --bin xdl test_python_integration.xdl
```

---

## Performance Notes

- Python calls have some overhead due to the Python C API
- Complex data structures are handled via object references
- For heavy numerical computing, consider:
  - Native XDL array operations
  - Linfa ML functions (pure Rust)
  - Polars DataFrames (pure Rust)

---

## Architecture

### Thread Safety

The Python manager uses thread-local storage for safe concurrent access:

```
┌─────────────────┐
│   XDL Thread    │
│  ┌───────────┐  │
│  │  Python   │  │
│  │  Manager  │  │
│  └───────────┘  │
│       │         │
│  ┌────▼──────┐  │
│  │  Module   │  │
│  │  Cache    │  │
│  └───────────┘  │
│       │         │
│  ┌────▼──────┐  │
│  │  Object   │  │
│  │  Store    │  │
│  └───────────┘  │
└─────────────────┘
```

### Object Management

Python objects are stored with unique IDs (`pyobj_1`, `pyobj_2`, etc.) allowing:

- Complex Python objects to be passed between XDL functions
- Automatic cleanup when objects go out of scope
- Safe reference counting via PyO3

---

## Complete Example: Scientific Computing

```idl
; ===================================
; Scientific Computing with Python
; ===================================

; Import numpy for numerical operations
np = PYTHON_IMPORT('numpy')

; Create arrays
arr = PYTHON_CALL(np, 'linspace', 0.0, 10.0, 100)
PRINT, 'Array created:', arr

; Calculate statistics
mean_val = PYTHON_CALL(np, 'mean', arr)
std_val = PYTHON_CALL(np, 'std', arr)
PRINT, 'Mean:', mean_val
PRINT, 'Std:', std_val

; Import scipy for advanced functions
scipy = PYTHON_IMPORT('scipy.integrate')

; Define a function to integrate (x^2)
; Note: For complex operations, consider writing Python functions
builtins = PYTHON_IMPORT('builtins')

; Get Python version
sys = PYTHON_IMPORT('sys')
version = PYTHON_CALL(sys, 'version')
PRINT, 'Python version:', version
```

---

## Troubleshooting

### "Python interpreter not found"

Set `PYO3_PYTHON` environment variable:

```bash
# Windows
set PYO3_PYTHON=C:\Python312\python.exe

# Linux/macOS
export PYO3_PYTHON=/usr/bin/python3
```

### "Module not found"

Ensure the module is installed in your Python environment:

```bash
pip install numpy scipy pandas
```

### "Python calls are slow"

For performance-critical operations:

1. Use native XDL functions when available
2. Use Polars DataFrames for data processing
3. Use Linfa ML for machine learning
4. Batch Python operations to reduce call overhead

---

## See Also

- [Polars DataFrame Reference](DATAFRAMES_REFERENCE.md) - Native Rust DataFrames
- [Linfa ML Reference](LINFA_ML_REFERENCE.md) - Native Rust ML
- [ML Complete Reference](ML_COMPLETE_REFERENCE.md) - XDL ML functions
