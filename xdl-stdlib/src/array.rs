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

/// SHIFT - Circular shift of array elements
/// SHIFT(array, s1 [, s2, s3, ...])
/// Shifts array elements by the specified amount(s) along each dimension
/// Positive shift moves elements to higher indices (wrapping around)
///
/// Examples:
///   arr = [1, 2, 3, 4, 5]
///   SHIFT(arr, 2)   ; Returns [4, 5, 1, 2, 3]
///   SHIFT(arr, -1)  ; Returns [2, 3, 4, 5, 1]
pub fn shift_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(format!(
            "SHIFT: Expected at least 2 arguments (array, shift), got {}",
            args.len()
        )));
    }

    // Handle 1D arrays
    match &args[0] {
        XdlValue::Array(arr) => {
            if arr.is_empty() {
                return Ok(XdlValue::Array(vec![]));
            }

            let shift_amount = match &args[1] {
                XdlValue::Long(n) => *n,
                XdlValue::Int(n) => *n as i32,
                XdlValue::Double(n) => *n as i32,
                XdlValue::Float(n) => *n as i32,
                _ => {
                    return Err(XdlError::TypeMismatch {
                        expected: "integer".to_string(),
                        actual: format!("{:?}", args[1].gdl_type()),
                    })
                }
            };

            let n = arr.len() as i32;
            // Normalize shift to be within bounds
            let normalized_shift = ((shift_amount % n) + n) % n;
            let shift_idx = normalized_shift as usize;

            // Perform circular shift
            let mut result = vec![0.0; arr.len()];
            for (i, &val) in arr.iter().enumerate() {
                let new_idx = (i + shift_idx) % arr.len();
                result[new_idx] = val;
            }

            Ok(XdlValue::Array(result))
        }
        XdlValue::MultiDimArray { data, shape } => {
            // For multi-dimensional arrays, apply shift to first dimension
            if data.is_empty() {
                return Ok(XdlValue::MultiDimArray {
                    data: vec![],
                    shape: shape.clone(),
                });
            }

            let shift_amount = match &args[1] {
                XdlValue::Long(n) => *n,
                XdlValue::Int(n) => *n as i32,
                XdlValue::Double(n) => *n as i32,
                XdlValue::Float(n) => *n as i32,
                _ => {
                    return Err(XdlError::TypeMismatch {
                        expected: "integer".to_string(),
                        actual: format!("{:?}", args[1].gdl_type()),
                    })
                }
            };

            let n = data.len() as i32;
            let normalized_shift = ((shift_amount % n) + n) % n;
            let shift_idx = normalized_shift as usize;

            let mut result = vec![0.0; data.len()];
            for (i, &val) in data.iter().enumerate() {
                let new_idx = (i + shift_idx) % data.len();
                result[new_idx] = val;
            }

            Ok(XdlValue::MultiDimArray {
                data: result,
                shape: shape.clone(),
            })
        }
        _ => Err(XdlError::TypeMismatch {
            expected: "array".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// ROTATE - Rotate 2D array by 90, 180, or 270 degrees
/// ROTATE(array, direction)
/// direction: 0 = no rotation, 1 = 90° CCW, 2 = 180°, 3 = 270° CCW (90° CW)
///            4 = transpose, 5 = transpose + 90°, 6 = transpose + 180°, 7 = transpose + 270°
///
/// For 1D arrays, ROTATE just reverses the array when direction is 2.
///
/// Examples:
///   arr = [[1, 2], [3, 4]]  ; 2x2 matrix
///   ROTATE(arr, 1)          ; Rotate 90° counter-clockwise
pub fn rotate_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(format!(
            "ROTATE: Expected 2 arguments (array, direction), got {}",
            args.len()
        )));
    }

    let direction = match &args[1] {
        XdlValue::Long(n) => (*n % 8) as i32,
        XdlValue::Int(n) => (*n % 8) as i32,
        XdlValue::Double(n) => (*n as i32) % 8,
        XdlValue::Float(n) => (*n as i32) % 8,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    match &args[0] {
        XdlValue::Array(arr) => {
            if arr.is_empty() {
                return Ok(XdlValue::Array(vec![]));
            }

            match direction {
                0 => Ok(XdlValue::Array(arr.clone())),
                2 | 6 => {
                    // 180° rotation for 1D = reverse
                    let mut result = arr.clone();
                    result.reverse();
                    Ok(XdlValue::Array(result))
                }
                _ => {
                    // Other rotations don't make sense for 1D arrays
                    Ok(XdlValue::Array(arr.clone()))
                }
            }
        }
        XdlValue::MultiDimArray { data, shape } => {
            if shape.len() != 2 {
                return Err(XdlError::DimensionError(
                    "ROTATE: Only 2D arrays are supported".to_string(),
                ));
            }

            let nrows = shape[0];
            let ncols = shape[1];

            match direction {
                0 => Ok(XdlValue::MultiDimArray {
                    data: data.clone(),
                    shape: shape.clone(),
                }),
                1 => {
                    // 90° CCW: (i, j) -> (ncols - 1 - j, i)
                    let mut result = vec![0.0; data.len()];
                    for i in 0..nrows {
                        for j in 0..ncols {
                            let old_idx = i * ncols + j;
                            let new_i = ncols - 1 - j;
                            let new_j = i;
                            let new_idx = new_i * nrows + new_j;
                            result[new_idx] = data[old_idx];
                        }
                    }
                    Ok(XdlValue::MultiDimArray {
                        data: result,
                        shape: vec![ncols, nrows],
                    })
                }
                2 => {
                    // 180°: (i, j) -> (nrows - 1 - i, ncols - 1 - j)
                    let mut result = vec![0.0; data.len()];
                    for i in 0..nrows {
                        for j in 0..ncols {
                            let old_idx = i * ncols + j;
                            let new_idx = (nrows - 1 - i) * ncols + (ncols - 1 - j);
                            result[new_idx] = data[old_idx];
                        }
                    }
                    Ok(XdlValue::MultiDimArray {
                        data: result,
                        shape: shape.clone(),
                    })
                }
                3 => {
                    // 270° CCW (90° CW): (i, j) -> (j, nrows - 1 - i)
                    let mut result = vec![0.0; data.len()];
                    for i in 0..nrows {
                        for j in 0..ncols {
                            let old_idx = i * ncols + j;
                            let new_i = j;
                            let new_j = nrows - 1 - i;
                            let new_idx = new_i * nrows + new_j;
                            result[new_idx] = data[old_idx];
                        }
                    }
                    Ok(XdlValue::MultiDimArray {
                        data: result,
                        shape: vec![ncols, nrows],
                    })
                }
                4 => {
                    // Transpose: (i, j) -> (j, i)
                    let result = transpose_2d(data, nrows, ncols)?;
                    Ok(XdlValue::MultiDimArray {
                        data: result,
                        shape: vec![ncols, nrows],
                    })
                }
                5 | 6 | 7 => {
                    // Transpose + rotation: first transpose, then rotate
                    let transposed = transpose_2d(data, nrows, ncols)?;
                    let rot_dir = direction - 4;
                    let transposed_val = XdlValue::MultiDimArray {
                        data: transposed,
                        shape: vec![ncols, nrows],
                    };
                    rotate_func(&[transposed_val, XdlValue::Long(rot_dir)])
                }
                _ => Ok(XdlValue::MultiDimArray {
                    data: data.clone(),
                    shape: shape.clone(),
                }),
            }
        }
        _ => Err(XdlError::TypeMismatch {
            expected: "array".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// REPLICATE - Create array by replicating a value or array
/// REPLICATE(value, d1 [, d2, d3, ...])
/// Creates an array with the specified dimensions filled with copies of value
///
/// Examples:
///   REPLICATE(3.14, 5)       ; Returns [3.14, 3.14, 3.14, 3.14, 3.14]
///   REPLICATE(0, 3, 3)       ; Returns 3x3 array of zeros
pub fn replicate_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(format!(
            "REPLICATE: Expected at least 2 arguments (value, dimensions...), got {}",
            args.len()
        )));
    }

    // Get the value to replicate
    let value = match &args[0] {
        XdlValue::Double(v) => *v,
        XdlValue::Float(v) => *v as f64,
        XdlValue::Long(v) => *v as f64,
        XdlValue::Int(v) => *v as f64,
        XdlValue::Byte(v) => *v as f64,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    // Extract dimensions
    let mut shape = Vec::new();
    for arg in args.iter().skip(1) {
        shape.push(extract_dimension(arg)?);
    }

    // Calculate total size
    let total_size: usize = shape.iter().product();
    let data = vec![value; total_size];

    // Return appropriate type based on dimensions
    if shape.len() == 1 {
        Ok(XdlValue::Array(data))
    } else {
        Ok(XdlValue::MultiDimArray { data, shape })
    }
}

/// MAKE_ARRAY - General array creation with various options
/// MAKE_ARRAY([d1, d2, ...] [, /TYPE] [, VALUE=value] [, /INDEX])
/// Creates an array with specified dimensions and initialization
///
/// Simplified implementation supporting basic usage patterns
pub fn make_array_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "MAKE_ARRAY: At least one dimension required".to_string(),
        ));
    }

    // Extract dimensions
    let mut shape = Vec::new();
    for arg in args {
        shape.push(extract_dimension(arg)?);
    }

    let total_size: usize = shape.iter().product();

    // Default to zeros (like most array creation functions)
    let data = vec![0.0; total_size];

    if shape.len() == 1 {
        Ok(XdlValue::Array(data))
    } else {
        Ok(XdlValue::MultiDimArray { data, shape })
    }
}

