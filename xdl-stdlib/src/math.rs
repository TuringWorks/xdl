//! Mathematical functions

use std::collections::HashMap;
use xdl_core::{GdlType, XdlError, XdlResult, XdlValue};

/// Convert XdlValue to f64 for mathematical operations
fn to_float(value: &XdlValue) -> XdlResult<f64> {
    match value {
        XdlValue::Byte(v) => Ok(*v as f64),
        XdlValue::Int(v) => Ok(*v as f64),
        XdlValue::Long(v) => Ok(*v as f64),
        XdlValue::Float(v) => Ok(*v as f64),
        XdlValue::Double(v) => Ok(*v),
        XdlValue::UInt(v) => Ok(*v as f64),
        XdlValue::ULong(v) => Ok(*v as f64),
        XdlValue::Long64(v) => Ok(*v as f64),
        XdlValue::ULong64(v) => Ok(*v as f64),
        _ => Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", value.gdl_type()),
        }),
    }
}

/// Convert f64 result back to appropriate XdlValue
fn from_float(value: f64, original_type: GdlType) -> XdlValue {
    match original_type {
        GdlType::Float => XdlValue::Float(value as f32),
        GdlType::Double => XdlValue::Double(value),
        GdlType::Byte => XdlValue::Byte(value as u8),
        GdlType::Int => XdlValue::Int(value as i16),
        GdlType::Long => XdlValue::Long(value as i32),
        GdlType::UInt => XdlValue::UInt(value as u16),
        GdlType::ULong => XdlValue::ULong(value as u32),
        GdlType::Long64 => XdlValue::Long64(value as i64),
        GdlType::ULong64 => XdlValue::ULong64(value as u64),
        _ => XdlValue::Double(value), // Default to double
    }
}

pub fn sin(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "SIN: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let input = &_args[0];

    // Handle MultiDimArray - preserve shape
    if let XdlValue::MultiDimArray { data, shape } = input {
        let result: Vec<f64> = data.iter().map(|&x| x.sin()).collect();
        return Ok(XdlValue::MultiDimArray {
            data: result,
            shape: shape.clone(),
        });
    }

    // Handle arrays
    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr.iter().map(|&x| x.sin()).collect();
        return Ok(XdlValue::Array(result));
    }

    // Handle scalar values
    let float_val = to_float(input)?;
    let result = float_val.sin();

    Ok(from_float(result, input.gdl_type()))
}

pub fn cos(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "COS: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let input = &_args[0];

    // Handle MultiDimArray - preserve shape
    if let XdlValue::MultiDimArray { data, shape } = input {
        let result: Vec<f64> = data.iter().map(|&x| x.cos()).collect();
        return Ok(XdlValue::MultiDimArray {
            data: result,
            shape: shape.clone(),
        });
    }

    // Handle arrays
    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr.iter().map(|&x| x.cos()).collect();
        return Ok(XdlValue::Array(result));
    }

    // Handle scalar values
    let float_val = to_float(input)?;
    let result = float_val.cos();

    Ok(from_float(result, input.gdl_type()))
}

pub fn exp(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "EXP: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let input = &_args[0];

    // Handle MultiDimArray - preserve shape
    if let XdlValue::MultiDimArray { data, shape } = input {
        let result: Vec<f64> = data.iter().map(|&x| x.exp()).collect();
        return Ok(XdlValue::MultiDimArray {
            data: result,
            shape: shape.clone(),
        });
    }

    // Handle arrays
    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr.iter().map(|&x| x.exp()).collect();
        return Ok(XdlValue::Array(result));
    }

    // Handle scalar values
    let float_val = to_float(input)?;
    let result = float_val.exp();

    Ok(from_float(result, input.gdl_type()))
}

pub fn sqrt(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "SQRT: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let input = &_args[0];

    // Handle MultiDimArray - preserve shape
    if let XdlValue::MultiDimArray { data, shape } = input {
        let result: Vec<f64> = data
            .iter()
            .map(|&x| {
                if x < 0.0 {
                    f64::NAN // Return NaN for negative values in arrays
                } else {
                    x.sqrt()
                }
            })
            .collect();
        return Ok(XdlValue::MultiDimArray {
            data: result,
            shape: shape.clone(),
        });
    }

    // Handle arrays
    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr
            .iter()
            .map(|&x| {
                if x < 0.0 {
                    f64::NAN // Return NaN for negative values in arrays
                } else {
                    x.sqrt()
                }
            })
            .collect();
        return Ok(XdlValue::Array(result));
    }

    // Handle scalar values
    let float_val = to_float(input)?;

    if float_val < 0.0 {
        return Err(XdlError::MathError(
            "SQRT: Argument must be non-negative".to_string(),
        ));
    }

    let result = float_val.sqrt();
    Ok(from_float(result, input.gdl_type()))
}

pub fn log(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "ALOG: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let input = &_args[0];
    let float_val = to_float(input)?;

    if float_val <= 0.0 {
        return Err(XdlError::MathError(
            "ALOG: Argument must be positive".to_string(),
        ));
    }

    let result = float_val.ln();
    Ok(from_float(result, input.gdl_type()))
}

pub fn log10(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "ALOG10: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let input = &_args[0];
    let float_val = to_float(input)?;

    if float_val <= 0.0 {
        return Err(XdlError::MathError(
            "ALOG10: Argument must be positive".to_string(),
        ));
    }

    let result = float_val.log10();
    Ok(from_float(result, input.gdl_type()))
}

pub fn abs(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "ABS: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let input = &_args[0];
    let float_val = to_float(input)?;
    let result = float_val.abs();

    Ok(from_float(result, input.gdl_type()))
}

pub fn tan(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "TAN: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let input = &_args[0];
    let float_val = to_float(input)?;
    let result = float_val.tan();

    Ok(from_float(result, input.gdl_type()))
}

pub fn asin(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "ASIN: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let input = &_args[0];
    let float_val = to_float(input)?;

    if !(-1.0..=1.0).contains(&float_val) {
        return Err(XdlError::MathError(
            "ASIN: Argument must be in range [-1, 1]".to_string(),
        ));
    }

    let result = float_val.asin();
    Ok(from_float(result, input.gdl_type()))
}

pub fn acos(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "ACOS: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let input = &_args[0];
    let float_val = to_float(input)?;

    if !(-1.0..=1.0).contains(&float_val) {
        return Err(XdlError::MathError(
            "ACOS: Argument must be in range [-1, 1]".to_string(),
        ));
    }

    let result = float_val.acos();
    Ok(from_float(result, input.gdl_type()))
}

pub fn atan(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "ATAN: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let input = &_args[0];
    let float_val = to_float(input)?;
    let result = float_val.atan();

    Ok(from_float(result, input.gdl_type()))
}

/// ATAN2 - Two-argument arctangent (angle from x-axis to point (x, y))
/// Result = ATAN(y, x) or ATAN(y/x) with correct quadrant
pub fn atan2(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 2 {
        return Err(XdlError::InvalidArgument(format!(
            "ATAN: Expected 2 arguments for ATAN(y, x), got {}",
            _args.len()
        )));
    }

    let y = to_float(&_args[0])?;
    let x = to_float(&_args[1])?;
    let result = y.atan2(x);

    Ok(from_float(result, _args[0].gdl_type()))
}

