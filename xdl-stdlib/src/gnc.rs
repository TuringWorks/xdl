// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 Ravindra Boddipalli

//! XDL-facing adapters for the typed GNC libraries.

use std::collections::HashMap;
use xdl_core::{XdlError, XdlResult, XdlValue};
use xdl_gnc_control::{PidConfig, PidController};
use xdl_gnc_core::{ScalarLimit, StepEvent, Timestamp, Vec3};
use xdl_gnc_guidance::{WaypointGuidance, WaypointGuidanceConfig};

fn scalar(value: &XdlValue, name: &str) -> XdlResult<f64> {
    let result = match value {
        XdlValue::Byte(value) => f64::from(*value),
        XdlValue::Int(value) => f64::from(*value),
        XdlValue::Long(value) => f64::from(*value),
        XdlValue::Float(value) => f64::from(*value),
        XdlValue::Double(value) => *value,
        XdlValue::UInt(value) => f64::from(*value),
        XdlValue::ULong(value) => f64::from(*value),
        XdlValue::Long64(value) => *value as f64,
        XdlValue::ULong64(value) => *value as f64,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric scalar".to_string(),
                actual: format!("{name}: {:?}", value.gdl_type()),
            })
        }
    };
    if result.is_finite() {
        Ok(result)
    } else {
        Err(XdlError::InvalidArgument(format!(
            "{name}: value must be finite"
        )))
    }
}

fn vector(value: &XdlValue, name: &str) -> XdlResult<Vec3> {
    let values = match value {
        XdlValue::Array(values) => values,
        XdlValue::MultiDimArray { data, shape }
            if data.len() == 3
                && shape
                    .iter()
                    .try_fold(1usize, |product, dimension| product.checked_mul(*dimension))
                    == Some(3) =>
        {
            data
        }
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "three-element numeric array".to_string(),
                actual: format!("{name}: {:?}", value.gdl_type()),
            })
        }
    };
    if values.len() != 3 || !values.iter().all(|value| value.is_finite()) {
        return Err(XdlError::InvalidArgument(format!(
            "{name}: expected three finite elements"
        )));
    }
    Ok(Vec3::new(values[0], values[1], values[2]))
}

fn timestamp(value: &XdlValue) -> XdlResult<Timestamp> {
    let value = scalar(value, "GNC_WAYPOINT timestamp")?;
    if value.fract() != 0.0 || value < i64::MIN as f64 || value >= 9_223_372_036_854_775_808.0 {
        return Err(XdlError::InvalidArgument(
            "GNC_WAYPOINT timestamp: expected an integer nanosecond value in i64 range".to_string(),
        ));
    }
    Ok(Timestamp::from_nanoseconds(value as i64))
}

fn optional_scalar(value: &XdlValue, name: &str) -> XdlResult<Option<f64>> {
    if matches!(value, XdlValue::Undefined) {
        Ok(None)
    } else {
        scalar(value, name).map(Some)
    }
}

/// `GNC_VEC3(x, y, z)` returns a three-element numeric XDL array.
pub fn vec3(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 3 {
        return Err(XdlError::InvalidArgument(format!(
            "GNC_VEC3: expected 3 arguments, got {}",
            args.len()
        )));
    }
    Ok(XdlValue::Array(
        args.iter()
            .map(|value| scalar(value, "GNC_VEC3"))
            .collect::<XdlResult<Vec<_>>>()?,
    ))
}

/// `GNC_WAYPOINT(position, waypoint, timestamp_ns, cruise_speed, radius)`
/// returns a structure containing the desired velocity and guidance status.
pub fn waypoint(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 5 {
        return Err(XdlError::InvalidArgument(format!(
            "GNC_WAYPOINT: expected 5 arguments, got {}",
            args.len()
        )));
    }
    let position = vector(&args[0], "GNC_WAYPOINT position")?;
    let target = vector(&args[1], "GNC_WAYPOINT waypoint")?;
    let timestamp = timestamp(&args[2])?;
    let cruise_speed = scalar(&args[3], "GNC_WAYPOINT cruise speed")?;
    let acceptance_radius = scalar(&args[4], "GNC_WAYPOINT acceptance radius")?;
    let config = WaypointGuidanceConfig::new(cruise_speed, acceptance_radius)
        .map_err(|error| XdlError::RuntimeError(error.to_string()))?;
    let result = WaypointGuidance::new(config)
        .command(timestamp, position, target)
        .map_err(|error| XdlError::RuntimeError(error.to_string()))?;

    let mut output = HashMap::new();
    output.insert(
        "DESIRED_VELOCITY".to_string(),
        XdlValue::Array(vec![
            result.value.desired_velocity.x,
            result.value.desired_velocity.y,
            result.value.desired_velocity.z,
        ]),
    );
    output.insert(
        "DISTANCE".to_string(),
        XdlValue::Double(result.value.distance_to_waypoint),
    );
    output.insert(
        "REACHED".to_string(),
        XdlValue::Long(i32::from(result.value.reached)),
    );
    output.insert(
        "DEGRADED".to_string(),
        XdlValue::Long(i32::from(!result.status.is_usable())),
    );
    Ok(XdlValue::Struct(output))
}