/// ARRAY_EQUAL - Test if two arrays are equal
/// ARRAY_EQUAL(array1, array2)
/// Returns 1 if arrays are equal (same size and all elements match), 0 otherwise
pub fn array_equal_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 2 {
        return Err(XdlError::InvalidArgument(format!(
            "ARRAY_EQUAL: Expected 2 arguments, got {}",
            args.len()
        )));
    }

    let equal = match (&args[0], &args[1]) {
        (XdlValue::Array(a), XdlValue::Array(b)) => {
            if a.len() != b.len() {
                false
            } else {
                a.iter().zip(b.iter()).all(|(x, y)| (x - y).abs() < f64::EPSILON)
            }
        }
        (XdlValue::MultiDimArray { data: a, shape: sa }, XdlValue::MultiDimArray { data: b, shape: sb }) => {
            if sa != sb || a.len() != b.len() {
                false
            } else {
                a.iter().zip(b.iter()).all(|(x, y)| (x - y).abs() < f64::EPSILON)
            }
        }
        // Scalar comparisons
        (a, b) => {
            if let (Ok(x), Ok(y)) = (a.to_double(), b.to_double()) {
                (x - y).abs() < f64::EPSILON
            } else {
                false
            }
        }
    };

    Ok(XdlValue::Long(if equal { 1 } else { 0 }))
}

