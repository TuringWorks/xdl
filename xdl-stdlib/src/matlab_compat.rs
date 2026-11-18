//! MATLAB Compatibility Functions
//!
//! This module provides MATLAB-style functions that are commonly used but not
//! available in standard IDL/GDL. These extend XDL's capabilities for users
//! coming from MATLAB.

use crate::XdlResult;
use std::{
    borrow::Cow,
    io::{BufRead, BufReader},
};
use xdl_core::{XdlError, XdlValue};

const MATLAB_MLDIVIDE_MAX_DIMENSION: usize = 4096;
const MATLAB_RESHAPE_MAX_ELEMENTS: usize = 16_777_216;
const MATLAB_READMATRIX_MAX_HEADER_LINES: usize = 1_000_000;
const MATLAB_COLON_MAX_ELEMENTS: usize = 1_000_000;

fn matlab_colon_scalar(value: &XdlValue, position: &str) -> XdlResult<f64> {
    let mut scalar = value;
    while let XdlValue::NestedArray(values) = scalar {
        if values.len() != 1 {
            break;
        }
        scalar = &values[0];
    }
    let number = match scalar {
        XdlValue::Byte(_)
        | XdlValue::Int(_)
        | XdlValue::Long(_)
        | XdlValue::Float(_)
        | XdlValue::Double(_)
        | XdlValue::UInt(_)
        | XdlValue::ULong(_)
        | XdlValue::Long64(_)
        | XdlValue::ULong64(_) => value.to_double()?,
        XdlValue::Array(values) if values.len() == 1 => values[0],
        XdlValue::MultiDimArray { data, shape }
            if data.len() == 1
                && shape
                    .iter()
                    .try_fold(1usize, |count, dimension| count.checked_mul(*dimension))
                    == Some(1) =>
        {
            data[0]
        }
        other => {
            return Err(XdlError::TypeMismatch {
                expected: format!("finite real numeric scalar for {position}"),
                actual: format!("{:?}", other.gdl_type()),
            });
        }
    };
    if !number.is_finite() {
        return Err(XdlError::InvalidArgument(format!(
            "MATLAB_COLON: {position} must be finite"
        )));
    }
    Ok(number)
}

/// Construct a MATLAB/Octave inclusive numeric colon vector from runtime scalars.
/// Inputs must be finite real numeric scalars; output is a 1D double array.
pub fn matlab_colon(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if !(2..=3).contains(&args.len()) {
        return Err(XdlError::InvalidArgument(format!(
            "MATLAB_COLON: Expected 2 or 3 scalar arguments, got {}",
            args.len()
        )));
    }

    let start = matlab_colon_scalar(&args[0], "start")?;
    let (step, stop) = if args.len() == 2 {
        let stop = matlab_colon_scalar(&args[1], "stop")?;
        (1.0, stop)
    } else {
        let step = matlab_colon_scalar(&args[1], "step")?;
        let stop = matlab_colon_scalar(&args[2], "stop")?;
        if step == 0.0 {
            return Err(XdlError::InvalidArgument(
                "MATLAB_COLON: step must be nonzero".into(),
            ));
        }
        (step, stop)
    };

    let distance = stop - start;
    let count = if distance == 0.0 {
        1.0
    } else if distance.signum() != step.signum() {
        0.0
    } else {
        let quotient = distance / step;
        if !quotient.is_finite() {
            return Err(XdlError::InvalidArgument(
                "MATLAB_COLON: range element count is not finite".into(),
            ));
        }
        let tolerance = 8.0 * f64::EPSILON * quotient.abs().max(1.0);
        (quotient + tolerance).floor() + 1.0
    };
    if !count.is_finite() || count > MATLAB_COLON_MAX_ELEMENTS as f64 {
        return Err(XdlError::InvalidArgument(format!(
            "MATLAB_COLON: range exceeds {MATLAB_COLON_MAX_ELEMENTS} elements"
        )));
    }

    let data: Vec<f64> = (0..count as usize)
        .map(|index| start + index as f64 * step)
        .collect();
    if data.iter().any(|value| !value.is_finite()) {
        return Err(XdlError::InvalidArgument(
            "MATLAB_COLON: generated range contains a non-finite value".into(),
        ));
    }
    Ok(XdlValue::Array(data))
}

/// Return IEEE-754 NaN for a MATLAB/Octave `NaN` literal lowered by the transpiler.
pub fn matlab_nan(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if !args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "MATLAB_NAN takes no arguments".into(),
        ));
    }
    Ok(XdlValue::Double(f64::NAN))
}

/// Return positive IEEE-754 infinity for a MATLAB/Octave `Inf` literal.
pub fn matlab_inf(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if !args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "MATLAB_INF takes no arguments".into(),
        ));
    }
    Ok(XdlValue::Double(f64::INFINITY))
}

/// Reshape numeric input using MATLAB's column-major linear element order.
/// The returned `MultiDimArray` stores the same logical values in XDL's
/// row-major layout.
pub fn matlab_reshape(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(format!(
            "MATLAB_RESHAPE: Expected an array and at least two dimensions, got {} arguments",
            args.len()
        )));
    }

    let mut target_shape = Vec::with_capacity(args.len() - 1);
    let mut target_len = 1usize;
    for (index, dimension) in args[1..].iter().enumerate() {
        let number = match dimension {
            XdlValue::Byte(_)
            | XdlValue::Int(_)
            | XdlValue::Long(_)
            | XdlValue::Float(_)
            | XdlValue::Double(_)
            | XdlValue::UInt(_)
            | XdlValue::ULong(_)
            | XdlValue::Long64(_)
            | XdlValue::ULong64(_) => dimension.to_double()?,
            other => {
                return Err(XdlError::TypeMismatch {
                    expected: format!("nonnegative integer dimension at position {}", index + 1),
                    actual: format!("{:?}", other.gdl_type()),
                })
            }
        };
        if !number.is_finite()
            || number < 0.0
            || number.fract() != 0.0
            || number >= usize::MAX as f64
            || number > MATLAB_RESHAPE_MAX_ELEMENTS as f64
        {
            return Err(XdlError::InvalidArgument(format!(
                "MATLAB_RESHAPE: dimension {} must be a nonnegative integer no larger than {MATLAB_RESHAPE_MAX_ELEMENTS}",
                index + 1
            )));
        }
        let dimension = number as usize;
        target_len = target_len.checked_mul(dimension).ok_or_else(|| {
            XdlError::InvalidArgument("MATLAB_RESHAPE: target shape product overflows".into())
        })?;
        if target_len > MATLAB_RESHAPE_MAX_ELEMENTS {
            return Err(XdlError::InvalidArgument(format!(
                "MATLAB_RESHAPE: target element count exceeds {MATLAB_RESHAPE_MAX_ELEMENTS}"
            )));
        }
        target_shape.push(dimension);
    }

    let source_shape = matlab_dimensions(&args[0]).map_err(|error| {
        XdlError::InvalidArgument(format!("MATLAB_RESHAPE: invalid input shape: {error}"))
    })?;
    let source_len = source_shape
        .iter()
        .try_fold(1usize, |count, dimension| count.checked_mul(*dimension))
        .ok_or_else(|| {
            XdlError::InvalidArgument("MATLAB_RESHAPE: input shape product overflows".into())
        })?;
    if target_len != source_len {
        return Err(XdlError::DimensionError(format!(
            "MATLAB_RESHAPE: target shape contains {target_len} elements but input shape contains {source_len}"
        )));
    }
    let row_major = matlab_reshape_flatten_row_major(&args[0])?;
    if source_len != row_major.len() {
        return Err(XdlError::InvalidArgument(
            "MATLAB_RESHAPE: input shape does not match its data length".into(),
        ));
    }

    let matlab_order = matlab_reshape_to_column_major(&row_major, &source_shape)?;
    let data = matlab_reshape_column_major_to_row_major(&matlab_order, &target_shape)?;
    Ok(XdlValue::MultiDimArray {
        data,
        shape: target_shape,
    })
}

/// MATLAB apostrophe transpose for real arrays, with scalar complex conjugation.
/// Complex array transposition remains an explicit error until the runtime has
/// a complex-array representation with stable shape semantics.
pub fn matlab_ctranspose(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "MATLAB_CTRANSPOSE: Expected 1 argument, got {}",
            args.len()
        )));
    }
    match &args[0] {
        XdlValue::Complex(value) => return Ok(XdlValue::Complex(value.conj())),
        XdlValue::DComplex(value) => return Ok(XdlValue::DComplex(value.conj())),
        XdlValue::MultiDimArray { data, shape } => {
            if shape.len() != 2 {
                return Err(XdlError::DimensionError(
                    "MATLAB_CTRANSPOSE: only two-dimensional arrays are supported".into(),
                ));
            }
            let (rows, columns) = (shape[0], shape[1]);
            if rows.checked_mul(columns) != Some(data.len()) {
                return Err(XdlError::DimensionError(
                    "MATLAB_CTRANSPOSE: array shape does not match its data".into(),
                ));
            }
            let transposed = (0..columns)
                .flat_map(|row| (0..rows).map(move |column| data[column * columns + row]))
                .collect();
            return Ok(XdlValue::MultiDimArray {
                data: transposed,
                shape: vec![columns, rows],
            });
        }
        value if matlab_contains_complex(value) => {
            return Err(XdlError::NotImplemented(
                "MATLAB_CTRANSPOSE: complex array transpose is not supported".into(),
            ));
        }
        XdlValue::Byte(_)
        | XdlValue::Int(_)
        | XdlValue::Long(_)
        | XdlValue::Float(_)
        | XdlValue::Double(_)
        | XdlValue::UInt(_)
        | XdlValue::ULong(_)
        | XdlValue::Long64(_)
        | XdlValue::ULong64(_) => return Ok(args[0].clone()),
        _ => (),
    }
    crate::array::transpose_func(args)
}

fn matlab_contains_complex(value: &XdlValue) -> bool {
    match value {
        XdlValue::Complex(_) | XdlValue::DComplex(_) => true,
        XdlValue::NestedArray(values) => values.iter().any(matlab_contains_complex),
        _ => false,
    }
}

fn matlab_reshape_flatten_row_major(value: &XdlValue) -> XdlResult<Vec<f64>> {
    match value {
        XdlValue::Array(values) | XdlValue::MultiDimArray { data: values, .. } => {
            Ok(values.clone())
        }
        XdlValue::NestedArray(values) => values
            .iter()
            .map(matlab_reshape_flatten_row_major)
            .collect::<XdlResult<Vec<_>>>()
            .map(|chunks| chunks.into_iter().flatten().collect()),
        XdlValue::Byte(_)
        | XdlValue::Int(_)
        | XdlValue::Long(_)
        | XdlValue::Float(_)
        | XdlValue::Double(_)
        | XdlValue::UInt(_)
        | XdlValue::ULong(_)
        | XdlValue::Long64(_)
        | XdlValue::ULong64(_) => Ok(vec![value.to_double()?]),
        other => Err(XdlError::TypeMismatch {
            expected: "real numeric scalar or rectangular array".into(),
            actual: format!("{:?}", other.gdl_type()),
        }),
    }
}

fn matlab_reshape_to_column_major(row_major: &[f64], shape: &[usize]) -> XdlResult<Vec<f64>> {
    let mut column_major = vec![0.0; row_major.len()];
    for (row_index, value) in row_major.iter().enumerate() {
        let column_index = matlab_reshape_column_index(row_index, shape)?;
        let slot = column_major.get_mut(column_index).ok_or_else(|| {
            XdlError::InvalidArgument("MATLAB_RESHAPE: malformed input shape index".into())
        })?;
        *slot = *value;
    }
    Ok(column_major)
}

fn matlab_reshape_column_major_to_row_major(
    column_major: &[f64],
    shape: &[usize],
) -> XdlResult<Vec<f64>> {
    let mut row_major = vec![0.0; column_major.len()];
    for (row_index, slot) in row_major.iter_mut().enumerate() {
        let column_index = matlab_reshape_column_index(row_index, shape)?;
        *slot = *column_major.get(column_index).ok_or_else(|| {
            XdlError::InvalidArgument("MATLAB_RESHAPE: malformed target shape index".into())
        })?;
    }
    Ok(row_major)
}

fn matlab_reshape_column_index(row_index: usize, shape: &[usize]) -> XdlResult<usize> {
    let mut remaining = row_index;
    let mut coordinates = vec![0usize; shape.len()];
    for (coordinate, dimension) in coordinates.iter_mut().zip(shape).rev() {
        if *dimension == 0 {
            return Err(XdlError::InvalidArgument(
                "MATLAB_RESHAPE: cannot index a zero-sized dimension".into(),
            ));
        }
        *coordinate = remaining % dimension;
        remaining /= dimension;
    }
    coordinates
        .iter()
        .zip(shape)
        .try_fold(
            (0usize, 1usize),
            |(index, stride), (coordinate, dimension)| {
                let term = coordinate.checked_mul(stride).ok_or_else(|| {
                    XdlError::InvalidArgument("MATLAB_RESHAPE: linear index overflows".into())
                })?;
                let next_index = index.checked_add(term).ok_or_else(|| {
                    XdlError::InvalidArgument("MATLAB_RESHAPE: linear index overflows".into())
                })?;
                let next_stride = stride.checked_mul(*dimension).ok_or_else(|| {
                    XdlError::InvalidArgument("MATLAB_RESHAPE: linear stride overflows".into())
                })?;
                Ok((next_index, next_stride))
            },
        )
        .map(|(index, _)| index)
}

