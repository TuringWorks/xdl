// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 Ravindra Boddipalli

//! Deterministic simulation primitives for the XDL GNC vertical slice.
//!
//! This crate intentionally starts with a small constant-acceleration truth model
//! and explicit sensor records. It provides a stable place to add scheduling,
//! disturbances, replay, and Monte Carlo execution without hiding time, frame,
//! unit, or validity semantics inside the estimator or controller.
//!
//! Units are fixed by record type and exposed through [`VectorMeta`]:
//! positions in metres, velocities in m/s, specific force in m/s^2, and
//! angular rates in rad/s.

use serde::{Deserialize, Serialize};
use xdl_gnc_core::{DurationNs, FrameId, GncError, Quality, SignalStatus, Timestamp, Unit, Vec3, VectorMeta};

/// A point-mass state expressed in `frame`. Position is in metres and velocity
/// in m/s.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KinematicState {
    pub timestamp: Timestamp,
    pub frame: FrameId,
    pub position: Vec3,
    pub velocity: Vec3,
}

impl KinematicState {
    pub fn new(
        timestamp: Timestamp,
        frame: FrameId,
        position: Vec3,
        velocity: Vec3,
    ) -> Result<Self, GncError> {
        if !position.is_finite() {
            return Err(GncError::NonFinite {
                context: "kinematic position".to_string(),
            });
        }
        if !velocity.is_finite() {
            return Err(GncError::NonFinite {
                context: "kinematic velocity".to_string(),
            });
        }
        Ok(Self {
            timestamp,
            frame,
            position,
            velocity,
        })
    }

    pub fn position_meta(&self) -> VectorMeta {
        VectorMeta::new(self.frame.clone(), Unit::Meter)
    }

    pub fn velocity_meta(&self) -> VectorMeta {
        VectorMeta::new(self.frame.clone(), Unit::MeterPerSecond)
    }
}

/// Constant-acceleration truth model using an explicit fixed-step update.
/// `acceleration` is in m/s^2 in `frame`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstantAccelerationModel {
    pub frame: FrameId,
    pub acceleration: Vec3,
}

impl ConstantAccelerationModel {
    pub fn new(frame: FrameId, acceleration: Vec3) -> Result<Self, GncError> {
        if !acceleration.is_finite() {
            return Err(GncError::NonFinite {
                context: "constant acceleration".to_string(),
            });
        }
        Ok(Self {
            frame,
            acceleration,
        })
    }

    pub fn step(&self, state: &KinematicState, dt: DurationNs) -> Result<KinematicState, GncError> {
        if state.frame != self.frame {
            return Err(GncError::InvalidValue {
                context: "constant-acceleration step".to_string(),
                reason: format!(
                    "state frame {:?} does not match model frame {:?}",
                    state.frame, self.frame
                ),
            });
        }
        if !dt.is_positive() {
            return Err(GncError::InvalidTime(
                "simulation step must be positive".to_string(),
            ));
        }
        let seconds = dt.as_seconds();
        let next_position = state.position
            + state.velocity * seconds
            + self.acceleration * (0.5 * seconds * seconds);
        let next_velocity = state.velocity + self.acceleration * seconds;
        let next_timestamp = state.timestamp.checked_add(dt)?;
        KinematicState::new(next_timestamp, self.frame.clone(), next_position, next_velocity)
    }
}

/// A deterministic idealized IMU sample. Specific force is in m/s^2 and angular
/// rate in rad/s, both in `frame`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImuMeasurement {
    pub status: SignalStatus,
    pub frame: FrameId,
    pub specific_force: Vec3,
    pub angular_rate: Vec3,
}

impl ImuMeasurement {
    pub fn new(
        timestamp: Timestamp,
        frame: FrameId,
        specific_force: Vec3,
        angular_rate: Vec3,
    ) -> Result<Self, GncError> {
        if !specific_force.is_finite() {
            return Err(GncError::NonFinite {
                context: "IMU specific force".to_string(),
            });
        }
        if !angular_rate.is_finite() {
            return Err(GncError::NonFinite {
                context: "IMU angular rate".to_string(),
            });
        }
        Ok(Self {
            status: SignalStatus::nominal(timestamp),
            frame,
            specific_force,
            angular_rate,
        })
    }

    pub fn specific_force_meta(&self) -> VectorMeta {
        VectorMeta::new(self.frame.clone(), Unit::MeterPerSecondSquared)
    }