/// SINH - Hyperbolic sine
pub fn sinh(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "SINH: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let input = &_args[0];

    // Handle MultiDimArray
    if let XdlValue::MultiDimArray { data, shape } = input {
        let result: Vec<f64> = data.iter().map(|&x| x.sinh()).collect();
        return Ok(XdlValue::MultiDimArray {
            data: result,
            shape: shape.clone(),
        });
    }

    // Handle 1D Array
    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr.iter().map(|&x| x.sinh()).collect();
        return Ok(XdlValue::Array(result));
    }

    // Handle scalar
    let float_val = to_float(input)?;
    Ok(from_float(float_val.sinh(), input.gdl_type()))
}

/// COSH - Hyperbolic cosine
pub fn cosh(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "COSH: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let input = &_args[0];

    // Handle MultiDimArray
    if let XdlValue::MultiDimArray { data, shape } = input {
        let result: Vec<f64> = data.iter().map(|&x| x.cosh()).collect();
        return Ok(XdlValue::MultiDimArray {
            data: result,
            shape: shape.clone(),
        });
    }

    // Handle 1D Array
    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr.iter().map(|&x| x.cosh()).collect();
        return Ok(XdlValue::Array(result));
    }

    // Handle scalar
    let float_val = to_float(input)?;
    Ok(from_float(float_val.cosh(), input.gdl_type()))
}

/// TANH - Hyperbolic tangent
pub fn tanh(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "TANH: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let input = &_args[0];

    // Handle MultiDimArray
    if let XdlValue::MultiDimArray { data, shape } = input {
        let result: Vec<f64> = data.iter().map(|&x| x.tanh()).collect();
        return Ok(XdlValue::MultiDimArray {
            data: result,
            shape: shape.clone(),
        });
    }

    // Handle 1D Array
    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr.iter().map(|&x| x.tanh()).collect();
        return Ok(XdlValue::Array(result));
    }

    // Handle scalar
    let float_val = to_float(input)?;
    Ok(from_float(float_val.tanh(), input.gdl_type()))
}

/// ASINH - Inverse hyperbolic sine
pub fn asinh(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "ASINH: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let input = &_args[0];

    // Handle MultiDimArray
    if let XdlValue::MultiDimArray { data, shape } = input {
        let result: Vec<f64> = data.iter().map(|&x| x.asinh()).collect();
        return Ok(XdlValue::MultiDimArray {
            data: result,
            shape: shape.clone(),
        });
    }

    // Handle 1D Array
    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr.iter().map(|&x| x.asinh()).collect();
        return Ok(XdlValue::Array(result));
    }

    // Handle scalar
    let float_val = to_float(input)?;
    Ok(from_float(float_val.asinh(), input.gdl_type()))
}

/// ACOSH - Inverse hyperbolic cosine
/// Note: Argument must be >= 1
pub fn acosh(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "ACOSH: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let input = &_args[0];

    // Handle MultiDimArray
    if let XdlValue::MultiDimArray { data, shape } = input {
        let result: Vec<f64> = data
            .iter()
            .map(|&x| if x >= 1.0 { x.acosh() } else { f64::NAN })
            .collect();
        return Ok(XdlValue::MultiDimArray {
            data: result,
            shape: shape.clone(),
        });
    }

    // Handle 1D Array
    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr
            .iter()
            .map(|&x| if x >= 1.0 { x.acosh() } else { f64::NAN })
            .collect();
        return Ok(XdlValue::Array(result));
    }

    // Handle scalar
    let float_val = to_float(input)?;
    if float_val < 1.0 {
        return Err(XdlError::MathError(
            "ACOSH: Argument must be >= 1".to_string(),
        ));
    }
    Ok(from_float(float_val.acosh(), input.gdl_type()))
}

/// ATANH - Inverse hyperbolic tangent
/// Note: Argument must be in range (-1, 1)
pub fn atanh(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "ATANH: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let input = &_args[0];

    // Handle MultiDimArray
    if let XdlValue::MultiDimArray { data, shape } = input {
        let result: Vec<f64> = data
            .iter()
            .map(|&x| {
                if x > -1.0 && x < 1.0 {
                    x.atanh()
                } else {
                    f64::NAN
                }
            })
            .collect();
        return Ok(XdlValue::MultiDimArray {
            data: result,
            shape: shape.clone(),
        });
    }

    // Handle 1D Array
    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr
            .iter()
            .map(|&x| {
                if x > -1.0 && x < 1.0 {
                    x.atanh()
                } else {
                    f64::NAN
                }
            })
            .collect();
        return Ok(XdlValue::Array(result));
    }

    // Handle scalar
    let float_val = to_float(input)?;
    if float_val <= -1.0 || float_val >= 1.0 {
        return Err(XdlError::MathError(
            "ATANH: Argument must be in range (-1, 1)".to_string(),
        ));
    }
    Ok(from_float(float_val.atanh(), input.gdl_type()))
}

pub fn floor(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "FLOOR: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let input = &_args[0];
    let float_val = to_float(input)?;
    let result = float_val.floor();

    Ok(from_float(result, input.gdl_type()))
}

pub fn ceil(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "CEIL: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let input = &_args[0];
    let float_val = to_float(input)?;
    let result = float_val.ceil();

    Ok(from_float(result, input.gdl_type()))
}

pub fn round(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "ROUND: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let input = &_args[0];
    let float_val = to_float(input)?;
    let result = float_val.round();

    Ok(from_float(result, input.gdl_type()))
}

/// Helper function to extract dimensions from arguments
/// Used by all *INDGEN and *GEN array generation functions
fn extract_dimensions(args: &[XdlValue]) -> XdlResult<Vec<usize>> {
    if args.is_empty() || args.len() > 8 {
        return Err(XdlError::InvalidArgument(format!(
            "Array generation: Expected 1-8 dimension arguments, got {}",
            args.len()
        )));
    }

    let mut dimensions = Vec::new();
    for arg in args {
        let dim = match arg {
            XdlValue::Long(v) => {
                if *v < 0 {
                    return Err(XdlError::InvalidArgument(
                        "Array dimensions must be non-negative".to_string(),
                    ));
                }
                *v as usize
            }
            XdlValue::Int(v) => {
                if *v < 0 {
                    return Err(XdlError::InvalidArgument(
                        "Array dimensions must be non-negative".to_string(),
                    ));
                }
                *v as usize
            }
            XdlValue::Byte(v) => *v as usize,
            XdlValue::Float(v) => {
                if *v < 0.0 {
                    return Err(XdlError::InvalidArgument(
                        "Array dimensions must be non-negative".to_string(),
                    ));
                }
                *v as usize
            }
            XdlValue::Double(v) => {
                if *v < 0.0 {
                    return Err(XdlError::InvalidArgument(
                        "Array dimensions must be non-negative".to_string(),
                    ));
                }
                *v as usize
            }
            _ => {
                return Err(XdlError::TypeMismatch {
                    expected: "numeric".to_string(),
                    actual: format!("{:?}", arg.gdl_type()),
                })
            }
        };
        dimensions.push(dim);
    }
    Ok(dimensions)
}

