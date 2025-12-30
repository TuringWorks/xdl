//! Complex number functions

use num_complex::Complex64;
use xdl_core::{XdlError, XdlResult, XdlValue};

/// COMPLEX - Create a complex number from real and imaginary parts
/// complex(real, imag) returns a complex number
pub fn complex(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 2 {
        return Err(XdlError::InvalidArgument(format!(
            "COMPLEX: Expected 2 arguments (real, imag), got {}",
            args.len()
        )));
    }

    let real = match &args[0] {
        XdlValue::Double(v) => *v,
        XdlValue::Float(v) => *v as f64,
        XdlValue::Long(v) => *v as f64,
        XdlValue::Int(v) => *v as f64,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let imag = match &args[1] {
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

    Ok(XdlValue::DComplex(Complex64::new(real, imag)))
}

/// REAL - Extract real part of complex number
pub fn real_part(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "REAL: Expected 1 argument, got {}",
            args.len()
        )));
    }

    match &args[0] {
        XdlValue::Complex(c) => Ok(XdlValue::Float(c.re)),
        XdlValue::DComplex(c) => Ok(XdlValue::Double(c.re)),
        // For real numbers, return as-is
        v => Ok(v.clone()),
    }
}

/// IMAGINARY - Extract imaginary part of complex number
pub fn imaginary_part(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "IMAGINARY: Expected 1 argument, got {}",
            args.len()
        )));
    }

    match &args[0] {
        XdlValue::Complex(c) => Ok(XdlValue::Float(c.im)),
        XdlValue::DComplex(c) => Ok(XdlValue::Double(c.im)),
        // For real numbers, return 0
        _ => Ok(XdlValue::Double(0.0)),
    }
}

/// CONJ - Complex conjugate
pub fn conj(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "CONJ: Expected 1 argument, got {}",
            args.len()
        )));
    }

    match &args[0] {
        XdlValue::Complex(c) => Ok(XdlValue::Complex(c.conj())),
        XdlValue::DComplex(c) => Ok(XdlValue::DComplex(c.conj())),
        // For real numbers, return as-is
        v => Ok(v.clone()),
    }
}

/// ABS for complex numbers - magnitude
pub fn complex_abs(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "ABS: Expected 1 argument, got {}",
            args.len()
        )));
    }

    match &args[0] {
        XdlValue::Complex(c) => Ok(XdlValue::Float(c.norm())),
        XdlValue::DComplex(c) => Ok(XdlValue::Double(c.norm())),
        XdlValue::Double(v) => Ok(XdlValue::Double(v.abs())),
        XdlValue::Float(v) => Ok(XdlValue::Float(v.abs())),
        XdlValue::Long(v) => Ok(XdlValue::Long(v.abs())),
        XdlValue::Int(v) => Ok(XdlValue::Int(v.abs())),
        _ => Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// ATAN with complex support - phase/argument of complex number
pub fn complex_atan(c: &Complex64) -> f64 {
    c.arg()
}

/// DCOMPLEX - Convert to double-precision complex
/// DCOMPLEX(real [, imag])
pub fn dcomplex(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "DCOMPLEX: Expected at least 1 argument".to_string(),
        ));
    }

    let real = match &args[0] {
        XdlValue::Double(v) => *v,
        XdlValue::Float(v) => *v as f64,
        XdlValue::Long(v) => *v as f64,
        XdlValue::Int(v) => *v as f64,
        XdlValue::Complex(c) => return Ok(XdlValue::DComplex(Complex64::new(c.re as f64, c.im as f64))),
        XdlValue::DComplex(c) => return Ok(XdlValue::DComplex(*c)),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let imag = if args.len() > 1 {
        match &args[1] {
            XdlValue::Double(v) => *v,
            XdlValue::Float(v) => *v as f64,
            XdlValue::Long(v) => *v as f64,
            XdlValue::Int(v) => *v as f64,
            _ => 0.0,
        }
    } else {
        0.0
    };

    Ok(XdlValue::DComplex(Complex64::new(real, imag)))
}

