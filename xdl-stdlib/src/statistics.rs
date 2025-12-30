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

/// Extract array data from XdlValue
fn extract_array(value: &XdlValue) -> XdlResult<Vec<f64>> {
    match value {
        XdlValue::Array(arr) => Ok(arr.clone()),
        XdlValue::MultiDimArray { data, .. } => Ok(data.clone()),
        _ => Err(XdlError::TypeMismatch {
            expected: "array".to_string(),
            actual: format!("{:?}", value.gdl_type()),
        }),
    }
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

// ============================================================
// Fitting Functions
// ============================================================

/// LINFIT - Linear fit (y = a + b*x)
/// LINFIT(x, y [, SIGMA=sigma])
/// Returns [a, b] coefficients
pub fn linfit(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "LINFIT: Expected at least 2 arguments (x, y)".to_string(),
        ));
    }

    let x = extract_array(&args[0])?;
    let y = extract_array(&args[1])?;

    if x.len() != y.len() {
        return Err(XdlError::InvalidArgument(
            "LINFIT: x and y arrays must have same length".to_string(),
        ));
    }

    let n = x.len() as f64;
    if n < 2.0 {
        return Err(XdlError::InvalidArgument(
            "LINFIT: Need at least 2 points".to_string(),
        ));
    }

    // Calculate sums for linear regression
    let sum_x: f64 = x.iter().sum();
    let sum_y: f64 = y.iter().sum();
    let sum_xy: f64 = x.iter().zip(y.iter()).map(|(xi, yi)| xi * yi).sum();
    let sum_x2: f64 = x.iter().map(|xi| xi * xi).sum();

    // Linear regression coefficients
    let denom = n * sum_x2 - sum_x * sum_x;
    if denom.abs() < 1e-15 {
        return Err(XdlError::RuntimeError(
            "LINFIT: Singular matrix (all x values the same?)".to_string(),
        ));
    }

    let b = (n * sum_xy - sum_x * sum_y) / denom;
    let a = (sum_y - b * sum_x) / n;

    Ok(XdlValue::Array(vec![a, b]))
}

/// POLY_FIT - Polynomial fit
/// POLY_FIT(x, y, degree)
/// Returns array of polynomial coefficients [a0, a1, a2, ...]
pub fn poly_fit(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(
            "POLY_FIT: Expected 3 arguments (x, y, degree)".to_string(),
        ));
    }

    let x = extract_array(&args[0])?;
    let y = extract_array(&args[1])?;
    let degree = match &args[2] {
        XdlValue::Long(n) => *n as usize,
        XdlValue::Int(n) => *n as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[2].gdl_type()),
            })
        }
    };

    if x.len() != y.len() {
        return Err(XdlError::InvalidArgument(
            "POLY_FIT: x and y arrays must have same length".to_string(),
        ));
    }

    let n = x.len();
    let m = degree + 1; // Number of coefficients

    if n < m {
        return Err(XdlError::InvalidArgument(
            "POLY_FIT: Need more data points than polynomial degree".to_string(),
        ));
    }

    // Build Vandermonde matrix X and solve X^T * X * c = X^T * y
    // Using normal equations (simple implementation)

    // Compute X^T * X (m x m symmetric matrix)
    let mut xtx = vec![vec![0.0; m]; m];
    for i in 0..m {
        for j in 0..m {
            for k in 0..n {
                xtx[i][j] += x[k].powi((i + j) as i32);
            }
        }
    }

    // Compute X^T * y (m vector)
    let mut xty = vec![0.0; m];
    for i in 0..m {
        for k in 0..n {
            xty[i] += x[k].powi(i as i32) * y[k];
        }
    }

    // Solve using Gaussian elimination
    let coeffs = solve_linear_system(&xtx, &xty)?;

    Ok(XdlValue::Array(coeffs))
}

