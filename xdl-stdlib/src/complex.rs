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
