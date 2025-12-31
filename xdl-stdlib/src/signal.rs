//! Signal processing functions

use xdl_core::{XdlError, XdlResult, XdlValue};

/// A_CORRELATE - Auto-correlation function
/// A_CORRELATE(array [, lag])
pub fn a_correlate(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "A_CORRELATE: Expected array argument".to_string(),
        ));
    }

    let data = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let lag = if args.len() > 1 {
        match &args[1] {
            XdlValue::Long(n) => *n as usize,
            XdlValue::Int(n) => *n as usize,
            _ => data.len() / 2,
        }
    } else {
        data.len() / 2
    };

    // Compute auto-correlation
    let n = data.len();
    let mean: f64 = data.iter().sum::<f64>() / n as f64;

    // Compute variance
    let variance: f64 = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n as f64;

    if variance == 0.0 {
        return Ok(XdlValue::Array(vec![1.0; lag.min(n)]));
    }

    let mut result = Vec::new();
    for k in 0..lag.min(n) {
        let mut sum = 0.0;
        for i in 0..(n - k) {
            sum += (data[i] - mean) * (data[i + k] - mean);
        }
        result.push(sum / ((n - k) as f64 * variance));
    }

    Ok(XdlValue::Array(result))
}

/// C_CORRELATE - Cross-correlation of two arrays
/// C_CORRELATE(array1, array2 [, lag])
pub fn c_correlate(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "C_CORRELATE: Expected 2 array arguments".to_string(),
        ));
    }

    let arr1 = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let arr2 = match &args[1] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let n = arr1.len().min(arr2.len());
    let lag = if args.len() > 2 {
        match &args[2] {
            XdlValue::Long(l) => *l as usize,
            XdlValue::Int(l) => *l as usize,
            _ => n / 2,
        }
    } else {
        n / 2
    };

    // Compute means
    let mean1: f64 = arr1.iter().take(n).sum::<f64>() / n as f64;
    let mean2: f64 = arr2.iter().take(n).sum::<f64>() / n as f64;

    // Compute cross-correlation
    let mut result = Vec::new();
    for k in 0..lag.min(n) {
        let mut sum = 0.0;
        for i in 0..(n - k) {
            sum += (arr1[i] - mean1) * (arr2[i + k] - mean2);
        }
        result.push(sum / (n - k) as f64);
    }

    Ok(XdlValue::Array(result))
}

/// SMOOTH - Smooth data with boxcar average
/// SMOOTH(array, width [, /EDGE_TRUNCATE])
pub fn smooth(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "SMOOTH: Expected array and width".to_string(),
        ));
    }

    let data = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let width = match &args[1] {
        XdlValue::Long(n) => *n as usize,
        XdlValue::Int(n) => *n as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    if width == 0 || width > data.len() {
        return Ok(XdlValue::Array(data.clone()));
    }

    let n = data.len();
    let half = width / 2;
    let mut result = Vec::with_capacity(n);

    for i in 0..n {
        let start = i.saturating_sub(half);
        let end = (i + half + 1).min(n);
        let sum: f64 = data[start..end].iter().sum();
        result.push(sum / (end - start) as f64);
    }

    Ok(XdlValue::Array(result))
}

/// DIGITAL_FILTER - Design and apply digital filter
/// DIGITAL_FILTER(cutoff_freq1, cutoff_freq2, a, b)
/// Simplified Butterworth-style filter
pub fn digital_filter(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "DIGITAL_FILTER: Expected cutoff frequencies".to_string(),
        ));
    }

    let f1 = match &args[0] {
        XdlValue::Double(v) => *v,
        XdlValue::Float(v) => *v as f64,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let f2 = match &args[1] {
        XdlValue::Double(v) => *v,
        XdlValue::Float(v) => *v as f64,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    // Simple filter coefficients (placeholder for full implementation)
    let a = vec![1.0, -0.5 * f1.cos(), 0.25 * (f1 + f2)];
    let b = vec![f1 * 0.1, f2 * 0.1];

    // Return filter coefficients as nested array
    let result = vec![XdlValue::Array(a), XdlValue::Array(b)];

    Ok(XdlValue::NestedArray(result))
}

/// HILBERT - Hilbert transform
/// HILBERT(array)
pub fn hilbert(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "HILBERT: Expected array argument".to_string(),
        ));
    }

    let data = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    // Simplified Hilbert transform using FFT approach (placeholder)
    // In full implementation, would use FFT, multiply by -i*sgn(f), then IFFT
    let n = data.len();
    let mut result = Vec::with_capacity(n);

    // Simple approximation: 90-degree phase shift
    for i in 0..n {
        let idx_prev = if i > 0 { i - 1 } else { n - 1 };
        let idx_next = if i < n - 1 { i + 1 } else { 0 };
        result.push((data[idx_next] - data[idx_prev]) * 0.5);
    }

    Ok(XdlValue::Array(result))
}

