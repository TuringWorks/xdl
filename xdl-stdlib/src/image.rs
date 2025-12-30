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

/// CANNY - Canny edge detection
/// CANNY(image [, low_threshold, high_threshold])
pub fn canny(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "CANNY: Expected image".to_string(),
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
            "CANNY: Expected 2D image".to_string(),
        ));
    }

    let low_threshold = if args.len() > 1 {
        match &args[1] {
            XdlValue::Double(v) => *v,
            XdlValue::Float(v) => *v as f64,
            _ => 50.0,
        }
    } else {
        50.0
    };

    let high_threshold = if args.len() > 2 {
        match &args[2] {
            XdlValue::Double(v) => *v,
            XdlValue::Float(v) => *v as f64,
            _ => 150.0,
        }
    } else {
        150.0
    };

    let (rows, cols) = (shape[0], shape[1]);

    // Step 1: Gaussian blur
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
    let blurred = match convol_2d(&data, &shape, &kernel, &kernel_shape)? {
        XdlValue::MultiDimArray { data, .. } => data,
        _ => return Err(XdlError::RuntimeError("CANNY: Internal error".to_string())),
    };

    // Step 2: Sobel gradients
    let gx_kernel = [[-1.0, 0.0, 1.0], [-2.0, 0.0, 2.0], [-1.0, 0.0, 1.0]];
    let gy_kernel = [[-1.0, -2.0, -1.0], [0.0, 0.0, 0.0], [1.0, 2.0, 1.0]];

    let mut gradient = vec![0.0; rows * cols];
    let mut direction = vec![0.0; rows * cols];

    for r in 1..rows - 1 {
        for c in 1..cols - 1 {
            let mut gx = 0.0;
            let mut gy = 0.0;
            for i in 0..3 {
                for j in 0..3 {
                    let val = blurred[(r + i - 1) * cols + (c + j - 1)];
                    gx += val * gx_kernel[i][j];
                    gy += val * gy_kernel[i][j];
                }
            }
            gradient[r * cols + c] = (gx * gx + gy * gy).sqrt();
            direction[r * cols + c] = gy.atan2(gx);
        }
    }

    // Step 3: Non-maximum suppression
    let mut suppressed = vec![0.0; rows * cols];
    let pi = std::f64::consts::PI;

    for r in 1..rows - 1 {
        for c in 1..cols - 1 {
            let angle = direction[r * cols + c];
            let g = gradient[r * cols + c];

            // Quantize angle to 4 directions
            let angle_deg = (angle * 180.0 / pi).abs();
            let (n1, n2) = if angle_deg < 22.5 || angle_deg >= 157.5 {
                (gradient[r * cols + c - 1], gradient[r * cols + c + 1])
            } else if angle_deg < 67.5 {
                (
                    gradient[(r - 1) * cols + c + 1],
                    gradient[(r + 1) * cols + c - 1],
                )
            } else if angle_deg < 112.5 {
                (gradient[(r - 1) * cols + c], gradient[(r + 1) * cols + c])
            } else {
                (
                    gradient[(r - 1) * cols + c - 1],
                    gradient[(r + 1) * cols + c + 1],
                )
            };

            if g >= n1 && g >= n2 {
                suppressed[r * cols + c] = g;
            }
        }
    }

    // Step 4: Double threshold and edge tracking
    let mut result = vec![0.0; rows * cols];
    for r in 1..rows - 1 {
        for c in 1..cols - 1 {
            let val = suppressed[r * cols + c];
            if val >= high_threshold {
                result[r * cols + c] = 255.0;
            } else if val >= low_threshold {
                // Check if connected to strong edge
                let mut connected = false;
                for dr in -1..=1 {
                    for dc in -1..=1 {
                        let nr = (r as i32 + dr) as usize;
                        let nc = (c as i32 + dc) as usize;
                        if suppressed[nr * cols + nc] >= high_threshold {
                            connected = true;
                            break;
                        }
                    }
                    if connected {
                        break;
                    }
                }
                if connected {
                    result[r * cols + c] = 255.0;
                }
            }
        }
    }

    Ok(XdlValue::MultiDimArray {
        data: result,
        shape,
    })
}

