//! Mathematical functions

use xdl_core::{Dimension, LongArray};
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

    // Handle multi-dimensional arrays
    if let XdlValue::MultiDimArray { data, shape } = input {
        let result: Vec<f64> = data
            .iter()
            .map(|&x| {
                if x < 0.0 {
                    f64::NAN // Return NaN for negative values
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

    // Handle 1D arrays
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

    // Handle complex numbers specially
    match input {
        XdlValue::Complex(c) => return Ok(XdlValue::Float(c.norm())),
        XdlValue::DComplex(c) => return Ok(XdlValue::Double(c.norm())),
        _ => {}
    }

    // Handle multi-dimensional arrays
    if let XdlValue::MultiDimArray { data, shape } = input {
        let result: Vec<f64> = data.iter().map(|&x| x.abs()).collect();
        return Ok(XdlValue::MultiDimArray {
            data: result,
            shape: shape.clone(),
        });
    }

    // Handle 1D arrays
    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr.iter().map(|&x| x.abs()).collect();
        return Ok(XdlValue::Array(result));
    }

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

pub fn sinh(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "SINH: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let input = &_args[0];

    // Handle arrays
    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr.iter().map(|&x| x.sinh()).collect();
        return Ok(XdlValue::Array(result));
    }

    // Handle multi-dimensional arrays
    if let XdlValue::MultiDimArray { data, shape } = input {
        let result: Vec<f64> = data.iter().map(|&x| x.sinh()).collect();
        return Ok(XdlValue::MultiDimArray {
            data: result,
            shape: shape.clone(),
        });
    }

    let float_val = to_float(input)?;
    let result = float_val.sinh();

    Ok(from_float(result, input.gdl_type()))
}

pub fn cosh(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "COSH: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let input = &_args[0];

    // Handle arrays
    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr.iter().map(|&x| x.cosh()).collect();
        return Ok(XdlValue::Array(result));
    }

    // Handle multi-dimensional arrays
    if let XdlValue::MultiDimArray { data, shape } = input {
        let result: Vec<f64> = data.iter().map(|&x| x.cosh()).collect();
        return Ok(XdlValue::MultiDimArray {
            data: result,
            shape: shape.clone(),
        });
    }

    let float_val = to_float(input)?;
    let result = float_val.cosh();

    Ok(from_float(result, input.gdl_type()))
}

pub fn tanh(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "TANH: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let input = &_args[0];

    // Handle arrays
    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr.iter().map(|&x| x.tanh()).collect();
        return Ok(XdlValue::Array(result));
    }

    // Handle multi-dimensional arrays
    if let XdlValue::MultiDimArray { data, shape } = input {
        let result: Vec<f64> = data.iter().map(|&x| x.tanh()).collect();
        return Ok(XdlValue::MultiDimArray {
            data: result,
            shape: shape.clone(),
        });
    }

    let float_val = to_float(input)?;
    let result = float_val.tanh();

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
    if _args.is_empty() || _args.len() > 2 {
        return Err(XdlError::InvalidArgument(format!(
            "ATAN: Expected 1 or 2 arguments, got {}",
            _args.len()
        )));
    }

    if _args.len() == 1 {
        // Single argument: atan(x)
        let input = &_args[0];
        let float_val = to_float(input)?;
        let result = float_val.atan();
        Ok(from_float(result, input.gdl_type()))
    } else {
        // Two arguments: atan(y, x) = atan2(y, x)
        let y = to_float(&_args[0])?;
        let x = to_float(&_args[1])?;
        let result = y.atan2(x);
        Ok(XdlValue::Double(result))
    }
}

