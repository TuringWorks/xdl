//! Array manipulation functions

use xdl_core::{XdlError, XdlResult, XdlValue};

pub struct ArrayFunctions;

impl ArrayFunctions {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ArrayFunctions {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to extract dimension size from XdlValue
fn extract_dimension(val: &XdlValue) -> XdlResult<usize> {
    match val {
        XdlValue::Long(n) => {
            if *n < 0 {
                return Err(XdlError::InvalidArgument(
                    "Array dimensions must be non-negative".to_string(),
                ));
            }
            Ok(*n as usize)
        }
        XdlValue::Int(n) => {
            if *n < 0 {
                return Err(XdlError::InvalidArgument(
                    "Array dimensions must be non-negative".to_string(),
                ));
            }
            Ok(*n as usize)
        }
        XdlValue::Byte(n) => Ok(*n as usize),
        XdlValue::Double(n) => {
            if *n < 0.0 {
                return Err(XdlError::InvalidArgument(
                    "Array dimensions must be non-negative".to_string(),
                ));
            }
            Ok(*n as usize)
        }
        XdlValue::Float(n) => {
            if *n < 0.0 {
                return Err(XdlError::InvalidArgument(
                    "Array dimensions must be non-negative".to_string(),
                ));
            }
            Ok(*n as usize)
        }
        _ => Err(XdlError::TypeMismatch {
            expected: "integer".to_string(),
            actual: format!("{:?}", val.gdl_type()),
        }),
    }
}

/// Calculate total size from dimension arguments
fn calculate_total_size(args: &[XdlValue]) -> XdlResult<usize> {
    let mut total_size = 1usize;
    for arg in args {
        let dim = extract_dimension(arg)?;
        total_size = total_size
            .checked_mul(dim)
            .ok_or_else(|| XdlError::InvalidArgument("Array size too large".to_string()))?;
    }
    Ok(total_size)
}

/// Create byte array: BYTARR(dimensions...)
/// Returns array of bytes (u8) initialized to zero
pub fn bytarr(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "BYTARR: At least one dimension required".to_string(),
        ));
    }

    // Extract dimensions
    let mut shape = Vec::new();
    for arg in args {
        shape.push(extract_dimension(arg)?);
    }

    let total_size = calculate_total_size(args)?;
    let data = vec![0.0; total_size];

    // If multi-dimensional, create MultiDimArray with shape
    if shape.len() > 1 {
        Ok(XdlValue::MultiDimArray { data, shape })
    } else {
        Ok(XdlValue::Array(data))
    }
}

/// Create integer array: INTARR(dimensions...)
/// Returns array of 16-bit integers initialized to zero
pub fn intarr(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "INTARR: At least one dimension required".to_string(),
        ));
    }

    // Extract dimensions
    let mut shape = Vec::new();
    for arg in args {
        shape.push(extract_dimension(arg)?);
    }

    let total_size = calculate_total_size(args)?;
    let data = vec![0.0; total_size];

    // If multi-dimensional, create MultiDimArray with shape
    if shape.len() > 1 {
        Ok(XdlValue::MultiDimArray { data, shape })
    } else {
        Ok(XdlValue::Array(data))
    }
}

/// Create long integer array: LONARR(dimensions...)
/// Returns array of 32-bit integers initialized to zero
pub fn lonarr(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "LONARR: At least one dimension required".to_string(),
        ));
    }

    // Extract dimensions
    let mut shape = Vec::new();
    for arg in args {
        shape.push(extract_dimension(arg)?);
    }

    let total_size = calculate_total_size(args)?;
    let data = vec![0.0; total_size];

    // If multi-dimensional, create MultiDimArray with shape
    if shape.len() > 1 {
        Ok(XdlValue::MultiDimArray { data, shape })
    } else {
        Ok(XdlValue::Array(data))
    }
}

/// Create floating-point array: FLTARR(dimensions...)
/// Returns array of 32-bit floats initialized to zero
pub fn fltarr(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "FLTARR: At least one dimension required".to_string(),
        ));
    }

    // Extract dimensions
    let mut shape = Vec::new();
    for arg in args {
        shape.push(extract_dimension(arg)?);
    }

    let total_size = calculate_total_size(args)?;
    let data = vec![0.0; total_size];

    // If multi-dimensional, create MultiDimArray with shape
    if shape.len() > 1 {
        Ok(XdlValue::MultiDimArray { data, shape })
    } else {
        Ok(XdlValue::Array(data))
    }
}

