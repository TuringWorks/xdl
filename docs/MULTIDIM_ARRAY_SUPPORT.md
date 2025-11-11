# Multi-Dimensional Array Support in XDL

**Date**: January 22, 2025  
**Status**: ✅ Implemented

---

## Overview

XDL now has native support for multi-dimensional arrays with explicit shape tracking. This enables efficient implementation of:
- 2D Convolution (Conv2D)
- Image processing
- Matrix operations
- Full LSTM implementation
- Batch processing

---

## Implementation

### New XdlValue Variant

```rust
XdlValue::MultiDimArray {
    data: Vec<f64>,      // Flattened data in row-major order
    shape: Vec<usize>,   // Dimensions: [rows, cols] or [depth, rows, cols]
}
```

### Memory Layout

**Row-Major Order** (C-style):
- 2D Array [rows, cols]: `data[i * cols + j]` accesses element at (i, j)
- 3D Array [depth, rows, cols]: `data[d * rows * cols + i * cols + j]`

---

## API Methods

### Creating Multi-Dimensional Arrays

```rust
// From data and shape
let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
let shape = vec![2, 3];  // 2 rows, 3 columns
let array = XdlValue::from_multidim(data, shape)?;
```

### Accessing Array Properties

```rust
// Get shape
let shape = array.shape();  // Some(vec![2, 3])

// Get data slice
let data = array.as_slice();  // Some(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0])

// Get total elements
let n = array.n_elements();  // 6
```

### Display

```rust
// Small arrays show all data
Array[2x3]: [1.000, 2.000, 3.000, 4.000, 5.000, 6.000]

// Large arrays show abbreviated
Array[100x100]: [1.234, 5.678, ..., 9.012] (10000)
```

---

## Shape Conventions

### 2D Arrays (Matrices)
`shape = [rows, cols]`

```
Example: 3x4 matrix
shape = [3, 4]
data = [a₀₀, a₀₁, a₀₂, a₀₃,  // row 0
        a₁₀, a₁₁, a₁₂, a₁₃,  // row 1  
        a₂₀, a₂₁, a₂₂, a₂₃]  // row 2
```

### 3D Arrays (Tensors)
`shape = [depth, rows, cols]`

```
Example: 2x3x4 tensor (2 matrices of 3x4)
shape = [2, 3, 4]
Total elements: 24
```

### 4D Arrays (Batches)
`shape = [batch, channels, height, width]`

```
Example: 8x3x32x32 (8 RGB images of 32x32)
shape = [8, 3, 32, 32]
Total elements: 24,576
```

---

## Indexing

### 2D Indexing

```rust
fn get_2d(data: &[f64], shape: &[usize], row: usize, col: usize) -> f64 {
    let cols = shape[1];
    data[row * cols + col]
}

fn set_2d(data: &mut [f64], shape: &[usize], row: usize, col: usize, val: f64) {
    let cols = shape[1];
    data[row * cols + col] = val;
}
```

### 3D Indexing

```rust
fn get_3d(data: &[f64], shape: &[usize], d: usize, row: usize, col: usize) -> f64 {
    let rows = shape[1];
    let cols = shape[2];
    data[d * rows * cols + row * cols + col]
}
```

---

## Use Cases

### 1. Conv2D Implementation

```rust
// Input: [height, width]
// Kernel: [k_h, k_w]
// Output: [out_h, out_w]

let input_shape = vec![28, 28];
let kernel_shape = vec![3, 3];
let output_shape = vec![26, 26];  // with valid padding

let input = XdlValue::from_multidim(input_data, input_shape)?;
let kernel = XdlValue::from_multidim(kernel_data, kernel_shape)?;
```

### 2. LSTM State Tensors

```rust
// Hidden state: [batch, hidden_size]
// Cell state: [batch, hidden_size]
// Input: [batch, seq_len, input_size]

let batch_size = 32;
let hidden_size = 128;
let seq_len = 10;

let h_shape = vec![batch_size, hidden_size];
let input_shape = vec![batch_size, seq_len, input_size];
```

### 3. Image Batches

```rust
// RGB images: [batch, channels, height, width]
let images_shape = vec![16, 3, 64, 64];  // 16 RGB 64x64 images
let images = XdlValue::from_multidim(image_data, images_shape)?;
```

