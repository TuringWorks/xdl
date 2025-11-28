//! Linfa Machine Learning integration for XDL
//!
//! Rust-native ML library (scikit-learn alternative)

use linfa::prelude::*;
use linfa_clustering::KMeans;
use linfa_linear::LinearRegression;
use linfa_nn::distance::L2Dist;
use linfa_reduction::Pca;
use ndarray::{Array1, Array2};
use std::collections::HashMap;
use std::sync::Mutex;
use xdl_core::{XdlError, XdlResult, XdlValue};

lazy_static::lazy_static! {
    static ref MODELS: Mutex<HashMap<String, ModelType>> = Mutex::new(HashMap::new());
    static ref MODEL_COUNTER: Mutex<usize> = Mutex::new(0);
}

// Store fitted models - using Box<dyn Any> pattern for type erasure
enum ModelType {
    KMeans(KMeans<f64, L2Dist>),
    LinearRegression(linfa_linear::FittedLinearRegression<f64>),
    Pca(linfa_reduction::Pca<f64>),
}

fn next_model_id(prefix: &str) -> String {
    let mut counter = MODEL_COUNTER.lock().unwrap();
    *counter += 1;
    format!("{}_{}", prefix, *counter)
}

fn xdl_to_array2(value: &XdlValue, n_cols: usize) -> XdlResult<Array2<f64>> {
    let flat: Vec<f64> = match value {
        XdlValue::Array(arr) => arr.clone(),
        XdlValue::MultiDimArray { data, .. } => data.clone(),
        _ => return Err(XdlError::RuntimeError("Expected numeric array".to_string())),
    };

    let n_rows = flat.len() / n_cols;
    if flat.len() != n_rows * n_cols {
        return Err(XdlError::RuntimeError(format!(
            "Array length {} not divisible by {} columns", flat.len(), n_cols
        )));
    }

    Array2::from_shape_vec((n_rows, n_cols), flat)
        .map_err(|e| XdlError::RuntimeError(format!("Failed to create array: {}", e)))
}

fn xdl_to_array1(value: &XdlValue) -> XdlResult<Array1<f64>> {
    let arr: Vec<f64> = match value {
        XdlValue::Array(arr) => arr.clone(),
        XdlValue::MultiDimArray { data, .. } => data.clone(),
        _ => return Err(XdlError::RuntimeError("Expected numeric array".to_string())),
    };
    Ok(Array1::from(arr))
}

fn array2_to_xdl(arr: &Array2<f64>) -> XdlValue {
    XdlValue::Array(arr.iter().cloned().collect())
}

fn array1_to_xdl(arr: &Array1<f64>) -> XdlValue {
    XdlValue::Array(arr.to_vec())
}

// =============================================================================
// K-Means Clustering
// =============================================================================

/// ML_KMEANS_FIT - Train K-Means model
/// Usage: model = ML_KMEANS_FIT(X, n_features, n_clusters, [max_iter], [tolerance])
pub fn ml_kmeans_fit(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::RuntimeError(
            "ML_KMEANS_FIT requires: X, n_features, n_clusters".to_string(),
        ));
    }

    let n_features = match &args[1] {
        XdlValue::Long(n) => *n as usize,
        _ => return Err(XdlError::RuntimeError("n_features must be integer".to_string())),
    };

    let x = xdl_to_array2(&args[0], n_features)?;

    let n_clusters = match &args[2] {
        XdlValue::Long(n) => *n as usize,
        _ => return Err(XdlError::RuntimeError("n_clusters must be integer".to_string())),
    };

    let max_iter = args.get(3).map(|v| match v {
        XdlValue::Long(n) => *n as u64,
        _ => 100,
    }).unwrap_or(100);

    let tolerance = args.get(4).map(|v| match v {
        XdlValue::Double(d) => *d,
        _ => 1e-4,
    }).unwrap_or(1e-4);

    let dataset = DatasetBase::from(x);

    let model = KMeans::params(n_clusters)
        .max_n_iterations(max_iter)
        .tolerance(tolerance)
        .fit(&dataset)
        .map_err(|e| XdlError::RuntimeError(format!("K-Means failed: {}", e)))?;

    let model_id = next_model_id("kmeans");
    let mut models = MODELS.lock().unwrap();
    models.insert(model_id.clone(), ModelType::KMeans(model));

    Ok(XdlValue::String(model_id))
}