/// HOUGH - Hough transform for line detection
/// HOUGH(edge_image) returns [accumulator, rho_values, theta_values]
pub fn hough(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "HOUGH: Expected edge image".to_string(),
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
            "HOUGH: Expected 2D image".to_string(),
        ));
    }

    let (rows, cols) = (shape[0], shape[1]);
    let pi = std::f64::consts::PI;

    // Hough parameters
    let num_thetas = 180;
    let diagonal = ((rows * rows + cols * cols) as f64).sqrt();
    let num_rhos = (2.0 * diagonal) as usize + 1;
    let rho_max = diagonal;

    // Create accumulator
    let mut accumulator = vec![0.0; num_rhos * num_thetas];

    // Theta values from -90 to 90 degrees
    let thetas: Vec<f64> = (0..num_thetas)
        .map(|i| (i as f64 - 90.0) * pi / 180.0)
        .collect();

    // Rho values
    let rhos: Vec<f64> = (0..num_rhos)
        .map(|i| i as f64 - rho_max)
        .collect();

    // Accumulate votes
    for r in 0..rows {
        for c in 0..cols {
            if data[r * cols + c] > 0.0 {
                for (t_idx, &theta) in thetas.iter().enumerate() {
                    let rho = c as f64 * theta.cos() + r as f64 * theta.sin();
                    let rho_idx = ((rho + rho_max).round() as usize).min(num_rhos - 1);
                    accumulator[rho_idx * num_thetas + t_idx] += 1.0;
                }
            }
        }
    }

    Ok(XdlValue::NestedArray(vec![
        XdlValue::MultiDimArray {
            data: accumulator,
            shape: vec![num_rhos, num_thetas],
        },
        XdlValue::Array(rhos),
        XdlValue::Array(thetas),
    ]))
}

/// RADON - Radon transform
/// RADON(image [, theta])
pub fn radon(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "RADON: Expected image".to_string(),
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
            "RADON: Expected 2D image".to_string(),
        ));
    }

    let (rows, cols) = (shape[0], shape[1]);
    let pi = std::f64::consts::PI;

    // Default angles: 0 to 179 degrees
    let num_thetas = 180;
    let thetas: Vec<f64> = (0..num_thetas).map(|i| i as f64 * pi / 180.0).collect();

    // Number of projections (rays)
    let diagonal = ((rows * rows + cols * cols) as f64).sqrt();
    let num_r = (2.0 * diagonal).ceil() as usize;

    let center_r = rows as f64 / 2.0;
    let center_c = cols as f64 / 2.0;

    let mut sinogram = vec![0.0; num_r * num_thetas];

    for (t_idx, &theta) in thetas.iter().enumerate() {
        let cos_t = theta.cos();
        let sin_t = theta.sin();

        for r_idx in 0..num_r {
            let r = r_idx as f64 - diagonal;
            let mut sum = 0.0;
            let mut count = 0;

            // Sample along the ray
            for s in -((diagonal as i32))..=((diagonal as i32)) {
                let x = r * cos_t - (s as f64) * sin_t + center_c;
                let y = r * sin_t + (s as f64) * cos_t + center_r;

                let xi = x.round() as i32;
                let yi = y.round() as i32;

                if xi >= 0 && xi < cols as i32 && yi >= 0 && yi < rows as i32 {
                    sum += data[yi as usize * cols + xi as usize];
                    count += 1;
                }
            }

            if count > 0 {
                sinogram[r_idx * num_thetas + t_idx] = sum;
            }
        }
    }

    Ok(XdlValue::MultiDimArray {
        data: sinogram,
        shape: vec![num_r, num_thetas],
    })
}

/// WATERSHED - Watershed segmentation
/// WATERSHED(image [, markers])
pub fn watershed(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "WATERSHED: Expected image".to_string(),
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
            "WATERSHED: Expected 2D image".to_string(),
        ));
    }

    let (rows, cols) = (shape[0], shape[1]);

    // Simple watershed using gradient magnitude as priority
    // Create priority queue based on intensity
    let mut labels: Vec<i32> = vec![0; rows * cols];
    let mut next_label = 1i32;

    // Find local minima as seeds
    for r in 1..rows - 1 {
        for c in 1..cols - 1 {
            let val = data[r * cols + c];
            let mut is_minimum = true;

            for dr in -1..=1 {
                for dc in -1..=1 {
                    if dr == 0 && dc == 0 {
                        continue;
                    }
                    let nr = (r as i32 + dr) as usize;
                    let nc = (c as i32 + dc) as usize;
                    if data[nr * cols + nc] < val {
                        is_minimum = false;
                        break;
                    }
                }
                if !is_minimum {
                    break;
                }
            }

            if is_minimum {
                labels[r * cols + c] = next_label;
                next_label += 1;
            }
        }
    }

    // Propagate labels using flooding
    let mut changed = true;
    while changed {
        changed = false;
        for r in 1..rows - 1 {
            for c in 1..cols - 1 {
                if labels[r * cols + c] != 0 {
                    continue;
                }

                let val = data[r * cols + c];
                let mut best_label = 0i32;
                let mut best_diff = f64::MAX;

                for dr in -1..=1 {
                    for dc in -1..=1 {
                        if dr == 0 && dc == 0 {
                            continue;
                        }
                        let nr = (r as i32 + dr) as usize;
                        let nc = (c as i32 + dc) as usize;
                        let neighbor_label = labels[nr * cols + nc];
                        if neighbor_label > 0 {
                            let diff = (data[nr * cols + nc] - val).abs();
                            if diff < best_diff {
                                best_diff = diff;
                                best_label = neighbor_label;
                            }
                        }
                    }
                }

                if best_label > 0 {
                    labels[r * cols + c] = best_label;
                    changed = true;
                }
            }
        }
    }

    // Convert to f64
    let result: Vec<f64> = labels.iter().map(|&l| l as f64).collect();

    Ok(XdlValue::MultiDimArray {
        data: result,
        shape,
    })
}

