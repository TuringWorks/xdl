//! Statistical functions module for XDL
//!
//! This module implements statistical functions equivalent to those in the original XDL.
//! Functions include descriptive statistics, probability density functions, and
//! cumulative distribution functions.

use xdl_core::{GdlType, XdlError, XdlResult, XdlValue};

/// Convert XdlValue to f64 for statistical calculations
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

/// Convert array of XdlValues to Vec<f64>
#[allow(dead_code)]
fn array_to_floats(arr: &[XdlValue]) -> XdlResult<Vec<f64>> {
    arr.iter().map(to_float).collect()
}

/// Convert f64 result back to appropriate XdlValue type
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

/// MOMENT - Calculate statistical moments
/// Returns: [mean, variance, skewness, kurtosis]
pub fn moment(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "MOMENT: Expected at least 1 argument".to_string(),
        ));
    }

    let values = match &args[0] {
        XdlValue::Array(arr) => {
            if arr.is_empty() {
                return Err(XdlError::InvalidArgument(
                    "MOMENT: Input array is empty".to_string(),
                ));
            }
            arr.clone()
        }
        val => {
            let num_val = to_float(val)?;
            vec![num_val]
        }
    };

    let n = values.len() as f64;
    if n < 1.0 {
        return Err(XdlError::InvalidArgument(
            "MOMENT: Need at least one value".to_string(),
        ));
    }

    // Calculate mean
    let mean = values.iter().sum::<f64>() / n;

    // Calculate variance
    let variance = if n > 1.0 {
        let sum_sq_diff: f64 = values.iter().map(|&x| (x - mean).powi(2)).sum();
        sum_sq_diff / (n - 1.0)
    } else {
        0.0
    };

    // Calculate skewness (third moment)
    let skewness = if variance > 0.0 && n > 2.0 {
        let sum_cubed_diff: f64 = values
            .iter()
            .map(|&x| ((x - mean) / variance.sqrt()).powi(3))
            .sum();
        sum_cubed_diff / n
    } else {
        0.0
    };

    // Calculate kurtosis (fourth moment)
    let kurtosis = if variance > 0.0 && n > 3.0 {
        let sum_fourth_diff: f64 = values
            .iter()
            .map(|&x| ((x - mean) / variance.sqrt()).powi(4))
            .sum();
        (sum_fourth_diff / n) - 3.0 // Subtract 3 for excess kurtosis
    } else {
        0.0
    };

    // Return array of moments
    Ok(XdlValue::Array(vec![mean, variance, skewness, kurtosis]))
}

/// VARIANCE - Calculate variance of input data
pub fn variance(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "VARIANCE: Expected at least 1 argument".to_string(),
        ));
    }

    let values = match &args[0] {
        XdlValue::Array(arr) => {
            if arr.is_empty() {
                return Err(XdlError::InvalidArgument(
                    "VARIANCE: Input array is empty".to_string(),
                ));
            }
            arr.clone()
        }
        val => {
            let _num_val = to_float(val)?;
            return Ok(XdlValue::Double(0.0)); // Variance of single value is 0
        }
    };

    let n = values.len() as f64;
    if n < 2.0 {
        return Ok(XdlValue::Double(0.0));
    }

    let mean = values.iter().sum::<f64>() / n;
    let sum_sq_diff: f64 = values.iter().map(|&x| (x - mean).powi(2)).sum();

    let variance = sum_sq_diff / (n - 1.0);
    Ok(XdlValue::Double(variance))
}

/// STDDEV - Calculate standard deviation
pub fn stddev(args: &[XdlValue]) -> XdlResult<XdlValue> {
    let var_result = variance(args)?;
    match var_result {
        XdlValue::Double(v) => Ok(XdlValue::Double(v.sqrt())),
        XdlValue::Float(v) => Ok(XdlValue::Float(v.sqrt())),
        _ => Err(XdlError::RuntimeError(
            "STDDEV: Unexpected variance result".to_string(),
        )),
    }
}

/// MEDIAN - Calculate median value
pub fn median(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "MEDIAN: Expected at least 1 argument".to_string(),
        ));
    }

    let mut values = match &args[0] {
        XdlValue::Array(arr) => {
            if arr.is_empty() {
                return Err(XdlError::InvalidArgument(
                    "MEDIAN: Input array is empty".to_string(),
                ));
            }
            arr.clone()
        }
        val => {
            let num_val = to_float(val)?;
            return Ok(from_float(num_val, val.gdl_type()));
        }
    };

    values.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let n = values.len();
    let median = if n % 2 == 0 {
        // Even number of elements - average of middle two
        (values[n / 2 - 1] + values[n / 2]) / 2.0
    } else {
        // Odd number of elements - middle element
        values[n / 2]
    };

    Ok(XdlValue::Double(median))
}

/// MEANABSDEV - Calculate mean absolute deviation
pub fn meanabsdev(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "MEANABSDEV: Expected at least 1 argument".to_string(),
        ));
    }

    let values = match &args[0] {
        XdlValue::Array(arr) => {
            if arr.is_empty() {
                return Err(XdlError::InvalidArgument(
                    "MEANABSDEV: Input array is empty".to_string(),
                ));
            }
            arr.clone()
        }
        val => {
            let _num_val = to_float(val)?;
            return Ok(XdlValue::Double(0.0)); // MAD of single value is 0
        }
    };

    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let mad = values.iter().map(|&x| (x - mean).abs()).sum::<f64>() / n;

    Ok(XdlValue::Double(mad))
}

