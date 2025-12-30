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
        XdlValue::Object(id) => {
            if *id == 0 {
                "<NULL>".to_string()
            } else {
                format!("<Object:{}>", id)
            }
        }
        XdlValue::Undefined => "!NULL".to_string(),
    };

    Ok(XdlValue::String(result))
}

/// STRTRIM - Remove leading/trailing whitespace from string
/// Syntax: result = STRTRIM(string [, flag])
/// flag = 0: no trimming (default)
/// flag = 1: remove leading whitespace
/// flag = 2: remove both leading and trailing whitespace
pub fn strtrim(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() || args.len() > 2 {
        return Err(XdlError::InvalidArgument(format!(
            "STRTRIM: Expected 1-2 arguments, got {}",
            args.len()
        )));
    }

    let s = match &args[0] {
        XdlValue::String(s) => s.clone(),
        // Also handle numeric values by converting to string first
        other => other.to_string_repr(),
    };

    // Default flag is 0 (no trimming), but commonly used as 2
    let flag = if args.len() == 2 {
        match &args[1] {
            XdlValue::Long(n) => *n,
            XdlValue::Int(n) => *n as i32,
            XdlValue::Byte(n) => *n as i32,
            XdlValue::Float(f) => *f as i32,
            XdlValue::Double(d) => *d as i32,
            _ => {
                return Err(XdlError::TypeMismatch {
                    expected: "integer".to_string(),
                    actual: format!("{:?}", args[1].gdl_type()),
                })
            }
        }
    } else {
        0
    };

    let result = match flag {
        0 => s,                          // No trimming
        1 => s.trim_start().to_string(), // Remove leading whitespace
        2 => s.trim().to_string(),       // Remove both leading and trailing
        _ => {
            return Err(XdlError::InvalidArgument(format!(
                "STRTRIM: Invalid flag value {}. Expected 0, 1, or 2",
                flag
            )))
        }
    };

    Ok(XdlValue::String(result))
}

/// STRJOIN - Join string array with delimiter
/// Syntax: result = STRJOIN(array [, delimiter])
pub fn strjoin(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() || args.len() > 2 {
        return Err(XdlError::InvalidArgument(format!(
            "STRJOIN: Expected 1-2 arguments, got {}",
            args.len()
        )));
    }

    let delimiter = if args.len() > 1 {
        match &args[1] {
            XdlValue::String(s) => s.as_str(),
            _ => {
                return Err(XdlError::TypeMismatch {
                    expected: "string".to_string(),
                    actual: format!("{:?}", args[1].gdl_type()),
                })
            }
        }
    } else {
        ""
    };

    // Extract strings from the input array
    let strings: Vec<String> = match &args[0] {
        XdlValue::String(s) => vec![s.clone()],
        XdlValue::NestedArray(arr) => {
            let mut result = Vec::new();
            for val in arr {
                match val {
                    XdlValue::String(s) => result.push(s.clone()),
                    other => result.push(other.to_string_repr()),
                }
            }
            result
        }
        XdlValue::Array(arr) => arr.iter().map(|v| v.to_string()).collect(),
        other => vec![other.to_string_repr()],
    };

    Ok(XdlValue::String(strings.join(delimiter)))
}

/// STRSPLIT - Split string by delimiter
/// Syntax: result = STRSPLIT(string, pattern [, /REGEX] [, /EXTRACT])
pub fn strsplit(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(format!(
            "STRSPLIT: Expected at least 2 arguments, got {}",
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

    let pattern = match &args[1] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    // Split the string by the delimiter
    let parts: Vec<XdlValue> = s
        .split(pattern.as_str())
        .map(|part| XdlValue::String(part.to_string()))
        .collect();

    Ok(XdlValue::NestedArray(parts))
}

/// STRCOMPRESS - Remove or compress whitespace
/// Syntax: result = STRCOMPRESS(string [, /REMOVE_ALL])
pub fn strcompress(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "STRCOMPRESS: Expected at least 1 argument, got 0".to_string(),
        ));
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

    // Check for /REMOVE_ALL flag (second argument as non-zero)
    let remove_all = if args.len() > 1 {
        match &args[1] {
            XdlValue::Long(n) => *n != 0,
            XdlValue::Int(n) => *n != 0,
            XdlValue::Byte(n) => *n != 0,
            _ => false,
        }
    } else {
        false
    };

    let result = if remove_all {
        // Remove all whitespace
        s.chars().filter(|c| !c.is_whitespace()).collect()
    } else {
        // Compress multiple whitespace to single space and trim
        let compressed: String = s.split_whitespace().collect::<Vec<&str>>().join(" ");
        compressed
    };

    Ok(XdlValue::String(result))
}