/// Create double precision array: DBLARR(dimensions...)
/// Returns array of 64-bit floats initialized to zero
pub fn dblarr(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "DBLARR: At least one dimension required".to_string(),
        ));
    }

    // Extract dimensions
    let mut shape = Vec::new();
    for arg in args {
        shape.push(extract_dimension(arg)?);
    }

    let total_size = calculate_total_size(args)?;
    let data = vec![0.0; total_size];

    // If multi-dimensional, create MultiDimArray with shape
    if shape.len() > 1 {
        Ok(XdlValue::MultiDimArray { data, shape })
    } else {
        Ok(XdlValue::Array(data))
    }
}

/// Create string array: STRARR(dimensions...)
/// Returns array of empty strings
/// Note: Currently returns numeric array as strings not yet fully supported
pub fn strarr(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "STRARR: At least one dimension required".to_string(),
        ));
    }

    let total_size = calculate_total_size(args)?;
    // For now, represent as array of zeros (strings not fully supported yet)
    // TODO: Add proper string array support to XdlValue
    let data = vec![0.0; total_size];
    Ok(XdlValue::Array(data))
}

/// Get number of elements in array: N_ELEMENTS(array)
pub fn n_elements(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "N_ELEMENTS: Expected 1 argument, got {}",
            args.len()
        )));
    }

    match &args[0] {
        XdlValue::Undefined => Ok(XdlValue::Long(0)),
        XdlValue::Array(arr) => Ok(XdlValue::Long(arr.len() as i32)),
        _ => Ok(XdlValue::Long(1)), // Scalar values have 1 element
    }
}

/// Find array elements matching condition: WHERE(array_expression)
/// Returns array of indices where condition is true (non-zero)
/// If no elements match, returns -1 (scalar)
///
/// Examples:
///   arr = [1, 5, 3, 8, 2, 9]
///   WHERE(arr GT 4)  ; Returns [1, 3, 5] (indices where arr > 4)
///   WHERE(arr EQ 0)  ; Returns -1 (no matches)
pub fn where_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() || args.len() > 2 {
        return Err(XdlError::InvalidArgument(format!(
            "WHERE: Expected 1-2 arguments, got {}",
            args.len()
        )));
    }

    // Get the input array (boolean/numeric)
    let input_arr = match &args[0] {
        XdlValue::Array(arr) => arr,
        XdlValue::Long(n) => {
            // Scalar: return 0 if non-zero, -1 if zero
            return if *n != 0 {
                Ok(XdlValue::Long(0))
            } else {
                Ok(XdlValue::Long(-1))
            };
        }
        XdlValue::Int(n) => {
            return if *n != 0 {
                Ok(XdlValue::Long(0))
            } else {
                Ok(XdlValue::Long(-1))
            };
        }
        XdlValue::Byte(n) => {
            return if *n != 0 {
                Ok(XdlValue::Long(0))
            } else {
                Ok(XdlValue::Long(-1))
            };
        }
        XdlValue::Double(n) => {
            return if *n != 0.0 {
                Ok(XdlValue::Long(0))
            } else {
                Ok(XdlValue::Long(-1))
            };
        }
        XdlValue::Float(n) => {
            return if *n != 0.0 {
                Ok(XdlValue::Long(0))
            } else {
                Ok(XdlValue::Long(-1))
            };
        }
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric array or boolean array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    // Find all indices where value is non-zero (true)
    let mut indices = Vec::new();
    for (i, &val) in input_arr.iter().enumerate() {
        // In GDL/IDL, any non-zero value is considered "true"
        if val != 0.0 {
            indices.push(i as f64);
        }
    }

    // Return results
    if indices.is_empty() {
        // No matches found - return -1 (IDL convention)
        Ok(XdlValue::Long(-1))
    } else if indices.len() == 1 {
        // Single match - return as scalar Long (IDL behavior)
        Ok(XdlValue::Long(indices[0] as i32))
    } else {
        // Multiple matches - return as array
        Ok(XdlValue::Array(indices))
    }
}

