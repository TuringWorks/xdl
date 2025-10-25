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
        XdlValue::MultiDimArray { data, .. } => {
            if data.is_empty() {
                return Err(XdlError::InvalidArgument(
                    "VARIANCE: Input array is empty".to_string(),
                ));
            }
            data.clone()
        }
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

/// CORRELATE - Compute correlation coefficient
/// CORRELATE(x, y) computes Pearson correlation coefficient
pub fn correlate(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "CORRELATE: Expected 2 arguments (x, y)".to_string(),
        ));
    }

    let x = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let y = match &args[1] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    if x.len() != y.len() {
        return Err(XdlError::DimensionError(
            "CORRELATE: Arrays must have same length".to_string(),
        ));
    }

    if x.is_empty() {
        return Err(XdlError::InvalidArgument(
            "CORRELATE: Arrays cannot be empty".to_string(),
        ));
    }

    // Calculate means
    let n = x.len() as f64;
    let mean_x: f64 = x.iter().sum::<f64>() / n;
    let mean_y: f64 = y.iter().sum::<f64>() / n;

    // Calculate correlation coefficient
    let mut sum_xy = 0.0;
    let mut sum_x2 = 0.0;
    let mut sum_y2 = 0.0;

    for i in 0..x.len() {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        sum_xy += dx * dy;
        sum_x2 += dx * dx;
        sum_y2 += dy * dy;
    }

    let r = if sum_x2 > 0.0 && sum_y2 > 0.0 {
        sum_xy / (sum_x2.sqrt() * sum_y2.sqrt())
    } else {
        0.0
    };

    Ok(XdlValue::Double(r))
}

/// REGRESS - Multiple linear regression
/// REGRESS(x, y [, /DOUBLE]) computes regression coefficients
/// x can be 1D (simple regression) or 2D (multiple regression)
/// Returns array of coefficients [intercept, slope, ...]
pub fn regress(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "REGRESS: Expected at least 2 arguments (x, y)".to_string(),
        ));
    }

    // Extract y values
    let y = match &args[1] {
        XdlValue::Array(arr) => arr.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let n = y.len();
    if n == 0 {
        return Err(XdlError::InvalidArgument(
            "REGRESS: Arrays cannot be empty".to_string(),
        ));
    }

    // Extract x values (can be 1D or 2D)
    let x_matrix: Vec<Vec<f64>> = match &args[0] {
        XdlValue::Array(arr) => {
            if arr.len() != n {
                return Err(XdlError::DimensionError(
                    "REGRESS: x and y must have same length".to_string(),
                ));
            }
            // Simple linear regression: y = a + b*x
            // Add column of ones for intercept
            vec![vec![1.0; n], arr.clone()]
        }
        XdlValue::MultiDimArray { data, shape } => {
            if shape.len() != 2 {
                return Err(XdlError::DimensionError(
                    "REGRESS: x must be 1D or 2D array".to_string(),
                ));
            }
            // Multiple regression
            let nrows = shape[0];
            let ncols = shape[1];
            if nrows != n {
                return Err(XdlError::DimensionError(
                    "REGRESS: Number of observations must match".to_string(),
                ));
            }
            // Add column of ones for intercept, then data columns
            let mut matrix = vec![vec![1.0; n]];
            for col in 0..ncols {
                let mut column = Vec::with_capacity(n);
                for row in 0..nrows {
                    column.push(data[row * ncols + col]);
                }
                matrix.push(column);
            }
            matrix
        }
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    // Solve normal equations: (X'X)^-1 X'y
    // Using nalgebra for matrix operations
    use nalgebra::{DMatrix, DVector};

    let n_predictors = x_matrix.len();

    // Build X matrix (n x p) in column-major order
    // Each column in x_matrix is a predictor variable
    let mut x_data = Vec::with_capacity(n * n_predictors);
    for predictor_col in &x_matrix {
        for &value in predictor_col {
            x_data.push(value);
        }
    }
    let x_mat = DMatrix::from_vec(n, n_predictors, x_data);
    let y_vec = DVector::from_vec(y);

    // Compute X'X and X'y
    let xtx = x_mat.transpose() * &x_mat;
    let xty = x_mat.transpose() * y_vec;

    // Solve (X'X)beta = X'y
    match xtx.lu().solve(&xty) {
        Some(beta) => {
            let coefficients: Vec<f64> = beta.iter().copied().collect();
            Ok(XdlValue::Array(coefficients))
        }
        None => Err(XdlError::RuntimeError(
            "REGRESS: Singular matrix, cannot compute regression".to_string(),
        )),
    }
}

/// LINFIT - Simple linear fit with optional errors
/// LINFIT(x, y) returns [intercept, slope] for y = a + b*x
pub fn linfit(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "LINFIT: Expected at least 2 arguments (x, y)".to_string(),
        ));
    }

    let x = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let y = match &args[1] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    if x.len() != y.len() {
        return Err(XdlError::DimensionError(
            "LINFIT: Arrays must have same length".to_string(),
        ));
    }

    if x.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "LINFIT: Need at least 2 data points".to_string(),
        ));
    }

    // Calculate means
    let n = x.len() as f64;
    let mean_x: f64 = x.iter().sum::<f64>() / n;
    let mean_y: f64 = y.iter().sum::<f64>() / n;

    // Calculate slope and intercept using least squares
    let mut sum_xy = 0.0;
    let mut sum_x2 = 0.0;

    for i in 0..x.len() {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        sum_xy += dx * dy;
        sum_x2 += dx * dx;
    }

    if sum_x2 == 0.0 {
        return Err(XdlError::RuntimeError(
            "LINFIT: Cannot fit - all x values are identical".to_string(),
        ));
    }

    let slope = sum_xy / sum_x2;
    let intercept = mean_y - slope * mean_x;

    Ok(XdlValue::Array(vec![intercept, slope]))
}

