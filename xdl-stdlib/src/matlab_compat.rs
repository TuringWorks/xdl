//! MATLAB Compatibility Functions
//!
//! This module provides MATLAB-style functions that are commonly used but not
//! available in standard IDL/GDL. These extend XDL's capabilities for users
//! coming from MATLAB.

use crate::XdlResult;
use xdl_core::{XdlError, XdlValue};

/// MESHGRID - Create 2D coordinate matrices from coordinate vectors
///
/// Usage:
///   MESHGRID, x_vec, y_vec, X, Y
///
/// Creates 2D coordinate matrices X and Y from 1D coordinate vectors x_vec and y_vec.
/// This is equivalent to MATLAB's [X, Y] = meshgrid(x_vec, y_vec).
///
/// Example:
///   x = FINDGEN(5)  ; [0, 1, 2, 3, 4]
///   y = FINDGEN(3)  ; [0, 1, 2]
///   MESHGRID, x, y, X, Y
///   ; X will be 5x3 matrix with rows = [0,1,2,3,4]
///   ; Y will be 5x3 matrix with cols = [0,1,2]
pub fn meshgrid(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 2 {
        return Err(XdlError::InvalidArgument(format!(
            "MESHGRID: Expected 2 arguments (x_vec, y_vec), got {}",
            args.len()
        )));
    }

    // Extract input vectors
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

    // Create X matrix (rows are copies of x_vec)
    let mut x_data = Vec::with_capacity(nx * ny);
    for _j in 0..ny {
        x_data.extend_from_slice(&x_vec);
    }

    // Create Y matrix (columns are copies of y_vec)
    let mut y_data = Vec::with_capacity(nx * ny);
    #[allow(clippy::needless_range_loop)]
    for j in 0..ny {
        for _i in 0..nx {
            y_data.push(y_vec[j]);
        }
    }

    // Return as nested array [X, Y]
    Ok(XdlValue::NestedArray(vec![
        XdlValue::MultiDimArray {
            data: x_data,
            shape: vec![nx, ny],
        },
        XdlValue::MultiDimArray {
            data: y_data,
            shape: vec![nx, ny],
        },
    ]))
}

/// LINSPACE - Generate linearly spaced vector
///
/// Usage:
///   result = LINSPACE(start, stop, n)
///
/// Generates n evenly spaced values from start to stop (inclusive).
/// This is equivalent to MATLAB's linspace(start, stop, n).
///
/// Example:
///   x = LINSPACE(0, 10, 11)  ; [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
pub fn linspace(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 || args.len() > 3 {
        return Err(XdlError::InvalidArgument(format!(
            "LINSPACE: Expected 2-3 arguments (start, stop, [n]), got {}",
            args.len()
        )));
    }

    let start = args[0].to_double()?;
    let stop = args[1].to_double()?;
    let n = if args.len() == 3 {
        args[2].to_long()? as usize
    } else {
        100 // Default to 100 points like MATLAB
    };

    if n < 2 {
        return Err(XdlError::InvalidArgument(
            "LINSPACE: n must be at least 2".to_string(),
        ));
    }

    let step = (stop - start) / ((n - 1) as f64);
    let data: Vec<f64> = (0..n).map(|i| start + (i as f64) * step).collect();

    Ok(XdlValue::Array(data))
}

/// LOGSPACE - Generate logarithmically spaced vector
///
/// Usage:
///   result = LOGSPACE(start, stop, n)
///
/// Generates n logarithmically spaced values from 10^start to 10^stop.
/// This is equivalent to MATLAB's logspace(start, stop, n).
///
/// Example:
///   x = LOGSPACE(0, 3, 4)  ; [1, 10, 100, 1000]
pub fn logspace(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 || args.len() > 3 {
        return Err(XdlError::InvalidArgument(format!(
            "LOGSPACE: Expected 2-3 arguments (start, stop, [n]), got {}",
            args.len()
        )));
    }

    let start = args[0].to_double()?;
    let stop = args[1].to_double()?;
    let n = if args.len() == 3 {
        args[2].to_long()? as usize
    } else {
        50 // Default to 50 points like MATLAB
    };

    if n < 2 {
        return Err(XdlError::InvalidArgument(
            "LOGSPACE: n must be at least 2".to_string(),
        ));
    }

    let step = (stop - start) / ((n - 1) as f64);
    let data: Vec<f64> = (0..n)
        .map(|i| 10_f64.powf(start + (i as f64) * step))
        .collect();

    Ok(XdlValue::Array(data))
}

/// REPMAT - Replicate and tile an array
///
/// Usage:
///   result = REPMAT(array, m, n)
///
/// Replicates array m times vertically and n times horizontally.
/// This is equivalent to MATLAB's repmat(array, m, n).
///
/// Example:
///   a = [1, 2]
///   b = REPMAT(a, 2, 3)  ; Creates 2x6 array
pub fn repmat(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 3 {
        return Err(XdlError::InvalidArgument(format!(
            "REPMAT: Expected 3 arguments (array, m, n), got {}",
            args.len()
        )));
    }

    let arr = match &args[0] {
        XdlValue::Array(a) => a.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let m = args[1].to_long()? as usize;
    let n = args[2].to_long()? as usize;

    let orig_len = arr.len();
    let mut result = Vec::with_capacity(orig_len * m * n);

    for _ in 0..m {
        for _ in 0..n {
            result.extend_from_slice(&arr);
        }
    }

    Ok(XdlValue::Array(result))
}

/// SQUEEZE - Remove singleton dimensions
///
/// Usage:
///   result = SQUEEZE(array)
///
/// Removes dimensions of size 1 from multi-dimensional array.
/// This is equivalent to MATLAB's squeeze(array).
pub fn squeeze(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "SQUEEZE: Expected 1 argument, got {}",
            args.len()
        )));
    }

    match &args[0] {
        XdlValue::MultiDimArray { data, shape } => {
            let new_shape: Vec<usize> = shape.iter().filter(|&&s| s > 1).copied().collect();

            if new_shape.is_empty() {
                // All dimensions were 1, return scalar
                Ok(XdlValue::Double(data[0]))
            } else if new_shape.len() == 1 {
                // Reduced to 1D array
                Ok(XdlValue::Array(data.clone()))
            } else {
                // Still multi-dimensional
                Ok(XdlValue::MultiDimArray {
                    data: data.clone(),
                    shape: new_shape,
                })
            }
        }
        XdlValue::Array(_) => Ok(args[0].clone()), // Already 1D, no change
        _ => Ok(args[0].clone()),                  // Scalar, no change
    }
}