/// UNIQ - Return indices of unique elements in a sorted array
/// UNIQ(array [, indices])
/// Returns indices of unique (non-repeating) elements
/// Note: Array should be sorted for proper behavior (like IDL)
///
/// Examples:
///   arr = [1, 1, 2, 2, 2, 3, 4, 4]
///   UNIQ(arr)  ; Returns [1, 4, 5, 7] (last index of each unique run)
pub fn uniq_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "UNIQ: Expected at least 1 argument".to_string(),
        ));
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
        return Ok(XdlValue::Long(-1));
    }

    if arr.len() == 1 {
        return Ok(XdlValue::Long(0));
    }

    // Find indices where value changes (or last element)
    let mut indices = Vec::new();
    for i in 0..arr.len() - 1 {
        if (arr[i] - arr[i + 1]).abs() > f64::EPSILON {
            indices.push(i as f64);
        }
    }
    // Always include the last element
    indices.push((arr.len() - 1) as f64);

    if indices.len() == 1 {
        Ok(XdlValue::Long(indices[0] as i32))
    } else {
        Ok(XdlValue::Array(indices))
    }
}

/// HISTOGRAM - Compute histogram of array values
/// HISTOGRAM(array [, BINSIZE=value] [, MIN=value] [, MAX=value] [, NBINS=value])
/// Returns array of counts for each bin
///
/// Simplified implementation with default binning
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

    // Find min and max
    let min_val = arr.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_val = arr.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

    // Default: 256 bins (like IDL for byte data)
    let nbins = if args.len() > 1 {
        match &args[1] {
            XdlValue::Long(n) => *n as usize,
            XdlValue::Int(n) => *n as usize,
            _ => 256,
        }
    } else {
        // Compute reasonable number of bins
        let range = max_val - min_val;
        if range <= 0.0 {
            1
        } else {
            (range.ceil() as usize).max(1).min(256)
        }
    };

    let bin_size = (max_val - min_val) / nbins as f64;

    // Count elements in each bin
    let mut counts = vec![0.0; nbins];
    for &val in arr {
        let bin = if bin_size > 0.0 {
            ((val - min_val) / bin_size).floor() as usize
        } else {
            0
        };
        let bin = bin.min(nbins - 1); // Clamp to last bin
        counts[bin] += 1.0;
    }

    Ok(XdlValue::Array(counts))
}

