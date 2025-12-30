//! Linear Algebra functions for XDL
//!
//! Implements matrix operations using nalgebra

use nalgebra::DMatrix;
use xdl_core::{XdlError, XdlResult, XdlValue};

/// IDENTITY - Create identity matrix
/// IDENTITY(n) creates an n×n identity matrix
pub fn identity(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "IDENTITY: Expected 1 argument (size), got {}",
            args.len()
        )));
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

    if n == 0 {
        return Err(XdlError::InvalidArgument(
            "IDENTITY: Size must be positive".to_string(),
        ));
    }

    // Create identity matrix
    let identity = DMatrix::<f64>::identity(n, n);
    let data: Vec<f64> = identity.iter().copied().collect();

    Ok(XdlValue::MultiDimArray {
        data,
        shape: vec![n, n],
    })
}

/// INVERT - Matrix inversion
/// INVERT(matrix) returns the inverse of a square matrix
pub fn invert(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "INVERT: Expected matrix argument".to_string(),
        ));
    }

    let (data, shape) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => (data.clone(), shape.clone()),
        XdlValue::Array(arr) => {
            // Assume square matrix, try to find dimensions
            let n = (arr.len() as f64).sqrt() as usize;
            if n * n != arr.len() {
                return Err(XdlError::DimensionError(
                    "INVERT: Array is not a square matrix. Use REFORM to specify dimensions."
                        .to_string(),
                ));
            }
            (arr.clone(), vec![n, n])
        }
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array or matrix".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if shape.len() != 2 || shape[0] != shape[1] {
        return Err(XdlError::DimensionError(
            "INVERT: Matrix must be square".to_string(),
        ));
    }

    let n = shape[0];

    // Convert to nalgebra matrix (column-major)
    let matrix = DMatrix::from_row_slice(n, n, &data);

    // Compute inverse
    match matrix.try_inverse() {
        Some(inv) => {
            let result_data: Vec<f64> = inv.iter().copied().collect();
            Ok(XdlValue::MultiDimArray {
                data: result_data,
                shape,
            })
        }
        None => Err(XdlError::RuntimeError(
            "INVERT: Matrix is singular (non-invertible)".to_string(),
        )),
    }
}

/// DETERM - Compute determinant
/// DETERM(matrix) returns the determinant of a square matrix
pub fn determ(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "DETERM: Expected matrix argument".to_string(),
        ));
    }

    let (data, shape) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => (data.clone(), shape.clone()),
        XdlValue::Array(arr) => {
            let n = (arr.len() as f64).sqrt() as usize;
            if n * n != arr.len() {
                return Err(XdlError::DimensionError(
                    "DETERM: Array is not a square matrix".to_string(),
                ));
            }
            (arr.clone(), vec![n, n])
        }
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array or matrix".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if shape.len() != 2 || shape[0] != shape[1] {
        return Err(XdlError::DimensionError(
            "DETERM: Matrix must be square".to_string(),
        ));
    }

    let n = shape[0];
    let matrix = DMatrix::from_row_slice(n, n, &data);
    let det = matrix.determinant();

    Ok(XdlValue::Double(det))
}