/// COMPLEXARR - Create an array of complex zeros
/// COMPLEXARR(d1 [, d2, ...])
pub fn complexarr(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "COMPLEXARR: Expected at least 1 dimension".to_string(),
        ));
    }

    let mut total_size = 1usize;
    let mut shape = Vec::new();

    for arg in args {
        let dim = match arg {
            XdlValue::Long(v) => *v as usize,
            XdlValue::Int(v) => *v as usize,
            _ => {
                return Err(XdlError::TypeMismatch {
                    expected: "integer".to_string(),
                    actual: format!("{:?}", arg.gdl_type()),
                })
            }
        };
        total_size *= dim;
        shape.push(dim);
    }

    // Create array of complex zeros
    let zeros: Vec<XdlValue> = (0..total_size)
        .map(|_| XdlValue::Complex(num_complex::Complex32::new(0.0, 0.0)))
        .collect();

    Ok(XdlValue::NestedArray(zeros))
}

/// DCOMPLEXARR - Create an array of double-precision complex zeros
/// DCOMPLEXARR(d1 [, d2, ...])
pub fn dcomplexarr(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "DCOMPLEXARR: Expected at least 1 dimension".to_string(),
        ));
    }

    let mut total_size = 1usize;
    let mut shape = Vec::new();

    for arg in args {
        let dim = match arg {
            XdlValue::Long(v) => *v as usize,
            XdlValue::Int(v) => *v as usize,
            _ => {
                return Err(XdlError::TypeMismatch {
                    expected: "integer".to_string(),
                    actual: format!("{:?}", arg.gdl_type()),
                })
            }
        };
        total_size *= dim;
        shape.push(dim);
    }

    // Create array of double complex zeros
    let zeros: Vec<XdlValue> = (0..total_size)
        .map(|_| XdlValue::DComplex(Complex64::new(0.0, 0.0)))
        .collect();

    Ok(XdlValue::NestedArray(zeros))
}

