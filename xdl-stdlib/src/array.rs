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

    let total_size = calculate_total_size(args)?;

    // For 1D arrays, return XdlValue::Array with byte values as f64
    // (XDL currently uses f64 arrays internally)
    let data = vec![0.0; total_size];
    Ok(XdlValue::Array(data))
}

/// Create integer array: INTARR(dimensions...)
/// Returns array of 16-bit integers initialized to zero
pub fn intarr(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "INTARR: At least one dimension required".to_string(),
        ));
    }

    let total_size = calculate_total_size(args)?;
    let data = vec![0.0; total_size];
    Ok(XdlValue::Array(data))
}

/// Create long integer array: LONARR(dimensions...)
/// Returns array of 32-bit integers initialized to zero
pub fn lonarr(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "LONARR: At least one dimension required".to_string(),
        ));
    }

    let total_size = calculate_total_size(args)?;
    let data = vec![0.0; total_size];
    Ok(XdlValue::Array(data))
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

    // Return MultiDimArray if more than 1 dimension, Array for 1D
    if shape.len() == 1 {
        Ok(XdlValue::Array(data))
    } else {
        Ok(XdlValue::MultiDimArray { data, shape })
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

    let total_size = calculate_total_size(args)?;
    let data = vec![0.0; total_size];
    Ok(XdlValue::Array(data))
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

    // Return MultiDimArray with proper shape metadata
    // This allows 3D graphics procedures to properly handle the data
    if new_dims.len() == 1 {
        // 1D array - return as regular Array
        Ok(XdlValue::Array(arr.clone()))
    } else {
        // Multi-dimensional - return with shape information
        Ok(XdlValue::MultiDimArray {
            data: arr.clone(),
            shape: new_dims,
        })
    }
}

