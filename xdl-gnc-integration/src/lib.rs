// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 Ravindra Boddipalli

//! End-to-end deterministic GNC scenario used to verify crate boundaries.

use xdl_gnc_actuators::{FirstOrderActuator, FirstOrderActuatorConfig};
use xdl_gnc_control::{PidConfig, PidController};
use xdl_gnc_core::{FrameId, GncError, Quaternion, ScalarLimit, Timestamp, Vec3};
use xdl_gnc_guidance::{WaypointGuidance, WaypointGuidanceConfig};
use xdl_gnc_navigation::{Covariance15, ErrorStateEkf, ErrorStateEkfConfig, StrapdownState};
use xdl_gnc_simulation::{
    ConstantAccelerationModel, GnssMeasurement, ImuMeasurement, KinematicState,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ScenarioSummary {
    pub steps: usize,
    pub truth: KinematicState,
    pub estimate: StrapdownState,
    pub final_command: f64,
    pub final_position_error: Vec3,
}

pub fn run_deterministic_scenario() -> Result<ScenarioSummary, GncError> {
    let navigation_frame = FrameId::Inertial;
    let body_frame = FrameId::Body;
    let step = xdl_gnc_core::DurationNs::from_seconds(0.1)?;
    let acceleration = Vec3::new(1.0, 0.0, 0.0);
    let truth_model = ConstantAccelerationModel::new(navigation_frame, acceleration)?;
    let mut truth = KinematicState::new(Timestamp::ZERO, navigation_frame, Vec3::ZERO, Vec3::ZERO)?;

    let nominal = StrapdownState::new(
        Timestamp::ZERO,
        Vec3::ZERO,
        Vec3::ZERO,
        Quaternion::IDENTITY,
        Vec3::ZERO,
        Vec3::ZERO,
    )?;
    let ekf_config = ErrorStateEkfConfig::new(0.01, 0.01, 0.001, 0.001, 0.25, 0.25)?;
    let mut ekf = ErrorStateEkf::new(
        ekf_config,
        nominal,
        diagonal_covariance(1.0),
        Vec3::new(0.0, 0.0, -9.81),
    )?;

    let guidance = WaypointGuidance::new(WaypointGuidanceConfig::new(2.0, 0.2)?);
    let pid_config = PidConfig::new(
        1.0,
        0.2,
        0.05,
        ScalarLimit::new(-5.0, 5.0)?,
        ScalarLimit::new(-10.0, 10.0)?,
    )?;
    let mut pid = PidController::new(pid_config);
    let actuator_config = FirstOrderActuatorConfig::new(0.2, ScalarLimit::new(-5.0, 5.0)?, 20.0)?;
    let mut actuator = FirstOrderActuator::new(actuator_config, Timestamp::ZERO, 0.0)?;
    let waypoint = Vec3::new(10.0, 0.0, 0.0);
    let mut final_command = 0.0;
    let mut final_estimate = nominal;

    for _ in 0..20 {
        truth = truth_model.step(&truth, step)?;
        let timestamp = truth.timestamp;
        let imu = ImuMeasurement::new(
            timestamp,
            body_frame,
            Vec3::new(acceleration.x, acceleration.y, acceleration.z + 9.81),
            Vec3::ZERO,
        )?;
        ekf.predict(imu)?;
        let gnss =
            GnssMeasurement::new(timestamp, navigation_frame, truth.position, truth.velocity)?;
        let estimate = ekf.update_gnss(gnss)?;
        final_estimate = estimate.nominal;

        let guidance_command = guidance.command(timestamp, final_estimate.position, waypoint)?;
        let pid_output = pid.step(
            timestamp,
            guidance_command.value.desired_velocity.x,
            final_estimate.velocity.x,
        )?;
        final_command = actuator.step(timestamp, pid_output.value.command)?.value;
    }

    Ok(ScenarioSummary {
        steps: 20,
        truth,
        estimate: final_estimate,
        final_command,
        final_position_error: final_estimate.position - truth.position,
    })
}

fn diagonal_covariance(value: f64) -> Covariance15 {
    let mut covariance = [[0.0; 15]; 15];
    for index in 0..15 {
        covariance[index][index] = value;
    }
    covariance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_scenario_connects_truth_estimator_guidance_control_and_actuator() {
        let first = run_deterministic_scenario().expect("scenario should run");
        let second = run_deterministic_scenario().expect("scenario should repeat");
        assert_eq!(first, second);
        assert_eq!(first.steps, 20);
        assert!(first.final_position_error.norm() < 0.5);
        assert!(first.final_command.is_finite());
    }
}