// Helper: Gaussian elimination to solve Ax = b
fn solve_linear_system(a: &[Vec<f64>], b: &[f64]) -> XdlResult<Vec<f64>> {
    let n = b.len();
    let mut aug: Vec<Vec<f64>> = a.iter().cloned().collect();

    // Augment with b
    for (i, row) in aug.iter_mut().enumerate() {
        row.push(b[i]);
    }

    // Forward elimination with partial pivoting
    for i in 0..n {
        // Find pivot
        let mut max_row = i;
        for k in (i + 1)..n {
            if aug[k][i].abs() > aug[max_row][i].abs() {
                max_row = k;
            }
        }
        aug.swap(i, max_row);

        if aug[i][i].abs() < 1e-15 {
            return Err(XdlError::RuntimeError(
                "POLY_FIT: Singular matrix".to_string(),
            ));
        }

        // Eliminate column
        for k in (i + 1)..n {
            let factor = aug[k][i] / aug[i][i];
            for j in i..=n {
                aug[k][j] -= factor * aug[i][j];
            }
        }
    }

    // Back substitution
    let mut x = vec![0.0; n];
    for i in (0..n).rev() {
        x[i] = aug[i][n];
        for j in (i + 1)..n {
            x[i] -= aug[i][j] * x[j];
        }
        x[i] /= aug[i][i];
    }

    Ok(x)
}

/// REGRESS - Multiple linear regression
/// REGRESS(y, x_matrix)
/// x_matrix columns are independent variables
/// Returns regression coefficients
pub fn regress(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "REGRESS: Expected 2 arguments (y, x_matrix)".to_string(),
        ));
    }

    let y = extract_array(&args[0])?;

    // Extract X matrix (can be 2D array or multiple 1D arrays)
    let x_cols = match &args[1] {
        XdlValue::MultiDimArray { data, shape } => {
            if shape.len() != 2 {
                return Err(XdlError::InvalidArgument(
                    "REGRESS: X must be 2D matrix".to_string(),
                ));
            }
            let n_rows = shape[0];
            let n_cols = shape[1];
            let mut cols = Vec::new();
            for j in 0..n_cols {
                let col: Vec<f64> = (0..n_rows).map(|i| data[i * n_cols + j]).collect();
                cols.push(col);
            }
            cols
        }
        XdlValue::Array(arr) => vec![arr.clone()],
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let n = y.len();
    let m = x_cols.len() + 1; // +1 for intercept

    // Build design matrix with intercept column
    let mut xtx = vec![vec![0.0; m]; m];
    let mut xty = vec![0.0; m];

    // First row/column for intercept
    xtx[0][0] = n as f64;
    xty[0] = y.iter().sum();

    for (j, col) in x_cols.iter().enumerate() {
        let j_idx = j + 1;
        xtx[0][j_idx] = col.iter().sum();
        xtx[j_idx][0] = xtx[0][j_idx];
        xty[j_idx] = col.iter().zip(y.iter()).map(|(xi, yi)| xi * yi).sum();
    }

    // Fill in X^T * X
    for (j1, col1) in x_cols.iter().enumerate() {
        for (j2, col2) in x_cols.iter().enumerate() {
            xtx[j1 + 1][j2 + 1] = col1.iter().zip(col2.iter()).map(|(a, b)| a * b).sum();
        }
    }

    // Solve
    let coeffs = solve_linear_system(&xtx, &xty)?;

    Ok(XdlValue::Array(coeffs))
}

/// CORRELATE - Compute correlation coefficient
pub fn correlate(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "CORRELATE: Expected 2 arguments (x, y)".to_string(),
        ));
    }

    let x = extract_array(&args[0])?;
    let y = extract_array(&args[1])?;

    if x.len() != y.len() {
        return Err(XdlError::InvalidArgument(
            "CORRELATE: x and y arrays must have same length".to_string(),
        ));
    }

    let n = x.len() as f64;
    let mean_x: f64 = x.iter().sum::<f64>() / n;
    let mean_y: f64 = y.iter().sum::<f64>() / n;

    let mut cov: f64 = 0.0;
    let mut var_x: f64 = 0.0;
    let mut var_y: f64 = 0.0;

    for i in 0..x.len() {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        cov += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }

    let denom = (var_x * var_y).sqrt();
    let r = if denom > 1e-15 { cov / denom } else { 0.0 };

    Ok(XdlValue::Double(r))
}