/// Solve `A * x = b` for finite real-valued systems using LU (square) or SVD
/// least-squares/minimum-norm semantics (rectangular).
///
/// Matrices use XDL's existing row-major `MultiDimArray` convention; nested
/// row arrays are accepted as the equivalent representation. A vector RHS
/// returns an `Array`, while a matrix RHS retains its two-dimensional shape.
pub fn matlab_mldivide(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 2 {
        return Err(XdlError::InvalidArgument(format!(
            "MATLAB_MLDIVIDE: Expected matrix A and RHS b, got {} arguments",
            args.len()
        )));
    }
    let (a_data, a_rows, a_cols, _) = matlab_mldivide_matrix(&args[0], "A", true)?;
    if a_rows == 0 || a_cols == 0 {
        return Err(XdlError::InvalidArgument(
            "MATLAB_MLDIVIDE: A must have nonzero dimensions".to_string(),
        ));
    }
    if a_rows > MATLAB_MLDIVIDE_MAX_DIMENSION || a_cols > MATLAB_MLDIVIDE_MAX_DIMENSION {
        return Err(XdlError::InvalidArgument(format!(
            "MATLAB_MLDIVIDE: A dimensions may not exceed {MATLAB_MLDIVIDE_MAX_DIMENSION}"
        )));
    }
    let (b_data, b_rows, b_cols, vector_rhs) = matlab_mldivide_matrix(&args[1], "b", false)?;
    if b_rows != a_rows {
        return Err(XdlError::DimensionError(format!(
            "MATLAB_MLDIVIDE: RHS row count {b_rows} does not match A dimension {a_rows}"
        )));
    }
    if b_cols == 0 || b_cols > MATLAB_MLDIVIDE_MAX_DIMENSION {
        return Err(XdlError::InvalidArgument(format!(
            "MATLAB_MLDIVIDE: RHS column count must be between 1 and {MATLAB_MLDIVIDE_MAX_DIMENSION}"
        )));
    }

    let a = nalgebra::DMatrix::from_row_slice(a_rows, a_cols, &a_data);
    let b = nalgebra::DMatrix::from_row_slice(b_rows, b_cols, &b_data);
    let solution = if a_rows == a_cols {
        a.lu()
            .solve(&b)
            .ok_or_else(|| XdlError::InvalidArgument("MATLAB_MLDIVIDE: A is singular".to_string()))
    } else {
        let svd = a.svd(true, true);
        let largest_singular_value = svd.singular_values.iter().copied().fold(0.0_f64, f64::max);
        // A scale-relative cutoff preserves the solution when the same
        // rectangular system is expressed in different physical units.
        let tolerance = largest_singular_value * f64::EPSILON * a_rows.max(a_cols) as f64;
        svd.solve(&b, tolerance)
            .map_err(|_| XdlError::RuntimeError("MATLAB_MLDIVIDE: SVD solve failed".to_string()))
    }?;
    // nalgebra iterates in column-major order; XDL matrix data is row-major.
    let solution_rows = a_cols;
    let row_major: Vec<f64> = (0..solution_rows * b_cols)
        .map(|index| solution[(index / b_cols, index % b_cols)])
        .collect();
    if row_major.iter().any(|value| !value.is_finite()) {
        return Err(XdlError::RuntimeError(
            "MATLAB_MLDIVIDE: solution contains non-finite values".to_string(),
        ));
    }
    if vector_rhs {
        Ok(XdlValue::Array(row_major))
    } else {
        Ok(XdlValue::MultiDimArray {
            data: row_major,
            shape: vec![solution_rows, b_cols],
        })
    }
}

/// Extract a strict 2D row-major numeric matrix (or a vector RHS).
fn matlab_mldivide_matrix(
    value: &XdlValue,
    label: &str,
    coefficient_matrix: bool,
) -> XdlResult<(Vec<f64>, usize, usize, bool)> {
    let result = match value {
        XdlValue::Array(data) => {
            matlab_mldivide_check_bounds(data.len(), 1, label)?;
            if data.iter().any(|value| !value.is_finite()) {
                return Err(XdlError::InvalidArgument(format!(
                    "MATLAB_MLDIVIDE: {label} must contain only finite values"
                )));
            }
            if coefficient_matrix {
                (data.clone(), 1, data.len(), false)
            } else {
                (data.clone(), data.len(), 1, true)
            }
        }
        XdlValue::MultiDimArray { data, shape } if shape.len() == 2 => {
            let expected = shape[0].checked_mul(shape[1]);
            if expected != Some(data.len()) {
                return Err(XdlError::InvalidArgument(format!(
                    "MATLAB_MLDIVIDE: {label} shape does not match its data length"
                )));
            }
            matlab_mldivide_check_bounds(shape[0], shape[1], label)?;
            if data.iter().any(|value| !value.is_finite()) {
                return Err(XdlError::InvalidArgument(format!(
                    "MATLAB_MLDIVIDE: {label} must contain only finite values"
                )));
            }
            (data.clone(), shape[0], shape[1], false)
        }
        XdlValue::NestedArray(rows) if rows.iter().all(|row| matches!(row, XdlValue::Array(_))) => {
            let row_data: Vec<&[f64]> = rows
                .iter()
                .filter_map(|row| match row {
                    XdlValue::Array(values) => Some(values.as_slice()),
                    _ => None,
                })
                .collect();
            let columns = row_data.first().map_or(0, |row| row.len());
            if row_data.iter().any(|row| row.len() != columns) {
                return Err(XdlError::InvalidArgument(format!(
                    "MATLAB_MLDIVIDE: {label} rows must have equal lengths"
                )));
            }
            matlab_mldivide_check_bounds(rows.len(), columns, label)?;
            if row_data
                .iter()
                .flat_map(|row| row.iter())
                .any(|value| !value.is_finite())
            {
                return Err(XdlError::InvalidArgument(format!(
                    "MATLAB_MLDIVIDE: {label} must contain only finite values"
                )));
            }
            (
                row_data
                    .into_iter()
                    .flat_map(|row| row.iter().copied())
                    .collect(),
                rows.len(),
                columns,
                false,
            )
        }
        XdlValue::MultiDimArray { shape, .. } => {
            return Err(XdlError::DimensionError(format!(
                "MATLAB_MLDIVIDE: {label} must be a vector or 2D matrix, got rank {}",
                shape.len()
            )))
        }
        other => {
            return Err(XdlError::TypeMismatch {
                expected: format!("numeric vector or 2D matrix for {label}"),
                actual: format!("{:?}", other.gdl_type()),
            })
        }
    };
    if result.0.iter().any(|value| !value.is_finite()) {
        return Err(XdlError::InvalidArgument(format!(
            "MATLAB_MLDIVIDE: {label} must contain only finite values"
        )));
    }
    Ok(result)
}

fn matlab_mldivide_check_bounds(rows: usize, columns: usize, label: &str) -> XdlResult<()> {
    if rows > MATLAB_MLDIVIDE_MAX_DIMENSION || columns > MATLAB_MLDIVIDE_MAX_DIMENSION {
        return Err(XdlError::InvalidArgument(format!(
            "MATLAB_MLDIVIDE: {label} dimensions may not exceed {MATLAB_MLDIVIDE_MAX_DIMENSION}"
        )));
    }
    Ok(())
}

/// MATLAB-compatible numeric CSV import for rectangular files, optionally
/// skipping an explicitly requested number of header lines.
pub fn matlab_readmatrix(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() || args.len() != 1 && args.len() != 3 {
        return Err(XdlError::InvalidArgument(format!(
            "MATLAB_READMATRIX: Expected a filename and optional 'NumHeaderLines' pair, got {} arguments",
            args.len()
        )));
    }
    let header_lines = if args.len() == 3 {
        match &args[1] {
            XdlValue::String(name) if name.eq_ignore_ascii_case("NumHeaderLines") => {}
            XdlValue::String(name) => {
                return Err(XdlError::InvalidArgument(format!(
                    "MATLAB_READMATRIX: Unsupported import option '{name}'"
                )))
            }
            other => {
                return Err(XdlError::TypeMismatch {
                    expected: "'NumHeaderLines' option name".to_string(),
                    actual: format!("{:?}", other.gdl_type()),
                })
            }
        }
        let count = match &args[2] {
            XdlValue::Byte(_)
            | XdlValue::Int(_)
            | XdlValue::Long(_)
            | XdlValue::Float(_)
            | XdlValue::Double(_)
            | XdlValue::UInt(_)
            | XdlValue::ULong(_)
            | XdlValue::Long64(_)
            | XdlValue::ULong64(_) => args[2].to_double()?,
            other => {
                return Err(XdlError::TypeMismatch {
                    expected: "positive integer NumHeaderLines value".to_string(),
                    actual: format!("{:?}", other.gdl_type()),
                })
            }
        };
        if !count.is_finite()
            || count < 1.0
            || count.fract() != 0.0
            || count > MATLAB_READMATRIX_MAX_HEADER_LINES as f64
        {
            return Err(XdlError::InvalidArgument(format!(
                "MATLAB_READMATRIX: NumHeaderLines must be a positive integer no larger than {MATLAB_READMATRIX_MAX_HEADER_LINES}"
            )));
        }
        count as usize
    } else {
        0
    };
    let filename = match &args[0] {
        XdlValue::String(path) => path,
        other => {
            return Err(XdlError::TypeMismatch {
                expected: "CSV filename string".to_string(),
                actual: format!("{:?}", other.gdl_type()),
            })
        }
    };
    let file = std::fs::File::open(filename).map_err(|error| {
        XdlError::RuntimeError(format!(
            "MATLAB_READMATRIX: Could not read '{filename}': {error}"
        ))
    })?;
    let mut input = BufReader::new(file);
    let skipped_lines = skip_physical_lines(&mut input, header_lines).map_err(|error| {
        XdlError::RuntimeError(format!(
            "MATLAB_READMATRIX: Could not read '{filename}': {error}"
        ))
    })?;
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(false)
        .from_reader(input);
    let rows = reader
        .records()
        .enumerate()
        .map(|(row_index, record)| {
            let record = record.map_err(|error| {
                XdlError::RuntimeError(format!(
                    "MATLAB_READMATRIX: Invalid CSV row {} in '{filename}': {error}",
                    skipped_lines + row_index + 1
                ))
            })?;
            let source_row = record
                .position()
                .map(|position| skipped_lines + position.line() as usize)
                .unwrap_or(skipped_lines + row_index + 1);
            if record.is_empty() {
                return Err(XdlError::InvalidArgument(format!(
                    "MATLAB_READMATRIX: CSV row {} in '{filename}' has no numeric columns",
                    source_row
                )));
            }
            record
                .iter()
                .enumerate()
                .map(|(column_index, field)| {
                    field.trim().parse::<f64>().map_err(|error| {
                        XdlError::InvalidArgument(format!(
                            "MATLAB_READMATRIX: Invalid number at row {}, column {} in '{filename}': {error}",
                            source_row,
                            column_index + 1
                        ))
                    })
                })
                .collect::<XdlResult<Vec<_>>>()
        })
        .collect::<XdlResult<Vec<_>>>()?;
    if rows.is_empty() || rows.first().is_some_and(Vec::is_empty) {
        return Err(XdlError::InvalidArgument(format!(
            "MATLAB_READMATRIX: '{filename}' contains no numeric data"
        )));
    }
    Ok(XdlValue::NestedArray(
        rows.into_iter().map(XdlValue::Array).collect(),
    ))
}

/// Consume up to `line_count` physical lines without allocating a buffer the
/// size of an untrusted header line. A final unterminated line counts as one.
fn skip_physical_lines<R: BufRead>(reader: &mut R, line_count: usize) -> std::io::Result<usize> {
    let mut skipped = 0;
    while skipped < line_count {
        let mut consumed_line_bytes = false;
        loop {
            let (consume_len, has_newline, at_eof) = {
                let available = reader.fill_buf()?;
                if available.is_empty() {
                    (0, false, true)
                } else if let Some(newline) = available.iter().position(|byte| *byte == b'\n') {
                    (newline + 1, true, false)
                } else {
                    (available.len(), false, false)
                }
            };
            if at_eof {
                if consumed_line_bytes {
                    skipped += 1;
                }
                return Ok(skipped);
            }
            reader.consume(consume_len);
            consumed_line_bytes = true;
            if has_newline {
                skipped += 1;
                break;
            }
        }
    }
    Ok(skipped)
}

/// MATLAB `length`: return the largest dimension of an array.
///
/// This deliberately remains separate from XDL's `N_ELEMENTS`, which returns
/// the total element count. Nested arrays must be rectangular so a ragged
/// value cannot produce a plausible but misleading dimension.
pub fn matlab_length(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "MATLAB_LENGTH: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let dimensions = matlab_dimensions(&args[0])?;
    let length = dimensions.into_iter().max().unwrap_or(0);
    Ok(XdlValue::Double(length as f64))
}

/// MATLAB `numel`: return the total scalar element count of a rectangular value.
pub fn matlab_numel(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "MATLAB_NUMEL: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let elements = matlab_dimensions(&args[0])?
        .into_iter()
        .try_fold(1usize, |count, dimension| count.checked_mul(dimension));
    let elements = elements.ok_or_else(|| {
        XdlError::InvalidArgument("MATLAB_NUMEL: element count overflow".to_string())
    })?;
    Ok(XdlValue::Double(elements as f64))
}