/// ATAN2 - Two-argument arctangent
/// ATAN2(y, x) computes arctan(y/x) with proper quadrant handling
/// Returns angle in radians in range [-π, π]
pub fn atan2(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 2 {
        return Err(XdlError::InvalidArgument(format!(
            "ATAN2: Expected 2 arguments (y, x), got {}",
            _args.len()
        )));
    }

    let y = to_float(&_args[0])?;
    let x = to_float(&_args[1])?;
    let result = y.atan2(x);
    Ok(XdlValue::Double(result))
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

/// Compute binomial coefficient: NCHOOSEK(n, k) = n! / (k! * (n-k)!)
/// Returns the number of ways to choose k items from n items
pub fn nchoosek(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 2 {
        return Err(XdlError::InvalidArgument(format!(
            "NCHOOSEK: Expected 2 arguments, got {}",
            _args.len()
        )));
    }

    let n = to_float(&_args[0])? as i64;
    let k = to_float(&_args[1])? as i64;

    // Validate inputs
    if n < 0 || k < 0 {
        return Err(XdlError::InvalidArgument(
            "NCHOOSEK: Arguments must be non-negative".to_string(),
        ));
    }

    if k > n {
        return Ok(XdlValue::Double(0.0));
    }

    // Use symmetry: C(n, k) = C(n, n-k)
    let k = k.min(n - k);

    // Compute binomial coefficient iteratively to avoid overflow
    let mut result = 1.0f64;
    for i in 0..k {
        result *= (n - i) as f64;
        result /= (i + 1) as f64;
    }

    Ok(XdlValue::Double(result))
}

/// Generate floating point array: FINDGEN(n) or FINDGEN(d1, d2, ..., dn)
pub fn findgen(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "FINDGEN: Expected at least 1 argument".to_string(),
        ));
    }

    // Parse all dimensions
    let mut dims: Vec<usize> = Vec::new();
    for arg in _args {
        let dim = match arg {
            XdlValue::Long(v) => *v as usize,
            XdlValue::Int(v) => *v as usize,
            XdlValue::Byte(v) => *v as usize,
            _ => {
                return Err(XdlError::TypeMismatch {
                    expected: "integer".to_string(),
                    actual: format!("{:?}", arg.gdl_type()),
                })
            }
        };
        dims.push(dim);
    }

    // Calculate total size
    let total_size: usize = dims.iter().product();

    // Generate array with sequential values
    let data: Vec<f64> = (0..total_size).map(|i| i as f64).collect();

    // Return appropriate type based on dimensions
    if dims.len() == 1 {
        Ok(XdlValue::Array(data))
    } else {
        Ok(XdlValue::MultiDimArray { data, shape: dims })
    }
}

/// Generate integer array: INDGEN(n)
pub fn indgen(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "INDGEN: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let n = match &_args[0] {
        XdlValue::Long(v) => *v as usize,
        XdlValue::Int(v) => *v as usize,
        XdlValue::Byte(v) => *v as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", _args[0].gdl_type()),
            })
        }
    };

    let data: Vec<i32> = (0..n as i32).collect();
    let dim = Dimension::from_size(n)?;
    let _array = LongArray::from_vec(data, dim, GdlType::Long)?;

    // For now, return the first element to show it works
    // TODO: Properly integrate array types with XdlValue
    Ok(XdlValue::Long(0))
}

/// Generate double precision array: DINDGEN(n)
pub fn dindgen(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "DINDGEN: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let n = match &_args[0] {
        XdlValue::Long(v) => *v as usize,
        XdlValue::Int(v) => *v as usize,
        XdlValue::Byte(v) => *v as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", _args[0].gdl_type()),
            })
        }
    };

    let data: Vec<f64> = (0..n).map(|i| i as f64).collect();
    Ok(XdlValue::Array(data))
}

/// Generate byte array: BINDGEN(n)
pub fn bindgen(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "BINDGEN: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let n = match &_args[0] {
        XdlValue::Long(v) => *v as usize,
        XdlValue::Int(v) => *v as usize,
        XdlValue::Byte(v) => *v as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", _args[0].gdl_type()),
            })
        }
    };

    let data: Vec<f64> = (0..n).map(|i| (i as u8) as f64).collect();
    Ok(XdlValue::Array(data))
}