/// Generate floating point array: FINDGEN(D1 [, D2, ..., D8])
/// Result = FINDGEN(D1 [, ..., D8] [, INCREMENT=value] [, START=value])
///
/// Creates an array with dimensions D1 through D8, filled with floating-point
/// values starting from START (default 0.0) and incrementing by INCREMENT (default 1.0).
///
/// Note: Currently INCREMENT and START keywords are not yet supported.
/// TODO: Add keyword argument support when evaluator supports them.
///
/// Examples:
///   FINDGEN(5)       ; Returns [0.0, 1.0, 2.0, 3.0, 4.0]
///   FINDGEN(3, 4)    ; Returns 3x4 array with values 0-11
///   FINDGEN(2, 3, 2) ; Returns 2x3x2 array with values 0-11
pub fn findgen(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    // Extract dimensions from arguments
    let dimensions = extract_dimensions(_args)?;

    // Calculate total size
    let total_size: usize = dimensions.iter().product();

    // Default values (TODO: get from keywords when supported)
    let start = 0.0_f64;
    let increment = 1.0_f64;

    // Generate data: start + (index * increment)
    let data: Vec<f64> = (0..total_size)
        .map(|i| start + (i as f64 * increment))
        .collect();

    // Return appropriate type based on number of dimensions
    if dimensions.len() == 1 {
        // 1D array - return simple Array
        Ok(XdlValue::Array(data))
    } else {
        // Multi-dimensional array - return MultiDimArray with shape
        Ok(XdlValue::MultiDimArray {
            data,
            shape: dimensions,
        })
    }
}

/// Generate double precision array: DINDGEN(D1 [, D2, ..., D8])
/// Result = DINDGEN(D1 [, ..., D8] [, INCREMENT=value] [, START=value])
///
/// Same as FINDGEN but explicitly for double precision.
/// Since our implementation uses f64 for numeric arrays by default,
/// DINDGEN is identical to FINDGEN.
///
/// See FINDGEN documentation for details.
pub fn dindgen(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    // DINDGEN is identical to FINDGEN in our implementation
    // since we use f64 for numeric arrays by default
    findgen(_args)
}

/// Generate byte integer array: BINDGEN(D1 [, D2, ..., D8])
/// Result = BINDGEN(D1 [, ..., D8] [, INCREMENT=value] [, START=value])
///
/// Creates an array with dimensions D1 through D8, filled with byte integer
/// values (0-255) starting from START (default 0) and incrementing by INCREMENT (default 1).
///
/// Examples:
///   BINDGEN(5)       ; Returns byte array [0, 1, 2, 3, 4]
///   BINDGEN(3, 4)    ; Returns 3x4 byte array with values 0-11
pub fn bindgen(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    let dimensions = extract_dimensions(_args)?;
    let total_size: usize = dimensions.iter().product();

    let start = 0_u8;
    let increment = 1_u8;

    // Generate data as f64 for compatibility with XdlValue::Array
    let data: Vec<f64> = (0..total_size)
        .map(|i| ((start as usize + i * increment as usize) % 256) as f64)
        .collect();

    if dimensions.len() == 1 {
        Ok(XdlValue::Array(data))
    } else {
        Ok(XdlValue::MultiDimArray {
            data,
            shape: dimensions,
        })
    }
}

/// Generate complex integer array: CINDGEN(D1 [, D2, ..., D8])
/// Result = CINDGEN(D1 [, ..., D8] [, INCREMENT=value] [, START=value])
///
/// Creates an array with dimensions D1 through D8, filled with complex values.
/// Real part increments from 0, imaginary part is 0.
///
/// Note: XDL currently represents complex as interleaved [real, imag, real, imag, ...]
pub fn cindgen(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    let dimensions = extract_dimensions(_args)?;
    let total_size: usize = dimensions.iter().product();

    // Generate complex data: [real0, imag0, real1, imag1, ...]
    let mut data = Vec::with_capacity(total_size * 2);
    for i in 0..total_size {
        data.push(i as f64); // Real part
        data.push(0.0); // Imaginary part
    }

    if dimensions.len() == 1 {
        Ok(XdlValue::Array(data))
    } else {
        Ok(XdlValue::MultiDimArray {
            data,
            shape: dimensions,
        })
    }
}

/// Generate double complex integer array: DCINDGEN(D1 [, D2, ..., D8])
/// Result = DCINDGEN(D1 [, ..., D8] [, INCREMENT=value] [, START=value])
///
/// Same as CINDGEN but for double precision complex.
/// Since we use f64 by default, this is identical to CINDGEN.
pub fn dcindgen(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    cindgen(_args)
}

/// Generate integer array: INDGEN(D1 [, D2, ..., D8])
/// Result = INDGEN(D1[, ..., D8] [, /BYTE | , /COMPLEX | , /DCOMPLEX | , /DOUBLE | , /FLOAT |
///                 INCREMENT=value | , /L64 | , /LONG | , /STRING | , /UINT | , /UL64 | , /ULONG]
///                [, START=value] [, TYPE=value])
///
/// Creates an array with dimensions D1 through D8, filled with integer values.
/// Type flags are not yet supported (TODO: add when evaluator supports keyword flags).
///
/// Examples:
///   INDGEN(5)       ; Returns [0, 1, 2, 3, 4]
///   INDGEN(3, 4)    ; Returns 3x4 array with values 0-11
pub fn indgen(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    let dimensions = extract_dimensions(_args)?;
    let total_size: usize = dimensions.iter().product();

    // Generate integer data as f64 for compatibility
    let data: Vec<f64> = (0..total_size).map(|i| i as f64).collect();

    if dimensions.len() == 1 {
        Ok(XdlValue::Array(data))
    } else {
        Ok(XdlValue::MultiDimArray {
            data,
            shape: dimensions,
        })
    }
}

/// Generate long integer array: LINDGEN(D1 [, D2, ..., D8])
/// Result = LINDGEN(D1 [, ..., D8] [, INCREMENT=value] [, START=value])
///
/// Creates an array with dimensions D1 through D8, filled with long integer values.
///
/// Examples:
///   LINDGEN(5)       ; Returns [0, 1, 2, 3, 4]
///   LINDGEN(3, 4)    ; Returns 3x4 array with values 0-11
pub fn lindgen(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    // Same as INDGEN for our implementation
    indgen(_args)
}

/// Generate 64-bit long integer array: L64INDGEN(D1 [, D2, ..., D8])
/// Result = L64INDGEN(D1 [, ..., D8] [, INCREMENT=value] [, START=value])
///
/// Creates an array with dimensions D1 through D8, filled with 64-bit long integer values.
pub fn l64indgen(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    indgen(_args)
}

/// Generate string array: SINDGEN(D1 [, D2, ..., D8])
/// Result = SINDGEN(D1 [, ..., D8] [, INCREMENT=value] [, START=value])
///
/// Creates an array with dimensions D1 through D8, filled with string representations
/// of integer values.
///
/// Note: Currently returns numeric array. String arrays need full XdlValue support.
pub fn sindgen(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    // For now, return numeric array (TODO: implement proper string arrays)
    indgen(_args)
}