/// CONVOL - 1D convolution (for signal processing)
/// CONVOL(signal, kernel)
pub fn convol_1d(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "CONVOL: Expected signal and kernel".to_string(),
        ));
    }

    let signal = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let kernel = match &args[1] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let n = signal.len();
    let k = kernel.len();
    let half_k = k / 2;

    let mut result = vec![0.0; n];

    for (i, result_item) in result.iter_mut().enumerate().take(n) {
        let mut sum = 0.0;
        for (j, &kernel_item) in kernel.iter().enumerate().take(k) {
            let idx = (i + j).wrapping_sub(half_k);
            if idx < n {
                sum += signal[idx] * kernel_item;
            }
        }
        *result_item = sum;
    }

    Ok(XdlValue::Array(result))
}

/// MEDIAN filter - Apply median filter to remove noise
/// MEDIAN(array, width)
pub fn median_filter(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "MEDIAN: Expected array and optional width".to_string(),
        ));
    }

    let data = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let width = if args.len() > 1 {
        match &args[1] {
            XdlValue::Long(n) => *n as usize,
            XdlValue::Int(n) => *n as usize,
            _ => 3,
        }
    } else {
        3
    };

    if width == 0 || width > data.len() {
        return Ok(XdlValue::Array(data.clone()));
    }

    let n = data.len();
    let half = width / 2;
    let mut result = Vec::with_capacity(n);

    for i in 0..n {
        let start = i.saturating_sub(half);
        let end = (i + half + 1).min(n);

        let mut window: Vec<f64> = data[start..end].to_vec();
        window.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mid = window.len() / 2;
        result.push(window[mid]);
    }

    Ok(XdlValue::Array(result))
}

