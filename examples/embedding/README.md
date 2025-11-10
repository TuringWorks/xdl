# XDL Embedding Examples

This directory contains examples of how to embed XDL in applications written in other programming languages.

## Overview

XDL provides C-compatible FFI (Foreign Function Interface) bindings that allow you to use XDL's scientific computing capabilities from:

- **Python** (via ctypes)
- **JavaScript** (via WebAssembly)
- **C/C++** (direct API usage)
- **Other languages** (Julia, R, etc.)

## Directory Structure

```
embedding/
├── README.md              # This file
├── python/
│   ├── xdl_wrapper.py     # Python ctypes wrapper
│   └── scientific_demo.py # Scientific computing examples
├── javascript/
│   └── xdl_demo.html      # WebAssembly browser demo
└── c/
    └── simple_example.c   # Direct C API usage
```

## Prerequisites

### Build XDL FFI Library

```bash
# From the XDL project root
cargo build --release --package xdl-ffi
```

This creates the shared library:
- `target/release/libxdl_ffi.dylib` (macOS)
- `target/release/libxdl_ffi.so` (Linux)
- `target/release/xdl_ffi.dll` (Windows)

### Python Requirements

```bash
pip install numpy  # For array operations
```

### JavaScript Requirements

Modern browser with WebAssembly support, or:
```bash
npm install wasm-pack  # For building WebAssembly modules
```

## Python Examples

### Basic Usage

```python
from xdl_wrapper import sin, cos, XdlContext

# Simple function calls
print(f"sin(1.57) = {sin(1.57)}")

# Context-based usage
ctx = XdlContext()
result = ctx.call_function("sqrt", 16.0)
print(f"sqrt(16) = {result}")
```

### Scientific Computing

```python
import numpy as np
from xdl_wrapper import XdlContext

ctx = XdlContext()

# Generate data
x = np.linspace(0, 2*np.pi, 1000)
y = np.sin(x) + 0.1 * np.random.normal(size=1000)

# Use XDL for computations
mean_val = ctx.call_function("mean", y.tolist())
std_val = ctx.call_function("stddev", y.tolist())

print(f"Mean: {mean_val:.4f}")
print(f"StdDev: {std_val:.4f}")
```

## JavaScript Examples

### Browser-Based Demo

Open `javascript/xdl_demo.html` in a modern web browser to see:
- Mathematical function calls
- Array operations
- Performance comparisons
- Interactive demonstrations

### WebAssembly Integration

```javascript
// Load XDL WebAssembly module
import init, { xdl_init, xdl_call_function } from './xdl_ffi.js';

async function main() {
    // Initialize WebAssembly
    await init();

    // Create XDL context
    const context = xdl_init();

    // Call functions
    const result = xdl_call_function(context, "sin", [1.57], 1);
    console.log(`sin(1.57) = ${result}`);
}
```

## C/C++ Examples

### Direct API Usage

```c
#include <stdio.h>

// Link with -lxdl_ffi
extern void* xdl_init();
extern double xdl_call_function(void* ctx, const char* name,
                               const double* args, int nargs);
extern void xdl_cleanup(void* ctx);

int main() {
    void* context = xdl_init();
    double args[] = {1.57};
    double result = xdl_call_function(context, "sin", args, 1);
    printf("sin(1.57) = %f\n", result);
    xdl_cleanup(context);
    return 0;
}
```

## Performance Notes

### GPU Acceleration

XDL automatically uses GPU acceleration for supported operations:

```python
# Large arrays benefit from GPU acceleration
large_data = list(range(1000000))
result = ctx.call_function("mean", large_data)  # GPU accelerated on macOS
```

### Memory Management

- **Python**: Automatic cleanup via context manager
- **JavaScript**: Manual cleanup required
- **C/C++**: Manual memory management

## Available Functions

### Mathematics
- `sin`, `cos`, `tan` - Trigonometric functions
- `exp`, `log`, `sqrt` - Exponential and power functions
- `min`, `max`, `mean`, `total` - Array reductions (GPU accelerated)

### Statistics
- `stddev`, `variance` - Dispersion measures
- `median` - Central tendency

### Arrays
- Array creation and manipulation
- Multi-dimensional array support

## Building Custom Applications

### Python Package

```python
# setup.py
from setuptools import setup

setup(
    name="my-xdl-app",
    packages=["myapp"],
    package_data={
        "myapp": ["libxdl_ffi.so"],  # Include library
    },
)
```

### Web Application

```html
<!DOCTYPE html>
<script type="module">
    import init, * as xdl from './pkg/xdl_ffi.js';

    async function run() {
        await init();
        const ctx = xdl.xdl_init();
        // Use XDL functions...
    }
    run();
</script>
```

## Troubleshooting

### Library Not Found
- Ensure XDL is built: `cargo build --release --package xdl-ffi`
- Check library path and naming conventions
- Set appropriate library path environment variables

### Function Errors
- Verify function names and signatures
- Check that required features are enabled in XDL build

### Performance Issues
- GPU acceleration requires compatible hardware/drivers
- Falls back to CPU automatically

## Contributing

To add new language bindings:

1. Create wrapper for the C API
2. Add examples and tests
3. Update this documentation
4. Submit a pull request

## See Also

- [Embedding Guide](../docs/EMBEDDING_GUIDE.md) - Comprehensive documentation
- [XDL FFI API](../xdl-ffi/src/embedding.rs) - C API implementation
- [Main Documentation](../docs/) - General XDL documentation</content>
</xai:function_call name="write">
<parameter name="filePath">examples/embedding/c/simple_example.c
