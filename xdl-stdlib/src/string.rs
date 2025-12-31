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

/// SPRINTF - Format values into a string using format specifiers
/// Syntax: SPRINTF(format, value1, value2, ...)
/// Supports: %d (integer), %f (float), %e (scientific), %s (string), %x (hex), %o (octal), %b (binary)
pub fn sprintf(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "SPRINTF: Expected format string and values".to_string(),
        ));
    }

    let format_str = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let values = &args[1..];
    let mut result = String::new();
    let mut value_idx = 0;
    let mut chars = format_str.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            if let Some(&next) = chars.peek() {
                if next == '%' {
                    // Escaped percent
                    chars.next();
                    result.push('%');
                    continue;
                }

                // Parse format specifier
                let mut width = String::new();
                let mut precision = String::new();
                let mut in_precision = false;

                // Parse width and precision
                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_digit() || ch == '.' || ch == '-' || ch == '+' {
                        chars.next();
                        if ch == '.' {
                            in_precision = true;
                        } else if in_precision {
                            precision.push(ch);
                        } else {
                            width.push(ch);
                        }
                    } else {
                        break;
                    }
                }

                // Get format character
                if let Some(fmt_char) = chars.next() {
                    if value_idx < values.len() {
                        let formatted = format_value(&values[value_idx], fmt_char, &width, &precision);
                        result.push_str(&formatted);
                        value_idx += 1;
                    } else {
                        // Not enough values, output format specifier as-is
                        result.push('%');
                        result.push_str(&width);
                        if in_precision {
                            result.push('.');
                            result.push_str(&precision);
                        }
                        result.push(fmt_char);
                    }
                }
            } else {
                result.push('%');
            }
        } else {
            result.push(c);
        }
    }

    Ok(XdlValue::String(result))
}

/// Helper function to format a single value according to format specifier
fn format_value(value: &XdlValue, fmt_char: char, width: &str, precision: &str) -> String {
    let width_val: usize = width.replace('-', "").replace('+', "").parse().unwrap_or(0);
    let precision_val: usize = precision.parse().unwrap_or(6);
    let left_align = width.starts_with('-');

    match fmt_char {
        'd' | 'i' => {
            // Integer
            let int_val = match value {
                XdlValue::Int(i) => *i as i64,
                XdlValue::Long(l) => *l as i64,
                XdlValue::Long64(l) => *l,
                XdlValue::Float(f) => *f as i64,
                XdlValue::Double(d) => *d as i64,
                XdlValue::Byte(b) => *b as i64,
                XdlValue::UInt(u) => *u as i64,
                XdlValue::ULong(u) => *u as i64,
                XdlValue::ULong64(u) => *u as i64,
                _ => 0,
            };
            if width_val > 0 {
                if left_align {
                    format!("{:<width$}", int_val, width = width_val)
                } else {
                    format!("{:>width$}", int_val, width = width_val)
                }
            } else {
                format!("{}", int_val)
            }
        }
        'f' | 'F' => {
            // Floating point
            let float_val = match value {
                XdlValue::Float(f) => *f as f64,
                XdlValue::Double(d) => *d,
                XdlValue::Int(i) => *i as f64,
                XdlValue::Long(l) => *l as f64,
                XdlValue::Long64(l) => *l as f64,
                _ => 0.0,
            };
            if width_val > 0 {
                if left_align {
                    format!("{:<width$.prec$}", float_val, width = width_val, prec = precision_val)
                } else {
                    format!("{:>width$.prec$}", float_val, width = width_val, prec = precision_val)
                }
            } else {
                format!("{:.prec$}", float_val, prec = precision_val)
            }
        }
        'e' | 'E' => {
            // Scientific notation
            let float_val = match value {
                XdlValue::Float(f) => *f as f64,
                XdlValue::Double(d) => *d,
                XdlValue::Int(i) => *i as f64,
                XdlValue::Long(l) => *l as f64,
                _ => 0.0,
            };
            if fmt_char == 'E' {
                format!("{:.prec$E}", float_val, prec = precision_val)
            } else {
                format!("{:.prec$e}", float_val, prec = precision_val)
            }
        }
        'g' | 'G' => {
            // Compact format (either f or e)
            let float_val = match value {
                XdlValue::Float(f) => *f as f64,
                XdlValue::Double(d) => *d,
                XdlValue::Int(i) => *i as f64,
                XdlValue::Long(l) => *l as f64,
                _ => 0.0,
            };
            let abs_val = float_val.abs();
            if abs_val < 1e-4 || abs_val >= 1e6 {
                if fmt_char == 'G' {
                    format!("{:.prec$E}", float_val, prec = precision_val)
                } else {
                    format!("{:.prec$e}", float_val, prec = precision_val)
                }
            } else {
                format!("{:.prec$}", float_val, prec = precision_val)
            }
        }
        's' => {
            // String
            let str_val = match value {
                XdlValue::String(s) => s.clone(),
                _ => format!("{:?}", value),
            };
            if width_val > 0 {
                if left_align {
                    format!("{:<width$}", str_val, width = width_val)
                } else {
                    format!("{:>width$}", str_val, width = width_val)
                }
            } else {
                str_val
            }
        }
        'x' => {
            // Hexadecimal lowercase
            let int_val = match value {
                XdlValue::Int(i) => *i as u64,
                XdlValue::Long(l) => *l as u64,
                XdlValue::Long64(l) => *l as u64,
                XdlValue::ULong64(u) => *u,
                XdlValue::Byte(b) => *b as u64,
                _ => 0,
            };
            format!("{:x}", int_val)
        }
        'X' => {
            // Hexadecimal uppercase
            let int_val = match value {
                XdlValue::Int(i) => *i as u64,
                XdlValue::Long(l) => *l as u64,
                XdlValue::Long64(l) => *l as u64,
                XdlValue::ULong64(u) => *u,
                XdlValue::Byte(b) => *b as u64,
                _ => 0,
            };
            format!("{:X}", int_val)
        }
        'o' => {
            // Octal
            let int_val = match value {
                XdlValue::Int(i) => *i as u64,
                XdlValue::Long(l) => *l as u64,
                XdlValue::Long64(l) => *l as u64,
                XdlValue::ULong64(u) => *u,
                XdlValue::Byte(b) => *b as u64,
                _ => 0,
            };
            format!("{:o}", int_val)
        }
        'b' => {
            // Binary
            let int_val = match value {
                XdlValue::Int(i) => *i as u64,
                XdlValue::Long(l) => *l as u64,
                XdlValue::Long64(l) => *l as u64,
                XdlValue::ULong64(u) => *u,
                XdlValue::Byte(b) => *b as u64,
                _ => 0,
            };
            format!("{:b}", int_val)
        }
        'c' => {
            // Character
            let char_val = match value {
                XdlValue::Int(i) => char::from_u32(*i as u32).unwrap_or('?'),
                XdlValue::Byte(b) => *b as char,
                XdlValue::String(s) => s.chars().next().unwrap_or('?'),
                _ => '?',
            };
            char_val.to_string()
        }
        _ => {
            // Unknown format, just return the value as string
            format!("{:?}", value)
        }
    }
}