/// Generate unsigned integer array: UINDGEN(D1 [, D2, ..., D8])
/// Result = UINDGEN(D1 [, ..., D8] [, INCREMENT=value] [, START=value])
///
/// Creates an array with dimensions D1 through D8, filled with unsigned integer values.
pub fn uindgen(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    indgen(_args)
}

/// Generate unsigned 64-bit long array: UL64INDGEN(D1 [, D2, ..., D8])
/// Result = UL64INDGEN(D1 [, ..., D8] [, INCREMENT=value] [, START=value])
///
/// Creates an array with dimensions D1 through D8, filled with unsigned 64-bit long values.
pub fn ul64indgen(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    indgen(_args)
}

/// Generate unsigned long integer array: ULINDGEN(D1 [, D2, ..., D8])
/// Result = ULINDGEN(D1 [, ..., D8] [, INCREMENT=value] [, START=value])
///
/// Creates an array with dimensions D1 through D8, filled with unsigned long integer values.
pub fn ulindgen(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    indgen(_args)
}

// ============================================================================
// KEYWORD-AWARE ARRAY GENERATION FUNCTIONS
// ============================================================================

/// Helper function to extract START and INCREMENT keyword values
fn extract_start_increment(keywords: &HashMap<String, XdlValue>) -> XdlResult<(f64, f64)> {
    let start = if let Some(val) = keywords.get("START") {
        val.to_double()?
    } else {
        0.0
    };

    let increment = if let Some(val) = keywords.get("INCREMENT") {
        val.to_double()?
    } else {
        1.0
    };

    Ok((start, increment))
}

/// FINDGEN with keyword support for START and INCREMENT
pub fn findgen_with_keywords(
    args: &[XdlValue],
    keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    let dimensions = extract_dimensions(args)?;
    let total_size: usize = dimensions.iter().product();
    let (start, increment) = extract_start_increment(keywords)?;

    let data: Vec<f64> = (0..total_size)
        .map(|i| start + (i as f64 * increment))
        .collect();

    if dimensions.len() == 1 {
        Ok(XdlValue::Array(data))
    } else {
        Ok(XdlValue::MultiDimArray {
            data,
            shape: dimensions,
        })
    }
}

/// DINDGEN with keyword support
pub fn dindgen_with_keywords(
    args: &[XdlValue],
    keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    findgen_with_keywords(args, keywords)
}

/// BINDGEN with keyword support
pub fn bindgen_with_keywords(
    args: &[XdlValue],
    keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    let dimensions = extract_dimensions(args)?;
    let total_size: usize = dimensions.iter().product();
    let (start, increment) = extract_start_increment(keywords)?;

    let data: Vec<f64> = (0..total_size)
        .map(|i| ((start as usize + i * increment as usize) % 256) as f64)
        .collect();

    if dimensions.len() == 1 {
        Ok(XdlValue::Array(data))
    } else {
        Ok(XdlValue::MultiDimArray {
            data,
            shape: dimensions,
        })
    }
}

/// CINDGEN with keyword support
pub fn cindgen_with_keywords(
    args: &[XdlValue],
    keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    let dimensions = extract_dimensions(args)?;
    let total_size: usize = dimensions.iter().product();
    let (start, increment) = extract_start_increment(keywords)?;

    let mut data = Vec::with_capacity(total_size * 2);
    for i in 0..total_size {
        data.push(start + (i as f64 * increment)); // Real part
        data.push(0.0); // Imaginary part
    }

    if dimensions.len() == 1 {
        Ok(XdlValue::Array(data))
    } else {
        Ok(XdlValue::MultiDimArray {
            data,
            shape: dimensions,
        })
    }
}

/// DCINDGEN with keyword support
pub fn dcindgen_with_keywords(
    args: &[XdlValue],
    keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    cindgen_with_keywords(args, keywords)
}

/// INDGEN with keyword support
pub fn indgen_with_keywords(
    args: &[XdlValue],
    keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    let dimensions = extract_dimensions(args)?;
    let total_size: usize = dimensions.iter().product();
    let (start, increment) = extract_start_increment(keywords)?;

    let data: Vec<f64> = (0..total_size)
        .map(|i| (start + (i as f64 * increment)).floor())
        .collect();

    if dimensions.len() == 1 {
        Ok(XdlValue::Array(data))
    } else {
        Ok(XdlValue::MultiDimArray {
            data,
            shape: dimensions,
        })
    }
}

/// LINDGEN with keyword support
pub fn lindgen_with_keywords(
    args: &[XdlValue],
    keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    indgen_with_keywords(args, keywords)
}

/// L64INDGEN with keyword support
pub fn l64indgen_with_keywords(
    args: &[XdlValue],
    keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    indgen_with_keywords(args, keywords)
}

/// SINDGEN with keyword support - returns string array
pub fn sindgen_with_keywords(
    args: &[XdlValue],
    keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    let dimensions = extract_dimensions(args)?;
    let total_size: usize = dimensions.iter().product();
    let (start, increment) = extract_start_increment(keywords)?;

    // Generate string representations of the values
    let strings: Vec<XdlValue> = (0..total_size)
        .map(|i| {
            let val = (start + (i as f64 * increment)).floor() as i64;
            XdlValue::String(val.to_string())
        })
        .collect();

    // Return as NestedArray of strings for now
    Ok(XdlValue::NestedArray(strings))
}

/// UINDGEN with keyword support
pub fn uindgen_with_keywords(
    args: &[XdlValue],
    keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    indgen_with_keywords(args, keywords)
}

/// UL64INDGEN with keyword support
pub fn ul64indgen_with_keywords(
    args: &[XdlValue],
    keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    indgen_with_keywords(args, keywords)
}

/// ULINDGEN with keyword support
pub fn ulindgen_with_keywords(
    args: &[XdlValue],
    keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    indgen_with_keywords(args, keywords)
}