/// Generate long integer array: LINDGEN(n)
pub fn lindgen(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "LINDGEN: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let n = match &_args[0] {
        XdlValue::Long(v) => *v as usize,
        XdlValue::Int(v) => *v as usize,
        XdlValue::Byte(v) => *v as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", _args[0].gdl_type()),
            })
        }
    };

    let data: Vec<f64> = (0..n).map(|i| i as f64).collect();
    Ok(XdlValue::Array(data))
}

/// Generate unsigned integer array: UINDGEN(n)
pub fn uindgen(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "UINDGEN: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let n = match &_args[0] {
        XdlValue::Long(v) => *v as usize,
        XdlValue::Int(v) => *v as usize,
        XdlValue::Byte(v) => *v as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", _args[0].gdl_type()),
            })
        }
    };

    let data: Vec<f64> = (0..n).map(|i| (i as u16) as f64).collect();
    Ok(XdlValue::Array(data))
}

/// Generate unsigned long integer array: ULINDGEN(n)
pub fn ulindgen(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "ULINDGEN: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let n = match &_args[0] {
        XdlValue::Long(v) => *v as usize,
        XdlValue::Int(v) => *v as usize,
        XdlValue::Byte(v) => *v as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", _args[0].gdl_type()),
            })
        }
    };

    let data: Vec<f64> = (0..n).map(|i| (i as u32) as f64).collect();
    Ok(XdlValue::Array(data))
}

/// Generate 64-bit integer array: L64INDGEN(n)
pub fn l64indgen(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    if _args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "L64INDGEN: Expected 1 argument, got {}",
            _args.len()
        )));
    }

    let n = match &_args[0] {
        XdlValue::Long(v) => *v as usize,
        XdlValue::Int(v) => *v as usize,
        XdlValue::Byte(v) => *v as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", _args[0].gdl_type()),
            })
        }
    };

    let data: Vec<f64> = (0..n).map(|i| i as f64).collect();
    Ok(XdlValue::Array(data))
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

/// FLOAT - Convert to floating point
pub fn float(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "FLOAT: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let input = &args[0];
    let float_val = to_float(input)? as f32;
    Ok(XdlValue::Float(float_val))
}

/// FIX - Convert to integer (truncate toward zero)
pub fn fix(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "FIX: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let input = &args[0];
    let float_val = to_float(input)?;
    Ok(XdlValue::Int(float_val as i16))
}

/// LONG - Convert to long integer
pub fn long(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "LONG: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let input = &args[0];
    let float_val = to_float(input)?;
    Ok(XdlValue::Long(float_val as i32))
}

/// BYTE - Convert to byte
pub fn byte(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "BYTE: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let input = &args[0];
    let float_val = to_float(input)?;
    Ok(XdlValue::Byte(float_val as u8))
}

/// DOUBLE - Convert to double precision
pub fn double(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "DOUBLE: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let input = &args[0];
    let float_val = to_float(input)?;
    Ok(XdlValue::Double(float_val))
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

/// RANDOMN - Generate normal/Gaussian random numbers
/// Uses Box-Muller transform to generate from uniform distribution
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

    // Helper to generate uniform [0,1) using LCG
    let mut rng_state = seed;
    let mut uniform = || -> f64 {
        let a = 1664525u64;
        let c = 1013904223u64;
        rng_state = a.wrapping_mul(rng_state).wrapping_add(c);
        ((rng_state % 1000000) as f64) / 1000000.0
    };

    // Box-Muller transform to generate normal random number
    let mut box_muller = || -> f64 {
        let u1 = uniform();
        let u2 = uniform();
        // Avoid log(0) by ensuring u1 > 0
        let u1 = if u1 < 1e-10 { 1e-10 } else { u1 };
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    };

    if args.len() == 1 {
        // Single normal random number
        Ok(XdlValue::Double(box_muller()))
    } else {
        // Array of normal random numbers
        let n = args[1].to_long()? as usize;
        let values: Vec<f64> = (0..n).map(|_| box_muller()).collect();
        Ok(XdlValue::Array(values))
    }
}