/// MIN - Find minimum value in array
pub fn min_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "MIN: Expected 1 argument, got {}",
            args.len()
        )));
    }

    match &args[0] {
        XdlValue::Array(arr) => {
            if arr.is_empty() {
                return Ok(XdlValue::Undefined);
            }
            let min_val = arr.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            Ok(XdlValue::Double(min_val))
        }
        val => Ok(val.clone()), // Single value is its own minimum
    }
}

/// MAX - Find maximum value in array
pub fn max_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "MAX: Expected 1 argument, got {}",
            args.len()
        )));
    }

    match &args[0] {
        XdlValue::Array(arr) => {
            if arr.is_empty() {
                return Ok(XdlValue::Undefined);
            }
            let max_val = arr.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            Ok(XdlValue::Double(max_val))
        }
        val => Ok(val.clone()), // Single value is its own maximum
    }
}

/// MEAN - Calculate mean of array
pub fn mean_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "MEAN: Expected 1 argument, got {}",
            args.len()
        )));
    }

    match &args[0] {
        XdlValue::Array(arr) => {
            if arr.is_empty() {
                return Ok(XdlValue::Undefined);
            }
            let sum: f64 = arr.iter().sum();
            let mean_val = sum / (arr.len() as f64);
            Ok(XdlValue::Double(mean_val))
        }
        val => {
            let num_val = val.to_double()?;
            Ok(XdlValue::Double(num_val))
        }
    }
}

/// TOTAL - Sum all elements in array
pub fn total_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "TOTAL: Expected 1 argument, got {}",
            args.len()
        )));
    }

    match &args[0] {
        XdlValue::Array(arr) => {
            let sum: f64 = arr.iter().sum();
            Ok(XdlValue::Double(sum))
        }
        val => {
            let num_val = val.to_double()?;
            Ok(XdlValue::Double(num_val))
        }
    }
}

/// REVERSE - Reverse array order
pub fn reverse_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "REVERSE: Expected 1 argument, got {}",
            args.len()
        )));
    }

    match &args[0] {
        XdlValue::Array(arr) => {
            let mut reversed = arr.clone();
            reversed.reverse();
            Ok(XdlValue::Array(reversed))
        }
        val => Ok(val.clone()), // Single value unchanged
    }
}

/// SORT - Sort array elements
pub fn sort_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "SORT: Expected 1 argument, got {}",
            args.len()
        )));
    }

    match &args[0] {
        XdlValue::Array(arr) => {
            let mut sorted = arr.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            Ok(XdlValue::Array(sorted))
        }
        val => Ok(val.clone()), // Single value unchanged
    }
}

/// SMOOTH - Simple moving average (boxcar smoothing)
/// SMOOTH(array, window_size)
/// Returns array of same length with edges handled by reflection
pub fn smooth_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() || args.len() > 2 {
        return Err(XdlError::InvalidArgument(format!(
            "SMOOTH: Expected 1-2 arguments (array, window_size), got {}",
            args.len()
        )));
    }

    let arr = match &args[0] {
        XdlValue::Array(a) => a,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if arr.is_empty() {
        return Ok(XdlValue::Array(vec![]));
    }

    // Default window size is 3
    let window_size = if args.len() == 2 {
        match &args[1] {
            XdlValue::Long(n) => *n as usize,
            XdlValue::Int(n) => *n as usize,
            XdlValue::Double(n) => *n as usize,
            XdlValue::Float(n) => *n as usize,
            _ => {
                return Err(XdlError::TypeMismatch {
                    expected: "numeric".to_string(),
                    actual: format!("{:?}", args[1].gdl_type()),
                })
            }
        }
    } else {
        3
    };

    if window_size == 0 {
        return Err(XdlError::InvalidArgument(
            "SMOOTH: Window size must be greater than 0".to_string(),
        ));
    }

    if window_size > arr.len() {
        return Err(XdlError::InvalidArgument(format!(
            "SMOOTH: Window size ({}) cannot exceed array length ({})",
            window_size,
            arr.len()
        )));
    }

    let n = arr.len();
    let mut result = vec![0.0; n];
    let half_window = window_size / 2;

    // Calculate moving average with edge reflection
    for i in 0..n {
        let mut sum = 0.0;
        let mut count = 0;

        for j in 0..window_size {
            let offset = j as i32 - half_window as i32;
            let idx = (i as i32 + offset) as isize;

            // Reflect at boundaries
            let actual_idx = if idx < 0 {
                (-idx) as usize
            } else if idx >= n as isize {
                let overflow = idx - n as isize;
                (n as isize - 2 - overflow) as usize
            } else {
                idx as usize
            };

            if actual_idx < n {
                sum += arr[actual_idx];
                count += 1;
            }
        }

        result[i] = if count > 0 {
            sum / count as f64
        } else {
            arr[i]
        };
    }

    Ok(XdlValue::Array(result))
}