/// REBIN - Resize array by averaging or replicating
/// Syntax: result = REBIN(array, new_dim1 [, new_dim2, ...])
pub fn rebin_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "REBIN: Expected array and at least one dimension".to_string(),
        ));
    }

    // For 1D arrays
    if let XdlValue::Array(arr) = &args[0] {
        let new_len = match &args[1] {
            XdlValue::Long(n) => *n as usize,
            XdlValue::Int(n) => *n as usize,
            _ => {
                return Err(XdlError::TypeMismatch {
                    expected: "integer".to_string(),
                    actual: format!("{:?}", args[1].gdl_type()),
                })
            }
        };

        if new_len == 0 {
            return Ok(XdlValue::Array(vec![]));
        }

        let old_len = arr.len();
        let mut result = vec![0.0; new_len];

        if new_len <= old_len {
            // Shrinking: average values
            let factor = old_len as f64 / new_len as f64;
            for i in 0..new_len {
                let start = (i as f64 * factor) as usize;
                let end = ((i + 1) as f64 * factor) as usize;
                let count = (end - start).max(1);
                let sum: f64 = arr[start..end.min(old_len)].iter().sum();
                result[i] = sum / count as f64;
            }
        } else {
            // Expanding: replicate values
            let factor = old_len as f64 / new_len as f64;
            for i in 0..new_len {
                let src_idx = ((i as f64 * factor) as usize).min(old_len - 1);
                result[i] = arr[src_idx];
            }
        }

        return Ok(XdlValue::Array(result));
    }

    // For MultiDimArrays
    if let XdlValue::MultiDimArray { data, shape } = &args[0] {
        // Collect new dimensions
        let mut new_shape = Vec::new();
        for i in 1..args.len() {
            let dim = match &args[i] {
                XdlValue::Long(n) => *n as usize,
                XdlValue::Int(n) => *n as usize,
                _ => {
                    return Err(XdlError::TypeMismatch {
                        expected: "integer".to_string(),
                        actual: format!("{:?}", args[i].gdl_type()),
                    })
                }
            };
            new_shape.push(dim);
        }

        // Pad new_shape if fewer dimensions given
        while new_shape.len() < shape.len() {
            new_shape.push(shape[new_shape.len()]);
        }

        // For 2D arrays, do proper rebin
        if shape.len() == 2 && new_shape.len() == 2 {
            let (old_rows, old_cols) = (shape[0], shape[1]);
            let (new_rows, new_cols) = (new_shape[0], new_shape[1]);

            let mut result = vec![0.0; new_rows * new_cols];

            let row_factor = old_rows as f64 / new_rows as f64;
            let col_factor = old_cols as f64 / new_cols as f64;

            for new_row in 0..new_rows {
                for new_col in 0..new_cols {
                    let start_row = (new_row as f64 * row_factor) as usize;
                    let end_row = (((new_row + 1) as f64 * row_factor) as usize).min(old_rows);
                    let start_col = (new_col as f64 * col_factor) as usize;
                    let end_col = (((new_col + 1) as f64 * col_factor) as usize).min(old_cols);

                    let mut sum = 0.0;
                    let mut count = 0;
                    for r in start_row..end_row {
                        for c in start_col..end_col {
                            sum += data[r * old_cols + c];
                            count += 1;
                        }
                    }

                    result[new_row * new_cols + new_col] = if count > 0 {
                        sum / count as f64
                    } else {
                        // Handle upsampling: nearest neighbor
                        let src_row = (start_row).min(old_rows - 1);
                        let src_col = (start_col).min(old_cols - 1);
                        data[src_row * old_cols + src_col]
                    };
                }
            }

            return Ok(XdlValue::MultiDimArray {
                data: result,
                shape: new_shape,
            });
        }

        // For other dimensions, just return with new shape (simplified)
        return Ok(XdlValue::MultiDimArray {
            data: data.clone(),
            shape: new_shape,
        });
    }

    Err(XdlError::TypeMismatch {
        expected: "array".to_string(),
        actual: format!("{:?}", args[0].gdl_type()),
    })
}

/// CONGRID - Resize array with interpolation
/// Syntax: result = CONGRID(array, new_dim1 [, new_dim2, ...] [, /INTERP])
pub fn congrid_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "CONGRID: Expected array and at least one dimension".to_string(),
        ));
    }

    // For 1D arrays
    if let XdlValue::Array(arr) = &args[0] {
        let new_len = match &args[1] {
            XdlValue::Long(n) => *n as usize,
            XdlValue::Int(n) => *n as usize,
            _ => {
                return Err(XdlError::TypeMismatch {
                    expected: "integer".to_string(),
                    actual: format!("{:?}", args[1].gdl_type()),
                })
            }
        };

        if new_len == 0 || arr.is_empty() {
            return Ok(XdlValue::Array(vec![]));
        }

        // Use linear interpolation
        let mut result = vec![0.0; new_len];
        let scale = (arr.len() - 1) as f64 / (new_len - 1).max(1) as f64;

        for i in 0..new_len {
            let src_pos = i as f64 * scale;
            let src_idx = src_pos.floor() as usize;
            let frac = src_pos - src_idx as f64;

            if src_idx + 1 < arr.len() {
                result[i] = arr[src_idx] * (1.0 - frac) + arr[src_idx + 1] * frac;
            } else {
                result[i] = arr[arr.len() - 1];
            }
        }

        return Ok(XdlValue::Array(result));
    }

    // For MultiDimArrays
    if let XdlValue::MultiDimArray { data, shape } = &args[0] {
        // Collect new dimensions
        let mut new_shape = Vec::new();
        for i in 1..args.len() {
            let dim = match &args[i] {
                XdlValue::Long(n) => *n as usize,
                XdlValue::Int(n) => *n as usize,
                _ => continue, // Skip non-integer args (could be keywords)
            };
            new_shape.push(dim);
        }

        // Pad new_shape if fewer dimensions given
        while new_shape.len() < shape.len() {
            new_shape.push(shape[new_shape.len()]);
        }

        // For 2D arrays, bilinear interpolation
        if shape.len() == 2 && new_shape.len() == 2 {
            let (old_rows, old_cols) = (shape[0], shape[1]);
            let (new_rows, new_cols) = (new_shape[0], new_shape[1]);

            let mut result = vec![0.0; new_rows * new_cols];

            let row_scale = (old_rows - 1).max(1) as f64 / (new_rows - 1).max(1) as f64;
            let col_scale = (old_cols - 1).max(1) as f64 / (new_cols - 1).max(1) as f64;

            for new_row in 0..new_rows {
                for new_col in 0..new_cols {
                    let src_row = new_row as f64 * row_scale;
                    let src_col = new_col as f64 * col_scale;

                    let r0 = (src_row.floor() as usize).min(old_rows - 1);
                    let r1 = (r0 + 1).min(old_rows - 1);
                    let c0 = (src_col.floor() as usize).min(old_cols - 1);
                    let c1 = (c0 + 1).min(old_cols - 1);

                    let row_frac = src_row - r0 as f64;
                    let col_frac = src_col - c0 as f64;

                    // Bilinear interpolation
                    let v00 = data[r0 * old_cols + c0];
                    let v01 = data[r0 * old_cols + c1];
                    let v10 = data[r1 * old_cols + c0];
                    let v11 = data[r1 * old_cols + c1];

                    let top = v00 * (1.0 - col_frac) + v01 * col_frac;
                    let bottom = v10 * (1.0 - col_frac) + v11 * col_frac;
                    let value = top * (1.0 - row_frac) + bottom * row_frac;

                    result[new_row * new_cols + new_col] = value;
                }
            }

            return Ok(XdlValue::MultiDimArray {
                data: result,
                shape: new_shape,
            });
        }

        // For other dimensions, fall back to simple resize
        return Ok(XdlValue::MultiDimArray {
            data: data.clone(),
            shape: new_shape,
        });
    }

    Err(XdlError::TypeMismatch {
        expected: "array".to_string(),
        actual: format!("{:?}", args[0].gdl_type()),
    })
}