/// CROSSP - Cross product of 3D vectors
/// CROSSP(v1, v2) returns the cross product v1 × v2
pub fn crossp(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 2 {
        return Err(XdlError::InvalidArgument(format!(
            "CROSSP: Expected 2 arguments (vectors), got {}",
            args.len()
        )));
    }

    let v1 = match &args[0] {
        XdlValue::Array(arr) => {
            if arr.len() != 3 {
                return Err(XdlError::DimensionError(
                    "CROSSP: Vectors must have 3 elements".to_string(),
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

    let v2 = match &args[1] {
        XdlValue::Array(arr) => {
            if arr.len() != 3 {
                return Err(XdlError::DimensionError(
                    "CROSSP: Vectors must have 3 elements".to_string(),
                ));
            }
            arr.clone()
        }
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    // Cross product: (a, b, c) × (d, e, f) = (bf-ce, cd-af, ae-bd)
    let result = vec![
        v1[1] * v2[2] - v1[2] * v2[1],
        v1[2] * v2[0] - v1[0] * v2[2],
        v1[0] * v2[1] - v1[1] * v2[0],
    ];

    Ok(XdlValue::Array(result))
}

/// DOTP - Dot product
/// DOTP(v1, v2) returns the dot product v1 · v2
pub fn dotp(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 2 {
        return Err(XdlError::InvalidArgument(format!(
            "DOTP: Expected 2 arguments (vectors), got {}",
            args.len()
        )));
    }

    let v1 = match &args[0] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let v2 = match &args[1] {
        XdlValue::Array(arr) => arr,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    if v1.len() != v2.len() {
        return Err(XdlError::DimensionError(
            "DOTP: Vectors must have same length".to_string(),
        ));
    }

    let dot: f64 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();

    Ok(XdlValue::Double(dot))
}

/// NORM - Vector or matrix norm
/// NORM(array [, p]) computes the p-norm (default p=2)
pub fn norm(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "NORM: Expected array argument".to_string(),
        ));
    }

    let p = if args.len() > 1 {
        match &args[1] {
            XdlValue::Int(v) => *v as i32,
            XdlValue::Long(v) => *v,
            _ => 2,
        }
    } else {
        2
    };

    match &args[0] {
        XdlValue::Array(arr) => {
            let result = match p {
                1 => arr.iter().map(|x| x.abs()).sum::<f64>(),
                2 => arr.iter().map(|x| x * x).sum::<f64>().sqrt(),
                _ if p > 0 => arr
                    .iter()
                    .map(|x| x.abs().powi(p))
                    .sum::<f64>()
                    .powf(1.0 / p as f64),
                _ => {
                    return Err(XdlError::InvalidArgument(
                        "NORM: p must be positive".to_string(),
                    ))
                }
            };
            Ok(XdlValue::Double(result))
        }
        XdlValue::MultiDimArray { data, .. } => {
            // For matrices, compute Frobenius norm (p=2)
            let result = data.iter().map(|x| x * x).sum::<f64>().sqrt();
            Ok(XdlValue::Double(result))
        }
        _ => Err(XdlError::TypeMismatch {
            expected: "array".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// DIAGONAL - Extract diagonal from matrix
/// DIAGONAL(matrix [, offset]) extracts the diagonal
pub fn diagonal(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "DIAGONAL: Expected matrix argument".to_string(),
        ));
    }

    let offset = if args.len() > 1 {
        match &args[1] {
            XdlValue::Int(v) => *v as i32,
            XdlValue::Long(v) => *v,
            _ => 0,
        }
    } else {
        0
    };

    let (data, shape) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => (data, shape),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "matrix".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if shape.len() != 2 {
        return Err(XdlError::DimensionError(
            "DIAGONAL: Expected 2D matrix".to_string(),
        ));
    }

    let nrows = shape[0];
    let ncols = shape[1];

    let mut diag = Vec::new();

    if offset >= 0 {
        let offset = offset as usize;
        for i in 0..nrows.min(ncols.saturating_sub(offset)) {
            diag.push(data[i * ncols + i + offset]);
        }
    } else {
        let offset = (-offset) as usize;
        for i in 0..nrows.saturating_sub(offset).min(ncols) {
            diag.push(data[(i + offset) * ncols + i]);
        }
    }

    Ok(XdlValue::Array(diag))
}

/// TRACE - Matrix trace (sum of diagonal elements)
/// TRACE(matrix) returns the sum of diagonal elements
pub fn trace(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "TRACE: Expected matrix argument".to_string(),
        ));
    }

    let diag_result = diagonal(args)?;

    match diag_result {
        XdlValue::Array(arr) => {
            let sum: f64 = arr.iter().sum();
            Ok(XdlValue::Double(sum))
        }
        _ => Err(XdlError::RuntimeError("TRACE: Internal error".to_string())),
    }
}

/// SVDC - Singular Value Decomposition
/// SVDC(matrix, w, u, v) computes SVD: A = U * W * V^T
/// Returns singular values in w
pub fn svdc(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "SVDC: Expected matrix argument".to_string(),
        ));
    }

    let (data, shape) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => (data.clone(), shape.clone()),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "matrix".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if shape.len() != 2 {
        return Err(XdlError::DimensionError(
            "SVDC: Expected 2D matrix".to_string(),
        ));
    }

    let m = shape[0];
    let n = shape[1];
    let matrix = DMatrix::from_row_slice(m, n, &data);

    // Compute SVD
    let svd = matrix.svd(true, true);

    // Return singular values as array
    let singular_values: Vec<f64> = svd.singular_values.iter().copied().collect();

    Ok(XdlValue::Array(singular_values))
}