/// FFT_2D - 2D Fast Fourier Transform
/// FFT_2D(array_2d [, direction])
/// direction: 1 for forward (default), -1 for inverse
pub fn fft_2d(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "FFT_2D: Expected 2D array argument".to_string(),
        ));
    }

    let data = match &args[0] {
        XdlValue::NestedArray(rows) => rows,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "2D array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let direction = if args.len() > 1 {
        match &args[1] {
            XdlValue::Long(n) => *n,
            XdlValue::Int(n) => *n as i32,
            _ => 1,
        }
    } else {
        1
    };

    let forward = direction >= 0;

    // Convert to 2D f64 array
    let rows: Vec<Vec<f64>> = data
        .iter()
        .filter_map(|row| {
            if let XdlValue::Array(arr) = row {
                Some(arr.clone())
            } else {
                None
            }
        })
        .collect();

    if rows.is_empty() {
        return Err(XdlError::InvalidArgument("FFT_2D: Empty array".to_string()));
    }

    let n_rows = rows.len();
    let n_cols = rows[0].len();

    // Simple 2D DFT (not optimized, for demonstration)
    let pi = std::f64::consts::PI;
    let mut result_real = vec![vec![0.0; n_cols]; n_rows];
    let mut result_imag = vec![vec![0.0; n_cols]; n_rows];

    // Row-wise transform
    for (i, row) in rows.iter().enumerate() {
        for k in 0..n_cols {
            let mut sum_real = 0.0;
            let mut sum_imag = 0.0;
            for (n, &x) in row.iter().enumerate() {
                let angle = 2.0 * pi * (k as f64) * (n as f64) / (n_cols as f64);
                if forward {
                    sum_real += x * angle.cos();
                    sum_imag -= x * angle.sin();
                } else {
                    sum_real += x * angle.cos();
                    sum_imag += x * angle.sin();
                }
            }
            result_real[i][k] = sum_real;
            result_imag[i][k] = sum_imag;
        }
    }

    // Column-wise transform
    let mut final_real = vec![vec![0.0; n_cols]; n_rows];
    let mut final_imag = vec![vec![0.0; n_cols]; n_rows];

    for j in 0..n_cols {
        for k in 0..n_rows {
            let mut sum_real = 0.0;
            let mut sum_imag = 0.0;
            for n in 0..n_rows {
                let angle = 2.0 * pi * (k as f64) * (n as f64) / (n_rows as f64);
                let (cos_a, sin_a) = (angle.cos(), angle.sin());
                if forward {
                    sum_real += result_real[n][j] * cos_a + result_imag[n][j] * sin_a;
                    sum_imag += result_imag[n][j] * cos_a - result_real[n][j] * sin_a;
                } else {
                    sum_real += result_real[n][j] * cos_a - result_imag[n][j] * sin_a;
                    sum_imag += result_imag[n][j] * cos_a + result_real[n][j] * sin_a;
                }
            }
            if !forward {
                sum_real /= (n_rows * n_cols) as f64;
                sum_imag /= (n_rows * n_cols) as f64;
            }
            final_real[k][j] = sum_real;
            final_imag[k][j] = sum_imag;
        }
    }

    // Return magnitude (sqrt(real^2 + imag^2))
    let result: Vec<XdlValue> = final_real
        .iter()
        .zip(final_imag.iter())
        .map(|(r_row, i_row)| {
            XdlValue::Array(
                r_row
                    .iter()
                    .zip(i_row.iter())
                    .map(|(&r, &i)| (r * r + i * i).sqrt())
                    .collect(),
            )
        })
        .collect();

    Ok(XdlValue::NestedArray(result))
}

/// HANNING - Create Hanning window
/// HANNING(n [, alpha])
pub fn hanning(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "HANNING: Expected window size".to_string(),
        ));
    }

    let n = match &args[0] {
        XdlValue::Long(v) => *v as usize,
        XdlValue::Int(v) => *v as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let alpha = if args.len() > 1 {
        match &args[1] {
            XdlValue::Double(v) => *v,
            XdlValue::Float(v) => *v as f64,
            _ => 0.5,
        }
    } else {
        0.5
    };

    let pi = std::f64::consts::PI;
    let result: Vec<f64> = (0..n)
        .map(|i| alpha - (1.0 - alpha) * (2.0 * pi * i as f64 / (n - 1) as f64).cos())
        .collect();

    Ok(XdlValue::Array(result))
}

/// HAMMING - Create Hamming window
/// HAMMING(n)
pub fn hamming(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "HAMMING: Expected window size".to_string(),
        ));
    }

    let n = match &args[0] {
        XdlValue::Long(v) => *v as usize,
        XdlValue::Int(v) => *v as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let pi = std::f64::consts::PI;
    let result: Vec<f64> = (0..n)
        .map(|i| 0.54 - 0.46 * (2.0 * pi * i as f64 / (n - 1) as f64).cos())
        .collect();

    Ok(XdlValue::Array(result))
}

/// BLACKMAN - Create Blackman window
/// BLACKMAN(n [, alpha])
pub fn blackman(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "BLACKMAN: Expected window size".to_string(),
        ));
    }

    let n = match &args[0] {
        XdlValue::Long(v) => *v as usize,
        XdlValue::Int(v) => *v as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let alpha = if args.len() > 1 {
        match &args[1] {
            XdlValue::Double(v) => *v,
            XdlValue::Float(v) => *v as f64,
            _ => 0.16,
        }
    } else {
        0.16
    };

    let a0 = (1.0 - alpha) / 2.0;
    let a1 = 0.5;
    let a2 = alpha / 2.0;
    let pi = std::f64::consts::PI;

    let result: Vec<f64> = (0..n)
        .map(|i| {
            let t = 2.0 * pi * i as f64 / (n - 1) as f64;
            a0 - a1 * t.cos() + a2 * (2.0 * t).cos()
        })
        .collect();

    Ok(XdlValue::Array(result))
}