/// MATLAB `size`: return dimension lengths, or one requested dimension.
pub fn matlab_size(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if !(1..=2).contains(&args.len()) {
        return Err(XdlError::InvalidArgument(format!(
            "MATLAB_SIZE: Expected 1-2 arguments, got {}",
            args.len()
        )));
    }

    let mut dimensions = matlab_dimensions(&args[0])?;
    if dimensions.is_empty() {
        dimensions = vec![1, 1];
    } else if dimensions.len() == 1 {
        dimensions.push(1);
    }
    while dimensions.len() > 2 && dimensions.last() == Some(&1) {
        dimensions.pop();
    }

    if let Some(dimension) = args.get(1) {
        let dimension_value = dimension.to_double()?;
        if !dimension_value.is_finite() || dimension_value < 1.0 || dimension_value.fract() != 0.0 {
            return Err(XdlError::InvalidArgument(
                "MATLAB_SIZE: dimension must be a positive integer".to_string(),
            ));
        }
        let size = if dimension_value > dimensions.len() as f64 {
            1.0
        } else {
            let dimension_index = dimension_value as usize - 1;
            dimensions[dimension_index] as f64
        };
        return Ok(XdlValue::Double(size));
    }

    Ok(XdlValue::Array(
        dimensions
            .into_iter()
            .map(|dimension| dimension as f64)
            .collect(),
    ))
}

/// MATLAB multiple-output `size` tail: product of dimensions from `dim` onward.
/// This is the value MATLAB assigns to the final output when fewer outputs are
/// requested than the array has dimensions.
pub fn matlab_size_tail(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 2 {
        return Err(XdlError::InvalidArgument(format!(
            "MATLAB_SIZE_TAIL: Expected 2 arguments, got {}",
            args.len()
        )));
    }

    let dimension_value = args[1].to_double()?;
    if !dimension_value.is_finite() || dimension_value < 1.0 || dimension_value.fract() != 0.0 {
        return Err(XdlError::InvalidArgument(
            "MATLAB_SIZE_TAIL: dimension must be a positive integer".to_string(),
        ));
    }

    let mut dimensions = matlab_dimensions(&args[0])?;
    if dimensions.is_empty() {
        dimensions = vec![1, 1];
    } else if dimensions.len() == 1 {
        dimensions.push(1);
    }

    // A dimension beyond rank contributes the implicit MATLAB singleton 1.
    if dimension_value > dimensions.len() as f64 {
        return Ok(XdlValue::Double(1.0));
    }
    let start = dimension_value as usize - 1;
    let product = dimensions[start..]
        .iter()
        .try_fold(1usize, |product, size| product.checked_mul(*size))
        .ok_or_else(|| {
            XdlError::InvalidArgument("MATLAB_SIZE_TAIL: dimension product overflow".to_string())
        })?;
    Ok(XdlValue::Double(product as f64))
}

/// MATLAB-style one-based linear indexing into a numeric value.
pub fn matlab_index(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 2 {
        return Err(XdlError::InvalidArgument(format!(
            "MATLAB_INDEX: Expected 2 arguments, got {}",
            args.len()
        )));
    }

    let index_value = match &args[1] {
        XdlValue::Byte(_)
        | XdlValue::Int(_)
        | XdlValue::Long(_)
        | XdlValue::Float(_)
        | XdlValue::Double(_)
        | XdlValue::UInt(_)
        | XdlValue::ULong(_)
        | XdlValue::Long64(_)
        | XdlValue::ULong64(_) => args[1].to_double()?,
        other => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric scalar index".to_string(),
                actual: format!("{:?}", other.gdl_type()),
            })
        }
    };
    if !index_value.is_finite()
        || index_value < 1.0
        || index_value.fract() != 0.0
        || index_value >= usize::MAX as f64
    {
        return Err(XdlError::InvalidArgument(
            "MATLAB_INDEX: index must be a positive integer".to_string(),
        ));
    }
    let index = index_value as usize;
    let dimensions = matlab_dimensions(&args[0])?;
    let elements = dimensions
        .iter()
        .try_fold(1usize, |count, dimension| count.checked_mul(*dimension))
        .ok_or_else(|| {
            XdlError::InvalidArgument("MATLAB_INDEX: array element count overflow".to_string())
        })?;
    if index > elements {
        return Err(XdlError::InvalidArgument(format!(
            "MATLAB_INDEX: index {index} exceeds the number of elements ({elements})"
        )));
    }

    matlab_numeric_element(&args[0], index)
}

fn matlab_numeric_element(value: &XdlValue, one_based_index: usize) -> XdlResult<XdlValue> {
    match value {
        XdlValue::Array(values) => values
            .get(one_based_index - 1)
            .copied()
            .map(XdlValue::Double)
            .ok_or_else(|| matlab_index_out_of_range(one_based_index)),
        XdlValue::MultiDimArray { data, .. } => data
            .get(one_based_index - 1)
            .copied()
            .map(XdlValue::Double)
            .ok_or_else(|| matlab_index_out_of_range(one_based_index)),
        XdlValue::NestedArray(values)
            if values.iter().all(|item| matches!(item, XdlValue::Array(_))) =>
        {
            let row_count = values.len();
            let row_index = (one_based_index - 1) % row_count;
            let column_index = (one_based_index - 1) / row_count;
            match values.get(row_index) {
                Some(XdlValue::Array(row)) => row
                    .get(column_index)
                    .copied()
                    .map(XdlValue::Double)
                    .ok_or_else(|| matlab_index_out_of_range(one_based_index)),
                _ => Err(XdlError::InvalidArgument(
                    "MATLAB_INDEX: expected rectangular matrix rows".to_string(),
                )),
            }
        }
        XdlValue::NestedArray(values) => {
            let outer_index = (one_based_index - 1) % values.len();
            let child_index = (one_based_index - 1) / values.len() + 1;
            let child = values
                .get(outer_index)
                .ok_or_else(|| matlab_index_out_of_range(one_based_index))?;
            matlab_numeric_element(child, child_index)
        }
        XdlValue::Byte(_)
        | XdlValue::Int(_)
        | XdlValue::Long(_)
        | XdlValue::Float(_)
        | XdlValue::Double(_)
        | XdlValue::UInt(_)
        | XdlValue::ULong(_)
        | XdlValue::Long64(_)
        | XdlValue::ULong64(_) => Ok(XdlValue::Double(value.to_double()?)),
        other => Err(XdlError::TypeMismatch {
            expected: "numeric array".to_string(),
            actual: format!("{:?}", other.gdl_type()),
        }),
    }
}

fn matlab_index_out_of_range(index: usize) -> XdlError {
    XdlError::InvalidArgument(format!("MATLAB_INDEX: index {index} is out of range"))
}

/// Read one-based linear indices from a numeric vector or array. Selection
/// follows MATLAB column-major order; flat XDL arrays cannot preserve the
/// result orientation of a MATLAB vector-index expression.
pub fn matlab_index_vector(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 2 {
        return Err(XdlError::InvalidArgument(format!(
            "MATLAB_INDEX_VECTOR: Expected 2 arguments, got {}",
            args.len()
        )));
    }

    let source = &args[0];
    let source_elements = matlab_dimensions(source)?
        .iter()
        .try_fold(1usize, |count, dimension| count.checked_mul(*dimension))
        .ok_or_else(|| {
            XdlError::InvalidArgument(
                "MATLAB_INDEX_VECTOR: source element count overflow".to_string(),
            )
        })?;
    let indices = match &args[1] {
        XdlValue::Array(indices) => indices.as_slice(),
        XdlValue::MultiDimArray { data, shape } if shape.len() == 1 && shape[0] == data.len() => {
            data.as_slice()
        }
        index => {
            let index = numeric_index(index, "MATLAB_INDEX_VECTOR")?;
            let zero_based = validate_linear_index(index, source_elements, "MATLAB_INDEX_VECTOR")?;
            return matlab_numeric_element(source, zero_based + 1);
        }
    };

    let selected = indices
        .iter()
        .map(|index| {
            let zero_based = validate_linear_index(*index, source_elements, "MATLAB_INDEX_VECTOR")?;
            match matlab_numeric_element(source, zero_based + 1)? {
                XdlValue::Double(value) => Ok(value),
                other => Err(XdlError::TypeMismatch {
                    expected: "numeric array element".to_string(),
                    actual: format!("{:?}", other.gdl_type()),
                }),
            }
        })
        .collect::<XdlResult<Vec<_>>>()?;
    Ok(XdlValue::Array(selected))
}

fn numeric_index(value: &XdlValue, helper: &str) -> XdlResult<f64> {
    match value {
        XdlValue::Byte(_)
        | XdlValue::Int(_)
        | XdlValue::Long(_)
        | XdlValue::Float(_)
        | XdlValue::Double(_)
        | XdlValue::UInt(_)
        | XdlValue::ULong(_)
        | XdlValue::Long64(_)
        | XdlValue::ULong64(_) => value.to_double(),
        other => Err(XdlError::TypeMismatch {
            expected: "numeric scalar or one-dimensional numeric index vector".to_string(),
            actual: format!("{helper}: {:?}", other.gdl_type()),
        }),
    }
}

fn validate_linear_index(index: f64, length: usize, helper: &str) -> XdlResult<usize> {
    if !index.is_finite() || index < 1.0 || index.fract() != 0.0 || index >= usize::MAX as f64 {
        return Err(XdlError::InvalidArgument(format!(
            "{helper}: every index must be a finite positive integer"
        )));
    }
    let zero_based = index as usize - 1;
    if zero_based >= length {
        return Err(XdlError::InvalidArgument(format!(
            "{helper}: index {} exceeds vector length ({length})",
            zero_based + 1
        )));
    }
    Ok(zero_based)
}

/// MATLAB `find` for the one-output bounded and directional forms.
/// Results are one-based; row-vector inputs preserve row orientation while
/// other inputs return a column vector.
pub fn matlab_find(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if !(1..=3).contains(&args.len()) {
        return Err(XdlError::InvalidArgument(format!(
            "MATLAB_FIND: Expected 1-3 arguments, got {}",
            args.len()
        )));
    }

    let limit = args
        .get(1)
        .map(|value| matlab_find_limit(value))
        .transpose()?;
    let direction = args
        .get(2)
        .map(|value| match value {
            XdlValue::String(direction) if direction.eq_ignore_ascii_case("first") => Ok("first"),
            XdlValue::String(direction) if direction.eq_ignore_ascii_case("last") => Ok("last"),
            XdlValue::String(_) => Err(XdlError::InvalidArgument(
                "MATLAB_FIND: direction must be 'first' or 'last'".to_string(),
            )),
            other => Err(XdlError::TypeMismatch {
                expected: "string direction ('first' or 'last')".to_string(),
                actual: format!("{:?}", other.gdl_type()),
            }),
        })
        .transpose()?
        .unwrap_or("first");

    let dimensions = matlab_find_dimensions(&args[0])?;
    let values = matlab_find_values(&args[0], &dimensions)?;
    let matching_indices: Vec<f64> = values
        .iter()
        .enumerate()
        .filter_map(|(index, value)| (*value != 0.0).then_some((index + 1) as f64))
        .collect();
    let indices = match (limit, direction) {
        (Some(limit), "last") => matching_indices
            .into_iter()
            .rev()
            .take(limit)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect(),
        (Some(limit), _) => matching_indices.into_iter().take(limit).collect(),
        (None, _) => matching_indices,
    };
    let shape = if dimensions == [0, 0] || (dimensions == [1, 1] && values.first() == Some(&0.0)) {
        vec![0, 0]
    } else if dimensions.len() == 2 && dimensions[0] == 1 {
        vec![1, indices.len()]
    } else {
        vec![indices.len(), 1]
    };
    Ok(XdlValue::MultiDimArray {
        data: indices,
        shape,
    })
}

/// First output of MATLAB's two-output `find`: one-based row subscripts.
pub fn matlab_find_rows(args: &[XdlValue]) -> XdlResult<XdlValue> {
    let (rows, _, _) = matlab_find_subscripts(args)?;
    Ok(XdlValue::MultiDimArray {
        shape: vec![rows.len(), 1],
        data: rows,
    })
}

/// Second output of MATLAB's two-output `find`: one-based column subscripts.
pub fn matlab_find_columns(args: &[XdlValue]) -> XdlResult<XdlValue> {
    let (_, columns, _) = matlab_find_subscripts(args)?;
    Ok(XdlValue::MultiDimArray {
        shape: vec![columns.len(), 1],
        data: columns,
    })
}

/// Third output of MATLAB's multi-output `find`: one-based linear indices.
pub fn matlab_find_indices(args: &[XdlValue]) -> XdlResult<XdlValue> {
    let (_, _, indices) = matlab_find_subscripts(args)?;
    Ok(XdlValue::MultiDimArray {
        shape: vec![indices.len(), 1],
        data: indices,
    })
}

/// Third output of MATLAB's three-output `find`: matching nonzero values.
pub fn matlab_find_nonzero_values(args: &[XdlValue]) -> XdlResult<XdlValue> {
    let (_, _, indices) = matlab_find_subscripts(args)?;
    let dimensions = matlab_find_dimensions(&args[0])?;
    let values = matlab_find_values(&args[0], &dimensions)?;
    let selected: Vec<f64> = indices
        .into_iter()
        .map(|index| {
            values
                .get(index as usize - 1)
                .copied()
                .ok_or_else(|| XdlError::RuntimeError("MATLAB_FIND: invalid internal index".into()))
        })
        .collect::<XdlResult<_>>()?;
    Ok(XdlValue::MultiDimArray {
        shape: vec![selected.len(), 1],
        data: selected,
    })
}

