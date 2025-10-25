//! Image processing functions
use xdl_core::{XdlError, XdlResult, XdlValue};

/// CONVOL - Perform convolution on an array
/// Syntax: result = CONVOL(array, kernel [, /CENTER] [, /EDGE_TRUNCATE])
///
/// Parameters:
/// - array: Input array (1D or 2D)
/// - kernel: Convolution kernel
/// - CENTER: If set (default=1), center the kernel over each data point
/// - EDGE_TRUNCATE: If set, truncate at edges instead of wrapping
pub fn convol(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "CONVOL: Expected at least 2 arguments (array, kernel)".to_string(),
        ));
    }

    // Extract array and kernel
    let (array_data, array_shape) = match &args[0] {
        XdlValue::Array(data) => (data.clone(), vec![data.len()]),
        XdlValue::MultiDimArray { data, shape } => (data.clone(), shape.clone()),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let (kernel_data, kernel_shape) = match &args[1] {
        XdlValue::Array(data) => (data.clone(), vec![data.len()]),
        XdlValue::MultiDimArray { data, shape } => (data.clone(), shape.clone()),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    // Handle 1D convolution
    if array_shape.len() == 1 {
        return convol_1d(&array_data, &kernel_data);
    }

    // Handle 2D convolution
    if array_shape.len() == 2 && kernel_shape.len() == 2 {
        return convol_2d(&array_data, &array_shape, &kernel_data, &kernel_shape);
    }

    Err(XdlError::InvalidArgument(
        "CONVOL: Only 1D and 2D arrays are currently supported".to_string(),
    ))
}

/// Perform 1D convolution
fn convol_1d(array: &[f64], kernel: &[f64]) -> XdlResult<XdlValue> {
    let array_len = array.len();
    let kernel_len = kernel.len();
    let kernel_half = kernel_len / 2;

    let mut result = vec![0.0; array_len];

    for (i, item) in result.iter_mut().enumerate().take(array_len) {
        let mut sum = 0.0;
        for (k, &kernel_val) in kernel.iter().enumerate().take(kernel_len) {
            let idx = i as i32 + k as i32 - kernel_half as i32;
            if idx >= 0 && idx < array_len as i32 {
                sum += array[idx as usize] * kernel_val;
            }
        }
        *item = sum;
    }

    Ok(XdlValue::Array(result))
}

/// Perform 2D convolution
fn convol_2d(
    array: &[f64],
    array_shape: &[usize],
    kernel: &[f64],
    kernel_shape: &[usize],
) -> XdlResult<XdlValue> {
    let rows = array_shape[0];
    let cols = array_shape[1];
    let kernel_rows = kernel_shape[0];
    let kernel_cols = kernel_shape[1];
    let kernel_row_half = kernel_rows / 2;
    let kernel_col_half = kernel_cols / 2;

    let mut result = vec![0.0; rows * cols];

    for r in 0..rows {
        for c in 0..cols {
            let mut sum = 0.0;

            // Apply kernel
            for kr in 0..kernel_rows {
                for kc in 0..kernel_cols {
                    // Calculate array position
                    let ar = r as i32 + kr as i32 - kernel_row_half as i32;
                    let ac = c as i32 + kc as i32 - kernel_col_half as i32;

                    // Check bounds (edge handling: truncate)
                    if ar >= 0 && ar < rows as i32 && ac >= 0 && ac < cols as i32 {
                        let array_idx = ar as usize * cols + ac as usize;
                        let kernel_idx = kr * kernel_cols + kc;
                        sum += array[array_idx] * kernel[kernel_idx];
                    }
                }
            }

            result[r * cols + c] = sum;
        }
    }

    Ok(XdlValue::MultiDimArray {
        data: result,
        shape: array_shape.to_vec(),
    })
}

/// DILATE - Morphological dilation
pub fn dilate(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "DILATE: Expected image".to_string(),
        ));
    }
    let (data, shape) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => (data.clone(), shape.clone()),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "image".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };
    if shape.len() != 2 {
        return Err(XdlError::DimensionError(
            "DILATE: Expected 2D image".to_string(),
        ));
    }
    let (rows, cols) = (shape[0], shape[1]);
    let mut result = vec![0.0; rows * cols];
    for r in 0..rows {
        for c in 0..cols {
            let mut max_val = data[r * cols + c];
            for dr in -1..=1 {
                for dc in -1..=1 {
                    let nr = r as i32 + dr;
                    let nc = c as i32 + dc;
                    if nr >= 0 && nr < rows as i32 && nc >= 0 && nc < cols as i32 {
                        let val = data[nr as usize * cols + nc as usize];
                        if val > max_val {
                            max_val = val;
                        }
                    }
                }
            }
            result[r * cols + c] = max_val;
        }
    }
    Ok(XdlValue::MultiDimArray {
        data: result,
        shape,
    })
}

