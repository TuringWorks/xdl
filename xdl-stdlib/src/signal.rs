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
        let start = if i < half { 0 } else { i - half };
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

    for i in 0..n {
        let mut sum = 0.0;
        for j in 0..k {
            let idx = (i + j).wrapping_sub(half_k);
            if idx < n {
                sum += signal[idx] * kernel[j];
            }
        }
        result[i] = sum;
    }

    Ok(XdlValue::Array(result))
}

/// MEDIAN filter - Apply median filter to remove noise
/// MEDIAN(array, width)
pub fn median_filter(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "MEDIAN: Expected array and width".to_string(),
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
        _ => 3,
    };

    if width == 0 || width > data.len() {
        return Ok(XdlValue::Array(data.clone()));
    }

    let n = data.len();
    let half = width / 2;
    let mut result = Vec::with_capacity(n);

    for i in 0..n {
        let start = if i < half { 0 } else { i - half };
        let end = (i + half + 1).min(n);

        let mut window: Vec<f64> = data[start..end].to_vec();
        window.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mid = window.len() / 2;
        result.push(window[mid]);
    }

    Ok(XdlValue::Array(result))
}
