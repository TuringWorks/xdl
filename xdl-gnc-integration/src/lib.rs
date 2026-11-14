// SPDX-License-Identifier: MIT
// Copyright (C) 2026 Ravindra Boddipalli

//! End-to-end deterministic GNC scenario used to verify crate boundaries.

use xdl_gnc_actuators::{FirstOrderActuator, FirstOrderActuatorConfig};
use xdl_gnc_control::{PidConfig, PidController};
use xdl_gnc_core::{FrameId, GncError, Quaternion, ScalarLimit, StepEvent, Timestamp, Vec3};
use xdl_gnc_guidance::{WaypointGuidance, WaypointGuidanceConfig};
use xdl_gnc_navigation::{Covariance15, ErrorStateEkf, ErrorStateEkfConfig, StrapdownState};
use xdl_gnc_simulation::{
    ConstantAccelerationModel, GnssMeasurement, ImuMeasurement, KinematicState, MonteCarloConfig,
    MonteCarloResult, ReplayEventLog, ReplayMeasurement, SensorNoise, SensorNoiseConfig,
};
use xdl_gnc_telemetry::{TelemetryLog, TelemetrySample};

/// Result of replaying a typed sensor stream through the error-state EKF.
#[derive(Debug, Clone, PartialEq)]
pub struct ReplaySummary {
    pub imu_events: usize,
    pub gnss_events: usize,
    pub degraded_events: usize,
    pub final_estimate: StrapdownState,
}