// ============================================================
// Interpolation Functions
// ============================================================

/// INTERPOL - Linear interpolation
/// INTERPOL(y, x, x_new) - Interpolate y at x_new points
pub fn interpol(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(
            "INTERPOL: Expected 3 arguments (y, x, x_new)".to_string(),
        ));
    }

    let y = extract_array(&args[0])?;
    let x = extract_array(&args[1])?;
    let x_new = extract_array(&args[2])?;

    if x.len() != y.len() {
        return Err(XdlError::InvalidArgument(
            "INTERPOL: x and y arrays must have same length".to_string(),
        ));
    }

    if x.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "INTERPOL: Need at least 2 points".to_string(),
        ));
    }

    let mut result = Vec::with_capacity(x_new.len());

    for &xn in &x_new {
        // Find bracketing interval
        let mut i = 0;
        while i < x.len() - 1 && x[i + 1] < xn {
            i += 1;
        }

        // Handle edge cases
        if xn <= x[0] {
            result.push(y[0]);
        } else if xn >= x[x.len() - 1] {
            result.push(y[y.len() - 1]);
        } else {
            // Linear interpolation
            let t = (xn - x[i]) / (x[i + 1] - x[i]);
            result.push(y[i] * (1.0 - t) + y[i + 1] * t);
        }
    }

    Ok(XdlValue::Array(result))
}

/// SPLINE - Cubic spline interpolation coefficients
/// SPLINE(x, y, t) - Spline interpolate y at points t
pub fn spline(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(
            "SPLINE: Expected 3 arguments (x, y, t)".to_string(),
        ));
    }

    let x = extract_array(&args[0])?;
    let y = extract_array(&args[1])?;
    let t = extract_array(&args[2])?;

    if x.len() != y.len() {
        return Err(XdlError::InvalidArgument(
            "SPLINE: x and y arrays must have same length".to_string(),
        ));
    }

    let n = x.len();
    if n < 3 {
        return Err(XdlError::InvalidArgument(
            "SPLINE: Need at least 3 points".to_string(),
        ));
    }

    // Compute cubic spline coefficients using natural spline (second derivative = 0 at endpoints)
    // Solve tridiagonal system for second derivatives
    let mut d2y = vec![0.0; n];
    let mut u = vec![0.0; n - 1];

    // Forward sweep
    for i in 1..n - 1 {
        let sig = (x[i] - x[i - 1]) / (x[i + 1] - x[i - 1]);
        let p = sig * d2y[i - 1] + 2.0;
        d2y[i] = (sig - 1.0) / p;
        u[i] = (y[i + 1] - y[i]) / (x[i + 1] - x[i]) - (y[i] - y[i - 1]) / (x[i] - x[i - 1]);
        u[i] = (6.0 * u[i] / (x[i + 1] - x[i - 1]) - sig * u[i - 1]) / p;
    }

    // Back substitution
    for k in (0..n - 2).rev() {
        d2y[k + 1] = d2y[k + 1] * d2y[k + 2] + u[k + 1];
    }

    // Evaluate spline at points t
    let mut result = Vec::with_capacity(t.len());

    for &ti in &t {
        // Find interval
        let mut k = 0;
        for j in 0..n - 1 {
            if ti >= x[j] && ti <= x[j + 1] {
                k = j;
                break;
            }
            if ti > x[n - 1] {
                k = n - 2;
            }
        }

        let h = x[k + 1] - x[k];
        let a = (x[k + 1] - ti) / h;
        let b = (ti - x[k]) / h;

        let yi = a * y[k] + b * y[k + 1]
            + ((a * a * a - a) * d2y[k] + (b * b * b - b) * d2y[k + 1]) * h * h / 6.0;

        result.push(yi);
    }

    Ok(XdlValue::Array(result))
}