/// MOVING_AVERAGE - Compute moving average with various edge handling options
/// MOVING_AVERAGE(array, window_size, edge_mode)
/// edge_mode: 0=truncate, 1=wrap, 2=reflect (default), 3=pad_with_mean
pub fn moving_average_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 || args.len() > 3 {
        return Err(XdlError::InvalidArgument(format!(
            "MOVING_AVERAGE: Expected 2-3 arguments (array, window_size, edge_mode), got {}",
            args.len()
        )));
    }

    let arr = match &args[0] {
        XdlValue::Array(a) => a,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if arr.is_empty() {
        return Ok(XdlValue::Array(vec![]));
    }

    let window_size = match &args[1] {
        XdlValue::Long(n) => *n as usize,
        XdlValue::Int(n) => *n as usize,
        XdlValue::Double(n) => *n as usize,
        XdlValue::Float(n) => *n as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    if window_size == 0 {
        return Err(XdlError::InvalidArgument(
            "MOVING_AVERAGE: Window size must be greater than 0".to_string(),
        ));
    }

    // Edge mode: 0=truncate, 1=wrap, 2=reflect (default), 3=pad_with_mean
    let edge_mode = if args.len() == 3 {
        match &args[2] {
            XdlValue::Long(n) => *n,
            XdlValue::Int(n) => *n as i32,
            _ => 2, // Default to reflect
        }
    } else {
        2 // Default to reflect
    };

    let n = arr.len();

    match edge_mode {
        0 => {
            // Truncate mode: only compute where full window fits
            if window_size > n {
                return Ok(XdlValue::Array(vec![]));
            }
            let result_size = n - window_size + 1;
            let mut result = vec![0.0; result_size];

            for i in 0..result_size {
                let mut sum = 0.0;
                for j in 0..window_size {
                    sum += arr[i + j];
                }
                result[i] = sum / window_size as f64;
            }

            Ok(XdlValue::Array(result))
        }
        1 => {
            // Wrap mode: wrap around at edges
            let mut result = vec![0.0; n];
            let half_window = window_size / 2;

            for (i, item) in result.iter_mut().enumerate().take(n) {
                let mut sum = 0.0;
                for j in 0..window_size {
                    let offset = j as i32 - half_window as i32;
                    let idx = ((i as i32 + offset).rem_euclid(n as i32)) as usize;
                    sum += arr[idx];
                }
                *item = sum / window_size as f64;
            }

            Ok(XdlValue::Array(result))
        }
        2 => {
            // Reflect mode: mirror at edges (same as SMOOTH)
            let mut result = vec![0.0; n];
            let half_window = window_size / 2;

            for i in 0..n {
                let mut sum = 0.0;
                let mut count = 0;

                for j in 0..window_size {
                    let offset = j as i32 - half_window as i32;
                    let idx = (i as i32 + offset) as isize;

                    let actual_idx = if idx < 0 {
                        (-idx) as usize
                    } else if idx >= n as isize {
                        let overflow = idx - n as isize;
                        let reflected = n as isize - 2 - overflow;
                        if reflected >= 0 {
                            reflected as usize
                        } else {
                            continue;
                        }
                    } else {
                        idx as usize
                    };

                    if actual_idx < n {
                        sum += arr[actual_idx];
                        count += 1;
                    }
                }

                result[i] = if count > 0 {
                    sum / count as f64
                } else {
                    arr[i]
                };
            }

            Ok(XdlValue::Array(result))
        }
        3 => {
            // Pad with mean mode: use array mean for out-of-bounds values
            let mean: f64 = arr.iter().sum::<f64>() / n as f64;
            let mut result = vec![0.0; n];
            let half_window = window_size / 2;

            for (i, item) in result.iter_mut().enumerate().take(n) {
                let mut sum = 0.0;
                for j in 0..window_size {
                    let offset = j as i32 - half_window as i32;
                    let idx = i as i32 + offset;

                    if idx < 0 || idx >= n as i32 {
                        sum += mean;
                    } else {
                        sum += arr[idx as usize];
                    }
                }
                *item = sum / window_size as f64;
            }

            Ok(XdlValue::Array(result))
        }
        _ => Err(XdlError::InvalidArgument(format!(
            "MOVING_AVERAGE: Invalid edge mode {}. Must be 0-3",
            edge_mode
        ))),
    }
}

/// WMA - Weighted Moving Average
/// WMA(array, window_size)
/// Linear weights: most recent value has highest weight
pub fn wma_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 2 {
        return Err(XdlError::InvalidArgument(format!(
            "WMA: Expected 2 arguments (array, window_size), got {}",
            args.len()
        )));
    }

    let arr = match &args[0] {
        XdlValue::Array(a) => a,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if arr.is_empty() {
        return Ok(XdlValue::Array(vec![]));
    }

    let window_size = match &args[1] {
        XdlValue::Long(n) => *n as usize,
        XdlValue::Int(n) => *n as usize,
        XdlValue::Double(n) => *n as usize,
        XdlValue::Float(n) => *n as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    if window_size == 0 {
        return Err(XdlError::InvalidArgument(
            "WMA: Window size must be greater than 0".to_string(),
        ));
    }

    if window_size > arr.len() {
        return Err(XdlError::InvalidArgument(format!(
            "WMA: Window size ({}) cannot exceed array length ({})",
            window_size,
            arr.len()
        )));
    }

    let n = arr.len();
    let mut result = vec![0.0; n - window_size + 1];

    // Calculate weight sum: 1 + 2 + 3 + ... + window_size
    let weight_sum: f64 = (window_size * (window_size + 1) / 2) as f64;

    for i in 0..=n - window_size {
        let mut weighted_sum = 0.0;
        for j in 0..window_size {
            let weight = (j + 1) as f64; // Linear weights: 1, 2, 3, ..., window_size
            weighted_sum += arr[i + j] * weight;
        }
        result[i] = weighted_sum / weight_sum;
    }

    Ok(XdlValue::Array(result))
}

/// EMA - Exponential Moving Average
/// EMA(array, alpha)
/// alpha: smoothing factor (0 < alpha <= 1), typically 2/(N+1) for N-period EMA
pub fn ema_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 2 {
        return Err(XdlError::InvalidArgument(format!(
            "EMA: Expected 2 arguments (array, alpha), got {}",
            args.len()
        )));
    }

    let arr = match &args[0] {
        XdlValue::Array(a) => a,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if arr.is_empty() {
        return Ok(XdlValue::Array(vec![]));
    }

    let alpha = match &args[1] {
        XdlValue::Double(a) => *a,
        XdlValue::Float(a) => *a as f64,
        XdlValue::Long(a) => *a as f64,
        XdlValue::Int(a) => *a as f64,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    if alpha <= 0.0 || alpha > 1.0 {
        return Err(XdlError::InvalidArgument(format!(
            "EMA: Alpha must be in range (0, 1], got {}",
            alpha
        )));
    }

    let n = arr.len();
    let mut result = vec![0.0; n];

    // Initialize with first value
    result[0] = arr[0];

    // Calculate EMA: EMA[t] = alpha * value[t] + (1 - alpha) * EMA[t-1]
    for i in 1..n {
        result[i] = alpha * arr[i] + (1.0 - alpha) * result[i - 1];
    }

    Ok(XdlValue::Array(result))
}

/// CUMULATIVE_AVERAGE - Cumulative (expanding) moving average
/// CUMULATIVE_AVERAGE(array)
/// Each element is the mean of all values up to and including that position
pub fn cumulative_average_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "CUMULATIVE_AVERAGE: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let arr = match &args[0] {
        XdlValue::Array(a) => a,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if arr.is_empty() {
        return Ok(XdlValue::Array(vec![]));
    }

    let n = arr.len();
    let mut result = vec![0.0; n];
    let mut cumsum = 0.0;

    for i in 0..n {
        cumsum += arr[i];
        result[i] = cumsum / (i + 1) as f64;
    }

    Ok(XdlValue::Array(result))
}

/// REFORM - Reshape array to new dimensions
/// REFORM(array, d1, d2, d3, ...)
/// Changes the dimensions of an array without changing the total number of elements
/// Elements are arranged in column-major order (like IDL/GDL)
///
/// Examples:
///   arr = FINDGEN(12)              ; [0, 1, 2, ..., 11]
///   reformed = REFORM(arr, 3, 4)   ; Reshape to 3x4
///   reformed = REFORM(arr, 2, 2, 3); Reshape to 2x2x3
pub fn reform_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(format!(
            "REFORM: Expected at least 2 arguments (array, dimensions...), got {}",
            args.len()
        )));
    }

    // Get input array
    let arr = match &args[0] {
        XdlValue::Array(a) => a,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    // Extract new dimensions
    let mut new_dims = Vec::new();
    for arg in args.iter().skip(1) {
        let dim = extract_dimension(arg)?;
        new_dims.push(dim);
    }

    // Calculate total size of new dimensions
    let new_size: usize = new_dims.iter().product();

    // Check that total size matches
    if new_size != arr.len() {
        return Err(XdlError::DimensionError(format!(
            "REFORM: New dimensions ({}) don't match array size ({})",
            new_size,
            arr.len()
        )));
    }

    // In current implementation, arrays are flat Vec<f64>
    // So REFORM just returns a clone (dimensions are implicit)
    // In a full implementation, this would update dimension metadata
    Ok(XdlValue::Array(arr.clone()))
}