---

## Validation

Arrays are validated on creation:

```rust
let data = vec![1.0, 2.0, 3.0];
let shape = vec![2, 2];  // Claims 4 elements

// ERROR: Data size 3 does not match shape [2, 2] (expected 4)
XdlValue::from_multidim(data, shape)?;  // Returns error
```

---

## Integration with Existing Functions

### Array Type Compatibility

```rust
// 1D arrays remain backward compatible
XdlValue::Array(vec![1.0, 2.0, 3.0])

// Multi-dim arrays for advanced operations
XdlValue::MultiDimArray {
    data: vec![1.0, 2.0, 3.0, 4.0],
    shape: vec![2, 2]
}
```

### Conversion Functions

```rust
// Get shape (works for both 1D and multi-dim)
let shape = value.shape();  // Some(vec![...])

// Get data slice
let data = value.as_slice();  // Some(&[...])

// Get element count
let n = value.n_elements();
```

---

## Performance Considerations

### Memory

- **Contiguous Storage**: All data in single `Vec<f64>`
- **Zero Copy**: Shape is metadata, data stays in place
- **Cache Friendly**: Row-major layout improves locality

### Complexity

- **Create**: O(1) - just validates size
- **Index**: O(1) - simple arithmetic
- **Clone**: O(n) - copies data vector

---

## Examples

### Creating a 2D Matrix

```rust
// 2x3 matrix
let data = vec![
    1.0, 2.0, 3.0,  // row 0
    4.0, 5.0, 6.0   // row 1
];
let matrix = XdlValue::from_multidim(data, vec![2, 3])?;
```

### Creating a 3D Tensor

```rust
// 2x2x2 cube
let data = vec![
    1.0, 2.0,  // depth 0, row 0
    3.0, 4.0,  // depth 0, row 1
    5.0, 6.0,  // depth 1, row 0
    7.0, 8.0   // depth 1, row 1
];
let tensor = XdlValue::from_multidim(data, vec![2, 2, 2])?;
```

### Reshaping (Conceptual)

```rust
// Start with 1D
let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];

// View as 2x3
let matrix = XdlValue::from_multidim(data.clone(), vec![2, 3])?;

// View as 3x2
let transposed = XdlValue::from_multidim(data, vec![3, 2])?;
```

---

## Future Enhancements

### Potential Additions

1. **Stride Support**: Non-contiguous views
2. **Transposition**: Efficient transpose without copying
3. **Broadcasting**: NumPy-style dimension broadcasting
4. **Slicing**: Extract sub-arrays efficiently
5. **Type Generic**: Support for `Vec<T>` beyond `f64`

### Advanced Operations

```rust
// Matrix multiplication (future)
fn matmul(a: &XdlValue, b: &XdlValue) -> XdlResult<XdlValue>

// Transpose (future)
fn transpose(arr: &XdlValue) -> XdlResult<XdlValue>

// Reshape (future)
fn reshape(arr: &XdlValue, new_shape: Vec<usize>) -> XdlResult<XdlValue>
```

---

## Compatibility

### Backward Compatibility

✅ Existing 1D `Array` type unchanged  
✅ All existing functions still work  
✅ New functions can use `MultiDimArray`  
✅ Helper methods work with both types

### Forward Compatibility

The design allows easy extension to:
- Different data types (int, complex)
- Strided arrays
- Memory-mapped arrays
- GPU tensors (future)

---

## Status

✅ **Implemented**: Core multi-dimensional array type  
✅ **Tested**: Compiles successfully  
✅ **Documented**: Complete API reference  
🚧 **Next**: Implement Conv2D and LSTM using new type

---

## Related Functions Ready to Implement

With multi-dimensional support, we can now implement:

1. **XDLML_Conv2D** - 2D Convolution
2. **XDLML_MaxPooling2D** - 2D Max Pooling
3. **XDLML_LSTM** - Full LSTM with gates
4. **XDLML_MatMul** - Matrix multiplication
5. **XDLML_Reshape** - Array reshaping
6. **XDLML_Transpose** - Matrix transpose

---

*Multi-dimensional array support implemented: January 22, 2025*  
*Ready for Conv2D and LSTM implementation*  
*Build status: ✅ PASSING*