/// BILINEAR - Bilinear interpolation for 2D arrays
/// BILINEAR(data, x, y) - Interpolate 2D data at fractional x, y coordinates
pub fn bilinear(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(
            "BILINEAR: Expected 3 arguments (data, x, y)".to_string(),
        ));
    }

    // Extract 2D data
    let (data, rows, cols) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => {
            if shape.len() != 2 {
                return Err(XdlError::InvalidArgument(
                    "BILINEAR: data must be 2D array".to_string(),
                ));
            }
            (data.clone(), shape[0], shape[1])
        }
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "2D array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let x_coords = extract_array(&args[1])?;
    let y_coords = extract_array(&args[2])?;

    if x_coords.len() != y_coords.len() {
        return Err(XdlError::InvalidArgument(
            "BILINEAR: x and y coordinate arrays must have same length".to_string(),
        ));
    }

    let mut result = Vec::with_capacity(x_coords.len());

    for i in 0..x_coords.len() {
        let x = x_coords[i];
        let y = y_coords[i];

        // Get integer and fractional parts
        let x0 = (x.floor() as usize).min(cols - 2);
        let y0 = (y.floor() as usize).min(rows - 2);
        let x1 = (x0 + 1).min(cols - 1);
        let y1 = (y0 + 1).min(rows - 1);

        let fx = x - x0 as f64;
        let fy = y - y0 as f64;

        // Get four corner values
        let v00 = data[y0 * cols + x0];
        let v01 = data[y0 * cols + x1];
        let v10 = data[y1 * cols + x0];
        let v11 = data[y1 * cols + x1];

        // Bilinear interpolation
        let top = v00 * (1.0 - fx) + v01 * fx;
        let bottom = v10 * (1.0 - fx) + v11 * fx;
        let value = top * (1.0 - fy) + bottom * fy;

        result.push(value);
    }

    Ok(XdlValue::Array(result))
}

// ============================================================
// Additional Statistical Functions (Phase 7 Completion)
// ============================================================

/// R_CORRELATE - Spearman rank correlation coefficient
/// R_CORRELATE(x, y)
pub fn r_correlate(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "R_CORRELATE: Expected 2 arguments (x, y)".to_string(),
        ));
    }

    let x = extract_array(&args[0])?;
    let y = extract_array(&args[1])?;

    if x.len() != y.len() {
        return Err(XdlError::InvalidArgument(
            "R_CORRELATE: x and y arrays must have same length".to_string(),
        ));
    }

    let n = x.len();
    if n < 2 {
        return Err(XdlError::InvalidArgument(
            "R_CORRELATE: Need at least 2 points".to_string(),
        ));
    }

    // Compute ranks for x
    let mut x_indexed: Vec<(usize, f64)> = x.iter().enumerate().map(|(i, &v)| (i, v)).collect();
    x_indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    let mut rank_x = vec![0.0; n];
    for (rank, (idx, _)) in x_indexed.iter().enumerate() {
        rank_x[*idx] = (rank + 1) as f64;
    }

    // Compute ranks for y
    let mut y_indexed: Vec<(usize, f64)> = y.iter().enumerate().map(|(i, &v)| (i, v)).collect();
    y_indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    let mut rank_y = vec![0.0; n];
    for (rank, (idx, _)) in y_indexed.iter().enumerate() {
        rank_y[*idx] = (rank + 1) as f64;
    }

    // Compute Spearman correlation using ranked data
    // rho = 1 - (6 * sum(d^2)) / (n * (n^2 - 1))
    let d_squared_sum: f64 = rank_x
        .iter()
        .zip(rank_y.iter())
        .map(|(rx, ry)| (rx - ry).powi(2))
        .sum();

    let rho = 1.0 - (6.0 * d_squared_sum) / (n as f64 * ((n * n) as f64 - 1.0));

    Ok(XdlValue::Double(rho))
}