/// TRANSPOSE - Transpose a 2D array (matrix)
/// TRANSPOSE(array [, permutation])
/// For 2D arrays: swaps rows and columns
/// For multi-dimensional: can specify axis permutation
///
/// Examples:
///   arr = REFORM(FINDGEN(6), 2, 3)  ; 2x3 matrix
///   TRANSPOSE(arr)                   ; Returns 3x2 matrix
pub fn transpose_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "TRANSPOSE: Expected at least 1 argument".to_string(),
        ));
    }

    match &args[0] {
        XdlValue::MultiDimArray { data, shape } => {
            if shape.len() == 2 {
                // 2D transpose
                let nrows = shape[0];
                let ncols = shape[1];
                let transposed = transpose_2d(data, nrows, ncols)?;
                Ok(XdlValue::MultiDimArray {
                    data: transposed,
                    shape: vec![ncols, nrows],
                })
            } else if shape.len() == 1 {
                // 1D array: transpose is identity
                Ok(XdlValue::MultiDimArray {
                    data: data.clone(),
                    shape: shape.clone(),
                })
            } else {
                // Higher dimensional: for now, just swap first two dimensions
                // Full implementation would support arbitrary permutations
                Err(XdlError::DimensionError(
                    "TRANSPOSE: Only 2D arrays fully supported".to_string(),
                ))
            }
        }
        XdlValue::Array(_) => Err(XdlError::DimensionError(
            "TRANSPOSE: Requires MultiDimArray with shape information. Use REFORM first."
                .to_string(),
        )),
        _ => Err(XdlError::TypeMismatch {
            expected: "array".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// MESHGRID - Create coordinate matrices from coordinate vectors
/// meshgrid(x, y) returns a MultiDimArray containing X grid values
/// In MATLAB: [X, Y] = meshgrid(x, y)
/// For XDL: We return X as a MultiDimArray. Y can be computed similarly or we return both in NestedArray
///
/// X(i,j) = x(j) for all i
/// Y(i,j) = y(i) for all j
pub fn meshgrid(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 2 {
        return Err(XdlError::InvalidArgument(format!(
            "MESHGRID: Expected 2 arguments (x, y), got {}",
            args.len()
        )));
    }

    // Extract x and y vectors
    let x_vec = match &args[0] {
        XdlValue::Array(arr) => arr.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let y_vec = match &args[1] {
        XdlValue::Array(arr) => arr.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let nx = x_vec.len();
    let ny = y_vec.len();

    // Create X matrix: X(i,j) = x(j)
    // Stored in row-major order: X[i*nx + j] = x[j]
    let mut x_matrix = Vec::with_capacity(ny * nx);
    for _i in 0..ny {
        for &x_val in &x_vec {
            x_matrix.push(x_val);
        }
    }

    // Create Y matrix: Y(i,j) = y(i)
    // Stored in row-major order: Y[i*nx + j] = y[i]
    let mut y_matrix = Vec::with_capacity(ny * nx);
    for &y_val in &y_vec {
        for _j in 0..nx {
            y_matrix.push(y_val);
        }
    }

    // Return as NestedArray with both X and Y matrices
    // User can access as: result = meshgrid(x, y)
    // Then X = result[0] and Y = result[1] (or we document X comes first)
    let x_grid = XdlValue::MultiDimArray {
        data: x_matrix,
        shape: vec![ny, nx],
    };

    let y_grid = XdlValue::MultiDimArray {
        data: y_matrix,
        shape: vec![ny, nx],
    };

    // Return both as nested array
    Ok(XdlValue::NestedArray(vec![x_grid, y_grid]))
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

/// ROTATE - Rotate a 2D array by 90-degree increments
/// ROTATE(array, direction)
/// direction: 1 = 90° CCW, 2 = 180°, 3 = 90° CW, 4 = transpose, 5 = reverse transpose
/// For MultiDimArray with shape info, or regular arrays interpreted as 2D
pub fn rotate_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "ROTATE: Expected at least 2 arguments (array, direction)".to_string(),
        ));
    }

    let direction = match &args[1] {
        XdlValue::Long(n) => *n,
        XdlValue::Int(n) => *n as i32,
        XdlValue::Byte(n) => *n as i32,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    match &args[0] {
        XdlValue::MultiDimArray { data, shape } => {
            if shape.len() != 2 {
                return Err(XdlError::DimensionError(
                    "ROTATE: Only 2D arrays are supported".to_string(),
                ));
            }

            let nrows = shape[0];
            let ncols = shape[1];

            let (new_data, new_shape) = match direction {
                1 => {
                    // 90° CCW: (i,j) -> (ncols-1-j, i)
                    let mut result = vec![0.0; data.len()];
                    for i in 0..nrows {
                        for j in 0..ncols {
                            result[(ncols - 1 - j) * nrows + i] = data[i * ncols + j];
                        }
                    }
                    (result, vec![ncols, nrows])
                }
                2 => {
                    // 180°: reverse all elements
                    let result: Vec<f64> = data.iter().rev().copied().collect();
                    (result, shape.clone())
                }
                3 => {
                    // 90° CW: (i,j) -> (j, nrows-1-i)
                    let mut result = vec![0.0; data.len()];
                    for i in 0..nrows {
                        for j in 0..ncols {
                            result[j * nrows + (nrows - 1 - i)] = data[i * ncols + j];
                        }
                    }
                    (result, vec![ncols, nrows])
                }
                4 => {
                    // Transpose: (i,j) -> (j,i)
                    let transposed = transpose_2d(data, nrows, ncols)?;
                    (transposed, vec![ncols, nrows])
                }
                5 => {
                    // Reverse transpose
                    let transposed = transpose_2d(data, nrows, ncols)?;
                    let result: Vec<f64> = transposed.iter().rev().copied().collect();
                    (result, vec![ncols, nrows])
                }
                _ => {
                    return Err(XdlError::InvalidArgument(format!(
                        "ROTATE: Invalid direction {}. Use 1-5",
                        direction
                    )))
                }
            };

            Ok(XdlValue::MultiDimArray {
                data: new_data,
                shape: new_shape,
            })
        }
        XdlValue::Array(_) => Err(XdlError::DimensionError(
            "ROTATE: Requires MultiDimArray with shape information. Use REFORM first.".to_string(),
        )),
        _ => Err(XdlError::TypeMismatch {
            expected: "array".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// SHIFT - Circular shift of array elements
/// SHIFT(array, shift_amount [, shift_y])
/// For 1D: shift_amount elements
/// For 2D: shift_amount in x, shift_y in y
pub fn shift_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "SHIFT: Expected at least 2 arguments".to_string(),
        ));
    }

    let shift_x = match &args[1] {
        XdlValue::Long(n) => *n as isize,
        XdlValue::Int(n) => *n as isize,
        XdlValue::Byte(n) => *n as isize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    match &args[0] {
        XdlValue::Array(arr) => {
            // 1D shift
            let n = arr.len() as isize;
            if n == 0 {
                return Ok(XdlValue::Array(vec![]));
            }

            let shift = ((shift_x % n) + n) % n; // Normalize to positive
            let mut result = vec![0.0; arr.len()];

            for (i, &val) in arr.iter().enumerate() {
                let new_idx = (i as isize + shift) % n;
                result[new_idx as usize] = val;
            }

            Ok(XdlValue::Array(result))
        }
        XdlValue::MultiDimArray { data, shape } => {
            if shape.len() == 2 {
                // 2D shift
                let shift_y = if args.len() >= 3 {
                    match &args[2] {
                        XdlValue::Long(n) => *n as isize,
                        XdlValue::Int(n) => *n as isize,
                        XdlValue::Byte(n) => *n as isize,
                        _ => 0,
                    }
                } else {
                    0
                };

                let nrows = shape[0] as isize;
                let ncols = shape[1] as isize;
                let mut result = vec![0.0; data.len()];

                for i in 0..nrows {
                    for j in 0..ncols {
                        let new_i = ((i + shift_y) % nrows + nrows) % nrows;
                        let new_j = ((j + shift_x) % ncols + ncols) % ncols;
                        result[(new_i * ncols + new_j) as usize] = data[(i * ncols + j) as usize];
                    }
                }

                Ok(XdlValue::MultiDimArray {
                    data: result,
                    shape: shape.clone(),
                })
            } else {
                // 1D shift for multidim with shape.len() == 1
                let n = data.len() as isize;
                let shift = ((shift_x % n) + n) % n;
                let mut result = vec![0.0; data.len()];

                for (i, &val) in data.iter().enumerate() {
                    let new_idx = (i as isize + shift) % n;
                    result[new_idx as usize] = val;
                }

                Ok(XdlValue::MultiDimArray {
                    data: result,
                    shape: shape.clone(),
                })
            }
        }
        _ => Err(XdlError::TypeMismatch {
            expected: "array".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// REBIN - Resize array by resampling or replication
/// REBIN(array, new_dim1 [, new_dim2, ...])
/// For upsampling: replicates values
/// For downsampling: averages values
pub fn rebin_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "REBIN: Expected at least 2 arguments (array, new_dimensions...)".to_string(),
        ));
    }

    // For simplicity, implement 1D rebin
    // A full implementation would handle multi-dimensional rebinning
    match &args[0] {
        XdlValue::Array(arr) => {
            let new_size = extract_dimension(&args[1])?;
            let old_size = arr.len();

            if new_size == 0 || old_size == 0 {
                return Ok(XdlValue::Array(vec![]));
            }

            let mut result = Vec::with_capacity(new_size);

            if new_size >= old_size {
                // Upsampling: replicate values
                let ratio = new_size as f64 / old_size as f64;
                for i in 0..new_size {
                    let src_idx = (i as f64 / ratio).floor() as usize;
                    let src_idx = src_idx.min(old_size - 1);
                    result.push(arr[src_idx]);
                }
            } else {
                // Downsampling: average values
                let ratio = old_size as f64 / new_size as f64;
                for i in 0..new_size {
                    let start = (i as f64 * ratio).floor() as usize;
                    let end = ((i + 1) as f64 * ratio).floor() as usize;
                    let end = end.min(old_size);

                    let sum: f64 = arr[start..end].iter().sum();
                    let count = (end - start) as f64;
                    result.push(sum / count);
                }
            }

            Ok(XdlValue::Array(result))
        }
        XdlValue::MultiDimArray { data, shape } => {
            // For 2D arrays
            if shape.len() == 2 && args.len() >= 3 {
                let new_rows = extract_dimension(&args[1])?;
                let new_cols = extract_dimension(&args[2])?;
                let old_rows = shape[0];
                let old_cols = shape[1];

                let mut result = Vec::with_capacity(new_rows * new_cols);

                let row_ratio = old_rows as f64 / new_rows as f64;
                let col_ratio = old_cols as f64 / new_cols as f64;

                for i in 0..new_rows {
                    for j in 0..new_cols {
                        let src_i = (i as f64 * row_ratio).floor() as usize;
                        let src_j = (j as f64 * col_ratio).floor() as usize;
                        let src_i = src_i.min(old_rows - 1);
                        let src_j = src_j.min(old_cols - 1);
                        result.push(data[src_i * old_cols + src_j]);
                    }
                }

                Ok(XdlValue::MultiDimArray {
                    data: result,
                    shape: vec![new_rows, new_cols],
                })
            } else {
                Err(XdlError::InvalidArgument(
                    "REBIN: Dimension mismatch".to_string(),
                ))
            }
        }
        _ => Err(XdlError::TypeMismatch {
            expected: "array".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// REPLICATE - Create array by replicating a value
/// REPLICATE(value, dim1 [, dim2, ...])
/// Creates an array filled with the specified value
pub fn replicate_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "REPLICATE: Expected at least 2 arguments (value, dimensions...)".to_string(),
        ));
    }

    // Extract dimensions
    let mut dims = Vec::new();
    for arg in args.iter().skip(1) {
        let dim = extract_dimension(arg)?;
        dims.push(dim);
    }

    let total_size: usize = dims.iter().product();

    // Extract the value to replicate
    match &args[0] {
        XdlValue::Float(v) => {
            let data = vec![*v as f64; total_size];
            if dims.len() == 1 {
                Ok(XdlValue::Array(data))
            } else {
                Ok(XdlValue::MultiDimArray { data, shape: dims })
            }
        }
        XdlValue::Double(v) => {
            let data = vec![*v; total_size];
            if dims.len() == 1 {
                Ok(XdlValue::Array(data))
            } else {
                Ok(XdlValue::MultiDimArray { data, shape: dims })
            }
        }
        XdlValue::Int(v) => {
            let data = vec![*v as f64; total_size];
            if dims.len() == 1 {
                Ok(XdlValue::Array(data))
            } else {
                Ok(XdlValue::MultiDimArray { data, shape: dims })
            }
        }
        XdlValue::Long(v) => {
            let data = vec![*v as f64; total_size];
            if dims.len() == 1 {
                Ok(XdlValue::Array(data))
            } else {
                Ok(XdlValue::MultiDimArray { data, shape: dims })
            }
        }
        XdlValue::Byte(v) => {
            let data = vec![*v as f64; total_size];
            if dims.len() == 1 {
                Ok(XdlValue::Array(data))
            } else {
                Ok(XdlValue::MultiDimArray { data, shape: dims })
            }
        }
        XdlValue::String(s) => {
            // For strings, create a NestedArray of strings
            let data = vec![XdlValue::String(s.clone()); total_size];
            Ok(XdlValue::NestedArray(data))
        }
        _ => Err(XdlError::TypeMismatch {
            expected: "scalar value".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// HISTOGRAM - Compute histogram of array values
/// HISTOGRAM(array [, min=min_val, max=max_val, binsize=bin_size, nbins=n_bins])
/// Returns array of bin counts
pub fn histogram_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "HISTOGRAM: Expected at least 1 argument".to_string(),
        ));
    }

    let arr = match &args[0] {
        XdlValue::Array(a) => a,
        XdlValue::MultiDimArray { data, .. } => data,
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

    // Find min and max values
    let min_val = arr.iter().copied().fold(f64::INFINITY, f64::min);
    let max_val = arr.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    // Default: 10 bins
    let nbins = if args.len() >= 2 {
        match &args[1] {
            XdlValue::Long(n) => *n as usize,
            XdlValue::Int(n) => *n as usize,
            _ => 10,
        }
    } else {
        10
    };

    if nbins == 0 {
        return Err(XdlError::InvalidArgument(
            "HISTOGRAM: Number of bins must be positive".to_string(),
        ));
    }

    let binsize = (max_val - min_val) / nbins as f64;
    let mut bins = vec![0.0; nbins];

    // Count values in each bin
    for &val in arr {
        if val.is_finite() {
            let bin_idx = ((val - min_val) / binsize).floor() as usize;
            let bin_idx = bin_idx.min(nbins - 1); // Handle edge case where val == max_val
            bins[bin_idx] += 1.0;
        }
    }

    Ok(XdlValue::Array(bins))
}

/// INTERPOL - 1D interpolation
/// INTERPOL(v, n) interpolates array v to n points
/// INTERPOL(v, x, u) interpolates v(x) at points u
pub fn interpol(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "INTERPOL: Expected at least 2 arguments".to_string(),
        ));
    }

    let v = match &args[0] {
        XdlValue::Array(a) => a,
        XdlValue::MultiDimArray { data, .. } => data,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if v.is_empty() {
        return Ok(XdlValue::Array(vec![]));
    }

    if args.len() == 2 {
        // Simple form: INTERPOL(v, n) - interpolate to n points
        let n = match &args[1] {
            XdlValue::Long(n) => *n as usize,
            XdlValue::Int(n) => *n as usize,
            _ => {
                return Err(XdlError::TypeMismatch {
                    expected: "integer".to_string(),
                    actual: format!("{:?}", args[1].gdl_type()),
                })
            }
        };

        if n == 0 {
            return Ok(XdlValue::Array(vec![]));
        }

        // Linear interpolation
        let mut result = Vec::with_capacity(n);
        let old_len = v.len();

        for i in 0..n {
            let x = i as f64 * (old_len - 1) as f64 / (n - 1).max(1) as f64;
            let idx = x.floor() as usize;
            let frac = x - idx as f64;

            if idx + 1 < old_len {
                // Linear interpolation between idx and idx+1
                let val = v[idx] * (1.0 - frac) + v[idx + 1] * frac;
                result.push(val);
            } else {
                // At the end, just use last value
                result.push(v[old_len - 1]);
            }
        }

        Ok(XdlValue::Array(result))
    } else {
        // Full form: INTERPOL(v, x, u) - interpolate v(x) at points u
        let x = match &args[1] {
            XdlValue::Array(a) => a,
            _ => {
                return Err(XdlError::TypeMismatch {
                    expected: "array".to_string(),
                    actual: format!("{:?}", args[1].gdl_type()),
                })
            }
        };

        let u = match &args[2] {
            XdlValue::Array(a) => a,
            _ => {
                return Err(XdlError::TypeMismatch {
                    expected: "array".to_string(),
                    actual: format!("{:?}", args[2].gdl_type()),
                })
            }
        };

        if v.len() != x.len() {
            return Err(XdlError::DimensionError(
                "INTERPOL: v and x must have same length".to_string(),
            ));
        }

        // Interpolate at each point in u
        let mut result = Vec::with_capacity(u.len());

        for &u_val in u {
            // Find bracketing points in x
            let val = if u_val <= x[0] {
                v[0]
            } else if u_val >= x[x.len() - 1] {
                v[v.len() - 1]
            } else {
                // Binary search for bracketing interval
                let mut idx = 0;
                for i in 0..x.len() - 1 {
                    if x[i] <= u_val && u_val <= x[i + 1] {
                        idx = i;
                        break;
                    }
                }

                // Linear interpolation
                let x0 = x[idx];
                let x1 = x[idx + 1];
                let v0 = v[idx];
                let v1 = v[idx + 1];
                let frac = (u_val - x0) / (x1 - x0);
                v0 * (1.0 - frac) + v1 * frac
            };

            result.push(val);
        }

        Ok(XdlValue::Array(result))
    }
}

/// CONGRID - Resize array using interpolation
/// CONGRID(array, new_dims...) resizes array to new dimensions with interpolation
/// More sophisticated than REBIN - uses interpolation rather than replication
pub fn congrid(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "CONGRID: Expected at least 2 arguments (array, new_dims...)".to_string(),
        ));
    }

    match &args[0] {
        XdlValue::Array(arr) => {
            // 1D case
            if args.len() != 2 {
                return Err(XdlError::InvalidArgument(
                    "CONGRID: 1D array expects 2 arguments (array, new_size)".to_string(),
                ));
            }

            let new_size = extract_dimension(&args[1])?;
            if new_size == 0 || arr.is_empty() {
                return Ok(XdlValue::Array(vec![]));
            }

            // Use linear interpolation
            let old_size = arr.len();
            let mut result = Vec::with_capacity(new_size);

            for i in 0..new_size {
                let x = i as f64 * (old_size - 1) as f64 / (new_size - 1).max(1) as f64;
                let idx = x.floor() as usize;
                let frac = x - idx as f64;

                if idx + 1 < old_size {
                    let val = arr[idx] * (1.0 - frac) + arr[idx + 1] * frac;
                    result.push(val);
                } else {
                    result.push(arr[old_size - 1]);
                }
            }

            Ok(XdlValue::Array(result))
        }
        XdlValue::MultiDimArray { data, shape } => {
            // Multi-dimensional case
            if shape.len() == 2 && args.len() >= 3 {
                // 2D interpolation
                let old_rows = shape[0];
                let old_cols = shape[1];
                let new_rows = extract_dimension(&args[1])?;
                let new_cols = extract_dimension(&args[2])?;

                if new_rows == 0 || new_cols == 0 {
                    return Ok(XdlValue::MultiDimArray {
                        data: vec![],
                        shape: vec![new_rows, new_cols],
                    });
                }

                let mut result = Vec::with_capacity(new_rows * new_cols);

                // Bilinear interpolation
                for i in 0..new_rows {
                    let y = i as f64 * (old_rows - 1) as f64 / (new_rows - 1).max(1) as f64;
                    let y_idx = y.floor() as usize;
                    let y_frac = y - y_idx as f64;

                    for j in 0..new_cols {
                        let x = j as f64 * (old_cols - 1) as f64 / (new_cols - 1).max(1) as f64;
                        let x_idx = x.floor() as usize;
                        let x_frac = x - x_idx as f64;

                        // Bilinear interpolation
                        let val = if y_idx + 1 < old_rows && x_idx + 1 < old_cols {
                            let v00 = data[y_idx * old_cols + x_idx];
                            let v01 = data[y_idx * old_cols + x_idx + 1];
                            let v10 = data[(y_idx + 1) * old_cols + x_idx];
                            let v11 = data[(y_idx + 1) * old_cols + x_idx + 1];

                            let v0 = v00 * (1.0 - x_frac) + v01 * x_frac;
                            let v1 = v10 * (1.0 - x_frac) + v11 * x_frac;
                            v0 * (1.0 - y_frac) + v1 * y_frac
                        } else if y_idx + 1 < old_rows {
                            // Only y interpolation
                            let v0 = data[y_idx * old_cols + x_idx.min(old_cols - 1)];
                            let v1 = data[(y_idx + 1) * old_cols + x_idx.min(old_cols - 1)];
                            v0 * (1.0 - y_frac) + v1 * y_frac
                        } else if x_idx + 1 < old_cols {
                            // Only x interpolation
                            let v0 = data[y_idx.min(old_rows - 1) * old_cols + x_idx];
                            let v1 = data[y_idx.min(old_rows - 1) * old_cols + x_idx + 1];
                            v0 * (1.0 - x_frac) + v1 * x_frac
                        } else {
                            // At corner
                            data[y_idx.min(old_rows - 1) * old_cols + x_idx.min(old_cols - 1)]
                        };

                        result.push(val);
                    }
                }

                Ok(XdlValue::MultiDimArray {
                    data: result,
                    shape: vec![new_rows, new_cols],
                })
            } else {
                Err(XdlError::InvalidArgument(
                    "CONGRID: Only 1D and 2D arrays currently supported".to_string(),
                ))
            }
        }
        _ => Err(XdlError::TypeMismatch {
            expected: "array".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// UNIQ - Find unique elements in sorted array
/// UNIQ(array) returns indices of unique elements
/// Input array should be sorted (use SORT first if needed)
/// Returns array of indices pointing to first occurrence of each unique value
pub fn uniq(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "UNIQ: Expected 1 argument, got {}",
            args.len()
        )));
    }

    match &args[0] {
        XdlValue::Array(arr) => {
            if arr.is_empty() {
                return Ok(XdlValue::Array(vec![]));
            }

            let mut indices = vec![0.0]; // First element is always unique
            let mut prev = arr[0];

            for (i, &val) in arr.iter().enumerate().skip(1) {
                if (val - prev).abs() > 1e-10 {
                    // Different value
                    indices.push(i as f64);
                    prev = val;
                }
            }

            Ok(XdlValue::Array(indices))
        }
        _ => Err(XdlError::TypeMismatch {
            expected: "array".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// ARRAY_INDICES - Convert 1D index to N-D indices
/// ARRAY_INDICES(array, index) converts 1D flat index to N-D subscripts
/// Returns array of indices for each dimension
pub fn array_indices(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 2 {
        return Err(XdlError::InvalidArgument(format!(
            "ARRAY_INDICES: Expected 2 arguments (array, index), got {}",
            args.len()
        )));
    }

    // Get array shape
    let shape = match &args[0] {
        XdlValue::MultiDimArray { shape, .. } => shape.clone(),
        XdlValue::Array(arr) => vec![arr.len()],
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    // Get flat index
    let flat_idx = match &args[1] {
        XdlValue::Long(i) => *i as usize,
        XdlValue::Int(i) => *i as usize,
        XdlValue::Double(i) => *i as usize,
        XdlValue::Float(i) => *i as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    // Convert to N-D indices
    let mut indices = Vec::with_capacity(shape.len());
    let mut remaining = flat_idx;

    for i in (0..shape.len()).rev() {
        let stride: usize = shape.iter().skip(i + 1).product();
        let idx = if stride > 0 {
            remaining / stride
        } else {
            remaining
        };
        indices.push(idx as f64);
        remaining %= stride.max(1);
    }

    indices.reverse();
    Ok(XdlValue::Array(indices))
}

/// ARRAY_EQUAL - Test if two arrays are equal
/// ARRAY_EQUAL(array1, array2 [, /NO_TYPECONV])
/// Returns 1 if arrays are equal (same dimensions and values), 0 otherwise
pub fn array_equal(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(format!(
            "ARRAY_EQUAL: Expected at least 2 arguments, got {}",
            args.len()
        )));
    }

    let equal = match (&args[0], &args[1]) {
        (XdlValue::Array(a1), XdlValue::Array(a2)) => {
            if a1.len() != a2.len() {
                false
            } else {
                a1.iter()
                    .zip(a2.iter())
                    .all(|(v1, v2)| (v1 - v2).abs() < 1e-10)
            }
        }
        (
            XdlValue::MultiDimArray {
                data: d1,
                shape: s1,
            },
            XdlValue::MultiDimArray {
                data: d2,
                shape: s2,
            },
        ) => {
            s1 == s2
                && d1.len() == d2.len()
                && d1
                    .iter()
                    .zip(d2.iter())
                    .all(|(v1, v2)| (v1 - v2).abs() < 1e-10)
        }
        _ => {
            // Different types or one is scalar
            false
        }
    };

    Ok(XdlValue::Long(if equal { 1 } else { 0 }))
}

/// PERMUTE - Permute array dimensions
/// PERMUTE(array, dimension_order)
/// Reorders dimensions according to specified permutation
/// e.g., PERMUTE(array, [2,0,1]) puts dimension 2 first, then 0, then 1
pub fn permute(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 2 {
        return Err(XdlError::InvalidArgument(format!(
            "PERMUTE: Expected 2 arguments (array, dimension_order), got {}",
            args.len()
        )));
    }

    match &args[0] {
        XdlValue::MultiDimArray { data, shape } => {
            let perm = match &args[1] {
                XdlValue::Array(p) => p.iter().map(|&x| x as usize).collect::<Vec<_>>(),
                _ => {
                    return Err(XdlError::TypeMismatch {
                        expected: "array".to_string(),
                        actual: format!("{:?}", args[1].gdl_type()),
                    })
                }
            };

            if perm.len() != shape.len() {
                return Err(XdlError::InvalidArgument(format!(
                    "PERMUTE: Permutation length ({}) must match number of dimensions ({})",
                    perm.len(),
                    shape.len()
                )));
            }

            // Validate permutation (all indices must be 0..n-1)
            let mut check = vec![false; shape.len()];
            for &p in &perm {
                if p >= shape.len() {
                    return Err(XdlError::InvalidArgument(format!(
                        "PERMUTE: Invalid dimension index {}",
                        p
                    )));
                }
                check[p] = true;
            }
            if !check.iter().all(|&x| x) {
                return Err(XdlError::InvalidArgument(
                    "PERMUTE: Permutation must contain all dimension indices".to_string(),
                ));
            }

            // Build new shape
            let new_shape: Vec<usize> = perm.iter().map(|&p| shape[p]).collect();

            // For 2D arrays (most common case), optimize
            if shape.len() == 2 && perm == vec![1, 0] {
                // Simple transpose
                let rows = shape[0];
                let cols = shape[1];
                let mut result = vec![0.0; data.len()];

                for i in 0..rows {
                    for j in 0..cols {
                        result[j * rows + i] = data[i * cols + j];
                    }
                }

                return Ok(XdlValue::MultiDimArray {
                    data: result,
                    shape: new_shape,
                });
            }

            // General case: permute arbitrary dimensions
            let total_size = shape.iter().product();
            let mut result = vec![0.0; total_size];

            // Compute strides for old and new layouts
            let old_strides: Vec<usize> = (0..shape.len())
                .map(|i| shape.iter().skip(i + 1).product::<usize>())
                .collect();
            let new_strides: Vec<usize> = (0..new_shape.len())
                .map(|i| new_shape.iter().skip(i + 1).product::<usize>())
                .collect();

            for (flat_idx, &data_val) in data.iter().enumerate().take(total_size) {
                // Convert flat index to old indices
                let mut old_indices = vec![0; shape.len()];
                let mut remaining = flat_idx;
                for i in 0..shape.len() {
                    old_indices[i] = remaining / old_strides[i].max(1);
                    remaining %= old_strides[i].max(1);
                }

                // Permute indices
                let new_indices: Vec<usize> = perm.iter().map(|&p| old_indices[p]).collect();

                // Convert new indices to flat index
                let new_flat_idx: usize = new_indices
                    .iter()
                    .zip(new_strides.iter())
                    .map(|(idx, stride)| idx * stride)
                    .sum();

                result[new_flat_idx] = data_val;
            }

            Ok(XdlValue::MultiDimArray {
                data: result,
                shape: new_shape,
            })
        }
        XdlValue::Array(_) => {
            // 1D array - permutation has no effect
            Ok(args[0].clone())
        }
        _ => Err(XdlError::TypeMismatch {
            expected: "array".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}
