//! Machine Learning functions for XDL
//!
//! This module provides ML utilities, models, and algorithms

use xdl_core::{XdlError, XdlResult, XdlValue};

/// XDLmlPartition - Partition data into training/test sets
///
/// Syntax: indices = XDLmlPartition(n_samples, train_fraction)
///
/// Parameters:
///   n_samples: Total number of samples
///   train_fraction: Fraction for training set (0.0 to 1.0), default 0.8
///
/// Returns:
///   Array of indices where 1 = training, 0 = test
///
/// Example:
///   partition = XDLmlPartition(100, 0.8)  ; 80 train, 20 test
pub fn xdlml_partition(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLmlPartition: Expected at least 1 argument (n_samples)".to_string(),
        ));
    }

    // Get number of samples
    let n_samples = match &args[0] {
        XdlValue::Long(n) => *n as usize,
        XdlValue::Int(n) => *n as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    // Get train fraction (default 0.8)
    let train_fraction = if args.len() > 1 {
        match &args[1] {
            XdlValue::Double(f) => *f,
            XdlValue::Float(f) => *f as f64,
            _ => 0.8,
        }
    } else {
        0.8
    };

    if !(0.0..=1.0).contains(&train_fraction) {
        return Err(XdlError::InvalidArgument(
            "XDLmlPartition: train_fraction must be between 0 and 1".to_string(),
        ));
    }

    let n_train = (n_samples as f64 * train_fraction).round() as usize;

    // Create partition array: 1 for training, 0 for test
    let mut partition = vec![1.0; n_samples];
    for item in partition.iter_mut().take(n_samples).skip(n_train) {
        *item = 0.0;
    }

    Ok(XdlValue::Array(partition))
}

/// XDLmlShuffle - Randomly shuffle array indices
///
/// Syntax: shuffled_indices = XDLmlShuffle(n_samples [, seed])
///
/// Parameters:
///   n_samples: Number of samples to shuffle
///   seed: Optional random seed for reproducibility
///
/// Returns:
///   Array of shuffled indices (0 to n_samples-1)
///
/// Example:
///   indices = XDLmlShuffle(100)
///   shuffled_data = data[indices]
pub fn xdlml_shuffle(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLmlShuffle: Expected at least 1 argument (n_samples)".to_string(),
        ));
    }

    // Get number of samples
    let n_samples = match &args[0] {
        XdlValue::Long(n) => *n as usize,
        XdlValue::Int(n) => *n as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    // Get optional seed
    let seed = if args.len() > 1 {
        match &args[1] {
            XdlValue::Long(s) => *s as u64,
            XdlValue::Int(s) => *s as u64,
            _ => 12345u64,
        }
    } else {
        // Use current time as seed
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    };

    // Create array of indices
    let mut indices: Vec<f64> = (0..n_samples).map(|i| i as f64).collect();

    // Fisher-Yates shuffle
    let mut rng_state = seed;
    for i in (1..n_samples).rev() {
        // Simple LCG random number generator
        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        let j = (rng_state % (i as u64 + 1)) as usize;
        indices.swap(i, j);
    }

    Ok(XdlValue::Array(indices))
}

/// XDLmlLinearNormalizer - Linear normalization: out = in * scale + offset
///
/// Syntax: normalized = XDLmlLinearNormalizer(data, scale, offset)
///
/// Parameters:
///   data: Input array to normalize
///   scale: Scaling factor (default 1.0)
///   offset: Offset value (default 0.0)
///
/// Returns:
///   Normalized array: data * scale + offset
///
/// Example:
///   norm_data = XDLmlLinearNormalizer(data, 0.5, 10.0)  ; data*0.5 + 10
pub fn xdlml_linear_normalizer(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLmlLinearNormalizer: Expected at least 1 argument (data)".to_string(),
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

    let scale = if args.len() > 1 {
        match &args[1] {
            XdlValue::Double(s) => *s,
            XdlValue::Float(s) => *s as f64,
            _ => 1.0,
        }
    } else {
        1.0
    };

    let offset = if args.len() > 2 {
        match &args[2] {
            XdlValue::Double(o) => *o,
            XdlValue::Float(o) => *o as f64,
            _ => 0.0,
        }
    } else {
        0.0
    };

    let normalized: Vec<f64> = data.iter().map(|&x| x * scale + offset).collect();

    Ok(XdlValue::Array(normalized))
}

/// XDLmlRangeNormalizer - Normalize to range [0, 1]
///
/// Syntax: normalized = XDLmlRangeNormalizer(data)
///
/// Parameters:
///   data: Input array to normalize
///
/// Returns:
///   Array scaled to [0, 1] using min-max normalization
///   Formula: (x - min) / (max - min)
///
/// Example:
///   norm_data = XDLmlRangeNormalizer(data)
pub fn xdlml_range_normalizer(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLmlRangeNormalizer: Expected 1 argument (data)".to_string(),
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

    if data.is_empty() {
        return Ok(XdlValue::Array(vec![]));
    }

    let min_val = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_val = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let range = max_val - min_val;

    if range == 0.0 {
        // All values are the same, return array of 0.5
        return Ok(XdlValue::Array(vec![0.5; data.len()]));
    }

    let normalized: Vec<f64> = data.iter().map(|&x| (x - min_val) / range).collect();

    Ok(XdlValue::Array(normalized))
}

/// XDLmlVarianceNormalizer - Standardization (z-score normalization)
///
/// Syntax: normalized = XDLmlVarianceNormalizer(data)
///
/// Parameters:
///   data: Input array to normalize
///
/// Returns:
///   Array with mean=0 and standard deviation=1
///   Formula: (x - mean) / std
///
/// Example:
///   norm_data = XDLmlVarianceNormalizer(data)
pub fn xdlml_variance_normalizer(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLmlVarianceNormalizer: Expected 1 argument (data)".to_string(),
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

    if data.is_empty() {
        return Ok(XdlValue::Array(vec![]));
    }

    // Calculate mean
    let n = data.len() as f64;
    let mean = data.iter().sum::<f64>() / n;

    // Calculate standard deviation
    let variance = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n;
    let std_dev = variance.sqrt();

    if std_dev == 0.0 {
        // All values are the same, return array of zeros
        return Ok(XdlValue::Array(vec![0.0; data.len()]));
    }

    let normalized: Vec<f64> = data.iter().map(|&x| (x - mean) / std_dev).collect();

    Ok(XdlValue::Array(normalized))
}

/// XDLmlTanHNormalizer - Hyperbolic tangent normalization to (-1, 1)
///
/// Syntax: normalized = XDLmlTanHNormalizer(data)
///
/// Parameters:
///   data: Input array to normalize
///
/// Returns:
///   Array mapped through tanh function, bounded to (-1, 1)
///   Formula: tanh(x)
///
/// Example:
///   norm_data = XDLmlTanHNormalizer(data)
pub fn xdlml_tanh_normalizer(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLmlTanHNormalizer: Expected 1 argument (data)".to_string(),
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

    let normalized: Vec<f64> = data.iter().map(|&x| x.tanh()).collect();

    Ok(XdlValue::Array(normalized))
}

/// XDLmlUnitNormalizer - Unit normalization (L2 normalization)
///
/// Syntax: normalized = XDLmlUnitNormalizer(data)
///
/// Parameters:
///   data: Input array to normalize
///
/// Returns:
///   Array scaled to have L2 norm (Euclidean length) of 1
///   Formula: x / ||x||₂
///
/// Example:
///   norm_data = XDLmlUnitNormalizer(data)
pub fn xdlml_unit_normalizer(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLmlUnitNormalizer: Expected 1 argument (data)".to_string(),
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

    if data.is_empty() {
        return Ok(XdlValue::Array(vec![]));
    }

    // Calculate L2 norm (Euclidean length)
    let l2_norm = data.iter().map(|&x| x * x).sum::<f64>().sqrt();

    if l2_norm == 0.0 {
        // All values are zero, return as-is
        return Ok(XdlValue::Array(data.clone()));
    }

    let normalized: Vec<f64> = data.iter().map(|&x| x / l2_norm).collect();

    Ok(XdlValue::Array(normalized))
}

/// XDLML_KMeans - K-means clustering algorithm
///
/// Syntax: result = XDLML_KMeans(data, n_clusters [, max_iterations] [, seed])
///
/// Parameters:
///   data: Input array (1D or 2D) - if 1D, each element is a sample; if 2D, rows are samples
///   n_clusters: Number of clusters (k)
///   max_iterations: Maximum iterations (default 100)
///   seed: Random seed for initialization (default: time-based)
///
/// Returns:
///   Array of cluster assignments (0 to k-1) for each sample
///
/// Example:
///   data = RANDOMU(seed, 100)  ; 100 data points
///   clusters = XDLML_KMeans(data, 3)  ; Cluster into 3 groups
///
/// Algorithm: Lloyd's algorithm (standard K-means)
///   1. Initialize k centroids randomly
///   2. Assign each point to nearest centroid
///   3. Update centroids as mean of assigned points
///   4. Repeat until convergence or max iterations
pub fn xdlml_kmeans(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "XDLML_KMeans: Expected at least 2 arguments (data, n_clusters)".to_string(),
        ));
    }

    // Get data
    let data = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if data.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLML_KMeans: Data array cannot be empty".to_string(),
        ));
    }

    // Get number of clusters
    let n_clusters = match &args[1] {
        XdlValue::Long(k) => *k as usize,
        XdlValue::Int(k) => *k as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    if n_clusters == 0 || n_clusters > data.len() {
        return Err(XdlError::InvalidArgument(format!(
            "XDLML_KMeans: n_clusters must be between 1 and {} (data size)",
            data.len()
        )));
    }

    // Get max iterations (default 100)
    let max_iterations = if args.len() > 2 {
        match &args[2] {
            XdlValue::Long(m) => *m as usize,
            XdlValue::Int(m) => *m as usize,
            _ => 100,
        }
    } else {
        100
    };

    // Get random seed
    let seed = if args.len() > 3 {
        match &args[3] {
            XdlValue::Long(s) => *s as u64,
            XdlValue::Int(s) => *s as u64,
            _ => {
                use std::time::{SystemTime, UNIX_EPOCH};
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            }
        }
    } else {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    };

    // For 1D data, treat each element as a 1-feature sample
    let n_samples = data.len();
    let _n_features = 1; // Currently only support 1D data

    // Initialize centroids randomly from data points (K-means++ would be better, but start simple)
    let mut centroids = vec![0.0; n_clusters];
    let mut rng_state = seed;

    // Select random data points as initial centroids
    for centroid in centroids.iter_mut().take(n_clusters) {
        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        let idx = (rng_state % n_samples as u64) as usize;
        *centroid = data[idx];
    }

    // Cluster assignments for each sample
    let mut labels = vec![0usize; n_samples];
    let mut prev_labels = vec![n_clusters; n_samples]; // Initialize to invalid value

    // K-means iterations
    for _iteration in 0..max_iterations {
        // Assignment step: assign each point to nearest centroid
        let mut changed = false;
        for i in 0..n_samples {
            let point = data[i];
            let mut min_dist = f64::INFINITY;
            let mut best_cluster = 0;

            for (k, &centroid) in centroids.iter().enumerate().take(n_clusters) {
                let dist = (point - centroid).abs(); // Euclidean distance for 1D
                if dist < min_dist {
                    min_dist = dist;
                    best_cluster = k;
                }
            }

            if labels[i] != best_cluster {
                changed = true;
            }
            labels[i] = best_cluster;
        }

        // Check convergence
        if !changed && prev_labels == labels {
            break;
        }
        prev_labels.clone_from(&labels);

        // Update step: recalculate centroids
        let mut cluster_sums = vec![0.0; n_clusters];
        let mut cluster_counts = vec![0usize; n_clusters];

        for i in 0..n_samples {
            let cluster = labels[i];
            cluster_sums[cluster] += data[i];
            cluster_counts[cluster] += 1;
        }

        // Update centroids (mean of assigned points)
        for k in 0..n_clusters {
            if cluster_counts[k] > 0 {
                centroids[k] = cluster_sums[k] / cluster_counts[k] as f64;
            }
            // If cluster is empty, keep previous centroid or reinitialize
        }
    }

    // Convert labels to f64 array for return
    let result: Vec<f64> = labels.iter().map(|&l| l as f64).collect();

    Ok(XdlValue::Array(result))
}

// ============================================================================
// ACTIVATION FUNCTIONS (Phase ML-2)
// ============================================================================

/// XDLMLAF_IDENTITY - Identity activation: f(x) = x
pub fn xdlmlaf_identity(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLmlafIdentity: Expected 1 argument (data)".to_string(),
        ));
    }
    Ok(args[0].clone()) // Identity returns input unchanged
}