/// LADFIT - Least Absolute Deviation (L1) linear fit
/// LADFIT(x, y) - Robust linear regression
/// Returns [intercept, slope]
pub fn ladfit(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "LADFIT: Expected 2 arguments (x, y)".to_string(),
        ));
    }

    let x = extract_array(&args[0])?;
    let y = extract_array(&args[1])?;

    if x.len() != y.len() {
        return Err(XdlError::InvalidArgument(
            "LADFIT: x and y arrays must have same length".to_string(),
        ));
    }

    let n = x.len();
    if n < 2 {
        return Err(XdlError::InvalidArgument(
            "LADFIT: Need at least 2 points".to_string(),
        ));
    }

    // Use iterative weighted least squares (IRLS) for L1 regression
    // Start with OLS solution
    let sum_x: f64 = x.iter().sum();
    let sum_y: f64 = y.iter().sum();
    let sum_xy: f64 = x.iter().zip(y.iter()).map(|(xi, yi)| xi * yi).sum();
    let sum_x2: f64 = x.iter().map(|xi| xi * xi).sum();

    let n_f = n as f64;
    let denom = n_f * sum_x2 - sum_x * sum_x;

    let mut slope = if denom.abs() > 1e-15 {
        (n_f * sum_xy - sum_x * sum_y) / denom
    } else {
        0.0
    };
    let mut intercept = (sum_y - slope * sum_x) / n_f;

    // Iteratively refine using weighted least squares
    let max_iter = 50;
    let tol = 1e-8;

    for _ in 0..max_iter {
        // Compute residuals
        let residuals: Vec<f64> = x
            .iter()
            .zip(y.iter())
            .map(|(xi, yi)| yi - (intercept + slope * xi))
            .collect();

        // Compute weights (1 / |residual|, with small epsilon to avoid division by zero)
        let weights: Vec<f64> = residuals
            .iter()
            .map(|r| 1.0 / (r.abs() + 1e-10))
            .collect();

        let total_weight: f64 = weights.iter().sum();

        // Weighted sums
        let w_sum_x: f64 = x.iter().zip(weights.iter()).map(|(xi, wi)| xi * wi).sum();
        let w_sum_y: f64 = y.iter().zip(weights.iter()).map(|(yi, wi)| yi * wi).sum();
        let w_sum_xy: f64 = x
            .iter()
            .zip(y.iter())
            .zip(weights.iter())
            .map(|((xi, yi), wi)| xi * yi * wi)
            .sum();
        let w_sum_x2: f64 = x
            .iter()
            .zip(weights.iter())
            .map(|(xi, wi)| xi * xi * wi)
            .sum();

        let w_denom = total_weight * w_sum_x2 - w_sum_x * w_sum_x;
        let new_slope = if w_denom.abs() > 1e-15 {
            (total_weight * w_sum_xy - w_sum_x * w_sum_y) / w_denom
        } else {
            slope
        };
        let new_intercept = (w_sum_y - new_slope * w_sum_x) / total_weight;

        // Check convergence
        if (new_slope - slope).abs() < tol && (new_intercept - intercept).abs() < tol {
            break;
        }

        slope = new_slope;
        intercept = new_intercept;
    }

    Ok(XdlValue::Array(vec![intercept, slope]))
}

/// SVDFIT - Fitting using Singular Value Decomposition
/// SVDFIT(x, y, m) - Fit polynomial of degree m-1 using SVD
/// Returns array of coefficients
pub fn svdfit(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(
            "SVDFIT: Expected 3 arguments (x, y, m)".to_string(),
        ));
    }

    let x = extract_array(&args[0])?;
    let y = extract_array(&args[1])?;
    let m = match &args[2] {
        XdlValue::Long(n) => *n as usize,
        XdlValue::Int(n) => *n as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[2].gdl_type()),
            })
        }
    };

    if x.len() != y.len() {
        return Err(XdlError::InvalidArgument(
            "SVDFIT: x and y arrays must have same length".to_string(),
        ));
    }

    let n = x.len();
    if n < m {
        return Err(XdlError::InvalidArgument(
            "SVDFIT: Need more data points than basis functions".to_string(),
        ));
    }

    // Build design matrix A (n x m)
    // Using polynomial basis: 1, x, x^2, ..., x^(m-1)
    let mut a_flat = vec![0.0; n * m];
    for i in 0..n {
        for j in 0..m {
            a_flat[i * m + j] = x[i].powi(j as i32);
        }
    }

    // Perform SVD using Jacobi algorithm (simplified)
    // A = U * S * V^T
    // Solution: x = V * S^-1 * U^T * b

    // For simplicity, use normal equations with regularization
    // (A^T * A + lambda * I) * c = A^T * y

    let lambda = 1e-10; // Small regularization

    // Compute A^T * A
    let mut ata = vec![vec![0.0; m]; m];
    for i in 0..m {
        for j in 0..m {
            for k in 0..n {
                ata[i][j] += a_flat[k * m + i] * a_flat[k * m + j];
            }
            if i == j {
                ata[i][j] += lambda; // Add regularization
            }
        }
    }

    // Compute A^T * y
    let mut aty = vec![0.0; m];
    for i in 0..m {
        for k in 0..n {
            aty[i] += a_flat[k * m + i] * y[k];
        }
    }

    // Solve using Gaussian elimination
    let coeffs = solve_linear_system(&ata, &aty)?;

    Ok(XdlValue::Array(coeffs))
}