/// ERODE - Morphological erosion
pub fn erode(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "ERODE: Expected image".to_string(),
        ));
    }
    let (data, shape) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => (data.clone(), shape.clone()),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "image".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };
    if shape.len() != 2 {
        return Err(XdlError::DimensionError(
            "ERODE: Expected 2D image".to_string(),
        ));
    }
    let (rows, cols) = (shape[0], shape[1]);
    let mut result = vec![0.0; rows * cols];
    for r in 0..rows {
        for c in 0..cols {
            let mut min_val = data[r * cols + c];
            for dr in -1..=1 {
                for dc in -1..=1 {
                    let nr = r as i32 + dr;
                    let nc = c as i32 + dc;
                    if nr >= 0 && nr < rows as i32 && nc >= 0 && nc < cols as i32 {
                        let val = data[nr as usize * cols + nc as usize];
                        if val < min_val {
                            min_val = val;
                        }
                    }
                }
            }
            result[r * cols + c] = min_val;
        }
    }
    Ok(XdlValue::MultiDimArray {
        data: result,
        shape,
    })
}

/// SOBEL - Sobel edge detection
pub fn sobel(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "SOBEL: Expected image".to_string(),
        ));
    }
    let (data, shape) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => (data.clone(), shape.clone()),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "image".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };
    if shape.len() != 2 {
        return Err(XdlError::DimensionError(
            "SOBEL: Expected 2D image".to_string(),
        ));
    }
    let (rows, cols) = (shape[0], shape[1]);
    let mut result = vec![0.0; rows * cols];
    // Sobel kernels
    let gx = [[-1.0, 0.0, 1.0], [-2.0, 0.0, 2.0], [-1.0, 0.0, 1.0]];
    let gy = [[-1.0, -2.0, -1.0], [0.0, 0.0, 0.0], [1.0, 2.0, 1.0]];
    for r in 1..rows - 1 {
        for c in 1..cols - 1 {
            let mut sum_x = 0.0;
            let mut sum_y = 0.0;
            for i in 0..3 {
                for j in 0..3 {
                    let val = data[(r + i - 1) * cols + (c + j - 1)];
                    sum_x += val * gx[i][j];
                    sum_y += val * gy[i][j];
                }
            }
            result[r * cols + c] = (sum_x * sum_x + sum_y * sum_y).sqrt();
        }
    }
    Ok(XdlValue::MultiDimArray {
        data: result,
        shape,
    })
}

/// ROBERTS - Roberts cross edge detection
pub fn roberts(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "ROBERTS: Expected image".to_string(),
        ));
    }
    let (data, shape) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => (data.clone(), shape.clone()),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "image".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };
    if shape.len() != 2 {
        return Err(XdlError::DimensionError(
            "ROBERTS: Expected 2D image".to_string(),
        ));
    }
    let (rows, cols) = (shape[0], shape[1]);
    let mut result = vec![0.0; rows * cols];

    // Roberts cross kernels
    for r in 0..rows - 1 {
        for c in 0..cols - 1 {
            let gx = data[r * cols + c] - data[(r + 1) * cols + (c + 1)];
            let gy = data[r * cols + (c + 1)] - data[(r + 1) * cols + c];
            result[r * cols + c] = (gx * gx + gy * gy).sqrt();
        }
    }
    Ok(XdlValue::MultiDimArray {
        data: result,
        shape,
    })
}