/// MAKE_ARRAY - Flexible array creation function
/// Result = MAKE_ARRAY([D1, ..., D8] [, DIMENSION=vector] [, /INDEX] [, /NOZERO]
///                     [, SIZE=vector] [, TYPE=type_code] [, VALUE=value])
///
/// Creates an array using flexible specification methods:
/// - Positional dimensions (D1-D8)
/// - DIMENSION keyword for dimension vector
/// - SIZE keyword for IDL-style size vector
/// - VALUE keyword to fill with specific value
/// - /INDEX flag to fill with index values (like INDGEN)
/// - /NOZERO flag to skip zero-initialization (no-op in our impl)
/// - TYPE keyword to specify data type
pub fn make_array(
    args: &[XdlValue],
    keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    // Determine dimensions from various sources
    let dimensions: Vec<usize> = if let Some(dim_val) = keywords.get("DIMENSION") {
        // DIMENSION keyword takes precedence
        match dim_val {
            XdlValue::Array(arr) => arr.iter().map(|v| *v as usize).collect(),
            XdlValue::MultiDimArray { data, .. } => data.iter().map(|v| *v as usize).collect(),
            _ => vec![dim_val.to_double()? as usize],
        }
    } else if let Some(size_val) = keywords.get("SIZE") {
        // SIZE keyword: IDL-style [ndims, d1, d2, ..., type, nelements]
        match size_val {
            XdlValue::Array(arr) if arr.len() >= 2 => {
                let ndims = arr[0] as usize;
                arr[1..=ndims].iter().map(|v| *v as usize).collect()
            }
            _ => {
                return Err(XdlError::InvalidArgument(
                    "SIZE keyword must be an array with [ndims, d1, d2, ...]".to_string(),
                ))
            }
        }
    } else if !args.is_empty() {
        // Use positional arguments
        extract_dimensions(args)?
    } else {
        return Err(XdlError::InvalidArgument(
            "MAKE_ARRAY: Must specify dimensions via arguments, DIMENSION, or SIZE keyword"
                .to_string(),
        ));
    };

    let total_size: usize = dimensions.iter().product();

    // Check for /INDEX flag
    let use_index = keywords.get("INDEX").is_some();

    // Check for VALUE keyword
    let fill_value = keywords.get("VALUE");

    // Generate data based on options
    let data: Vec<f64> = if use_index {
        // Fill with index values (like INDGEN)
        let (start, increment) = extract_start_increment(keywords)?;
        (0..total_size)
            .map(|i| start + (i as f64 * increment))
            .collect()
    } else if let Some(val) = fill_value {
        // Fill with specified value
        let fill = val.to_double()?;
        vec![fill; total_size]
    } else {
        // Default: fill with zeros
        vec![0.0; total_size]
    };

    if dimensions.len() == 1 {
        Ok(XdlValue::Array(data))
    } else {
        Ok(XdlValue::MultiDimArray {
            data,
            shape: dimensions,
        })
    }
}

/// FIX function - truncates to integer (floor for positive, ceil for negative)
/// IDL/GDL FIX converts to integer by truncation towards zero
pub fn fix(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "FIX: Expected at least 1 argument".to_string(),
        ));
    }

    match &_args[0] {
        XdlValue::Array(arr) => {
            let result: Vec<f64> = arr.iter().map(|v| v.trunc()).collect();
            Ok(XdlValue::Array(result))
        }
        XdlValue::MultiDimArray { data, shape } => {
            let result: Vec<f64> = data.iter().map(|v| v.trunc()).collect();
            Ok(XdlValue::MultiDimArray {
                data: result,
                shape: shape.clone(),
            })
        }
        _ => {
            let val = _args[0].to_double()?;
            Ok(XdlValue::Long(val.trunc() as i32))
        }
    }
}

/// MESHGRID function - creates coordinate arrays from coordinate vectors
/// Usage: MESHGRID, x, y, xx, yy
/// Creates 2D coordinate matrices from 1D coordinate vectors
pub fn meshgrid(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 2 {
        return Err(XdlError::InvalidArgument(format!(
            "MESHGRID: Expected 2 arguments (x, y), got {}",
            _args.len()
        )));
    }

    let x_vec = match &_args[0] {
        XdlValue::Array(arr) => arr.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", _args[0].gdl_type()),
            })
        }
    };

    let y_vec = match &_args[1] {
        XdlValue::Array(arr) => arr.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", _args[1].gdl_type()),
            })
        }
    };

    let nx = x_vec.len();
    let ny = y_vec.len();

    // Create XX: each row is a copy of x_vec
    let mut xx_data = Vec::with_capacity(nx * ny);
    for _ in 0..ny {
        xx_data.extend_from_slice(&x_vec);
    }

    // Create YY: each column is a copy of y_vec
    let mut yy_data = Vec::with_capacity(nx * ny);
    for &y_val in &y_vec {
        for _ in 0..nx {
            yy_data.push(y_val);
        }
    }

    // Return as a 2-element nested array [XX, YY]
    Ok(XdlValue::NestedArray(vec![
        XdlValue::MultiDimArray {
            data: xx_data,
            shape: vec![nx, ny],
        },
        XdlValue::MultiDimArray {
            data: yy_data,
            shape: vec![nx, ny],
        },
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sin() {
        let args = vec![XdlValue::Double(0.0)];
        let result = sin(&args).unwrap();
        assert!(matches!(result, XdlValue::Double(v) if v.abs() < 1e-10));

        let args = vec![XdlValue::Double(std::f64::consts::PI / 2.0)];
        let result = sin(&args).unwrap();
        assert!(matches!(result, XdlValue::Double(v) if (v - 1.0).abs() < 1e-10));
    }

    #[test]
    fn test_cos() {
        let args = vec![XdlValue::Double(0.0)];
        let result = cos(&args).unwrap();
        assert!(matches!(result, XdlValue::Double(v) if (v - 1.0).abs() < 1e-10));

        let args = vec![XdlValue::Double(std::f64::consts::PI)];
        let result = cos(&args).unwrap();
        assert!(matches!(result, XdlValue::Double(v) if (v + 1.0).abs() < 1e-10));
    }

    #[test]
    fn test_exp() {
        let args = vec![XdlValue::Double(0.0)];
        let result = exp(&args).unwrap();
        assert!(matches!(result, XdlValue::Double(v) if (v - 1.0).abs() < 1e-10));

        let args = vec![XdlValue::Double(1.0)];
        let result = exp(&args).unwrap();
        assert!(matches!(result, XdlValue::Double(v) if (v - std::f64::consts::E).abs() < 1e-10));
    }

    #[test]
    fn test_log() {
        let args = vec![XdlValue::Double(std::f64::consts::E)];
        let result = log(&args).unwrap();
        assert!(matches!(result, XdlValue::Double(v) if (v - 1.0).abs() < 1e-10));

        // Test error for non-positive input
        let args = vec![XdlValue::Double(-1.0)];
        assert!(log(&args).is_err());
    }

    #[test]
    fn test_sqrt() {
        let args = vec![XdlValue::Double(4.0)];
        let result = sqrt(&args).unwrap();
        assert!(matches!(result, XdlValue::Double(v) if (v - 2.0).abs() < 1e-10));

        let args = vec![XdlValue::Double(0.0)];
        let result = sqrt(&args).unwrap();
        assert!(matches!(result, XdlValue::Double(v) if v.abs() < 1e-10));

        // Test error for negative input
        let args = vec![XdlValue::Double(-1.0)];
        assert!(sqrt(&args).is_err());
    }
}

/// FIX - Convert to integer type
pub fn fix_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "FIX: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let input = &args[0];
    match input {
        XdlValue::Array(arr) => {
            let result: Vec<f64> = arr.iter().map(|&x| (x as i32) as f64).collect();
            Ok(XdlValue::Array(result))
        }
        _ => {
            let val = input.to_double()?;
            Ok(XdlValue::Long(val as i32))
        }
    }
}

/// LONG - Convert to long integer type
pub fn long_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "LONG: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let input = &args[0];
    match input {
        XdlValue::Array(arr) => {
            let result: Vec<f64> = arr.iter().map(|&x| (x as i32) as f64).collect();
            Ok(XdlValue::Array(result))
        }
        _ => {
            let val = input.to_double()?;
            Ok(XdlValue::Long(val as i32))
        }
    }
}

/// FLOAT - Convert to float type
pub fn float_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "FLOAT: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let input = &args[0];
    match input {
        XdlValue::Array(arr) => {
            Ok(XdlValue::Array(arr.clone())) // Already f64, compatible with float
        }
        _ => {
            let val = input.to_double()?;
            Ok(XdlValue::Float(val as f32))
        }
    }
}