/// LA_EIGENVAL - Compute eigenvalues of a matrix
/// LA_EIGENVAL(matrix) returns eigenvalues
pub fn la_eigenval(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "LA_EIGENVAL: Expected matrix argument".to_string(),
        ));
    }

    let (data, shape) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => (data.clone(), shape.clone()),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "matrix".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if shape.len() != 2 || shape[0] != shape[1] {
        return Err(XdlError::DimensionError(
            "LA_EIGENVAL: Expected square matrix".to_string(),
        ));
    }

    let n = shape[0];
    let matrix = DMatrix::from_row_slice(n, n, &data);

    // Compute eigenvalues
    let eigen = matrix.symmetric_eigen();
    let eigenvalues: Vec<f64> = eigen.eigenvalues.iter().copied().collect();
    Ok(XdlValue::Array(eigenvalues))
}

/// LUDC - LU Decomposition
/// LUDC(matrix) computes LU decomposition
pub fn ludc(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "LUDC: Expected matrix argument".to_string(),
        ));
    }

    let (data, shape) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => (data.clone(), shape.clone()),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "matrix".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if shape.len() != 2 || shape[0] != shape[1] {
        return Err(XdlError::DimensionError(
            "LUDC: Expected square matrix".to_string(),
        ));
    }

    let n = shape[0];
    let matrix = DMatrix::from_row_slice(n, n, &data);

    // Compute LU decomposition
    let lu = matrix.lu();
    let l = lu.l();
    let result_data: Vec<f64> = l.iter().copied().collect();

    Ok(XdlValue::MultiDimArray {
        data: result_data,
        shape,
    })
}

/// LUSOL - Solve linear system using LU decomposition
/// LUSOL(lu_matrix, b) solves A*x = b
pub fn lusol(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "LUSOL: Expected LU matrix and vector b".to_string(),
        ));
    }

    let (data, shape) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => (data.clone(), shape.clone()),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "matrix".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let b = match &args[1] {
        XdlValue::Array(arr) => arr.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    if shape.len() != 2 || shape[0] != shape[1] {
        return Err(XdlError::DimensionError(
            "LUSOL: Expected square matrix".to_string(),
        ));
    }

    let n = shape[0];
    let matrix = DMatrix::from_row_slice(n, n, &data);
    let lu = matrix.lu();

    let b_vec = nalgebra::DVector::from_vec(b);
    match lu.solve(&b_vec) {
        Some(x) => {
            let result: Vec<f64> = x.iter().copied().collect();
            Ok(XdlValue::Array(result))
        }
        None => Err(XdlError::RuntimeError(
            "LUSOL: Could not solve system".to_string(),
        )),
    }
}

/// LA_EIGENVEC - Compute eigenvectors of a symmetric matrix
/// LA_EIGENVEC(matrix) returns eigenvectors as columns
pub fn la_eigenvec(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "LA_EIGENVEC: Expected matrix argument".to_string(),
        ));
    }

    let (data, shape) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => (data.clone(), shape.clone()),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "matrix".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if shape.len() != 2 || shape[0] != shape[1] {
        return Err(XdlError::DimensionError(
            "LA_EIGENVEC: Expected square matrix".to_string(),
        ));
    }

    let n = shape[0];
    let matrix = DMatrix::from_row_slice(n, n, &data);

    // Compute eigendecomposition
    let eigen = matrix.symmetric_eigen();
    let eigenvectors: Vec<f64> = eigen.eigenvectors.iter().copied().collect();

    Ok(XdlValue::MultiDimArray {
        data: eigenvectors,
        shape,
    })
}

/// LA_LINEAR_EQUATION - Solve a system of linear equations
/// LA_LINEAR_EQUATION(A, b) solves A*x = b
pub fn la_linear_equation(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "LA_LINEAR_EQUATION: Expected matrix A and vector b".to_string(),
        ));
    }

    let (data, shape) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => (data.clone(), shape.clone()),
        XdlValue::Array(arr) => {
            let n = (arr.len() as f64).sqrt() as usize;
            if n * n != arr.len() {
                return Err(XdlError::DimensionError(
                    "LA_LINEAR_EQUATION: Matrix dimensions invalid".to_string(),
                ));
            }
            (arr.clone(), vec![n, n])
        }
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "matrix".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let b = match &args[1] {
        XdlValue::Array(arr) => arr.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    if shape.len() != 2 || shape[0] != shape[1] {
        return Err(XdlError::DimensionError(
            "LA_LINEAR_EQUATION: Expected square matrix".to_string(),
        ));
    }

    if shape[0] != b.len() {
        return Err(XdlError::DimensionError(
            "LA_LINEAR_EQUATION: Matrix rows must match vector length".to_string(),
        ));
    }

    let n = shape[0];
    let matrix = DMatrix::from_row_slice(n, n, &data);
    let b_vec = nalgebra::DVector::from_vec(b);

    // Solve using LU decomposition
    let lu = matrix.lu();
    match lu.solve(&b_vec) {
        Some(x) => {
            let result: Vec<f64> = x.iter().copied().collect();
            Ok(XdlValue::Array(result))
        }
        None => Err(XdlError::RuntimeError(
            "LA_LINEAR_EQUATION: System is singular".to_string(),
        )),
    }
}