    pub fn angular_rate_meta(&self) -> VectorMeta {
        VectorMeta::new(self.frame.clone(), Unit::RadianPerSecond)
    }
}

/// A GNSS position/velocity sample with explicit quality status. Position is in
/// metres and velocity in m/s, both in `frame`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GnssMeasurement {
    pub status: SignalStatus,
    pub frame: FrameId,
    pub position: Vec3,
    pub velocity: Vec3,
}

/// One timestamped replay record. Measurements are optional because real logs
/// commonly contain asynchronous sensor streams and dropouts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayRecord {
    pub timestamp: Timestamp,
    pub imu: Option<ImuMeasurement>,
    pub gnss: Option<GnssMeasurement>,
}

/// Ordered, deterministic sensor replay log. All records in one log share a
/// single frame, fixed by the first record.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayLog {
    frame: Option<FrameId>,
    records: Vec<ReplayRecord>,
}

impl ReplayLog {
    pub fn new() -> Self {
        Self::default()
    }

    /// The frame all records in this log are expressed in, once one is pushed.
    pub fn frame(&self) -> Option<&FrameId> {
        self.frame.as_ref()
    }

    pub fn push(&mut self, record: ReplayRecord) -> Result<(), GncError> {
        if let Some(previous) = self.records.last() {
            if record.timestamp <= previous.timestamp {
                return Err(GncError::InvalidTime(
                    "replay timestamps must be strictly increasing".to_string(),
                ));
            }
        }
        if let Some(imu) = &record.imu {
            if imu.status.timestamp != record.timestamp {
                return Err(GncError::InvalidTime(
                    "IMU timestamp does not match replay timestamp".to_string(),
                ));
            }
        }
        if let Some(gnss) = &record.gnss {
            if gnss.status.timestamp != record.timestamp {
                return Err(GncError::InvalidTime(
                    "GNSS timestamp does not match replay timestamp".to_string(),
                ));
            }
        }
        let record_frames = record
            .imu
            .iter()
            .map(|imu| &imu.frame)
            .chain(record.gnss.iter().map(|gnss| &gnss.frame));
        for frame in record_frames {
            match &self.frame {
                None => self.frame = Some(frame.clone()),
                Some(expected) if expected == frame => {}
                Some(expected) => {
                    return Err(GncError::InvalidValue {
                        context: "replay record".to_string(),
                        reason: format!("frame {frame:?} does not match log frame {expected:?}"),
                    });
                }
            }
        }
        self.records.push(record);
        Ok(())
    }