/// GAMMA - Gamma function
/// GAMMA(x) computes the gamma function Γ(x)
/// For positive integers: Γ(n) = (n-1)!
pub fn gamma(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "GAMMA: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let input = &args[0];

    // Handle arrays
    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr.iter().map(|&x| gamma_function(x)).collect();
        return Ok(XdlValue::Array(result));
    }

    let x = to_float(input)?;
    let result = gamma_function(x);
    Ok(from_float(result, input.gdl_type()))
}

/// Helper: Gamma function using Lanczos approximation
fn gamma_function(x: f64) -> f64 {
    // Lanczos approximation coefficients (g=7)
    const G: f64 = 7.0;
    const COEF: [f64; 9] = [
        0.9999999999998099,
        676.5203681218851,
        -1259.139216722403,
        771.3234287776531,
        -176.6150291621406,
        12.507343278686905,
        -0.1385710952657201,
        9.984369578019572e-6,
        1.505632735149312e-7,
    ];

    if x < 0.5 {
        // Reflection formula: Γ(1-x)Γ(x) = π/sin(πx)
        std::f64::consts::PI / ((std::f64::consts::PI * x).sin() * gamma_function(1.0 - x))
    } else {
        let x = x - 1.0;
        let mut a = COEF[0];
        for (i, &c) in COEF.iter().enumerate().skip(1) {
            a += c / (x + i as f64);
        }
        let t = x + G + 0.5;
        (2.0 * std::f64::consts::PI).sqrt() * t.powf(x + 0.5) * (-t).exp() * a
    }
}

/// LNGAMMA - Natural log of gamma function
/// LNGAMMA(x) computes ln(Γ(x))
pub fn lngamma(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "LNGAMMA: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let input = &args[0];
    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr.iter().map(|&x| gamma_function(x).ln()).collect();
        return Ok(XdlValue::Array(result));
    }

    let x = to_float(input)?;
    let result = gamma_function(x).ln();
    Ok(from_float(result, input.gdl_type()))
}

/// ERF - Error function
/// ERF(x) = (2/√π) ∫₀ˣ e^(-t²) dt
pub fn erf(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "ERF: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let input = &args[0];

    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr.iter().map(|&x| erf_function(x)).collect();
        return Ok(XdlValue::Array(result));
    }

    let x = to_float(input)?;
    let result = erf_function(x);
    Ok(from_float(result, input.gdl_type()))
}

/// Helper: Error function using polynomial approximation
fn erf_function(x: f64) -> f64 {
    // Abramowitz and Stegun approximation
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

    sign * y
}

/// ERFC - Complementary error function
/// ERFC(x) = 1 - ERF(x)
pub fn erfc(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "ERFC: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let input = &args[0];

    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr.iter().map(|&x| 1.0 - erf_function(x)).collect();
        return Ok(XdlValue::Array(result));
    }

    let x = to_float(input)?;
    let result = 1.0 - erf_function(x);
    Ok(from_float(result, input.gdl_type()))
}