/// NDGRID - Generate N-D coordinate arrays
///
/// Usage:
///   NDGRID, x1, x2, ..., xn, X1, X2, ..., Xn
///
/// Similar to MESHGRID but follows matrix indexing convention.
/// For 2D case, NDGRID is transpose of MESHGRID.
pub fn ndgrid(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "NDGRID: Expected at least 2 input vectors".to_string(),
        ));
    }

    // For now, implement 2D case (can be extended to N-D)
    if args.len() == 2 {
        // Extract vectors
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

        // NDGRID: X varies along columns, Y varies along rows (opposite of MESHGRID)
        let mut x_data = Vec::with_capacity(nx * ny);
        #[allow(clippy::needless_range_loop)]
        for _i in 0..nx {
            for j in 0..ny {
                x_data.push(x_vec[j]);
            }
        }

        let mut y_data = Vec::with_capacity(nx * ny);
        #[allow(clippy::needless_range_loop)]
        for i in 0..nx {
            for _j in 0..ny {
                y_data.push(y_vec[i]);
            }
        }

        Ok(XdlValue::NestedArray(vec![
            XdlValue::MultiDimArray {
                data: x_data,
                shape: vec![nx, ny],
            },
            XdlValue::MultiDimArray {
                data: y_data,
                shape: vec![nx, ny],
            },
        ]))
    } else {
        Err(XdlError::NotImplemented(
            "NDGRID: N-D case (N > 2) not yet implemented".to_string(),
        ))
    }
}

/// INTERP1 - 1D interpolation
///
/// Usage:
///   result = INTERP1(x, y, xi, [method])
///
/// Interpolates to find yi = f(xi) where f is defined by (x, y) pairs.
/// Methods: 'linear' (default), 'nearest'
///
/// This is a simplified version of MATLAB's interp1.
pub fn interp1(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 || args.len() > 4 {
        return Err(XdlError::InvalidArgument(format!(
            "INTERP1: Expected 3-4 arguments (x, y, xi, [method]), got {}",
            args.len()
        )));
    }

    let x = match &args[0] {
        XdlValue::Array(arr) => arr.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let y = match &args[1] {
        XdlValue::Array(arr) => arr.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let xi = match &args[2] {
        XdlValue::Array(arr) => arr.clone(),
        XdlValue::Double(v) => vec![*v],
        XdlValue::Float(v) => vec![*v as f64],
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array or scalar".to_string(),
                actual: format!("{:?}", args[2].gdl_type()),
            })
        }
    };

    if x.len() != y.len() {
        return Err(XdlError::InvalidArgument(
            "INTERP1: x and y must have same length".to_string(),
        ));
    }

    // Linear interpolation (default)
    let mut yi = Vec::with_capacity(xi.len());

    for &xi_val in &xi {
        // Find surrounding points
        let mut idx = 0;
        for (i, &x_val) in x.iter().enumerate() {
            if x_val <= xi_val {
                idx = i;
            } else {
                break;
            }
        }

        // Interpolate
        let y_val = if idx == x.len() - 1 {
            // Beyond range, use last value
            y[idx]
        } else if xi_val == x[idx] {
            // Exact match
            y[idx]
        } else {
            // Linear interpolation
            let x0 = x[idx];
            let x1 = x[idx + 1];
            let y0 = y[idx];
            let y1 = y[idx + 1];
            y0 + (y1 - y0) * (xi_val - x0) / (x1 - x0)
        };

        yi.push(y_val);
    }

    Ok(XdlValue::Array(yi))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linspace() {
        let args = vec![
            XdlValue::Double(0.0),
            XdlValue::Double(10.0),
            XdlValue::Long(11),
        ];
        let result = linspace(&args).unwrap();

        if let XdlValue::Array(arr) = result {
            assert_eq!(arr.len(), 11);
            assert_eq!(arr[0], 0.0);
            assert_eq!(arr[10], 10.0);
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_logspace() {
        let args = vec![
            XdlValue::Double(0.0),
            XdlValue::Double(3.0),
            XdlValue::Long(4),
        ];
        let result = logspace(&args).unwrap();

        if let XdlValue::Array(arr) = result {
            assert_eq!(arr.len(), 4);
            assert!((arr[0] - 1.0).abs() < 1e-10);
            assert!((arr[3] - 1000.0).abs() < 1e-6);
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_meshgrid() {
        let x = XdlValue::Array(vec![1.0, 2.0, 3.0]);
        let y = XdlValue::Array(vec![4.0, 5.0]);

        let result = meshgrid(&[x, y]).unwrap();

        if let XdlValue::NestedArray(arrays) = result {
            assert_eq!(arrays.len(), 2);
        } else {
            panic!("Expected nested array result");
        }
    }
}