/// LA_LEAST_SQUARES - Solve overdetermined linear system
/// LA_LEAST_SQUARES(A, b) finds x that minimizes ||A*x - b||
pub fn la_least_squares(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "LA_LEAST_SQUARES: Expected matrix A and vector b".to_string(),
        ));
    }

    let (data, shape) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => (data.clone(), shape.clone()),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "matrix".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let b = match &args[1] {
        XdlValue::Array(arr) => arr.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    if shape.len() != 2 {
        return Err(XdlError::DimensionError(
            "LA_LEAST_SQUARES: Expected 2D matrix".to_string(),
        ));
    }

    let m = shape[0];
    let n = shape[1];
    let matrix = DMatrix::from_row_slice(m, n, &data);
    let b_vec = nalgebra::DVector::from_vec(b);

    // Solve using SVD for least squares
    let svd = matrix.svd(true, true);
    match svd.solve(&b_vec, 1e-10) {
        Ok(x) => {
            let result: Vec<f64> = x.iter().copied().collect();
            Ok(XdlValue::Array(result))
        }
        Err(_) => Err(XdlError::RuntimeError(
            "LA_LEAST_SQUARES: Could not solve system".to_string(),
        )),
    }
}

/// LA_CHOLESKY - Cholesky decomposition for positive-definite matrices
/// LA_CHOLESKY(matrix) returns lower triangular L such that A = L*L^T
pub fn la_cholesky(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "LA_CHOLESKY: Expected matrix argument".to_string(),
        ));
    }

    let (data, shape) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => (data.clone(), shape.clone()),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "matrix".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if shape.len() != 2 || shape[0] != shape[1] {
        return Err(XdlError::DimensionError(
            "LA_CHOLESKY: Expected square matrix".to_string(),
        ));
    }

    let n = shape[0];
    let matrix = DMatrix::from_row_slice(n, n, &data);

    // Compute Cholesky decomposition
    match matrix.cholesky() {
        Some(chol) => {
            let l = chol.l();
            let result_data: Vec<f64> = l.iter().copied().collect();
            Ok(XdlValue::MultiDimArray {
                data: result_data,
                shape,
            })
        }
        None => Err(XdlError::RuntimeError(
            "LA_CHOLESKY: Matrix is not positive definite".to_string(),
        )),
    }
}

/// LA_TRIDC - Tridiagonal decomposition
/// LA_TRIDC(matrix) returns tridiagonal form
pub fn la_tridc(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "LA_TRIDC: Expected matrix argument".to_string(),
        ));
    }

    let (data, shape) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => (data.clone(), shape.clone()),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "matrix".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if shape.len() != 2 || shape[0] != shape[1] {
        return Err(XdlError::DimensionError(
            "LA_TRIDC: Expected square matrix".to_string(),
        ));
    }

    let n = shape[0];

    // Extract tridiagonal elements
    // Main diagonal
    let mut diag = Vec::with_capacity(n);
    // Sub-diagonal
    let mut sub = Vec::with_capacity(n - 1);
    // Super-diagonal
    let mut sup = Vec::with_capacity(n - 1);

    for i in 0..n {
        diag.push(data[i * n + i]);
        if i < n - 1 {
            sub.push(data[(i + 1) * n + i]);
            sup.push(data[i * n + i + 1]);
        }
    }

    // Return as nested array: [diagonal, sub-diagonal, super-diagonal]
    Ok(XdlValue::NestedArray(vec![
        XdlValue::Array(diag),
        XdlValue::Array(sub),
        XdlValue::Array(sup),
    ]))
}