/// CURVEFIT - Non-linear curve fitting (Levenberg-Marquardt)
/// CURVEFIT(x, y, params, function_name)
/// For now, implements Gaussian fitting: y = a[0] * exp(-((x - a[1])/a[2])^2)
pub fn curvefit(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(
            "CURVEFIT: Expected at least 3 arguments (x, y, initial_params)".to_string(),
        ));
    }

    let x = extract_array(&args[0])?;
    let y = extract_array(&args[1])?;
    let mut params = extract_array(&args[2])?;

    if x.len() != y.len() {
        return Err(XdlError::InvalidArgument(
            "CURVEFIT: x and y arrays must have same length".to_string(),
        ));
    }

    // Ensure we have 3 parameters for Gaussian fit
    while params.len() < 3 {
        params.push(1.0);
    }

    // Gaussian model: f(x) = a[0] * exp(-((x - a[1])/a[2])^2)
    let gaussian = |x: f64, a: &[f64]| -> f64 {
        if a[2].abs() < 1e-15 {
            return 0.0;
        }
        a[0] * (-(((x - a[1]) / a[2]).powi(2))).exp()
    };

    // Jacobian for Gaussian
    let jacobian = |x: f64, a: &[f64]| -> Vec<f64> {
        if a[2].abs() < 1e-15 {
            return vec![0.0, 0.0, 0.0];
        }
        let z = (x - a[1]) / a[2];
        let exp_term = (-z * z).exp();
        vec![
            exp_term,                              // df/da[0]
            a[0] * exp_term * 2.0 * z / a[2],      // df/da[1]
            a[0] * exp_term * 2.0 * z * z / a[2],  // df/da[2]
        ]
    };

    // Levenberg-Marquardt optimization
    let max_iter = 100;
    let mut lambda = 0.001;
    let n_params = params.len();

    for _iter in 0..max_iter {
        // Compute residuals and Jacobian
        let mut residuals = Vec::with_capacity(x.len());
        let mut jac_matrix = vec![vec![0.0; n_params]; x.len()];

        for i in 0..x.len() {
            residuals.push(y[i] - gaussian(x[i], &params));
            let jac_row = jacobian(x[i], &params);
            for j in 0..n_params {
                jac_matrix[i][j] = jac_row[j];
            }
        }

        // Compute J^T * J and J^T * r
        let mut jtj = vec![vec![0.0; n_params]; n_params];
        let mut jtr = vec![0.0; n_params];

        for i in 0..n_params {
            for j in 0..n_params {
                for k in 0..x.len() {
                    jtj[i][j] += jac_matrix[k][i] * jac_matrix[k][j];
                }
            }
            for k in 0..x.len() {
                jtr[i] += jac_matrix[k][i] * residuals[k];
            }
        }

        // Add damping: (J^T*J + lambda*diag(J^T*J)) * delta = J^T * r
        for i in 0..n_params {
            jtj[i][i] *= 1.0 + lambda;
        }

        // Solve for delta
        if let Ok(delta) = solve_linear_system(&jtj, &jtr) {
            // Try new parameters
            let new_params: Vec<f64> = params.iter().zip(delta.iter()).map(|(p, d)| p + d).collect();

            // Compute new error
            let old_error: f64 = residuals.iter().map(|r| r * r).sum();
            let new_residuals: Vec<f64> = x
                .iter()
                .zip(y.iter())
                .map(|(&xi, &yi)| yi - gaussian(xi, &new_params))
                .collect();
            let new_error: f64 = new_residuals.iter().map(|r| r * r).sum();

            if new_error < old_error {
                params = new_params;
                lambda /= 10.0;

                // Check convergence
                let delta_norm: f64 = delta.iter().map(|d| d * d).sum::<f64>().sqrt();
                if delta_norm < 1e-8 {
                    break;
                }
            } else {
                lambda *= 10.0;
            }
        } else {
            lambda *= 10.0;
        }

        if lambda > 1e10 {
            break; // Convergence failed
        }
    }

    Ok(XdlValue::Array(params))
}