/// BESSEL_J - Bessel function of the first kind
/// BESSEL_J(x, n) computes Jₙ(x)
pub fn bessel_j(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 2 {
        return Err(XdlError::InvalidArgument(format!(
            "BESSEL_J: Expected 2 arguments (x, n), got {}",
            args.len()
        )));
    }

    let x = to_float(&args[0])?;
    let n = match &args[1] {
        XdlValue::Int(v) => *v as i32,
        XdlValue::Long(v) => *v,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let result = bessel_j_function(x, n);
    Ok(XdlValue::Double(result))
}

/// Helper: Bessel function J using series expansion (for small x)
fn bessel_j_function(x: f64, n: i32) -> f64 {
    if n < 0 {
        return if n % 2 == 0 {
            bessel_j_function(x, -n)
        } else {
            -bessel_j_function(x, -n)
        };
    }

    // Series expansion for small to moderate x
    let mut sum = 0.0;
    let half_x = x / 2.0;

    for k in 0..50 {
        let term = (-1.0_f64).powi(k as i32) * half_x.powi(n + 2 * (k as i32))
            / (factorial(k) * factorial(n as usize + k));
        sum += term;
        if term.abs() < 1e-15 * sum.abs() {
            break;
        }
    }

    sum
}

/// Helper: Factorial function
fn factorial(n: usize) -> f64 {
    if n == 0 || n == 1 {
        1.0
    } else {
        (1..=n).fold(1.0, |acc, x| acc * x as f64)
    }
}

/// FACTORIAL - Factorial function
/// FACTORIAL(n) computes n!
pub fn factorial_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "FACTORIAL: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let n = match &args[0] {
        XdlValue::Int(v) => *v as usize,
        XdlValue::Long(v) => *v as usize,
        XdlValue::Byte(v) => *v as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if n > 170 {
        return Err(XdlError::MathError(
            "FACTORIAL: Argument too large (>170)".to_string(),
        ));
    }

    let result = factorial(n);
    Ok(XdlValue::Double(result))
}

/// ASINH - Inverse hyperbolic sine
/// ASINH(x) = ln(x + sqrt(x² + 1))
pub fn asinh(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "ASINH: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let input = &args[0];

    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr.iter().map(|&x| x.asinh()).collect();
        return Ok(XdlValue::Array(result));
    }

    let x = to_float(input)?;
    let result = x.asinh();
    Ok(from_float(result, input.gdl_type()))
}

/// ACOSH - Inverse hyperbolic cosine
/// ACOSH(x) = ln(x + sqrt(x² - 1)), x >= 1
pub fn acosh(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "ACOSH: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let input = &args[0];

    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr
            .iter()
            .map(|&x| if x < 1.0 { f64::NAN } else { x.acosh() })
            .collect();
        return Ok(XdlValue::Array(result));
    }

    let x = to_float(input)?;
    if x < 1.0 {
        return Err(XdlError::MathError(
            "ACOSH: Argument must be >= 1".to_string(),
        ));
    }
    let result = x.acosh();
    Ok(from_float(result, input.gdl_type()))
}

/// ATANH - Inverse hyperbolic tangent
/// ATANH(x) = 0.5 * ln((1+x)/(1-x)), |x| < 1
pub fn atanh(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "ATANH: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let input = &args[0];

    if let XdlValue::Array(arr) = input {
        let result: Vec<f64> = arr
            .iter()
            .map(|&x| if x.abs() >= 1.0 { f64::NAN } else { x.atanh() })
            .collect();
        return Ok(XdlValue::Array(result));
    }

    let x = to_float(input)?;
    if x.abs() >= 1.0 {
        return Err(XdlError::MathError(
            "ATANH: Argument must be in range (-1, 1)".to_string(),
        ));
    }
    let result = x.atanh();
    Ok(from_float(result, input.gdl_type()))
}

/// BETA - Beta function
/// BETA(x, y) = Γ(x)Γ(y)/Γ(x+y)
pub fn beta(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 2 {
        return Err(XdlError::InvalidArgument(format!(
            "BETA: Expected 2 arguments (x, y), got {}",
            args.len()
        )));
    }

    let x = to_float(&args[0])?;
    let y = to_float(&args[1])?;

    // Beta(x, y) = Gamma(x) * Gamma(y) / Gamma(x + y)
    let result = gamma_function(x) * gamma_function(y) / gamma_function(x + y);
    Ok(XdlValue::Double(result))
}

