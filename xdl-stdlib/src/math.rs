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
