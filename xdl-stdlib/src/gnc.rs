// SPDX-License-Identifier: MIT
// Copyright (C) 2026 Ravindra Boddipalli

//! XDL-facing adapters for the typed GNC libraries.

use std::collections::HashMap;
use xdl_core::{XdlError, XdlResult, XdlValue};
use xdl_gnc_actuators::{FirstOrderActuator, FirstOrderActuatorConfig};
use xdl_gnc_control::{PidConfig, PidController};
use xdl_gnc_core::{ScalarLimit, StepEvent, Timestamp, Vec3};
use xdl_gnc_guidance::{WaypointGuidance, WaypointGuidanceConfig};
use xdl_gnc_navigation::{AlphaBetaConfig, AlphaBetaFilter};
use xdl_gnc_simulation::{GnssMeasurement, KinematicState};
use xdl_gnc_telemetry::{TelemetryLog, TelemetrySample};

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

fn numeric_array(value: &XdlValue, name: &str) -> XdlResult<Vec<f64>> {
    let values = match value {
        XdlValue::Array(values) => values.clone(),
        XdlValue::MultiDimArray { data, .. } => data.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric array".to_string(),
                actual: format!("{name}: {:?}", value.gdl_type()),
            })
        }
    };
    if values.iter().all(|value| value.is_finite()) {
        Ok(values)
    } else {
        Err(XdlError::InvalidArgument(format!(
            "{name}: values must be finite"
        )))
    }
}

fn dimension(value: &XdlValue, name: &str) -> XdlResult<usize> {
    let value = scalar(value, name)?;
    if value.fract() != 0.0 || value <= 0.0 || value > usize::MAX as f64 {
        return Err(XdlError::InvalidArgument(format!(
            "{name}: expected a positive integer dimension"
        )));
    }
    Ok(value as usize)
}

