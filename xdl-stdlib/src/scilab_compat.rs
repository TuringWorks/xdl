//! Small runtime helpers for the currently supported Scilab source subset.

use xdl_core::{XdlError, XdlResult, XdlValue};

/// Validate a one-based Scilab scalar index and convert it to XDL's zero-based
/// array-index value. Keeping this check at runtime prevents zero, negative,
/// fractional, non-finite, and unrepresentable indices from becoming plausible
/// but incorrect array accesses.
pub fn scilab_index(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(format!(
            "SCILAB_INDEX: Expected 1 argument, got {}",
            args.len()
        )));
    }
    let index = positive_integer_index(&args[0], "SCILAB_INDEX")?;
    Ok(XdlValue::Long((index - 1) as i32))
}

/// Read an inclusive Scilab one-based range from a numeric vector.
/// Arguments are `(vector, start, end)` or `(vector, start, step, end)`.
pub fn scilab_index_range(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if !matches!(args.len(), 3 | 4) {
        return Err(XdlError::InvalidArgument(format!(
            "SCILAB_INDEX_RANGE: Expected 3 or 4 arguments, got {}",
            args.len()
        )));
    }
    let values = match &args[0] {
        XdlValue::Array(values) => values,
        XdlValue::MultiDimArray { data, shape } if shape.as_slice() == [data.len()] => data,
        other => {
            return Err(XdlError::TypeMismatch {
                expected: "one-dimensional numeric vector".to_string(),
                actual: format!("{:?}", other.gdl_type()),
            });
        }
    };
    let (start_arg, step_arg, end_arg) = match args {
        [_, start, end] => (start, None, end),
        [_, start, step, end] => (start, Some(step), end),
        _ => {
            return Err(XdlError::InvalidArgument(
                "SCILAB_INDEX_RANGE: Expected 3 or 4 arguments".to_string(),
            ));
        }
    };
    let start = positive_integer_index(start_arg, "SCILAB_INDEX_RANGE")?;
    let end = positive_integer_index(end_arg, "SCILAB_INDEX_RANGE")?;
    let step = step_arg
        .map(|value| nonzero_integer_step(value, "SCILAB_INDEX_RANGE"))
        .transpose()?
        .unwrap_or(1);
    if start > values.len() || end > values.len() {
        return Err(XdlError::InvalidArgument(format!(
            "SCILAB_INDEX_RANGE: range bounds {start}:{end} exceed vector length {}",
            values.len()
        )));
    }
    let selected = if step > 0 {
        if start > end {
            Vec::new()
        } else {
            (start - 1..end)
                .step_by(step as usize)
                .map(|index| values[index])
                .collect()
        }
    } else if start < end {
        Vec::new()
    } else {
        (end - 1..=start - 1)
            .rev()
            .step_by(step.unsigned_abs() as usize)
            .map(|index| values[index])
            .collect()
    };
    Ok(XdlValue::Array(selected))
}

fn nonzero_integer_step(value: &XdlValue, helper: &str) -> XdlResult<i32> {
    let step = value.to_double()?;
    if !step.is_finite() || step.fract() != 0.0 || step == 0.0 || step < i32::MIN as f64 || step > i32::MAX as f64 {
        return Err(XdlError::InvalidArgument(format!(
            "{helper}: expected a nonzero integer step representable by XDL, got {step}"
        )));
    }
    Ok(step as i32)
}

fn positive_integer_index(value: &XdlValue, helper: &str) -> XdlResult<usize> {
    let index = value.to_double()?;
    if !index.is_finite() || index < 1.0 || index.fract() != 0.0 || index > i32::MAX as f64 + 1.0 {
        return Err(XdlError::InvalidArgument(format!(
            "{helper}: expected a positive integer index representable by XDL, got {index}"
        )));
    }
    Ok(index as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scilab_index_converts_positive_one_based_indices() {
        assert_eq!(
            scilab_index(&[XdlValue::Double(1.0)]).unwrap(),
            XdlValue::Long(0)
        );
        assert_eq!(
            scilab_index(&[XdlValue::Long(9)]).unwrap(),
            XdlValue::Long(8)
        );
        assert_eq!(
            scilab_index(&[XdlValue::Double(i32::MAX as f64 + 1.0)]).unwrap(),
            XdlValue::Long(i32::MAX)
        );
    }

    #[test]
    fn scilab_index_rejects_invalid_indices() {
        for value in [
            0.0,
            -1.0,
            1.5,
            f64::NAN,
            f64::INFINITY,
            i32::MAX as f64 + 2.0,
        ] {
            assert!(scilab_index(&[XdlValue::Double(value)]).is_err(), "{value}");
        }
        assert!(scilab_index(&[]).is_err());
        assert!(scilab_index(&[XdlValue::String("2".to_string())]).is_err());
    }

    #[test]
    fn scilab_index_range_uses_inclusive_bounds_steps_and_empty_ranges() {
        let values = XdlValue::Array(vec![10.0, 20.0, 30.0, 40.0, 50.0]);
        assert_eq!(
            scilab_index_range(&[values.clone(), XdlValue::Long(2), XdlValue::Long(4),]).unwrap(),
            XdlValue::Array(vec![20.0, 30.0, 40.0])
        );
        assert_eq!(
            scilab_index_range(&[
                values.clone(),
                XdlValue::Long(1),
                XdlValue::Long(2),
                XdlValue::Long(5),
            ])
            .unwrap(),
            XdlValue::Array(vec![10.0, 30.0, 50.0])
        );
        assert_eq!(
            scilab_index_range(&[values, XdlValue::Long(4), XdlValue::Long(2)]).unwrap(),
            XdlValue::Array(Vec::new())
        );
    }

    #[test]
    fn scilab_index_range_supports_descending_negative_steps() {
        let values = XdlValue::Array(vec![10.0, 20.0, 30.0, 40.0, 50.0]);
        assert_eq!(
            scilab_index_range(&[
                values.clone(),
                XdlValue::Long(5),
                XdlValue::Long(-2),
                XdlValue::Long(1),
            ])
            .unwrap(),
            XdlValue::Array(vec![50.0, 30.0, 10.0])
        );
        assert_eq!(
            scilab_index_range(&[
                values,
                XdlValue::Long(1),
                XdlValue::Long(-1),
                XdlValue::Long(5),
            ])
            .unwrap(),
            XdlValue::Array(Vec::new())
        );
    }

    #[test]
    fn scilab_index_range_rejects_invalid_bounds_and_steps() {
        let values = XdlValue::Array(vec![1.0, 2.0, 3.0]);
        for args in [
            vec![values.clone(), XdlValue::Long(0), XdlValue::Long(2)],
            vec![values.clone(), XdlValue::Long(1), XdlValue::Long(4)],
            vec![
                values.clone(),
                XdlValue::Long(1),
                XdlValue::Long(0),
                XdlValue::Long(2),
            ],
            vec![values.clone(), XdlValue::Double(1.5), XdlValue::Long(2)],
        ] {
            assert!(scilab_index_range(&args).is_err());
        }
    }
}