/// BUTTERWORTH - Butterworth filter coefficients
/// BUTTERWORTH(order, cutoff)
pub fn butterworth(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "BUTTERWORTH: Expected order and cutoff frequency".to_string(),
        ));
    }

    let order = match &args[0] {
        XdlValue::Long(v) => *v as usize,
        XdlValue::Int(v) => *v as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let cutoff = match &args[1] {
        XdlValue::Double(v) => *v,
        XdlValue::Float(v) => *v as f64,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    // Butterworth polynomial coefficients (simplified)
    // For a normalized Butterworth filter
    let pi = std::f64::consts::PI;
    let wc = (pi * cutoff).tan(); // Pre-warped cutoff frequency

    // Calculate poles in s-plane
    let mut a_coeffs = vec![1.0];

    for k in 0..order {
        let angle = pi * (2 * k + order + 1) as f64 / (2 * order) as f64;
        let pole_real = wc * angle.cos();
        let pole_imag = wc * angle.sin();

        // For real poles (when angle is 0 or pi)
        if pole_imag.abs() < 1e-10 {
            let new_a = vec![1.0, -pole_real];
            a_coeffs = convolve_poly(&a_coeffs, &new_a);
        } else if k < order / 2 {
            // Complex conjugate pair
            let new_a = vec![1.0, -2.0 * pole_real, pole_real * pole_real + pole_imag * pole_imag];
            a_coeffs = convolve_poly(&a_coeffs, &new_a);
        }
    }

    // Normalize gain
    let gain: f64 = wc.powi(order as i32);
    let b_coeffs = vec![gain];

    Ok(XdlValue::NestedArray(vec![
        XdlValue::Array(a_coeffs),
        XdlValue::Array(b_coeffs),
    ]))
}

/// Helper function to convolve two polynomials
fn convolve_poly(a: &[f64], b: &[f64]) -> Vec<f64> {
    let n = a.len() + b.len() - 1;
    let mut result = vec![0.0; n];
    for (i, &ai) in a.iter().enumerate() {
        for (j, &bj) in b.iter().enumerate() {
            result[i + j] += ai * bj;
        }
    }
    result
}

/// SAVGOL - Savitzky-Golay smoothing filter
/// SAVGOL(width, degree [, derivative])
pub fn savgol(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "SAVGOL: Expected width and degree".to_string(),
        ));
    }

    let width = match &args[0] {
        XdlValue::Long(v) => *v as usize,
        XdlValue::Int(v) => *v as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let degree = match &args[1] {
        XdlValue::Long(v) => *v as usize,
        XdlValue::Int(v) => *v as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let derivative = if args.len() > 2 {
        match &args[2] {
            XdlValue::Long(v) => *v as usize,
            XdlValue::Int(v) => *v as usize,
            _ => 0,
        }
    } else {
        0
    };

    if width % 2 == 0 {
        return Err(XdlError::InvalidArgument(
            "SAVGOL: Width must be odd".to_string(),
        ));
    }

    if degree >= width {
        return Err(XdlError::InvalidArgument(
            "SAVGOL: Degree must be less than width".to_string(),
        ));
    }

    let half = (width / 2) as i32;
    let m = degree + 1;

    // Build the Vandermonde matrix
    let mut mat = vec![vec![0.0; m]; width];
    for i in 0..width {
        let x = (i as i32 - half) as f64;
        for j in 0..m {
            mat[i][j] = x.powi(j as i32);
        }
    }

    // Compute (A^T A)^(-1) A^T using simple least squares
    // For simplicity, we compute the smoothing coefficients directly
    let coeffs = compute_savgol_coeffs(width, degree, derivative);

    Ok(XdlValue::Array(coeffs))
}

/// Compute Savitzky-Golay filter coefficients
fn compute_savgol_coeffs(width: usize, degree: usize, derivative: usize) -> Vec<f64> {
    let half = (width / 2) as i32;
    let m = degree + 1;

    // Build Vandermonde matrix
    let mut mat = vec![vec![0.0; m]; width];
    for i in 0..width {
        let x = (i as i32 - half) as f64;
        for j in 0..m {
            mat[i][j] = x.powi(j as i32);
        }
    }

    // Compute A^T A
    let mut ata = vec![vec![0.0; m]; m];
    for i in 0..m {
        for j in 0..m {
            for k in 0..width {
                ata[i][j] += mat[k][i] * mat[k][j];
            }
        }
    }

    // Simple matrix inversion (Gauss-Jordan) for small matrices
    let mut aug = vec![vec![0.0; 2 * m]; m];
    for i in 0..m {
        for j in 0..m {
            aug[i][j] = ata[i][j];
        }
        aug[i][m + i] = 1.0;
    }

    for i in 0..m {
        let pivot = aug[i][i];
        if pivot.abs() < 1e-10 {
            continue;
        }
        for j in 0..2 * m {
            aug[i][j] /= pivot;
        }
        for k in 0..m {
            if k != i {
                let factor = aug[k][i];
                for j in 0..2 * m {
                    aug[k][j] -= factor * aug[i][j];
                }
            }
        }
    }

    // Extract inverse
    let mut ata_inv = vec![vec![0.0; m]; m];
    for i in 0..m {
        for j in 0..m {
            ata_inv[i][j] = aug[i][m + j];
        }
    }

    // Compute (A^T A)^(-1) A^T
    let mut pinv = vec![vec![0.0; width]; m];
    for i in 0..m {
        for j in 0..width {
            for k in 0..m {
                pinv[i][j] += ata_inv[i][k] * mat[j][k];
            }
        }
    }

    // Get the row corresponding to the derivative
    let fact: f64 = (1..=derivative).map(|i| i as f64).product::<f64>();
    let deriv_row = derivative.min(m - 1);

    pinv[deriv_row].iter().map(|&v| v * fact).collect()
}

/// LEEFILT - Lee filter for speckle noise reduction
/// LEEFILT(array, width)
pub fn leefilt(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "LEEFILT: Expected array and window size".to_string(),
        ));
    }

    let data = match &args[0] {
        XdlValue::Array(arr) => arr.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let width = match &args[1] {
        XdlValue::Long(v) => *v as usize,
        XdlValue::Int(v) => *v as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let n = data.len();
    let half = width / 2;
    let mut result = vec![0.0; n];

    // Estimate noise variance (using median absolute deviation)
    let mut sorted = data.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = sorted[n / 2];
    let mad: f64 = sorted.iter().map(|&x| (x - median).abs()).sum::<f64>() / n as f64;
    let noise_var = (mad * 1.4826).powi(2);

    for i in 0..n {
        let start = i.saturating_sub(half);
        let end = (i + half + 1).min(n);
        let window: Vec<f64> = data[start..end].to_vec();

        let local_mean: f64 = window.iter().sum::<f64>() / window.len() as f64;
        let local_var: f64 =
            window.iter().map(|&x| (x - local_mean).powi(2)).sum::<f64>() / window.len() as f64;

        // Lee filter formula
        let weight = if local_var > noise_var {
            1.0 - noise_var / local_var
        } else {
            0.0
        };

        result[i] = local_mean + weight * (data[i] - local_mean);
    }

    Ok(XdlValue::Array(result))
}

/// WV_HAAR - Haar wavelet transform
/// WV_HAAR(array)
pub fn wv_haar(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "WV_HAAR: Expected array argument".to_string(),
        ));
    }

    let data = match &args[0] {
        XdlValue::Array(arr) => arr.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let n = data.len();
    if n < 2 {
        return Ok(XdlValue::Array(data));
    }

    // Pad to power of 2 if needed
    let next_pow2 = n.next_power_of_two();
    let mut signal = data;
    signal.resize(next_pow2, 0.0);

    let sqrt2 = std::f64::consts::SQRT_2;
    let mut output = signal.clone();
    let mut len = next_pow2;

    while len >= 2 {
        let half = len / 2;
        let mut temp = vec![0.0; len];

        for i in 0..half {
            temp[i] = (output[2 * i] + output[2 * i + 1]) / sqrt2;
            temp[half + i] = (output[2 * i] - output[2 * i + 1]) / sqrt2;
        }

        for i in 0..len {
            output[i] = temp[i];
        }

        len = half;
    }

    Ok(XdlValue::Array(output))
}

