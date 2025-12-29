//! Type-specific method dispatch for OOP syntax support
//!
//! This module provides method dispatch for built-in types like Array, String,
//! MultiDimArray, and NestedArray, allowing OOP-style syntax like:
//! - `arr->Sum()`, `arr->Mean()`, `arr->Sort()`
//! - `str->ToUpper()`, `str->Length()`, `str->Contains("substr")`

use xdl_core::{XdlError, XdlResult, XdlValue};
use xdl_stdlib::{array, statistics, string};

/// Dispatch methods on Array values (1D arrays)
pub fn call_array_method(arr: &[f64], method: &str, args: &[XdlValue]) -> XdlResult<XdlValue> {
    let arr_val = XdlValue::Array(arr.to_vec());

    match method.to_uppercase().as_str() {
        // === Aggregation methods ===
        "SUM" | "TOTAL" => array::total_func(&[arr_val]),
        "MEAN" | "AVG" | "AVERAGE" => array::mean_func(&[arr_val]),
        "MIN" | "MINIMUM" => array::min_func(&[arr_val]),
        "MAX" | "MAXIMUM" => array::max_func(&[arr_val]),

        // === Statistics ===
        "VARIANCE" | "VAR" => statistics::variance(&[arr_val]),
        "STDDEV" | "STD" => statistics::stddev(&[arr_val]),
        "MEDIAN" => statistics::median(&[arr_val]),
        "SKEWNESS" => statistics::skewness(&[arr_val]),
        "KURTOSIS" => statistics::kurtosis(&[arr_val]),
        "MOMENT" => statistics::moment(&[arr_val]),
        "MEANABSDEV" => statistics::meanabsdev(&[arr_val]),

        // === Array operations ===
        "SORT" | "SORTED" => array::sort_func(&[arr_val]),
        "REVERSE" | "REVERSED" => array::reverse_func(&[arr_val]),
        "UNIQUE" | "UNIQ" => array::uniq_func(&[arr_val]),

        // === Information ===
        "LENGTH" | "LEN" | "COUNT" | "SIZE" => Ok(XdlValue::Long(arr.len() as i32)),
        "N_ELEMENTS" => array::n_elements(&[arr_val]),

        // === Operations with arguments ===
        "WHERE" => {
            // arr->Where() returns indices where non-zero
            array::where_func(&[arr_val])
        }
        "SMOOTH" => {
            // arr->Smooth(window_size)
            let mut all_args = vec![arr_val];
            all_args.extend_from_slice(args);
            array::smooth_func(&all_args)
        }
        "SHIFT" => {
            // arr->Shift(offset)
            let mut all_args = vec![arr_val];
            all_args.extend_from_slice(args);
            array::shift_func(&all_args)
        }
        "HISTOGRAM" => {
            // arr->Histogram(nbins)
            let mut all_args = vec![arr_val];
            all_args.extend_from_slice(args);
            array::histogram_func(&all_args)
        }
        "REBIN" => {
            // arr->Rebin(new_size)
            let mut all_args = vec![arr_val];
            all_args.extend_from_slice(args);
            array::rebin_func(&all_args)
        }
        "CONGRID" => {
            // arr->Congrid(new_size)
            let mut all_args = vec![arr_val];
            all_args.extend_from_slice(args);
            array::congrid_func(&all_args)
        }

        _ => Err(XdlError::NotImplemented(format!(
            "Array method '{}'. Available: Sum, Mean, Min, Max, Sort, Reverse, \
             Unique, Length, Variance, Stddev, Median, Skewness, Kurtosis, \
             Where, Smooth, Shift, Histogram, Rebin, Congrid",
            method
        ))),
    }
}

