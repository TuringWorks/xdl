# Embedding XDL in Applications

This guide explains how to embed the XDL scientific computing library in applications written in other languages, including Python, JavaScript, C/C++, and more.

## Overview

XDL provides C-compatible Foreign Function Interface (FFI) bindings that allow you to call XDL functions from other programming languages. This enables you to leverage XDL's 235+ mathematical functions, GPU acceleration, and visualization capabilities in your applications.

## Architecture

```text
Your Application (Python/JavaScript/C++)
         ↓
    XDL C API (libxdl_ffi)
         ↓
    XDL Rust Core (xdl-stdlib, xdl-amp, etc.)
```

## Supported Platforms

- **macOS**: Native library with Metal GPU acceleration
- **Linux**: Native library (GPU acceleration via CUDA/ROCm if available)
- **Windows**: Native library (GPU acceleration via CUDA/DirectML)
- **Web**: WebAssembly compilation for browser-based applications

## Quick Start

### 1. Build XDL with FFI Support

```bash
# Clone the repository
git clone https://github.com/TuringWorks/xdl
cd xdl

# Build the FFI library
cargo build --release --package xdl-ffi
```

The compiled library will be in `target/release/`:

- `libxdl_ffi.dylib` (macOS)
- `libxdl_ffi.so` (Linux)
- `xdl_ffi.dll` (Windows)

### 2. Python Example

```python
import ctypes
import os

# Load XDL library
lib = ctypes.CDLL('./libxdl_ffi.dylib')

# Initialize context
lib.xdl_init.restype = ctypes.c_void_p
context = lib.xdl_init()

# Call functions
lib.xdl_call_function.argtypes = [
    ctypes.c_void_p, ctypes.c_char_p,
    ctypes.POINTER(ctypes.c_double), ctypes.c_int
]
lib.xdl_call_function.restype = ctypes.c_double

# Compute sin(1.57)
args = (ctypes.c_double * 1)(1.57)
result = lib.xdl_call_function(context, b"sin", args, 1)
print(f"sin(1.57) = {result}")

# Cleanup
lib.xdl_cleanup(context)
```

## Language-Specific Guides

### .NET/'C#'

#### .NET/C# Setup

1. Build the XDL FFI library as shown above
2. Add the library to your project directory or system PATH
3. Use the provided `XdlWrapper.cs` class or create your own P/Invoke declarations

#### Basic Usage

```csharp
using XdlSharp;

// Static methods (shared context)
double result = XdlMath.Sin(Math.PI / 2);
Console.WriteLine($"sin(π/2) = {result:F4}");

// Instance methods
using (var ctx = new XdlContext())
{
    result = ctx.CallFunction("sqrt", 16.0);
    Console.WriteLine($"sqrt(16) = {result:F4}");
}
```

#### Advanced Usage

```csharp
using XdlSharp;

var ctx = new XdlContext();

// Trigonometric functions
double[] angles = { 0, Math.PI/6, Math.PI/4, Math.PI/3, Math.PI/2 };
foreach (double angle in angles)
{
    double sinVal = ctx.CallFunction("sin", angle);
    double cosVal = ctx.CallFunction("cos", angle);
    Console.WriteLine($"{angle:F4}: sin={sinVal:F4}, cos={cosVal:F4}");
}

// Performance comparison
var stopwatch = System.Diagnostics.Stopwatch.StartNew();
// ... XDL computations ...
stopwatch.Stop();
Console.WriteLine($"XDL time: {stopwatch.ElapsedMilliseconds}ms");
```

### Java

#### Java Setup

1. Build the XDL FFI library as shown above
2. Set `java.library.path` to include the library directory:

   ```bash
   java -Djava.library.path=/path/to/xdl/library YourApp
   ```

3. Use the provided `XdlWrapper.java` class

#### Basic Usage

