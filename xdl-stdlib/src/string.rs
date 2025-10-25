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

/// STRTRIM - Remove leading and/or trailing whitespace from string
/// Syntax: result = STRTRIM(string [, flag])
/// flag: 0=both (default), 1=leading only, 2=trailing only
pub fn strtrim(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() || args.len() > 2 {
        return Err(XdlError::InvalidArgument(
            "STRTRIM: Expected 1 or 2 arguments".to_string(),
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

    // Get flag (0=both, 1=leading, 2=trailing)
    let flag = if args.len() == 2 {
        match &args[1] {
            XdlValue::Long(n) => *n,
            XdlValue::Int(n) => *n as i32,
            XdlValue::Byte(n) => *n as i32,
            _ => {
                return Err(XdlError::TypeMismatch {
                    expected: "integer".to_string(),
                    actual: format!("{:?}", args[1].gdl_type()),
                })
            }
        }
    } else {
        0 // Default: trim both
    };

    let result = match flag {
        1 => s.trim_start().to_string(), // Leading only
        2 => s.trim_end().to_string(),   // Trailing only
        _ => s.trim().to_string(),       // Both (default for 0 or any other value)
    };

    Ok(XdlValue::String(result))
}

/// STRJOIN - Join array of strings with delimiter
/// Syntax: result = STRJOIN(string_array [, delimiter])
pub fn strjoin(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() || args.len() > 2 {
        return Err(XdlError::InvalidArgument(
            "STRJOIN: Expected 1 or 2 arguments".to_string(),
        ));
    }

    // Get array of strings
    let strings = match &args[0] {
        XdlValue::Array(arr) => {
            // Convert array elements to strings
            arr.iter().map(|v| format!("{}", v)).collect::<Vec<_>>()
        }
        XdlValue::NestedArray(nested) => {
            // Convert nested array elements to strings
            nested
                .iter()
                .map(|v| match v {
                    XdlValue::String(s) => s.clone(),
                    XdlValue::Int(n) => n.to_string(),
                    XdlValue::Long(n) => n.to_string(),
                    XdlValue::Float(f) => f.to_string(),
                    XdlValue::Double(d) => d.to_string(),
                    XdlValue::Byte(b) => b.to_string(),
                    _ => format!("{:?}", v),
                })
                .collect::<Vec<_>>()
        }
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    // Get delimiter (default is empty string)
    let delimiter = if args.len() == 2 {
        match &args[1] {
            XdlValue::String(s) => s.clone(),
            _ => {
                return Err(XdlError::TypeMismatch {
                    expected: "string".to_string(),
                    actual: format!("{:?}", args[1].gdl_type()),
                })
            }
        }
    } else {
        String::new()
    };

    Ok(XdlValue::String(strings.join(&delimiter)))
}

/// STRSPLIT - Split string by delimiter
/// Syntax: result = STRSPLIT(string, pattern [, /EXTRACT])
/// For now, simple implementation without regex
pub fn strsplit(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "STRSPLIT: Expected at least 2 arguments".to_string(),
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

    let delimiter = match &args[1] {
        XdlValue::String(d) => d,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    // Split the string
    let parts: Vec<XdlValue> = s
        .split(delimiter.as_str())
        .map(|part| XdlValue::String(part.to_string()))
        .collect();

    // Return as nested array (array of strings)
    Ok(XdlValue::NestedArray(parts))
}

/// STRCMP - Compare two strings
/// Syntax: result = STRCMP(string1, string2 [, /FOLD_CASE])
/// Returns 1 if equal, 0 if not
pub fn strcmp(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "STRCMP: Expected at least 2 arguments".to_string(),
        ));
    }

    let s1 = match &args[0] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let s2 = match &args[1] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    // Case-sensitive comparison
    let result = if s1 == s2 { 1 } else { 0 };
    Ok(XdlValue::Long(result))
}

/// STRCOMPRESS - Remove excess whitespace
/// Syntax: result = STRCOMPRESS(string [, /REMOVE_ALL])
pub fn strcompress(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "STRCOMPRESS: Expected at least 1 argument".to_string(),
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

    // Compress multiple spaces to single space and trim
    let result = s.split_whitespace().collect::<Vec<_>>().join(" ");

    Ok(XdlValue::String(result))
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
        XdlValue::Undefined => "!NULL".to_string(),
    };

    Ok(XdlValue::String(result))
}

/// STRMATCH - Pattern matching with wildcards
/// STRMATCH(string, pattern [, /FOLD_CASE])
/// Supports * (any chars) and ? (single char) wildcards
/// Returns 1 if match, 0 if no match
/// For array of strings, returns array of match results
pub fn strmatch(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "STRMATCH: Expected at least 2 arguments (string, pattern)".to_string(),
        ));
    }

    let pattern = match &args[1] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    // Simple wildcard matching implementation
    // * matches any sequence of characters
    // ? matches any single character
    let matches_pattern = |s: &str, pat: &str| -> bool { wildcard_match(s, pat) };

    match &args[0] {
        XdlValue::String(s) => {
            let result = if matches_pattern(s, pattern) { 1 } else { 0 };
            Ok(XdlValue::Long(result))
        }
        XdlValue::NestedArray(arr) => {
            // For array of strings, return array of results
            let results: Vec<f64> = arr
                .iter()
                .map(|v| match v {
                    XdlValue::String(s) => {
                        if matches_pattern(s, pattern) {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    _ => 0.0,
                })
                .collect();
            Ok(XdlValue::Array(results))
        }
        _ => Err(XdlError::TypeMismatch {
            expected: "string or string array".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// Helper function for wildcard pattern matching
/// Supports * (any chars) and ? (single char)
fn wildcard_match(text: &str, pattern: &str) -> bool {
    let text_chars: Vec<char> = text.chars().collect();
    let pat_chars: Vec<char> = pattern.chars().collect();

    wildcard_match_recursive(&text_chars, &pat_chars, 0, 0)
}

fn wildcard_match_recursive(
    text: &[char],
    pattern: &[char],
    text_idx: usize,
    pat_idx: usize,
) -> bool {
    // If we've matched the entire pattern and text
    if pat_idx == pattern.len() && text_idx == text.len() {
        return true;
    }

    // If pattern is exhausted but text remains
    if pat_idx == pattern.len() {
        return false;
    }

    // If text is exhausted but pattern has only '*' remaining
    if text_idx == text.len() {
        return pattern[pat_idx..].iter().all(|&c| c == '*');
    }

    match pattern[pat_idx] {
        '*' => {
            // Try matching zero or more characters
            // First, try matching zero characters (skip the *)
            if wildcard_match_recursive(text, pattern, text_idx, pat_idx + 1) {
                return true;
            }
            // Try matching one more character and continue with *
            wildcard_match_recursive(text, pattern, text_idx + 1, pat_idx)
        }
        '?' => {
            // Match any single character
            wildcard_match_recursive(text, pattern, text_idx + 1, pat_idx + 1)
        }
        c => {
            // Exact character match
            if text[text_idx] == c {
                wildcard_match_recursive(text, pattern, text_idx + 1, pat_idx + 1)
            } else {
                false
            }
        }
    }
}

/// STRREPLACE - Replace all occurrences of search string with replacement
/// STRREPLACE(string, search, replace)
pub fn strreplace(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 3 {
        return Err(XdlError::InvalidArgument(format!(
            "STRREPLACE: Expected 3 arguments (string, search, replace), got {}",
            args.len()
        )));
    }

    let string = match &args[0] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let search = match &args[1] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let replace = match &args[2] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[2].gdl_type()),
            })
        }
    };

    let result = string.replace(search, replace);
    Ok(XdlValue::String(result))
}

