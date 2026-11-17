//! MATLAB Compatibility Functions
//!
//! This module provides MATLAB-style functions that are commonly used but not
//! available in standard IDL/GDL. These extend XDL's capabilities for users
//! coming from MATLAB.

use crate::XdlResult;
use std::borrow::Cow;
use xdl_core::{XdlError, XdlValue};

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

/// MATLAB `find` with the default all-dimensions, ascending-index behavior.
/// Results are one-based; row-vector inputs preserve row orientation while
/// other inputs return a column vector.
pub fn matlab_find(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "MATLAB_FIND: Expected 1 argument, got {}",
            args.len()
        )));
    }

    let dimensions = matlab_dimensions(&args[0])?;
    let values = matlab_numeric_values(&args[0])?;
    let indices: Vec<f64> = values
        .iter()
        .enumerate()
        .filter_map(|(index, value)| (*value != 0.0).then_some((index + 1) as f64))
        .collect();
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
    use super::{matlab_find, matlab_index, matlab_length, matlab_numel, matlab_size};
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