fn matrix_length(rows: usize, columns: usize, name: &str) -> XdlResult<usize> {
    rows.checked_mul(columns)
        .ok_or_else(|| XdlError::InvalidArgument(format!("{name}: matrix dimensions overflow")))
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

/// `GNC_ACTUATOR_STEP(timestamp_ns, command, time_constant, output_min,
/// output_max, rate_limit, previous_timestamp_ns, previous_output)` performs
/// one explicit-state first-order actuator update.
/// GNC_STATE_SPACE_STEP(A, B, C, D, state, input, states, inputs, outputs)
/// performs one explicit-state discrete state-space update using row-major
/// flattened matrices. It returns OUTPUT from the current state and STATE for
/// the next call.
pub fn state_space_step(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 9 {
        return Err(XdlError::InvalidArgument(format!(
            "GNC_STATE_SPACE_STEP: expected 9 arguments, got {}",
            args.len()
        )));
    }
    let states = dimension(&args[6], "GNC_STATE_SPACE_STEP states")?;
    let inputs = dimension(&args[7], "GNC_STATE_SPACE_STEP inputs")?;
    let outputs = dimension(&args[8], "GNC_STATE_SPACE_STEP outputs")?;
    let a = numeric_array(&args[0], "GNC_STATE_SPACE_STEP A")?;
    let b = numeric_array(&args[1], "GNC_STATE_SPACE_STEP B")?;
    let c = numeric_array(&args[2], "GNC_STATE_SPACE_STEP C")?;
    let d = numeric_array(&args[3], "GNC_STATE_SPACE_STEP D")?;
    let state = numeric_array(&args[4], "GNC_STATE_SPACE_STEP state")?;
    let input = numeric_array(&args[5], "GNC_STATE_SPACE_STEP input")?;
    let expected = [
        (states, states, &a, "A"),
        (states, inputs, &b, "B"),
        (outputs, states, &c, "C"),
        (outputs, inputs, &d, "D"),
    ];
    for (rows, columns, matrix, name) in expected {
        let length = matrix_length(rows, columns, &format!("GNC_STATE_SPACE_STEP {name}"))?;
        if matrix.len() != length {
            return Err(XdlError::InvalidArgument(format!(
                "GNC_STATE_SPACE_STEP {name}: expected {length} elements, got {}",
                matrix.len()
            )));
        }
    }
    if state.len() != states || input.len() != inputs {
        return Err(XdlError::InvalidArgument(
            "GNC_STATE_SPACE_STEP: state or input length does not match dimensions".to_string(),
        ));
    }
    let output = (0..outputs)
        .map(|row| {
            let state_term = (0..states)
                .map(|column| c[row * states + column] * state[column])
                .sum::<f64>();
            let input_term = (0..inputs)
                .map(|column| d[row * inputs + column] * input[column])
                .sum::<f64>();
            state_term + input_term
        })
        .collect::<Vec<_>>();
    let next_state = (0..states)
        .map(|row| {
            let state_term = (0..states)
                .map(|column| a[row * states + column] * state[column])
                .sum::<f64>();
            let input_term = (0..inputs)
                .map(|column| b[row * inputs + column] * input[column])
                .sum::<f64>();
            state_term + input_term
        })
        .collect::<Vec<_>>();
    if output
        .iter()
        .chain(next_state.iter())
        .any(|value| !value.is_finite())
    {
        return Err(XdlError::RuntimeError(
            "GNC_STATE_SPACE_STEP: output must be finite".to_string(),
        ));
    }
    Ok(XdlValue::Struct(HashMap::from([
        ("OUTPUT".to_string(), XdlValue::Array(output)),
        ("STATE".to_string(), XdlValue::Array(next_state)),
    ])))
}

pub fn actuator_step(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 8 {
        return Err(XdlError::InvalidArgument(format!(
            "GNC_ACTUATOR_STEP: expected 8 arguments, got {}",
            args.len()
        )));
    }
    let config = FirstOrderActuatorConfig::new(
        scalar(&args[2], "GNC_ACTUATOR_STEP time constant")?,
        ScalarLimit::new(
            scalar(&args[3], "GNC_ACTUATOR_STEP output minimum")?,
            scalar(&args[4], "GNC_ACTUATOR_STEP output maximum")?,
        )
        .map_err(|error| XdlError::RuntimeError(error.to_string()))?,
        scalar(&args[5], "GNC_ACTUATOR_STEP rate limit")?,
    )
    .map_err(|error| XdlError::RuntimeError(error.to_string()))?;
    let mut actuator = FirstOrderActuator::with_state(
        config,
        timestamp(&args[6])?,
        scalar(&args[7], "GNC_ACTUATOR_STEP previous output")?,
    )
    .map_err(|error| XdlError::RuntimeError(error.to_string()))?;
    let result = actuator
        .step(
            timestamp(&args[0])?,
            scalar(&args[1], "GNC_ACTUATOR_STEP command")?,
        )
        .map_err(|error| XdlError::RuntimeError(error.to_string()))?;
    let mut output = HashMap::new();
    output.insert("OUTPUT".to_string(), XdlValue::Double(result.value));
    output.insert(
        "SATURATED".to_string(),
        XdlValue::Long(i32::from(result.events.contains(&StepEvent::Saturated))),
    );
    output.insert(
        "RATE_LIMITED".to_string(),
        XdlValue::Long(i32::from(result.events.contains(&StepEvent::RateLimited))),
    );
    Ok(XdlValue::Struct(output))
}

/// `GNC_ALPHA_BETA_STEP(timestamp_ns, initial_timestamp_ns, position, velocity,
/// acceleration, gnss_position, gnss_velocity, position_gain, velocity_gain)`
/// runs one explicit-state alpha-beta navigation update. The first six vectors
/// are expressed in the current navigation frame; the binding uses the
/// inertial frame until named XDL frame objects are exposed.
pub fn alpha_beta_step(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 9 {
        return Err(XdlError::InvalidArgument(format!(
            "GNC_ALPHA_BETA_STEP: expected 9 arguments, got {}",
            args.len()
        )));
    }
    let timestamp_value = timestamp(&args[0])?;
    let initial_timestamp = timestamp(&args[1])?;
    let initial_state = KinematicState::new(
        initial_timestamp,
        xdl_gnc_core::FrameId::Inertial,
        vector(&args[2], "GNC_ALPHA_BETA_STEP position")?,
        vector(&args[3], "GNC_ALPHA_BETA_STEP velocity")?,
    )
    .map_err(|error| XdlError::RuntimeError(error.to_string()))?;
    let config = AlphaBetaConfig::new(
        scalar(&args[7], "GNC_ALPHA_BETA_STEP position gain")?,
        scalar(&args[8], "GNC_ALPHA_BETA_STEP velocity gain")?,
    )
    .map_err(|error| XdlError::RuntimeError(error.to_string()))?;
    let gnss = GnssMeasurement::new(
        timestamp_value,
        xdl_gnc_core::FrameId::Inertial,
        vector(&args[5], "GNC_ALPHA_BETA_STEP GNSS position")?,
        vector(&args[6], "GNC_ALPHA_BETA_STEP GNSS velocity")?,
    )
    .map_err(|error| XdlError::RuntimeError(error.to_string()))?;
    let mut filter = AlphaBetaFilter::new(config, initial_state);
    let result = filter
        .step(
            timestamp_value,
            vector(&args[4], "GNC_ALPHA_BETA_STEP acceleration")?,
            Some(gnss),
        )
        .map_err(|error| XdlError::RuntimeError(error.to_string()))?;
    let mut output = HashMap::new();
    output.insert(
        "POSITION".to_string(),
        XdlValue::Array(vec![
            result.state.position.x,
            result.state.position.y,
            result.state.position.z,
        ]),
    );
    output.insert(
        "VELOCITY".to_string(),
        XdlValue::Array(vec![
            result.state.velocity.x,
            result.state.velocity.y,
            result.state.velocity.z,
        ]),
    );
    if let Some(residual) = result.position_residual {
        output.insert(
            "POSITION_RESIDUAL".to_string(),
            XdlValue::Array(vec![residual.x, residual.y, residual.z]),
        );
    }
    if let Some(residual) = result.velocity_residual {
        output.insert(
            "VELOCITY_RESIDUAL".to_string(),
            XdlValue::Array(vec![residual.x, residual.y, residual.z]),
        );
    }
    output.insert(
        "DEGRADED".to_string(),
        XdlValue::Long(i32::from(!result.status.is_usable())),
    );
    Ok(XdlValue::Struct(output))
}

/// `GNC_TELEMETRY_JSON(timestamp_ns, sequence, source, position, velocity,
/// command)` returns one newline-terminated JSON telemetry record.
pub fn telemetry_json(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 6 {
        return Err(XdlError::InvalidArgument(format!(
            "GNC_TELEMETRY_JSON: expected 6 arguments, got {}",
            args.len()
        )));
    }
    let source = match &args[2] {
        XdlValue::String(value) => value.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string source name".to_string(),
                actual: format!("GNC_TELEMETRY_JSON: {:?}", args[2].gdl_type()),
            })
        }
    };
    let sequence = scalar(&args[1], "GNC_TELEMETRY_JSON sequence")?;
    if sequence.fract() != 0.0 || sequence < 0.0 || sequence >= 18_446_744_073_709_551_616.0 {
        return Err(XdlError::InvalidArgument(
            "GNC_TELEMETRY_JSON sequence: expected a non-negative integer in u64 range".to_string(),
        ));
    }
    let mut sample = TelemetrySample::new(timestamp(&args[0])?, sequence as u64, source);
    sample.position = Some(vector(&args[3], "GNC_TELEMETRY_JSON position")?);
    sample.velocity = Some(vector(&args[4], "GNC_TELEMETRY_JSON velocity")?);
    sample.command = Some(scalar(&args[5], "GNC_TELEMETRY_JSON command")?);
    let mut log = TelemetryLog::new();
    log.push(sample)
        .map_err(|error| XdlError::RuntimeError(error.to_string()))?;
    Ok(XdlValue::String(log.to_json_lines().map_err(|error| {
        XdlError::RuntimeError(error.to_string())
    })?))
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

    #[test]
    fn state_space_binding_returns_current_output_and_next_state() {
        let result = state_space_step(&[
            XdlValue::Array(vec![1.0]),
            XdlValue::Array(vec![1.0]),
            XdlValue::Array(vec![1.0]),
            XdlValue::Array(vec![0.0]),
            XdlValue::Array(vec![0.0]),
            XdlValue::Array(vec![2.0]),
            XdlValue::Long(1),
            XdlValue::Long(1),
            XdlValue::Long(1),
        ])
        .expect("valid state-space step");
        let XdlValue::Struct(fields) = result else {
            panic!("expected state-space structure");
        };
        assert_eq!(fields.get("OUTPUT"), Some(&XdlValue::Array(vec![0.0])));
        assert_eq!(fields.get("STATE"), Some(&XdlValue::Array(vec![2.0])));
    }

    #[test]
    fn actuator_binding_reports_rate_limiting() {
        let result = actuator_step(&[
            XdlValue::Long64(1_000_000_000),
            XdlValue::Double(1.0),
            XdlValue::Double(1.0),
            XdlValue::Double(-1.0),
            XdlValue::Double(1.0),
            XdlValue::Double(0.25),
            XdlValue::Long64(0),
            XdlValue::Double(0.0),
        ])
        .expect("valid actuator step");
        let XdlValue::Struct(fields) = result else {
            panic!("expected actuator structure");
        };
        assert_eq!(fields.get("RATE_LIMITED"), Some(&XdlValue::Long(1)));
    }

    #[test]
    fn alpha_beta_binding_returns_updated_state_and_residuals() {
        let result = alpha_beta_step(&[
            XdlValue::Long(1),
            XdlValue::Long(0),
            XdlValue::Array(vec![0.0, 0.0, 0.0]),
            XdlValue::Array(vec![0.0, 0.0, 0.0]),
            XdlValue::Array(vec![1.0, 0.0, 0.0]),
            XdlValue::Array(vec![1.0, 0.0, 0.0]),
            XdlValue::Array(vec![1.0, 0.0, 0.0]),
            XdlValue::Double(0.5),
            XdlValue::Double(0.5),
        ])
        .expect("valid navigation step");
        let XdlValue::Struct(fields) = result else {
            panic!("expected navigation structure");
        };
        assert!(fields.contains_key("POSITION"));
        assert!(fields.contains_key("POSITION_RESIDUAL"));
    }

    #[test]
    fn telemetry_binding_returns_json_lines() {
        let value = telemetry_json(&[
            XdlValue::Long(1),
            XdlValue::Long(0),
            XdlValue::String("control".to_string()),
            XdlValue::Array(vec![1.0, 2.0, 3.0]),
            XdlValue::Array(vec![4.0, 5.0, 6.0]),
            XdlValue::Double(0.75),
        ])
        .expect("valid telemetry sample");
        let XdlValue::String(json) = value else {
            panic!("expected JSON string");
        };
        assert!(json.contains("\"source\":\"control\""));
        assert!(json.ends_with('\n'));
    }
}