/// ML_KMEANS_PREDICT - Predict clusters
/// Usage: labels = ML_KMEANS_PREDICT(model, X, n_features)
pub fn ml_kmeans_predict(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::RuntimeError(
            "ML_KMEANS_PREDICT requires: model_id, X, n_features".to_string(),
        ));
    }

    let model_id = match &args[0] {
        XdlValue::String(s) => s,
        _ => return Err(XdlError::RuntimeError("model_id must be string".to_string())),
    };

    let n_features = match &args[2] {
        XdlValue::Long(n) => *n as usize,
        _ => return Err(XdlError::RuntimeError("n_features must be integer".to_string())),
    };

    let x = xdl_to_array2(&args[1], n_features)?;

    let models = MODELS.lock().unwrap();
    let model = models.get(model_id)
        .ok_or_else(|| XdlError::RuntimeError(format!("Model not found: {}", model_id)))?;

    if let ModelType::KMeans(kmeans) = model {
        let dataset = DatasetBase::from(x);
        let predictions = kmeans.predict(&dataset);
        let labels: Vec<f64> = predictions.iter().map(|&x| x as f64).collect();
        Ok(XdlValue::Array(labels))
    } else {
        Err(XdlError::RuntimeError("Not a K-Means model".to_string()))
    }
}

/// ML_KMEANS_CENTROIDS - Get centroids
/// Usage: centroids = ML_KMEANS_CENTROIDS(model)
pub fn ml_kmeans_centroids(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError("ML_KMEANS_CENTROIDS requires model_id".to_string()));
    }

    let model_id = match &args[0] {
        XdlValue::String(s) => s,
        _ => return Err(XdlError::RuntimeError("model_id must be string".to_string())),
    };

    let models = MODELS.lock().unwrap();
    let model = models.get(model_id)
        .ok_or_else(|| XdlError::RuntimeError(format!("Model not found: {}", model_id)))?;

    if let ModelType::KMeans(kmeans) = model {
        Ok(array2_to_xdl(kmeans.centroids()))
    } else {
        Err(XdlError::RuntimeError("Not a K-Means model".to_string()))
    }
}

// =============================================================================
// Linear Regression
// =============================================================================

/// ML_LINEAR_FIT - Train linear regression
/// Usage: model = ML_LINEAR_FIT(X, y, n_features)
pub fn ml_linear_fit(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::RuntimeError(
            "ML_LINEAR_FIT requires: X, y, n_features".to_string(),
        ));
    }

    let n_features = match &args[2] {
        XdlValue::Long(n) => *n as usize,
        _ => return Err(XdlError::RuntimeError("n_features must be integer".to_string())),
    };

    let x = xdl_to_array2(&args[0], n_features)?;
    let y = xdl_to_array1(&args[1])?;

    let dataset = Dataset::new(x, y);

    let model = LinearRegression::default()
        .fit(&dataset)
        .map_err(|e| XdlError::RuntimeError(format!("Linear regression failed: {}", e)))?;

    let model_id = next_model_id("linear");
    let mut models = MODELS.lock().unwrap();
    models.insert(model_id.clone(), ModelType::LinearRegression(model));

    Ok(XdlValue::String(model_id))
}