/// LABEL_REGION - Connected component labeling
/// LABEL_REGION(binary_image)
pub fn label_region(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "LABEL_REGION: Expected binary image".to_string(),
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
            "LABEL_REGION: Expected 2D image".to_string(),
        ));
    }

    let (rows, cols) = (shape[0], shape[1]);
    let mut labels: Vec<i32> = vec![0; rows * cols];
    let mut next_label = 1i32;

    // Two-pass connected component labeling
    for r in 0..rows {
        for c in 0..cols {
            if data[r * cols + c] > 0.0 && labels[r * cols + c] == 0 {
                // Flood fill from this pixel
                let mut stack = vec![(r, c)];
                labels[r * cols + c] = next_label;

                while let Some((sr, sc)) = stack.pop() {
                    for dr in -1..=1 {
                        for dc in -1..=1 {
                            if dr == 0 && dc == 0 {
                                continue;
                            }
                            let nr = sr as i32 + dr;
                            let nc = sc as i32 + dc;
                            if nr >= 0 && nr < rows as i32 && nc >= 0 && nc < cols as i32 {
                                let idx = nr as usize * cols + nc as usize;
                                if data[idx] > 0.0 && labels[idx] == 0 {
                                    labels[idx] = next_label;
                                    stack.push((nr as usize, nc as usize));
                                }
                            }
                        }
                    }
                }
                next_label += 1;
            }
        }
    }

    let result: Vec<f64> = labels.iter().map(|&l| l as f64).collect();

    Ok(XdlValue::MultiDimArray {
        data: result,
        shape,
    })
}

/// MORPH_OPEN - Morphological opening (erosion followed by dilation)
/// MORPH_OPEN(image [, structuring_element])
pub fn morph_open(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "MORPH_OPEN: Expected image".to_string(),
        ));
    }

    // First erode, then dilate
    let eroded = erode(args)?;
    dilate(&[eroded])
}

/// MORPH_CLOSE - Morphological closing (dilation followed by erosion)
/// MORPH_CLOSE(image [, structuring_element])
pub fn morph_close(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "MORPH_CLOSE: Expected image".to_string(),
        ));
    }

    // First dilate, then erode
    let dilated = dilate(args)?;
    erode(&[dilated])
}

/// HIST_EQUAL - Histogram equalization
/// HIST_EQUAL(image)
pub fn hist_equal(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "HIST_EQUAL: Expected image".to_string(),
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

    let n = data.len();
    if n == 0 {
        return Ok(XdlValue::MultiDimArray { data, shape });
    }

    // Find min and max
    let min_val = data.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_val = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    if (max_val - min_val).abs() < 1e-10 {
        return Ok(XdlValue::MultiDimArray { data, shape });
    }

    // Build histogram (256 bins)
    let num_bins = 256;
    let mut histogram = vec![0usize; num_bins];

    for &val in &data {
        let bin = (((val - min_val) / (max_val - min_val)) * (num_bins - 1) as f64).round() as usize;
        histogram[bin.min(num_bins - 1)] += 1;
    }

    // Build CDF
    let mut cdf = vec![0.0; num_bins];
    cdf[0] = histogram[0] as f64;
    for i in 1..num_bins {
        cdf[i] = cdf[i - 1] + histogram[i] as f64;
    }

    // Normalize CDF
    let cdf_min = cdf.iter().find(|&&x| x > 0.0).copied().unwrap_or(0.0);
    let cdf_max = cdf[num_bins - 1];

    // Apply equalization
    let result: Vec<f64> = data
        .iter()
        .map(|&val| {
            let bin =
                (((val - min_val) / (max_val - min_val)) * (num_bins - 1) as f64).round() as usize;
            let bin = bin.min(num_bins - 1);
            ((cdf[bin] - cdf_min) / (cdf_max - cdf_min)) * (max_val - min_val) + min_val
        })
        .collect();

    Ok(XdlValue::MultiDimArray {
        data: result,
        shape,
    })
}