/// CUMSUM - Cumulative sum of array elements
pub fn cumsum_func(
    args: &[XdlValue],
    _keywords: &std::collections::HashMap<String, XdlValue>,
) -> Result<XdlValue, XdlError> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument("CUMSUM requires an array argument".to_string()));
    }

    let data = match &args[0] {
        XdlValue::Array(arr) => arr.clone(),
        XdlValue::MultiDimArray { data, shape: _ } => data.clone(),
        _ => return Err(XdlError::TypeMismatch {
            expected: "array".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    };

    let mut result = Vec::with_capacity(data.len());
    let mut sum = 0.0;
    for val in data {
        sum += val;
        result.push(sum);
    }

    Ok(XdlValue::Array(result))
}

/// CUMPROD - Cumulative product of array elements
pub fn cumprod_func(
    args: &[XdlValue],
    _keywords: &std::collections::HashMap<String, XdlValue>,
) -> Result<XdlValue, XdlError> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument("CUMPROD requires an array argument".to_string()));
    }

    let data = match &args[0] {
        XdlValue::Array(arr) => arr.clone(),
        XdlValue::MultiDimArray { data, shape: _ } => data.clone(),
        _ => return Err(XdlError::TypeMismatch {
            expected: "array".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    };

    let mut result = Vec::with_capacity(data.len());
    let mut prod = 1.0;
    for val in data {
        prod *= val;
        result.push(prod);
    }

    Ok(XdlValue::Array(result))
}

/// ARGMIN - Index of minimum value in array
pub fn argmin_func(
    args: &[XdlValue],
    _keywords: &std::collections::HashMap<String, XdlValue>,
) -> Result<XdlValue, XdlError> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument("ARGMIN requires an array argument".to_string()));
    }

    let data = match &args[0] {
        XdlValue::Array(arr) => arr.clone(),
        XdlValue::MultiDimArray { data, shape: _ } => data.clone(),
        _ => return Err(XdlError::TypeMismatch {
            expected: "array".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    };

    if data.is_empty() {
        return Ok(XdlValue::Long(-1));
    }

    let mut min_idx = 0usize;
    let mut min_val = data[0];
    for (i, &val) in data.iter().enumerate() {
        if val < min_val {
            min_val = val;
            min_idx = i;
        }
    }

    Ok(XdlValue::Long(min_idx as i32))
}

/// ARGMAX - Index of maximum value in array
pub fn argmax_func(
    args: &[XdlValue],
    _keywords: &std::collections::HashMap<String, XdlValue>,
) -> Result<XdlValue, XdlError> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument("ARGMAX requires an array argument".to_string()));
    }

    let data = match &args[0] {
        XdlValue::Array(arr) => arr.clone(),
        XdlValue::MultiDimArray { data, shape: _ } => data.clone(),
        _ => return Err(XdlError::TypeMismatch {
            expected: "array".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    };

    if data.is_empty() {
        return Ok(XdlValue::Long(-1));
    }

    let mut max_idx = 0usize;
    let mut max_val = data[0];
    for (i, &val) in data.iter().enumerate() {
        if val > max_val {
            max_val = val;
            max_idx = i;
        }
    }

    Ok(XdlValue::Long(max_idx as i32))
}

/// DIFF - Differences between consecutive elements
pub fn diff_func(
    args: &[XdlValue],
    _keywords: &std::collections::HashMap<String, XdlValue>,
) -> Result<XdlValue, XdlError> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument("DIFF requires an array argument".to_string()));
    }

    let data = match &args[0] {
        XdlValue::Array(arr) => arr.clone(),
        XdlValue::MultiDimArray { data, shape: _ } => data.clone(),
        _ => return Err(XdlError::TypeMismatch {
            expected: "array".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    };

    if data.len() < 2 {
        return Ok(XdlValue::Array(vec![]));
    }

    let mut result = Vec::with_capacity(data.len() - 1);
    for i in 1..data.len() {
        result.push(data[i] - data[i - 1]);
    }

    Ok(XdlValue::Array(result))
}

/// APPEND - Append arrays together
pub fn append_func(
    args: &[XdlValue],
    _keywords: &std::collections::HashMap<String, XdlValue>,
) -> Result<XdlValue, XdlError> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument("APPEND requires at least two array arguments".to_string()));
    }

    let mut result = Vec::new();

    for arg in args {
        match arg {
            XdlValue::Array(arr) => result.extend(arr.iter().cloned()),
            XdlValue::MultiDimArray { data, shape: _ } => result.extend(data.iter().cloned()),
            XdlValue::Float(f) => result.push(*f as f64),
            XdlValue::Double(d) => result.push(*d),
            XdlValue::Int(i) => result.push(*i as f64),
            XdlValue::Long(l) => result.push(*l as f64),
            _ => return Err(XdlError::TypeMismatch {
                expected: "array or scalar".to_string(),
                actual: format!("{:?}", arg.gdl_type()),
            }),
        }
    }

    Ok(XdlValue::Array(result))
}