/// GCD - Greatest Common Divisor
/// GCD(a, b) computes the GCD of two integers
pub fn gcd(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 2 {
        return Err(XdlError::InvalidArgument(format!(
            "GCD: Expected 2 arguments, got {}",
            args.len()
        )));
    }

    let a = match &args[0] {
        XdlValue::Int(v) => (*v).abs() as i64,
        XdlValue::Long(v) => (*v).abs() as i64,
        XdlValue::Long64(v) => v.abs(),
        XdlValue::Double(v) => (*v as i64).abs(),
        XdlValue::Float(v) => (*v as i64).abs(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let b = match &args[1] {
        XdlValue::Int(v) => (*v).abs() as i64,
        XdlValue::Long(v) => (*v).abs() as i64,
        XdlValue::Long64(v) => v.abs(),
        XdlValue::Double(v) => (*v as i64).abs(),
        XdlValue::Float(v) => (*v as i64).abs(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    // Euclidean algorithm
    let result = gcd_helper(a, b);
    Ok(XdlValue::Long64(result))
}

fn gcd_helper(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}

/// LCM - Least Common Multiple
/// LCM(a, b) computes the LCM of two integers
pub fn lcm(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 2 {
        return Err(XdlError::InvalidArgument(format!(
            "LCM: Expected 2 arguments, got {}",
            args.len()
        )));
    }

    let a = match &args[0] {
        XdlValue::Int(v) => (*v).abs() as i64,
        XdlValue::Long(v) => (*v).abs() as i64,
        XdlValue::Long64(v) => v.abs(),
        XdlValue::Double(v) => (*v as i64).abs(),
        XdlValue::Float(v) => (*v as i64).abs(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let b = match &args[1] {
        XdlValue::Int(v) => (*v).abs() as i64,
        XdlValue::Long(v) => (*v).abs() as i64,
        XdlValue::Long64(v) => v.abs(),
        XdlValue::Double(v) => (*v as i64).abs(),
        XdlValue::Float(v) => (*v as i64).abs(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    if a == 0 || b == 0 {
        return Ok(XdlValue::Long64(0));
    }

    // LCM(a, b) = |a * b| / GCD(a, b)
    let result = (a * b).abs() / gcd_helper(a, b);
    Ok(XdlValue::Long64(result))
}

/// POLY - Evaluate polynomial
/// POLY(x, coeffs) evaluates polynomial with given coefficients
/// Result = c[0] + c[1]*x + c[2]*x^2 + ... + c[n]*x^n
pub fn poly(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 2 {
        return Err(XdlError::InvalidArgument(format!(
            "POLY: Expected 2 arguments (x, coeffs), got {}",
            args.len()
        )));
    }

    let x = to_float(&args[0])?;

    let coeffs = match &args[1] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    if coeffs.is_empty() {
        return Ok(XdlValue::Double(0.0));
    }

    // Horner's method for polynomial evaluation
    let mut result = coeffs[coeffs.len() - 1];
    for i in (0..coeffs.len() - 1).rev() {
        result = result * x + coeffs[i];
    }

    Ok(XdlValue::Double(result))
}

/// BINOMIAL - Binomial coefficient
/// BINOMIAL(n, k) computes "n choose k" = n! / (k! * (n-k)!)
pub fn binomial(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 2 {
        return Err(XdlError::InvalidArgument(format!(
            "BINOMIAL: Expected 2 arguments (n, k), got {}",
            args.len()
        )));
    }

    let n = match &args[0] {
        XdlValue::Int(v) => *v as i64,
        XdlValue::Long(v) => *v as i64,
        XdlValue::Long64(v) => *v,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let k = match &args[1] {
        XdlValue::Int(v) => *v as i64,
        XdlValue::Long(v) => *v as i64,
        XdlValue::Long64(v) => *v,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    if k < 0 || k > n {
        return Ok(XdlValue::Double(0.0));
    }

    // Use efficient calculation: C(n,k) = n! / (k! * (n-k)!)
    // Optimize by computing C(n,k) = C(n, n-k) when k > n/2
    let k = if k > n - k { n - k } else { k };

    let mut result = 1.0;
    for i in 0..k {
        result *= (n - i) as f64 / (i + 1) as f64;
    }

    Ok(XdlValue::Double(result))
}

/// UINT - Convert to unsigned integer (16-bit)
pub fn uint(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "UINT: Expected 1 argument".to_string(),
        ));
    }

    let input = &args[0];

    // Handle arrays
    if let XdlValue::Array(arr) = input {
        let result: Vec<XdlValue> = arr.iter().map(|&x| XdlValue::UInt(x as u16)).collect();
        return Ok(XdlValue::NestedArray(result));
    }

    let value = match input {
        XdlValue::Byte(v) => *v as u16,
        XdlValue::Int(v) => *v as u16,
        XdlValue::Long(v) => *v as u16,
        XdlValue::Float(v) => *v as u16,
        XdlValue::Double(v) => *v as u16,
        XdlValue::UInt(v) => *v,
        XdlValue::ULong(v) => *v as u16,
        XdlValue::Long64(v) => *v as u16,
        XdlValue::ULong64(v) => *v as u16,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: format!("{:?}", input.gdl_type()),
            })
        }
    };

    Ok(XdlValue::UInt(value))
}

/// ULONG - Convert to unsigned long integer (32-bit)
pub fn ulong(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "ULONG: Expected 1 argument".to_string(),
        ));
    }

    let input = &args[0];

    // Handle arrays
    if let XdlValue::Array(arr) = input {
        let result: Vec<XdlValue> = arr.iter().map(|&x| XdlValue::ULong(x as u32)).collect();
        return Ok(XdlValue::NestedArray(result));
    }

    let value = match input {
        XdlValue::Byte(v) => *v as u32,
        XdlValue::Int(v) => *v as u32,
        XdlValue::Long(v) => *v as u32,
        XdlValue::Float(v) => *v as u32,
        XdlValue::Double(v) => *v as u32,
        XdlValue::UInt(v) => *v as u32,
        XdlValue::ULong(v) => *v,
        XdlValue::Long64(v) => *v as u32,
        XdlValue::ULong64(v) => *v as u32,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: format!("{:?}", input.gdl_type()),
            })
        }
    };

    Ok(XdlValue::ULong(value))
}