    pub fn records(&self) -> &[ReplayRecord] {
        &self.records
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

impl GnssMeasurement {
    pub fn new(
        timestamp: Timestamp,
        frame: FrameId,
        position: Vec3,
        velocity: Vec3,
    ) -> Result<Self, GncError> {
        if !position.is_finite() {
            return Err(GncError::NonFinite {
                context: "GNSS position".to_string(),
            });
        }
        if !velocity.is_finite() {
            return Err(GncError::NonFinite {
                context: "GNSS velocity".to_string(),
            });
        }
        Ok(Self {
            status: SignalStatus::nominal(timestamp),
            frame,
            position,
            velocity,
        })
    }

    pub fn position_meta(&self) -> VectorMeta {
        VectorMeta::new(self.frame.clone(), Unit::Meter)
    }

    pub fn velocity_meta(&self) -> VectorMeta {
        VectorMeta::new(self.frame.clone(), Unit::MeterPerSecond)
    }

    pub fn with_quality(mut self, quality: Quality) -> Self {
        self.status.quality = quality;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xdl_gnc_core::Quality;

    fn body() -> FrameId {
        FrameId::Body
    }

    #[test]
    fn constant_acceleration_step_is_deterministic() {
        let model = ConstantAccelerationModel::new(body(), Vec3::new(1.0, 0.0, 0.0))
            .expect("finite acceleration");
        let state = KinematicState::new(
            Timestamp::ZERO,
            body(),
            Vec3::ZERO,
            Vec3::new(10.0, 0.0, 0.0),
        )
        .expect("finite state");
        let next = model
            .step(&state, DurationNs::from_seconds(2.0).expect("finite duration"))
            .expect("positive step");

        assert_eq!(next.timestamp.as_nanoseconds(), 2_000_000_000);
        assert_eq!(next.position, Vec3::new(22.0, 0.0, 0.0));
        assert_eq!(next.velocity, Vec3::new(12.0, 0.0, 0.0));
        assert_eq!(next.frame, body());
    }

    #[test]
    fn simulation_rejects_zero_step() {
        let model = ConstantAccelerationModel::new(body(), Vec3::ZERO).expect("finite acceleration");
        let state = KinematicState::new(Timestamp::ZERO, body(), Vec3::ZERO, Vec3::ZERO)
            .expect("finite state");
        assert!(model.step(&state, DurationNs::ZERO).is_err());
    }

    #[test]
    fn model_rejects_state_in_a_different_frame() {
        let model = ConstantAccelerationModel::new(body(), Vec3::ZERO).expect("finite acceleration");
        let state = KinematicState::new(Timestamp::ZERO, FrameId::Inertial, Vec3::ZERO, Vec3::ZERO)
            .expect("finite state");
        let dt = DurationNs::from_seconds(1.0).expect("finite duration");
        assert!(model.step(&state, dt).is_err());
    }

    #[test]
    fn state_metadata_reports_units_for_frame() {
        let state = KinematicState::new(Timestamp::ZERO, body(), Vec3::ZERO, Vec3::ZERO)
            .expect("finite state");
        assert!(state
            .position_meta()
            .require(&body(), Unit::Meter, "position")
            .is_ok());
        assert!(state
            .velocity_meta()
            .require(&body(), Unit::MeterPerSecond, "velocity")
            .is_ok());
    }

    #[test]
    fn sensor_records_reject_non_finite_values() {
        assert!(ImuMeasurement::new(
            Timestamp::ZERO,
            body(),
            Vec3::new(f64::NAN, 0.0, 0.0),
            Vec3::ZERO,
        )
        .is_err());
        assert!(GnssMeasurement::new(
            Timestamp::ZERO,
            body(),
            Vec3::ZERO,
            Vec3::new(0.0, f64::INFINITY, 0.0),
        )
        .is_err());
    }

    #[test]
    fn imu_metadata_carries_frame_and_rate_units() {
        let imu = ImuMeasurement::new(Timestamp::ZERO, body(), Vec3::ZERO, Vec3::ZERO)
            .expect("finite IMU");
        assert!(imu
            .specific_force_meta()
            .require(&body(), Unit::MeterPerSecondSquared, "specific force")
            .is_ok());
        assert!(imu
            .angular_rate_meta()
            .require(&body(), Unit::RadianPerSecond, "angular rate")
            .is_ok());
    }

    #[test]
    fn degraded_status_is_usable_but_stale_status_is_not() {
        let mut gnss = GnssMeasurement::new(Timestamp::ZERO, body(), Vec3::ZERO, Vec3::ZERO)
            .expect("finite GNSS");
        gnss = gnss.with_quality(Quality::Degraded);
        assert!(gnss.status.is_usable());
        gnss = gnss.with_quality(Quality::Stale);
        assert!(!gnss.status.is_usable());
    }

    #[test]
    fn replay_log_requires_strictly_increasing_timestamps() {
        let mut log = ReplayLog::new();
        let record = ReplayRecord {
            timestamp: Timestamp::from_nanoseconds(1),
            imu: None,
            gnss: None,
        };
        log.push(record.clone()).expect("first record");
        assert!(log.push(record).is_err());
        assert_eq!(log.records().len(), 1);
    }

    #[test]
    fn replay_log_rejects_mixed_frames() {
        let mut log = ReplayLog::new();
        let t0 = Timestamp::from_nanoseconds(1);
        let t1 = Timestamp::from_nanoseconds(2);
        let body_gnss = GnssMeasurement::new(t0, body(), Vec3::ZERO, Vec3::ZERO).expect("GNSS");
        log.push(ReplayRecord {
            timestamp: t0,
            imu: None,
            gnss: Some(body_gnss),
        })
        .expect("first frame fixes log frame");
        assert_eq!(log.frame(), Some(&body()));

        let inertial_gnss =
            GnssMeasurement::new(t1, FrameId::Inertial, Vec3::ZERO, Vec3::ZERO).expect("GNSS");
        let mismatched = log.push(ReplayRecord {
            timestamp: t1,
            imu: None,
            gnss: Some(inertial_gnss),
        });
        assert!(mismatched.is_err());
        assert_eq!(log.records().len(), 1);
    }
}
