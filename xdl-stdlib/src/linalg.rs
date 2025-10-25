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
    match matrix.symmetric_eigen() {
        eigen => {
            let eigenvalues: Vec<f64> = eigen.eigenvalues.iter().copied().collect();
            Ok(XdlValue::Array(eigenvalues))
        }
    }
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