/// Dispatch methods on String values
pub fn call_string_method(s: &str, method: &str, args: &[XdlValue]) -> XdlResult<XdlValue> {
    let str_val = XdlValue::String(s.to_string());

    match method.to_uppercase().as_str() {
        // === Case conversion ===
        "TOUPPER" | "UPPER" | "UPPERCASE" | "UPCASE" => string::strupcase(&[str_val]),
        "TOLOWER" | "LOWER" | "LOWERCASE" | "LOWCASE" => string::strlowcase(&[str_val]),

        // === Information ===
        "LENGTH" | "LEN" => string::strlen(&[str_val]),

        // === Trimming ===
        "TRIM" | "STRIP" => string::strtrim(&[str_val, XdlValue::Long(2)]), // Both ends
        "LTRIM" | "TRIMLEFT" | "LSTRIP" => string::strtrim(&[str_val, XdlValue::Long(1)]),
        "RTRIM" | "TRIMRIGHT" | "RSTRIP" => string::strtrim(&[str_val, XdlValue::Long(0)]),
        "COMPRESS" => string::strcompress(&[str_val]),

        // === Search ===
        "CONTAINS" => {
            if args.is_empty() {
                return Err(XdlError::InvalidArgument(
                    "Contains() requires a substring argument".to_string(),
                ));
            }
            let pos = string::strpos(&[str_val, args[0].clone()])?;
            match pos {
                XdlValue::Long(n) => Ok(XdlValue::Long(if n >= 0 { 1 } else { 0 })),
                _ => Ok(XdlValue::Long(0)),
            }
        }
        "INDEXOF" | "FIND" | "POS" => {
            if args.is_empty() {
                return Err(XdlError::InvalidArgument(
                    "IndexOf() requires a substring argument".to_string(),
                ));
            }
            string::strpos(&[str_val, args[0].clone()])
        }
        "STARTSWITH" => {
            if args.is_empty() {
                return Err(XdlError::InvalidArgument(
                    "StartsWith() requires a prefix argument".to_string(),
                ));
            }
            let prefix = match &args[0] {
                XdlValue::String(p) => p.as_str(),
                _ => return Ok(XdlValue::Long(0)),
            };
            Ok(XdlValue::Long(if s.starts_with(prefix) { 1 } else { 0 }))
        }
        "ENDSWITH" => {
            if args.is_empty() {
                return Err(XdlError::InvalidArgument(
                    "EndsWith() requires a suffix argument".to_string(),
                ));
            }
            let suffix = match &args[0] {
                XdlValue::String(p) => p.as_str(),
                _ => return Ok(XdlValue::Long(0)),
            };
            Ok(XdlValue::Long(if s.ends_with(suffix) { 1 } else { 0 }))
        }

        // === Splitting/Joining ===
        "SPLIT" => {
            let delim = if args.is_empty() {
                XdlValue::String(" ".to_string())
            } else {
                args[0].clone()
            };
            string::strsplit(&[str_val, delim])
        }

        // === Substring ===
        "SUBSTRING" | "SUBSTR" | "MID" => {
            if args.is_empty() {
                return Err(XdlError::InvalidArgument(
                    "Substring() requires start position".to_string(),
                ));
            }
            let mut all_args = vec![str_val];
            all_args.extend_from_slice(args);
            string::strmid(&all_args)
        }

        // === Replacement ===
        "REPLACE" => {
            if args.len() < 2 {
                return Err(XdlError::InvalidArgument(
                    "Replace() requires pattern and replacement arguments".to_string(),
                ));
            }
            string::strreplace(&[str_val, args[0].clone(), args[1].clone()])
        }

        // === Comparison ===
        "EQUALS" | "EQ" => {
            if args.is_empty() {
                return Err(XdlError::InvalidArgument(
                    "Equals() requires a string argument".to_string(),
                ));
            }
            string::strcmp(&[str_val, args[0].clone()])
        }

        // === Regex ===
        "MATCH" | "REGEX" => {
            if args.is_empty() {
                return Err(XdlError::InvalidArgument(
                    "Match() requires a pattern argument".to_string(),
                ));
            }
            string::stregex(&[str_val, args[0].clone()])
        }

        _ => Err(XdlError::NotImplemented(format!(
            "String method '{}'. Available: ToUpper, ToLower, Length, Trim, \
             LTrim, RTrim, Compress, Contains, IndexOf, StartsWith, EndsWith, \
             Split, Substring, Replace, Equals, Match",
            method
        ))),
    }
}