/// PERCENTILES - Calculate percentiles of data
/// PERCENTILES(data, percentiles_array)
/// Returns values at specified percentiles (0-100)
pub fn percentiles(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 2 {
        return Err(XdlError::InvalidArgument(format!(
            "PERCENTILES: Expected 2 arguments (data, percentiles), got {}",
            args.len()
        )));
    }

    let mut values = match &args[0] {
        XdlValue::Array(arr) => {
            if arr.is_empty() {
                return Err(XdlError::InvalidArgument(
                    "PERCENTILES: Input array is empty".to_string(),
                ));
            }
            arr.clone()
        }
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let percentile_vals = match &args[1] {
        XdlValue::Array(arr) => arr.clone(),
        XdlValue::Double(v) => vec![*v],
        XdlValue::Float(v) => vec![*v as f64],
        XdlValue::Long(v) => vec![*v as f64],
        XdlValue::Int(v) => vec![*v as f64],
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric or array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    // Sort the data
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = values.len();

    // Calculate percentiles
    let mut results = Vec::new();
    for &p in &percentile_vals {
        if !(0.0..=100.0).contains(&p) {
            return Err(XdlError::InvalidArgument(format!(
                "PERCENTILES: Percentile {} out of range [0, 100]",
                p
            )));
        }

        // Linear interpolation method
        let pos = (p / 100.0) * (n - 1) as f64;
        let lower_idx = pos.floor() as usize;
        let upper_idx = pos.ceil() as usize;

        let result = if lower_idx == upper_idx {
            values[lower_idx]
        } else {
            let frac = pos - lower_idx as f64;
            values[lower_idx] * (1.0 - frac) + values[upper_idx] * frac
        };

        results.push(result);
    }

    if results.len() == 1 {
        Ok(XdlValue::Double(results[0]))
    } else {
        Ok(XdlValue::Array(results))
    }
}

/// ROBUST_MEAN - Robust mean using iterative sigma clipping
/// ROBUST_MEAN(data [, sigma_cut])
/// Computes mean after removing outliers beyond sigma_cut standard deviations
pub fn robust_mean(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "ROBUST_MEAN: Expected at least 1 argument".to_string(),
        ));
    }

    let values = match &args[0] {
        XdlValue::Array(arr) => {
            if arr.is_empty() {
                return Err(XdlError::InvalidArgument(
                    "ROBUST_MEAN: Input array is empty".to_string(),
                ));
            }
            arr.clone()
        }
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    // Default sigma cutoff
    let sigma_cut = if args.len() >= 2 {
        to_float(&args[1])?
    } else {
        3.0 // Default 3-sigma clipping
    };

    // Iteratively clip outliers
    let mut clipped_values = values.clone();
    let max_iterations = 10;

    for _ in 0..max_iterations {
        if clipped_values.len() < 2 {
            break;
        }

        let n = clipped_values.len() as f64;
        let mean: f64 = clipped_values.iter().sum::<f64>() / n;
        let variance: f64 = clipped_values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>()
            / n;
        let stddev = variance.sqrt();

        if stddev == 0.0 {
            break;
        }

        // Keep values within sigma_cut standard deviations
        let new_clipped: Vec<f64> = clipped_values
            .iter()
            .filter(|&&x| (x - mean).abs() <= sigma_cut * stddev)
            .copied()
            .collect();

        // If no values were removed, we're done
        if new_clipped.len() == clipped_values.len() {
            break;
        }

        clipped_values = new_clipped;
    }

    if clipped_values.is_empty() {
        return Err(XdlError::RuntimeError(
            "ROBUST_MEAN: All values were clipped".to_string(),
        ));
    }

    let robust_mean: f64 = clipped_values.iter().sum::<f64>() / clipped_values.len() as f64;
    Ok(XdlValue::Double(robust_mean))
}