/// ANY - Test if any element is non-zero (true)
pub fn any_func(
    args: &[XdlValue],
    _keywords: &std::collections::HashMap<String, XdlValue>,
) -> Result<XdlValue, XdlError> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument("ANY requires an array argument".to_string()));
    }

    let data = match &args[0] {
        XdlValue::Array(arr) => arr.clone(),
        XdlValue::MultiDimArray { data, shape: _ } => data.clone(),
        _ => return Err(XdlError::TypeMismatch {
            expected: "array".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    };

    let any_true = data.iter().any(|&x| x != 0.0);
    Ok(XdlValue::Int(if any_true { 1 } else { 0 }))
}

/// ALL - Test if all elements are non-zero (true)
pub fn all_func(
    args: &[XdlValue],
    _keywords: &std::collections::HashMap<String, XdlValue>,
) -> Result<XdlValue, XdlError> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument("ALL requires an array argument".to_string()));
    }

    let data = match &args[0] {
        XdlValue::Array(arr) => arr.clone(),
        XdlValue::MultiDimArray { data, shape: _ } => data.clone(),
        _ => return Err(XdlError::TypeMismatch {
            expected: "array".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    };

    let all_true = data.iter().all(|&x| x != 0.0);
    Ok(XdlValue::Int(if all_true { 1 } else { 0 }))
}

/// FLATTEN - Flatten multi-dimensional array to 1D
pub fn flatten_func(
    args: &[XdlValue],
    _keywords: &std::collections::HashMap<String, XdlValue>,
) -> Result<XdlValue, XdlError> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument("FLATTEN requires an array argument".to_string()));
    }

    let data = match &args[0] {
        XdlValue::Array(arr) => arr.clone(),
        XdlValue::MultiDimArray { data, shape: _ } => data.clone(),
        _ => return Err(XdlError::TypeMismatch {
            expected: "array".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    };

    Ok(XdlValue::Array(data))
}

/// NONZERO - Return indices of non-zero elements
pub fn nonzero_func(
    args: &[XdlValue],
    _keywords: &std::collections::HashMap<String, XdlValue>,
) -> Result<XdlValue, XdlError> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument("NONZERO requires an array argument".to_string()));
    }

    let data = match &args[0] {
        XdlValue::Array(arr) => arr.clone(),
        XdlValue::MultiDimArray { data, shape: _ } => data.clone(),
        _ => return Err(XdlError::TypeMismatch {
            expected: "array".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    };

    let indices: Vec<f64> = data
        .iter()
        .enumerate()
        .filter(|(_, &v)| v != 0.0)
        .map(|(i, _)| i as f64)
        .collect();

    Ok(XdlValue::Array(indices))
}

/// CLIP - Clip array values to range [min, max]
pub fn clip_func(
    args: &[XdlValue],
    _keywords: &std::collections::HashMap<String, XdlValue>,
) -> Result<XdlValue, XdlError> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument("CLIP requires array, min, max arguments".to_string()));
    }

    let data = match &args[0] {
        XdlValue::Array(arr) => arr.clone(),
        XdlValue::MultiDimArray { data, shape: _ } => data.clone(),
        _ => return Err(XdlError::TypeMismatch {
            expected: "array".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    };

    let min_val = match &args[1] {
        XdlValue::Float(f) => *f as f64,
        XdlValue::Double(d) => *d,
        XdlValue::Int(i) => *i as f64,
        XdlValue::Long(l) => *l as f64,
        _ => return Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[1].gdl_type()),
        }),
    };

    let max_val = match &args[2] {
        XdlValue::Float(f) => *f as f64,
        XdlValue::Double(d) => *d,
        XdlValue::Int(i) => *i as f64,
        XdlValue::Long(l) => *l as f64,
        _ => return Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[2].gdl_type()),
        }),
    };

    let result: Vec<f64> = data
        .iter()
        .map(|&x| x.max(min_val).min(max_val))
        .collect();

    Ok(XdlValue::Array(result))
}