/// ML_LINEAR_PREDICT - Predict with linear regression
/// Usage: predictions = ML_LINEAR_PREDICT(model, X, n_features)
pub fn ml_linear_predict(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::RuntimeError(
            "ML_LINEAR_PREDICT requires: model_id, X, n_features".to_string(),
        ));
    }

    let model_id = match &args[0] {
        XdlValue::String(s) => s,
        _ => return Err(XdlError::RuntimeError("model_id must be string".to_string())),
    };

    let n_features = match &args[2] {
        XdlValue::Long(n) => *n as usize,
        _ => return Err(XdlError::RuntimeError("n_features must be integer".to_string())),
    };

    let x = xdl_to_array2(&args[1], n_features)?;

    let models = MODELS.lock().unwrap();
    let model = models.get(model_id)
        .ok_or_else(|| XdlError::RuntimeError(format!("Model not found: {}", model_id)))?;

    if let ModelType::LinearRegression(linear) = model {
        let predictions = linear.predict(&x);
        Ok(array1_to_xdl(&predictions))
    } else {
        Err(XdlError::RuntimeError("Not a linear regression model".to_string()))
    }
}

/// ML_LINEAR_COEFFICIENTS - Get coefficients
/// Usage: coeffs = ML_LINEAR_COEFFICIENTS(model)
pub fn ml_linear_coefficients(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError("ML_LINEAR_COEFFICIENTS requires model_id".to_string()));
    }

    let model_id = match &args[0] {
        XdlValue::String(s) => s,
        _ => return Err(XdlError::RuntimeError("model_id must be string".to_string())),
    };

    let models = MODELS.lock().unwrap();
    let model = models.get(model_id)
        .ok_or_else(|| XdlError::RuntimeError(format!("Model not found: {}", model_id)))?;

    if let ModelType::LinearRegression(linear) = model {
        Ok(array1_to_xdl(linear.params()))
    } else {
        Err(XdlError::RuntimeError("Not a linear regression model".to_string()))
    }
}

/// ML_LINEAR_INTERCEPT - Get intercept
/// Usage: intercept = ML_LINEAR_INTERCEPT(model)
pub fn ml_linear_intercept(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError("ML_LINEAR_INTERCEPT requires model_id".to_string()));
    }

    let model_id = match &args[0] {
        XdlValue::String(s) => s,
        _ => return Err(XdlError::RuntimeError("model_id must be string".to_string())),
    };

    let models = MODELS.lock().unwrap();
    let model = models.get(model_id)
        .ok_or_else(|| XdlError::RuntimeError(format!("Model not found: {}", model_id)))?;

    if let ModelType::LinearRegression(linear) = model {
        Ok(XdlValue::Double(linear.intercept()))
    } else {
        Err(XdlError::RuntimeError("Not a linear regression model".to_string()))
    }
}

// =============================================================================
// Logistic Regression (simplified using linear + sigmoid)
// =============================================================================

/// ML_LOGISTIC_FIT - Train logistic regression
/// Usage: model = ML_LOGISTIC_FIT(X, y, n_features)
pub fn ml_logistic_fit(args: &[XdlValue]) -> XdlResult<XdlValue> {
    // Use linear regression as base, apply sigmoid at prediction time
    ml_linear_fit(args)
}

/// ML_LOGISTIC_PREDICT - Predict with logistic regression
/// Usage: predictions = ML_LOGISTIC_PREDICT(model, X, n_features)
pub fn ml_logistic_predict(args: &[XdlValue]) -> XdlResult<XdlValue> {
    let linear_result = ml_linear_predict(args)?;

    if let XdlValue::Array(predictions) = linear_result {
        let labels: Vec<f64> = predictions.iter()
            .map(|&p| {
                let sigmoid = 1.0 / (1.0 + (-p).exp());
                if sigmoid > 0.5 { 1.0 } else { 0.0 }
            })
            .collect();
        Ok(XdlValue::Array(labels))
    } else {
        Err(XdlError::RuntimeError("Unexpected prediction result".to_string()))
    }
}

// =============================================================================
// PCA (Principal Component Analysis)
// =============================================================================