/// LONG64 - Convert to 64-bit signed integer
pub fn long64(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "LONG64: Expected 1 argument".to_string(),
        ));
    }

    let input = &args[0];

    // Handle arrays
    if let XdlValue::Array(arr) = input {
        let result: Vec<XdlValue> = arr.iter().map(|&x| XdlValue::Long64(x as i64)).collect();
        return Ok(XdlValue::NestedArray(result));
    }

    let value = match input {
        XdlValue::Byte(v) => *v as i64,
        XdlValue::Int(v) => *v as i64,
        XdlValue::Long(v) => *v as i64,
        XdlValue::Float(v) => *v as i64,
        XdlValue::Double(v) => *v as i64,
        XdlValue::UInt(v) => *v as i64,
        XdlValue::ULong(v) => *v as i64,
        XdlValue::Long64(v) => *v,
        XdlValue::ULong64(v) => *v as i64,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: format!("{:?}", input.gdl_type()),
            })
        }
    };

    Ok(XdlValue::Long64(value))
}

/// ULONG64 - Convert to 64-bit unsigned integer
pub fn ulong64(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "ULONG64: Expected 1 argument".to_string(),
        ));
    }

    let input = &args[0];

    // Handle arrays
    if let XdlValue::Array(arr) = input {
        let result: Vec<XdlValue> = arr.iter().map(|&x| XdlValue::ULong64(x as u64)).collect();
        return Ok(XdlValue::NestedArray(result));
    }

    let value = match input {
        XdlValue::Byte(v) => *v as u64,
        XdlValue::Int(v) => *v as u64,
        XdlValue::Long(v) => *v as u64,
        XdlValue::Float(v) => *v as u64,
        XdlValue::Double(v) => *v as u64,
        XdlValue::UInt(v) => *v as u64,
        XdlValue::ULong(v) => *v as u64,
        XdlValue::Long64(v) => *v as u64,
        XdlValue::ULong64(v) => *v,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: format!("{:?}", input.gdl_type()),
            })
        }
    };

    Ok(XdlValue::ULong64(value))
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