```java
// Static methods (shared context)
double result = XdlWrapper.Sin(Math.PI / 2);
System.out.printf("sin(π/2) = %.4f%n", result);

// Instance methods
XdlWrapper xdl = new XdlWrapper();
try {
    result = xdl.callFunction("sqrt", 16.0);
    System.out.printf("sqrt(16) = %.4f%n", result);
} finally {
    xdl.close();
}
```

#### Advanced Usage

```java
XdlWrapper xdl = new XdlWrapper();
try {
    // Generate test data
    double[] data = new double[1000];
    Random random = new Random(42);
    for (int i = 0; i < data.length; i++) {
        data[i] = random.nextDouble() * Math.PI * 2;
    }

    // Statistical computations
    double mean = xdl.callFunction("mean", data);
    double stddev = xdl.callFunction("stddev", data);

    System.out.printf("Mean: %.4f%n", mean);
    System.out.printf("StdDev: %.4f%n", stddev);
} finally {
    xdl.close();
}
```

### Go

#### Go Setup

1. Build the XDL FFI library as shown above
2. Ensure the library is in the system library path or current directory
3. Use the provided `xdl.go` package with cgo

#### Basic Usage

```go
package main

import (
    "fmt"
    "math"
    "./xdl"
)

func main() {
    // Global functions (shared context)
    result := xdl.Sin(math.Pi / 2)
    fmt.Printf("sin(π/2) = %.4f\n", result)

    // Context-based usage
    ctx, err := xdl.NewContext()
    if err != nil {
        panic(err)
    }
    defer ctx.Close()

    result = ctx.CallFunction("sqrt", 16.0)
    fmt.Printf("sqrt(16) = %.4f\n", result)
}
```

#### Advanced Usage

```go
ctx, _ := xdl.NewContext()
defer ctx.Close()

// Generate test data
rand.Seed(42)
data := make([]float64, 1000)
for i := range data {
    data[i] = rand.Float64() * math.Pi * 2
}

// Statistical computations
mean := ctx.CallFunction("mean", data...)
stddev := ctx.CallFunction("stddev", data...)

fmt.Printf("Mean: %.4f\n", mean)
fmt.Printf("StdDev: %.4f\n", stddev)

// Performance comparison
start := time.Now()
// ... XDL computations ...
elapsed := time.Since(start)
fmt.Printf("XDL time: %v\n", elapsed)
```

### Python

#### Python Installation

1. Build the XDL FFI library as shown above
2. Copy the library file to your Python project
3. Use the provided wrapper or create your own ctypes interface

#### Using the Python Wrapper

```python
from xdl_wrapper import XdlContext, sin, cos, sqrt

# Using convenience functions
print(f"sin(π/2) = {sin(3.14159/2)}")
print(f"cos(0) = {cos(0)}")
print(f"sqrt(16) = {sqrt(16)}")

# Using context directly
ctx = XdlContext()
result = ctx.call_function("sin", 1.57)
print(f"sin(1.57) = {result}")
```

#### Advanced Python Usage

```python
import numpy as np
from xdl_wrapper import XdlContext

ctx = XdlContext()

# Work with NumPy arrays
data = np.random.normal(0, 1, 1000)
mean_val = ctx.call_function("mean", data.tolist())
print(f"Mean of 1000 random numbers: {mean_val}")

# GPU-accelerated operations (when available)
large_array = np.random.random(1000000)
sum_val = ctx.call_function("total", large_array.tolist())
print(f"Sum of 1M elements: {sum_val}")
```

### JavaScript/WebAssembly

#### Setup

1. Compile XDL to WebAssembly:

```bash
# Install wasm-pack
cargo install wasm-pack

# Build WebAssembly package
wasm-pack build --target web --out-dir pkg xdl-ffi
```

2. Include in your web application:

```html
<script type="module">
    import init, { xdl_init, xdl_call_function, xdl_cleanup } from './pkg/xdl_ffi.js';

    async function run() {
        await init();

        const context = xdl_init();
        const result = xdl_call_function(context, "sin", [1.57], 1);
        console.log(`sin(1.57) = ${result}`);

        xdl_cleanup(context);
    }

    run();
</script>
```