/// ML_PCA_FIT - Fit PCA model
/// Usage: model = ML_PCA_FIT(X, n_features, n_components)
pub fn ml_pca_fit(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::RuntimeError(
            "ML_PCA_FIT requires: X, n_features, n_components".to_string(),
        ));
    }

    let n_features = match &args[1] {
        XdlValue::Long(n) => *n as usize,
        _ => return Err(XdlError::RuntimeError("n_features must be integer".to_string())),
    };

    let x = xdl_to_array2(&args[0], n_features)?;

    let n_components = match &args[2] {
        XdlValue::Long(n) => *n as usize,
        _ => return Err(XdlError::RuntimeError("n_components must be integer".to_string())),
    };

    let dataset = DatasetBase::from(x);

    let model = Pca::params(n_components)
        .fit(&dataset)
        .map_err(|e| XdlError::RuntimeError(format!("PCA failed: {}", e)))?;

    let model_id = next_model_id("pca");
    let mut models = MODELS.lock().unwrap();
    models.insert(model_id.clone(), ModelType::Pca(model));

    Ok(XdlValue::String(model_id))
}

/// ML_PCA_TRANSFORM - Transform data
/// Usage: X_transformed = ML_PCA_TRANSFORM(model, X, n_features)
pub fn ml_pca_transform(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::RuntimeError(
            "ML_PCA_TRANSFORM requires: model_id, X, n_features".to_string(),
        ));
    }

    let model_id = match &args[0] {
        XdlValue::String(s) => s,
        _ => return Err(XdlError::RuntimeError("model_id must be string".to_string())),
    };

    let n_features = match &args[2] {
        XdlValue::Long(n) => *n as usize,
        _ => return Err(XdlError::RuntimeError("n_features must be integer".to_string())),
    };

    let x = xdl_to_array2(&args[1], n_features)?;

    let models = MODELS.lock().unwrap();
    let model = models.get(model_id)
        .ok_or_else(|| XdlError::RuntimeError(format!("Model not found: {}", model_id)))?;

    if let ModelType::Pca(pca) = model {
        let dataset = DatasetBase::from(x);
        let transformed = pca.transform(dataset);
        Ok(array2_to_xdl(transformed.records()))
    } else {
        Err(XdlError::RuntimeError("Not a PCA model".to_string()))
    }
}

/// ML_PCA_COMPONENTS - Get components
/// Usage: components = ML_PCA_COMPONENTS(model)
pub fn ml_pca_components(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError("ML_PCA_COMPONENTS requires model_id".to_string()));
    }

    let model_id = match &args[0] {
        XdlValue::String(s) => s,
        _ => return Err(XdlError::RuntimeError("model_id must be string".to_string())),
    };

    let models = MODELS.lock().unwrap();
    let model = models.get(model_id)
        .ok_or_else(|| XdlError::RuntimeError(format!("Model not found: {}", model_id)))?;

    if let ModelType::Pca(pca) = model {
        Ok(array2_to_xdl(pca.components()))
    } else {
        Err(XdlError::RuntimeError("Not a PCA model".to_string()))
    }
}

/// ML_PCA_VARIANCE - Get explained variance
/// Usage: variance = ML_PCA_VARIANCE(model)
pub fn ml_pca_variance(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError("ML_PCA_VARIANCE requires model_id".to_string()));
    }

    let model_id = match &args[0] {
        XdlValue::String(s) => s,
        _ => return Err(XdlError::RuntimeError("model_id must be string".to_string())),
    };

    let models = MODELS.lock().unwrap();
    let model = models.get(model_id)
        .ok_or_else(|| XdlError::RuntimeError(format!("Model not found: {}", model_id)))?;

    if let ModelType::Pca(pca) = model {
        let variance = pca.explained_variance_ratio();
        Ok(array1_to_xdl(&variance))
    } else {
        Err(XdlError::RuntimeError("Not a PCA model".to_string()))
    }
}

// =============================================================================
// Utility Functions
// =============================================================================