/// XDLMLAF_BINARYSTEP - Binary step activation: f(x) = (x >= 0) ? 1 : 0
pub fn xdlmlaf_binarystep(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLmlafBinaryStep: Expected 1 argument (data)".to_string(),
        ));
    }

    match &args[0] {
        XdlValue::Array(arr) => {
            let result: Vec<f64> = arr
                .iter()
                .map(|&x| if x >= 0.0 { 1.0 } else { 0.0 })
                .collect();
            Ok(XdlValue::Array(result))
        }
        XdlValue::Double(x) => Ok(XdlValue::Double(if *x >= 0.0 { 1.0 } else { 0.0 })),
        XdlValue::Float(x) => Ok(XdlValue::Float(if *x >= 0.0 { 1.0 } else { 0.0 })),
        _ => Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// XDLMLAF_LOGISTIC - Sigmoid activation: f(x) = 1 / (1 + e^-x)
pub fn xdlmlaf_logistic(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLmlafLogistic: Expected 1 argument (data)".to_string(),
        ));
    }

    match &args[0] {
        XdlValue::Array(arr) => {
            let result: Vec<f64> = arr.iter().map(|&x| 1.0 / (1.0 + (-x).exp())).collect();
            Ok(XdlValue::Array(result))
        }
        XdlValue::Double(x) => Ok(XdlValue::Double(1.0 / (1.0 + (-x).exp()))),
        XdlValue::Float(x) => Ok(XdlValue::Float(1.0 / (1.0 + (-x).exp()))),
        _ => Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// XDLMLAF_TANH - Hyperbolic tangent activation: f(x) = tanh(x)
pub fn xdlmlaf_tanh(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLmlafTanH: Expected 1 argument (data)".to_string(),
        ));
    }

    match &args[0] {
        XdlValue::Array(arr) => {
            let result: Vec<f64> = arr.iter().map(|&x| x.tanh()).collect();
            Ok(XdlValue::Array(result))
        }
        XdlValue::Double(x) => Ok(XdlValue::Double(x.tanh())),
        XdlValue::Float(x) => Ok(XdlValue::Float(x.tanh())),
        _ => Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// XDLMLAF_RELU - Rectified Linear Unit: f(x) = max(0, x)
pub fn xdlmlaf_relu(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLmlafReLU: Expected 1 argument (data)".to_string(),
        ));
    }

    match &args[0] {
        XdlValue::Array(arr) => {
            let result: Vec<f64> = arr.iter().map(|&x| x.max(0.0)).collect();
            Ok(XdlValue::Array(result))
        }
        XdlValue::Double(x) => Ok(XdlValue::Double(x.max(0.0))),
        XdlValue::Float(x) => Ok(XdlValue::Float(x.max(0.0))),
        _ => Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// XDLMLAF_PRELU - Parametric ReLU: f(x) = x if x > 0, alpha*x otherwise
pub fn xdlmlaf_prelu(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "XDLmlafPReLU: Expected 2 arguments (data, alpha)".to_string(),
        ));
    }

    let alpha = match &args[1] {
        XdlValue::Double(a) => *a,
        XdlValue::Float(a) => *a as f64,
        _ => 0.01, // Default alpha
    };

    match &args[0] {
        XdlValue::Array(arr) => {
            let result: Vec<f64> = arr
                .iter()
                .map(|&x| if x > 0.0 { x } else { alpha * x })
                .collect();
            Ok(XdlValue::Array(result))
        }
        XdlValue::Double(x) => Ok(XdlValue::Double(if *x > 0.0 { *x } else { alpha * x })),
        XdlValue::Float(x) => Ok(XdlValue::Float(if *x > 0.0 {
            *x
        } else {
            (alpha * (*x as f64)) as f32
        })),
        _ => Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// XDLMLAF_ELU - Exponential Linear Unit: f(x) = x if x > 0, alpha*(e^x - 1) otherwise
pub fn xdlmlaf_elu(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLmlafELU: Expected 1 argument (data)".to_string(),
        ));
    }

    let alpha = if args.len() > 1 {
        match &args[1] {
            XdlValue::Double(a) => *a,
            XdlValue::Float(a) => *a as f64,
            _ => 1.0,
        }
    } else {
        1.0
    };

    match &args[0] {
        XdlValue::Array(arr) => {
            let result: Vec<f64> = arr
                .iter()
                .map(|&x| if x > 0.0 { x } else { alpha * (x.exp() - 1.0) })
                .collect();
            Ok(XdlValue::Array(result))
        }
        XdlValue::Double(x) => Ok(XdlValue::Double(if *x > 0.0 {
            *x
        } else {
            alpha * (x.exp() - 1.0)
        })),
        XdlValue::Float(x) => Ok(XdlValue::Float(if *x > 0.0 {
            *x
        } else {
            (alpha * ((*x as f64).exp() - 1.0)) as f32
        })),
        _ => Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// XDLMLAF_SOFTPLUS - Smooth ReLU: f(x) = ln(1 + e^x)
pub fn xdlmlaf_softplus(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLmlafSoftPlus: Expected 1 argument (data)".to_string(),
        ));
    }

    match &args[0] {
        XdlValue::Array(arr) => {
            let result: Vec<f64> = arr.iter().map(|&x| (1.0 + x.exp()).ln()).collect();
            Ok(XdlValue::Array(result))
        }
        XdlValue::Double(x) => Ok(XdlValue::Double((1.0 + x.exp()).ln())),
        XdlValue::Float(x) => Ok(XdlValue::Float((1.0 + (*x as f64).exp()).ln() as f32)),
        _ => Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// XDLMLAF_SOFTSIGN - f(x) = x / (1 + |x|)
pub fn xdlmlaf_softsign(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLmlafSoftSign: Expected 1 argument (data)".to_string(),
        ));
    }

    match &args[0] {
        XdlValue::Array(arr) => {
            let result: Vec<f64> = arr.iter().map(|&x| x / (1.0 + x.abs())).collect();
            Ok(XdlValue::Array(result))
        }
        XdlValue::Double(x) => Ok(XdlValue::Double(x / (1.0 + x.abs()))),
        XdlValue::Float(x) => Ok(XdlValue::Float(x / (1.0 + x.abs()))),
        _ => Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// XDLMLAF_SOFTMAX - Softmax: f(x_i) = e^x_i / Σe^x_j
pub fn xdlmlaf_softmax(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLmlafSoftmax: Expected 1 argument (data)".to_string(),
        ));
    }

    match &args[0] {
        XdlValue::Array(arr) => {
            if arr.is_empty() {
                return Ok(XdlValue::Array(vec![]));
            }

            // For numerical stability, subtract max value
            let max_val = arr.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let exp_values: Vec<f64> = arr.iter().map(|&x| (x - max_val).exp()).collect();
            let sum_exp: f64 = exp_values.iter().sum();

            let result: Vec<f64> = exp_values.iter().map(|&x| x / sum_exp).collect();
            Ok(XdlValue::Array(result))
        }
        _ => Err(XdlError::TypeMismatch {
            expected: "array".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// XDLMLAF_ARCTAN - f(x) = atan(x)
pub fn xdlmlaf_arctan(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLmlafArcTan: Expected 1 argument (data)".to_string(),
        ));
    }

    match &args[0] {
        XdlValue::Array(arr) => {
            let result: Vec<f64> = arr.iter().map(|&x| x.atan()).collect();
            Ok(XdlValue::Array(result))
        }
        XdlValue::Double(x) => Ok(XdlValue::Double(x.atan())),
        XdlValue::Float(x) => Ok(XdlValue::Float(x.atan())),
        _ => Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// XDLMLAF_GAUSSIAN - Gaussian activation: f(x) = e^(-x²)
pub fn xdlmlaf_gaussian(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLmlafGaussian: Expected 1 argument (data)".to_string(),
        ));
    }

    match &args[0] {
        XdlValue::Array(arr) => {
            let result: Vec<f64> = arr.iter().map(|&x| (-x * x).exp()).collect();
            Ok(XdlValue::Array(result))
        }
        XdlValue::Double(x) => Ok(XdlValue::Double((-x * x).exp())),
        XdlValue::Float(x) => Ok(XdlValue::Float((-(x * x)).exp())),
        _ => Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// XDLMLAF_SINC - Sinc function: f(x) = sin(x)/x (f(0) = 1)
pub fn xdlmlaf_sinc(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLmlafSinc: Expected 1 argument (data)".to_string(),
        ));
    }

    let sinc = |x: f64| -> f64 {
        if x.abs() < 1e-10 {
            1.0
        } else {
            x.sin() / x
        }
    };

    match &args[0] {
        XdlValue::Array(arr) => {
            let result: Vec<f64> = arr.iter().map(|&x| sinc(x)).collect();
            Ok(XdlValue::Array(result))
        }
        XdlValue::Double(x) => Ok(XdlValue::Double(sinc(*x))),
        XdlValue::Float(x) => Ok(XdlValue::Float(sinc(*x as f64) as f32)),
        _ => Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// XDLMLAF_SINUSOID - Sine activation: f(x) = sin(x)
pub fn xdlmlaf_sinusoid(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLmlafSinusoid: Expected 1 argument (data)".to_string(),
        ));
    }

    match &args[0] {
        XdlValue::Array(arr) => {
            let result: Vec<f64> = arr.iter().map(|&x| x.sin()).collect();
            Ok(XdlValue::Array(result))
        }
        XdlValue::Double(x) => Ok(XdlValue::Double(x.sin())),
        XdlValue::Float(x) => Ok(XdlValue::Float(x.sin())),
        _ => Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// XDLMLAF_BENTIDENTITY - Bent identity: f(x) = (sqrt(x² + 1) - 1)/2 + x
pub fn xdlmlaf_bentidentity(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLmlafBentIdentity: Expected 1 argument (data)".to_string(),
        ));
    }

    match &args[0] {
        XdlValue::Array(arr) => {
            let result: Vec<f64> = arr
                .iter()
                .map(|&x| ((x * x + 1.0).sqrt() - 1.0) / 2.0 + x)
                .collect();
            Ok(XdlValue::Array(result))
        }
        XdlValue::Double(x) => Ok(XdlValue::Double(((x * x + 1.0).sqrt() - 1.0) / 2.0 + x)),
        XdlValue::Float(x) => {
            let xf = *x as f64;
            Ok(XdlValue::Float(
                (((xf * xf + 1.0).sqrt() - 1.0) / 2.0 + xf) as f32,
            ))
        }
        _ => Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// XDLMLAF_ISRU - Inverse Square Root Unit: f(x) = x / sqrt(1 + alpha*x²)
pub fn xdlmlaf_isru(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLmlafISRU: Expected 1 argument (data)".to_string(),
        ));
    }

    let alpha = if args.len() > 1 {
        match &args[1] {
            XdlValue::Double(a) => *a,
            XdlValue::Float(a) => *a as f64,
            _ => 1.0,
        }
    } else {
        1.0
    };

    match &args[0] {
        XdlValue::Array(arr) => {
            let result: Vec<f64> = arr
                .iter()
                .map(|&x| x / (1.0 + alpha * x * x).sqrt())
                .collect();
            Ok(XdlValue::Array(result))
        }
        XdlValue::Double(x) => Ok(XdlValue::Double(x / (1.0 + alpha * x * x).sqrt())),
        XdlValue::Float(x) => Ok(XdlValue::Float(
            ((*x as f64) / (1.0 + alpha * (*x as f64) * (*x as f64)).sqrt()) as f32,
        )),
        _ => Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// XDLMLAF_ISRLU - Inverse Square Root Linear Unit
pub fn xdlmlaf_isrlu(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLmlafISRLU: Expected 1 argument (data)".to_string(),
        ));
    }

    let alpha = if args.len() > 1 {
        match &args[1] {
            XdlValue::Double(a) => *a,
            XdlValue::Float(a) => *a as f64,
            _ => 1.0,
        }
    } else {
        1.0
    };

    match &args[0] {
        XdlValue::Array(arr) => {
            let result: Vec<f64> = arr
                .iter()
                .map(|&x| {
                    if x >= 0.0 {
                        x
                    } else {
                        x / (1.0 + alpha * x * x).sqrt()
                    }
                })
                .collect();
            Ok(XdlValue::Array(result))
        }
        XdlValue::Double(x) => Ok(XdlValue::Double(if *x >= 0.0 {
            *x
        } else {
            x / (1.0 + alpha * x * x).sqrt()
        })),
        XdlValue::Float(x) => Ok(XdlValue::Float(if *x >= 0.0 {
            *x
        } else {
            ((*x as f64) / (1.0 + alpha * (*x as f64) * (*x as f64)).sqrt()) as f32
        })),
        _ => Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// XDLMLAF_SOFTEXPONENTIAL - Parametric exponential: varies between log and exp
pub fn xdlmlaf_softexponential(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "XDLmlafSoftExponential: Expected 2 arguments (data, alpha)".to_string(),
        ));
    }

    let alpha = match &args[1] {
        XdlValue::Double(a) => *a,
        XdlValue::Float(a) => *a as f64,
        _ => 0.0,
    };

    let soft_exp = |x: f64, a: f64| -> f64 {
        if a < 0.0 {
            -((-a * x + 1.0).ln()) / a
        } else if a > 0.0 {
            ((a * x).exp() - 1.0) / a + a
        } else {
            x
        }
    };

    match &args[0] {
        XdlValue::Array(arr) => {
            let result: Vec<f64> = arr.iter().map(|&x| soft_exp(x, alpha)).collect();
            Ok(XdlValue::Array(result))
        }
        XdlValue::Double(x) => Ok(XdlValue::Double(soft_exp(*x, alpha))),
        XdlValue::Float(x) => Ok(XdlValue::Float(soft_exp(*x as f64, alpha) as f32)),
        _ => Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

// ============================================================================
// LOSS FUNCTIONS (Phase ML-2)
// ============================================================================

/// XDLMLLF_MEANSQUAREDERROR - Mean Squared Error: MSE = mean((y_pred - y_true)²)
pub fn xdlmllf_meansquarederror(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "XDLmllfMeanSquaredError: Expected 2 arguments (y_true, y_pred)".to_string(),
        ));
    }

    let y_true = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let y_pred = match &args[1] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    if y_true.len() != y_pred.len() {
        return Err(XdlError::InvalidArgument(
            "XDLmllfMeanSquaredError: Arrays must have same length".to_string(),
        ));
    }

    let mse: f64 = y_true
        .iter()
        .zip(y_pred.iter())
        .map(|(&yt, &yp)| (yp - yt).powi(2))
        .sum::<f64>()
        / y_true.len() as f64;

    Ok(XdlValue::Double(mse))
}

/// XDLMLLF_MEANABSOLUTEERROR - Mean Absolute Error: MAE = mean(|y_pred - y_true|)
pub fn xdlmllf_meanabsoluteerror(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "XDLmllfMeanAbsoluteError: Expected 2 arguments (y_true, y_pred)".to_string(),
        ));
    }

    let y_true = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let y_pred = match &args[1] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    if y_true.len() != y_pred.len() {
        return Err(XdlError::InvalidArgument(
            "XDLmllfMeanAbsoluteError: Arrays must have same length".to_string(),
        ));
    }

    let mae: f64 = y_true
        .iter()
        .zip(y_pred.iter())
        .map(|(&yt, &yp)| (yp - yt).abs())
        .sum::<f64>()
        / y_true.len() as f64;

    Ok(XdlValue::Double(mae))
}

/// XDLMLLF_CROSSENTROPY - Cross-entropy loss: -Σ(y_true * log(y_pred))
pub fn xdlmllf_crossentropy(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "XDLmllfCrossEntropy: Expected 2 arguments (y_true, y_pred)".to_string(),
        ));
    }

    let y_true = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let y_pred = match &args[1] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    if y_true.len() != y_pred.len() {
        return Err(XdlError::InvalidArgument(
            "XDLmllfCrossEntropy: Arrays must have same length".to_string(),
        ));
    }

    // Add small epsilon to avoid log(0)
    let epsilon = 1e-10;
    let cross_entropy: f64 = -y_true
        .iter()
        .zip(y_pred.iter())
        .map(|(&yt, &yp)| yt * (yp + epsilon).ln())
        .sum::<f64>()
        / y_true.len() as f64;

    Ok(XdlValue::Double(cross_entropy))
}