/// STRPUT - Insert/overlay string at position
/// STRPUT(destination, insert, position)
pub fn strput(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 3 {
        return Err(XdlError::InvalidArgument(format!(
            "STRPUT: Expected 3 arguments (destination, insert, position), got {}",
            args.len()
        )));
    }

    let dest = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let insert = match &args[1] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let position = match &args[2] {
        XdlValue::Long(n) => *n as usize,
        XdlValue::Int(n) => *n as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[2].gdl_type()),
            })
        }
    };

    let mut chars: Vec<char> = dest.chars().collect();

    // Extend string if needed
    while chars.len() < position {
        chars.push(' ');
    }

    // Overlay/insert the new string
    let insert_chars: Vec<char> = insert.chars().collect();
    for (i, &ch) in insert_chars.iter().enumerate() {
        let idx = position + i;
        if idx < chars.len() {
            chars[idx] = ch;
        } else {
            chars.push(ch);
        }
    }

    let result: String = chars.iter().collect();
    Ok(XdlValue::String(result))
}

/// STRMESSAGE - Return error message text for error code
/// STRMESSAGE(error_code)
pub fn strmessage(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "STRMESSAGE: Expected error code argument".to_string(),
        ));
    }

    let code = match &args[0] {
        XdlValue::Long(n) => *n,
        XdlValue::Int(n) => *n as i32,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    // Simple implementation - map common error codes to messages
    let message = match code {
        0 => "Success",
        -1 => "General error",
        -2 => "File not found",
        -3 => "Invalid argument",
        -4 => "Type mismatch",
        -5 => "Dimension error",
        -6 => "Math error",
        -7 => "Runtime error",
        _ => "Unknown error",
    };

    Ok(XdlValue::String(message.to_string()))
}

/// FORMAT_AXIS_VALUES - Format numeric values for axis labels (placeholder)
/// Simplified version for now
pub fn format_axis_values(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "FORMAT_AXIS_VALUES: Expected value argument".to_string(),
        ));
    }

    match &args[0] {
        XdlValue::Double(v) => Ok(XdlValue::String(format!("{:.2}", v))),
        XdlValue::Float(v) => Ok(XdlValue::String(format!("{:.2}", v))),
        XdlValue::Long(v) => Ok(XdlValue::String(format!("{}", v))),
        XdlValue::Int(v) => Ok(XdlValue::String(format!("{}", v))),
        XdlValue::Array(arr) => {
            let formatted: Vec<XdlValue> = arr
                .iter()
                .map(|&x| XdlValue::String(format!("{:.2}", x)))
                .collect();
            Ok(XdlValue::NestedArray(formatted))
        }
        _ => Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}