/// WV_IHAAR - Inverse Haar wavelet transform
/// WV_IHAAR(array)
pub fn wv_ihaar(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "WV_IHAAR: Expected array argument".to_string(),
        ));
    }

    let data = match &args[0] {
        XdlValue::Array(arr) => arr.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let n = data.len();
    if n < 2 {
        return Ok(XdlValue::Array(data));
    }

    let sqrt2 = std::f64::consts::SQRT_2;
    let mut output = data;
    let mut len = 2;

    while len <= n {
        let half = len / 2;
        let mut temp = vec![0.0; len];

        for i in 0..half {
            temp[2 * i] = (output[i] + output[half + i]) / sqrt2;
            temp[2 * i + 1] = (output[i] - output[half + i]) / sqrt2;
        }

        for i in 0..len {
            output[i] = temp[i];
        }

        len *= 2;
    }

    Ok(XdlValue::Array(output))
}

/// WV_DWT - Discrete Wavelet Transform using Daubechies-4
/// WV_DWT(array [, level])
pub fn wv_dwt(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "WV_DWT: Expected array argument".to_string(),
        ));
    }

    let data = match &args[0] {
        XdlValue::Array(arr) => arr.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let level = if args.len() > 1 {
        match &args[1] {
            XdlValue::Long(v) => *v as usize,
            XdlValue::Int(v) => *v as usize,
            _ => 1,
        }
    } else {
        1
    };

    // Daubechies-4 coefficients
    let h0 = (1.0 + 3.0_f64.sqrt()) / (4.0 * 2.0_f64.sqrt());
    let h1 = (3.0 + 3.0_f64.sqrt()) / (4.0 * 2.0_f64.sqrt());
    let h2 = (3.0 - 3.0_f64.sqrt()) / (4.0 * 2.0_f64.sqrt());
    let h3 = (1.0 - 3.0_f64.sqrt()) / (4.0 * 2.0_f64.sqrt());

    let lowpass = vec![h0, h1, h2, h3];
    let highpass = vec![h3, -h2, h1, -h0];

    let mut signal = data;
    let mut all_details = Vec::new();

    for _ in 0..level {
        if signal.len() < 4 {
            break;
        }

        let n = signal.len();
        let half = n / 2;
        let mut approx = vec![0.0; half];
        let mut detail = vec![0.0; half];

        for i in 0..half {
            for (j, (&lp, &hp)) in lowpass.iter().zip(highpass.iter()).enumerate() {
                let idx = (2 * i + j) % n;
                approx[i] += signal[idx] * lp;
                detail[i] += signal[idx] * hp;
            }
        }

        all_details.push(XdlValue::Array(detail));
        signal = approx;
    }

    // Return approximation coefficients and detail coefficients
    let mut result = vec![XdlValue::Array(signal)];
    result.extend(all_details);

    Ok(XdlValue::NestedArray(result))
}