/// LINSPACE - Create linearly spaced array
pub fn linspace_func(
    args: &[XdlValue],
    _keywords: &std::collections::HashMap<String, XdlValue>,
) -> Result<XdlValue, XdlError> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument("LINSPACE requires start, stop, num arguments".to_string()));
    }

    let start = match &args[0] {
        XdlValue::Float(f) => *f as f64,
        XdlValue::Double(d) => *d,
        XdlValue::Int(i) => *i as f64,
        XdlValue::Long(l) => *l as f64,
        _ => return Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    };

    let stop = match &args[1] {
        XdlValue::Float(f) => *f as f64,
        XdlValue::Double(d) => *d,
        XdlValue::Int(i) => *i as f64,
        XdlValue::Long(l) => *l as f64,
        _ => return Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[1].gdl_type()),
        }),
    };

    let num = match &args[2] {
        XdlValue::Int(i) => *i as usize,
        XdlValue::Long(l) => *l as usize,
        XdlValue::Float(f) => *f as usize,
        XdlValue::Double(d) => *d as usize,
        _ => return Err(XdlError::TypeMismatch {
            expected: "integer".to_string(),
            actual: format!("{:?}", args[2].gdl_type()),
        }),
    };

    if num < 2 {
        return Ok(XdlValue::Array(vec![start]));
    }

    let step = (stop - start) / (num - 1) as f64;
    let result: Vec<f64> = (0..num).map(|i| start + step * i as f64).collect();

    Ok(XdlValue::Array(result))
}

/// LOGSPACE - Create logarithmically spaced array
pub fn logspace_func(
    args: &[XdlValue],
    _keywords: &std::collections::HashMap<String, XdlValue>,
) -> Result<XdlValue, XdlError> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument("LOGSPACE requires start_exp, stop_exp, num arguments".to_string()));
    }

    let start_exp = match &args[0] {
        XdlValue::Float(f) => *f as f64,
        XdlValue::Double(d) => *d,
        XdlValue::Int(i) => *i as f64,
        XdlValue::Long(l) => *l as f64,
        _ => return Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    };

    let stop_exp = match &args[1] {
        XdlValue::Float(f) => *f as f64,
        XdlValue::Double(d) => *d,
        XdlValue::Int(i) => *i as f64,
        XdlValue::Long(l) => *l as f64,
        _ => return Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[1].gdl_type()),
        }),
    };

    let num = match &args[2] {
        XdlValue::Int(i) => *i as usize,
        XdlValue::Long(l) => *l as usize,
        XdlValue::Float(f) => *f as usize,
        XdlValue::Double(d) => *d as usize,
        _ => return Err(XdlError::TypeMismatch {
            expected: "integer".to_string(),
            actual: format!("{:?}", args[2].gdl_type()),
        }),
    };

    if num < 2 {
        return Ok(XdlValue::Array(vec![10.0_f64.powf(start_exp)]));
    }

    let step = (stop_exp - start_exp) / (num - 1) as f64;
    let result: Vec<f64> = (0..num)
        .map(|i| 10.0_f64.powf(start_exp + step * i as f64))
        .collect();

    Ok(XdlValue::Array(result))
}

/// ARANGE - Create array with evenly spaced values (like Python numpy.arange)
pub fn arange_func(
    args: &[XdlValue],
    _keywords: &std::collections::HashMap<String, XdlValue>,
) -> Result<XdlValue, XdlError> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument("ARANGE requires at least one argument".to_string()));
    }

    let (start, stop, step) = match args.len() {
        1 => {
            let stop = match &args[0] {
                XdlValue::Float(f) => *f as f64,
                XdlValue::Double(d) => *d,
                XdlValue::Int(i) => *i as f64,
                XdlValue::Long(l) => *l as f64,
                _ => return Err(XdlError::TypeMismatch {
                    expected: "numeric".to_string(),
                    actual: format!("{:?}", args[0].gdl_type()),
                }),
            };
            (0.0, stop, 1.0)
        }
        2 => {
            let start = match &args[0] {
                XdlValue::Float(f) => *f as f64,
                XdlValue::Double(d) => *d,
                XdlValue::Int(i) => *i as f64,
                XdlValue::Long(l) => *l as f64,
                _ => return Err(XdlError::TypeMismatch {
                    expected: "numeric".to_string(),
                    actual: format!("{:?}", args[0].gdl_type()),
                }),
            };
            let stop = match &args[1] {
                XdlValue::Float(f) => *f as f64,
                XdlValue::Double(d) => *d,
                XdlValue::Int(i) => *i as f64,
                XdlValue::Long(l) => *l as f64,
                _ => return Err(XdlError::TypeMismatch {
                    expected: "numeric".to_string(),
                    actual: format!("{:?}", args[1].gdl_type()),
                }),
            };
            (start, stop, 1.0)
        }
        _ => {
            let start = match &args[0] {
                XdlValue::Float(f) => *f as f64,
                XdlValue::Double(d) => *d,
                XdlValue::Int(i) => *i as f64,
                XdlValue::Long(l) => *l as f64,
                _ => return Err(XdlError::TypeMismatch {
                    expected: "numeric".to_string(),
                    actual: format!("{:?}", args[0].gdl_type()),
                }),
            };
            let stop = match &args[1] {
                XdlValue::Float(f) => *f as f64,
                XdlValue::Double(d) => *d,
                XdlValue::Int(i) => *i as f64,
                XdlValue::Long(l) => *l as f64,
                _ => return Err(XdlError::TypeMismatch {
                    expected: "numeric".to_string(),
                    actual: format!("{:?}", args[1].gdl_type()),
                }),
            };
            let step = match &args[2] {
                XdlValue::Float(f) => *f as f64,
                XdlValue::Double(d) => *d,
                XdlValue::Int(i) => *i as f64,
                XdlValue::Long(l) => *l as f64,
                _ => return Err(XdlError::TypeMismatch {
                    expected: "numeric".to_string(),
                    actual: format!("{:?}", args[2].gdl_type()),
                }),
            };
            (start, stop, step)
        }
    };

    if step == 0.0 {
        return Err(XdlError::InvalidArgument("ARANGE step cannot be zero".to_string()));
    }

    let mut result = Vec::new();
    let mut val = start;
    if step > 0.0 {
        while val < stop {
            result.push(val);
            val += step;
        }
    } else {
        while val > stop {
            result.push(val);
            val += step;
        }
    }

    Ok(XdlValue::Array(result))
}