/// Replays asynchronous IMU/GNSS events through a navigation estimator.
///
/// The estimator is supplied by the caller so the runner does not hide
/// covariance, gravity, or initial-state policy. Events must be ordered with
/// IMU propagation before a same-timestamp GNSS update, as required by the
/// estimator's timestamp contract. Frame checks are explicit at this boundary.
pub fn run_error_state_replay(
    log: &ReplayEventLog,
    mut ekf: ErrorStateEkf,
    imu_frame: FrameId,
    gnss_frame: FrameId,
) -> Result<ReplaySummary, GncError> {
    let mut imu_events = 0;
    let mut gnss_events = 0;
    let mut degraded_events = 0;
    let mut cursor = log.cursor();

    while let Some(event) = cursor.advance() {
        match event.measurement {
            ReplayMeasurement::Imu(measurement) => {
                if measurement.frame != imu_frame {
                    return Err(GncError::InvalidValue {
                        context: "GNC replay IMU frame".to_string(),
                        reason: format!(
                            "frame {:?} does not match configured frame {:?}",
                            measurement.frame, imu_frame
                        ),
                    });
                }
                ekf.predict(measurement)?;
                imu_events += 1;
            }
            ReplayMeasurement::Gnss(measurement) => {
                if measurement.frame != gnss_frame {
                    return Err(GncError::InvalidValue {
                        context: "GNC replay GNSS frame".to_string(),
                        reason: format!(
                            "frame {:?} does not match configured frame {:?}",
                            measurement.frame, gnss_frame
                        ),
                    });
                }
                let estimate = ekf.update_gnss(measurement)?;
                if estimate.events.contains(&StepEvent::DegradedInput) {
                    degraded_events += 1;
                }
                gnss_events += 1;
            }
        }
    }

    Ok(ReplaySummary {
        imu_events,
        gnss_events,
        degraded_events,
        final_estimate: ekf.nominal(),
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScenarioSummary {
    pub steps: usize,
    pub truth: KinematicState,
    pub estimate: StrapdownState,
    pub final_command: f64,
    pub final_position_error: Vec3,
    pub telemetry: TelemetryLog,
}

pub fn run_deterministic_scenario() -> Result<ScenarioSummary, GncError> {
    run_scenario(None, None)
}

pub fn run_seeded_noisy_scenario(seed: u64) -> Result<ScenarioSummary, GncError> {
    let imu_config = SensorNoiseConfig {
        frame: FrameId::Body,
        imu_specific_force_stddev: Vec3::new(0.01, 0.01, 0.01),
        imu_angular_rate_stddev: Vec3::new(0.001, 0.001, 0.001),
        gnss_position_stddev: Vec3::ZERO,
        gnss_velocity_stddev: Vec3::ZERO,
    };
    let gnss_config = SensorNoiseConfig {
        frame: FrameId::Inertial,
        imu_specific_force_stddev: Vec3::ZERO,
        imu_angular_rate_stddev: Vec3::ZERO,
        gnss_position_stddev: Vec3::new(0.02, 0.02, 0.02),
        gnss_velocity_stddev: Vec3::new(0.01, 0.01, 0.01),
    };
    let mut imu_noise = SensorNoise::new(imu_config, seed)?;
    let mut gnss_noise = SensorNoise::new(gnss_config, seed ^ 0xA5A5_A5A5_A5A5_A5A5)?;
    run_scenario(Some(&mut imu_noise), Some(&mut gnss_noise))
}

pub fn run_seeded_noisy_monte_carlo(
    config: MonteCarloConfig,
) -> Result<MonteCarloResult, GncError> {
    config.run(|seed| Ok(run_seeded_noisy_scenario(seed)?.final_position_error.norm()))
}

fn run_scenario(
    mut imu_noise: Option<&mut SensorNoise>,
    mut gnss_noise: Option<&mut SensorNoise>,
) -> Result<ScenarioSummary, GncError> {
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
        navigation_frame,
        body_frame,
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
    let mut telemetry = TelemetryLog::new();

    for _ in 0..20 {
        truth = truth_model.step(&truth, step)?;
        let timestamp = truth.timestamp;
        let mut imu = ImuMeasurement::new(
            timestamp,
            body_frame,
            Vec3::new(acceleration.x, acceleration.y, acceleration.z + 9.81),
            Vec3::ZERO,
        )?;
        if let Some(noise) = imu_noise.as_deref_mut() {
            imu = noise.apply_imu(&imu)?;
        }
        ekf.predict(imu)?;
        let mut gnss =
            GnssMeasurement::new(timestamp, navigation_frame, truth.position, truth.velocity)?;
        if let Some(noise) = gnss_noise.as_deref_mut() {
            gnss = noise.apply_gnss(&gnss)?;
        }
        let estimate = ekf.update_gnss(gnss)?;
        final_estimate = estimate.nominal;

        let guidance_command = guidance.command(timestamp, final_estimate.position, waypoint)?;
        let pid_output = pid.step(
            timestamp,
            guidance_command.value.desired_velocity.x,
            final_estimate.velocity.x,
        )?;
        let actuator_output = actuator.step(timestamp, pid_output.value.command)?;
        final_command = actuator_output.value;
        let mut sample =
            TelemetrySample::new(timestamp, telemetry.samples().len() as u64, "closed_loop");
        sample.frame = Some(navigation_frame);
        sample.position = Some(final_estimate.position);
        sample.velocity = Some(final_estimate.velocity);
        sample.command = Some(final_command);
        sample.events.extend(guidance_command.events);
        sample.events.extend(pid_output.events);
        sample.events.extend(actuator_output.events);
        telemetry.push(sample)?;
    }

    Ok(ScenarioSummary {
        steps: 20,
        truth,
        estimate: final_estimate,
        final_command,
        final_position_error: final_estimate.position - truth.position,
        telemetry,
    })
}

fn diagonal_covariance(value: f64) -> Covariance15 {
    std::array::from_fn(|row| std::array::from_fn(|column| if row == column { value } else { 0.0 }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use xdl_gnc_core::Quality;

    #[test]
    fn deterministic_scenario_connects_truth_estimator_guidance_control_and_actuator() {
        let first = run_deterministic_scenario().expect("scenario should run");
        let second = run_deterministic_scenario().expect("scenario should repeat");
        assert_eq!(first, second);
        assert_eq!(first.steps, 20);
        assert_eq!(first.telemetry.samples().len(), 20);
        assert!(first
            .telemetry
            .to_json_lines()
            .expect("telemetry JSON")
            .contains("closed_loop"));
        assert!(first.final_position_error.norm() < 0.5);
        assert!(first.final_command.is_finite());
    }

    #[test]
    fn seeded_noisy_scenario_and_monte_carlo_are_reproducible() {
        let first = run_seeded_noisy_scenario(42).expect("noisy scenario");
        let second = run_seeded_noisy_scenario(42).expect("noisy scenario");
        assert_eq!(first, second);
        assert!(first.final_position_error.is_finite());
        let result = run_seeded_noisy_monte_carlo(MonteCarloConfig { runs: 3, seed: 7 })
            .expect("Monte Carlo scenario");
        assert_eq!(result.outcomes.len(), 3);
        assert!(result.mean.is_finite());
    }

    #[test]
    fn replay_runner_processes_multi_rate_events_and_preserves_degraded_status() {
        let navigation_frame = FrameId::Inertial;
        let body_frame = FrameId::Body;
        let nominal = StrapdownState::new(
            Timestamp::ZERO,
            Vec3::ZERO,
            Vec3::ZERO,
            Quaternion::IDENTITY,
            Vec3::ZERO,
            Vec3::ZERO,
        )
        .expect("finite nominal state");
        let config = ErrorStateEkfConfig::new(0.01, 0.01, 0.001, 0.001, 0.25, 0.25)
            .expect("valid EKF configuration");
        let ekf = ErrorStateEkf::new(
            config,
            nominal,
            diagonal_covariance(1.0),
            Vec3::new(0.0, 0.0, -9.81),
            navigation_frame,
            body_frame,
        )
        .expect("valid EKF");
        let mut log = ReplayEventLog::new();

        for index in 1..=5 {
            let timestamp = Timestamp::from_nanoseconds(index * 100_000_000);
            let imu =
                ImuMeasurement::new(timestamp, body_frame, Vec3::new(1.0, 0.0, 9.81), Vec3::ZERO)
                    .expect("finite IMU");
            let gnss = GnssMeasurement::new(timestamp, navigation_frame, Vec3::ZERO, Vec3::ZERO)
                .expect("finite GNSS");
            log.push(ReplayMeasurement::Imu(imu)).expect("ordered IMU");
            log.push(ReplayMeasurement::Gnss(if index == 3 {
                gnss.with_quality(Quality::Degraded)
            } else {
                gnss
            }))
            .expect("ordered GNSS");
        }

        let summary = run_error_state_replay(&log, ekf, body_frame, navigation_frame)
            .expect("replay should run");
        assert_eq!(summary.imu_events, 5);
        assert_eq!(summary.gnss_events, 5);
        assert_eq!(summary.degraded_events, 1);
        assert_eq!(
            summary.final_estimate.timestamp,
            Timestamp::from_nanoseconds(500_000_000)
        );
    }

    #[test]
    fn replay_runner_rejects_an_unconfigured_sensor_frame() {
        let nominal = StrapdownState::new(
            Timestamp::ZERO,
            Vec3::ZERO,
            Vec3::ZERO,
            Quaternion::IDENTITY,
            Vec3::ZERO,
            Vec3::ZERO,
        )
        .expect("finite nominal state");
        let config = ErrorStateEkfConfig::new(0.01, 0.01, 0.001, 0.001, 0.25, 0.25)
            .expect("valid EKF configuration");
        let ekf = ErrorStateEkf::new(
            config,
            nominal,
            diagonal_covariance(1.0),
            Vec3::new(0.0, 0.0, -9.81),
            FrameId::Inertial,
            FrameId::Body,
        )
        .expect("valid EKF");
        let timestamp = Timestamp::from_nanoseconds(100_000_000);
        let imu = ImuMeasurement::new(timestamp, FrameId::Inertial, Vec3::ZERO, Vec3::ZERO)
            .expect("finite IMU");
        let mut log = ReplayEventLog::new();
        log.push(ReplayMeasurement::Imu(imu)).expect("ordered IMU");

        assert!(run_error_state_replay(&log, ekf, FrameId::Body, FrameId::Inertial).is_err());
    }
}