/// POWER_SPECTRUM - Compute power spectral density
/// POWER_SPECTRUM(array [, window])
pub fn power_spectrum(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "POWER_SPECTRUM: Expected array argument".to_string(),
        ));
    }

    let data = match &args[0] {
        XdlValue::Array(arr) => arr.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let n = data.len();
    let pi = std::f64::consts::PI;

    // Apply Hanning window if specified
    let windowed: Vec<f64> = if args.len() > 1 {
        let window = match &args[1] {
            XdlValue::Array(w) => w.clone(),
            _ => (0..n)
                .map(|i| 0.5 - 0.5 * (2.0 * pi * i as f64 / (n - 1) as f64).cos())
                .collect(),
        };
        data.iter()
            .zip(window.iter())
            .map(|(&d, &w)| d * w)
            .collect()
    } else {
        data.clone()
    };

    // Compute DFT and power spectrum
    let mut power = vec![0.0; n / 2 + 1];

    for k in 0..=n / 2 {
        let mut real = 0.0;
        let mut imag = 0.0;
        for (i, &x) in windowed.iter().enumerate() {
            let angle = 2.0 * pi * k as f64 * i as f64 / n as f64;
            real += x * angle.cos();
            imag -= x * angle.sin();
        }
        power[k] = (real * real + imag * imag) / n as f64;
    }

    Ok(XdlValue::Array(power))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_median_filter_basic() {
        let data = vec![1.0, 3.0, 2.0, 5.0, 4.0];
        let args = vec![XdlValue::Array(data.clone()), XdlValue::Long(3)];
        let result = median_filter(&args);
        assert!(result.is_ok());
        match result.unwrap() {
            XdlValue::Array(arr) => {
                // For window size 3, should smooth the data
                assert_eq!(arr.len(), 5);
                // Check that all values are from the original data (medians)
                for &val in &arr {
                    assert!(data.contains(&val));
                }
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_median_filter_width_1() {
        let data = vec![1.0, 2.0, 3.0];
        let args = vec![XdlValue::Array(data.clone()), XdlValue::Long(1)];
        let result = median_filter(&args);
        assert!(result.is_ok());
        match result.unwrap() {
            XdlValue::Array(arr) => {
                // Width 1 should return original data
                assert_eq!(arr, data);
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_median_filter_no_width() {
        let data = vec![1.0, 3.0, 2.0];
        let args = vec![XdlValue::Array(data.clone())];
        let result = median_filter(&args);
        assert!(result.is_ok());
        match result.unwrap() {
            XdlValue::Array(arr) => {
                // Default width 3 should work
                assert_eq!(arr.len(), 3);
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_a_correlate_basic() {
        let signal = vec![1.0, 2.0, 3.0];
        let args = vec![XdlValue::Array(signal)];
        let result = a_correlate(&args);
        assert!(result.is_ok());
        match result.unwrap() {
            XdlValue::Array(arr) => {
                assert_eq!(arr.len(), 1); // Default lag = n/2 = 1
                assert!(arr[0] >= -1.0 && arr[0] <= 1.0); // Correlation coefficient
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_c_correlate_basic() {
        let signal1 = vec![1.0, 2.0, 3.0];
        let signal2 = vec![1.0, 0.0, 0.0];
        let args = vec![XdlValue::Array(signal1), XdlValue::Array(signal2)];
        let result = c_correlate(&args);
        assert!(result.is_ok());
        match result.unwrap() {
            XdlValue::Array(arr) => {
                assert_eq!(arr.len(), 1); // Default lag = min(n1,n2)/2 = 1
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_smooth_basic() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let args = vec![XdlValue::Array(data), XdlValue::Long(3)];
        let result = smooth(&args);
        assert!(result.is_ok());
        match result.unwrap() {
            XdlValue::Array(arr) => {
                assert_eq!(arr.len(), 5);
                // Check that smoothing occurred (edges should be different)
                assert!(arr[0] != 1.0 || arr[4] != 5.0);
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_hilbert_transform() {
        let data = vec![1.0, 0.0, -1.0, 0.0];
        let args = vec![XdlValue::Array(data)];
        let result = hilbert(&args);
        assert!(result.is_ok());
        match result.unwrap() {
            XdlValue::Array(arr) => {
                assert_eq!(arr.len(), 4);
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_convol_1d_basic() {
        let data = vec![1.0, 2.0, 3.0];
        let kernel = vec![0.5, 0.5];
        let args = vec![XdlValue::Array(data), XdlValue::Array(kernel)];
        let result = convol_1d(&args);
        assert!(result.is_ok());
        match result.unwrap() {
            XdlValue::Array(arr) => {
                assert_eq!(arr.len(), 3);
            }
            _ => panic!("Expected array"),
        }
    }
}