/// STRTOK - Tokenize a string using delimiters
/// Syntax: STRTOK(string, delimiters, [/EXTRACT], [/PRESERVE_NULL])
pub fn strtok(
    args: &[XdlValue],
    keywords: &std::collections::HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "STRTOK: Expected string argument".to_string(),
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

    // Get delimiter (default is whitespace)
    let delimiters = if args.len() > 1 {
        match &args[1] {
            XdlValue::String(s) => s.clone(),
            _ => " \t\n".to_string(),
        }
    } else {
        " \t\n".to_string()
    };

    let preserve_null = keywords.contains_key("PRESERVE_NULL");
    let extract = keywords.contains_key("EXTRACT");

    // Tokenize
    let tokens: Vec<XdlValue> = if preserve_null {
        // Keep empty tokens
        input
            .split(|c| delimiters.contains(c))
            .map(|s| XdlValue::String(s.to_string()))
            .collect()
    } else {
        // Skip empty tokens
        input
            .split(|c| delimiters.contains(c))
            .filter(|s| !s.is_empty())
            .map(|s| XdlValue::String(s.to_string()))
            .collect()
    };

    if extract && !tokens.is_empty() {
        // Return first token only
        Ok(tokens.into_iter().next().unwrap_or(XdlValue::String(String::new())))
    } else {
        Ok(XdlValue::NestedArray(tokens))
    }
}

/// STRPUT - Insert a substring into a string at a position
/// Syntax: STRPUT, destination, source, position
pub fn strput(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(
            "STRPUT: Expected destination, source, position".to_string(),
        ));
    }

    let mut dest = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let source = match &args[1] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let position = match &args[2] {
        XdlValue::Int(i) => *i as usize,
        XdlValue::Long(l) => *l as usize,
        XdlValue::Float(f) => *f as usize,
        XdlValue::Double(d) => *d as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[2].gdl_type()),
            })
        }
    };

    // Replace characters at position
    if position < dest.len() {
        let end_pos = (position + source.len()).min(dest.len());
        let new_str = format!(
            "{}{}{}",
            &dest[..position],
            &source[..(end_pos - position).min(source.len())],
            if end_pos < dest.len() { &dest[end_pos..] } else { "" }
        );
        dest = new_str;
    }

    Ok(XdlValue::String(dest))
}