/// QR - QR decomposition
/// QR(matrix) returns [Q, R] where A = Q*R
pub fn qr(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "QR: Expected matrix argument".to_string(),
        ));
    }

    let (data, shape) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => (data.clone(), shape.clone()),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "matrix".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if shape.len() != 2 {
        return Err(XdlError::DimensionError(
            "QR: Expected 2D matrix".to_string(),
        ));
    }

    let m = shape[0];
    let n = shape[1];
    let matrix = DMatrix::from_row_slice(m, n, &data);

    // Compute QR decomposition
    let qr = matrix.qr();
    let q = qr.q();
    let r = qr.r();

    let q_data: Vec<f64> = q.iter().copied().collect();
    let r_data: Vec<f64> = r.iter().copied().collect();

    Ok(XdlValue::NestedArray(vec![
        XdlValue::MultiDimArray {
            data: q_data,
            shape: vec![m, m],
        },
        XdlValue::MultiDimArray {
            data: r_data,
            shape: vec![m, n],
        },
    ]))
}

/// RANK - Compute matrix rank
/// RANK(matrix [, tolerance])
pub fn matrix_rank(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "RANK: Expected matrix argument".to_string(),
        ));
    }

    let (data, shape) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => (data.clone(), shape.clone()),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "matrix".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let tolerance = if args.len() > 1 {
        match &args[1] {
            XdlValue::Double(v) => *v,
            XdlValue::Float(v) => *v as f64,
            _ => 1e-10,
        }
    } else {
        1e-10
    };

    if shape.len() != 2 {
        return Err(XdlError::DimensionError(
            "RANK: Expected 2D matrix".to_string(),
        ));
    }

    let m = shape[0];
    let n = shape[1];
    let matrix = DMatrix::from_row_slice(m, n, &data);

    // Compute rank via SVD
    let svd = matrix.svd(false, false);
    let rank = svd.singular_values.iter().filter(|&&s| s > tolerance).count();

    Ok(XdlValue::Long(rank as i32))
}

/// CRAMER - Solve linear system using Cramer's rule
/// CRAMER(A, b) solves A*x = b using determinants
pub fn cramer(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "CRAMER: Expected matrix A and vector b".to_string(),
        ));
    }

    let (data, shape) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => (data.clone(), shape.clone()),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "matrix".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let b = match &args[1] {
        XdlValue::Array(arr) => arr.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    if shape.len() != 2 || shape[0] != shape[1] {
        return Err(XdlError::DimensionError(
            "CRAMER: Expected square matrix".to_string(),
        ));
    }

    let n = shape[0];
    if n != b.len() {
        return Err(XdlError::DimensionError(
            "CRAMER: Matrix size must match vector length".to_string(),
        ));
    }

    let matrix = DMatrix::from_row_slice(n, n, &data);
    let det_a = matrix.determinant();

    if det_a.abs() < 1e-15 {
        return Err(XdlError::RuntimeError(
            "CRAMER: Matrix is singular".to_string(),
        ));
    }

    let mut result = Vec::with_capacity(n);

    for i in 0..n {
        // Replace i-th column with b
        let mut modified = data.clone();
        for j in 0..n {
            modified[j * n + i] = b[j];
        }

        let modified_matrix = DMatrix::from_row_slice(n, n, &modified);
        let det_i = modified_matrix.determinant();
        result.push(det_i / det_a);
    }

    Ok(XdlValue::Array(result))
}

/// MATRIX_MULTIPLY - Matrix multiplication
/// MATRIX_MULTIPLY(A, B) or A ## B
pub fn matrix_multiply(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "MATRIX_MULTIPLY: Expected two matrices".to_string(),
        ));
    }

    let (data_a, shape_a) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => (data.clone(), shape.clone()),
        XdlValue::Array(arr) => {
            // Treat as column vector
            (arr.clone(), vec![arr.len(), 1])
        }
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "matrix".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let (data_b, shape_b) = match &args[1] {
        XdlValue::MultiDimArray { data, shape } => (data.clone(), shape.clone()),
        XdlValue::Array(arr) => {
            // Treat as row vector
            (arr.clone(), vec![1, arr.len()])
        }
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "matrix".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    if shape_a.len() != 2 || shape_b.len() != 2 {
        return Err(XdlError::DimensionError(
            "MATRIX_MULTIPLY: Expected 2D matrices".to_string(),
        ));
    }

    if shape_a[1] != shape_b[0] {
        return Err(XdlError::DimensionError(format!(
            "MATRIX_MULTIPLY: Incompatible dimensions: {}x{} and {}x{}",
            shape_a[0], shape_a[1], shape_b[0], shape_b[1]
        )));
    }

    let m = shape_a[0];
    let k = shape_a[1];
    let n = shape_b[1];

    let a = DMatrix::from_row_slice(m, k, &data_a);
    let b = DMatrix::from_row_slice(k, n, &data_b);

    let c = a * b;
    let result_data: Vec<f64> = c.iter().copied().collect();

    Ok(XdlValue::MultiDimArray {
        data: result_data,
        shape: vec![m, n],
    })
}