/// EDGE_DOG - Edge detection using Difference of Gaussians
/// EDGE_DOG(image [, sigma1, sigma2])
pub fn edge_dog(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "EDGE_DOG: Expected image".to_string(),
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
            "EDGE_DOG: Expected 2D image".to_string(),
        ));
    }

    let sigma1 = if args.len() > 1 {
        match &args[1] {
            XdlValue::Double(v) => *v,
            XdlValue::Float(v) => *v as f64,
            _ => 1.0,
        }
    } else {
        1.0
    };

    let sigma2 = if args.len() > 2 {
        match &args[2] {
            XdlValue::Double(v) => *v,
            XdlValue::Float(v) => *v as f64,
            _ => sigma1 * 1.6,
        }
    } else {
        sigma1 * 1.6
    };

    // Create two Gaussian kernels
    fn make_gaussian_kernel(sigma: f64) -> (Vec<f64>, Vec<usize>) {
        let size = (6.0 * sigma).ceil() as usize | 1; // Ensure odd
        let half = size / 2;
        let mut kernel = vec![0.0; size * size];
        let mut sum = 0.0;

        for i in 0..size {
            for j in 0..size {
                let x = i as f64 - half as f64;
                let y = j as f64 - half as f64;
                let val = (-((x * x + y * y) / (2.0 * sigma * sigma))).exp();
                kernel[i * size + j] = val;
                sum += val;
            }
        }

        // Normalize
        for v in &mut kernel {
            *v /= sum;
        }

        (kernel, vec![size, size])
    }

    let (k1, s1) = make_gaussian_kernel(sigma1);
    let (k2, s2) = make_gaussian_kernel(sigma2);

    let blur1 = match convol_2d(&data, &shape, &k1, &s1)? {
        XdlValue::MultiDimArray { data, .. } => data,
        _ => return Err(XdlError::RuntimeError("EDGE_DOG: Internal error".to_string())),
    };

    let blur2 = match convol_2d(&data, &shape, &k2, &s2)? {
        XdlValue::MultiDimArray { data, .. } => data,
        _ => return Err(XdlError::RuntimeError("EDGE_DOG: Internal error".to_string())),
    };

    // Difference
    let result: Vec<f64> = blur1
        .iter()
        .zip(blur2.iter())
        .map(|(&a, &b)| (a - b).abs())
        .collect();

    Ok(XdlValue::MultiDimArray {
        data: result,
        shape,
    })
}

/// LAPLACIAN - Laplacian edge detection
/// LAPLACIAN(image)
pub fn laplacian(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "LAPLACIAN: Expected image".to_string(),
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
            "LAPLACIAN: Expected 2D image".to_string(),
        ));
    }

    // Laplacian kernel
    let kernel = vec![0.0, 1.0, 0.0, 1.0, -4.0, 1.0, 0.0, 1.0, 0.0];
    let kernel_shape = vec![3, 3];

    let result = convol_2d(&data, &shape, &kernel, &kernel_shape)?;

    // Take absolute value
    match result {
        XdlValue::MultiDimArray { data, shape } => {
            let abs_data: Vec<f64> = data.iter().map(|&x| x.abs()).collect();
            Ok(XdlValue::MultiDimArray {
                data: abs_data,
                shape,
            })
        }
        _ => Err(XdlError::RuntimeError(
            "LAPLACIAN: Internal error".to_string(),
        )),
    }
}

/// MEDIAN_2D - 2D Median filter
/// MEDIAN_2D(image [, width])
pub fn median_2d(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "MEDIAN_2D: Expected image".to_string(),
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
            "MEDIAN_2D: Expected 2D image".to_string(),
        ));
    }

    let width = if args.len() > 1 {
        match &args[1] {
            XdlValue::Long(v) => *v as usize,
            XdlValue::Int(v) => *v as usize,
            _ => 3,
        }
    } else {
        3
    };

    let (rows, cols) = (shape[0], shape[1]);
    let half = width / 2;
    let mut result = vec![0.0; rows * cols];

    for r in 0..rows {
        for c in 0..cols {
            let mut window = Vec::new();

            for dr in 0..width {
                for dc in 0..width {
                    let nr = r as i32 + dr as i32 - half as i32;
                    let nc = c as i32 + dc as i32 - half as i32;
                    if nr >= 0 && nr < rows as i32 && nc >= 0 && nc < cols as i32 {
                        window.push(data[nr as usize * cols + nc as usize]);
                    }
                }
            }

            window.sort_by(|a, b| a.partial_cmp(b).unwrap());
            result[r * cols + c] = window[window.len() / 2];
        }
    }

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