/// ML_TRAIN_TEST_SPLIT - Split data into training and test sets
/// Usage: [X_train, X_test, y_train, y_test] = ML_TRAIN_TEST_SPLIT(X, y, n_features, test_ratio)
pub fn ml_train_test_split(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 4 {
        return Err(XdlError::RuntimeError(
            "ML_TRAIN_TEST_SPLIT requires: X, y, n_features, test_ratio".to_string(),
        ));
    }

    let n_features = match &args[2] {
        XdlValue::Long(n) => *n as usize,
        _ => return Err(XdlError::RuntimeError("n_features must be integer".to_string())),
    };

    let x = xdl_to_array2(&args[0], n_features)?;
    let y = xdl_to_array1(&args[1])?;

    let test_ratio = match &args[3] {
        XdlValue::Double(d) => *d,
        XdlValue::Float(f) => *f as f64,
        _ => return Err(XdlError::RuntimeError("test_ratio must be float".to_string())),
    };

    let n_samples = x.nrows();
    let n_test = (n_samples as f64 * test_ratio).round() as usize;
    let n_train = n_samples - n_test;

    let x_train = x.slice(ndarray::s![..n_train, ..]).to_owned();
    let x_test = x.slice(ndarray::s![n_train.., ..]).to_owned();
    let y_train = y.slice(ndarray::s![..n_train]).to_owned();
    let y_test = y.slice(ndarray::s![n_train..]).to_owned();

    Ok(XdlValue::NestedArray(vec![
        array2_to_xdl(&x_train),
        array2_to_xdl(&x_test),
        array1_to_xdl(&y_train),
        array1_to_xdl(&y_test),
    ]))
}

/// ML_ACCURACY - Calculate accuracy score
/// Usage: accuracy = ML_ACCURACY(y_true, y_pred)
pub fn ml_accuracy(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::RuntimeError("ML_ACCURACY requires: y_true, y_pred".to_string()));
    }

    let y_true = xdl_to_array1(&args[0])?;
    let y_pred = xdl_to_array1(&args[1])?;

    if y_true.len() != y_pred.len() {
        return Err(XdlError::RuntimeError("Arrays must have same length".to_string()));
    }

    let correct: usize = y_true.iter().zip(y_pred.iter())
        .filter(|(&t, &p)| (t - p).abs() < 1e-10)
        .count();

    Ok(XdlValue::Double(correct as f64 / y_true.len() as f64))
}

/// ML_MSE - Mean squared error
/// Usage: mse = ML_MSE(y_true, y_pred)
pub fn ml_mse(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::RuntimeError("ML_MSE requires: y_true, y_pred".to_string()));
    }

    let y_true = xdl_to_array1(&args[0])?;
    let y_pred = xdl_to_array1(&args[1])?;

    if y_true.len() != y_pred.len() {
        return Err(XdlError::RuntimeError("Arrays must have same length".to_string()));
    }

    let mse: f64 = y_true.iter().zip(y_pred.iter())
        .map(|(&t, &p)| (t - p).powi(2))
        .sum::<f64>() / y_true.len() as f64;

    Ok(XdlValue::Double(mse))
}

/// ML_R2_SCORE - R² score (coefficient of determination)
/// Usage: r2 = ML_R2_SCORE(y_true, y_pred)
pub fn ml_r2_score(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::RuntimeError("ML_R2_SCORE requires: y_true, y_pred".to_string()));
    }

    let y_true = xdl_to_array1(&args[0])?;
    let y_pred = xdl_to_array1(&args[1])?;

    if y_true.len() != y_pred.len() {
        return Err(XdlError::RuntimeError("Arrays must have same length".to_string()));
    }

    let mean = y_true.mean().unwrap_or(0.0);
    let ss_res: f64 = y_true.iter().zip(y_pred.iter())
        .map(|(&t, &p)| (t - p).powi(2)).sum();
    let ss_tot: f64 = y_true.iter().map(|&t| (t - mean).powi(2)).sum();

    let r2 = if ss_tot > 0.0 { 1.0 - ss_res / ss_tot } else { 0.0 };
    Ok(XdlValue::Double(r2))
}

/// ML_DROP_MODEL - Remove model from memory
/// Usage: ML_DROP_MODEL, model_id
pub fn ml_drop_model(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError("ML_DROP_MODEL requires model_id".to_string()));
    }

    let model_id = match &args[0] {
        XdlValue::String(s) => s,
        _ => return Err(XdlError::RuntimeError("model_id must be string".to_string())),
    };

    let mut models = MODELS.lock().unwrap();
    models.remove(model_id);
    Ok(XdlValue::Undefined)
}