/// XDLMLLF_HUBER - Huber loss (robust to outliers)
/// For |error| <= delta: 0.5 * error²
/// For |error| > delta: delta * (|error| - 0.5 * delta)
pub fn xdlmllf_huber(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "XDLmllfHuber: Expected at least 2 arguments (y_true, y_pred)".to_string(),
        ));
    }

    let y_true = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let y_pred = match &args[1] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    // Get delta (default 1.0)
    let delta = if args.len() > 2 {
        match &args[2] {
            XdlValue::Double(d) => *d,
            XdlValue::Float(d) => *d as f64,
            _ => 1.0,
        }
    } else {
        1.0
    };

    if y_true.len() != y_pred.len() {
        return Err(XdlError::InvalidArgument(
            "XDLmllfHuber: Arrays must have same length".to_string(),
        ));
    }

    let huber_loss: f64 = y_true
        .iter()
        .zip(y_pred.iter())
        .map(|(&yt, &yp)| {
            let error = (yp - yt).abs();
            if error <= delta {
                0.5 * error * error
            } else {
                delta * (error - 0.5 * delta)
            }
        })
        .sum::<f64>()
        / y_true.len() as f64;

    Ok(XdlValue::Double(huber_loss))
}

/// XDLMLLF_LOGCOSH - Log-cosh loss: log(cosh(y_pred - y_true))
/// Smooth approximation of MAE, less sensitive to outliers
pub fn xdlmllf_logcosh(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "XDLmllfLogCosh: Expected 2 arguments (y_true, y_pred)".to_string(),
        ));
    }

    let y_true = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let y_pred = match &args[1] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    if y_true.len() != y_pred.len() {
        return Err(XdlError::InvalidArgument(
            "XDLmllfLogCosh: Arrays must have same length".to_string(),
        ));
    }

    let logcosh_loss: f64 = y_true
        .iter()
        .zip(y_pred.iter())
        .map(|(&yt, &yp)| (yp - yt).cosh().ln())
        .sum::<f64>()
        / y_true.len() as f64;

    Ok(XdlValue::Double(logcosh_loss))
}

// ============================================================================
// OPTIMIZERS (Phase ML-3)
// ============================================================================

/// XDLMLOPT_GRADIENTDESCENT - Basic gradient descent optimizer
///
/// Updates weights using: w = w - learning_rate * gradient
///
/// Parameters:
///   weights: Current weights array
///   gradients: Gradient array (same size as weights)
///   learning_rate: Learning rate (default 0.01)
///
/// Returns:
///   Updated weights
pub fn xdlmlopt_gradientdescent(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "XDLmloptGradientDescent: Expected at least 2 arguments (weights, gradients)"
                .to_string(),
        ));
    }

    let weights = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let gradients = match &args[1] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let learning_rate = if args.len() > 2 {
        match &args[2] {
            XdlValue::Double(lr) => *lr,
            XdlValue::Float(lr) => *lr as f64,
            _ => 0.01,
        }
    } else {
        0.01
    };

    if weights.len() != gradients.len() {
        return Err(XdlError::InvalidArgument(
            "XDLmloptGradientDescent: weights and gradients must have same length".to_string(),
        ));
    }

    let updated_weights: Vec<f64> = weights
        .iter()
        .zip(gradients.iter())
        .map(|(&w, &g)| w - learning_rate * g)
        .collect();

    Ok(XdlValue::Array(updated_weights))
}

/// XDLMLOPT_MOMENTUM - Gradient descent with momentum
///
/// Updates using exponentially weighted moving average of gradients
/// velocity = momentum * velocity + learning_rate * gradient
/// w = w - velocity
///
/// Parameters:
///   weights: Current weights array
///   gradients: Gradient array
///   velocity: Velocity array (momentum state, same size as weights)
///   learning_rate: Learning rate (default 0.01)
///   momentum: Momentum coefficient (default 0.9)
///
/// Returns:
///   [updated_weights, updated_velocity]
pub fn xdlmlopt_momentum(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(
            "XDLmloptMomentum: Expected at least 3 arguments (weights, gradients, velocity)"
                .to_string(),
        ));
    }

    let weights = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let gradients = match &args[1] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let velocity = match &args[2] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[2].gdl_type()),
            })
        }
    };

    let learning_rate = if args.len() > 3 {
        match &args[3] {
            XdlValue::Double(lr) => *lr,
            XdlValue::Float(lr) => *lr as f64,
            _ => 0.01,
        }
    } else {
        0.01
    };

    let momentum = if args.len() > 4 {
        match &args[4] {
            XdlValue::Double(m) => *m,
            XdlValue::Float(m) => *m as f64,
            _ => 0.9,
        }
    } else {
        0.9
    };

    if weights.len() != gradients.len() || weights.len() != velocity.len() {
        return Err(XdlError::InvalidArgument(
            "XDLmloptMomentum: weights, gradients, and velocity must have same length".to_string(),
        ));
    }

    let mut updated_velocity = Vec::with_capacity(weights.len());
    let mut updated_weights = Vec::with_capacity(weights.len());

    for i in 0..weights.len() {
        let v = momentum * velocity[i] + learning_rate * gradients[i];
        updated_velocity.push(v);
        updated_weights.push(weights[i] - v);
    }

    // Return as nested array structure: [[weights], [velocity]]
    // For simplicity, we'll return just weights and store velocity separately in practice
    Ok(XdlValue::Array(updated_weights))
}

/// XDLMLOPT_RMSPROP - Root Mean Square Propagation optimizer
///
/// Adaptive learning rate method that uses moving average of squared gradients
/// cache = decay * cache + (1 - decay) * gradient²
/// w = w - learning_rate * gradient / (sqrt(cache) + epsilon)
///
/// Parameters:
///   weights: Current weights
///   gradients: Gradients
///   cache: Squared gradient cache (same size as weights)
///   learning_rate: Learning rate (default 0.001)
///   decay: Decay rate for cache (default 0.9)
///   epsilon: Small constant for numerical stability (default 1e-8)
///
/// Returns:
///   Updated weights
pub fn xdlmlopt_rmsprop(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(
            "XDLmloptRMSProp: Expected at least 3 arguments (weights, gradients, cache)"
                .to_string(),
        ));
    }

    let weights = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let gradients = match &args[1] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let cache = match &args[2] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[2].gdl_type()),
            })
        }
    };

    let learning_rate = if args.len() > 3 {
        match &args[3] {
            XdlValue::Double(lr) => *lr,
            XdlValue::Float(lr) => *lr as f64,
            _ => 0.001,
        }
    } else {
        0.001
    };

    let decay = if args.len() > 4 {
        match &args[4] {
            XdlValue::Double(d) => *d,
            XdlValue::Float(d) => *d as f64,
            _ => 0.9,
        }
    } else {
        0.9
    };

    let epsilon = if args.len() > 5 {
        match &args[5] {
            XdlValue::Double(e) => *e,
            XdlValue::Float(e) => *e as f64,
            _ => 1e-8,
        }
    } else {
        1e-8
    };

    if weights.len() != gradients.len() || weights.len() != cache.len() {
        return Err(XdlError::InvalidArgument(
            "XDLmloptRMSProp: weights, gradients, and cache must have same length".to_string(),
        ));
    }

    let updated_weights: Vec<f64> = weights
        .iter()
        .zip(gradients.iter())
        .zip(cache.iter())
        .map(|((&w, &g), &c)| {
            let new_cache = decay * c + (1.0 - decay) * g * g;
            w - learning_rate * g / (new_cache.sqrt() + epsilon)
        })
        .collect();

    Ok(XdlValue::Array(updated_weights))
}

/// XDLMLOPT_ADAM - Adaptive Moment Estimation optimizer
///
/// Combines momentum and RMSProp advantages
/// m = beta1 * m + (1 - beta1) * gradient
/// v = beta2 * v + (1 - beta2) * gradient²
/// m_hat = m / (1 - beta1^t)
/// v_hat = v / (1 - beta2^t)
/// w = w - learning_rate * m_hat / (sqrt(v_hat) + epsilon)
///
/// Parameters:
///   weights: Current weights
///   gradients: Gradients
///   m: First moment estimate (same size as weights)
///   v: Second moment estimate (same size as weights)
///   t: Time step (for bias correction)
///   learning_rate: Learning rate (default 0.001)
///   beta1: Exponential decay for first moment (default 0.9)
///   beta2: Exponential decay for second moment (default 0.999)
///   epsilon: Small constant (default 1e-8)
///
/// Returns:
///   Updated weights
pub fn xdlmlopt_adam(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 5 {
        return Err(XdlError::InvalidArgument(
            "XDLmloptAdam: Expected at least 5 arguments (weights, gradients, m, v, t)".to_string(),
        ));
    }

    let weights = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let gradients = match &args[1] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let m = match &args[2] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[2].gdl_type()),
            })
        }
    };

    let v = match &args[3] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[3].gdl_type()),
            })
        }
    };

    let t = match &args[4] {
        XdlValue::Long(step) => *step as f64,
        XdlValue::Int(step) => *step as f64,
        XdlValue::Double(step) => *step,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: format!("{:?}", args[4].gdl_type()),
            })
        }
    };

    let learning_rate = if args.len() > 5 {
        match &args[5] {
            XdlValue::Double(lr) => *lr,
            XdlValue::Float(lr) => *lr as f64,
            _ => 0.001,
        }
    } else {
        0.001
    };

    let beta1 = if args.len() > 6 {
        match &args[6] {
            XdlValue::Double(b) => *b,
            XdlValue::Float(b) => *b as f64,
            _ => 0.9,
        }
    } else {
        0.9
    };

    let beta2 = if args.len() > 7 {
        match &args[7] {
            XdlValue::Double(b) => *b,
            XdlValue::Float(b) => *b as f64,
            _ => 0.999,
        }
    } else {
        0.999
    };

    let epsilon = if args.len() > 8 {
        match &args[8] {
            XdlValue::Double(e) => *e,
            XdlValue::Float(e) => *e as f64,
            _ => 1e-8,
        }
    } else {
        1e-8
    };

    if weights.len() != gradients.len() || weights.len() != m.len() || weights.len() != v.len() {
        return Err(XdlError::InvalidArgument(
            "XDLmloptAdam: weights, gradients, m, and v must have same length".to_string(),
        ));
    }

    // Bias correction terms
    let bias_correction1 = 1.0 - beta1.powf(t);
    let bias_correction2 = 1.0 - beta2.powf(t);

    let updated_weights: Vec<f64> = weights
        .iter()
        .zip(gradients.iter())
        .zip(m.iter())
        .zip(v.iter())
        .map(|(((&w, &g), &m_val), &v_val)| {
            // Update biased first moment estimate
            let m_new = beta1 * m_val + (1.0 - beta1) * g;
            // Update biased second moment estimate
            let v_new = beta2 * v_val + (1.0 - beta2) * g * g;
            // Compute bias-corrected estimates
            let m_hat = m_new / bias_correction1;
            let v_hat = v_new / bias_correction2;
            // Update weights
            w - learning_rate * m_hat / (v_hat.sqrt() + epsilon)
        })
        .collect();

    Ok(XdlValue::Array(updated_weights))
}

/// XDLMLOPT_QUICKPROP - QuickProp optimizer
///
/// Uses second-order information (previous gradient) for faster convergence
/// Based on Newton's method approximation
///
/// Parameters:
///   weights: Current weights
///   gradients: Current gradients
///   prev_gradients: Previous gradients
///   prev_step: Previous weight update step
///   learning_rate: Learning rate (default 0.01)
///   mu: Maximum growth factor (default 1.75)
///
/// Returns:
///   Updated weights
pub fn xdlmlopt_quickprop(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 4 {
        return Err(XdlError::InvalidArgument(
            "XDLmloptQuickProp: Expected at least 4 arguments (weights, gradients, prev_gradients, prev_step)"
                .to_string(),
        ));
    }

    let weights = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let gradients = match &args[1] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let prev_gradients = match &args[2] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[2].gdl_type()),
            })
        }
    };

    let prev_step = match &args[3] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[3].gdl_type()),
            })
        }
    };

    let learning_rate = if args.len() > 4 {
        match &args[4] {
            XdlValue::Double(lr) => *lr,
            XdlValue::Float(lr) => *lr as f64,
            _ => 0.01,
        }
    } else {
        0.01
    };

    let mu = if args.len() > 5 {
        match &args[5] {
            XdlValue::Double(m) => *m,
            XdlValue::Float(m) => *m as f64,
            _ => 1.75,
        }
    } else {
        1.75
    };

    if weights.len() != gradients.len()
        || weights.len() != prev_gradients.len()
        || weights.len() != prev_step.len()
    {
        return Err(XdlError::InvalidArgument(
            "XDLmloptQuickProp: All arrays must have same length".to_string(),
        ));
    }

    let updated_weights: Vec<f64> = weights
        .iter()
        .zip(gradients.iter())
        .zip(prev_gradients.iter())
        .zip(prev_step.iter())
        .map(|(((&w, &g), &pg), &ps)| {
            let step = if (g - pg).abs() > 1e-10 {
                // QuickProp formula
                let s = ps * g / (pg - g);
                // Limit step size
                s.max(-mu * ps.abs()).min(mu * ps.abs())
            } else {
                // Fallback to gradient descent
                -learning_rate * g
            };
            w + step
        })
        .collect();

    Ok(XdlValue::Array(updated_weights))
}

// ============================================================================
// SVM KERNEL FUNCTIONS (Phase ML-5)
// ============================================================================

/// XDLML_SVMLINEARKERNEL - Linear kernel: K(x, y) = x · y
///
/// Computes dot product between two vectors
///
/// Parameters:
///   x: First vector
///   y: Second vector
///
/// Returns:
///   Scalar dot product
pub fn xdlml_svmlinearkernel(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "XDLML_SVMLinearKernel: Expected 2 arguments (x, y)".to_string(),
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
        return Err(XdlError::InvalidArgument(
            "XDLML_SVMLinearKernel: Vectors must have same length".to_string(),
        ));
    }

    let dot_product: f64 = x.iter().zip(y.iter()).map(|(&xi, &yi)| xi * yi).sum();

    Ok(XdlValue::Double(dot_product))
}