/// ARG / PHASE - Phase/argument of complex number
/// ARG(z) returns the argument (phase) of z in radians
pub fn arg(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "ARG: Expected complex number".to_string(),
        ));
    }

    match &args[0] {
        XdlValue::Complex(c) => Ok(XdlValue::Float(c.arg())),
        XdlValue::DComplex(c) => Ok(XdlValue::Double(c.arg())),
        XdlValue::Double(v) => {
            // Real number: phase is 0 or pi depending on sign
            Ok(XdlValue::Double(if *v >= 0.0 {
                0.0
            } else {
                std::f64::consts::PI
            }))
        }
        XdlValue::Float(v) => Ok(XdlValue::Float(if *v >= 0.0 {
            0.0
        } else {
            std::f32::consts::PI
        })),
        _ => Err(XdlError::TypeMismatch {
            expected: "complex or real".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// COMPLEX_EXP - Complex exponential
/// COMPLEX_EXP(z) returns e^z
pub fn complex_exp(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "COMPLEX_EXP: Expected complex number".to_string(),
        ));
    }

    match &args[0] {
        XdlValue::Complex(c) => {
            let result = c.exp();
            Ok(XdlValue::Complex(result))
        }
        XdlValue::DComplex(c) => {
            let result = c.exp();
            Ok(XdlValue::DComplex(result))
        }
        XdlValue::Double(v) => Ok(XdlValue::Double(v.exp())),
        XdlValue::Float(v) => Ok(XdlValue::Float(v.exp())),
        _ => Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// COMPLEX_LOG - Complex natural logarithm
/// COMPLEX_LOG(z) returns ln(z)
pub fn complex_log(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "COMPLEX_LOG: Expected complex number".to_string(),
        ));
    }

    match &args[0] {
        XdlValue::Complex(c) => {
            let result = c.ln();
            Ok(XdlValue::Complex(result))
        }
        XdlValue::DComplex(c) => {
            let result = c.ln();
            Ok(XdlValue::DComplex(result))
        }
        XdlValue::Double(v) => {
            if *v > 0.0 {
                Ok(XdlValue::Double(v.ln()))
            } else {
                // Return complex for non-positive
                let c = Complex64::new(*v, 0.0);
                Ok(XdlValue::DComplex(c.ln()))
            }
        }
        XdlValue::Float(v) => {
            if *v > 0.0 {
                Ok(XdlValue::Float(v.ln()))
            } else {
                let c = num_complex::Complex32::new(*v, 0.0);
                Ok(XdlValue::Complex(c.ln()))
            }
        }
        _ => Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// COMPLEX_SQRT - Complex square root
/// COMPLEX_SQRT(z) returns sqrt(z)
pub fn complex_sqrt(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "COMPLEX_SQRT: Expected number".to_string(),
        ));
    }

    match &args[0] {
        XdlValue::Complex(c) => Ok(XdlValue::Complex(c.sqrt())),
        XdlValue::DComplex(c) => Ok(XdlValue::DComplex(c.sqrt())),
        XdlValue::Double(v) => {
            if *v >= 0.0 {
                Ok(XdlValue::Double(v.sqrt()))
            } else {
                let c = Complex64::new(*v, 0.0);
                Ok(XdlValue::DComplex(c.sqrt()))
            }
        }
        XdlValue::Float(v) => {
            if *v >= 0.0 {
                Ok(XdlValue::Float(v.sqrt()))
            } else {
                let c = num_complex::Complex32::new(*v, 0.0);
                Ok(XdlValue::Complex(c.sqrt()))
            }
        }
        _ => Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// COMPLEX_SIN - Complex sine
pub fn complex_sin(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "COMPLEX_SIN: Expected number".to_string(),
        ));
    }

    match &args[0] {
        XdlValue::Complex(c) => Ok(XdlValue::Complex(c.sin())),
        XdlValue::DComplex(c) => Ok(XdlValue::DComplex(c.sin())),
        XdlValue::Double(v) => Ok(XdlValue::Double(v.sin())),
        XdlValue::Float(v) => Ok(XdlValue::Float(v.sin())),
        _ => Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// COMPLEX_COS - Complex cosine
pub fn complex_cos(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "COMPLEX_COS: Expected number".to_string(),
        ));
    }

    match &args[0] {
        XdlValue::Complex(c) => Ok(XdlValue::Complex(c.cos())),
        XdlValue::DComplex(c) => Ok(XdlValue::DComplex(c.cos())),
        XdlValue::Double(v) => Ok(XdlValue::Double(v.cos())),
        XdlValue::Float(v) => Ok(XdlValue::Float(v.cos())),
        _ => Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// POLAR - Create complex from polar coordinates
/// POLAR(r, theta) returns r * e^(i*theta)
pub fn polar(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "POLAR: Expected r and theta".to_string(),
        ));
    }

    let r = match &args[0] {
        XdlValue::Double(v) => *v,
        XdlValue::Float(v) => *v as f64,
        XdlValue::Long(v) => *v as f64,
        XdlValue::Int(v) => *v as f64,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let theta = match &args[1] {
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

    Ok(XdlValue::DComplex(Complex64::from_polar(r, theta)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complex_creation() {
        let args = vec![XdlValue::Double(3.0), XdlValue::Double(4.0)];
        let result = complex(&args).unwrap();

        if let XdlValue::DComplex(c) = result {
            assert_eq!(c.re, 3.0);
            assert_eq!(c.im, 4.0);
        } else {
            panic!("Expected DComplex");
        }
    }

    #[test]
    fn test_real_part() {
        let c = XdlValue::DComplex(Complex64::new(3.0, 4.0));
        let result = real_part(&[c]).unwrap();

        if let XdlValue::Double(v) = result {
            assert_eq!(v, 3.0);
        } else {
            panic!("Expected Double");
        }
    }

    #[test]
    fn test_imaginary_part() {
        let c = XdlValue::DComplex(Complex64::new(3.0, 4.0));
        let result = imaginary_part(&[c]).unwrap();

        if let XdlValue::Double(v) = result {
            assert_eq!(v, 4.0);
        } else {
            panic!("Expected Double");
        }
    }
}