/// PREWITT - Prewitt edge detection
pub fn prewitt(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "PREWITT: Expected image".to_string(),
        ));
    }
    let (data, shape) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => (data.clone(), shape.clone()),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "image".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };
    if shape.len() != 2 {
        return Err(XdlError::DimensionError(
            "PREWITT: Expected 2D image".to_string(),
        ));
    }
    let (rows, cols) = (shape[0], shape[1]);
    let mut result = vec![0.0; rows * cols];

    // Prewitt kernels
    let gx = [[-1.0, 0.0, 1.0], [-1.0, 0.0, 1.0], [-1.0, 0.0, 1.0]];
    let gy = [[-1.0, -1.0, -1.0], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]];

    for r in 1..rows - 1 {
        for c in 1..cols - 1 {
            let mut sum_x = 0.0;
            let mut sum_y = 0.0;
            for i in 0..3 {
                for j in 0..3 {
                    let val = data[(r + i - 1) * cols + (c + j - 1)];
                    sum_x += val * gx[i][j];
                    sum_y += val * gy[i][j];
                }
            }
            result[r * cols + c] = (sum_x * sum_x + sum_y * sum_y).sqrt();
        }
    }
    Ok(XdlValue::MultiDimArray {
        data: result,
        shape,
    })
}

/// GAUSSIAN_FILTER - Apply Gaussian blur
pub fn gaussian_filter(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "GAUSSIAN_FILTER: Expected image".to_string(),
        ));
    }
    let (data, shape) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => (data.clone(), shape.clone()),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "image".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };
    if shape.len() != 2 {
        return Err(XdlError::DimensionError(
            "GAUSSIAN_FILTER: Expected 2D image".to_string(),
        ));
    }

    // Simple 3x3 Gaussian kernel
    let kernel = vec![
        1.0 / 16.0,
        2.0 / 16.0,
        1.0 / 16.0,
        2.0 / 16.0,
        4.0 / 16.0,
        2.0 / 16.0,
        1.0 / 16.0,
        2.0 / 16.0,
        1.0 / 16.0,
    ];
    let kernel_shape = vec![3, 3];

    convol_2d(&data, &shape, &kernel, &kernel_shape)
}

/// THRESHOLD - Binary threshold
pub fn threshold(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "THRESHOLD: Expected image and threshold value".to_string(),
        ));
    }
    let (data, shape) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => (data.clone(), shape.clone()),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "image".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let threshold_val = match &args[1] {
        XdlValue::Double(v) => *v,
        XdlValue::Float(v) => *v as f64,
        XdlValue::Long(v) => *v as f64,
        XdlValue::Int(v) => *v as f64,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let result: Vec<f64> = data
        .iter()
        .map(|&x| if x >= threshold_val { 1.0 } else { 0.0 })
        .collect();

    Ok(XdlValue::MultiDimArray {
        data: result,
        shape,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convol_1d() {
        let array = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let kernel = vec![1.0, 1.0, 1.0]; // Moving average kernel

        let result = convol_1d(&array, &kernel).unwrap();
        if let XdlValue::Array(data) = result {
            // Check that it's a valid convolution result
            assert_eq!(data.len(), 5);
        } else {
            panic!("Expected Array result");
        }
    }

    #[test]
    fn test_convol_2d() {
        // Simple 3x3 array
        let array = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let array_shape = vec![3, 3];

        // 3x3 averaging kernel
        let kernel = vec![1.0 / 9.0; 9];
        let kernel_shape = vec![3, 3];

        let result = convol_2d(&array, &array_shape, &kernel, &kernel_shape).unwrap();
        if let XdlValue::MultiDimArray { data, shape } = result {
            assert_eq!(shape, vec![3, 3]);
            assert_eq!(data.len(), 9);
        } else {
            panic!("Expected MultiDimArray result");
        }
    }
}