fn matlab_find_subscripts(args: &[XdlValue]) -> XdlResult<(Vec<f64>, Vec<f64>, Vec<f64>)> {
    if !(1..=3).contains(&args.len()) {
        return Err(XdlError::InvalidArgument(format!(
            "MATLAB_FIND_SUBSCRIPTS: Expected 1-3 arguments, got {}",
            args.len()
        )));
    }
    let limit = args.get(1).map(matlab_find_limit).transpose()?;
    let direction = args
        .get(2)
        .map(|value| match value {
            XdlValue::String(direction) if direction.eq_ignore_ascii_case("first") => Ok(false),
            XdlValue::String(direction) if direction.eq_ignore_ascii_case("last") => Ok(true),
            XdlValue::String(_) => Err(XdlError::InvalidArgument(
                "MATLAB_FIND: direction must be 'first' or 'last'".to_string(),
            )),
            other => Err(XdlError::TypeMismatch {
                expected: "string direction ('first' or 'last')".to_string(),
                actual: format!("{:?}", other.gdl_type()),
            }),
        })
        .transpose()?
        .unwrap_or(false);
    let dimensions = matlab_find_dimensions(&args[0])?;
    let values = matlab_find_values(&args[0], &dimensions)?;
    let matches: Vec<usize> = values
        .iter()
        .enumerate()
        .filter(|(_, value)| **value != 0.0)
        .map(|(index, _)| index)
        .collect();
    let matches: Vec<usize> = match (limit, direction) {
        (Some(limit), true) => matches
            .into_iter()
            .rev()
            .take(limit)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect(),
        (Some(limit), false) => matches.into_iter().take(limit).collect(),
        (None, _) => matches,
    };
    // MATLAB's two-subscript form collapses dimensions 2..N into the
    // column coordinate. Rank-one values use their normalized [rows, cols]
    // orientation (e.g. XDL Array is a row vector).
    let row_count = dimensions[0];
    let rows = matches
        .iter()
        .map(|index| (index % row_count + 1) as f64)
        .collect();
    let columns = matches
        .iter()
        .map(|index| (index / row_count + 1) as f64)
        .collect();
    let linear_indices = matches.iter().map(|index| (index + 1) as f64).collect();
    Ok((rows, columns, linear_indices))
}

/// MATLAB treats arrays as at least 2-D and ignores trailing singleton
/// dimensions when determining the effective rank.
fn matlab_find_dimensions(value: &XdlValue) -> XdlResult<Vec<usize>> {
    if matches!(value, XdlValue::MultiDimArray { shape, data } if shape.is_empty() && data.len() == 1)
    {
        return Ok(vec![1, 1]);
    }
    let mut dimensions = matlab_dimensions(value)?;
    if dimensions.is_empty() {
        return Ok(vec![0, 0]);
    }
    while dimensions.len() > 2 && dimensions.last() == Some(&1) {
        dimensions.pop();
    }
    if dimensions.len() == 1 {
        dimensions.push(1);
    }
    Ok(dimensions)
}

/// Return elements in MATLAB's column-major linear order. XDL's shaped
/// MultiDimArray and recursively nested N-D representation use row-major
/// flattening; the existing 2-D nested-row extractor already converts its
/// result to column-major order.
fn matlab_find_values<'a>(value: &'a XdlValue, dimensions: &[usize]) -> XdlResult<Cow<'a, [f64]>> {
    let needs_reorder = matches!(value, XdlValue::MultiDimArray { .. })
        || (matches!(value, XdlValue::NestedArray(_)) && dimensions.len() > 2);
    let values = if needs_reorder && matches!(value, XdlValue::NestedArray(_)) {
        matlab_find_nested_values(value)?
    } else {
        matlab_numeric_values(value)?
    };
    if !needs_reorder || values.len() < 2 || dimensions.len() < 2 {
        return Ok(values);
    }
    let mut column_major = vec![0.0; values.len()];
    let mut coordinates = vec![0usize; dimensions.len()];
    for (row_major_index, value) in values.iter().enumerate() {
        let mut remaining = row_major_index;
        for axis in (0..dimensions.len()).rev() {
            let size = dimensions[axis];
            if size == 0 {
                return Ok(Cow::Owned(Vec::new()));
            }
            coordinates[axis] = remaining % size;
            remaining /= size;
        }
        let column_major_index = coordinates
            .iter()
            .zip(dimensions)
            .fold((0usize, 1usize), |(index, stride), (coordinate, size)| {
                (index + coordinate * stride, stride * size)
            })
            .0;
        column_major[column_major_index] = *value;
    }
    Ok(Cow::Owned(column_major))
}

/// Flatten nested N-D containers in their structural row-major order without
/// applying the special 2-D nested-row conversion in `matlab_numeric_values`.
fn matlab_find_nested_values(value: &XdlValue) -> XdlResult<Cow<'_, [f64]>> {
    match value {
        XdlValue::NestedArray(values) => values
            .iter()
            .map(matlab_find_nested_values)
            .collect::<XdlResult<Vec<_>>>()
            .map(|chunks| Cow::Owned(chunks.into_iter().flat_map(Cow::into_owned).collect())),
        XdlValue::Array(values) => Ok(Cow::Borrowed(values)),
        XdlValue::MultiDimArray { data, .. } => Ok(Cow::Borrowed(data)),
        scalar => matlab_numeric_values(scalar),
    }
}

fn matlab_find_limit(value: &XdlValue) -> XdlResult<usize> {
    let number = match value {
        XdlValue::Byte(_)
        | XdlValue::Int(_)
        | XdlValue::Long(_)
        | XdlValue::Float(_)
        | XdlValue::Double(_)
        | XdlValue::UInt(_)
        | XdlValue::ULong(_)
        | XdlValue::Long64(_)
        | XdlValue::ULong64(_) => value.to_double()?,
        other => {
            return Err(XdlError::TypeMismatch {
                expected: "positive finite integer limit".to_string(),
                actual: format!("{:?}", other.gdl_type()),
            })
        }
    };
    if !number.is_finite() || number < 1.0 || number.fract() != 0.0 {
        return Err(XdlError::InvalidArgument(
            "MATLAB_FIND: k must be a positive finite integer".to_string(),
        ));
    }
    Ok(number.min(usize::MAX as f64) as usize)
}

fn matlab_numeric_values(value: &XdlValue) -> XdlResult<Cow<'_, [f64]>> {
    match value {
        XdlValue::Byte(number) => Ok(Cow::Owned(vec![f64::from(*number)])),
        XdlValue::Int(number) => Ok(Cow::Owned(vec![f64::from(*number)])),
        XdlValue::Long(number) => Ok(Cow::Owned(vec![f64::from(*number)])),
        XdlValue::Float(number) => Ok(Cow::Owned(vec![f64::from(*number)])),
        XdlValue::Double(number) => Ok(Cow::Owned(vec![*number])),
        XdlValue::Array(values) => Ok(Cow::Borrowed(values)),
        XdlValue::MultiDimArray { data, .. } => Ok(Cow::Borrowed(data)),
        XdlValue::NestedArray(values)
            if values
                .iter()
                .all(|value| matches!(value, XdlValue::Array(_))) =>
        {
            let rows: Vec<&[f64]> = values
                .iter()
                .filter_map(|value| match value {
                    XdlValue::Array(row) => Some(row.as_slice()),
                    _ => None,
                })
                .collect();
            let width = rows.first().map_or(0, |row| row.len());
            Ok(Cow::Owned(
                (0..width)
                    .flat_map(|column| rows.iter().filter_map(move |row| row.get(column).copied()))
                    .collect(),
            ))
        }
        XdlValue::NestedArray(values) => values
            .iter()
            .map(matlab_numeric_values)
            .collect::<XdlResult<Vec<_>>>()
            .map(|chunks| Cow::Owned(chunks.into_iter().flat_map(Cow::into_owned).collect())),
        other => Err(XdlError::TypeMismatch {
            expected: "numeric scalar or array".to_string(),
            actual: format!("{:?}", other.gdl_type()),
        }),
    }
}

fn matlab_dimensions(value: &XdlValue) -> XdlResult<Vec<usize>> {
    match value {
        XdlValue::Undefined => Ok(vec![0, 0]),
        XdlValue::Array(values) if values.is_empty() => Ok(vec![0, 0]),
        XdlValue::Array(values) => Ok(vec![1, values.len()]),
        XdlValue::MultiDimArray { shape, data } => {
            let elements = shape
                .iter()
                .try_fold(1usize, |count, dimension| count.checked_mul(*dimension));
            if elements != Some(data.len()) {
                return Err(XdlError::InvalidArgument(
                    "MATLAB_ARRAY_SHAPE: shape does not match the data length".to_string(),
                ));
            }
            Ok(shape.clone())
        }
        XdlValue::NestedArray(values) => {
            if values.is_empty() {
                return Ok(vec![0, 0]);
            }
            if values
                .iter()
                .all(|value| matches!(value, XdlValue::Array(_)))
            {
                let row_width = match values.first() {
                    Some(XdlValue::Array(row)) => row.len(),
                    _ => {
                        return Err(XdlError::InvalidArgument(
                            "MATLAB_ARRAY_SHAPE: expected rectangular matrix rows".to_string(),
                        ))
                    }
                };
                if values
                    .iter()
                    .any(|value| matches!(value, XdlValue::Array(row) if row.len() != row_width))
                {
                    return Err(XdlError::InvalidArgument(
                        "MATLAB_ARRAY_SHAPE: nested matrix rows must have equal lengths"
                            .to_string(),
                    ));
                }
                return Ok(vec![values.len(), row_width]);
            }
            let child_shape = matlab_dimensions(&values[0])?;
            for value in &values[1..] {
                if matlab_dimensions(value)? != child_shape {
                    return Err(XdlError::InvalidArgument(
                        "MATLAB_ARRAY_SHAPE: nested array must be rectangular".to_string(),
                    ));
                }
            }
            Ok(std::iter::once(values.len()).chain(child_shape).collect())
        }
        XdlValue::String(value) if value.is_empty() => Ok(vec![0, 0]),
        // XDL's String representation cannot distinguish MATLAB char vectors
        // from string scalars; treat nonempty values as char vectors here.
        XdlValue::String(value) => Ok(vec![1, value.encode_utf16().count()]),
        _ => Ok(vec![1, 1]),
    }
}

#[cfg(test)]
mod matlab_length_tests {
    use super::{
        matlab_find, matlab_find_columns, matlab_find_indices, matlab_find_rows, matlab_index,
        matlab_index_vector, matlab_length, matlab_numel, matlab_size, matlab_size_tail,
    };
    use xdl_core::XdlValue;

    #[test]
    fn matlab_length_returns_largest_vector_or_matrix_dimension() {
        let cases = [
            (XdlValue::Double(4.0), 1),
            (XdlValue::Array(vec![]), 0),
            (XdlValue::Array(vec![1.0, 2.0, 3.0]), 3),
            (
                XdlValue::NestedArray(vec![
                    XdlValue::Array(vec![1.0, 2.0, 3.0]),
                    XdlValue::Array(vec![4.0, 5.0, 6.0]),
                ]),
                3,
            ),
            (
                XdlValue::MultiDimArray {
                    data: vec![0.0; 24],
                    shape: vec![2, 3, 4],
                },
                4,
            ),
        ];

        for (value, expected) in cases {
            assert_eq!(
                matlab_length(&[value]).unwrap(),
                XdlValue::Double(expected as f64)
            );
        }
    }

    #[test]
    fn matlab_length_rejects_ragged_nested_arrays() {
        let ragged = XdlValue::NestedArray(vec![
            XdlValue::Array(vec![1.0, 2.0]),
            XdlValue::Array(vec![3.0]),
        ]);
        assert!(matlab_length(&[ragged]).is_err());
    }

    #[test]
    fn matlab_numel_counts_scalar_elements_in_nested_matrices() {
        let matrix = XdlValue::NestedArray(vec![
            XdlValue::Array(vec![1.0, 2.0, 3.0]),
            XdlValue::Array(vec![4.0, 5.0, 6.0]),
        ]);
        assert_eq!(matlab_numel(&[matrix]).unwrap(), XdlValue::Double(6.0));
    }

    #[test]
    fn matlab_size_reports_vector_matrix_and_trailing_dimension_shapes() {
        let cases = [
            (XdlValue::Double(4.0), vec![1.0, 1.0]),
            (XdlValue::Array(vec![1.0, 2.0, 3.0]), vec![1.0, 3.0]),
            (
                XdlValue::NestedArray(vec![
                    XdlValue::Array(vec![1.0, 2.0, 3.0]),
                    XdlValue::Array(vec![4.0, 5.0, 6.0]),
                ]),
                vec![2.0, 3.0],
            ),
            (
                XdlValue::NestedArray(vec![
                    XdlValue::Double(1.0),
                    XdlValue::Double(2.0),
                    XdlValue::Double(3.0),
                ]),
                vec![3.0, 1.0],
            ),
            (
                XdlValue::MultiDimArray {
                    data: vec![0.0; 24],
                    shape: vec![2, 3, 4, 1],
                },
                vec![2.0, 3.0, 4.0],
            ),
        ];

        for (value, expected) in cases {
            assert_eq!(matlab_size(&[value]).unwrap(), XdlValue::Array(expected));
        }
    }

