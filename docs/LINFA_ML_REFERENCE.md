# XDL Native Machine Learning Reference (Linfa)

**Version**: 1.0
**Date**: November 2025
**Status**: Complete ✅
**Feature Flag**: `ml`

---

## Overview

XDL includes native Rust machine learning capabilities powered by [Linfa](https://github.com/rust-ml/linfa), a Rust machine learning framework inspired by scikit-learn. These functions provide:

- **Pure Rust**: No Python dependencies required
- **Performance**: Native speed with zero-copy data handling
- **Memory Safety**: Rust's guarantees apply to ML operations
- **Integration**: Direct use of XDL arrays

---

## Enabling Native ML

Native ML functions require the `ml` feature flag:

```bash
# Build with ML support
cargo build --features ml

# Or in Cargo.toml
[dependencies]
xdl-stdlib = { version = "0.1", features = ["ml"] }
```

---

## Function Reference

### K-Means Clustering

#### `ML_KMEANS_FIT(X, n_features, n_clusters, [max_iter], [tolerance])`

Train a K-Means clustering model.

**Parameters:**
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `X` | array | required | Input data (flattened) |
| `n_features` | integer | required | Number of features per sample |
| `n_clusters` | integer | required | Number of clusters |
| `max_iter` | integer | `100` | Maximum iterations |
| `tolerance` | float | `1e-4` | Convergence threshold |

**Returns:** Model ID (string)

**Example:**
```idl
; Create sample data (100 samples, 2 features)
X = RANDOMN(seed, 200)  ; 100 * 2 = 200 values

; Fit K-Means with 3 clusters
model = ML_KMEANS_FIT(X, 2, 3)
```

---

#### `ML_KMEANS_PREDICT(model_id, X, n_features)`

Predict cluster labels for new data.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `model_id` | string | Fitted model ID |
| `X` | array | Input data (flattened) |
| `n_features` | integer | Number of features |

**Returns:** Array of cluster labels (0 to k-1)

**Example:**
```idl
labels = ML_KMEANS_PREDICT(model, X, 2)
PRINT, 'Cluster assignments:', labels
```

---

#### `ML_KMEANS_CENTROIDS(model_id)`

Get cluster centroids.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `model_id` | string | Fitted model ID |

**Returns:** Flattened array of centroids

**Example:**
```idl
centroids = ML_KMEANS_CENTROIDS(model)
```

---

### Linear Regression

#### `ML_LINEAR_FIT(X, y, n_features)`

Train a linear regression model.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `X` | array | Feature matrix (flattened) |
| `y` | array | Target values |
| `n_features` | integer | Number of features |

**Returns:** Model ID (string)

**Example:**
```idl
; Features: [x1, x2] for 100 samples
X = RANDOMN(seed, 200)
; Target: y = 2*x1 + 3*x2 + noise
y = FLTARR(100)
FOR i = 0, 99 DO y[i] = 2*X[i*2] + 3*X[i*2+1] + RANDOMN(seed2)

model = ML_LINEAR_FIT(X, y, 2)
```

---

#### `ML_LINEAR_PREDICT(model_id, X, n_features)`

Predict with linear regression model.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `model_id` | string | Fitted model ID |
| `X` | array | Feature matrix (flattened) |
| `n_features` | integer | Number of features |

**Returns:** Array of predictions

**Example:**
```idl
predictions = ML_LINEAR_PREDICT(model, X_test, 2)
```

---

#### `ML_LINEAR_COEFFICIENTS(model_id)`

Get regression coefficients.

**Returns:** Array of coefficients (one per feature)

**Example:**
```idl
coeffs = ML_LINEAR_COEFFICIENTS(model)
PRINT, 'Coefficients:', coeffs
```

---

#### `ML_LINEAR_INTERCEPT(model_id)`

Get regression intercept.

**Returns:** Scalar intercept value

**Example:**
```idl
intercept = ML_LINEAR_INTERCEPT(model)
PRINT, 'Intercept:', intercept
```

---

### Logistic Regression

#### `ML_LOGISTIC_FIT(X, y, n_features)`

Train a logistic regression classifier.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `X` | array | Feature matrix (flattened) |
| `y` | array | Binary labels (0 or 1) |
| `n_features` | integer | Number of features |

**Returns:** Model ID (string)

**Example:**
```idl
; Binary classification
model = ML_LOGISTIC_FIT(X_train, y_train, 4)
```

---

#### `ML_LOGISTIC_PREDICT(model_id, X, n_features)`

Predict class labels.

**Returns:** Array of predicted labels (0 or 1)

**Example:**
```idl
predictions = ML_LOGISTIC_PREDICT(model, X_test, 4)
```

---

### Principal Component Analysis (PCA)

#### `ML_PCA_FIT(X, n_features, n_components)`

Fit a PCA model for dimensionality reduction.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `X` | array | Input data (flattened) |
| `n_features` | integer | Number of original features |
| `n_components` | integer | Number of components to keep |

**Returns:** Model ID (string)

**Example:**
```idl
; Reduce 10 features to 2 components
model = ML_PCA_FIT(X, 10, 2)
```

---

#### `ML_PCA_TRANSFORM(model_id, X, n_features)`

Transform data to reduced dimensions.

**Returns:** Transformed data (flattened)

**Example:**
```idl
X_reduced = ML_PCA_TRANSFORM(model, X, 10)
```

---

#### `ML_PCA_COMPONENTS(model_id)`

Get principal components.

**Returns:** Flattened components matrix

**Example:**
```idl
components = ML_PCA_COMPONENTS(model)
```

---

#### `ML_PCA_VARIANCE(model_id)`

Get explained variance ratio.

**Returns:** Array of variance ratios (sums to ~1.0)

**Example:**
```idl
variance = ML_PCA_VARIANCE(model)
PRINT, 'Explained variance:', variance
PRINT, 'Total:', TOTAL(variance)
```

---

### Model Evaluation

#### `ML_TRAIN_TEST_SPLIT(X, y, n_features, test_ratio)`

Split data into training and test sets.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `X` | array | Feature matrix (flattened) |
| `y` | array | Target values |
| `n_features` | integer | Number of features |
| `test_ratio` | float | Fraction for test set (0-1) |

**Returns:** Array of 4 elements: `[X_train, X_test, y_train, y_test]`

**Example:**
```idl
split = ML_TRAIN_TEST_SPLIT(X, y, 4, 0.2)
X_train = split[0]
X_test = split[1]
y_train = split[2]
y_test = split[3]
```

---

#### `ML_ACCURACY(y_true, y_pred)`

Calculate classification accuracy.

**Returns:** Accuracy score (0 to 1)

**Example:**
```idl
acc = ML_ACCURACY(y_test, predictions)
PRINT, 'Accuracy:', acc * 100, '%'
```

---

#### `ML_MSE(y_true, y_pred)`

Calculate mean squared error.

**Returns:** MSE value

**Example:**
```idl
mse = ML_MSE(y_test, predictions)
PRINT, 'MSE:', mse
```

---

#### `ML_R2_SCORE(y_true, y_pred)`

Calculate R² coefficient of determination.

**Returns:** R² score (-∞ to 1, higher is better)

**Example:**
```idl
r2 = ML_R2_SCORE(y_test, predictions)
PRINT, 'R²:', r2
```

---

### Memory Management

#### `ML_DROP_MODEL(model_id)`

Remove model from memory.

**Example:**
```idl
ML_DROP_MODEL, model
```

---

## Complete Examples

### K-Means Clustering Example

```idl
; Generate synthetic data: 3 clusters
n_samples = 300
n_features = 2

; Cluster 1: center (0, 0)
X1 = RANDOMN(seed1, 200) * 0.5
; Cluster 2: center (3, 3)
X2 = RANDOMN(seed2, 200) * 0.5 + 3.0
; Cluster 3: center (0, 3)
X3 = RANDOMN(seed3, 200) * 0.5
FOR i = 0, 99 DO X3[i*2+1] = X3[i*2+1] + 3.0

; Combine data
X = [X1, X2, X3]

; Fit K-Means
model = ML_KMEANS_FIT(X, 2, 3, 100, 1e-4)

; Predict clusters
labels = ML_KMEANS_PREDICT(model, X, 2)

; Get centroids
centroids = ML_KMEANS_CENTROIDS(model)
PRINT, 'Cluster centroids:', centroids

; Clean up
ML_DROP_MODEL, model
```

### Linear Regression Example

```idl
; Generate data: y = 2*x + 1 + noise
n_samples = 100
X = FINDGEN(n_samples) / 10.0
y = 2.0 * X + 1.0 + RANDOMN(seed, n_samples) * 0.5

; Split data
split = ML_TRAIN_TEST_SPLIT(X, y, 1, 0.2)
X_train = split[0]
X_test = split[1]
y_train = split[2]
y_test = split[3]

; Fit model
model = ML_LINEAR_FIT(X_train, y_train, 1)

; Get coefficients
coef = ML_LINEAR_COEFFICIENTS(model)
intercept = ML_LINEAR_INTERCEPT(model)
PRINT, 'Learned: y =', coef[0], '* x +', intercept

; Predict and evaluate
y_pred = ML_LINEAR_PREDICT(model, X_test, 1)
mse = ML_MSE(y_test, y_pred)
r2 = ML_R2_SCORE(y_test, y_pred)
PRINT, 'MSE:', mse
PRINT, 'R²:', r2

; Clean up
ML_DROP_MODEL, model
```

### PCA Dimensionality Reduction

```idl
; High-dimensional data (10 features)
n_samples = 200
n_features = 10
X = RANDOMN(seed, n_samples * n_features)

; Reduce to 2 components
model = ML_PCA_FIT(X, n_features, 2)

; Transform data
X_reduced = ML_PCA_TRANSFORM(model, X, n_features)

; Check explained variance
variance = ML_PCA_VARIANCE(model)
PRINT, 'Component 1 explains:', variance[0] * 100, '% variance'
PRINT, 'Component 2 explains:', variance[1] * 100, '% variance'
PRINT, 'Total explained:', TOTAL(variance) * 100, '%'

; Clean up
ML_DROP_MODEL, model
```

---

## Data Format Notes

### Flattened Arrays

All Linfa ML functions expect data in **row-major flattened** format:

```
For 3 samples with 2 features each:
Sample 0: [x0, y0]
Sample 1: [x1, y1]
Sample 2: [x2, y2]

Flattened: [x0, y0, x1, y1, x2, y2]
```

### Converting Multi-dimensional Arrays

```idl
; If you have a 2D array, flatten it:
X_2d = REFORM(X_2d, N_ELEMENTS(X_2d))
```

---

## Comparison with XDLML Functions

XDL has two ML systems:

| Feature | XDLML_* Functions | ML_* Functions (Linfa) |
|---------|------------------|----------------------|
| Implementation | Custom XDL | Linfa (Rust) |
| Speed | Good | Excellent |
| Features | 50+ functions | Core algorithms |
| Neural Networks | Yes | No |
| SVM | Yes | No |
| K-Means | Yes | Yes |
| Linear Regression | Yes | Yes |
| PCA | Yes | Yes |

Use **XDLML_*** for neural networks and advanced features.
Use **ML_*** for fastest native performance on core algorithms.