/// SEARCHSORTED - Find indices where elements should be inserted to maintain order
pub fn searchsorted_func(
    args: &[XdlValue],
    _keywords: &std::collections::HashMap<String, XdlValue>,
) -> Result<XdlValue, XdlError> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument("SEARCHSORTED requires sorted_array and values arguments".to_string()));
    }

    let sorted_arr = match &args[0] {
        XdlValue::Array(arr) => arr.clone(),
        XdlValue::MultiDimArray { data, shape: _ } => data.clone(),
        _ => return Err(XdlError::TypeMismatch {
            expected: "array".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    };

    let values = match &args[1] {
        XdlValue::Array(arr) => arr.clone(),
        XdlValue::MultiDimArray { data, shape: _ } => data.clone(),
        XdlValue::Float(f) => vec![*f as f64],
        XdlValue::Double(d) => vec![*d],
        XdlValue::Int(i) => vec![*i as f64],
        XdlValue::Long(l) => vec![*l as f64],
        _ => return Err(XdlError::TypeMismatch {
            expected: "array or scalar".to_string(),
            actual: format!("{:?}", args[1].gdl_type()),
        }),
    };

    let indices: Vec<f64> = values
        .iter()
        .map(|&val| {
            match sorted_arr.binary_search_by(|x| x.partial_cmp(&val).unwrap_or(std::cmp::Ordering::Equal)) {
                Ok(i) => i as f64,
                Err(i) => i as f64,
            }
        })
        .collect();

    Ok(XdlValue::Array(indices))
}

/// DIGITIZE - Return indices of bins to which each value belongs
pub fn digitize_func(
    args: &[XdlValue],
    _keywords: &std::collections::HashMap<String, XdlValue>,
) -> Result<XdlValue, XdlError> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument("DIGITIZE requires array and bins arguments".to_string()));
    }

    let data = match &args[0] {
        XdlValue::Array(arr) => arr.clone(),
        XdlValue::MultiDimArray { data, shape: _ } => data.clone(),
        _ => return Err(XdlError::TypeMismatch {
            expected: "array".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    };

    let bins = match &args[1] {
        XdlValue::Array(arr) => arr.clone(),
        XdlValue::MultiDimArray { data, shape: _ } => data.clone(),
        _ => return Err(XdlError::TypeMismatch {
            expected: "array".to_string(),
            actual: format!("{:?}", args[1].gdl_type()),
        }),
    };

    let indices: Vec<f64> = data
        .iter()
        .map(|&val| {
            match bins.binary_search_by(|x| x.partial_cmp(&val).unwrap_or(std::cmp::Ordering::Equal)) {
                Ok(i) => i as f64,
                Err(i) => i as f64,
            }
        })
        .collect();

    Ok(XdlValue::Array(indices))
}

/// TILE - Repeat array along each dimension
pub fn tile_func(
    args: &[XdlValue],
    _keywords: &std::collections::HashMap<String, XdlValue>,
) -> Result<XdlValue, XdlError> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument("TILE requires array and reps arguments".to_string()));
    }

    let data = match &args[0] {
        XdlValue::Array(arr) => arr.clone(),
        XdlValue::MultiDimArray { data, shape: _ } => data.clone(),
        _ => return Err(XdlError::TypeMismatch {
            expected: "array".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    };

    let reps = match &args[1] {
        XdlValue::Int(i) => *i as usize,
        XdlValue::Long(l) => *l as usize,
        XdlValue::Float(f) => *f as usize,
        XdlValue::Double(d) => *d as usize,
        _ => return Err(XdlError::TypeMismatch {
            expected: "integer".to_string(),
            actual: format!("{:?}", args[1].gdl_type()),
        }),
    };

    let mut result = Vec::with_capacity(data.len() * reps);
    for _ in 0..reps {
        result.extend(data.iter().cloned());
    }

    Ok(XdlValue::Array(result))
}