/// DOUBLE - Convert to double type
pub fn double_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "DOUBLE: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let input = &args[0];
    match input {
        XdlValue::Array(arr) => Ok(XdlValue::Array(arr.clone())),
        _ => {
            let val = input.to_double()?;
            Ok(XdlValue::Double(val))
        }
    }
}

/// BYTE - Convert to byte (u8) type
pub fn byte_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "BYTE: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let input = &args[0];
    match input {
        XdlValue::Array(arr) => {
            let result: Vec<f64> = arr.iter().map(|&x| (x as u8) as f64).collect();
            Ok(XdlValue::Array(result))
        }
        XdlValue::MultiDimArray { data, shape } => {
            let result: Vec<f64> = data.iter().map(|&x| (x as u8) as f64).collect();
            Ok(XdlValue::MultiDimArray {
                data: result,
                shape: shape.clone(),
            })
        }
        XdlValue::String(s) => {
            // Convert string to byte array
            let bytes: Vec<f64> = s.bytes().map(|b| b as f64).collect();
            Ok(XdlValue::Array(bytes))
        }
        _ => {
            let val = input.to_double()?;
            Ok(XdlValue::Byte(val as u8))
        }
    }
}

/// UINT - Convert to unsigned integer (u16) type
pub fn uint_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "UINT: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let input = &args[0];
    match input {
        XdlValue::Array(arr) => {
            let result: Vec<f64> = arr.iter().map(|&x| (x as u16) as f64).collect();
            Ok(XdlValue::Array(result))
        }
        XdlValue::MultiDimArray { data, shape } => {
            let result: Vec<f64> = data.iter().map(|&x| (x as u16) as f64).collect();
            Ok(XdlValue::MultiDimArray {
                data: result,
                shape: shape.clone(),
            })
        }
        _ => {
            let val = input.to_double()?;
            Ok(XdlValue::UInt(val as u16))
        }
    }
}

/// ULONG - Convert to unsigned long (u32) type
pub fn ulong_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "ULONG: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let input = &args[0];
    match input {
        XdlValue::Array(arr) => {
            let result: Vec<f64> = arr.iter().map(|&x| (x as u32) as f64).collect();
            Ok(XdlValue::Array(result))
        }
        XdlValue::MultiDimArray { data, shape } => {
            let result: Vec<f64> = data.iter().map(|&x| (x as u32) as f64).collect();
            Ok(XdlValue::MultiDimArray {
                data: result,
                shape: shape.clone(),
            })
        }
        _ => {
            let val = input.to_double()?;
            Ok(XdlValue::ULong(val as u32))
        }
    }
}

/// LONG64 - Convert to 64-bit signed integer type
pub fn long64_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "LONG64: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let input = &args[0];
    match input {
        XdlValue::Array(arr) => {
            let result: Vec<f64> = arr.iter().map(|&x| (x as i64) as f64).collect();
            Ok(XdlValue::Array(result))
        }
        XdlValue::MultiDimArray { data, shape } => {
            let result: Vec<f64> = data.iter().map(|&x| (x as i64) as f64).collect();
            Ok(XdlValue::MultiDimArray {
                data: result,
                shape: shape.clone(),
            })
        }
        _ => {
            let val = input.to_double()?;
            Ok(XdlValue::Long64(val as i64))
        }
    }
}

/// ULONG64 - Convert to 64-bit unsigned integer type
pub fn ulong64_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "ULONG64: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let input = &args[0];
    match input {
        XdlValue::Array(arr) => {
            let result: Vec<f64> = arr.iter().map(|&x| (x as u64) as f64).collect();
            Ok(XdlValue::Array(result))
        }
        XdlValue::MultiDimArray { data, shape } => {
            let result: Vec<f64> = data.iter().map(|&x| (x as u64) as f64).collect();
            Ok(XdlValue::MultiDimArray {
                data: result,
                shape: shape.clone(),
            })
        }
        _ => {
            let val = input.to_double()?;
            Ok(XdlValue::ULong64(val as u64))
        }
    }
}

/// FFT - Fast Fourier Transform
/// FFT(array [, direction] [, /INVERSE])
/// Returns complex FFT of input array
/// direction: 1 (forward, default) or -1 (inverse)
pub fn fft(args: &[XdlValue]) -> XdlResult<XdlValue> {
    use rustfft::{num_complex::Complex64, FftPlanner};

    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "FFT: Expected at least 1 argument (array)".to_string(),
        ));
    }

    // Get input array
    let input_arr = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if input_arr.is_empty() {
        return Err(XdlError::InvalidArgument(
            "FFT: Input array cannot be empty".to_string(),
        ));
    }

    // Check for inverse flag
    let inverse = if args.len() > 1 {
        match &args[1] {
            XdlValue::Long(n) => *n < 0,
            XdlValue::Int(n) => *n < 0,
            _ => false,
        }
    } else {
        false
    };

    // Convert input to complex numbers
    let mut buffer: Vec<Complex64> = input_arr.iter().map(|&x| Complex64::new(x, 0.0)).collect();

    // Create FFT planner and get the appropriate FFT
    let mut planner = FftPlanner::<f64>::new();
    let fft = if inverse {
        planner.plan_fft_inverse(buffer.len())
    } else {
        planner.plan_fft_forward(buffer.len())
    };

    // Perform FFT
    fft.process(&mut buffer);

    // For inverse FFT, normalize by 1/N (like IDL/GDL)
    if inverse {
        let n = buffer.len() as f64;
        for val in buffer.iter_mut() {
            *val /= n;
        }
    }

    // Convert result to interleaved real/imaginary array
    // Format: [real0, imag0, real1, imag1, ...]
    let result: Vec<f64> = buffer.iter().flat_map(|c| vec![c.re, c.im]).collect();

    Ok(XdlValue::Array(result))
}

/// RANDOMU - Generate uniform random numbers
pub fn randomu(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "RANDOMU: Expected at least 1 argument (seed)".to_string(),
        ));
    }

    // Get seed from first argument
    let seed = match &args[0] {
        XdlValue::Long(v) => *v as u64,
        XdlValue::Int(v) => *v as u64,
        XdlValue::Byte(v) => *v as u64,
        XdlValue::Double(v) => *v as u64,
        XdlValue::Float(v) => *v as u64,
        _ => 12345u64, // Default seed
    };

    if args.len() == 1 {
        // Single random number - simple LCG
        let a = 1664525u64;
        let c = 1013904223u64;
        let rand_val = ((a.wrapping_mul(seed).wrapping_add(c) % 1000000) as f64) / 1000000.0;
        Ok(XdlValue::Double(rand_val))
    } else {
        // Array of random numbers
        let n = args[1].to_long()? as usize;
        let mut values = Vec::new();
        let mut current_seed = seed;

        for _i in 0..n {
            let a = 1664525u64;
            let c = 1013904223u64;
            current_seed = a.wrapping_mul(current_seed).wrapping_add(c);
            let rand_val = ((current_seed % 1000000) as f64) / 1000000.0;
            values.push(rand_val);
        }
        Ok(XdlValue::Array(values))
    }
}