/// SKEWNESS - Calculate skewness (third moment)  
pub fn skewness(args: &[XdlValue]) -> XdlResult<XdlValue> {
    let moments = moment(args)?;
    match moments {
        XdlValue::Array(arr) if arr.len() >= 3 => Ok(XdlValue::Double(arr[2])),
        _ => Err(XdlError::RuntimeError(
            "SKEWNESS: Failed to calculate moments".to_string(),
        )),
    }
}

/// KURTOSIS - Calculate kurtosis (fourth moment)
pub fn kurtosis(args: &[XdlValue]) -> XdlResult<XdlValue> {
    let moments = moment(args)?;
    match moments {
        XdlValue::Array(arr) if arr.len() >= 4 => Ok(XdlValue::Double(arr[3])),
        _ => Err(XdlError::RuntimeError(
            "KURTOSIS: Failed to calculate moments".to_string(),
        )),
    }
}

// Basic implementations without GSL - will be enhanced later with proper GSL bindings

/// GAUSS_PDF - Gaussian probability density function (basic implementation)
pub fn gauss_pdf(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "GAUSS_PDF: Expected at least 1 argument".to_string(),
        ));
    }

    // Basic implementation using standard normal CDF approximation
    // This is a placeholder - should be replaced with GSL implementation
    let x = to_float(&args[0])?;

    // Standard normal CDF approximation (Abramowitz and Stegun)
    let t = 1.0 / (1.0 + 0.2316419 * x.abs());
    let d = 0.3989423 * (-x * x / 2.0).exp();
    let prob =
        d * t * (0.3193815 + t * (-0.3565638 + t * (1.781478 + t * (-1.821256 + t * 1.330274))));

    let result = if x >= 0.0 { 1.0 - prob } else { prob };
    Ok(from_float(result, args[0].gdl_type()))
}

/// T_PDF - Student's t-distribution PDF (basic implementation)
pub fn t_pdf(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "T_PDF: Expected 2 arguments (value, degrees_of_freedom)".to_string(),
        ));
    }

    let _x = to_float(&args[0])?;
    let df = to_float(&args[1])?;

    if df <= 0.0 {
        return Err(XdlError::InvalidArgument(
            "T_PDF: Degrees of freedom must be positive".to_string(),
        ));
    }

    // Basic t-distribution CDF approximation
    // This is a placeholder - should be replaced with GSL implementation
    let result = 0.5; // Placeholder value

    Ok(from_float(result, args[0].gdl_type()))
}

/// CHISQR_PDF - Chi-square probability density function (basic implementation)
pub fn chisqr_pdf(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "CHISQR_PDF: Expected 2 arguments (value, degrees_of_freedom)".to_string(),
        ));
    }

    let x = to_float(&args[0])?;
    let df = to_float(&args[1])?;

    if df <= 0.0 {
        return Err(XdlError::InvalidArgument(
            "CHISQR_PDF: Degrees of freedom must be positive".to_string(),
        ));
    }

    if x < 0.0 {
        return Ok(XdlValue::Double(0.0));
    }

    // Basic chi-square PDF approximation
    // This is a placeholder - should be replaced with GSL implementation
    let result = 0.5; // Placeholder value

    Ok(from_float(result, args[0].gdl_type()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variance() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let args = vec![XdlValue::Array(data)];
        let result = variance(&args).unwrap();

        // Expected variance = 2.5
        match result {
            XdlValue::Double(v) => assert!((v - 2.5).abs() < 1e-10),
            _ => panic!("Expected double result"),
        }
    }

    #[test]
    fn test_median() {
        // Odd number of elements
        let data = vec![1.0, 3.0, 2.0, 5.0, 4.0];
        let args = vec![XdlValue::Array(data)];
        let result = median(&args).unwrap();

        match result {
            XdlValue::Double(v) => assert!((v - 3.0).abs() < 1e-10),
            _ => panic!("Expected double result"),
        }

        // Even number of elements
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let args = vec![XdlValue::Array(data)];
        let result = median(&args).unwrap();

        match result {
            XdlValue::Double(v) => assert!((v - 2.5).abs() < 1e-10),
            _ => panic!("Expected double result"),
        }
    }

    #[test]
    fn test_meanabsdev() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let args = vec![XdlValue::Array(data)];
        let result = meanabsdev(&args).unwrap();

        // Mean = 3.0, MAD = (2+1+0+1+2)/5 = 1.2
        match result {
            XdlValue::Double(v) => assert!((v - 1.2).abs() < 1e-10),
            _ => panic!("Expected double result"),
        }
    }

    #[test]
    fn test_moment() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let args = vec![XdlValue::Array(data)];
        let result = moment(&args).unwrap();

        match result {
            XdlValue::Array(moments) => {
                assert!(moments.len() >= 2);
                assert!((moments[0] - 3.0).abs() < 1e-10); // mean
                assert!((moments[1] - 2.5).abs() < 1e-10); // variance
            }
            _ => panic!("Expected array result"),
        }
    }
}