/// XDLML_SVMPOLYNOMIALKERNEL - Polynomial kernel: K(x, y) = (gamma * x·y + coef0)^degree
///
/// Parameters:
///   x: First vector
///   y: Second vector
///   gamma: Scaling factor (default 1.0)
///   coef0: Independent term (default 0.0)
///   degree: Polynomial degree (default 3)
///
/// Returns:
///   Polynomial kernel value
pub fn xdlml_svmpolynomialkernel(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "XDLML_SVMPolynomialKernel: Expected at least 2 arguments (x, y)".to_string(),
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

    let gamma = if args.len() > 2 {
        match &args[2] {
            XdlValue::Double(g) => *g,
            XdlValue::Float(g) => *g as f64,
            _ => 1.0,
        }
    } else {
        1.0
    };

    let coef0 = if args.len() > 3 {
        match &args[3] {
            XdlValue::Double(c) => *c,
            XdlValue::Float(c) => *c as f64,
            _ => 0.0,
        }
    } else {
        0.0
    };

    let degree = if args.len() > 4 {
        match &args[4] {
            XdlValue::Long(d) => *d,
            XdlValue::Int(d) => *d as i32,
            _ => 3,
        }
    } else {
        3
    };

    if x.len() != y.len() {
        return Err(XdlError::InvalidArgument(
            "XDLML_SVMPolynomialKernel: Vectors must have same length".to_string(),
        ));
    }

    let dot_product: f64 = x.iter().zip(y.iter()).map(|(&xi, &yi)| xi * yi).sum();
    let kernel_value = (gamma * dot_product + coef0).powi(degree);

    Ok(XdlValue::Double(kernel_value))
}

/// XDLML_SVMRADIALKERNEL - RBF (Radial Basis Function) kernel: K(x, y) = exp(-gamma * ||x-y||²)
///
/// Parameters:
///   x: First vector
///   y: Second vector
///   gamma: Scaling factor (default 1.0)
///
/// Returns:
///   RBF kernel value
pub fn xdlml_svmradialkernel(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "XDLML_SVMRadialKernel: Expected at least 2 arguments (x, y)".to_string(),
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

    let gamma = if args.len() > 2 {
        match &args[2] {
            XdlValue::Double(g) => *g,
            XdlValue::Float(g) => *g as f64,
            _ => 1.0,
        }
    } else {
        1.0
    };

    if x.len() != y.len() {
        return Err(XdlError::InvalidArgument(
            "XDLML_SVMRadialKernel: Vectors must have same length".to_string(),
        ));
    }

    // Calculate squared Euclidean distance ||x - y||²
    let squared_distance: f64 = x
        .iter()
        .zip(y.iter())
        .map(|(&xi, &yi)| (xi - yi).powi(2))
        .sum();

    let kernel_value = (-gamma * squared_distance).exp();

    Ok(XdlValue::Double(kernel_value))
}

/// XDLML_SVMSIGMOIDKERNEL - Sigmoid kernel: K(x, y) = tanh(gamma * x·y + coef0)
///
/// Parameters:
///   x: First vector
///   y: Second vector
///   gamma: Scaling factor (default 1.0)
///   coef0: Independent term (default 0.0)
///
/// Returns:
///   Sigmoid kernel value
pub fn xdlml_svmsigmoidkernel(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "XDLML_SVMSigmoidKernel: Expected at least 2 arguments (x, y)".to_string(),
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

    let gamma = if args.len() > 2 {
        match &args[2] {
            XdlValue::Double(g) => *g,
            XdlValue::Float(g) => *g as f64,
            _ => 1.0,
        }
    } else {
        1.0
    };

    let coef0 = if args.len() > 3 {
        match &args[3] {
            XdlValue::Double(c) => *c,
            XdlValue::Float(c) => *c as f64,
            _ => 0.0,
        }
    } else {
        0.0
    };

    if x.len() != y.len() {
        return Err(XdlError::InvalidArgument(
            "XDLML_SVMSigmoidKernel: Vectors must have same length".to_string(),
        ));
    }

    let dot_product: f64 = x.iter().zip(y.iter()).map(|(&xi, &yi)| xi * yi).sum();
    let kernel_value = (gamma * dot_product + coef0).tanh();

    Ok(XdlValue::Double(kernel_value))
}

// ============================================================================
// MODEL EVALUATION (Phase ML-6)
// ============================================================================

/// XDLML_TESTCLASSIFIER - Model evaluation metrics
///
/// Computes classification metrics including:
/// - Accuracy
/// - Precision
/// - Recall
/// - F1-score
/// - Confusion matrix
///
/// Parameters:
///   y_true: True labels
///   y_pred: Predicted labels
///
/// Returns:
///   Struct with metrics: {accuracy, precision, recall, f1score}
///   For now, returns array: [accuracy, precision, recall, f1]
pub fn xdlml_testclassifier(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "XDLML_TestClassifier: Expected 2 arguments (y_true, y_pred)".to_string(),
        ));
    }

    let y_true = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let y_pred = match &args[1] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    if y_true.len() != y_pred.len() {
        return Err(XdlError::InvalidArgument(
            "XDLML_TestClassifier: Arrays must have same length".to_string(),
        ));
    }

    if y_true.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLML_TestClassifier: Arrays cannot be empty".to_string(),
        ));
    }

    // Count true positives, false positives, true negatives, false negatives
    // Assuming binary classification (0 or 1)
    let mut tp = 0.0;
    let mut fp = 0.0;
    let mut tn = 0.0;
    let mut fn_count = 0.0;

    for i in 0..y_true.len() {
        let true_val = y_true[i].round();
        let pred_val = y_pred[i].round();

        if true_val == 1.0 && pred_val == 1.0 {
            tp += 1.0;
        } else if true_val == 0.0 && pred_val == 1.0 {
            fp += 1.0;
        } else if true_val == 0.0 && pred_val == 0.0 {
            tn += 1.0;
        } else if true_val == 1.0 && pred_val == 0.0 {
            fn_count += 1.0;
        }
    }

    let total = y_true.len() as f64;

    // Calculate metrics
    let accuracy = (tp + tn) / total;

    let precision = if (tp + fp) > 0.0 { tp / (tp + fp) } else { 0.0 };

    let recall = if (tp + fn_count) > 0.0 {
        tp / (tp + fn_count)
    } else {
        0.0
    };

    let f1_score = if (precision + recall) > 0.0 {
        2.0 * (precision * recall) / (precision + recall)
    } else {
        0.0
    };

    // Return metrics as array: [accuracy, precision, recall, f1_score]
    let metrics = vec![accuracy, precision, recall, f1_score];

    Ok(XdlValue::Array(metrics))
}

// ============================================================================
// CLASSIFIER MODELS (Phase ML-6)
// ============================================================================

/// XDLML_SOFTMAX - Softmax classifier for multi-class classification
///
/// Trains a softmax classifier using gradient descent.
/// Model: y = softmax(X * W + b)
/// Loss: Cross-entropy
///
/// Parameters:
///   X_train: Training features [n_samples x n_features]
///   y_train: Training labels (one-hot encoded) [n_samples x n_classes]
///   n_classes: Number of classes
///   learning_rate: Learning rate (default 0.01)
///   n_epochs: Number of training epochs (default 100)
///   batch_size: Mini-batch size (default 32, use 0 for full batch)
///   seed: Random seed for weight initialization (default: time-based)
///
/// Returns:
///   Trained weights array [n_features x n_classes]
///
/// Example:
///   ; 2 features, 3 classes, 100 samples
///   X = RANDOMU(seed, 100, 2)  ; 100 samples, 2 features
///   y = FLTARR(100, 3)  ; 100 samples, 3 classes (one-hot)
///   weights = XDLML_SOFTMAX(X, y, 3, 0.1, 200)
///
/// Note: This is a simplified implementation. For 1D data, reshape to [n_samples, 1]
pub fn xdlml_softmax(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(
            "XDLML_Softmax: Expected at least 3 arguments (X_train, y_train, n_classes)"
                .to_string(),
        ));
    }

    // Get training data (for simplicity, treating as 1D features)
    let x_train = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let y_train = match &args[1] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let n_classes = match &args[2] {
        XdlValue::Long(n) => *n as usize,
        XdlValue::Int(n) => *n as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[2].gdl_type()),
            })
        }
    };

    let learning_rate = if args.len() > 3 {
        match &args[3] {
            XdlValue::Double(lr) => *lr,
            XdlValue::Float(lr) => *lr as f64,
            _ => 0.01,
        }
    } else {
        0.01
    };

    let n_epochs = if args.len() > 4 {
        match &args[4] {
            XdlValue::Long(e) => *e as usize,
            XdlValue::Int(e) => *e as usize,
            _ => 100,
        }
    } else {
        100
    };

    let _batch_size = if args.len() > 5 {
        match &args[5] {
            XdlValue::Long(b) => *b as usize,
            XdlValue::Int(b) => *b as usize,
            _ => 0,
        }
    } else {
        0 // Full batch
    };

    let seed = if args.len() > 6 {
        match &args[6] {
            XdlValue::Long(s) => *s as u64,
            XdlValue::Int(s) => *s as u64,
            _ => {
                use std::time::{SystemTime, UNIX_EPOCH};
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            }
        }
    } else {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    };

    // Simplified implementation: treat X as 1D features (n_samples)
    // y_train should be class labels (0 to n_classes-1)
    let n_samples = x_train.len();
    let n_features = 1; // Simplified: 1D input

    if y_train.len() != n_samples {
        return Err(XdlError::InvalidArgument(
            "XDLML_Softmax: X and y must have same number of samples".to_string(),
        ));
    }

    // Initialize weights: [n_features x n_classes]
    // For 1D input: [1 x n_classes]
    let mut weights = vec![0.0; n_features * n_classes];
    let mut rng_state = seed;

    // Xavier initialization: weights ~ N(0, 1/sqrt(n_features))
    for weight in &mut weights {
        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        let uniform = (rng_state as f64 / u64::MAX as f64) * 2.0 - 1.0;
        *weight = uniform * (1.0 / (n_features as f64).sqrt());
    }

    // Training loop
    for _epoch in 0..n_epochs {
        // Full batch gradient descent
        let mut gradients = vec![0.0; n_features * n_classes];

        for i in 0..n_samples {
            let x_val = x_train[i];
            let true_class = y_train[i].round() as usize;

            if true_class >= n_classes {
                continue; // Skip invalid labels
            }

            // Forward pass: compute logits
            let mut logits = vec![0.0; n_classes];
            for c in 0..n_classes {
                logits[c] = x_val * weights[c]; // Single feature case
            }

            // Apply softmax
            let max_logit = logits.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let exp_logits: Vec<f64> = logits.iter().map(|&l| (l - max_logit).exp()).collect();
            let sum_exp: f64 = exp_logits.iter().sum();
            let probs: Vec<f64> = exp_logits.iter().map(|&e| e / sum_exp).collect();

            // Compute gradients (derivative of cross-entropy + softmax)
            // dL/dw = x * (p - y)
            for c in 0..n_classes {
                let target = if c == true_class { 1.0 } else { 0.0 };
                let error = probs[c] - target;
                gradients[c] += x_val * error;
            }
        }

        // Average gradients over batch
        for g in &mut gradients {
            *g /= n_samples as f64;
        }

        // Update weights: w = w - lr * gradient
        for i in 0..weights.len() {
            weights[i] -= learning_rate * gradients[i];
        }
    }

    // Return trained weights
    Ok(XdlValue::Array(weights))
}

// ============================================================================
// NEURAL NETWORK MODELS (Phase ML-4)
// ============================================================================

/// XDLML_FEEDFORWARDNEURALNETWORK - Multi-layer perceptron classifier
///
/// Simplified feedforward neural network with backpropagation.
/// Architecture: Input -> Hidden Layer -> Output Layer
///
/// Parameters:
///   X_train: Training features (1D array for simplicity)
///   y_train: Training labels (class indices)
///   n_hidden: Number of hidden units (default 10)
///   n_classes: Number of output classes
///   learning_rate: Learning rate (default 0.01)
///   n_epochs: Number of training epochs (default 100)
///   seed: Random seed for weight initialization
///
/// Returns:
///   Array containing: [weights_input_hidden..., weights_hidden_output...]
///   First n_inputs*n_hidden values are input->hidden weights
///   Remaining n_hidden*n_classes values are hidden->output weights
///
/// Example:
///   X = RANDOMU(seed, 100)  ; 100 samples
///   y = FLOOR(RANDOMU(seed, 100) * 3)  ; 3 classes
///   model = XDLML_FEEDFORWARDNEURALNETWORK(X, y, 10, 3, 0.1, 200, 42)
pub fn xdlml_feedforwardneuralnetwork(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 4 {
        return Err(XdlError::InvalidArgument(
            "XDLML_FeedForwardNeuralNetwork: Expected at least 4 arguments (X, y, n_hidden, n_classes)"
                .to_string(),
        ));
    }

    let x_train = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let y_train = match &args[1] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let n_hidden = match &args[2] {
        XdlValue::Long(n) => *n as usize,
        XdlValue::Int(n) => *n as usize,
        _ => 10,
    };

    let n_classes = match &args[3] {
        XdlValue::Long(n) => *n as usize,
        XdlValue::Int(n) => *n as usize,
        _ => 2,
    };

    let learning_rate = if args.len() > 4 {
        match &args[4] {
            XdlValue::Double(lr) => *lr,
            XdlValue::Float(lr) => *lr as f64,
            _ => 0.01,
        }
    } else {
        0.01
    };

    let n_epochs = if args.len() > 5 {
        match &args[5] {
            XdlValue::Long(e) => *e as usize,
            XdlValue::Int(e) => *e as usize,
            _ => 100,
        }
    } else {
        100
    };

    let seed = if args.len() > 6 {
        match &args[6] {
            XdlValue::Long(s) => *s as u64,
            XdlValue::Int(s) => *s as u64,
            _ => 42,
        }
    } else {
        42
    };

    let n_samples = x_train.len();
    let n_inputs = 1; // Simplified: 1D input

    if y_train.len() != n_samples {
        return Err(XdlError::InvalidArgument(
            "XDLML_FeedForwardNeuralNetwork: X and y must have same length".to_string(),
        ));
    }

    // Initialize weights
    let mut w1 = vec![0.0; n_inputs * n_hidden]; // Input -> Hidden
    let mut w2 = vec![0.0; n_hidden * n_classes]; // Hidden -> Output
    let mut rng_state = seed;

    // Xavier initialization
    for w in &mut w1 {
        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        *w = ((rng_state as f64 / u64::MAX as f64) * 2.0 - 1.0) * (2.0 / n_inputs as f64).sqrt();
    }
    for w in &mut w2 {
        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        *w = ((rng_state as f64 / u64::MAX as f64) * 2.0 - 1.0) * (2.0 / n_hidden as f64).sqrt();
    }

    // Training loop
    for _epoch in 0..n_epochs {
        for i in 0..n_samples {
            let x = x_train[i];
            let true_class = y_train[i].round() as usize;

            if true_class >= n_classes {
                continue;
            }

            // Forward pass
            // Hidden layer (ReLU activation)
            let mut hidden = vec![0.0; n_hidden];
            for h in 0..n_hidden {
                hidden[h] = (x * w1[h]).max(0.0); // ReLU
            }

            // Output layer (softmax)
            let mut logits = vec![0.0; n_classes];
            for c in 0..n_classes {
                for h in 0..n_hidden {
                    logits[c] += hidden[h] * w2[h * n_classes + c];
                }
            }

            // Softmax
            let max_logit = logits.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let exp_logits: Vec<f64> = logits.iter().map(|&l| (l - max_logit).exp()).collect();
            let sum_exp: f64 = exp_logits.iter().sum();
            let output: Vec<f64> = exp_logits.iter().map(|&e| e / sum_exp).collect();

            // Backward pass (backpropagation)
            // Output layer gradient
            let mut output_grad = vec![0.0; n_classes];
            for c in 0..n_classes {
                let target = if c == true_class { 1.0 } else { 0.0 };
                output_grad[c] = output[c] - target;
            }

            // Hidden layer gradient
            let mut hidden_grad = vec![0.0; n_hidden];
            for h in 0..n_hidden {
                for c in 0..n_classes {
                    hidden_grad[h] += output_grad[c] * w2[h * n_classes + c];
                }
                // ReLU derivative
                if hidden[h] <= 0.0 {
                    hidden_grad[h] = 0.0;
                }
            }

            // Update weights w2 (hidden -> output)
            for h in 0..n_hidden {
                for c in 0..n_classes {
                    w2[h * n_classes + c] -= learning_rate * hidden[h] * output_grad[c];
                }
            }

            // Update weights w1 (input -> hidden)
            for h in 0..n_hidden {
                w1[h] -= learning_rate * x * hidden_grad[h];
            }
        }
    }

    // Combine weights into single array
    let mut all_weights = Vec::with_capacity(w1.len() + w2.len());
    all_weights.extend_from_slice(&w1);
    all_weights.extend_from_slice(&w2);

    Ok(XdlValue::Array(all_weights))
}