/// `GNC_PID_STEP(timestamp_ns, setpoint, measurement, kp, ki, kd, output_min,
/// output_max, integral_min, integral_max, integral, previous_error,
/// previous_timestamp_ns)` performs one explicit-state PID update. Use
/// `UNDEFINED` for both previous values on the first step, then feed the
/// returned `INTEGRAL`, `ERROR`, and timestamp back into the next call.
pub fn pid_step(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 13 {
        return Err(XdlError::InvalidArgument(format!(
            "GNC_PID_STEP: expected 13 arguments, got {}",
            args.len()
        )));
    }
    let config = PidConfig::new(
        scalar(&args[3], "GNC_PID_STEP kp")?,
        scalar(&args[4], "GNC_PID_STEP ki")?,
        scalar(&args[5], "GNC_PID_STEP kd")?,
        ScalarLimit::new(
            scalar(&args[6], "GNC_PID_STEP output minimum")?,
            scalar(&args[7], "GNC_PID_STEP output maximum")?,
        )
        .map_err(|error| XdlError::RuntimeError(error.to_string()))?,
        ScalarLimit::new(
            scalar(&args[8], "GNC_PID_STEP integral minimum")?,
            scalar(&args[9], "GNC_PID_STEP integral maximum")?,
        )
        .map_err(|error| XdlError::RuntimeError(error.to_string()))?,
    )
    .map_err(|error| XdlError::RuntimeError(error.to_string()))?;
    let previous_error = optional_scalar(&args[11], "GNC_PID_STEP previous error")?;
    let previous_timestamp = if matches!(&args[12], XdlValue::Undefined) {
        None
    } else {
        Some(timestamp(&args[12])?)
    };
    let mut controller = PidController::with_state(
        config,
        scalar(&args[10], "GNC_PID_STEP integral")?,
        previous_error,
        previous_timestamp,
    )
    .map_err(|error| XdlError::RuntimeError(error.to_string()))?;
    let result = controller
        .step(
            timestamp(&args[0])?,
            scalar(&args[1], "GNC_PID_STEP setpoint")?,
            scalar(&args[2], "GNC_PID_STEP measurement")?,
        )
        .map_err(|error| XdlError::RuntimeError(error.to_string()))?;
    let mut output = HashMap::new();
    output.insert(
        "COMMAND".to_string(),
        XdlValue::Double(result.value.command),
    );
    output.insert("ERROR".to_string(), XdlValue::Double(result.value.error));
    output.insert(
        "DERIVATIVE".to_string(),
        XdlValue::Double(result.value.derivative),
    );
    output.insert(
        "INTEGRAL".to_string(),
        XdlValue::Double(result.value.integral),
    );
    output.insert(
        "SATURATED".to_string(),
        XdlValue::Long(i32::from(result.events.contains(&StepEvent::Saturated))),
    );
    Ok(XdlValue::Struct(output))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec3_binding_returns_an_xdl_array() {
        assert_eq!(
            vec3(&[
                XdlValue::Double(1.0),
                XdlValue::Long(2),
                XdlValue::Float(3.0)
            ])
            .expect("valid vector"),
            XdlValue::Array(vec![1.0, 2.0, 3.0])
        );
    }

    #[test]
    fn waypoint_binding_returns_guidance_structure() {
        let result = waypoint(&[
            XdlValue::Array(vec![0.0, 0.0, 0.0]),
            XdlValue::Array(vec![3.0, 4.0, 0.0]),
            XdlValue::Long64(0),
            XdlValue::Double(5.0),
            XdlValue::Double(0.5),
        ])
        .expect("valid guidance request");
        let XdlValue::Struct(fields) = result else {
            panic!("expected guidance structure");
        };
        assert_eq!(
            fields.get("DESIRED_VELOCITY"),
            Some(&XdlValue::Array(vec![3.0, 4.0, 0.0]))
        );
        assert_eq!(fields.get("REACHED"), Some(&XdlValue::Long(0)));
    }

    #[test]
    fn pid_binding_round_trips_explicit_state() {
        let args = vec![
            XdlValue::Long64(0),
            XdlValue::Double(2.0),
            XdlValue::Double(1.0),
            XdlValue::Double(1.0),
            XdlValue::Double(0.0),
            XdlValue::Double(0.0),
            XdlValue::Double(-10.0),
            XdlValue::Double(10.0),
            XdlValue::Double(-10.0),
            XdlValue::Double(10.0),
            XdlValue::Double(0.0),
            XdlValue::Undefined,
            XdlValue::Undefined,
        ];
        let XdlValue::Struct(fields) = pid_step(&args).expect("valid PID step") else {
            panic!("expected PID structure");
        };
        assert_eq!(fields.get("COMMAND"), Some(&XdlValue::Double(1.0)));
        assert_eq!(fields.get("ERROR"), Some(&XdlValue::Double(1.0)));
    }
}