/// TRANSPOSE - Transpose a 2D array (matrix)
/// TRANSPOSE(array [, permutation])
/// For 2D arrays: swaps rows and columns
/// For multi-dimensional: can specify axis permutation
///
/// Note: Current implementation assumes 2D matrices stored in row-major order
/// and requires explicit dimension information. Since XDL arrays are currently
/// flat Vec<f64>, this is a simplified implementation.
///
/// Examples:
///   arr = [[1, 2, 3], [4, 5, 6]]  ; 2x3 matrix
///   TRANSPOSE(arr)                 ; Returns 3x2 matrix
pub fn transpose_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "TRANSPOSE: Expected at least 1 argument".to_string(),
        ));
    }

    // Get input array
    let _arr = match &args[0] {
        XdlValue::Array(a) => a,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    // For simplicity in current flat array implementation:
    // If no dimension info is available, we can't properly transpose
    // This is a limitation of the current Vec<f64> representation
    //
    // A full implementation would require dimension metadata with arrays
    // For now, we'll implement a helper that works with explicit dimensions
    //
    // Return error suggesting REFORM should be used with proper dimensions
    Err(XdlError::RuntimeError(
        "TRANSPOSE: Requires dimension metadata. Current array implementation uses flat vectors. \
         Use REFORM with explicit dimensions instead."
            .to_string(),
    ))
}

/// TRANSPOSE_2D - Helper function to transpose a 2D array with known dimensions
/// This is a working implementation when dimensions are known
/// transpose_2d(array, nrows, ncols) -> transposed array
pub fn transpose_2d(arr: &[f64], nrows: usize, ncols: usize) -> XdlResult<Vec<f64>> {
    if arr.len() != nrows * ncols {
        return Err(XdlError::DimensionError(format!(
            "Array size {} doesn't match dimensions {}x{}",
            arr.len(),
            nrows,
            ncols
        )));
    }

    let mut result = vec![0.0; arr.len()];

    // Transpose: result[j, i] = arr[i, j]
    // In row-major: arr[i*ncols + j] -> result[j*nrows + i]
    for i in 0..nrows {
        for j in 0..ncols {
            result[j * nrows + i] = arr[i * ncols + j];
        }
    }

    Ok(result)
}