/// Dispatch methods on MultiDimArray values (N-dimensional arrays)
pub fn call_multidim_method(
    data: &[f64],
    shape: &[usize],
    method: &str,
    args: &[XdlValue],
) -> XdlResult<XdlValue> {
    let arr_val = XdlValue::MultiDimArray {
        data: data.to_vec(),
        shape: shape.to_vec(),
    };

    match method.to_uppercase().as_str() {
        // === Aggregation (operates on flattened data) ===
        "SUM" | "TOTAL" => {
            let sum: f64 = data.iter().sum();
            Ok(XdlValue::Double(sum))
        }
        "MEAN" | "AVG" | "AVERAGE" => {
            if data.is_empty() {
                Ok(XdlValue::Undefined)
            } else {
                let mean = data.iter().sum::<f64>() / data.len() as f64;
                Ok(XdlValue::Double(mean))
            }
        }
        "MIN" | "MINIMUM" => {
            if data.is_empty() {
                Ok(XdlValue::Undefined)
            } else {
                let min_val = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                Ok(XdlValue::Double(min_val))
            }
        }
        "MAX" | "MAXIMUM" => {
            if data.is_empty() {
                Ok(XdlValue::Undefined)
            } else {
                let max_val = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                Ok(XdlValue::Double(max_val))
            }
        }

        // === Statistics ===
        "VARIANCE" | "VAR" => statistics::variance(&[XdlValue::Array(data.to_vec())]),
        "STDDEV" | "STD" => statistics::stddev(&[XdlValue::Array(data.to_vec())]),
        "MEDIAN" => statistics::median(&[XdlValue::Array(data.to_vec())]),

        // === Shape information ===
        "SHAPE" | "DIMS" | "DIMENSIONS" => {
            Ok(XdlValue::Array(shape.iter().map(|&d| d as f64).collect()))
        }
        "NDIM" | "NDIMS" | "RANK" => Ok(XdlValue::Long(shape.len() as i32)),
        "LENGTH" | "SIZE" | "N_ELEMENTS" => Ok(XdlValue::Long(data.len() as i32)),

        // === Reshaping ===
        "FLATTEN" | "FLAT" | "RAVEL" => Ok(XdlValue::Array(data.to_vec())),
        "RESHAPE" | "REFORM" => {
            // arr->Reshape(new_dims...)
            if args.is_empty() {
                return Err(XdlError::InvalidArgument(
                    "Reshape() requires dimension arguments".to_string(),
                ));
            }
            let mut all_args = vec![arr_val];
            all_args.extend_from_slice(args);
            array::reform_func(&all_args)
        }
        "TRANSPOSE" => array::transpose_func(&[arr_val]),

        // === Sorting ===
        "SORT" | "SORTED" => {
            let mut sorted = data.to_vec();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            Ok(XdlValue::Array(sorted))
        }
        "REVERSE" | "REVERSED" => {
            let mut reversed = data.to_vec();
            reversed.reverse();
            Ok(XdlValue::Array(reversed))
        }

        _ => Err(XdlError::NotImplemented(format!(
            "MultiDimArray method '{}'. Available: Sum, Mean, Min, Max, \
             Variance, Stddev, Median, Shape, Ndim, Length, Flatten, \
             Reshape, Transpose, Sort, Reverse",
            method
        ))),
    }
}