/// STRCMP - Compare strings
/// Syntax: result = STRCMP(string1, string2 [, n] [, /FOLD_CASE])
/// Returns 1 if equal, 0 if not equal (IDL convention)
pub fn strcmp(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(format!(
            "STRCMP: Expected at least 2 arguments, got {}",
            args.len()
        )));
    }

    let s1 = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let s2 = match &args[1] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    // Optional: compare only first n characters
    let (cmp_s1, cmp_s2) = if args.len() > 2 {
        let n = match &args[2] {
            XdlValue::Long(n) => *n as usize,
            XdlValue::Int(n) => *n as usize,
            _ => s1.len().max(s2.len()),
        };
        (
            s1.chars().take(n).collect::<String>(),
            s2.chars().take(n).collect::<String>(),
        )
    } else {
        (s1.clone(), s2.clone())
    };

    // Check for /FOLD_CASE flag (case-insensitive comparison)
    let fold_case = if args.len() > 3 {
        match &args[3] {
            XdlValue::Long(n) => *n != 0,
            XdlValue::Int(n) => *n != 0,
            XdlValue::Byte(n) => *n != 0,
            _ => false,
        }
    } else {
        false
    };

    let equal = if fold_case {
        cmp_s1.to_lowercase() == cmp_s2.to_lowercase()
    } else {
        cmp_s1 == cmp_s2
    };

    // IDL convention: 1 if equal, 0 if not equal
    Ok(XdlValue::Long(if equal { 1 } else { 0 }))
}

/// STREGEX - Regular expression matching
/// Syntax: result = STREGEX(string, pattern [, /BOOLEAN] [, /EXTRACT])
pub fn stregex(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(format!(
            "STREGEX: Expected at least 2 arguments, got {}",
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

    let pattern = match &args[1] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    // Try to compile the regex
    let re = match regex::Regex::new(pattern) {
        Ok(r) => r,
        Err(e) => {
            return Err(XdlError::InvalidArgument(format!(
                "STREGEX: Invalid regex pattern: {}",
                e
            )))
        }
    };

    // Check for /BOOLEAN flag (just return 0 or 1)
    let boolean_mode = if args.len() > 2 {
        match &args[2] {
            XdlValue::Long(n) => *n != 0,
            XdlValue::Int(n) => *n != 0,
            XdlValue::Byte(n) => *n != 0,
            _ => false,
        }
    } else {
        false
    };

    if boolean_mode {
        // Return 1 if match, 0 if no match
        Ok(XdlValue::Long(if re.is_match(s) { 1 } else { 0 }))
    } else {
        // Return position of match, or -1 if not found
        match re.find(s) {
            Some(m) => Ok(XdlValue::Long(m.start() as i32)),
            None => Ok(XdlValue::Long(-1)),
        }
    }
}

/// STRREPLACE - Replace occurrences in string
/// Syntax: result = STRREPLACE(string, pattern, replacement)
/// Replaces all occurrences by default (IDL convention)
pub fn strreplace(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(format!(
            "STRREPLACE: Expected at least 3 arguments, got {}",
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

    let pattern = match &args[1] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let replacement = match &args[2] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[2].gdl_type()),
            })
        }
    };

    // Replace all occurrences by default (IDL convention)
    let result = s.replace(pattern.as_str(), replacement.as_str());

    Ok(XdlValue::String(result))
}

/// READS - Read values from a string
/// IDL syntax: READS, string_expression, variable [, variable, ...]
/// This function parses whitespace-separated values from a string
pub fn reads(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "READS: Expected at least 1 argument (string to parse)".to_string(),
        ));
    }

    let input = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    // Split on whitespace and parse values
    let parts: Vec<&str> = input.split_whitespace().collect();

    // If there are additional arguments, they indicate the expected types
    // For now, return an array of parsed values
    let mut values: Vec<f64> = Vec::new();
    for part in parts {
        if let Ok(val) = part.parse::<f64>() {
            values.push(val);
        } else if let Ok(val) = part.parse::<i64>() {
            values.push(val as f64);
        }
        // Skip non-numeric parts for now
    }

    // Return as array
    Ok(XdlValue::Array(values))
}

/// READS_STRING - Read a single value from a string as a string
/// Returns parsed parts as an array of strings
pub fn reads_string(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "READS_STRING: Expected at least 1 argument".to_string(),
        ));
    }

    let input = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    // Split on whitespace
    let parts: Vec<XdlValue> = input
        .split_whitespace()
        .map(|s| XdlValue::String(s.to_string()))
        .collect();

    Ok(XdlValue::NestedArray(parts))
}