    #[test]
    fn matlab_size_selects_requested_dimension_and_rejects_invalid_dimensions() {
        let matrix = XdlValue::NestedArray(vec![
            XdlValue::Array(vec![1.0, 2.0, 3.0]),
            XdlValue::Array(vec![4.0, 5.0, 6.0]),
        ]);
        assert_eq!(
            matlab_size(&[matrix.clone(), XdlValue::Long(1)]).unwrap(),
            XdlValue::Double(2.0)
        );
        assert_eq!(
            matlab_size(&[matrix.clone(), XdlValue::Long(2)]).unwrap(),
            XdlValue::Double(3.0)
        );
        assert_eq!(
            matlab_size(&[matrix.clone(), XdlValue::Long(4)]).unwrap(),
            XdlValue::Double(1.0)
        );
        assert!(matlab_size(&[matrix.clone(), XdlValue::Long(0)]).is_err());
        assert!(matlab_size(&[matrix, XdlValue::Double(1.5)]).is_err());
    }

    #[test]
    fn matlab_size_tail_multiplies_dimensions_from_starting_dimension() {
        let cases = [
            (
                XdlValue::NestedArray(vec![
                    XdlValue::Array(vec![1.0, 2.0, 3.0]),
                    XdlValue::Array(vec![4.0, 5.0, 6.0]),
                ]),
                2,
                3.0,
            ),
            (
                XdlValue::MultiDimArray {
                    data: vec![0.0; 24],
                    shape: vec![2, 3, 4],
                },
                2,
                12.0,
            ),
            (
                XdlValue::MultiDimArray {
                    data: vec![0.0; 24],
                    shape: vec![2, 3, 4, 1],
                },
                3,
                4.0,
            ),
            (
                XdlValue::MultiDimArray {
                    data: vec![0.0; 24],
                    shape: vec![2, 3, 4, 1],
                },
                4,
                1.0,
            ),
            (
                XdlValue::MultiDimArray {
                    data: vec![0.0; 24],
                    shape: vec![2, 3, 4],
                },
                4,
                1.0,
            ),
            (XdlValue::Double(7.0), 1, 1.0),
            (XdlValue::Undefined, 1, 0.0),
        ];

        for (value, dimension, expected) in cases {
            assert_eq!(
                matlab_size_tail(&[value, XdlValue::Long(dimension)]).unwrap(),
                XdlValue::Double(expected)
            );
        }
    }

    #[test]
    fn matlab_size_tail_rejects_invalid_dimensions_and_argument_counts() {
        let value = XdlValue::MultiDimArray {
            data: vec![1.0, 2.0],
            shape: vec![2],
        };
        for dimension in [
            XdlValue::Long(0),
            XdlValue::Double(-1.0),
            XdlValue::Double(1.5),
            XdlValue::Double(f64::INFINITY),
            XdlValue::Double(f64::NAN),
        ] {
            assert!(matlab_size_tail(&[value.clone(), dimension]).is_err());
        }
        assert!(matlab_size_tail(&[value.clone()]).is_err());
        assert!(matlab_size_tail(&[value, XdlValue::Long(1), XdlValue::Long(2)]).is_err());
    }