/// RANDOMN - Generate normally distributed random numbers (Gaussian/normal distribution)
/// Usage: result = RANDOMN(seed [, d1, d2, ...])
pub fn randomn(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "RANDOMN: Expected at least 1 argument (seed)".to_string(),
        ));
    }

    // Get seed from first argument
    let seed = match &args[0] {
        XdlValue::Long(v) => *v as u64,
        XdlValue::Int(v) => *v as u64,
        XdlValue::Byte(v) => *v as u64,
        XdlValue::Double(v) => *v as u64,
        XdlValue::Float(v) => *v as u64,
        _ => 12345u64, // Default seed
    };

    // Box-Muller transform to convert uniform random to normal distribution
    fn box_muller(u1: f64, u2: f64) -> (f64, f64) {
        let r = (-2.0 * u1.ln()).sqrt();
        let theta = 2.0 * std::f64::consts::PI * u2;
        (r * theta.cos(), r * theta.sin())
    }

    // Generate uniform random numbers using LCG
    fn uniform_random(seed: &mut u64) -> f64 {
        let a = 1664525u64;
        let c = 1013904223u64;
        *seed = a.wrapping_mul(*seed).wrapping_add(c);
        ((*seed % 1000000) as f64) / 1000000.0
    }

    if args.len() == 1 {
        // Single random number
        let mut current_seed = seed;
        let u1 = uniform_random(&mut current_seed);
        let u2 = uniform_random(&mut current_seed);
        let (z0, _z1) = box_muller(u1, u2);
        Ok(XdlValue::Double(z0))
    } else {
        // Array of random numbers
        let n = args[1].to_long()? as usize;
        let mut values = Vec::new();
        let mut current_seed = seed;

        for _i in 0..n.div_ceil(2) {
            let u1 = uniform_random(&mut current_seed);
            let u2 = uniform_random(&mut current_seed);
            let (z0, z1) = box_muller(u1, u2);
            values.push(z0);
            if values.len() < n {
                values.push(z1);
            }
        }
        values.truncate(n);
        Ok(XdlValue::Array(values))
    }
}

// ============================================================
// Special Mathematical Functions (using libm)
// ============================================================

/// ERF - Error function
pub fn erf(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "ERF: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let input = &args[0];

    // Handle MultiDimArray
    if let XdlValue::MultiDimArray { data, shape } = input {
        let result: Vec<f64> = data.iter().map(|&x| libm::erf(x)).collect();
        return Ok(XdlValue::MultiDimArray {
            data: result,
            shape: shape.clone(),
        });
    }

    // Handle 1D Array
    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr.iter().map(|&x| libm::erf(x)).collect();
        return Ok(XdlValue::Array(result));
    }

    // Handle scalar
    let val = to_float(input)?;
    Ok(from_float(libm::erf(val), input.gdl_type()))
}

/// ERFC - Complementary error function (1 - erf(x))
pub fn erfc(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "ERFC: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let input = &args[0];

    // Handle MultiDimArray
    if let XdlValue::MultiDimArray { data, shape } = input {
        let result: Vec<f64> = data.iter().map(|&x| libm::erfc(x)).collect();
        return Ok(XdlValue::MultiDimArray {
            data: result,
            shape: shape.clone(),
        });
    }

    // Handle 1D Array
    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr.iter().map(|&x| libm::erfc(x)).collect();
        return Ok(XdlValue::Array(result));
    }

    // Handle scalar
    let val = to_float(input)?;
    Ok(from_float(libm::erfc(val), input.gdl_type()))
}

/// GAMMA - Gamma function
pub fn gamma_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "GAMMA: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let input = &args[0];

    // Handle MultiDimArray
    if let XdlValue::MultiDimArray { data, shape } = input {
        let result: Vec<f64> = data.iter().map(|&x| libm::tgamma(x)).collect();
        return Ok(XdlValue::MultiDimArray {
            data: result,
            shape: shape.clone(),
        });
    }

    // Handle 1D Array
    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr.iter().map(|&x| libm::tgamma(x)).collect();
        return Ok(XdlValue::Array(result));
    }

    // Handle scalar
    let val = to_float(input)?;
    Ok(from_float(libm::tgamma(val), input.gdl_type()))
}

/// LNGAMMA - Natural log of gamma function
pub fn lngamma(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "LNGAMMA: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let input = &args[0];

    // Handle MultiDimArray
    if let XdlValue::MultiDimArray { data, shape } = input {
        let result: Vec<f64> = data.iter().map(|&x| libm::lgamma(x)).collect();
        return Ok(XdlValue::MultiDimArray {
            data: result,
            shape: shape.clone(),
        });
    }

    // Handle 1D Array
    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr.iter().map(|&x| libm::lgamma(x)).collect();
        return Ok(XdlValue::Array(result));
    }

    // Handle scalar
    let val = to_float(input)?;
    Ok(from_float(libm::lgamma(val), input.gdl_type()))
}

/// FACTORIAL - Factorial function (n!)
pub fn factorial(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "FACTORIAL: Expected 1 argument, got {}",
            args.len()
        )));
    }

    fn factorial_val(n: i64) -> f64 {
        if n < 0 {
            f64::NAN
        } else if n <= 1 {
            1.0
        } else if n <= 20 {
            // Use iterative for small values
            let mut result = 1u64;
            for i in 2..=n as u64 {
                result = result.saturating_mul(i);
            }
            result as f64
        } else {
            // Use gamma function for large values: n! = gamma(n+1)
            libm::tgamma((n + 1) as f64)
        }
    }

    let input = &args[0];

    // Handle MultiDimArray
    if let XdlValue::MultiDimArray { data, shape } = input {
        let result: Vec<f64> = data.iter().map(|&x| factorial_val(x as i64)).collect();
        return Ok(XdlValue::MultiDimArray {
            data: result,
            shape: shape.clone(),
        });
    }

    // Handle 1D Array
    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr.iter().map(|&x| factorial_val(x as i64)).collect();
        return Ok(XdlValue::Array(result));
    }

    // Handle scalar
    let n = match input {
        XdlValue::Long(v) => *v as i64,
        XdlValue::Int(v) => *v as i64,
        XdlValue::Byte(v) => *v as i64,
        XdlValue::Float(v) => *v as i64,
        XdlValue::Double(v) => *v as i64,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: format!("{:?}", input.gdl_type()),
            })
        }
    };
    Ok(XdlValue::Double(factorial_val(n)))
}

/// BESELJ - Bessel function of the first kind
pub fn beselj(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(format!(
            "BESELJ: Expected 2 arguments (x, n), got {}",
            args.len()
        )));
    }

    let n = match &args[1] {
        XdlValue::Long(v) => *v,
        XdlValue::Int(v) => *v as i32,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let input = &args[0];

    // Handle arrays
    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr.iter().map(|&x| libm::jn(n, x)).collect();
        return Ok(XdlValue::Array(result));
    }

    if let XdlValue::MultiDimArray { data, shape } = input {
        let result: Vec<f64> = data.iter().map(|&x| libm::jn(n, x)).collect();
        return Ok(XdlValue::MultiDimArray {
            data: result,
            shape: shape.clone(),
        });
    }

    // Handle scalar
    let val = to_float(input)?;
    Ok(from_float(libm::jn(n, val), input.gdl_type()))
}