/// Dispatch methods on NestedArray values (arrays of arrays)
pub fn call_nested_array_method(
    rows: &[XdlValue],
    method: &str,
    _args: &[XdlValue],
) -> XdlResult<XdlValue> {
    match method.to_uppercase().as_str() {
        // === Size information ===
        "LENGTH" | "SIZE" | "COUNT" => Ok(XdlValue::Long(rows.len() as i32)),
        "NROWS" | "ROWS" => Ok(XdlValue::Long(rows.len() as i32)),
        "NCOLS" | "COLS" => {
            // Assume all rows have same length, check first row
            if rows.is_empty() {
                Ok(XdlValue::Long(0))
            } else if let XdlValue::Array(first_row) = &rows[0] {
                Ok(XdlValue::Long(first_row.len() as i32))
            } else {
                Ok(XdlValue::Long(1))
            }
        }
        "SHAPE" | "DIMS" => {
            let nrows = rows.len();
            let ncols = if rows.is_empty() {
                0
            } else if let XdlValue::Array(first_row) = &rows[0] {
                first_row.len()
            } else {
                1
            };
            Ok(XdlValue::Array(vec![nrows as f64, ncols as f64]))
        }
        "NDIM" | "RANK" => Ok(XdlValue::Long(2)), // Nested arrays are 2D

        // === Flattening ===
        "FLATTEN" | "FLAT" | "RAVEL" => {
            // Flatten nested array to 1D
            let mut result = Vec::new();
            for row in rows {
                match row {
                    XdlValue::Array(arr) => result.extend(arr.iter().cloned()),
                    XdlValue::Double(d) => result.push(*d),
                    XdlValue::Float(f) => result.push(*f as f64),
                    XdlValue::Long(l) => result.push(*l as f64),
                    _ => {} // Skip non-numeric values
                }
            }
            Ok(XdlValue::Array(result))
        }

        // === Aggregation (on flattened data) ===
        "SUM" | "TOTAL" => {
            let mut sum = 0.0;
            for row in rows {
                if let XdlValue::Array(arr) = row {
                    sum += arr.iter().sum::<f64>();
                }
            }
            Ok(XdlValue::Double(sum))
        }
        "MEAN" | "AVG" => {
            let mut sum = 0.0;
            let mut count = 0;
            for row in rows {
                if let XdlValue::Array(arr) = row {
                    sum += arr.iter().sum::<f64>();
                    count += arr.len();
                }
            }
            if count == 0 {
                Ok(XdlValue::Undefined)
            } else {
                Ok(XdlValue::Double(sum / count as f64))
            }
        }
        "MIN" | "MINIMUM" => {
            let mut min_val = f64::INFINITY;
            for row in rows {
                if let XdlValue::Array(arr) = row {
                    for &v in arr {
                        if v < min_val {
                            min_val = v;
                        }
                    }
                }
            }
            if min_val == f64::INFINITY {
                Ok(XdlValue::Undefined)
            } else {
                Ok(XdlValue::Double(min_val))
            }
        }
        "MAX" | "MAXIMUM" => {
            let mut max_val = f64::NEG_INFINITY;
            for row in rows {
                if let XdlValue::Array(arr) = row {
                    for &v in arr {
                        if v > max_val {
                            max_val = v;
                        }
                    }
                }
            }
            if max_val == f64::NEG_INFINITY {
                Ok(XdlValue::Undefined)
            } else {
                Ok(XdlValue::Double(max_val))
            }
        }

        _ => Err(XdlError::NotImplemented(format!(
            "NestedArray method '{}'. Available: Length, NRows, NCols, Shape, \
             Ndim, Flatten, Sum, Mean, Min, Max",
            method
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array_sum() {
        let arr = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = call_array_method(&arr, "Sum", &[]).unwrap();
        assert_eq!(result, XdlValue::Double(15.0));
    }

    #[test]
    fn test_array_mean() {
        let arr = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = call_array_method(&arr, "Mean", &[]).unwrap();
        assert_eq!(result, XdlValue::Double(3.0));
    }

    #[test]
    fn test_array_length() {
        let arr = vec![1.0, 2.0, 3.0];
        let result = call_array_method(&arr, "Length", &[]).unwrap();
        assert_eq!(result, XdlValue::Long(3));
    }

    #[test]
    fn test_array_min_max() {
        let arr = vec![3.0, 1.0, 4.0, 1.0, 5.0];
        let min_result = call_array_method(&arr, "Min", &[]).unwrap();
        let max_result = call_array_method(&arr, "Max", &[]).unwrap();
        assert_eq!(min_result, XdlValue::Double(1.0));
        assert_eq!(max_result, XdlValue::Double(5.0));
    }

    #[test]
    fn test_string_toupper() {
        let result = call_string_method("hello", "ToUpper", &[]).unwrap();
        assert_eq!(result, XdlValue::String("HELLO".to_string()));
    }

    #[test]
    fn test_string_tolower() {
        let result = call_string_method("HELLO", "ToLower", &[]).unwrap();
        assert_eq!(result, XdlValue::String("hello".to_string()));
    }

    #[test]
    fn test_string_length() {
        let result = call_string_method("hello", "Length", &[]).unwrap();
        assert_eq!(result, XdlValue::Long(5));
    }

    #[test]
    fn test_string_contains() {
        let result = call_string_method(
            "hello world",
            "Contains",
            &[XdlValue::String("world".to_string())],
        )
        .unwrap();
        assert_eq!(result, XdlValue::Long(1));

        let result_not_found = call_string_method(
            "hello world",
            "Contains",
            &[XdlValue::String("xyz".to_string())],
        )
        .unwrap();
        assert_eq!(result_not_found, XdlValue::Long(0));
    }

    #[test]
    fn test_string_indexof() {
        let result = call_string_method(
            "hello world",
            "IndexOf",
            &[XdlValue::String("world".to_string())],
        )
        .unwrap();
        assert_eq!(result, XdlValue::Long(6));
    }

    #[test]
    fn test_string_startswith() {
        let result = call_string_method(
            "hello world",
            "StartsWith",
            &[XdlValue::String("hello".to_string())],
        )
        .unwrap();
        assert_eq!(result, XdlValue::Long(1));
    }

    #[test]
    fn test_string_endswith() {
        let result = call_string_method(
            "hello world",
            "EndsWith",
            &[XdlValue::String("world".to_string())],
        )
        .unwrap();
        assert_eq!(result, XdlValue::Long(1));
    }

    #[test]
    fn test_multidim_shape() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let shape = vec![2, 3];
        let result = call_multidim_method(&data, &shape, "Shape", &[]).unwrap();
        assert_eq!(result, XdlValue::Array(vec![2.0, 3.0]));
    }

    #[test]
    fn test_multidim_ndim() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let shape = vec![2, 3];
        let result = call_multidim_method(&data, &shape, "Ndim", &[]).unwrap();
        assert_eq!(result, XdlValue::Long(2));
    }

    #[test]
    fn test_multidim_flatten() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let shape = vec![2, 3];
        let result = call_multidim_method(&data, &shape, "Flatten", &[]).unwrap();
        assert_eq!(result, XdlValue::Array(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]));
    }

    #[test]
    fn test_nested_array_shape() {
        let rows = vec![
            XdlValue::Array(vec![1.0, 2.0, 3.0]),
            XdlValue::Array(vec![4.0, 5.0, 6.0]),
        ];
        let result = call_nested_array_method(&rows, "Shape", &[]).unwrap();
        assert_eq!(result, XdlValue::Array(vec![2.0, 3.0]));
    }

    #[test]
    fn test_nested_array_flatten() {
        let rows = vec![
            XdlValue::Array(vec![1.0, 2.0]),
            XdlValue::Array(vec![3.0, 4.0]),
        ];
        let result = call_nested_array_method(&rows, "Flatten", &[]).unwrap();
        assert_eq!(result, XdlValue::Array(vec![1.0, 2.0, 3.0, 4.0]));
    }

    #[test]
    fn test_unknown_method_error() {
        let arr = vec![1.0, 2.0, 3.0];
        let result = call_array_method(&arr, "NonExistentMethod", &[]);
        assert!(result.is_err());
        if let Err(XdlError::NotImplemented(msg)) = result {
            assert!(msg.contains("NonExistentMethod"));
            assert!(msg.contains("Available:"));
        }
    }
}