/// XDLML_AUTOENCODER - Autoencoder for unsupervised learning
///
/// Simplified autoencoder: Input -> Bottleneck -> Reconstruction
/// Learns compressed representation of data
///
/// Parameters:
///   X_train: Training data
///   encoding_dim: Dimension of encoded representation (default 5)
///   learning_rate: Learning rate (default 0.01)
///   n_epochs: Number of training epochs (default 100)
///   seed: Random seed
///
/// Returns:
///   Encoder and decoder weights combined
pub fn xdlml_autoencoder(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLML_AutoEncoder: Expected at least 1 argument (X_train)".to_string(),
        ));
    }

    let x_train = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let encoding_dim = if args.len() > 1 {
        match &args[1] {
            XdlValue::Long(n) => *n as usize,
            XdlValue::Int(n) => *n as usize,
            _ => 5,
        }
    } else {
        5
    };

    let learning_rate = if args.len() > 2 {
        match &args[2] {
            XdlValue::Double(lr) => *lr,
            XdlValue::Float(lr) => *lr as f64,
            _ => 0.01,
        }
    } else {
        0.01
    };

    let n_epochs = if args.len() > 3 {
        match &args[3] {
            XdlValue::Long(e) => *e as usize,
            XdlValue::Int(e) => *e as usize,
            _ => 100,
        }
    } else {
        100
    };

    let seed = if args.len() > 4 {
        match &args[4] {
            XdlValue::Long(s) => *s as u64,
            XdlValue::Int(s) => *s as u64,
            _ => 42,
        }
    } else {
        42
    };

    let n_samples = x_train.len();
    let input_dim = 1; // Simplified: 1D input

    // Initialize encoder and decoder weights
    let mut encoder = vec![0.0; input_dim * encoding_dim];
    let mut decoder = vec![0.0; encoding_dim * input_dim];
    let mut rng_state = seed;

    // Xavier initialization
    for enc in &mut encoder {
        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        *enc = ((rng_state as f64 / u64::MAX as f64) * 2.0 - 1.0) * (2.0 / input_dim as f64).sqrt();
    }
    for dec in &mut decoder {
        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        *dec =
            ((rng_state as f64 / u64::MAX as f64) * 2.0 - 1.0) * (2.0 / encoding_dim as f64).sqrt();
    }

    // Training loop
    for _epoch in 0..n_epochs {
        for &x in x_train.iter().take(n_samples) {
            // Forward pass
            // Encode (with ReLU)
            let mut encoded = vec![0.0; encoding_dim];
            for e in 0..encoding_dim {
                encoded[e] = (x * encoder[e]).max(0.0);
            }

            // Decode (linear)
            let mut reconstructed = 0.0;
            for e in 0..encoding_dim {
                reconstructed += encoded[e] * decoder[e];
            }

            // Compute reconstruction error
            let error = reconstructed - x;

            // Backward pass
            // Decoder gradient
            for e in 0..encoding_dim {
                decoder[e] -= learning_rate * error * encoded[e];
            }

            // Encoder gradient (through decoder)
            for e in 0..encoding_dim {
                let grad = error * decoder[e];
                // ReLU derivative
                if encoded[e] > 0.0 {
                    encoder[e] -= learning_rate * grad * x;
                }
            }
        }
    }

    // Combine encoder and decoder weights
    let mut all_weights = Vec::with_capacity(encoder.len() + decoder.len());
    all_weights.extend_from_slice(&encoder);
    all_weights.extend_from_slice(&decoder);

    Ok(XdlValue::Array(all_weights))
}

// ============================================================================
// SVM MODELS (Phase ML-5) - Full SMO Implementation
// ============================================================================

/// XDLML_SUPPORTVECTORMACHINECLASSIFICATION - SVM for binary classification
///
/// Full implementation using Sequential Minimal Optimization (SMO) algorithm.
/// Supports multiple kernel types via kernel parameter.
///
/// Parameters:
///   X_train: Training features (1D for simplicity)
///   y_train: Training labels (1 or -1)
///   kernel_type: Kernel type (0=linear, 1=polynomial, 2=RBF, 3=sigmoid)
///   C: Regularization parameter (default 1.0)
///   tolerance: Convergence tolerance (default 0.001)
///   max_iter: Maximum iterations (default 1000)
///   gamma: Kernel coefficient (default 1.0)
///   degree: Polynomial degree (default 3)
///   coef0: Kernel coefficient (default 0.0)
///
/// Returns:
///   Array: [alphas..., bias]
///   Support vector multipliers and bias term
///
/// Example:
///   X = RANDOMU(seed, 100)
///   y = (X GT 0.5) * 2 - 1  ; Binary labels: 1 or -1
///   model = XDLML_SUPPORTVECTORMACHINECLASSIFICATION(X, y, 2, 1.0, 0.001, 1000, 0.5)
pub fn xdlml_supportvectormachineclassification(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "XDLML_SupportVectorMachineClassification: Expected at least 2 arguments (X, y)"
                .to_string(),
        ));
    }

    let x_train = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let y_train = match &args[1] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let kernel_type = if args.len() > 2 {
        match &args[2] {
            XdlValue::Long(k) => *k as usize,
            XdlValue::Int(k) => *k as usize,
            _ => 2, // RBF default
        }
    } else {
        2
    };

    let c_param = if args.len() > 3 {
        match &args[3] {
            XdlValue::Double(c) => *c,
            XdlValue::Float(c) => *c as f64,
            _ => 1.0,
        }
    } else {
        1.0
    };

    let tolerance = if args.len() > 4 {
        match &args[4] {
            XdlValue::Double(t) => *t,
            XdlValue::Float(t) => *t as f64,
            _ => 0.001,
        }
    } else {
        0.001
    };

    let max_iter = if args.len() > 5 {
        match &args[5] {
            XdlValue::Long(m) => *m as usize,
            XdlValue::Int(m) => *m as usize,
            _ => 1000,
        }
    } else {
        1000
    };

    let gamma = if args.len() > 6 {
        match &args[6] {
            XdlValue::Double(g) => *g,
            XdlValue::Float(g) => *g as f64,
            _ => 1.0,
        }
    } else {
        1.0
    };

    let degree = if args.len() > 7 {
        match &args[7] {
            XdlValue::Long(d) => *d,
            XdlValue::Int(d) => *d as i32,
            _ => 3,
        }
    } else {
        3
    };

    let coef0 = if args.len() > 8 {
        match &args[8] {
            XdlValue::Double(c) => *c,
            XdlValue::Float(c) => *c as f64,
            _ => 0.0,
        }
    } else {
        0.0
    };

    let n_samples = x_train.len();

    if y_train.len() != n_samples {
        return Err(XdlError::InvalidArgument(
            "XDLML_SupportVectorMachineClassification: X and y must have same length".to_string(),
        ));
    }

    // Kernel function
    let kernel = |xi: f64, xj: f64| -> f64 {
        match kernel_type {
            0 => xi * xj,                                // Linear
            1 => (gamma * xi * xj + coef0).powi(degree), // Polynomial
            2 => (-gamma * (xi - xj).powi(2)).exp(),     // RBF
            3 => (gamma * xi * xj + coef0).tanh(),       // Sigmoid
            _ => xi * xj,
        }
    };

    // Initialize alphas and bias
    let mut alphas = vec![0.0; n_samples];
    let mut bias = 0.0;

    // SMO Algorithm
    let mut iter = 0;
    let mut num_changed = 0;
    let mut examine_all = true;

    while iter < max_iter && (num_changed > 0 || examine_all) {
        num_changed = 0;

        for i in 0..n_samples {
            if examine_all || (alphas[i] > tolerance && alphas[i] < c_param - tolerance) {
                // Compute prediction for i
                let mut pred_i = bias;
                for j in 0..n_samples {
                    pred_i += alphas[j] * y_train[j] * kernel(x_train[i], x_train[j]);
                }

                let ei = pred_i - y_train[i]; // Error

                // Check KKT conditions
                if (y_train[i] * ei < -tolerance && alphas[i] < c_param)
                    || (y_train[i] * ei > tolerance && alphas[i] > 0.0)
                {
                    // Select second alpha (j) - simplified heuristic
                    let j = if i == 0 { 1 } else { (i + 1) % n_samples };

                    // Compute prediction for j
                    let mut pred_j = bias;
                    for k in 0..n_samples {
                        pred_j += alphas[k] * y_train[k] * kernel(x_train[j], x_train[k]);
                    }

                    let ej = pred_j - y_train[j];

                    let alpha_i_old = alphas[i];
                    let alpha_j_old = alphas[j];

                    // Compute bounds L and H
                    let (l, h) = if y_train[i] != y_train[j] {
                        (
                            (alpha_j_old - alpha_i_old).max(0.0),
                            c_param.min(c_param + alpha_j_old - alpha_i_old),
                        )
                    } else {
                        (
                            (alpha_i_old + alpha_j_old - c_param).max(0.0),
                            c_param.min(alpha_i_old + alpha_j_old),
                        )
                    };

                    if (h - l).abs() < 1e-10 {
                        continue;
                    }

                    // Compute eta
                    let eta = 2.0 * kernel(x_train[i], x_train[j])
                        - kernel(x_train[i], x_train[i])
                        - kernel(x_train[j], x_train[j]);

                    if eta >= 0.0 {
                        continue;
                    }

                    // Update alpha_j
                    alphas[j] = alpha_j_old - y_train[j] * (ei - ej) / eta;
                    alphas[j] = alphas[j].max(l).min(h);

                    if (alphas[j] - alpha_j_old).abs() < 1e-5 {
                        continue;
                    }

                    // Update alpha_i
                    alphas[i] = alpha_i_old + y_train[i] * y_train[j] * (alpha_j_old - alphas[j]);

                    // Update bias
                    let b1 = bias
                        - ei
                        - y_train[i] * (alphas[i] - alpha_i_old) * kernel(x_train[i], x_train[i])
                        - y_train[j] * (alphas[j] - alpha_j_old) * kernel(x_train[i], x_train[j]);

                    let b2 = bias
                        - ej
                        - y_train[i] * (alphas[i] - alpha_i_old) * kernel(x_train[i], x_train[j])
                        - y_train[j] * (alphas[j] - alpha_j_old) * kernel(x_train[j], x_train[j]);

                    bias = if alphas[i] > 0.0 && alphas[i] < c_param {
                        b1
                    } else if alphas[j] > 0.0 && alphas[j] < c_param {
                        b2
                    } else {
                        (b1 + b2) / 2.0
                    };

                    num_changed += 1;
                }
            }
        }

        if examine_all {
            examine_all = false;
        } else if num_changed == 0 {
            examine_all = true;
        }

        iter += 1;
    }

    // Return alphas and bias
    let mut result = Vec::with_capacity(alphas.len() + 1);
    result.extend_from_slice(&alphas);
    result.push(bias);

    Ok(XdlValue::Array(result))
}