/// PERCENTILES - Calculate percentiles of data
/// PERCENTILES(data, p) where p is percentile(s) from 0 to 100
pub fn percentiles(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "PERCENTILES: Expected 2 arguments (data, percentile)".to_string(),
        ));
    }

    let mut data = extract_array(&args[0])?;
    if data.is_empty() {
        return Err(XdlError::InvalidArgument(
            "PERCENTILES: Data array is empty".to_string(),
        ));
    }

    // Sort data
    data.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = data.len();

    // Get percentile values
    let percentile_vals = extract_array(&args[1])?;

    let mut results = Vec::with_capacity(percentile_vals.len());

    for p in percentile_vals {
        let p = p.clamp(0.0, 100.0);
        let idx = (p / 100.0) * (n - 1) as f64;
        let lower = idx.floor() as usize;
        let upper = (lower + 1).min(n - 1);
        let frac = idx - lower as f64;

        let value = data[lower] * (1.0 - frac) + data[upper] * frac;
        results.push(value);
    }

    if results.len() == 1 {
        Ok(XdlValue::Double(results[0]))
    } else {
        Ok(XdlValue::Array(results))
    }
}

/// ROBUST_MEAN - Compute mean with outlier rejection
/// ROBUST_MEAN(data, sigma_clip)
pub fn robust_mean(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "ROBUST_MEAN: Expected at least 1 argument".to_string(),
        ));
    }

    let data = extract_array(&args[0])?;
    if data.is_empty() {
        return Err(XdlError::InvalidArgument(
            "ROBUST_MEAN: Data array is empty".to_string(),
        ));
    }

    let sigma_clip = if args.len() > 1 {
        args[1].to_double().unwrap_or(3.0)
    } else {
        3.0 // Default 3-sigma clipping
    };

    // Iterative sigma-clipping
    let mut filtered_data = data.clone();
    let max_iter = 10;

    for _ in 0..max_iter {
        if filtered_data.is_empty() {
            break;
        }

        let mean: f64 = filtered_data.iter().sum::<f64>() / filtered_data.len() as f64;
        let variance: f64 = filtered_data.iter().map(|x| (x - mean).powi(2)).sum::<f64>()
            / (filtered_data.len() - 1).max(1) as f64;
        let std_dev = variance.sqrt();

        let threshold = sigma_clip * std_dev;
        let old_len = filtered_data.len();
        filtered_data.retain(|&x| (x - mean).abs() <= threshold);

        if filtered_data.len() == old_len {
            break; // No more outliers removed
        }
    }

    if filtered_data.is_empty() {
        // Fall back to median if all points rejected
        let mut sorted = data.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = sorted[sorted.len() / 2];
        Ok(XdlValue::Double(median))
    } else {
        let mean: f64 = filtered_data.iter().sum::<f64>() / filtered_data.len() as f64;
        Ok(XdlValue::Double(mean))
    }
}

