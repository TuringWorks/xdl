//! String manipulation functions

use xdl_core::{XdlError, XdlResult, XdlValue};

pub struct StringFunctions;

impl StringFunctions {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StringFunctions {
    fn default() -> Self {
        Self::new()
    }
}

/// STRLEN - Get string length
pub fn strlen(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "STRLEN: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let s = match &args[0] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    Ok(XdlValue::Long(s.len() as i32))
}

/// STRPOS - Find substring position
pub fn strpos(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 2 {
        return Err(XdlError::InvalidArgument(format!(
            "STRPOS: Expected 2 arguments, got {}",
            args.len()
        )));
    }

    let haystack = match &args[0] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let needle = match &args[1] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    match haystack.find(needle) {
        Some(pos) => Ok(XdlValue::Long(pos as i32)),
        None => Ok(XdlValue::Long(-1)), // XDL returns -1 if not found
    }
}

/// STRMID - Extract substring
pub fn strmid(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 || args.len() > 3 {
        return Err(XdlError::InvalidArgument(format!(
            "STRMID: Expected 2-3 arguments, got {}",
            args.len()
        )));
    }

    let s = match &args[0] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let start = match &args[1] {
        XdlValue::Long(n) => *n as usize,
        XdlValue::Int(n) => *n as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let result = if args.len() == 3 {
        let length = match &args[2] {
            XdlValue::Long(n) => *n as usize,
            XdlValue::Int(n) => *n as usize,
            _ => {
                return Err(XdlError::TypeMismatch {
                    expected: "integer".to_string(),
                    actual: format!("{:?}", args[2].gdl_type()),
                })
            }
        };

        let end = std::cmp::min(start + length, s.len());
        if start < s.len() {
            s[start..end].to_string()
        } else {
            String::new()
        }
    } else if start < s.len() {
        s[start..].to_string()
    } else {
        String::new()
    };

    Ok(XdlValue::String(result))
}

/// STRUPCASE - Convert to uppercase
pub fn strupcase(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "STRUPCASE: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let s = match &args[0] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    Ok(XdlValue::String(s.to_uppercase()))
}

/// STRLOWCASE - Convert to lowercase
pub fn strlowcase(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "STRLOWCASE: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let s = match &args[0] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    Ok(XdlValue::String(s.to_lowercase()))
}

/// STRING - Convert any value to string representation
/// Syntax: result = STRING(expression [, FORMAT=format_string])
pub fn string_fn(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "STRING: Expected at least 1 argument, got 0".to_string(),
        ));
    }

    // For now, implement basic conversion without FORMAT keyword support
    // Format support can be added later as an enhancement
    let value = &args[0];

    let result = match value {
        XdlValue::String(s) => s.clone(),
        XdlValue::Int(n) => n.to_string(),
        XdlValue::Long(n) => n.to_string(),
        XdlValue::Float(f) => {
            // Format floats with consistent scientific notation for very small/large values
            if f.abs() < 1e-4 || f.abs() >= 1e7 {
                format!("{:e}", f)
            } else {
                format!("{}", f)
            }
        }
        XdlValue::Double(d) => {
            // Format doubles with consistent scientific notation for very small/large values
            if d.abs() < 1e-4 || d.abs() >= 1e7 {
                format!("{:e}", d)
            } else {
                format!("{}", d)
            }
        }
        XdlValue::Byte(b) => b.to_string(),
        XdlValue::Array(arr) => {
            // For arrays, show size
            // In current implementation, arrays are Vec<f64>
            format!("Array({})", arr.len())
        }
        XdlValue::NestedArray(nested) => {
            // For nested arrays, show element count
            format!("NestedArray({})", nested.len())
        }
        XdlValue::MultiDimArray { data: _, shape } => {
            // For multi-dim arrays, show shape
            let shape_str = shape
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join("x");
            format!("Array[{}]", shape_str)
        }
        XdlValue::UInt(n) => n.to_string(),
        XdlValue::ULong(n) => n.to_string(),
        XdlValue::Long64(n) => n.to_string(),
        XdlValue::ULong64(n) => n.to_string(),
        XdlValue::Complex(c) => format!("({}, {}i)", c.re, c.im),
        XdlValue::DComplex(c) => format!("({}, {}i)", c.re, c.im),
        XdlValue::Pointer(p) => format!("<Pointer:0x{:x}>", p),
        XdlValue::ObjRef(o) => format!("<Object:0x{:x}>", o),
        XdlValue::PythonObject(id) => format!("<Python:{}>", id),
        XdlValue::DataFrame(id) => format!("<DataFrame:{}>", id),
        XdlValue::Struct(_) => "<Struct>".to_string(),
        XdlValue::Undefined => "!NULL".to_string(),
    };

    Ok(XdlValue::String(result))
}