    #[test]
    fn matlab_find_returns_one_based_column_indices_and_empty_column() {
        assert_eq!(
            matlab_find(&[XdlValue::Array(vec![0.0, 2.0, 0.0, f64::NAN])]).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![2.0, 4.0],
                shape: vec![1, 2],
            }
        );
        assert_eq!(
            matlab_find(&[XdlValue::Array(vec![0.0, 0.0])]).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![],
                shape: vec![1, 0],
            }
        );
    }

    #[test]
    fn matlab_find_flattens_rectangular_matrices_column_major() {
        let matrix = XdlValue::NestedArray(vec![
            XdlValue::Array(vec![0.0, 1.0, 0.0]),
            XdlValue::Array(vec![2.0, 0.0, 3.0]),
        ]);
        assert_eq!(
            matlab_find(&[matrix]).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![2.0, 3.0, 6.0],
                shape: vec![3, 1],
            }
        );
    }

    #[test]
    fn matlab_find_two_outputs_return_one_based_row_and_column_indices() {
        let matrix = XdlValue::NestedArray(vec![
            XdlValue::Array(vec![0.0, 2.0, 0.0]),
            XdlValue::Array(vec![3.0, 0.0, 4.0]),
        ]);
        assert_eq!(
            matlab_find_rows(std::slice::from_ref(&matrix)).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![2.0, 1.0, 2.0],
                shape: vec![3, 1],
            }
        );
        assert_eq!(
            matlab_find_columns(&[matrix]).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![1.0, 2.0, 3.0],
                shape: vec![3, 1],
            }
        );
    }

    #[test]
    fn matlab_find_nd_uses_column_major_linear_order_and_collapsed_column_subscripts() {
        // XDL MultiDimArray data is row-major; MATLAB find traverses these
        // [2, 2, 2] coordinates in column-major order.
        let array = XdlValue::MultiDimArray {
            data: vec![0.0, 1.0, 2.0, 0.0, 3.0, 0.0, 0.0, 4.0],
            shape: vec![2, 2, 2],
        };
        assert_eq!(
            matlab_find_indices(std::slice::from_ref(&array)).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![2.0, 3.0, 5.0, 8.0],
                shape: vec![4, 1],
            }
        );
        assert_eq!(
            matlab_find_rows(std::slice::from_ref(&array)).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![2.0, 1.0, 1.0, 2.0],
                shape: vec![4, 1],
            }
        );
        assert_eq!(
            matlab_find_columns(&[array]).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![1.0, 2.0, 3.0, 4.0],
                shape: vec![4, 1],
            }
        );

        let nested = XdlValue::NestedArray(vec![
            XdlValue::NestedArray(vec![
                XdlValue::Array(vec![0.0, 1.0]),
                XdlValue::Array(vec![2.0, 0.0]),
            ]),
            XdlValue::NestedArray(vec![
                XdlValue::Array(vec![3.0, 0.0]),
                XdlValue::Array(vec![0.0, 4.0]),
            ]),
        ]);
        assert_eq!(
            matlab_find_indices(&[nested]).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![2.0, 3.0, 5.0, 8.0],
                shape: vec![4, 1],
            }
        );
    }

    #[test]
    fn matlab_find_vectors_preserve_row_or_column_coordinates_and_limit_direction() {
        let row = XdlValue::Array(vec![0.0, 3.0, 0.0, 4.0]);
        assert_eq!(
            matlab_find_rows(std::slice::from_ref(&row)).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![1.0, 1.0],
                shape: vec![2, 1],
            }
        );
        assert_eq!(
            matlab_find_columns(&[row.clone()]).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![2.0, 4.0],
                shape: vec![2, 1],
            }
        );
        let last_args = [row, XdlValue::Long(1), XdlValue::String("last".into())];
        assert_eq!(
            matlab_find_indices(&last_args).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![4.0],
                shape: vec![1, 1],
            }
        );

        let column = XdlValue::NestedArray(vec![
            XdlValue::Double(0.0),
            XdlValue::Double(3.0),
            XdlValue::Double(0.0),
            XdlValue::Double(4.0),
        ]);
        assert_eq!(
            matlab_find_rows(std::slice::from_ref(&column)).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![2.0, 4.0],
                shape: vec![2, 1],
            }
        );
        assert_eq!(
            matlab_find_columns(&[column]).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![1.0, 1.0],
                shape: vec![2, 1],
            }
        );
    }

    #[test]
    fn matlab_find_nd_first_and_last_limits_keep_column_major_order() {
        let array = XdlValue::MultiDimArray {
            data: vec![0.0, 1.0, 2.0, 0.0, 3.0, 0.0, 0.0, 4.0],
            shape: vec![2, 2, 2],
        };
        let first = [array.clone(), XdlValue::Long(2)];
        assert_eq!(
            matlab_find(&first).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![2.0, 3.0],
                shape: vec![2, 1],
            }
        );
        let last = [array, XdlValue::Long(2), XdlValue::String("last".into())];
        assert_eq!(
            matlab_find_indices(&last).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![5.0, 8.0],
                shape: vec![2, 1],
            }
        );
        assert_eq!(
            matlab_find_rows(&last).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![1.0, 2.0],
                shape: vec![2, 1],
            }
        );
    }

    #[test]
    fn matlab_find_two_outputs_honor_limit_and_direction() {
        let matrix = XdlValue::NestedArray(vec![
            XdlValue::Array(vec![0.0, 2.0, 0.0]),
            XdlValue::Array(vec![3.0, 0.0, 4.0]),
        ]);
        let first_args = [matrix.clone(), XdlValue::Long(2)];
        assert_eq!(
            matlab_find_rows(&first_args).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![2.0, 1.0],
                shape: vec![2, 1]
            }
        );
        let last_args = [matrix, XdlValue::Long(2), XdlValue::String("last".into())];
        assert_eq!(
            matlab_find_columns(&last_args).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![2.0, 3.0],
                shape: vec![2, 1]
            }
        );
        assert_eq!(
            matlab_find_indices(&last_args).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![3.0, 6.0],
                shape: vec![2, 1]
            }
        );
    }

    #[test]
    fn matlab_find_two_outputs_shape_empty_results_accept_vectors_and_reject_ragged_matrices() {
        let zeros = XdlValue::MultiDimArray {
            data: vec![0.0; 6],
            shape: vec![2, 3],
        };
        assert_eq!(
            matlab_find_rows(std::slice::from_ref(&zeros)).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![],
                shape: vec![0, 1],
            }
        );
        assert_eq!(
            matlab_find_columns(&[zeros]).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![],
                shape: vec![0, 1],
            }
        );
        let ragged = XdlValue::NestedArray(vec![
            XdlValue::Array(vec![1.0, 2.0]),
            XdlValue::Array(vec![3.0]),
        ]);
        assert!(matlab_find_rows(&[ragged]).is_err());
        assert_eq!(
            matlab_find_rows(&[XdlValue::Array(vec![1.0, 0.0])]).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![1.0],
                shape: vec![1, 1],
            }
        );
    }

    #[test]
    fn matlab_find_returns_column_for_column_vector_and_matrix() {
        let column = XdlValue::NestedArray(vec![
            XdlValue::Double(0.0),
            XdlValue::Double(2.0),
            XdlValue::Double(0.0),
            XdlValue::Double(4.0),
        ]);
        assert_eq!(
            matlab_find(&[column]).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![2.0, 4.0],
                shape: vec![2, 1],
            }
        );
        let matrix = XdlValue::NestedArray(vec![
            XdlValue::Array(vec![0.0, 1.0]),
            XdlValue::Array(vec![2.0, 0.0]),
        ]);
        assert_eq!(
            matlab_find(&[matrix]).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![2.0, 3.0],
                shape: vec![2, 1],
            }
        );
    }

    #[test]
    fn matlab_find_empty_matrix_and_zero_scalar_follow_empty_matrix_shape() {
        for input in [XdlValue::Array(vec![]), XdlValue::Double(0.0)] {
            assert_eq!(
                matlab_find(&[input]).unwrap(),
                XdlValue::MultiDimArray {
                    data: vec![],
                    shape: vec![0, 0],
                }
            );
        }
    }

    #[test]
    fn matlab_find_handles_nonzero_scalar_as_one_based_singleton() {
        assert_eq!(
            matlab_find(&[XdlValue::Double(7.0)]).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![1.0],
                shape: vec![1, 1],
            }
        );
    }

    #[test]
    fn matlab_find_first_and_last_limits_keep_indices_ascending() {
        let row = XdlValue::Array(vec![1.0, 0.0, 2.0, 0.0, 3.0]);
        assert_eq!(
            matlab_find(&[row.clone(), XdlValue::Long(2)]).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![1.0, 3.0],
                shape: vec![1, 2]
            }
        );
        assert_eq!(
            matlab_find(&[
                row.clone(),
                XdlValue::Long(2),
                XdlValue::String("last".into())
            ])
            .unwrap(),
            XdlValue::MultiDimArray {
                data: vec![3.0, 5.0],
                shape: vec![1, 2]
            }
        );
        assert_eq!(
            matlab_find(&[row, XdlValue::Long(20), XdlValue::String("last".into())]).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![1.0, 3.0, 5.0],
                shape: vec![1, 3]
            }
        );
    }

    #[test]
    fn matlab_find_limit_and_direction_reject_invalid_values() {
        let input = XdlValue::Array(vec![1.0, 0.0, 2.0]);
        for invalid_limit in [
            XdlValue::Long(0),
            XdlValue::Double(-1.0),
            XdlValue::Double(1.5),
            XdlValue::Double(f64::INFINITY),
            XdlValue::String("2".into()),
        ] {
            assert!(matlab_find(&[input.clone(), invalid_limit]).is_err());
        }
        assert!(matlab_find(&[
            input.clone(),
            XdlValue::Long(1),
            XdlValue::String("middle".into())
        ])
        .is_err());
        assert!(matlab_find(&[input.clone(), XdlValue::Long(1), XdlValue::Long(1)]).is_err());
        assert!(matlab_find(&[
            input.clone(),
            XdlValue::Long(1),
            XdlValue::String("first".into()),
            XdlValue::Long(1)
        ])
        .is_err());
    }

    #[test]
    fn matlab_find_limits_preserve_matrix_column_major_order_and_column_shape() {
        let matrix = XdlValue::NestedArray(vec![
            XdlValue::Array(vec![0.0, 1.0, 0.0]),
            XdlValue::Array(vec![2.0, 0.0, 3.0]),
        ]);
        assert_eq!(
            matlab_find(&[
                matrix.clone(),
                XdlValue::Long(2),
                XdlValue::String("last".into())
            ])
            .unwrap(),
            XdlValue::MultiDimArray {
                data: vec![3.0, 6.0],
                shape: vec![2, 1]
            }
        );
        assert_eq!(
            matlab_find(&[matrix, XdlValue::Long(1)]).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![2.0],
                shape: vec![1, 1]
            }
        );
    }

    #[test]
    fn matlab_find_saturates_oversized_finite_limit_to_all_matches() {
        let input = XdlValue::Array(vec![1.0, 0.0, 2.0]);
        assert_eq!(
            matlab_find(&[input, XdlValue::Double(f64::MAX)]).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![1.0, 3.0],
                shape: vec![1, 2],
            }
        );
    }

    #[test]
    fn matlab_index_reads_flat_shaped_and_nested_numeric_values() {
        let flat = XdlValue::Array(vec![10.0, 20.0, 30.0]);
        assert_eq!(
            matlab_index(&[flat, XdlValue::Long(2)]).unwrap(),
            XdlValue::Double(20.0)
        );

        let shaped = XdlValue::MultiDimArray {
            data: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            shape: vec![2, 3],
        };
        assert_eq!(
            matlab_index(&[shaped, XdlValue::Long(5)]).unwrap(),
            XdlValue::Double(5.0)
        );

        let nested_matrix = XdlValue::NestedArray(vec![
            XdlValue::Array(vec![1.0, 3.0, 5.0]),
            XdlValue::Array(vec![2.0, 4.0, 6.0]),
        ]);
        let indexed: Vec<XdlValue> = (1..=6)
            .map(|index| matlab_index(&[nested_matrix.clone(), XdlValue::Long(index)]).unwrap())
            .collect();
        assert_eq!(
            indexed,
            (1..=6)
                .map(|index| XdlValue::Double(index as f64))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn matlab_index_rejects_invalid_indices_and_ragged_inputs() {
        let array = XdlValue::Array(vec![7.0, 8.0]);
        for index in [
            XdlValue::Long(0),
            XdlValue::Long(-1),
            XdlValue::Double(1.5),
            XdlValue::Double(f64::NAN),
        ] {
            assert!(matlab_index(&[array.clone(), index]).is_err());
        }
        assert!(matlab_index(&[array.clone(), XdlValue::Long(3)]).is_err());
        assert!(matlab_index(&[array, XdlValue::String("1".to_string())]).is_err());

        let ragged = XdlValue::NestedArray(vec![
            XdlValue::Array(vec![1.0, 2.0]),
            XdlValue::Array(vec![3.0]),
        ]);
        assert!(matlab_index(&[ragged, XdlValue::Long(1)]).is_err());
    }

    #[test]
    fn matlab_index_vector_preserves_index_order_and_duplicates() {
        let source = XdlValue::Array(vec![10.0, 20.0, 30.0, 40.0]);
        let indices = XdlValue::Array(vec![4.0, 1.0, 4.0, 2.0]);
        assert_eq!(
            matlab_index_vector(&[source.clone(), indices]).unwrap(),
            XdlValue::Array(vec![40.0, 10.0, 40.0, 20.0])
        );
        assert_eq!(
            matlab_index_vector(&[source, XdlValue::Long(3)]).unwrap(),
            XdlValue::Double(30.0)
        );
        let matrix = XdlValue::NestedArray(vec![
            XdlValue::Array(vec![10.0, 20.0]),
            XdlValue::Array(vec![30.0, 40.0]),
        ]);
        assert_eq!(
            matlab_index_vector(&[matrix, XdlValue::Array(vec![2.0, 4.0])]).unwrap(),
            XdlValue::Array(vec![30.0, 40.0])
        );
    }

    #[test]
    fn matlab_index_vector_rejects_invalid_indices_and_shapes() {
        let source = XdlValue::Array(vec![10.0, 20.0, 30.0]);
        for index in [0.0, -1.0, 1.5, f64::NAN, f64::INFINITY, 4.0] {
            assert!(
                matlab_index_vector(&[source.clone(), XdlValue::Array(vec![1.0, index])]).is_err()
            );
        }
        assert_eq!(
            matlab_index_vector(&[source.clone(), XdlValue::Array(Vec::new())]).unwrap(),
            XdlValue::Array(Vec::new())
        );
        assert!(
            matlab_index_vector(&[XdlValue::Array(Vec::new()), XdlValue::Array(vec![1.0])])
                .is_err()
        );
        assert_eq!(
            matlab_index_vector(&[XdlValue::Array(Vec::new()), XdlValue::Array(Vec::new())])
                .unwrap(),
            XdlValue::Array(Vec::new())
        );
        assert!(matlab_index_vector(&[source.clone(), XdlValue::String("1".into())]).is_err());
        assert!(matlab_index_vector(&[
            source,
            XdlValue::MultiDimArray {
                data: vec![1.0, 2.0],
                shape: vec![1, 2],
            }
        ])
        .is_err());
    }
}

#[cfg(test)]
mod matlab_mldivide_tests {
    use super::matlab_mldivide;
    use xdl_core::XdlValue;

    fn matrix(rows: &[&[f64]]) -> XdlValue {
        XdlValue::NestedArray(
            rows.iter()
                .map(|row| XdlValue::Array(row.to_vec()))
                .collect(),
        )
    }

    fn assert_close(actual: &[f64], expected: &[f64]) {
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in actual.iter().zip(expected) {
            assert!((actual - expected).abs() < 1e-10, "{actual} != {expected}");
        }
    }

    #[test]
    fn solves_square_system_and_preserves_vector_shape() {
        let a = matrix(&[&[3.0, 1.0], &[1.0, 2.0]]);
        let b = XdlValue::Array(vec![9.0, 8.0]);
        match matlab_mldivide(&[a, b]).unwrap() {
            XdlValue::Array(solution) => assert_close(&solution, &[2.0, 3.0]),
            other => panic!("expected vector result, got {other:?}"),
        }
    }

    #[test]
    fn solves_multiple_rhs_columns_in_row_major_shape() {
        let a = XdlValue::MultiDimArray {
            data: vec![3.0, 1.0, 1.0, 2.0],
            shape: vec![2, 2],
        };
        let b = XdlValue::MultiDimArray {
            data: vec![9.0, 4.0, 8.0, 5.0],
            shape: vec![2, 2],
        };
        assert_eq!(
            matlab_mldivide(&[a, b]).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![2.0, 0.6, 3.0, 2.2],
                shape: vec![2, 2]
            }
        );
    }

    #[test]
    fn rectangular_left_division_returns_least_squares_and_minimum_norm_solutions() {
        let overdetermined = matrix(&[&[1.0, 0.0], &[1.0, 1.0], &[1.0, 2.0]]);
        match matlab_mldivide(&[overdetermined, XdlValue::Array(vec![1.0, 2.0, 2.0])]).unwrap() {
            XdlValue::Array(solution) => assert_close(&solution, &[7.0 / 6.0, 0.5]),
            other => panic!("expected overdetermined vector solution, got {other:?}"),
        }

        let overdetermined = matrix(&[&[1.0, 0.0], &[1.0, 1.0], &[1.0, 2.0]]);
        let multiple_rhs = matrix(&[&[1.0, 2.0], &[2.0, 4.0], &[2.0, 4.0]]);
        match matlab_mldivide(&[overdetermined, multiple_rhs]).unwrap() {
            XdlValue::MultiDimArray { data, shape } => {
                assert_eq!(shape, [2, 2]);
                assert_close(&data, &[7.0 / 6.0, 7.0 / 3.0, 0.5, 1.0]);
            }
            other => panic!("expected rectangular matrix solution, got {other:?}"),
        }

        let underdetermined = matrix(&[&[1.0, 1.0]]);
        match matlab_mldivide(&[underdetermined, XdlValue::Array(vec![2.0])]).unwrap() {
            XdlValue::Array(solution) => assert_close(&solution, &[1.0, 1.0]),
            other => panic!("expected minimum-norm vector solution, got {other:?}"),
        }
    }

    #[test]
    fn rectangular_solve_is_invariant_to_uniform_scaling() {
        for scale in [1.0e-12, 1.0e12] {
            let a = matrix(&[&[scale, 0.0], &[0.0, scale], &[scale, scale]]);
            let b = XdlValue::Array(vec![scale, 2.0 * scale, 3.0 * scale]);
            match matlab_mldivide(&[a, b]).unwrap() {
                XdlValue::Array(solution) => assert_close(&solution, &[1.0, 2.0]),
                other => panic!("expected scale-invariant vector solution, got {other:?}"),
            }
        }
    }

    #[test]
    fn rank_deficient_rectangular_solve_returns_a_minimum_norm_solution() {
        let a = matrix(&[&[1.0, 1.0], &[2.0, 2.0], &[3.0, 3.0]]);
        let b = XdlValue::Array(vec![2.0, 4.0, 6.0]);
        match matlab_mldivide(&[a, b]).unwrap() {
            XdlValue::Array(solution) => assert_close(&solution, &[1.0, 1.0]),
            other => panic!("expected minimum-norm vector solution, got {other:?}"),
        }
    }

    #[test]
    fn rejects_singular_and_invalid_shapes() {
        let singular = matrix(&[&[1.0, 2.0], &[2.0, 4.0]]);
        assert!(matlab_mldivide(&[singular, XdlValue::Array(vec![3.0, 6.0])]).is_err());

        let rectangular = matrix(&[&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]]);
        assert!(matlab_mldivide(&[rectangular, XdlValue::Array(vec![1.0])]).is_err());

        let square = matrix(&[&[1.0, 0.0], &[0.0, 1.0]]);
        assert!(matlab_mldivide(&[square, XdlValue::Array(vec![1.0])]).is_err());

        let ragged = XdlValue::NestedArray(vec![
            XdlValue::Array(vec![1.0, 0.0]),
            XdlValue::Array(vec![0.0]),
        ]);
        assert!(matlab_mldivide(&[ragged, XdlValue::Array(vec![1.0, 2.0])]).is_err());

        let identity = matrix(&[&[1.0, 0.0], &[0.0, 1.0]]);
        assert!(
            matlab_mldivide(&[identity.clone(), XdlValue::Array(vec![f64::NAN, 1.0])]).is_err()
        );
        assert!(matlab_mldivide(&[identity, XdlValue::String("not numeric".into())]).is_err());

        let tiny = matrix(&[&[1.0e-320]]);
        assert!(matlab_mldivide(&[tiny, XdlValue::Array(vec![1.0e308])]).is_err());
    }
}