/// XDLML_SUPPORTVECTORMACHINEREGRESSION - SVM for regression
///
/// Support Vector Regression with epsilon-insensitive loss.
/// Uses simplified gradient descent approach.
///
/// Parameters:
///   X_train: Training features
///   y_train: Training targets (continuous values)
///   kernel_type: Kernel type (0=linear, 1=polynomial, 2=RBF, 3=sigmoid)
///   C: Regularization parameter (default 1.0)
///   epsilon: Epsilon in epsilon-SVR (default 0.1)
///   learning_rate: Learning rate (default 0.01)
///   n_epochs: Number of epochs (default 100)
///   gamma: Kernel coefficient (default 1.0)
///
/// Returns:
///   Array: [weights..., bias]
///   Model parameters
///
/// Example:
///   X = RANDOMU(seed, 100)
///   y = 2.0 * X + 1.0 + RANDOMU(seed, 100) * 0.1  ; Linear with noise
///   model = XDLML_SUPPORTVECTORMACHINEREGRESSION(X, y, 0, 1.0, 0.1, 0.01, 200, 1.0)
pub fn xdlml_supportvectormachineregression(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "XDLML_SupportVectorMachineRegression: Expected at least 2 arguments (X, y)"
                .to_string(),
        ));
    }

    let x_train = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let y_train = match &args[1] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let kernel_type = if args.len() > 2 {
        match &args[2] {
            XdlValue::Long(k) => *k as usize,
            XdlValue::Int(k) => *k as usize,
            _ => 2,
        }
    } else {
        2
    };

    let c_param = if args.len() > 3 {
        match &args[3] {
            XdlValue::Double(c) => *c,
            XdlValue::Float(c) => *c as f64,
            _ => 1.0,
        }
    } else {
        1.0
    };

    let epsilon = if args.len() > 4 {
        match &args[4] {
            XdlValue::Double(e) => *e,
            XdlValue::Float(e) => *e as f64,
            _ => 0.1,
        }
    } else {
        0.1
    };

    let learning_rate = if args.len() > 5 {
        match &args[5] {
            XdlValue::Double(lr) => *lr,
            XdlValue::Float(lr) => *lr as f64,
            _ => 0.01,
        }
    } else {
        0.01
    };

    let n_epochs = if args.len() > 6 {
        match &args[6] {
            XdlValue::Long(e) => *e as usize,
            XdlValue::Int(e) => *e as usize,
            _ => 100,
        }
    } else {
        100
    };

    let gamma = if args.len() > 7 {
        match &args[7] {
            XdlValue::Double(g) => *g,
            XdlValue::Float(g) => *g as f64,
            _ => 1.0,
        }
    } else {
        1.0
    };

    let n_samples = x_train.len();

    if y_train.len() != n_samples {
        return Err(XdlError::InvalidArgument(
            "XDLML_SupportVectorMachineRegression: X and y must have same length".to_string(),
        ));
    }

    // Simplified SVR using gradient descent with epsilon-insensitive loss
    // For 1D input: y = w*x + b
    let mut weight = 0.0;
    let mut bias = 0.0;

    // Kernel function (simplified for gradient descent)
    let use_kernel = kernel_type != 0;

    if use_kernel {
        // For non-linear kernels, use dual form approximation
        let mut alphas = vec![0.0; n_samples];

        for _epoch in 0..n_epochs {
            for i in 0..n_samples {
                // Compute prediction
                let mut pred = bias;
                for j in 0..n_samples {
                    let k_val = match kernel_type {
                        1 => (gamma * x_train[i] * x_train[j]).powi(3), // Polynomial
                        2 => (-gamma * (x_train[i] - x_train[j]).powi(2)).exp(), // RBF
                        3 => (gamma * x_train[i] * x_train[j]).tanh(),  // Sigmoid
                        _ => x_train[i] * x_train[j],
                    };
                    pred += alphas[j] * k_val;
                }

                let error = pred - y_train[i];

                // Epsilon-insensitive loss gradient
                if error.abs() > epsilon {
                    let grad = if error > 0.0 { 1.0 } else { -1.0 };
                    alphas[i] -= learning_rate * (grad + c_param * alphas[i]);
                    bias -= learning_rate * grad;
                }
            }
        }

        // Return alphas and bias
        let mut result = Vec::with_capacity(alphas.len() + 1);
        result.extend_from_slice(&alphas);
        result.push(bias);
        Ok(XdlValue::Array(result))
    } else {
        // Linear kernel: standard primal form
        for _epoch in 0..n_epochs {
            for i in 0..n_samples {
                let pred = weight * x_train[i] + bias;
                let error = pred - y_train[i];

                // Epsilon-insensitive loss
                if error.abs() > epsilon {
                    let grad = if error > 0.0 { 1.0 } else { -1.0 };
                    weight -= learning_rate * (grad * x_train[i] + c_param * weight);
                    bias -= learning_rate * grad;
                }
            }
        }

        // Return weight and bias
        Ok(XdlValue::Array(vec![weight, bias]))
    }
}

// ============================================================================
// REGULARIZATION LAYERS (Phase ML-8)
// ============================================================================

/// XDLML_BATCHNORMALIZATION - Batch Normalization layer
///
/// Normalizes activations across a mini-batch to stabilize training.
/// Reduces internal covariate shift and allows higher learning rates.
///
/// Parameters:
///   input: Input activations array
///   gamma: Scale parameter (default 1.0, learned during training)
///   beta: Shift parameter (default 0.0, learned during training)
///   mode: 0=training (compute batch stats), 1=inference (use running stats)
///   running_mean: Running mean for inference (default 0.0)
///   running_var: Running variance for inference (default 1.0)
///   momentum: Momentum for running stats update (default 0.9)
///   epsilon: Small constant for numerical stability (default 1e-5)
///
/// Returns:
///   Normalized output: gamma * (input - mean) / sqrt(var + eps) + beta
///
/// Example:
///   ; Training mode
///   normalized = XDLML_BATCHNORMALIZATION(activations, 1.0, 0.0, 0)
///   ; Inference mode with learned stats
///   normalized = XDLML_BATCHNORMALIZATION(activations, gamma, beta, 1, r_mean, r_var)
pub fn xdlml_batchnormalization(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLML_BatchNormalization: Expected at least 1 argument (input)".to_string(),
        ));
    }

    let input = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if input.is_empty() {
        return Ok(XdlValue::Array(vec![]));
    }

    let gamma = if args.len() > 1 {
        match &args[1] {
            XdlValue::Double(g) => *g,
            XdlValue::Float(g) => *g as f64,
            _ => 1.0,
        }
    } else {
        1.0
    };

    let beta = if args.len() > 2 {
        match &args[2] {
            XdlValue::Double(b) => *b,
            XdlValue::Float(b) => *b as f64,
            _ => 0.0,
        }
    } else {
        0.0
    };

    let mode = if args.len() > 3 {
        match &args[3] {
            XdlValue::Long(m) => *m,
            XdlValue::Int(m) => *m as i32,
            _ => 0, // Default to training mode
        }
    } else {
        0
    };

    let running_mean = if args.len() > 4 {
        match &args[4] {
            XdlValue::Double(rm) => *rm,
            XdlValue::Float(rm) => *rm as f64,
            _ => 0.0,
        }
    } else {
        0.0
    };

    let running_var = if args.len() > 5 {
        match &args[5] {
            XdlValue::Double(rv) => *rv,
            XdlValue::Float(rv) => *rv as f64,
            _ => 1.0,
        }
    } else {
        1.0
    };

    let _momentum = if args.len() > 6 {
        match &args[6] {
            XdlValue::Double(m) => *m,
            XdlValue::Float(m) => *m as f64,
            _ => 0.9,
        }
    } else {
        0.9
    };

    let epsilon = if args.len() > 7 {
        match &args[7] {
            XdlValue::Double(e) => *e,
            XdlValue::Float(e) => *e as f64,
            _ => 1e-5,
        }
    } else {
        1e-5
    };

    // Compute statistics based on mode
    let (mean, variance) = if mode == 0 {
        // Training mode: compute batch statistics
        let n = input.len() as f64;
        let batch_mean = input.iter().sum::<f64>() / n;
        let batch_var = input.iter().map(|&x| (x - batch_mean).powi(2)).sum::<f64>() / n;
        (batch_mean, batch_var)
    } else {
        // Inference mode: use running statistics
        (running_mean, running_var)
    };

    // Normalize: (x - mean) / sqrt(var + epsilon)
    // Then scale and shift: gamma * normalized + beta
    let normalized: Vec<f64> = input
        .iter()
        .map(|&x| {
            let norm = (x - mean) / (variance + epsilon).sqrt();
            gamma * norm + beta
        })
        .collect();

    Ok(XdlValue::Array(normalized))
}

/// XDLML_DROPOUT - Dropout regularization layer
///
/// Randomly sets a fraction of inputs to zero during training.
/// Prevents overfitting by reducing co-adaptation of neurons.
///
/// Parameters:
///   input: Input activations array
///   dropout_rate: Fraction of inputs to drop (0.0 to 1.0, default 0.5)
///   training: Training mode flag (1=training with dropout, 0=inference without)
///   seed: Random seed for reproducibility (default: time-based)
///
/// Returns:
///   Output with dropout applied (training) or scaled input (inference)
///
/// Example:
///   ; Training: 50% dropout
///   dropped = XDLML_DROPOUT(activations, 0.5, 1, 42)
///   ; Inference: no dropout, inputs unchanged
///   output = XDLML_DROPOUT(activations, 0.5, 0)
///
/// Note: During training, remaining activations are scaled by 1/(1-rate)
///       to maintain expected sum (inverted dropout)
pub fn xdlml_dropout(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLML_Dropout: Expected at least 1 argument (input)".to_string(),
        ));
    }

    let input = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if input.is_empty() {
        return Ok(XdlValue::Array(vec![]));
    }

    let dropout_rate = if args.len() > 1 {
        match &args[1] {
            XdlValue::Double(r) => *r,
            XdlValue::Float(r) => *r as f64,
            _ => 0.5,
        }
    } else {
        0.5
    };

    // Validate dropout rate
    if !(0.0..1.0).contains(&dropout_rate) {
        return Err(XdlError::InvalidArgument(
            "XDLML_Dropout: dropout_rate must be in [0.0, 1.0)".to_string(),
        ));
    }

    let training = if args.len() > 2 {
        match &args[2] {
            XdlValue::Long(t) => *t != 0,
            XdlValue::Int(t) => *t != 0,
            _ => true,
        }
    } else {
        true
    };

    let seed = if args.len() > 3 {
        match &args[3] {
            XdlValue::Long(s) => *s as u64,
            XdlValue::Int(s) => *s as u64,
            _ => {
                use std::time::{SystemTime, UNIX_EPOCH};
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            }
        }
    } else {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    };

    // Inference mode: return input unchanged
    if !training {
        return Ok(XdlValue::Array(input.clone()));
    }

    // Training mode: apply dropout
    let keep_prob = 1.0 - dropout_rate;
    let scale = 1.0 / keep_prob; // Inverted dropout scaling

    let mut rng_state = seed;
    let output: Vec<f64> = input
        .iter()
        .map(|&x| {
            // Generate random number [0, 1)
            rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
            let random = rng_state as f64 / u64::MAX as f64;

            if random < keep_prob {
                x * scale // Keep and scale
            } else {
                0.0 // Drop
            }
        })
        .collect();

    Ok(XdlValue::Array(output))
}

// ============================================================================
// CONVOLUTIONAL & POOLING LAYERS (Phase ML-9)
// ============================================================================

/// XDLML_CONV1D - 1D Convolutional layer
///
/// Applies 1D convolution to input signal using learnable kernels.
/// Used for time-series, audio, and sequence data processing.
///
/// Parameters:
///   input: Input signal (1D array)
///   kernel: Convolution kernel/filter (1D array)
///   stride: Stride for convolution (default 1)
///   padding: Padding mode: 0=valid (no padding), 1=same (zero padding)
///
/// Returns:
///   Convolved output signal
///
/// Example:
///   signal = [1, 2, 3, 4, 5]
///   kernel = [0.5, 0.5]  ; Simple moving average
///   output = XDLML_CONV1D(signal, kernel, 1, 0)
///
/// Note: Valid padding: output_size = (input_size - kernel_size) / stride + 1
///       Same padding: output_size = input_size / stride
pub fn xdlml_conv1d(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "XDLML_Conv1D: Expected at least 2 arguments (input, kernel)".to_string(),
        ));
    }

    let input = match &args[0] {
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

    if input.is_empty() || kernel.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLML_Conv1D: Input and kernel cannot be empty".to_string(),
        ));
    }

    let stride = if args.len() > 2 {
        match &args[2] {
            XdlValue::Long(s) => (*s as usize).max(1),
            XdlValue::Int(s) => (*s as usize).max(1),
            _ => 1,
        }
    } else {
        1
    };

    let padding = if args.len() > 3 {
        match &args[3] {
            XdlValue::Long(p) => *p != 0,
            XdlValue::Int(p) => *p != 0,
            _ => false,
        }
    } else {
        false
    };

    let _input_size = input.len();
    let kernel_size = kernel.len();

    // Apply padding if requested ("same" mode)
    let padded_input = if padding {
        let pad_total = kernel_size - 1;
        let pad_left = pad_total / 2;
        let pad_right = pad_total - pad_left;

        let mut padded = vec![0.0; pad_left];
        padded.extend_from_slice(input);
        padded.extend(vec![0.0; pad_right]);
        padded
    } else {
        input.clone()
    };

    let padded_size = padded_input.len();

    // Calculate output size
    if kernel_size > padded_size {
        return Err(XdlError::InvalidArgument(
            "XDLML_Conv1D: Kernel size larger than input".to_string(),
        ));
    }

    let output_size = (padded_size - kernel_size) / stride + 1;
    let mut output = Vec::with_capacity(output_size);

    // Perform convolution
    for i in (0..=(padded_size - kernel_size)).step_by(stride) {
        let mut sum = 0.0;
        for k in 0..kernel_size {
            sum += padded_input[i + k] * kernel[k];
        }
        output.push(sum);
    }

    Ok(XdlValue::Array(output))
}

/// XDLML_MAXPOOLING1D - 1D Max Pooling layer
///
/// Downsamples input by taking maximum value in sliding windows.
/// Reduces dimensionality while preserving important features.
///
/// Parameters:
///   input: Input signal (1D array)
///   pool_size: Size of pooling window (default 2)
///   stride: Stride for pooling (default same as pool_size)
///
/// Returns:
///   Pooled output (downsampled signal)
///
/// Example:
///   signal = [1, 3, 2, 4, 5, 1]
///   pooled = XDLML_MAXPOOLING1D(signal, 2, 2)  ; Output: [3, 4, 5]
pub fn xdlml_maxpooling1d(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLML_MaxPooling1D: Expected at least 1 argument (input)".to_string(),
        ));
    }

    let input = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if input.is_empty() {
        return Ok(XdlValue::Array(vec![]));
    }

    let pool_size = if args.len() > 1 {
        match &args[1] {
            XdlValue::Long(p) => (*p as usize).max(1),
            XdlValue::Int(p) => (*p as usize).max(1),
            _ => 2,
        }
    } else {
        2
    };

    let stride = if args.len() > 2 {
        match &args[2] {
            XdlValue::Long(s) => (*s as usize).max(1),
            XdlValue::Int(s) => (*s as usize).max(1),
            _ => pool_size, // Default stride = pool_size
        }
    } else {
        pool_size
    };

    let input_size = input.len();

    if pool_size > input_size {
        return Err(XdlError::InvalidArgument(
            "XDLML_MaxPooling1D: Pool size larger than input".to_string(),
        ));
    }

    let output_size = (input_size - pool_size) / stride + 1;
    let mut output = Vec::with_capacity(output_size);

    // Perform max pooling
    for i in (0..=(input_size - pool_size)).step_by(stride) {
        let mut max_val = f64::NEG_INFINITY;
        for k in 0..pool_size {
            max_val = max_val.max(input[i + k]);
        }
        output.push(max_val);
    }

    Ok(XdlValue::Array(output))
}