/// BESELY - Bessel function of the second kind
pub fn besely(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(format!(
            "BESELY: Expected 2 arguments (x, n), got {}",
            args.len()
        )));
    }

    let n = match &args[1] {
        XdlValue::Long(v) => *v,
        XdlValue::Int(v) => *v as i32,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let input = &args[0];

    // Handle arrays
    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr.iter().map(|&x| libm::yn(n, x)).collect();
        return Ok(XdlValue::Array(result));
    }

    if let XdlValue::MultiDimArray { data, shape } = input {
        let result: Vec<f64> = data.iter().map(|&x| libm::yn(n, x)).collect();
        return Ok(XdlValue::MultiDimArray {
            data: result,
            shape: shape.clone(),
        });
    }

    // Handle scalar
    let val = to_float(input)?;
    Ok(from_float(libm::yn(n, val), input.gdl_type()))
}

/// BESELI - Modified Bessel function of the first kind I_n(x)
/// Using the relation I_n(x) = i^(-n) * J_n(i*x) and series expansion
pub fn beseli(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(format!(
            "BESELI: Expected 2 arguments (x, n), got {}",
            args.len()
        )));
    }

    let n = match &args[1] {
        XdlValue::Long(v) => *v as i32,
        XdlValue::Int(v) => *v as i32,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    // Compute modified Bessel I_n(x) using series expansion
    fn bessel_i(x: f64, n: i32) -> f64 {
        let n = n.unsigned_abs();
        if x == 0.0 {
            return if n == 0 { 1.0 } else { 0.0 };
        }

        let x2 = x * 0.5;
        let mut sum = 0.0;
        let mut term = libm::pow(x2, n as f64) / libm::tgamma((n + 1) as f64);
        let x2_sq = x2 * x2;

        for k in 0..100 {
            sum += term;
            term *= x2_sq / ((k + 1) as f64 * (n + k + 1) as f64);
            if term.abs() < 1e-15 * sum.abs() {
                break;
            }
        }
        sum
    }

    let input = &args[0];

    // Handle arrays
    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr.iter().map(|&x| bessel_i(x, n)).collect();
        return Ok(XdlValue::Array(result));
    }

    if let XdlValue::MultiDimArray { data, shape } = input {
        let result: Vec<f64> = data.iter().map(|&x| bessel_i(x, n)).collect();
        return Ok(XdlValue::MultiDimArray {
            data: result,
            shape: shape.clone(),
        });
    }

    // Handle scalar
    let val = to_float(input)?;
    Ok(from_float(bessel_i(val, n), input.gdl_type()))
}

/// BESELK - Modified Bessel function of the second kind K_n(x)
pub fn beselk(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(format!(
            "BESELK: Expected 2 arguments (x, n), got {}",
            args.len()
        )));
    }

    let n = match &args[1] {
        XdlValue::Long(v) => *v as i32,
        XdlValue::Int(v) => *v as i32,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    // Compute K_n(x) using asymptotic expansion for large x,
    // or relation to I_n for small x
    fn bessel_k(x: f64, n: i32) -> f64 {
        if x <= 0.0 {
            return f64::INFINITY;
        }

        let n = n.unsigned_abs() as i32;

        // For large x, use asymptotic expansion
        if x > 2.0 {
            let mu = 4.0 * (n as f64) * (n as f64);
            let mut term = 1.0;
            let mut sum = 1.0;
            let x8 = 8.0 * x;

            for k in 1..20 {
                let k_f = k as f64;
                term *= (mu - (2.0 * k_f - 1.0).powi(2)) / (k_f * x8);
                sum += term;
                if term.abs() < 1e-15 * sum.abs() {
                    break;
                }
            }

            (std::f64::consts::PI / (2.0 * x)).sqrt() * (-x).exp() * sum
        } else {
            // Use recurrence relation and series for small x
            // K_n(x) = (pi/2) * (I_{-n}(x) - I_n(x)) / sin(n*pi)
            // For integer n, use limiting form
            let x2 = x * 0.5;
            let gamma_n = libm::tgamma(n as f64);
            let mut sum = 0.5 * gamma_n * libm::pow(x2, -(n as f64));

            if n == 0 {
                // K_0 special case
                sum = -(x2.ln() + 0.5772156649) * bessel_i_local(x, 0);
                let x2_sq = x2 * x2;
                let mut term = 1.0;
                let mut psi = -0.5772156649;
                for k in 1..50 {
                    term *= x2_sq / (k as f64).powi(2);
                    psi += 1.0 / k as f64;
                    sum += term * psi;
                    if term.abs() < 1e-15 * sum.abs() {
                        break;
                    }
                }
            }
            sum
        }
    }

    fn bessel_i_local(x: f64, n: i32) -> f64 {
        let n = n.unsigned_abs();
        if x == 0.0 {
            return if n == 0 { 1.0 } else { 0.0 };
        }
        let x2 = x * 0.5;
        let mut sum = 0.0;
        let mut term = libm::pow(x2, n as f64) / libm::tgamma((n + 1) as f64);
        let x2_sq = x2 * x2;
        for k in 0..100 {
            sum += term;
            term *= x2_sq / ((k + 1) as f64 * (n + k + 1) as f64);
            if term.abs() < 1e-15 * sum.abs() {
                break;
            }
        }
        sum
    }

    let input = &args[0];

    // Handle arrays
    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr.iter().map(|&x| bessel_k(x, n)).collect();
        return Ok(XdlValue::Array(result));
    }

    if let XdlValue::MultiDimArray { data, shape } = input {
        let result: Vec<f64> = data.iter().map(|&x| bessel_k(x, n)).collect();
        return Ok(XdlValue::MultiDimArray {
            data: result,
            shape: shape.clone(),
        });
    }

    // Handle scalar
    let val = to_float(input)?;
    Ok(from_float(bessel_k(val, n), input.gdl_type()))
}

#[cfg(test)]
mod more_tests {
    use super::*;

    #[test]
    fn test_abs() {
        let args = vec![XdlValue::Double(-5.5)];
        let result = abs(&args).unwrap();
        assert!(matches!(result, XdlValue::Double(v) if (v - 5.5).abs() < 1e-10));

        let args = vec![XdlValue::Long(-42)];
        let result = abs(&args).unwrap();
        assert!(matches!(result, XdlValue::Long(42)));
    }

    #[test]
    fn test_type_preservation() {
        // Test that functions preserve input types
        let args = vec![XdlValue::Float(1.0)];
        let result = sin(&args).unwrap();
        assert!(matches!(result, XdlValue::Float(_)));

        let args = vec![XdlValue::Long(1)];
        let result = abs(&args).unwrap();
        assert!(matches!(result, XdlValue::Long(_)));
    }

    #[test]
    fn test_argument_count_errors() {
        // Test that functions reject wrong argument counts
        let args = vec![];
        assert!(sin(&args).is_err());

        let args = vec![XdlValue::Double(1.0), XdlValue::Double(2.0)];
        assert!(cos(&args).is_err());
    }
}