/// TRIMMED_MEAN - Mean after trimming percentage from each tail
/// TRIMMED_MEAN(data, trim_fraction)
/// trim_fraction: fraction to trim from each end (0.0 to 0.5)
pub fn trimmed_mean(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 2 {
        return Err(XdlError::InvalidArgument(format!(
            "TRIMMED_MEAN: Expected 2 arguments (data, trim_fraction), got {}",
            args.len()
        )));
    }

    let mut values = match &args[0] {
        XdlValue::Array(arr) => {
            if arr.is_empty() {
                return Err(XdlError::InvalidArgument(
                    "TRIMMED_MEAN: Input array is empty".to_string(),
                ));
            }
            arr.clone()
        }
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let trim_frac = to_float(&args[1])?;
    if !(0.0..0.5).contains(&trim_frac) {
        return Err(XdlError::InvalidArgument(format!(
            "TRIMMED_MEAN: Trim fraction {} must be in range [0, 0.5)",
            trim_frac
        )));
    }

    // Sort the values
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let n = values.len();
    let trim_count = (n as f64 * trim_frac).floor() as usize;

    if trim_count * 2 >= n {
        return Err(XdlError::InvalidArgument(
            "TRIMMED_MEAN: Trim amount would remove all data".to_string(),
        ));
    }

    // Calculate mean of middle values
    let trimmed_values = &values[trim_count..n - trim_count];
    let mean: f64 = trimmed_values.iter().sum::<f64>() / trimmed_values.len() as f64;

    Ok(XdlValue::Double(mean))
}

/// RESISTANT_MEAN - Resistant mean using median-based approach
/// Similar to robust_mean but uses median absolute deviation
pub fn resistant_mean(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "RESISTANT_MEAN: Expected at least 1 argument".to_string(),
        ));
    }

    let mut values = match &args[0] {
        XdlValue::Array(arr) => {
            if arr.is_empty() {
                return Err(XdlError::InvalidArgument(
                    "RESISTANT_MEAN: Input array is empty".to_string(),
                ));
            }
            arr.clone()
        }
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    // Calculate median
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = values.len();
    let med = if n % 2 == 0 {
        (values[n / 2 - 1] + values[n / 2]) / 2.0
    } else {
        values[n / 2]
    };

    // Calculate MAD (Median Absolute Deviation)
    let mut abs_devs: Vec<f64> = values.iter().map(|&x| (x - med).abs()).collect();
    abs_devs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mad = if abs_devs.len().is_multiple_of(2) {
        (abs_devs[abs_devs.len() / 2 - 1] + abs_devs[abs_devs.len() / 2]) / 2.0
    } else {
        abs_devs[abs_devs.len() / 2]
    };

    // Use MAD-based robust estimate
    // Keep values within 3*MAD of median
    let cutoff = if mad > 0.0 { 4.5 * mad } else { 1e10 };

    let filtered: Vec<f64> = values
        .iter()
        .filter(|&&x| (x - med).abs() <= cutoff)
        .copied()
        .collect();

    if filtered.is_empty() {
        return Ok(XdlValue::Double(med));
    }

    let resistant_mean: f64 = filtered.iter().sum::<f64>() / filtered.len() as f64;
    Ok(XdlValue::Double(resistant_mean))
}