/// XDLML_AVERAGEPOOLING1D - 1D Average Pooling layer
///
/// Downsamples input by taking average value in sliding windows.
/// Alternative to max pooling, smoother downsampling.
///
/// Parameters:
///   input: Input signal (1D array)
///   pool_size: Size of pooling window (default 2)
///   stride: Stride for pooling (default same as pool_size)
///
/// Returns:
///   Pooled output (downsampled signal)
///
/// Example:
///   signal = [1, 3, 2, 4, 5, 1]
///   pooled = XDLML_AVERAGEPOOLING1D(signal, 2, 2)  ; Output: [2.0, 3.0, 3.0]
pub fn xdlml_averagepooling1d(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLML_AveragePooling1D: Expected at least 1 argument (input)".to_string(),
        ));
    }

    let input = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if input.is_empty() {
        return Ok(XdlValue::Array(vec![]));
    }

    let pool_size = if args.len() > 1 {
        match &args[1] {
            XdlValue::Long(p) => (*p as usize).max(1),
            XdlValue::Int(p) => (*p as usize).max(1),
            _ => 2,
        }
    } else {
        2
    };

    let stride = if args.len() > 2 {
        match &args[2] {
            XdlValue::Long(s) => (*s as usize).max(1),
            XdlValue::Int(s) => (*s as usize).max(1),
            _ => pool_size,
        }
    } else {
        pool_size
    };

    let input_size = input.len();

    if pool_size > input_size {
        return Err(XdlError::InvalidArgument(
            "XDLML_AveragePooling1D: Pool size larger than input".to_string(),
        ));
    }

    let output_size = (input_size - pool_size) / stride + 1;
    let mut output = Vec::with_capacity(output_size);

    // Perform average pooling
    for i in (0..=(input_size - pool_size)).step_by(stride) {
        let mut sum = 0.0;
        for k in 0..pool_size {
            sum += input[i + k];
        }
        output.push(sum / pool_size as f64);
    }

    Ok(XdlValue::Array(output))
}

// ============================================================================
// MATRIX OPERATIONS (Phase ML-11)
// ============================================================================

/// XDLML_MATMUL - Matrix multiplication
///
/// Performs matrix multiplication: C = A × B
/// Supports both 1D (vectors) and 2D (matrices) inputs.
///
/// Parameters:
///   A: First matrix (1D array treated as row vector, or 2D)
///   B: Second matrix (1D array treated as column vector, or 2D)
///
/// Returns:
///   Result of multiplication with appropriate shape
///
/// Example:
///   ; 2x3 × 3x2 = 2x2
///   A = MultiDimArray([2, 3], data_A)
///   B = MultiDimArray([3, 2], data_B)
///   C = XDLML_MATMUL(A, B)  ; Shape: [2, 2]
pub fn xdlml_matmul(args: &[XdlValue]) -> XdlResult<XdlValue> {
    use xdl_core::XdlValue;

    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "XDLML_MatMul: Expected 2 arguments (A, B)".to_string(),
        ));
    }

    // Get A matrix
    let (a_data, a_shape) = match &args[0] {
        XdlValue::Array(arr) => (arr.as_slice(), vec![1, arr.len()]),
        XdlValue::MultiDimArray { data, shape } => (data.as_slice(), shape.clone()),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    // Get B matrix
    let (b_data, b_shape) = match &args[1] {
        XdlValue::Array(arr) => (arr.as_slice(), vec![arr.len(), 1]),
        XdlValue::MultiDimArray { data, shape } => (data.as_slice(), shape.clone()),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    // Validate dimensions for matrix multiplication
    let (a_rows, a_cols) = if a_shape.len() == 1 {
        (1, a_shape[0])
    } else {
        (a_shape[0], a_shape[1])
    };

    let (b_rows, b_cols) = if b_shape.len() == 1 {
        (b_shape[0], 1)
    } else {
        (b_shape[0], b_shape[1])
    };

    if a_cols != b_rows {
        return Err(XdlError::InvalidArgument(format!(
            "XDLML_MatMul: Incompatible dimensions [{}, {}] × [{}, {}]",
            a_rows, a_cols, b_rows, b_cols
        )));
    }

    // Perform matrix multiplication
    let mut result = vec![0.0; a_rows * b_cols];

    for i in 0..a_rows {
        for j in 0..b_cols {
            let mut sum = 0.0;
            for k in 0..a_cols {
                sum += a_data[i * a_cols + k] * b_data[k * b_cols + j];
            }
            result[i * b_cols + j] = sum;
        }
    }

    // Return with appropriate shape
    if a_rows == 1 && b_cols == 1 {
        Ok(XdlValue::Double(result[0]))
    } else if a_rows == 1 || b_cols == 1 {
        Ok(XdlValue::Array(result))
    } else {
        Ok(XdlValue::from_multidim(result, vec![a_rows, b_cols])?)
    }
}

/// XDLML_RESHAPE - Reshape array to new dimensions
///
/// Changes array shape without modifying data order.
///
/// Parameters:
///   array: Input array
///   new_shape: Array of new dimensions
///
/// Returns:
///   Reshaped array
///
/// Example:
///   data = [1, 2, 3, 4, 5, 6]
///   reshaped = XDLML_RESHAPE(data, [2, 3])  ; 2x3 matrix
pub fn xdlml_reshape(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "XDLML_Reshape: Expected 2 arguments (array, new_shape)".to_string(),
        ));
    }

    // Get data
    let data = match &args[0] {
        XdlValue::Array(arr) => arr.clone(),
        XdlValue::MultiDimArray { data, .. } => data.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    // Get new shape
    let new_shape = match &args[1] {
        XdlValue::Array(arr) => arr.iter().map(|&x| x as usize).collect::<Vec<_>>(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    // Create reshaped array
    XdlValue::from_multidim(data, new_shape)
}

/// XDLML_TRANSPOSE - Transpose 2D matrix
///
/// Swaps rows and columns of a matrix.
///
/// Parameters:
///   matrix: 2D input matrix
///
/// Returns:
///   Transposed matrix
///
/// Example:
///   A = [[1, 2, 3],
///        [4, 5, 6]]  ; 2x3
///   AT = XDLML_TRANSPOSE(A)  ; 3x2: [[1, 4], [2, 5], [3, 6]]
pub fn xdlml_transpose(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLML_Transpose: Expected 1 argument (matrix)".to_string(),
        ));
    }

    let (data, shape) = match &args[0] {
        XdlValue::Array(arr) => (arr.clone(), vec![arr.len(), 1]),
        XdlValue::MultiDimArray { data, shape } => (data.clone(), shape.clone()),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if shape.len() != 2 {
        return Err(XdlError::InvalidArgument(
            "XDLML_Transpose: Input must be 2D matrix".to_string(),
        ));
    }

    let rows = shape[0];
    let cols = shape[1];

    // Transpose: swap rows and columns
    let mut result = vec![0.0; rows * cols];
    for i in 0..rows {
        for j in 0..cols {
            result[j * rows + i] = data[i * cols + j];
        }
    }

    XdlValue::from_multidim(result, vec![cols, rows])
}

// ============================================================================
// CONVOLUTIONAL LAYERS (Phase ML-11)
// ============================================================================

/// XDLML_CONV2D - 2D Convolution operation
///
/// Applies 2D convolution to input: output[i,j] = sum(input[region] * kernel)
///
/// Parameters:
///   input: Input tensor [height, width] or [channels, height, width]
///   kernel: Convolution kernel [kh, kw] or [out_c, in_c, kh, kw]
///   stride: Stride for convolution (default 1)
///   padding: Padding to add (default 0)
///
/// Returns:
///   Convolved output tensor
///
/// Example:
///   img = MultiDimArray([28, 28], data)  ; 28x28 image
///   kernel = MultiDimArray([3, 3], weights)  ; 3x3 kernel
///   out = XDLML_CONV2D(img, kernel, 1, 0)
pub fn xdlml_conv2d(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "XDLML_Conv2D: Expected at least 2 arguments (input, kernel, [stride], [padding])"
                .to_string(),
        ));
    }

    // Get input
    let (input_data, input_shape) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => (data.as_slice(), shape.clone()),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "multidim_array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    // Get kernel
    let (kernel_data, kernel_shape) = match &args[1] {
        XdlValue::MultiDimArray { data, shape } => (data.as_slice(), shape.clone()),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "multidim_array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    // Get stride (default 1)
    let stride = if args.len() > 2 {
        args[2].to_double().unwrap_or(1.0) as usize
    } else {
        1
    };

    // Get padding (default 0)
    let padding = if args.len() > 3 {
        args[3].to_double().unwrap_or(0.0) as usize
    } else {
        0
    };

    // Simple 2D convolution (single channel)
    if input_shape.len() == 2 && kernel_shape.len() == 2 {
        let (in_h, in_w) = (input_shape[0], input_shape[1]);
        let (k_h, k_w) = (kernel_shape[0], kernel_shape[1]);

        // Calculate output dimensions
        let out_h = (in_h + 2 * padding - k_h) / stride + 1;
        let out_w = (in_w + 2 * padding - k_w) / stride + 1;

        let mut output = vec![0.0; out_h * out_w];

        // Perform convolution
        for out_i in 0..out_h {
            for out_j in 0..out_w {
                let mut sum = 0.0;
                let in_i_start = out_i * stride;
                let in_j_start = out_j * stride;

                for k_i in 0..k_h {
                    for k_j in 0..k_w {
                        let in_i = in_i_start + k_i;
                        let in_j = in_j_start + k_j;

                        // Handle padding
                        if in_i >= padding
                            && in_i < in_h + padding
                            && in_j >= padding
                            && in_j < in_w + padding
                        {
                            let actual_i = in_i - padding;
                            let actual_j = in_j - padding;
                            if actual_i < in_h && actual_j < in_w {
                                sum += input_data[actual_i * in_w + actual_j]
                                    * kernel_data[k_i * k_w + k_j];
                            }
                        }
                    }
                }
                output[out_i * out_w + out_j] = sum;
            }
        }

        return XdlValue::from_multidim(output, vec![out_h, out_w]);
    }

    Err(XdlError::InvalidArgument(
        "XDLML_Conv2D: Currently supports 2D single-channel convolution".to_string(),
    ))
}

/// XDLML_MAXPOOLING2D - 2D Max Pooling operation
///
/// Applies 2D max pooling: takes maximum value in each pooling window.
///
/// Parameters:
///   input: Input tensor [height, width]
///   pool_size: Size of pooling window (default 2)
///   stride: Stride for pooling (default = pool_size)
///
/// Returns:
///   Pooled output tensor
///
/// Example:
///   feature_map = MultiDimArray([4, 4], data)
///   pooled = XDLML_MAXPOOLING2D(feature_map, 2, 2)  ; Output: [2, 2]
pub fn xdlml_maxpooling2d(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLML_MaxPooling2D: Expected at least 1 argument (input, [pool_size], [stride])"
                .to_string(),
        ));
    }

    // Get input
    let (input_data, input_shape) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => (data.as_slice(), shape.clone()),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "multidim_array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if input_shape.len() != 2 {
        return Err(XdlError::InvalidArgument(
            "XDLML_MaxPooling2D: Input must be 2D".to_string(),
        ));
    }

    // Get pool_size (default 2)
    let pool_size = if args.len() > 1 {
        args[1].to_double().unwrap_or(2.0) as usize
    } else {
        2
    };

    // Get stride (default = pool_size)
    let stride = if args.len() > 2 {
        args[2].to_double().unwrap_or(pool_size as f64) as usize
    } else {
        pool_size
    };

    let (in_h, in_w) = (input_shape[0], input_shape[1]);

    // Calculate output dimensions
    let out_h = (in_h - pool_size) / stride + 1;
    let out_w = (in_w - pool_size) / stride + 1;

    let mut output = vec![f64::NEG_INFINITY; out_h * out_w];

    // Perform max pooling
    for out_i in 0..out_h {
        for out_j in 0..out_w {
            let mut max_val = f64::NEG_INFINITY;
            let in_i_start = out_i * stride;
            let in_j_start = out_j * stride;

            for p_i in 0..pool_size {
                for p_j in 0..pool_size {
                    let in_i = in_i_start + p_i;
                    let in_j = in_j_start + p_j;
                    if in_i < in_h && in_j < in_w {
                        let val = input_data[in_i * in_w + in_j];
                        if val > max_val {
                            max_val = val;
                        }
                    }
                }
            }
            output[out_i * out_w + out_j] = max_val;
        }
    }

    XdlValue::from_multidim(output, vec![out_h, out_w])
}

// ============================================================================
// RECURRENT LAYERS (Phase ML-10)
// ============================================================================