#### WebAssembly Example

See `examples/embedding/javascript/xdl_demo.html` for a complete interactive demo.

### C/C++

#### Direct C API Usage

```c
#include <stdio.h>

// XDL FFI declarations (would be in a header file)
extern void* xdl_init();
extern void xdl_cleanup(void* context);
extern double xdl_call_function(void* context, const char* name,
                               const double* args, int nargs);

int main() {
    // Initialize XDL
    void* context = xdl_init();
    if (!context) {
        fprintf(stderr, "Failed to initialize XDL\n");
        return 1;
    }

    // Call functions
    double args[] = {1.57};
    double result = xdl_call_function(context, "sin", args, 1);
    printf("sin(1.57) = %f\n", result);

    // Cleanup
    xdl_cleanup(context);
    return 0;
}
```

#### CMake Integration

```cmake
# Find XDL
find_library(XDL_FFI_LIBRARY xdl_ffi PATHS /path/to/xdl/target/release)
find_path(XDL_FFI_INCLUDE_DIR xdl_ffi.h PATHS /path/to/xdl/include)

# Link to your application
target_link_libraries(your_app ${XDL_FFI_LIBRARY})
target_include_directories(your_app PRIVATE ${XDL_FFI_INCLUDE_DIR})
```

### Other Languages

#### R

```r
# Load XDL via C interface
dyn.load("libxdl_ffi.so")

# Call functions using .C interface
result <- .C("xdl_call_function",
             context = as.integer(0),  # Would need proper context management
             name = as.character("sin"),
             args = as.double(1.57),
             nargs = as.integer(1),
             result = as.double(0))$result
```

#### Julia

```julia
# Load shared library
const xdl_lib = Libdl.dlopen("libxdl_ffi.so")

# Get function pointers
xdl_init = Libdl.dlsym(xdl_lib, :xdl_init)
xdl_call_function = Libdl.dlsym(xdl_lib, :xdl_call_function)
xdl_cleanup = Libdl.dlsym(xdl_lib, :xdl_cleanup)

# Use functions
context = ccall(xdl_init, Ptr{Cvoid}, ())
result = ccall(xdl_call_function, Cdouble,
               (Ptr{Cvoid}, Cstring, Ptr{Cdouble}, Cint),
               context, "sin", [1.57], 1)
ccall(xdl_cleanup, Cvoid, (Ptr{Cvoid},), context)
```

## Available Functions