/// MESHGRID - Create 2D coordinate matrices from coordinate vectors
///
/// Usage:
///   MESHGRID, x_vec, y_vec, X, Y
///
/// Creates 2D coordinate matrices X and Y from 1D coordinate vectors x_vec and y_vec.
/// This is equivalent to MATLAB's [X, Y] = meshgrid(x_vec, y_vec).
///
/// Example:
///   x = FINDGEN(5)  ; [0, 1, 2, 3, 4]
///   y = FINDGEN(3)  ; [0, 1, 2]
///   MESHGRID, x, y, X, Y
///   ; X will be 5x3 matrix with rows = [0,1,2,3,4]
///   ; Y will be 5x3 matrix with cols = [0,1,2]
pub fn meshgrid(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 2 {
        return Err(XdlError::InvalidArgument(format!(
            "MESHGRID: Expected 2 arguments (x_vec, y_vec), got {}",
            args.len()
        )));
    }

    // Extract input vectors
    let x_vec = match &args[0] {
        XdlValue::Array(arr) => arr.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let y_vec = match &args[1] {
        XdlValue::Array(arr) => arr.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let nx = x_vec.len();
    let ny = y_vec.len();

    // Create X matrix (rows are copies of x_vec)
    let mut x_data = Vec::with_capacity(nx * ny);
    for _j in 0..ny {
        x_data.extend_from_slice(&x_vec);
    }

    // Create Y matrix (columns are copies of y_vec)
    let mut y_data = Vec::with_capacity(nx * ny);
    #[allow(clippy::needless_range_loop)]
    for j in 0..ny {
        for _i in 0..nx {
            y_data.push(y_vec[j]);
        }
    }

    // Return as nested array [X, Y]
    Ok(XdlValue::NestedArray(vec![
        XdlValue::MultiDimArray {
            data: x_data,
            shape: vec![nx, ny],
        },
        XdlValue::MultiDimArray {
            data: y_data,
            shape: vec![nx, ny],
        },
    ]))
}

/// LINSPACE - Generate linearly spaced vector
///
/// Usage:
///   result = LINSPACE(start, stop, n)
///
/// Generates n evenly spaced values from start to stop (inclusive).
/// This is equivalent to MATLAB's linspace(start, stop, n).
///
/// Example:
///   x = LINSPACE(0, 10, 11)  ; [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
pub fn linspace(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 || args.len() > 3 {
        return Err(XdlError::InvalidArgument(format!(
            "LINSPACE: Expected 2-3 arguments (start, stop, [n]), got {}",
            args.len()
        )));
    }

    let start = args[0].to_double()?;
    let stop = args[1].to_double()?;
    let n = if args.len() == 3 {
        args[2].to_long()? as usize
    } else {
        100 // Default to 100 points like MATLAB
    };

    if n < 2 {
        return Err(XdlError::InvalidArgument(
            "LINSPACE: n must be at least 2".to_string(),
        ));
    }

    let step = (stop - start) / ((n - 1) as f64);
    let data: Vec<f64> = (0..n).map(|i| start + (i as f64) * step).collect();

    Ok(XdlValue::Array(data))
}

/// LOGSPACE - Generate logarithmically spaced vector
///
/// Usage:
///   result = LOGSPACE(start, stop, n)
///
/// Generates n logarithmically spaced values from 10^start to 10^stop.
/// This is equivalent to MATLAB's logspace(start, stop, n).
///
/// Example:
///   x = LOGSPACE(0, 3, 4)  ; [1, 10, 100, 1000]
pub fn logspace(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 || args.len() > 3 {
        return Err(XdlError::InvalidArgument(format!(
            "LOGSPACE: Expected 2-3 arguments (start, stop, [n]), got {}",
            args.len()
        )));
    }

    let start = args[0].to_double()?;
    let stop = args[1].to_double()?;
    let n = if args.len() == 3 {
        args[2].to_long()? as usize
    } else {
        50 // Default to 50 points like MATLAB
    };

    if n < 2 {
        return Err(XdlError::InvalidArgument(
            "LOGSPACE: n must be at least 2".to_string(),
        ));
    }

    let step = (stop - start) / ((n - 1) as f64);
    let data: Vec<f64> = (0..n)
        .map(|i| 10_f64.powf(start + (i as f64) * step))
        .collect();

    Ok(XdlValue::Array(data))
}

#[cfg(test)]
mod matlab_reshape_tests {
    use super::{matlab_find_indices, matlab_reshape};
    use xdl_core::XdlValue;

    #[test]
    fn reshapes_3d_values_in_matlab_column_major_order_and_find_sees_that_order() {
        let source = XdlValue::Array(vec![
            10.0, 0.0, 30.0, 0.0, 0.0, 60.0, 0.0, 80.0, 0.0, 0.0, 0.0, 120.0,
        ]);
        let reshaped = matlab_reshape(&[
            source,
            XdlValue::Long(2),
            XdlValue::Long(3),
            XdlValue::Long(2),
        ])
        .unwrap();
        assert_eq!(
            reshaped,
            XdlValue::MultiDimArray {
                data: vec![10.0, 0.0, 30.0, 0.0, 0.0, 0.0, 0.0, 80.0, 0.0, 0.0, 60.0, 120.0],
                shape: vec![2, 3, 2],
            }
        );
        assert_eq!(
            matlab_find_indices(std::slice::from_ref(&reshaped)).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![1.0, 3.0, 6.0, 8.0, 12.0],
                shape: vec![5, 1],
            }
        );
    }

    #[test]
    fn accepts_scalar_rectangular_nested_and_shaped_numeric_inputs() {
        let cases = [
            (XdlValue::Double(7.0), vec![1usize, 1], vec![7.0]),
            (
                XdlValue::Array(vec![1.0, 2.0, 3.0, 4.0]),
                vec![2, 2],
                vec![1.0, 3.0, 2.0, 4.0],
            ),
            (
                XdlValue::NestedArray(vec![
                    XdlValue::Array(vec![1.0, 2.0]),
                    XdlValue::Array(vec![3.0, 4.0]),
                ]),
                vec![2, 2],
                vec![1.0, 2.0, 3.0, 4.0],
            ),
            (
                XdlValue::MultiDimArray {
                    data: vec![1.0, 2.0, 3.0, 4.0],
                    shape: vec![2, 2],
                },
                vec![2, 2],
                vec![1.0, 2.0, 3.0, 4.0],
            ),
        ];
        for (input, shape, expected) in cases {
            let dimensions = shape
                .iter()
                .copied()
                .map(|dimension| XdlValue::Long(dimension as i32))
                .collect::<Vec<_>>();
            let mut args = vec![input];
            args.extend(dimensions);
            let result = matlab_reshape(&args).unwrap();
            assert_eq!(
                result,
                XdlValue::MultiDimArray {
                    data: expected,
                    shape
                }
            );
        }
    }

    #[test]
    fn rejects_invalid_dimensions_and_shape_products() {
        let source = XdlValue::Array(vec![1.0, 2.0, 3.0, 4.0]);
        for dimension in [
            XdlValue::Long(-1),
            XdlValue::Double(1.5),
            XdlValue::Double(f64::NAN),
            XdlValue::Double(f64::INFINITY),
            XdlValue::String("2".into()),
        ] {
            assert!(matlab_reshape(&[source.clone(), dimension, XdlValue::Long(2)]).is_err());
        }
        assert!(matlab_reshape(&[source.clone(), XdlValue::Long(2)]).is_err());
        assert!(matlab_reshape(&[source.clone(), XdlValue::Long(2), XdlValue::Long(3)]).is_err());
        assert!(matlab_reshape(&[source, XdlValue::Long(0), XdlValue::Long(0)]).is_err());
    }

    #[test]
    fn rejects_ragged_nested_and_inconsistent_shaped_inputs() {
        let ragged = XdlValue::NestedArray(vec![
            XdlValue::Array(vec![1.0, 2.0]),
            XdlValue::Array(vec![3.0]),
        ]);
        assert!(matlab_reshape(&[ragged, XdlValue::Long(1), XdlValue::Long(3)]).is_err());
        let malformed = XdlValue::MultiDimArray {
            data: vec![1.0, 2.0, 3.0],
            shape: vec![2, 2],
        };
        assert!(matlab_reshape(&[malformed, XdlValue::Long(2), XdlValue::Long(2)]).is_err());
    }
}

/// REPMAT - Replicate and tile an array
///
/// Usage:
///   result = REPMAT(array, m, n)
///
/// Replicates array m times vertically and n times horizontally.
/// This is equivalent to MATLAB's repmat(array, m, n).
///
/// Example:
///   a = [1, 2]
///   b = REPMAT(a, 2, 3)  ; Creates 2x6 array
pub fn repmat(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 3 {
        return Err(XdlError::InvalidArgument(format!(
            "REPMAT: Expected 3 arguments (array, m, n), got {}",
            args.len()
        )));
    }

    let arr = match &args[0] {
        XdlValue::Array(a) => a.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let m = args[1].to_long()? as usize;
    let n = args[2].to_long()? as usize;

    let orig_len = arr.len();
    let mut result = Vec::with_capacity(orig_len * m * n);

    for _ in 0..m {
        for _ in 0..n {
            result.extend_from_slice(&arr);
        }
    }

    Ok(XdlValue::Array(result))
}

/// SQUEEZE - Remove singleton dimensions
///
/// Usage:
///   result = SQUEEZE(array)
///
/// Removes dimensions of size 1 from multi-dimensional array.
/// This is equivalent to MATLAB's squeeze(array).
pub fn squeeze(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "SQUEEZE: Expected 1 argument, got {}",
            args.len()
        )));
    }

    match &args[0] {
        XdlValue::MultiDimArray { data, shape } => {
            let new_shape: Vec<usize> = shape.iter().filter(|&&s| s > 1).copied().collect();

            if new_shape.is_empty() {
                // All dimensions were 1, return scalar
                Ok(XdlValue::Double(data[0]))
            } else if new_shape.len() == 1 {
                // Reduced to 1D array
                Ok(XdlValue::Array(data.clone()))
            } else {
                // Still multi-dimensional
                Ok(XdlValue::MultiDimArray {
                    data: data.clone(),
                    shape: new_shape,
                })
            }
        }
        XdlValue::Array(_) => Ok(args[0].clone()), // Already 1D, no change
        _ => Ok(args[0].clone()),                  // Scalar, no change
    }
}

/// NDGRID - Generate N-D coordinate arrays
///
/// Usage:
///   NDGRID, x1, x2, ..., xn, X1, X2, ..., Xn
///
/// Similar to MESHGRID but follows matrix indexing convention.
/// For 2D case, NDGRID is transpose of MESHGRID.
pub fn ndgrid(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "NDGRID: Expected at least 2 input vectors".to_string(),
        ));
    }

    // For now, implement 2D case (can be extended to N-D)
    if args.len() == 2 {
        // Extract vectors
        let x_vec = match &args[0] {
            XdlValue::Array(arr) => arr.clone(),
            _ => {
                return Err(XdlError::TypeMismatch {
                    expected: "array".to_string(),
                    actual: format!("{:?}", args[0].gdl_type()),
                })
            }
        };

        let y_vec = match &args[1] {
            XdlValue::Array(arr) => arr.clone(),
            _ => {
                return Err(XdlError::TypeMismatch {
                    expected: "array".to_string(),
                    actual: format!("{:?}", args[1].gdl_type()),
                })
            }
        };

        let nx = x_vec.len();
        let ny = y_vec.len();

        // NDGRID: X varies along columns, Y varies along rows (opposite of MESHGRID)
        let mut x_data = Vec::with_capacity(nx * ny);
        #[allow(clippy::needless_range_loop)]
        for _i in 0..nx {
            for j in 0..ny {
                x_data.push(x_vec[j]);
            }
        }

        let mut y_data = Vec::with_capacity(nx * ny);
        #[allow(clippy::needless_range_loop)]
        for i in 0..nx {
            for _j in 0..ny {
                y_data.push(y_vec[i]);
            }
        }

        Ok(XdlValue::NestedArray(vec![
            XdlValue::MultiDimArray {
                data: x_data,
                shape: vec![nx, ny],
            },
            XdlValue::MultiDimArray {
                data: y_data,
                shape: vec![nx, ny],
            },
        ]))
    } else {
        Err(XdlError::NotImplemented(
            "NDGRID: N-D case (N > 2) not yet implemented".to_string(),
        ))
    }
}