/// STRMID_BYTES - Extract substring by byte position (for multi-byte strings)
pub fn strmid_bytes(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "STRMID_BYTES: Expected string and position".to_string(),
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

    let start = match &args[1] {
        XdlValue::Int(i) => *i as usize,
        XdlValue::Long(l) => *l as usize,
        _ => 0,
    };

    let length = if args.len() > 2 {
        match &args[2] {
            XdlValue::Int(i) => Some(*i as usize),
            XdlValue::Long(l) => Some(*l as usize),
            _ => None,
        }
    } else {
        None
    };

    let bytes = input.as_bytes();
    let end = match length {
        Some(len) => (start + len).min(bytes.len()),
        None => bytes.len(),
    };

    if start >= bytes.len() {
        return Ok(XdlValue::String(String::new()));
    }

    let result = String::from_utf8_lossy(&bytes[start..end]).to_string();
    Ok(XdlValue::String(result))
}

/// BYTE - Convert string to byte array or value to byte
pub fn str_to_byte(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "BYTE: Expected argument".to_string(),
        ));
    }

    match &args[0] {
        XdlValue::String(s) => {
            let bytes: Vec<f64> = s.bytes().map(|b| b as f64).collect();
            Ok(XdlValue::Array(bytes))
        }
        XdlValue::Int(i) => Ok(XdlValue::Byte(*i as u8)),
        XdlValue::Long(l) => Ok(XdlValue::Byte(*l as u8)),
        XdlValue::Float(f) => Ok(XdlValue::Byte(*f as u8)),
        XdlValue::Double(d) => Ok(XdlValue::Byte(*d as u8)),
        XdlValue::Array(arr) => {
            let bytes: Vec<f64> = arr.iter().map(|&v| (v as u8) as f64).collect();
            Ok(XdlValue::Array(bytes))
        }
        _ => Err(XdlError::TypeMismatch {
            expected: "string or numeric".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// STRING_FROM_BYTES - Convert byte array back to string
pub fn string_from_bytes(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "STRING_FROM_BYTES: Expected byte array".to_string(),
        ));
    }

    let bytes: Vec<u8> = match &args[0] {
        XdlValue::Array(arr) => arr.iter().map(|&v| v as u8).collect(),
        XdlValue::MultiDimArray { data, .. } => data.iter().map(|&v| v as u8).collect(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let result = String::from_utf8_lossy(&bytes).to_string();
    Ok(XdlValue::String(result))
}

/// STRPOS_ALL - Find all occurrences of substring
pub fn strpos_all(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "STRPOS_ALL: Expected string and substring".to_string(),
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

    let search = match &args[1] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    if search.is_empty() {
        return Ok(XdlValue::Array(vec![]));
    }

    let positions: Vec<f64> = input
        .match_indices(&search)
        .map(|(i, _)| i as f64)
        .collect();

    Ok(XdlValue::Array(positions))
}

/// STRCOUNT - Count occurrences of substring
pub fn strcount(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "STRCOUNT: Expected string and substring".to_string(),
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

    let search = match &args[1] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    if search.is_empty() {
        return Ok(XdlValue::Long(0));
    }

    let count = input.matches(&search).count();
    Ok(XdlValue::Long(count as i32))
}

/// STRPAD - Pad string to specified length
pub fn strpad(
    args: &[XdlValue],
    keywords: &std::collections::HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "STRPAD: Expected string and length".to_string(),
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

    let length = match &args[1] {
        XdlValue::Int(i) => *i as usize,
        XdlValue::Long(l) => *l as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let pad_char = keywords
        .get("PAD")
        .and_then(|v| match v {
            XdlValue::String(s) => s.chars().next(),
            _ => None,
        })
        .unwrap_or(' ');

    let left = keywords.contains_key("LEFT");

    let result = if input.len() >= length {
        input[..length].to_string()
    } else if left {
        format!("{:>width$}", input, width = length).replace(' ', &pad_char.to_string())
    } else {
        format!("{:<width$}", input, width = length).replace(' ', &pad_char.to_string())
    };

    Ok(XdlValue::String(result))
}

/// STRREVERSE - Reverse a string
pub fn strreverse(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "STRREVERSE: Expected string argument".to_string(),
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

    let reversed: String = input.chars().rev().collect();
    Ok(XdlValue::String(reversed))
}