All functions listed below are available through the FFI interface and work across all supported languages (Python, JavaScript, .NET/C#, Java, Go, C/C++).

### Mathematical Functions

- `sin(x)` - Sine function
- `cos(x)` - Cosine function
- `tan(x)` - Tangent function
- `asin(x)` - Inverse sine
- `acos(x)` - Inverse cosine
- `atan(x)` - Inverse tangent
- `exp(x)` - Exponential function
- `log(x)` - Natural logarithm
- `log10(x)` - Base-10 logarithm
- `sqrt(x)` - Square root
- `pow(x, y)` - Power function

### Statistical Functions

- `mean(array)` - Arithmetic mean
- `stddev(array)` - Standard deviation
- `variance(array)` - Variance
- `min(array)` - Minimum value
- `max(array)` - Maximum value
- `total(array)` - Sum of elements

### GPU-Accelerated Functions

Functions marked with ⚡ have GPU acceleration available:

- ⚡ `min(array)` - GPU-accelerated minimum
- ⚡ `max(array)` - GPU-accelerated maximum
- ⚡ `mean(array)` - GPU-accelerated mean
- ⚡ `total(array)` - GPU-accelerated sum

## Memory Management

### Context Lifetime

```python
# Python
ctx = XdlContext()  # Creates context
# ... use context ...
# Context automatically cleaned up when object is deleted
```

```javascript
// JavaScript
const context = xdl_init();
// ... use context ...
xdl_cleanup(context);  // Manual cleanup required
```

### Array Handling

```python
# Python - arrays are converted automatically
data = [1, 2, 3, 4, 5]
result = ctx.call_function("mean", data)
```

```c
// C - manual memory management
double data[] = {1, 2, 3, 4, 5};
double result = xdl_call_function(context, "mean", data, 5);
```

## Error Handling

### Python

```python
from xdl_wrapper import XdlContext, XdlError

try:
    ctx = XdlContext()
    result = ctx.call_function("invalid_function", 1.0)
except XdlError as e:
    print(f"XDL error: {e}")
```

### JavaScript

```javascript
try {
    const result = xdl_call_function(context, "sin", [1.57], 1);
    console.log(`Result: ${result}`);
} catch (error) {
    console.error(`XDL error: ${error}`);
}
```

### C

```c
double result = xdl_call_function(context, "sin", args, 1);
if (result == 0.0 && /* check if this is actually an error */) {
    const char* error = xdl_get_last_error();
    fprintf(stderr, "XDL error: %s\n", error);
}
```

## Performance Considerations

### GPU Acceleration

XDL automatically uses GPU acceleration when available:

```python
# Large arrays automatically use GPU acceleration
large_data = list(range(1000000))
result = ctx.call_function("mean", large_data)  # GPU accelerated
```

### Memory Copying

Minimize data transfer between languages:

```python
# Good: Process in batches
for batch in data_batches:
    result = ctx.call_function("process", batch)

# Bad: Transfer entire dataset each time
for item in large_dataset:
    result = ctx.call_function("process", [item])
```

## Building Applications

### Python Applications

```python
# setup.py
from setuptools import setup, find_packages

setup(
    name="my-xdl-app",
    packages=find_packages(),
    package_data={
        "myapp": ["libxdl_ffi.so"],  # Include XDL library
    },
)
```

### Web Applications

```javascript
// webpack.config.js
module.exports = {
    experiments: {
        asyncWebAssembly: true,
    },
    // ... other config
};
```

### Desktop Applications

```rust
// Cargo.toml
[dependencies]
xdl-ffi = { path = "../xdl/xdl-ffi" }

// Or for other languages, link to the compiled library
```

## Examples and Demos

See the `examples/embedding/` directory for complete examples:

- `python/xdl_wrapper.py` - Python ctypes wrapper
- `python/scientific_demo.py` - Scientific computing demo
- `javascript/xdl_demo.html` - WebAssembly browser demo

## Troubleshooting

### Library Not Found

**Error**: `Could not find XDL library`

**Solution**:

- Ensure the library is built: `cargo build --release --package xdl-ffi`
- Check library path and platform-specific naming
- Set `LD_LIBRARY_PATH` (Linux) or `DYLD_LIBRARY_PATH` (macOS)

### Function Not Found

**Error**: `Function X not available`

**Solution**:

- Check function name spelling
- Verify function is implemented in current XDL version
- Some functions may require specific build features

### Memory Issues

**Error**: `Segmentation fault` or memory corruption

**Solution**:

- Ensure proper context lifecycle management
- Check array bounds and data types
- Verify thread safety (XDL contexts are not thread-safe)

### GPU Not Available

**Warning**: `GPU acceleration not available`

**Solution**:

- Check GPU drivers and Metal/CUDA installation
- Falls back to CPU automatically
- Performance will be reduced but functionality preserved

## Contributing

To add new FFI bindings:

1. Add function to `xdl-ffi/src/embedding.rs`
2. Mark with `#[no_mangle]` and `extern "C"`
3. Update language-specific wrappers
4. Add tests and documentation

## License

XDL embedding follows the same MIT license as the main project.</content>
</xai:function_call name="write">
< parameter name="filePath">examples/embedding/python/scientific_demo.py