/// RANDOM_POISSON - Generate Poisson-distributed random numbers
/// RANDOM_POISSON(seed, lambda, n)
/// lambda: mean of the Poisson distribution
pub fn random_poisson(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(format!(
            "RANDOM_POISSON: Expected at least 2 arguments (seed, lambda [, n]), got {}",
            args.len()
        )));
    }

    // Get seed
    let seed = match &args[0] {
        XdlValue::Long(v) => *v as u64,
        XdlValue::Int(v) => *v as u64,
        XdlValue::Double(v) => *v as u64,
        XdlValue::Float(v) => *v as u64,
        _ => 12345u64,
    };

    // Get lambda (mean)
    let lambda = to_float(&args[1])?;
    if lambda < 0.0 {
        return Err(XdlError::InvalidArgument(
            "RANDOM_POISSON: Lambda must be non-negative".to_string(),
        ));
    }

    // Get count (optional)
    let count = if args.len() >= 3 {
        match &args[2] {
            XdlValue::Long(v) => *v as usize,
            XdlValue::Int(v) => *v as usize,
            _ => 1,
        }
    } else {
        1
    };

    // Simple LCG for random numbers
    let mut rng_state = seed;
    let mut uniform = || -> f64 {
        let a = 1664525u64;
        let c = 1013904223u64;
        rng_state = a.wrapping_mul(rng_state).wrapping_add(c);
        ((rng_state % 1000000) as f64) / 1000000.0
    };

    // Generate Poisson random variates using Knuth's algorithm
    let mut poisson = || -> f64 {
        let l = (-lambda).exp();
        let mut k = 0.0;
        let mut p = 1.0;

        loop {
            k += 1.0;
            p *= uniform();
            if p <= l {
                break;
            }
        }

        k - 1.0
    };

    if count == 1 {
        Ok(XdlValue::Double(poisson()))
    } else {
        let values: Vec<f64> = (0..count).map(|_| poisson()).collect();
        Ok(XdlValue::Array(values))
    }
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

    #[test]
    fn test_correlate_perfect_positive() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let args = vec![XdlValue::Array(x), XdlValue::Array(y)];
        let result = correlate(&args).unwrap();

        match result {
            XdlValue::Double(r) => assert!((r - 1.0).abs() < 1e-10),
            _ => panic!("Expected double result"),
        }
    }

    #[test]
    fn test_correlate_perfect_negative() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![10.0, 8.0, 6.0, 4.0, 2.0];
        let args = vec![XdlValue::Array(x), XdlValue::Array(y)];
        let result = correlate(&args).unwrap();

        match result {
            XdlValue::Double(r) => assert!((r + 1.0).abs() < 1e-10),
            _ => panic!("Expected double result"),
        }
    }

    #[test]
    fn test_linfit_simple() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let args = vec![XdlValue::Array(x), XdlValue::Array(y)];
        let result = linfit(&args).unwrap();

        match result {
            XdlValue::Array(coeffs) => {
                assert_eq!(coeffs.len(), 2);
                assert!((coeffs[0] - 0.0).abs() < 1e-10); // intercept
                assert!((coeffs[1] - 2.0).abs() < 1e-10); // slope
            }
            _ => panic!("Expected array result"),
        }
    }

    #[test]
    fn test_linfit_with_intercept() {
        let x = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let y = vec![3.0, 5.0, 7.0, 9.0, 11.0];
        let args = vec![XdlValue::Array(x), XdlValue::Array(y)];
        let result = linfit(&args).unwrap();

        match result {
            XdlValue::Array(coeffs) => {
                assert_eq!(coeffs.len(), 2);
                assert!((coeffs[0] - 3.0).abs() < 1e-10); // intercept
                assert!((coeffs[1] - 2.0).abs() < 1e-10); // slope
            }
            _ => panic!("Expected array result"),
        }
    }

    #[test]
    fn test_regress_simple() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let args = vec![XdlValue::Array(x), XdlValue::Array(y)];
        let result = regress(&args).unwrap();

        match result {
            XdlValue::Array(coeffs) => {
                assert_eq!(coeffs.len(), 2);
                assert!((coeffs[0] - 0.0).abs() < 1e-10); // intercept
                assert!((coeffs[1] - 2.0).abs() < 1e-10); // slope
            }
            _ => panic!("Expected array result"),
        }
    }

    #[test]
    fn test_regress_noisy_data() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let y = vec![2.1, 3.9, 6.2, 7.8, 10.1, 11.9, 14.2, 15.8];
        let args = vec![XdlValue::Array(x), XdlValue::Array(y)];
        let result = regress(&args).unwrap();

        match result {
            XdlValue::Array(coeffs) => {
                assert_eq!(coeffs.len(), 2);
                // Should be close to y = 2*x
                assert!((coeffs[0] - 0.0).abs() < 0.1); // intercept close to 0
                assert!((coeffs[1] - 2.0).abs() < 0.1); // slope close to 2
            }
            _ => panic!("Expected array result"),
        }
    }
}