/// TRIMMED_MEAN - Mean with percentage of extreme values removed
/// TRIMMED_MEAN(data, percent)
pub fn trimmed_mean(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "TRIMMED_MEAN: Expected at least 1 argument".to_string(),
        ));
    }

    let mut data = extract_array(&args[0])?;
    if data.is_empty() {
        return Err(XdlError::InvalidArgument(
            "TRIMMED_MEAN: Data array is empty".to_string(),
        ));
    }

    let percent = if args.len() > 1 {
        args[1].to_double().unwrap_or(5.0).clamp(0.0, 49.0)
    } else {
        5.0 // Default 5% trimmed mean
    };

    data.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let n = data.len();
    let trim_count = ((percent / 100.0) * n as f64).ceil() as usize;

    if 2 * trim_count >= n {
        // Just return median
        return Ok(XdlValue::Double(data[n / 2]));
    }

    let trimmed = &data[trim_count..n - trim_count];
    let mean: f64 = trimmed.iter().sum::<f64>() / trimmed.len() as f64;

    Ok(XdlValue::Double(mean))
}

/// RESISTANT_MEAN - Compute resistant mean using median-based estimate
pub fn resistant_mean(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "RESISTANT_MEAN: Expected at least 1 argument".to_string(),
        ));
    }

    let mut data = extract_array(&args[0])?;
    if data.is_empty() {
        return Err(XdlError::InvalidArgument(
            "RESISTANT_MEAN: Data array is empty".to_string(),
        ));
    }

    // Compute biweight mean (Tukey's biweight)
    data.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = data[data.len() / 2];

    // Compute MAD (median absolute deviation)
    let mut abs_devs: Vec<f64> = data.iter().map(|x| (x - median).abs()).collect();
    abs_devs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mad = abs_devs[abs_devs.len() / 2];

    if mad < 1e-15 {
        return Ok(XdlValue::Double(median));
    }

    // Compute biweight mean
    let c = 6.0; // Tuning constant
    let mut sum_weights = 0.0;
    let mut weighted_sum = 0.0;

    for &x in &data {
        let u = (x - median) / (c * mad);
        if u.abs() < 1.0 {
            let w = (1.0 - u * u).powi(2);
            sum_weights += w;
            weighted_sum += w * x;
        }
    }

    let result = if sum_weights > 0.0 {
        weighted_sum / sum_weights
    } else {
        median
    };

    Ok(XdlValue::Double(result))
}

/// RANDOM_POISSON - Generate Poisson distributed random numbers
pub fn random_poisson(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "RANDOM_POISSON: Expected at least 1 argument (lambda)".to_string(),
        ));
    }

    let lambda = args[0].to_double()?;
    if lambda <= 0.0 {
        return Err(XdlError::InvalidArgument(
            "RANDOM_POISSON: lambda must be positive".to_string(),
        ));
    }

    let n = if args.len() > 1 {
        args[1].to_long()? as usize
    } else {
        1
    };

    let mut result = Vec::with_capacity(n);

    // Use Knuth's algorithm for small lambda, or normal approximation for large lambda
    // Simple deterministic seed for reproducibility
    let mut seed: u64 = 12345;
    let lcg = |s: &mut u64| -> f64 {
        *s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
        (*s as f64) / (u64::MAX as f64)
    };

    for _ in 0..n {
        let value: f64 = if lambda < 30.0 {
            // Knuth's algorithm
            let l = (-lambda).exp();
            let mut k = 0i32;
            let mut p = 1.0f64;
            loop {
                k += 1;
                p *= lcg(&mut seed);
                if p <= l {
                    break;
                }
            }
            (k - 1) as f64
        } else {
            // Normal approximation using Box-Muller transform
            let u1: f64 = lcg(&mut seed);
            let u2: f64 = lcg(&mut seed);
            let z = ((-2.0f64 * u1.ln()).sqrt()) * (2.0 * std::f64::consts::PI * u2).cos();
            (lambda + (lambda as f64).sqrt() * z).round().max(0.0)
        };
        result.push(value);
    }

    if result.len() == 1 {
        Ok(XdlValue::Double(result[0]))
    } else {
        Ok(XdlValue::Array(result))
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
}