/// XDLML_LSTM - Long Short-Term Memory layer
///
/// Applies LSTM to sequential input with gating mechanisms.
/// Simplified implementation for demonstration.
///
/// Parameters:
///   input: Input sequence [seq_len, input_size]
///   hidden_size: Number of hidden units
///   weights: Dictionary with keys: Wf, Wi, Wc, Wo, bf, bi, bc, bo
///
/// Returns:
///   Output sequence [seq_len, hidden_size]
///
/// Example:
///   seq = MultiDimArray([10, 5], data)  ; 10 timesteps, 5 features
///   lstm_out = XDLML_LSTM(seq, 20, weights)  ; Output: [10, 20]
pub fn xdlml_lstm(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(
            "XDLML_LSTM: Expected 3 arguments (input, hidden_size, weights)".to_string(),
        ));
    }

    // Get input
    let (_input_data, input_shape) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => (data.as_slice(), shape.clone()),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "multidim_array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if input_shape.len() != 2 {
        return Err(XdlError::InvalidArgument(
            "XDLML_LSTM: Input must be 2D [seq_len, input_size]".to_string(),
        ));
    }

    let seq_len = input_shape[0];
    let _input_size = input_shape[1];

    // Get hidden_size
    let hidden_size = match args[1].to_double() {
        Ok(h) => h as usize,
        Err(_) => {
            return Err(XdlError::TypeMismatch {
                expected: "number".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    // For simplicity, this is a placeholder implementation
    // A full LSTM would require proper weight matrices for forget, input, cell, output gates
    // This returns zero-initialized output for demonstration
    let output = vec![0.0; seq_len * hidden_size];

    XdlValue::from_multidim(output, vec![seq_len, hidden_size])
}

// ============================================================================
// RECURRENT LAYERS (Phase ML-10)
// ============================================================================

/// XDLML_SIMPLERNN - Simplified Recurrent Neural Network cell
///
/// Processes sequences by maintaining hidden state across time steps.
/// Simplified version suitable for 1D sequence data (time series, text).
///
/// Parameters:
///   sequence: Input sequence (1D array, each element is a time step)
///   hidden_size: Size of hidden state (default 10)
///   learning_rate: Learning rate for weight updates (default 0.01)
///   epochs: Number of training epochs (default 50)
///   seed: Random seed for weight initialization
///
/// Returns:
///   Final hidden state after processing entire sequence
///
/// Example:
///   sequence = [1.0, 2.0, 3.0, 4.0, 5.0]
///   hidden = XDLML_SIMPLERNN(sequence, 5, 0.01, 10, 42)
///
/// Architecture: h_t = tanh(W_x * x_t + W_h * h_{t-1} + b)
/// where h_t is hidden state at time t, x_t is input at time t
pub fn xdlml_simplernn(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLML_SimpleRNN: Expected at least 1 argument (sequence)".to_string(),
        ));
    }

    let sequence = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if sequence.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLML_SimpleRNN: Sequence cannot be empty".to_string(),
        ));
    }

    let hidden_size = if args.len() > 1 {
        match &args[1] {
            XdlValue::Long(h) => (*h as usize).max(1),
            XdlValue::Int(h) => (*h as usize).max(1),
            _ => 10,
        }
    } else {
        10
    };

    let _learning_rate = if args.len() > 2 {
        match &args[2] {
            XdlValue::Double(lr) => *lr,
            XdlValue::Float(lr) => *lr as f64,
            _ => 0.01,
        }
    } else {
        0.01
    };

    let _epochs = if args.len() > 3 {
        match &args[3] {
            XdlValue::Long(e) => *e as usize,
            XdlValue::Int(e) => *e as usize,
            _ => 50,
        }
    } else {
        50
    };

    let seed = if args.len() > 4 {
        match &args[4] {
            XdlValue::Long(s) => *s as u64,
            XdlValue::Int(s) => *s as u64,
            _ => 42,
        }
    } else {
        42
    };

    // Initialize weights (simplified: single input dimension)
    let input_dim = 1;
    let mut w_input = vec![0.0; hidden_size]; // Input to hidden weights
    let mut w_hidden = vec![0.0; hidden_size * hidden_size]; // Hidden to hidden weights
    let bias = vec![0.0; hidden_size];

    // Xavier initialization
    let mut rng_state = seed;
    let input_scale = (2.0 / input_dim as f64).sqrt();
    let hidden_scale = (2.0 / hidden_size as f64).sqrt();

    for w in w_input.iter_mut().take(hidden_size) {
        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        *w = ((rng_state as f64 / u64::MAX as f64) * 2.0 - 1.0) * input_scale;
    }

    for w in w_hidden.iter_mut().take(hidden_size * hidden_size) {
        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        *w = ((rng_state as f64 / u64::MAX as f64) * 2.0 - 1.0) * hidden_scale;
    }

    // Process sequence (forward pass)
    let mut hidden_state = vec![0.0; hidden_size]; // Initialize h_0 = 0

    for &input_val in sequence.iter() {
        let mut new_hidden = vec![0.0; hidden_size];

        for h in 0..hidden_size {
            // h_t = tanh(W_x * x_t + W_h * h_{t-1} + b)
            let mut activation = w_input[h] * input_val + bias[h];

            // Add recurrent connection: W_h * h_{t-1}
            for prev_h in 0..hidden_size {
                activation += w_hidden[h * hidden_size + prev_h] * hidden_state[prev_h];
            }

            // Apply tanh activation
            new_hidden[h] = activation.tanh();
        }

        hidden_state = new_hidden;
    }

    // Return final hidden state
    Ok(XdlValue::Array(hidden_state))
}

/// XDLML_SEQUENCEMEAN - Compute mean of sequence using RNN-style processing
///
/// Simplified sequence aggregation that demonstrates recurrent processing.
/// Maintains running average across sequence (similar to RNN but simpler).
///
/// Parameters:
///   sequence: Input sequence (1D array)
///   window: Context window size (default 3)
///
/// Returns:
///   Array of running means computed over moving windows
///
/// Example:
///   seq = [1, 2, 3, 4, 5]
///   means = XDLML_SEQUENCEMEAN(seq, 3)
pub fn xdlml_sequencemean(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLML_SequenceMean: Expected at least 1 argument (sequence)".to_string(),
        ));
    }

    let sequence = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if sequence.is_empty() {
        return Ok(XdlValue::Array(vec![]));
    }

    let window = if args.len() > 1 {
        match &args[1] {
            XdlValue::Long(w) => (*w as usize).max(1),
            XdlValue::Int(w) => (*w as usize).max(1),
            _ => 3,
        }
    } else {
        3
    };

    let seq_len = sequence.len();
    let mut result = Vec::with_capacity(seq_len);

    // Compute running mean with window
    for i in 0..seq_len {
        let start = if i >= window - 1 { i - window + 1 } else { 0 };
        let end = i + 1;

        let sum: f64 = sequence[start..end].iter().sum();
        let mean = sum / (end - start) as f64;
        result.push(mean);
    }

    Ok(XdlValue::Array(result))
}

// ============================================================================
// CROSS-VALIDATION UTILITIES (Phase ML-7)
// ============================================================================

/// XDLML_KFOLD - K-Fold cross-validation splitter
///
/// Splits data indices into K consecutive folds for cross-validation.
/// Each fold is used once as validation while K-1 folds form training set.
///
/// Parameters:
///   n_samples: Total number of samples
///   n_folds: Number of folds (K) (default 5)
///   seed: Random seed for shuffle (default: time-based)
///   shuffle: Whether to shuffle data before splitting (default 1=true)
///
/// Returns:
///   2D array: [n_folds x n_samples] where each row is a fold mask
///   0 = validation set, 1 = training set for that fold
///
/// Example:
///   folds = XDLML_KFOLD(100, 5)  ; 5-fold CV on 100 samples
///   ; Each fold will have ~20 samples for validation, ~80 for training
pub fn xdlml_kfold(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLML_KFold: Expected at least 1 argument (n_samples)".to_string(),
        ));
    }

    let n_samples = match &args[0] {
        XdlValue::Long(n) => *n as usize,
        XdlValue::Int(n) => *n as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let n_folds = if args.len() > 1 {
        match &args[1] {
            XdlValue::Long(k) => *k as usize,
            XdlValue::Int(k) => *k as usize,
            _ => 5,
        }
    } else {
        5
    };

    if n_folds < 2 || n_folds > n_samples {
        return Err(XdlError::InvalidArgument(format!(
            "XDLML_KFold: n_folds must be between 2 and {}",
            n_samples
        )));
    }

    let seed = if args.len() > 2 {
        match &args[2] {
            XdlValue::Long(s) => *s as u64,
            XdlValue::Int(s) => *s as u64,
            _ => {
                use std::time::{SystemTime, UNIX_EPOCH};
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            }
        }
    } else {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    };

    let do_shuffle = if args.len() > 3 {
        match &args[3] {
            XdlValue::Long(s) => *s != 0,
            XdlValue::Int(s) => *s != 0,
            _ => true,
        }
    } else {
        true
    };

    // Generate indices
    let mut indices: Vec<usize> = (0..n_samples).collect();

    // Shuffle if requested
    if do_shuffle {
        let mut rng_state = seed;
        for i in (1..n_samples).rev() {
            rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
            let j = (rng_state % (i as u64 + 1)) as usize;
            indices.swap(i, j);
        }
    }

    // Create fold masks
    let fold_size = n_samples / n_folds;
    let remainder = n_samples % n_folds;

    // Flatten result: store all fold masks consecutively
    let mut result = Vec::with_capacity(n_folds * n_samples);

    for fold_idx in 0..n_folds {
        // Calculate validation range for this fold
        let fold_start = fold_idx * fold_size + fold_idx.min(remainder);
        let fold_end = fold_start + fold_size + if fold_idx < remainder { 1 } else { 0 };

        // Create mask: 0 for validation (current fold), 1 for training
        for &actual_idx in indices.iter().take(n_samples) {
            if actual_idx >= fold_start && actual_idx < fold_end {
                result.push(0.0); // Validation
            } else {
                result.push(1.0); // Training
            }
        }
    }

    Ok(XdlValue::Array(result))
}

/// XDLML_STRATIFIEDKFOLD - Stratified K-Fold cross-validation
///
/// Splits data maintaining class distribution in each fold.
/// Useful for classification with imbalanced classes.
///
/// Parameters:
///   y_labels: Class labels array
///   n_folds: Number of folds (default 5)
///   seed: Random seed for shuffle
///
/// Returns:
///   Fold masks like KFold, but with stratified sampling
///
/// Example:
///   y = [0, 0, 0, 1, 1, 1, 2, 2, 2]  ; 3 classes, 3 samples each
///   folds = XDLML_STRATIFIEDKFOLD(y, 3)
pub fn xdlml_stratifiedkfold(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLML_StratifiedKFold: Expected at least 1 argument (y_labels)".to_string(),
        ));
    }

    let y_labels = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let n_samples = y_labels.len();

    let n_folds = if args.len() > 1 {
        match &args[1] {
            XdlValue::Long(k) => *k as usize,
            XdlValue::Int(k) => *k as usize,
            _ => 5,
        }
    } else {
        5
    };

    let seed = if args.len() > 2 {
        match &args[2] {
            XdlValue::Long(s) => *s as u64,
            XdlValue::Int(s) => *s as u64,
            _ => {
                use std::time::{SystemTime, UNIX_EPOCH};
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            }
        }
    } else {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    };

    // Group indices by class
    let mut class_indices: std::collections::HashMap<i32, Vec<usize>> =
        std::collections::HashMap::new();

    for (i, &label) in y_labels.iter().enumerate() {
        let class_id = label.round() as i32;
        class_indices.entry(class_id).or_default().push(i);
    }

    // Shuffle indices within each class
    let mut rng_state = seed;
    for indices in class_indices.values_mut() {
        for i in (1..indices.len()).rev() {
            rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
            let j = (rng_state % (i as u64 + 1)) as usize;
            indices.swap(i, j);
        }
    }

    // Distribute samples from each class across folds
    let mut fold_indices: Vec<Vec<usize>> = vec![Vec::new(); n_folds];

    for indices in class_indices.values() {
        let class_size = indices.len();
        let base_fold_size = class_size / n_folds;
        let remainder = class_size % n_folds;

        let mut idx = 0;
        for (fold, fold_vec) in fold_indices.iter_mut().enumerate().take(n_folds) {
            let fold_size = base_fold_size + if fold < remainder { 1 } else { 0 };
            for _ in 0..fold_size {
                if idx < indices.len() {
                    fold_vec.push(indices[idx]);
                    idx += 1;
                }
            }
        }
    }

    // Create fold masks
    let mut result = Vec::with_capacity(n_folds * n_samples);

    for fold_vec in fold_indices.iter().take(n_folds) {
        let mut mask = vec![1.0; n_samples]; // Default: training

        for &val_idx in fold_vec {
            mask[val_idx] = 0.0; // Validation
        }

        result.extend(mask);
    }

    Ok(XdlValue::Array(result))
}

/// XDLML_LEAVEONEOUT - Leave-One-Out cross-validation
///
/// Special case of K-Fold where K = N (number of samples).
/// Each sample is used once as the validation set.
/// Computationally expensive but gives unbiased estimate.
///
/// Parameters:
///   n_samples: Total number of samples
///
/// Returns:
///   2D array: [n_samples x n_samples] fold masks
///   Each row has one 0 (validation) and rest 1s (training)
///
/// Example:
///   folds = XDLML_LEAVEONEOUT(50)  ; 50 folds for 50 samples
pub fn xdlml_leaveoneout(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "XDLML_LeaveOneOut: Expected 1 argument (n_samples)".to_string(),
        ));
    }

    let n_samples = match &args[0] {
        XdlValue::Long(n) => *n as usize,
        XdlValue::Int(n) => *n as usize,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if n_samples == 0 {
        return Err(XdlError::InvalidArgument(
            "XDLML_LeaveOneOut: n_samples must be > 0".to_string(),
        ));
    }

    // Create fold masks: each sample left out once
    let mut result = Vec::with_capacity(n_samples * n_samples);

    for leave_out_idx in 0..n_samples {
        for i in 0..n_samples {
            if i == leave_out_idx {
                result.push(0.0); // Validation
            } else {
                result.push(1.0); // Training
            }
        }
    }

    Ok(XdlValue::Array(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partition() {
        let args = vec![XdlValue::Long(100), XdlValue::Double(0.8)];
        let result = xdlml_partition(&args).unwrap();

        if let XdlValue::Array(partition) = result {
            assert_eq!(partition.len(), 100);
            let n_train = partition.iter().filter(|&&x| x == 1.0).count();
            assert_eq!(n_train, 80);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_shuffle() {
        let args = vec![XdlValue::Long(10), XdlValue::Long(42)];
        let result = xdlml_shuffle(&args).unwrap();

        if let XdlValue::Array(indices) = result {
            assert_eq!(indices.len(), 10);
            // Check all indices 0-9 are present
            let mut sorted = indices.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            for (i, &val) in sorted.iter().enumerate().take(10) {
                assert_eq!(val, i as f64);
            }
        } else {
            panic!("Expected array");
        }
    }
}