/// INTERP1 - 1D interpolation
///
/// Usage:
///   result = INTERP1(x, y, xi, [method])
///
/// Interpolates to find yi = f(xi) where f is defined by (x, y) pairs.
/// Methods: 'linear' (default), 'nearest'. Samples must be strictly monotonic;
/// queries outside the sample interval return NaN, matching MATLAB's default
/// non-extrapolating behavior.
pub fn interp1(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 || args.len() > 4 {
        return Err(XdlError::InvalidArgument(format!(
            "INTERP1: Expected 3-4 arguments (x, y, xi, [method]), got {}",
            args.len()
        )));
    }

    let x = match &args[0] {
        XdlValue::Array(arr) => arr.as_slice(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let y = match &args[1] {
        XdlValue::Array(arr) => arr.as_slice(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let xi = match &args[2] {
        XdlValue::Array(arr) => arr.clone(),
        XdlValue::Double(v) => vec![*v],
        XdlValue::Float(v) => vec![*v as f64],
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "array or scalar".to_string(),
                actual: format!("{:?}", args[2].gdl_type()),
            })
        }
    };

    if x.len() != y.len() {
        return Err(XdlError::InvalidArgument(
            "INTERP1: x and y must have same length".to_string(),
        ));
    }

    if x.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "INTERP1: x and y must contain at least two samples".to_string(),
        ));
    }

    if x.iter().any(|sample| !sample.is_finite()) {
        return Err(XdlError::InvalidArgument(
            "INTERP1: sample locations must be finite".to_string(),
        ));
    }

    let ascending = x[1] > x[0];
    if x.windows(2).any(|pair| {
        if ascending {
            pair[1] <= pair[0]
        } else {
            pair[1] >= pair[0]
        }
    }) {
        return Err(XdlError::InvalidArgument(
            "INTERP1: sample locations must be strictly monotonic".to_string(),
        ));
    }

    let method = match args.get(3) {
        None => "linear",
        Some(XdlValue::String(method)) if method.eq_ignore_ascii_case("linear") => "linear",
        Some(XdlValue::String(method)) if method.eq_ignore_ascii_case("nearest") => "nearest",
        Some(XdlValue::String(method)) => {
            return Err(XdlError::InvalidArgument(format!(
                "INTERP1: unsupported method '{method}'; use 'linear' or 'nearest'"
            )))
        }
        Some(value) => {
            return Err(XdlError::TypeMismatch {
                expected: "string interpolation method".to_string(),
                actual: format!("{:?}", value.gdl_type()),
            })
        }
    };

    let mut yi = Vec::with_capacity(xi.len());

    for &xi_val in &xi {
        let outside = if ascending {
            xi_val < x[0] || xi_val > x[x.len() - 1]
        } else {
            xi_val > x[0] || xi_val < x[x.len() - 1]
        };
        if outside || xi_val.is_nan() {
            yi.push(f64::NAN);
            continue;
        }

        let upper = x.partition_point(|&sample| {
            if ascending {
                sample <= xi_val
            } else {
                sample >= xi_val
            }
        });
        let lower = upper.saturating_sub(1);

        let y_val = if x[lower] == xi_val || lower == x.len() - 1 {
            y[lower]
        } else if method == "nearest" {
            if (xi_val - x[lower]).abs() < (x[upper] - xi_val).abs() {
                y[lower]
            } else {
                y[upper]
            }
        } else {
            y[lower] + (y[upper] - y[lower]) * (xi_val - x[lower]) / (x[upper] - x[lower])
        };

        yi.push(y_val);
    }

    Ok(XdlValue::Array(yi))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matlab_colon_builds_inclusive_runtime_ranges() {
        assert_eq!(
            matlab_colon(&[XdlValue::Double(2.0), XdlValue::Double(5.0)]).unwrap(),
            XdlValue::Array(vec![2.0, 3.0, 4.0, 5.0])
        );
        assert_eq!(
            matlab_colon(&[
                XdlValue::Double(5.0),
                XdlValue::Double(-2.0),
                XdlValue::Double(1.0),
            ])
            .unwrap(),
            XdlValue::Array(vec![5.0, 3.0, 1.0])
        );
        assert_eq!(
            matlab_colon(&[
                XdlValue::Double(0.0),
                XdlValue::Double(0.1),
                XdlValue::Double(0.3),
            ])
            .unwrap(),
            XdlValue::Array(vec![0.0, 0.1, 0.2, 0.30000000000000004])
        );
        assert_eq!(
            matlab_colon(&[
                XdlValue::Double(1.0),
                XdlValue::Double(-1.0),
                XdlValue::Double(3.0),
            ])
            .unwrap(),
            XdlValue::Array(vec![])
        );
        assert_eq!(
            matlab_colon(&[XdlValue::Double(5.0), XdlValue::Double(1.0)]).unwrap(),
            XdlValue::Array(vec![])
        );
    }

    #[test]
    fn matlab_colon_validates_scalars_step_and_allocation_bound() {
        assert!(matlab_colon(&[
            XdlValue::Double(1.0),
            XdlValue::Double(0.0),
            XdlValue::Double(2.0)
        ])
        .unwrap_err()
        .to_string()
        .contains("step must be nonzero"));
        assert!(
            matlab_colon(&[XdlValue::Double(f64::NAN), XdlValue::Double(2.0)])
                .unwrap_err()
                .to_string()
                .contains("start must be finite")
        );
        assert!(matlab_colon(&[XdlValue::Array(vec![1.0, 2.0]), XdlValue::Double(2.0)]).is_err());
        assert_eq!(
            matlab_colon(&[XdlValue::Array(vec![1.0]), XdlValue::Array(vec![3.0])]).unwrap(),
            XdlValue::Array(vec![1.0, 2.0, 3.0])
        );
        assert!(
            matlab_colon(&[XdlValue::Double(1.0), XdlValue::Double(1_000_001.0)])
                .unwrap_err()
                .to_string()
                .contains("exceeds 1000000 elements")
        );
        assert!(matlab_colon(&[XdlValue::Double(1.0)]).is_err());
    }
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEMP_CSV_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temporary_csv(contents: &str) -> String {
        let path = std::env::temp_dir().join(format!(
            "xdl-readmatrix-{}-{}-{}.csv",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock after epoch")
                .as_nanos(),
            TEMP_CSV_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::write(&path, contents).expect("write CSV fixture");
        path.to_string_lossy().into_owned()
    }

    #[test]
    fn matlab_non_finite_literal_helpers_return_ieee_values() {
        assert!(matches!(matlab_nan(&[]), Ok(XdlValue::Double(value)) if value.is_nan()));
        assert!(matches!(matlab_inf(&[]), Ok(XdlValue::Double(value)) if value == f64::INFINITY));
        assert!(matlab_nan(&[XdlValue::Int(1)]).is_err());
        assert!(matlab_inf(&[XdlValue::Int(1)]).is_err());
    }

    #[test]
    fn matlab_ctranspose_transposes_real_shapes_and_conjugates_complex_scalars() {
        assert_eq!(
            matlab_ctranspose(&[XdlValue::Array(vec![7.0, 8.0])]).unwrap(),
            XdlValue::NestedArray(vec![XdlValue::Array(vec![7.0]), XdlValue::Array(vec![8.0]),])
        );
        assert_eq!(
            matlab_ctranspose(&[XdlValue::Double(5.0)]).unwrap(),
            XdlValue::Double(5.0)
        );
        assert_eq!(
            matlab_ctranspose(&[XdlValue::NestedArray(vec![
                XdlValue::Array(vec![1.0, 2.0, 3.0]),
                XdlValue::Array(vec![4.0, 5.0, 6.0]),
            ])])
            .unwrap(),
            XdlValue::NestedArray(vec![
                XdlValue::Array(vec![1.0, 4.0]),
                XdlValue::Array(vec![2.0, 5.0]),
                XdlValue::Array(vec![3.0, 6.0]),
            ])
        );
        assert_eq!(
            matlab_ctranspose(&[XdlValue::MultiDimArray {
                data: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
                shape: vec![2, 3],
            }])
            .unwrap(),
            XdlValue::MultiDimArray {
                data: vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0],
                shape: vec![3, 2],
            }
        );
        assert_eq!(
            matlab_ctranspose(&[XdlValue::DComplex(num_complex::Complex64::new(2.0, 3.0))])
                .unwrap(),
            XdlValue::DComplex(num_complex::Complex64::new(2.0, -3.0))
        );
    }

    #[test]
    fn matlab_ctranspose_rejects_complex_arrays_instead_of_skipping_conjugation() {
        let complex_array =
            XdlValue::NestedArray(vec![XdlValue::NestedArray(vec![XdlValue::DComplex(
                num_complex::Complex64::new(1.0, 2.0),
            )])]);
        let error = matlab_ctranspose(&[complex_array]).unwrap_err();
        assert!(error.to_string().contains("complex array transpose"));
    }

    #[test]
    fn three_output_find_returns_nonzero_values_in_column_major_order() {
        let matrix = XdlValue::MultiDimArray {
            data: vec![0.0, 2.0, 0.0, 3.0, 0.0, 4.0],
            shape: vec![2, 3],
        };
        assert_eq!(
            matlab_find_nonzero_values(&[matrix]).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![3.0, 2.0, 4.0],
                shape: vec![3, 1],
            }
        );
    }

    #[test]
    fn three_output_find_values_follow_first_and_last_limits_for_nd_arrays() {
        let array = XdlValue::MultiDimArray {
            data: vec![0.0, 1.0, 2.0, 0.0, 3.0, 0.0, 0.0, 4.0],
            shape: vec![2, 2, 2],
        };
        let first = [array.clone(), XdlValue::Long(2)];
        assert_eq!(
            matlab_find_nonzero_values(&first).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![3.0, 2.0],
                shape: vec![2, 1],
            }
        );
        let last = [array, XdlValue::Long(2), XdlValue::String("last".into())];
        assert_eq!(
            matlab_find_nonzero_values(&last).unwrap(),
            XdlValue::MultiDimArray {
                data: vec![1.0, 4.0],
                shape: vec![2, 1],
            }
        );
    }

    #[test]
    fn matlab_readmatrix_reads_rectangular_numeric_csv() {
        let path = temporary_csv("1,2.5\n-3,4\n");
        let result = matlab_readmatrix(&[XdlValue::String(path.clone())]).unwrap();
        fs::remove_file(path).expect("remove CSV fixture");
        assert_eq!(
            result,
            XdlValue::NestedArray(vec![
                XdlValue::Array(vec![1.0, 2.5]),
                XdlValue::Array(vec![-3.0, 4.0]),
            ])
        );
    }

    #[test]
    fn matlab_readmatrix_skips_explicit_header_lines() {
        let path = temporary_csv("time,velocity\nseconds,m/s\n0,1.5\n1,2.5\n");
        let result = matlab_readmatrix(&[
            XdlValue::String(path.clone()),
            XdlValue::String("numheaderlines".into()),
            XdlValue::Long(2),
        ])
        .unwrap();
        fs::remove_file(path).expect("remove CSV fixture");
        assert_eq!(
            result,
            XdlValue::NestedArray(vec![
                XdlValue::Array(vec![0.0, 1.5]),
                XdlValue::Array(vec![1.0, 2.5]),
            ])
        );
    }

    #[test]
    fn matlab_readmatrix_counts_physical_lines_inside_quoted_headers() {
        let path = temporary_csv("\"experiment\ninstrument\",units\n0,1.5\n1,2.5\n");
        let result = matlab_readmatrix(&[
            XdlValue::String(path.clone()),
            XdlValue::String("NumHeaderLines".into()),
            XdlValue::Long(2),
        ])
        .unwrap();
        fs::remove_file(path).expect("remove CSV fixture");
        assert_eq!(
            result,
            XdlValue::NestedArray(vec![
                XdlValue::Array(vec![0.0, 1.5]),
                XdlValue::Array(vec![1.0, 2.5]),
            ])
        );
    }

    #[test]
    fn matlab_readmatrix_rejects_bad_header_options_and_header_only_files() {
        let path = temporary_csv("header\n");
        for args in [
            vec![
                XdlValue::String(path.clone()),
                XdlValue::String("HeaderLines".into()),
                XdlValue::Long(1),
            ],
            vec![
                XdlValue::String(path.clone()),
                XdlValue::String("NumHeaderLines".into()),
                XdlValue::Double(0.5),
            ],
            vec![
                XdlValue::String(path.clone()),
                XdlValue::String("NumHeaderLines".into()),
                XdlValue::Long(1_000_001),
            ],
        ] {
            assert!(matlab_readmatrix(&args).is_err());
        }
        assert!(matlab_readmatrix(&[
            XdlValue::String(path.clone()),
            XdlValue::String("NumHeaderLines".into()),
            XdlValue::Long(1),
        ])
        .is_err());
        fs::remove_file(path).expect("remove CSV fixture");
    }

    #[test]
    fn matlab_readmatrix_rejects_ragged_malformed_and_empty_csv() {
        for contents in ["1,2\n3\n", "1,nope\n", "\n"] {
            let path = temporary_csv(contents);
            let result = matlab_readmatrix(&[XdlValue::String(path.clone())]);
            fs::remove_file(path).expect("remove CSV fixture");
            assert!(result.is_err(), "unexpectedly accepted {contents:?}");
        }
    }

    fn interp_values(result: XdlValue) -> Vec<f64> {
        match result {
            XdlValue::Array(values) => values,
            other => panic!("Expected interpolation array, got {other:?}"),
        }
    }

    #[test]
    fn interp1_linear_interpolates_and_returns_nan_outside_domain() {
        let result = interp1(&[
            XdlValue::Array(vec![0.0, 1.0, 2.0]),
            XdlValue::Array(vec![0.0, 10.0, 20.0]),
            XdlValue::Array(vec![-1.0, 0.0, 0.5, 2.0, 3.0]),
        ])
        .unwrap();
        let values = interp_values(result);
        assert!(values[0].is_nan());
        assert_eq!(&values[1..4], &[0.0, 5.0, 20.0]);
        assert!(values[4].is_nan());
    }

    #[test]
    fn interp1_nearest_uses_nearest_sample_and_handles_descending_samples() {
        let result = interp1(&[
            XdlValue::Array(vec![0.0, 2.0]),
            XdlValue::Array(vec![5.0, 9.0]),
            XdlValue::Array(vec![0.4, 1.0, 1.4]),
            XdlValue::String("nearest".to_string()),
        ])
        .unwrap();
        assert_eq!(interp_values(result), vec![5.0, 9.0, 9.0]);

        let descending = interp1(&[
            XdlValue::Array(vec![2.0, 1.0, 0.0]),
            XdlValue::Array(vec![20.0, 10.0, 0.0]),
            XdlValue::Array(vec![1.5, 0.5]),
        ])
        .unwrap();
        assert_eq!(interp_values(descending), vec![15.0, 5.0]);
    }

    #[test]
    fn interp1_rejects_invalid_sample_axes_and_methods() {
        let duplicate_samples = [
            XdlValue::Array(vec![0.0, 1.0, 1.0]),
            XdlValue::Array(vec![0.0, 10.0, 20.0]),
            XdlValue::Double(0.5),
        ];
        assert!(interp1(&duplicate_samples).is_err());

        let unsupported_method = [
            XdlValue::Array(vec![0.0, 1.0]),
            XdlValue::Array(vec![0.0, 10.0]),
            XdlValue::Double(0.5),
            XdlValue::String("cubic".to_string()),
        ];
        assert!(interp1(&unsupported_method).is_err());
    }

    #[test]
    fn test_linspace() {
        let args = vec![
            XdlValue::Double(0.0),
            XdlValue::Double(10.0),
            XdlValue::Long(11),
        ];
        let result = linspace(&args).unwrap();

        if let XdlValue::Array(arr) = result {
            assert_eq!(arr.len(), 11);
            assert_eq!(arr[0], 0.0);
            assert_eq!(arr[10], 10.0);
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_logspace() {
        let args = vec![
            XdlValue::Double(0.0),
            XdlValue::Double(3.0),
            XdlValue::Long(4),
        ];
        let result = logspace(&args).unwrap();

        if let XdlValue::Array(arr) = result {
            assert_eq!(arr.len(), 4);
            assert!((arr[0] - 1.0).abs() < 1e-10);
            assert!((arr[3] - 1000.0).abs() < 1e-6);
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_meshgrid() {
        let x = XdlValue::Array(vec![1.0, 2.0, 3.0]);
        let y = XdlValue::Array(vec![4.0, 5.0]);

        let result = meshgrid(&[x, y]).unwrap();

        if let XdlValue::NestedArray(arrays) = result {
            assert_eq!(arrays.len(), 2);
        } else {
            panic!("Expected nested array result");
        }
    }
}