/// COND - Matrix condition number
/// COND(matrix [, norm]) computes the condition number
pub fn cond(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "COND: Expected matrix argument".to_string(),
        ));
    }

    let (data, shape) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => (data.clone(), shape.clone()),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "matrix".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if shape.len() != 2 {
        return Err(XdlError::DimensionError(
            "COND: Expected 2D matrix".to_string(),
        ));
    }

    let m = shape[0];
    let n = shape[1];
    let matrix = DMatrix::from_row_slice(m, n, &data);

    // Compute condition number via SVD
    let svd = matrix.svd(false, false);
    let singular_values: Vec<f64> = svd.singular_values.iter().copied().collect();

    if singular_values.is_empty() {
        return Ok(XdlValue::Double(f64::INFINITY));
    }

    let max_sv = singular_values
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let min_sv = singular_values
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);

    if min_sv < 1e-15 {
        Ok(XdlValue::Double(f64::INFINITY))
    } else {
        Ok(XdlValue::Double(max_sv / min_sv))
    }
}

/// PINV - Moore-Penrose pseudoinverse
/// PINV(matrix [, tolerance])
pub fn pinv(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "PINV: Expected matrix argument".to_string(),
        ));
    }

    let (data, shape) = match &args[0] {
        XdlValue::MultiDimArray { data, shape } => (data.clone(), shape.clone()),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "matrix".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let tolerance = if args.len() > 1 {
        match &args[1] {
            XdlValue::Double(v) => *v,
            XdlValue::Float(v) => *v as f64,
            _ => 1e-10,
        }
    } else {
        1e-10
    };

    if shape.len() != 2 {
        return Err(XdlError::DimensionError(
            "PINV: Expected 2D matrix".to_string(),
        ));
    }

    let m = shape[0];
    let n = shape[1];
    let matrix = DMatrix::from_row_slice(m, n, &data);

    // Compute pseudoinverse via SVD
    let svd = matrix.svd(true, true);

    // Get U, S, V^T
    let u = svd.u.unwrap();
    let v_t = svd.v_t.unwrap();
    let singular_values = &svd.singular_values;

    // Compute S^+
    let mut s_plus = DMatrix::zeros(n, m);
    for i in 0..singular_values.len().min(n).min(m) {
        let s = singular_values[i];
        if s > tolerance {
            s_plus[(i, i)] = 1.0 / s;
        }
    }

    // Pseudoinverse = V * S^+ * U^T
    let pinv_matrix = v_t.transpose() * s_plus * u.transpose();
    let result_data: Vec<f64> = pinv_matrix.iter().copied().collect();

    Ok(XdlValue::MultiDimArray {
        data: result_data,
        shape: vec![n, m],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let args = vec![XdlValue::Long(3)];
        let result = identity(&args).unwrap();

        if let XdlValue::MultiDimArray { data, shape } = result {
            assert_eq!(shape, vec![3, 3]);
            assert_eq!(data, vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);
        } else {
            panic!("Expected MultiDimArray");
        }
    }

    #[test]
    fn test_crossp() {
        let v1 = XdlValue::Array(vec![1.0, 0.0, 0.0]);
        let v2 = XdlValue::Array(vec![0.0, 1.0, 0.0]);
        let result = crossp(&[v1, v2]).unwrap();

        if let XdlValue::Array(arr) = result {
            assert_eq!(arr, vec![0.0, 0.0, 1.0]);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_dotp() {
        let v1 = XdlValue::Array(vec![1.0, 2.0, 3.0]);
        let v2 = XdlValue::Array(vec![4.0, 5.0, 6.0]);
        let result = dotp(&[v1, v2]).unwrap();

        if let XdlValue::Double(val) = result {
            assert!((val - 32.0).abs() < 1e-10); // 1*4 + 2*5 + 3*6 = 32
        } else {
            panic!("Expected Double");
        }
    }

    #[test]
    fn test_determ() {
        // 2x2 identity should have determinant 1
        let matrix = XdlValue::MultiDimArray {
            data: vec![1.0, 0.0, 0.0, 1.0],
            shape: vec![2, 2],
        };
        let result = determ(&[matrix]).unwrap();

        if let XdlValue::Double(val) = result {
            assert!((val - 1.0).abs() < 1e-10);
        } else {
            panic!("Expected Double");
        }
    }
}
